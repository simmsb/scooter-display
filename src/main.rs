#![no_std]
#![no_main]

// extern crate alloc;

use core::num::{NonZeroU8, NonZeroU16};

use defmt::info;
use embassy_executor::Spawner;
use panic_probe as _;

use at32f4xx_hal::{
    self as hal,
    can::util::NominalBitTiming,
    crm::{Clocks, Enable, Reset},
    gpio::{GpioBusExt as _, OutputPin, PinSpeed as _, Speed},
    pac::{GPIOA, Peripherals},
    prelude::*,
    serial::Serial2,
    signature::IDCode,
    timer::{Channel1, Timer},
    uart::{
        self,
        config::{DmaConfig, Parity, StopBits, WordLength},
    },
};
use cortex_m_rt::entry;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use defmt_rtt as _;
use static_cell::StaticCell;

// mod allocator;
pub mod bluetooth;
mod can;
mod display;
mod time_driver;
mod ui;

#[embassy_executor::task]
async fn async_main(spawner: Spawner, dp: Peripherals, cp: cortex_m::Peripherals, clocks: Clocks) {
    async_main_(spawner, dp, cp, clocks).await;
}

async fn async_main_(
    spawner: Spawner,
    dp: Peripherals,
    mut cp: cortex_m::Peripherals,
    clocks: Clocks,
) {
    info!("Yep");

    // IOMUX clocks start off and hal doesn't know to enable them
    at32f4xx_hal::pac::IOMUX::enable(&dp.CRM);
    at32f4xx_hal::pac::IOMUX::reset(&dp.CRM);
    let gpiob_bus = dp
        .GPIOB
        .bus_u16()
        .into_push_pull_output()
        .speed(Speed::High);
    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();
    let gpiof = dp.GPIOF.split();

    dp.IOMUX.remap().modify(|_r, w| {
        w.swjtag_mux().swd();
        w.tmr2_mux().mux1();
        // w.tmr2_mux().mux0();
        w
    });
    dp.IOMUX.remap7().modify(|_r, w| {
        w.swjtag_gmux().swd();
        w
    });

    info!("Setup gpio");

    // allocator::init();

    // info!("Setup alloc");

    // let wwdt_sts = dp.WWDT.sts().read();
    // let wwdt_ctrl = dp.WWDT.ctrl().read();
    // let wwdt_cfg = dp.WWDT.cfg().read();
    // info!("wwdt: {}, {}, {}",
    //       defmt::Debug2Format(&wwdt_sts),
    //       defmt::Debug2Format(&wwdt_ctrl),
    //       defmt::Debug2Format(&wwdt_cfg)
    // );

    // let wdt_cmd = dp.WDT.cmd().read();
    // let wdt_div = dp.WDT.div().read();
    // let wdt_rld = dp.WDT.rld().read();
    // let wdt_sts = dp.WDT.sts().read();
    // info!("wdt: {}, {}, {}, {}",
    //       defmt::Debug2Format(&wdt_cmd),
    //       defmt::Debug2Format(&wdt_div),
    //       defmt::Debug2Format(&wdt_rld),
    //       defmt::Debug2Format(&wdt_sts)
    // );

    // let icache_enabled = at32f4xx_hal::pac::SCB::icache_enabled();
    // let dcache_enabled = at32f4xx_hal::pac::SCB::dcache_enabled();
    cp.SCB.enable_icache();
    // info!("icache: {}, dcache: {}", icache_enabled, dcache_enabled);

    let mut delay = Timer::syst(cp.SYST, &clocks).delay();

    let backlight_pwm_pin = gpioa.pa15.into_alternate().speed(Speed::High);
    let mut backlight_pwm = dp
        .TMR2
        .pwm_hz(Channel1::new(backlight_pwm_pin), 32.kHz(), &clocks)
        .split();
    backlight_pwm.set_duty(backlight_pwm.get_max_duty() / 8);
    backlight_pwm.enable();

    let display = display::init(
        gpioc.pc0.into_push_pull_output().speed(Speed::High),
        gpioc.pc13.into_push_pull_output().speed(Speed::High),
        gpioc.pc14.into_push_pull_output().speed(Speed::High),
        gpioc.pc15.into_push_pull_output().speed(Speed::High),
        gpioc.pc1.into_push_pull_output().speed(Speed::High),
        gpiob_bus,
        &mut delay,
        backlight_pwm,
    );

    let can = at32f4xx_hal::can::Can::new(dp.CAN1, gpioa.pa11, gpioa.pa12, &clocks);

    static CAN_CELL: StaticCell<at32f4xx_hal::can::Can> = StaticCell::new();
    let can = CAN_CELL.init(can);
    can.modify_config()
        .set_loopback(false)
        .set_silent(false)
        .set_bitrate(250_000);
    can.enable().await;
    can.wakeup();
    can.set_automatic_wakeup(true);

    let (can_tx, can_rx) = can.split();

    let usart2 = Serial2::<u8>::new(
        dp.USART2,
        (gpioa.pa2, gpioa.pa3),
        uart::Config {
            baudrate: 57600.bps(),
            wordlength: WordLength::DataBits8,
            parity: Parity::ParityNone,
            stopbits: StopBits::STOP1,
            dma: DmaConfig::None,
        },
        &clocks,
    )
    .unwrap();
    let (usart2_tx, usart2_rx) = usart2.split();

    spawner.spawn(bluetooth::bluetooth_rx(usart2_rx).unwrap());
    spawner.spawn(bluetooth::bluetooth_tx(usart2_tx).unwrap());
    spawner.spawn(can::can_rx(can_rx).unwrap());
    spawner.spawn(can::can_tx(can_tx).unwrap());
    spawner.spawn(ui::ui(display).unwrap());

    loop {
        defmt::debug!("Tick");
        embassy_time::Timer::after_secs(5).await;
    }
}

#[entry]
fn main() -> ! {
    cortex_m::asm::delay(100000);
    let dp = unsafe { hal::pac::Peripherals::steal() };
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let crm = dp.CRM.constrain();
    let clocks = crm
        .cfgr
        .use_hext(8.MHz())
        .sclk(96.MHz()) // can seems to fall over if this is clocked any higher
        .pclk1(48.MHz())
        .pclk2(48.MHz())
        .freeze();

    defmt::debug!("Starting up with clocks: {:#}", clocks);

    critical_section::with(|cs| {
        time_driver::init(cs, &clocks);
    });

    info!("Setup time driver");

    static EXECUTOR: StaticCell<embassy_executor::Executor> = StaticCell::new();
    let executor = EXECUTOR.init(embassy_executor::Executor::new());

    let dp = unsafe { hal::pac::Peripherals::steal() };
    executor.run(move |spawner| {
        spawner.spawn(async_main(spawner, dp, cp, clocks).unwrap());
    });
}
