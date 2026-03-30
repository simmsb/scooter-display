#![no_std]
#![no_main]

extern crate alloc;

use defmt::info;
use panic_probe as _;

use at32f4xx_hal::{
    self as hal,
    crm::{Clocks, Enable, Reset},
    gpio::{GpioBusExt as _, OutputPin, PinSpeed as _, Speed},
    pac::{GPIOA, Peripherals},
    prelude::*,
    signature::IDCode,
    timer::{Channel1, Timer},
};
use cortex_m_rt::entry;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use defmt_rtt as _;
use static_cell::StaticCell;

// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     core::intrinsics::abort();
// }

mod allocator;
mod display;
mod slint;
mod time_driver;

#[embassy_executor::task]
async fn async_main(dp: Peripherals, cp: cortex_m::Peripherals, clocks: Clocks) {
    async_main_(dp, cp, clocks).await;
}

async fn async_main_(dp: Peripherals, mut cp: cortex_m::Peripherals, clocks: Clocks) {
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

    allocator::init();

    info!("Setup alloc");

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

    // let mut backlight = gpioa.pa8.into_push_pull_output();
    // backlight.set_high();

    // !!!! these pins are needed
    let mut pc13 = gpioc.pc13.into_push_pull_output().speed(Speed::High);
    pc13.set_low();
    let mut pc0 = gpioc.pc0.into_push_pull_output().speed(Speed::High);
    pc0.set_high();

    // let mut pf4 = gpiof.pf4.into_push_pull_output();
    // pf4.set_low();
    // let mut pf6 = gpiof.pf6.into_push_pull_output();
    // pf6.set_high();

    // let mut pa7 = gpioa.pa7.into_push_pull_output();
    // pa7.set_high();
    // let mut pa4 = gpioa.pa4.into_push_pull_output();
    // pa4.set_high();

    let mut display = display::init(
        gpioc.pc14.into_push_pull_output().speed(Speed::High),
        gpioc.pc15.into_push_pull_output().speed(Speed::High),
        gpioc.pc1.into_push_pull_output().speed(Speed::High),
        gpiob_bus,
        &mut delay,
    );

    display.wake(&mut delay);
    display.clear(Rgb565::RED);

    // 0,0   : top right
    // 320,0 : top left
    //

    display.fill_solid(
        &embedded_graphics::primitives::Rectangle {
            top_left: Point::new(0, 466),
            size: Size::new(10, 10),
        },
        Rgb565::BLUE,
    );

    loop {
        info!("Loop");
        embassy_time::Timer::after_secs(1).await;

        let gpioa = unsafe { GPIOA::steal() };
        let pa15_mode = gpioa.cfghr().read().iomc15().variant();
        let pa15_function = gpioa.cfghr().read().iofc15().bits();
        info!("{} {:b}", defmt::Debug2Format(&pa15_mode), pa15_function);

        let wpen = gpioa.wpr().read().bits();
        info!("wpen: {:b}", wpen);

        dp.IOMUX.remap().modify(|_r, w| {
            w.swjtag_mux().swd();
            w.tmr2_mux().mux1();
            w
        });
        let iomux_out = dp.IOMUX.remap().read().bits();
        info!("iomux:remap: {:b}", iomux_out);
        // cortex_m::asm::delay(0xfffff);
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
        .sclk(96.MHz())
        .pclk1(24.MHz())
        .pclk2(24.MHz())
        .hclk(96.MHz())
        .freeze();

    critical_section::with(|cs| {
        time_driver::init(cs, &clocks);
    });

    info!("Setup time driver");

    static EXECUTOR: StaticCell<embassy_executor::Executor> = StaticCell::new();
    let executor = EXECUTOR.init(embassy_executor::Executor::new());

    let dp = unsafe { hal::pac::Peripherals::steal() };
    executor.run(move |spawner| {
        spawner.spawn(async_main(dp, cp, clocks).unwrap());
    });
}
