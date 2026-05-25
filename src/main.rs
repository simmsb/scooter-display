#![no_std]
#![no_main]

use at32f4xx_hal::{interrupt, spi::MODE_3, time::Hertz};
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
    adc, bluetooth, buttons, can, display, noodle, operation, rtc, system_state, time_driver, ui
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
    mut cp: cortex_m::Peripherals,
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

    cp.SCB.enable_icache();

    let mut delay = Timer::syst(cp.SYST, &clocks).delay();

    let mut power_button = ExtiInput::new(gpioa.pa1.into_input().internal_pull_up(true), exti.ch1);

    if !scooter_display::ON_BENCH {
        defmt::info!("Waiting for power press");
        // wait for power button press
        loop {
            power_button.wait_for_low().await;

            // ensure it was pressed for two second (time it takes for controller to boot)
            if let Err(TimeoutError) = power_button
                .wait_for_high()
                .with_timeout(Duration::from_secs(2))
                .await
            {
                break;
            }
        }
    }

    defmt::info!("Starting peripheral init");
    let mut system_power = gpioa.pa8.into_push_pull_output();
    system_power.set_low();
    let mut gpiof_4 = gpiof.pf4.into_push_pull_output();
    gpiof_4.set_low();

    rtc::init_rtc(dp.ERTC, &mut dp.CRM, &mut dp.PWC);

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

    // low_spawner.spawn(ui::ui(display).unwrap());
    high_spawner.spawn(system_state::system_state_updater().unwrap());
    high_spawner.spawn(adc::adc_task(adc, adc_ch12, adc_ch13, adc_ch15).unwrap());
    high_spawner.spawn(operation::operation_task().unwrap());

    defmt::info!("Setting up spi flash");

    let mut spi_config = at32f4xx_hal::spi::Config::default();
    spi_config.mode.phase = at32f4xx_hal::spi::Phase::CaptureOnFirstTransition;
    spi_config.mode.polarity = at32f4xx_hal::spi::Polarity::IdleLow;
    spi_config.bit_order = at32f4xx_hal::spi::BitOrder::MsbFirst;
    spi_config.frequency = Hertz::MHz(1);
    let spi: at32f4xx_hal::spi::Spi<at32f4xx_hal::spi::mode::Master> = at32f4xx_hal::spi::Spi::new(
        dp.SPI1,
        Some(gpioa.pa5),
        Some(gpioa.pa7),
        Some(gpioa.pa6),
        Some(gpioa.pa4),
        spi_config,
        &clocks,
    );

    high_spawner.spawn(noodle::worker(spi).unwrap());

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
    cortex_m::asm::delay(100000);
    let dp = unsafe { hal::pac::Peripherals::steal() };
    let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();
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
