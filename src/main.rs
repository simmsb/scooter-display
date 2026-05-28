#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use at32f4xx_hal::interrupt;
use embassy_executor::{InterruptExecutor, SendSpawner, Spawner};

use embassy_time::{Duration, TimeoutError, WithTimeout};
#[cfg(feature = "panic-probe")]
use panic_probe as _;

use at32f4xx_hal::{
    self as hal,
    adc::{Adc, config::AdcConfig},
    crm::{Clocks, Enable, Reset},
    exti::{ExtiExt as _, ExtiInput},
    gpio::{GpioBusExt as _, PinSpeed as _, Speed},
    pac::{NVIC, Peripherals},
    prelude::*,
    serial::Serial2,
    timer::{Channel1, Timer},
    uart::{
        self, Serial5,
        config::{DmaConfig, Parity, StopBits, WordLength},
    },
};
use cortex_m_rt::entry;

use defmt_rtt as _;
use static_cell::StaticCell;

use scooter_display::{
    adc, bluetooth, buttons, can, display, noodle, operation, rtc, system_state, time_driver, ui,
};

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();

#[cfg(feature = "panic-scram")]
#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    scooter_display::scram::scram();
}

#[cfg(feature = "panic-scram")]
#[cortex_m_rt::exception(trampoline = true)]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    scooter_display::scram::scram();
}

#[embassy_executor::task]
async fn async_main(
    low_spawner: Spawner,
    high_spawner: SendSpawner,
    dp: Peripherals,
    cp: cortex_m::Peripherals,
    clocks: Clocks,
) {
    async_main_(low_spawner, high_spawner, dp, cp, clocks).await;
}

async fn async_main_(
    low_spawner: Spawner,
    high_spawner: SendSpawner,
    mut dp: Peripherals,
    cp: cortex_m::Peripherals,
    clocks: Clocks,
) {
    defmt::info!("Main stage startup begins");

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

    let exti = dp.EXINT.split();

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

    let mut delay = Timer::syst(cp.SYST, &clocks).delay();

    let power_button = ExtiInput::new(gpioa.pa1.into_input().internal_pull_up(true), exti.ch1);

    // if !scooter_display::ON_BENCH {
    //     defmt::info!("Waiting for power press");
    //     // wait for power button press
    //     loop {
    //         power_button.wait_for_low().await;

    //         // ensure it was pressed for two second (time it takes for controller to boot)
    //         if let Err(TimeoutError) = power_button
    //             .wait_for_high()
    //             .with_timeout(Duration::from_secs(3))
    //             .await
    //         {
    //             break;
    //         }
    //     }
    // }

    defmt::info!("Starting peripheral init");

    // one of these controls voltage on the USB port
    let mut system_power = gpioa.pa8.into_push_pull_output();
    system_power.set_high();
    let mut gpiof_4 = gpiof.pf4.into_push_pull_output();
    gpiof_4.set_high();

    rtc::init_rtc(dp.ERTC, &mut dp.CRM, &mut dp.PWC);

    let backlight_pwm_pin = gpioa.pa15.into_alternate().speed(Speed::High);
    let mut backlight_pwm = dp
        .TMR2
        .pwm_hz(Channel1::new(backlight_pwm_pin), 32.kHz(), &clocks)
        .split();
    backlight_pwm.set_duty(backlight_pwm.get_max_duty() / 8);
    // backlight_pwm.enable();

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

    let uart5 = Serial5::<u8>::new(
        dp.UART5,
        (gpioc.pc12, gpiod.pd2),
        uart::Config {
            baudrate: 9600.bps(),
            wordlength: WordLength::DataBits8,
            parity: Parity::ParityNone,
            stopbits: StopBits::STOP1,
            dma: DmaConfig::None,
        },
        &clocks,
    )
    .unwrap();

    let adc_config = AdcConfig::default();
    let adc = Adc::adc1(dp.ADC1, true, adc_config);

    let adc_ch12 = gpioc.pc2.into_analog();
    let adc_ch13 = gpioc.pc3.into_analog();
    let adc_ch15 = gpioc.pc5.into_analog();

    bluetooth::start_bluetooth(high_spawner, usart2);
    buttons::start_buttons(high_spawner, uart5, power_button);
    can::start_can(high_spawner, can_tx, can_rx);

    low_spawner.spawn(ui::ui(display).unwrap());
    high_spawner.spawn(system_state::system_state_updater().unwrap());
    high_spawner.spawn(adc::adc_task(adc, adc_ch12, adc_ch13, adc_ch15).unwrap());
    high_spawner.spawn(operation::operation_task().unwrap());

    low_spawner.spawn(noodle::worker(dp.FLASH).unwrap());

    defmt::info!("Startup complete");

    loop {
        defmt::debug!("Tick");
        embassy_time::Timer::after_secs(10).await;
    }
}

#[interrupt]
fn OTGFS() {
    unsafe {
        EXECUTOR_HIGH.on_interrupt();
    }
}

#[entry]
fn main() -> ! {
    // cortex_m::asm::delay(100000);
    let dp = unsafe { hal::pac::Peripherals::steal() };
    let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();

    dp.CRM.ctrl().reset();
    dp.CRM.cfg().reset();
    dp.CRM.clkint().reset();
    dp.CRM.pll().reset();
    dp.CRM.misc1().modify(|_, w| unsafe {
        w.clkoutdiv()
            .bits(0)
            .hickdiv()
            .bit(false)
            .clkout_sel3()
            .bit(false)
    });

    cp.SCB.enable_icache();
    // let vtor = cp.SCB.vtor.read();
    // defmt::info!("VTOR at: {:x}", vtor);

    // let pwc_ctrl = dp.PWC.ctrl().read().bits();
    // let pwc_ctrlsts = dp.PWC.ctrlsts().read().bits();
    // defmt::info!("PWC: {:x} {:x}", pwc_ctrl, pwc_ctrlsts);

    // let crm_ctrl = dp.CRM.ctrl().read();
    // let crm_ctrlsts = dp.CRM.ctrlsts().read();
    // let crm_cfg = dp.CRM.cfg().read();
    // let crm_apb2rst = dp.CRM.apb2rst().read();
    // let crm_apb1rst = dp.CRM.apb1rst().read();
    // let crm_apb2en = dp.CRM.apb2en().read();
    // let crm_apb1en = dp.CRM.apb1en().read();
    // let crm_ahbrst = dp.CRM.ahbrst().read();
    // let crm_ahben = dp.CRM.ahben().read();
    // let crm_pll = dp.CRM.pll().read();
    // let crm_misc1 = dp.CRM.misc1().read();
    // let crm_misc2 = dp.CRM.misc2().read();
    // cortex_m::asm::delay(100000);
    // defmt::info!("CRM1.1: {}", defmt::Debug2Format(&crm_ctrl));
    // cortex_m::asm::delay(100000);
    // defmt::info!("CRM1.2: {}", defmt::Debug2Format(&crm_ctrlsts));
    // cortex_m::asm::delay(100000);
    // defmt::info!("CRM1.3: {}", defmt::Debug2Format(&crm_cfg));
    // cortex_m::asm::delay(100000);
    // defmt::info!("CRM1.4: {}", defmt::Debug2Format(&crm_apb2rst));
    // cortex_m::asm::delay(100000);
    // defmt::info!("CRM1.5: {}", defmt::Debug2Format(&crm_apb1rst));
    // cortex_m::asm::delay(100000);
    // defmt::info!("CRM2.1: {}", defmt::Debug2Format(&crm_apb2en));
    // cortex_m::asm::delay(100000);
    // defmt::info!("CRM2.2: {}", defmt::Debug2Format(&crm_apb1en));
    // cortex_m::asm::delay(100000);
    // defmt::info!(
    //     "CRM2.3: {} {}",
    //     defmt::Debug2Format(&crm_ahbrst),
    //     defmt::Debug2Format(&crm_ahben)
    // );
    // cortex_m::asm::delay(100000);
    // defmt::info!("CRM2.4: {}", defmt::Debug2Format(&crm_pll));
    // cortex_m::asm::delay(10000);
    // defmt::info!(
    //     "CRM3: {} {}",
    //     defmt::Debug2Format(&crm_misc1),
    //     defmt::Debug2Format(&crm_misc2)
    // );

    // The bootloader jumps to us with some clocks enabled, we need to manually
    // disable the peripheral clocks and then system clocks here so that we can
    // modify them.

    dp.CRM.apb2en().modify(|_, w| {
        w.iomux()
            .bit(false)
            .gpioa()
            .bit(false)
            .gpiof()
            .bit(false)
            .spi1()
            .bit(false)
    });
    dp.CRM.apb1en().modify(|_, w| w.can1().bit(false));
    dp.CRM.ahben().modify(|_, w| w.dma1().bit(false));

    dp.CRM
        .ctrl()
        .modify(|_, w| w.pllen().clear_bit().hexten().clear_bit());

    dp.CRM.cfg().modify(|_, w| unsafe {
        w.pllrcs()
            .clear_bit()
            .pllmult3_0()
            .bits(0)
            .pllmult5_4()
            .bits(0)
    });

    // differences:
    //
    // CTRL.hexten: true on bootload
    // CTRL.pllen: true on bootload
    // CFG.pllrcs: true on bootload
    // CFG.pllmult3_0: 1 on bootload
    // CFG.pllmult5_4: 1 on bootload
    //
    // APB2EN: iomux | gpioa | gpiof | spi1 enabled on bootload (nothing on clean)
    // APB1EN: can1 on bootload (nothing on clean)
    // AHBEN: dma1 on bootload

    // starting at 0x8000
    // 0.000000 [INFO ] VTOR at: 8008000
    // 0.000000 [INFO ] PWC: 0 0
    // 0.000000 [INFO ] CRM1: 3038783 1c000003 20050000 0 0
    // 0.000000 [INFO ] CRM2: 1085 2000000 0 15 1f10
    // 0.000000 [INFO ] CRM3: 100000 d
    //
    // 0.000000 [INFO ] CRM1.1: CTRL { hicken: true, hickstbl: true, hicktrim: 32, hickcal: 135, hexten: true, hextstbl: true, hextbyps: false, cfden: false, pllen: true, pllstbl: true }
    // 0.000000 [INFO ] CRM1.2: CTRLSTS { licken: true, lickstbl: true, rstfc: false, nrstf: true, porrstf: true, swrstf: true, wdtrstf: false, wwdtrstf: false, lprstf: false }
    // 0.000000 [INFO ] CRM1.3: CFG { sclksel: 0, sclksts: 0, ahbdiv: 0, apb1div: 0, apb2div: 0, adcdiv1_0: 0, pllrcs: true, pllhextdiv: false, pllmult3_0: 1, usbdiv1_0: 0, clkout_sel: 0, usbdiv2: false, adcdiv2: false, pllmult5_4: 1 }
    // 0.000000 [INFO ] CRM1.4: APB2RST { iomux: false, exint: false, gpioa: false, gpiob: false, gpioc: false, gpiod: false, gpiof: false, adc1: false, tmr1: false, spi1: false, usart1: false, tmr9: false, tmr10: false, tmr11: false, acc: false }
    // 0.000000 [INFO ] CRM1.5: APB1RST { tmr2: false, tmr3: false, tmr4: false, tmr5: false, cmp: false, wwdt: false, spi2: false, usart2: false, usart3: false, uart4: false, uart5: false, i2c1: false, i2c2: false, can1: false, pwc: false }
    // 0.000000 [INFO ] CRM2.1: APB2EN { iomux: true, gpioa: true, gpiob: false, gpioc: false, gpiod: false, gpiof: true, adc1: false, tmr1: false, spi1: true, usart1: false, tmr9: false, tmr10: false, tmr11: false, acc: false }
    // 0.000000 [INFO ] CRM2.2: APB1EN { tmr2: false, tmr3: false, tmr4: false, tmr5: false, cmp: false, wwdt: false, spi2: false, usart2: false, usart3: false, uart4: false, uart5: false, i2c1: false, i2c2: false, can1: true, pwc: false }
    // 0.000000 [INFO ] CRM2.3: AHBRST { otgfs1: false } AHBEN { dma1: true, dma2: false, sram: true, flash: true, crc: false, sdio1: false, otgfs1: false }
    // 0.000000 [INFO ] CRM2.4: PLL { pll_fr: 0, pll_ms: 1, pll_ns: 31, pll_fref: 0, pllcfgen: false }
    // 0.000000 [INFO ] CRM3: MISC1 { hickcal_key: 0, clkout_sel3: false, hickdiv: false, clkoutdiv: 0 } MISC2 { auto_step_en: 0, hick_to_usb: false, hick_to_sclk: false }
    //
    // at 0x0
    // 0.000000 [INFO ] VTOR at: 8000000
    // 0.000000 [INFO ] PWC: 0 0
    // 0.000000 [INFO ] CRM1: 8783 1c000003 0 0 0
    // 0.000000 [INFO ] CRM2: 0 0 0 14 1f10
    // 0.000000 [INFO ] CRM3: 100000 d
    //
    // 0.000000 [INFO ] CRM1: CTRL { hicken: true, hickstbl: true, hicktrim: 32, hickcal: 135, hexten: false, hextstbl: false, hextbyps: false, cfden: false, pllen: false, pllstbl: false }
    //                        CTRLSTS { licken: true, lickstbl: true, rstfc: false, nrstf: true, porrstf: true, swrstf: true, wdtrstf: false, wwdtrstf: false, lprstf: false }
    //                        CFG { sclksel: 0, sclksts: 0, ahbdiv: 0, apb1div: 0, apb2div: 0, adcdiv1_0: 0, pllrcs: false, pllhextdiv: false, pllmult3_0: 0, usbdiv1_0: 0, clkout_sel: 0, usbdiv2: false, adcdiv2: false, pllmult5_4: 0 }
    //                        APB2RST { iomux: false, exint: false, gpioa: false, gpiob: false, gpioc: false, gpiod: false, gpiof: false, adc1: false, tmr1: false, spi1: false, usart1: false, tmr9: false, tmr10: false, tmr11: false, acc: false }
    //                        APB1RST { tmr2: false, tmr3: false, tmr4: false, tmr5: false, cmp: false, wwdt: false, spi2: false, usart2: false, usart3: false, uart4: false, uart5: false, i2c1: false, i2c2: false, can1: false, pwc: false }
    // 0.000000 [INFO ] CRM2: APB2EN { iomux: false, gpioa: false, gpiob: false, gpioc: false, gpiod: false, gpiof: false, adc1: false, tmr1: false, spi1: false, usart1: false, tmr9: false, tmr10: false, tmr11: false, acc: false }
    //                        APB1EN { tmr2: false, tmr3: false, tmr4: false, tmr5: false, cmp: false, wwdt: false, spi2: false, usart2: false, usart3: false, uart4: false, uart5: false, i2c1: false, i2c2: false, can1: false, pwc: false }
    //                        AHBRST { otgfs1: false } AHBEN { dma1: false, dma2: false, sram: true, flash: true, crc: false, sdio1: false, otgfs1: false }
    //                        PLL { pll_fr: 0, pll_ms: 1, pll_ns: 31, pll_fref: 0, pllcfgen: false }
    // 0.000000 [INFO ] CRM3: MISC1 { hickcal_key: 0, clkout_sel3: false, hickdiv: false, clkoutdiv: 0 } MISC2 { auto_step_en: 0, hick_to_usb: false, hick_to_sclk: false }
    // 0.000000 [DEBUG] Starting up with clocks: Clocks { sclk: 96000000 Hz, hclk: 96000000 Hz, pclk1: 48000000 Hz, pclk2: 48000000 Hz, tmr1clk: 96000000 Hz, tmr2clk: 96000000 Hz, usb48m: None }

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

    defmt::info!("Setup time driver");

    static EXECUTOR: StaticCell<embassy_executor::Executor> = StaticCell::new();
    let executor = EXECUTOR.init(embassy_executor::Executor::new());

    unsafe {
        cp.NVIC.set_priority(at32f4xx_hal::interrupt::OTGFS, 6);
        NVIC::unpend(at32f4xx_hal::interrupt::OTGFS);
        NVIC::unmask(at32f4xx_hal::interrupt::OTGFS);
    }
    let high_spawner = EXECUTOR_HIGH.start(at32f4xx_hal::interrupt::OTGFS);

    let dp = unsafe { hal::pac::Peripherals::steal() };
    executor.run(move |spawner| {
        spawner.spawn(async_main(spawner, high_spawner, dp, cp, clocks).unwrap());
    });
}
