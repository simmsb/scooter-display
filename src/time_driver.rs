use core::{
    cell::{Cell, RefCell},
    sync::atomic::{AtomicU32, Ordering, compiler_fence},
};

use at32f4xx_hal::{
    crm::{BusTimerClock as _, Clocks, Enable as _, Reset},
    interrupt,
    pac::{NVIC, TMR1},
};
use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_time_driver::{Driver, TICK_HZ};
use embassy_time_queue_utils::Queue;

pub(crate) struct AlarmState {
    pub(crate) timestamp: Cell<u64>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    pub(crate) const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

type T = TMR1;
const OVF_INTR: interrupt = at32f4xx_hal::interrupt::TMR1_OVF_TMR10;
const CC_INTR: interrupt = at32f4xx_hal::interrupt::TMR1_CH;

struct TmrDriver {
    period: AtomicU32,
    alarm: Mutex<CriticalSectionRawMutex, AlarmState>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TmrDriver = TmrDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

fn calc_now(period: u32, counter: u16) -> u64 {
    ((period as u64) << 15) + ((counter as u32 ^ ((period & 1) << 15)) as u64)
}

#[interrupt]
fn TMR1_OVF_TMR10() {
    DRIVER.on_interrupt();
}

#[interrupt]
fn TMR1_CH() {
    DRIVER.on_interrupt();
}

// stm32 -> at32 naming:
//
// cen   -> tmren
// ctrlN -> crN
// cnt   -> cval
// psc   -> div
// arr   -> pr
// egr   -> swevt
// urs   -> ovfs
// ug    -> ovfswtr
// ccrN  -> cNdt
// dier  -> iden
// uie   -> ovfien
// uif   -> ovfif
// ccNie -> cNien
// sr    -> ists
// ccNif -> cNif

pub fn init(cs: CriticalSection, clocks: &Clocks) {
    DRIVER.init(cs, clocks);
}

impl TmrDriver {
    fn init(&'static self, _cs: CriticalSection, clocks: &Clocks) {
        let r = unsafe { T::steal() };

        unsafe {
            T::enable_unchecked();
            T::reset_unchecked();
        }

        r.ctrl1().modify(|r, w| w.tmren().disable());
        r.cval().write(|w| w.set(0));

        let clk = T::timer_clock(clocks);
        assert!(
            clk.raw() % TICK_HZ as u32 == 0,
            "Undivisible clock: {} % {} != 0",
            clk.raw(),
            TICK_HZ as u32
        );
        let psc = clk.raw() / TICK_HZ as u32 - 1;
        let psc = match psc.try_into() {
            Ok(n) => n,
            Err(_) => panic!("psc division overflow {}", psc),
        };

        r.div().write(|w| w.set(psc));
        r.pr().write(|w| w.set(u16::MAX));

        r.ctrl1().modify(|_, w| w.ovfs().set_bit());
        r.swevt().write(|w| w.ovfswtr().set_bit());
        r.ctrl1().modify(|_, w| w.ovfs().clear_bit());

        r.c1dt().write(|w| w.set(0x8000));

        r.iden().write(|w| w.ovfien().set_bit().c1ien().set_bit());

        NVIC::unpend(OVF_INTR);
        NVIC::unpend(CC_INTR);
        unsafe {
            NVIC::unmask(OVF_INTR);
            NVIC::unmask(CC_INTR);
        }

        r.ctrl1().modify(|_, w| w.tmren().enable());
    }

    fn on_interrupt(&self) {
        let r = unsafe { T::steal() };

        critical_section::with(|cs| {
            let ists = r.ists().read();
            let iden = r.iden().read();

            r.ists().write(|w| unsafe { w.bits(!ists.bits()) });

            if ists.ovfif().is_overflow() {
                self.next_period();
            }

            if ists.c1if().is_capture_compare() {
                self.next_period();
            }

            if ists.c2if().is_capture_compare() && iden.c2ien().is_enabled() {
                self.trigger_alarm(cs);
            }
        })
    }

    fn next_period(&self) {
        let r = unsafe { T::steal() };

        // We only modify the period from the timer interrupt, so we know this can't race.
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);
        let t = (period as u64) << 15;

        critical_section::with(move |cs| {
            r.iden().modify(|_r, w| {
                let alarm = self.alarm.borrow(cs);
                let at = alarm.timestamp.get();

                if at < t + 0xc000 {
                    // just enable it. `set_alarm` has already set the correct CCR val.
                    w.c2ien().set_bit();
                }

                w
            });
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self
            .queue
            .borrow(cs)
            .borrow_mut()
            .next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self
                .queue
                .borrow(cs)
                .borrow_mut()
                .next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let r = unsafe { T::steal() };

        self.alarm.borrow(cs).timestamp.set(timestamp);

        let t = self.now();
        if timestamp <= t {
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.
            r.iden().modify(|_, w| w.c2ien().disable());

            self.alarm.borrow(cs).timestamp.set(u64::MAX);

            return false;
        }

        defmt::trace!("Setting c2d2 to {} with now of {}", timestamp, t);

        // Write the CCR value regardless of whether we're going to enable it now or not.
        // This way, when we enable it later, the right value is already set.
        r.c2dt().write(|w| w.set(timestamp as u16));

        // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
        let diff = timestamp - t;

        defmt::trace!("diff: {}, diff < 0xc000: {}", diff, diff < 0xc000);

        r.iden().modify(|_, w| w.c2ien().bit(diff < 0xc000));

        // Reevaluate if the alarm timestamp is still in the future
        let t = self.now();

        defmt::trace!("ok, now: {}", t);
        if timestamp <= t {
            // If alarm timestamp has passed since we set it, we have a race condition and
            // the alarm may or may not have fired.
            // Disarm the alarm and return `false` to indicate that.
            // It is the caller's responsibility to handle this ambiguity.
            r.iden().modify(|_r, w| w.c2ien().disable());

            self.alarm.borrow(cs).timestamp.set(u64::MAX);

            return false;
        }

        defmt::trace!(
            "NVIC enabled: ovf:{}, cc:{}",
            NVIC::is_enabled(OVF_INTR),
            NVIC::is_enabled(CC_INTR)
        );
        defmt::trace!(
            "NVIC pending: ovf:{}, cc:{}",
            NVIC::is_pending(OVF_INTR),
            NVIC::is_pending(CC_INTR)
        );
        defmt::trace!(
            "NVIC prio: ovf:{}, cc:{}",
            NVIC::get_priority(OVF_INTR),
            NVIC::get_priority(CC_INTR)
        );

        defmt::trace!("ISTS: {}", r.ists().read().bits());

        // We're confident the alarm will ring in the future.
        true
    }
}

impl Driver for TmrDriver {
    fn now(&self) -> u64 {
        let r = unsafe { T::steal() };

        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = r.cval().read().bits() as u16;
        calc_now(period, counter)
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();

            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        })
    }
}
