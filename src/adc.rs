use at32f4xx_hal::{
    adc::{Adc, config::SampleTime},
    gpio::{Analog, Pin},
    pac::ADC1,
};
use embassy_futures::select::{Either3, select3};
use embassy_time::Duration;
use no_std_moving_average::MovingAverage;

pub static ADC_READINGS: embassy_sync::watch::Watch<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    AdcReading,
    4,
> = embassy_sync::watch::Watch::new();

pub static THROTTLE_READINGS: embassy_sync::watch::Watch<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    Throttle,
    4,
> = embassy_sync::watch::Watch::new();

#[derive(Clone, Copy)]
pub enum AdcReading {
    Throttle(Throttle),
    AmbientLight(AmbientLight),
}

#[derive(Eq, PartialEq, Default, defmt::Format, Clone, Copy)]
pub struct Throttle(pub u16);

impl Throttle {
    pub const INITIAL: Self = Self(0);

    fn from_raw(raw: u16) -> Self {
        // value the adc reads when throttle is fully depressed
        const MAX_RAW: u32 = 2820;

        // value the adc reads when the throttle is unpressed
        const MIN_RAW: u32 = 730;

        // value we report to the controller when throttle is fully depressed
        const OUT_MAX: u32 = 512;

        Self(
            (raw as u32)
                .clamp(MIN_RAW, MAX_RAW)
                .saturating_sub(MIN_RAW)
                .saturating_mul(OUT_MAX)
                .saturating_div(MAX_RAW - MIN_RAW)
                .saturating_truncate(),
        )
    }
}

#[derive(Eq, PartialEq, Default, defmt::Format, Clone, Copy)]
pub struct AmbientLight(pub u8);

impl AmbientLight {
    pub const INITIAL: Self = Self(0);

    fn from_raw(raw: u16) -> Self {
        // value the adc reads when light sensor is fully exposed
        const MAX_RAW: u32 = 4096;

        // value the adc reads when light sensor is covered
        const MIN_RAW: u32 = 900;

        // what should we consider max
        const OUT_MAX: u32 = 64;

        Self(
            (raw as u32)
                .clamp(MIN_RAW, MAX_RAW)
                .saturating_sub(MIN_RAW)
                .saturating_mul(OUT_MAX)
                .saturating_div(MAX_RAW - MIN_RAW)
                .saturating_truncate(),
        )
    }
}

async fn adc_task_(
    mut adc: Adc<ADC1>,
    // ambient light
    ch12: Pin<'C', 2, Analog>,

    // throttle
    ch13: Pin<'C', 3, Analog>,

    // unknown/ battery voltage
    ch15: Pin<'C', 5, Analog>,
) {
    let mut sample_ambient_light_ticker = embassy_time::Ticker::every(Duration::from_secs(1));
    let mut sample_throttle_ticker = embassy_time::Ticker::every(Duration::from_millis(100));
    let mut sample_ch15_ticker = embassy_time::Ticker::every(Duration::from_secs(1));

    let state_reading_ch = ADC_READINGS.sender();
    let throttle_reading_ch = THROTTLE_READINGS.sender();

    let mut throttle_averager = MovingAverage::<u16, u32, 4>::new();
    let mut ambient_light_averager = MovingAverage::<u16, u32, 16>::new();

    loop {
        match select3(
            sample_ambient_light_ticker.next(),
            sample_throttle_ticker.next(),
            sample_ch15_ticker.next(),
        )
        .await
        {
            Either3::First(_) => {
                let val = adc.convert(&ch12, SampleTime::Cycles_480).await;
                let avg = ambient_light_averager.average(val);
                state_reading_ch.send(AdcReading::AmbientLight(AmbientLight::from_raw(avg)));
            }
            Either3::Second(_) => {
                let val = adc.convert(&ch13, SampleTime::Cycles_480).await;
                let avg = throttle_averager.average(val);
                let thr = Throttle::from_raw(avg);
                state_reading_ch.send(AdcReading::Throttle(thr));
                throttle_reading_ch.send(thr);
            }
            Either3::Third(_) => {
                let _val = adc.convert(&ch15, SampleTime::Cycles_480).await;
            }
        }
    }
}

#[embassy_executor::task]
pub async fn adc_task(
    adc: Adc<ADC1>,
    ch12: Pin<'C', 2, Analog>,
    ch13: Pin<'C', 3, Analog>,
    ch15: Pin<'C', 5, Analog>,
) {
    adc_task_(adc, ch12, ch13, ch15).await;
}
