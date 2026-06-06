use at32f4xx_hal::{
    adc::{Adc, config::SampleTime},
    gpio::{Analog, Pin},
    pac::ADC1,
};
use embassy_time::Duration;
use no_std_moving_average::MovingAverage;

pub static ADC_READINGS: embassy_sync::pubsub::PubSubChannel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    AdcReading,
    4,
    4,
    1,
> = embassy_sync::pubsub::PubSubChannel::new();

pub static THROTTLE_READINGS: embassy_sync::watch::Watch<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    Throttle,
    4,
> = embassy_sync::watch::Watch::new();

pub static AMBIENT_READINGS: embassy_sync::watch::Watch<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    AmbientLight,
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

    // value we report to the controller when throttle is fully depressed
    const OUT_MAX: u32 = 450;

    fn from_raw(raw: u16) -> Self {
        // value the adc reads when throttle is fully depressed
        const MAX_RAW: u32 = 2820;

        // value the adc reads when the throttle is unpressed
        const MIN_RAW: u32 = 730;

        Self(
            (raw as u32)
                .clamp(MIN_RAW, MAX_RAW)
                .saturating_sub(MIN_RAW)
                .saturating_mul(Self::OUT_MAX)
                .saturating_div(MAX_RAW - MIN_RAW)
                .saturating_truncate(),
        )
    }

    pub(crate) fn for_bluetooth(&self) -> u8 {
        const MAX_BT: u32 = 146;

        (self.0 as u32)
            .saturating_mul(MAX_BT)
            .saturating_div(Self::OUT_MAX)
            .saturating_truncate()
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Default, defmt::Format, Clone, Copy)]
pub struct AmbientLight {
    // pub raw: u16,
    pub mapped: u8,
}

impl AmbientLight {
    // value the adc reads when light sensor is fully exposed
    const MAX_RAW: u32 = 4096;

    // value the adc reads when light sensor is covered
    const MIN_RAW: u32 = 0;

    // what should we consider max
    const OUT_MAX: u32 = 64;

    pub const INITIAL: Self = Self {
        // raw: Self::MIN_RAW as u16,
        mapped: 0,
    };

    fn from_raw(raw: u16) -> Self {
        Self {
            // raw,
            mapped: (raw as u32)
                .clamp(Self::MIN_RAW, Self::MAX_RAW)
                .saturating_sub(Self::MIN_RAW)
                .saturating_mul(Self::OUT_MAX)
                .saturating_div(Self::MAX_RAW - Self::MIN_RAW)
                .saturating_truncate(),
        }
    }
}

async fn adc_task_(
    mut adc: Adc<ADC1>,
    // ambient light
    ch12: Pin<'C', 2, Analog>,

    // throttle
    ch13: Pin<'C', 3, Analog>,

    // unknown/ battery voltage
    _ch15: Pin<'C', 5, Analog>,
) {
    let mut do_sample_ticker = embassy_time::Ticker::every(Duration::from_millis(100));
    // let mut sample_ch15_ticker = embassy_time::Ticker::every(Duration::from_secs(1));

    let state_reading_ch = ADC_READINGS.publisher().unwrap();
    let throttle_reading_ch = THROTTLE_READINGS.sender();
    let ambient_reading_ch = AMBIENT_READINGS.sender();

    let mut throttle_averager = MovingAverage::<u16, u32, 4>::new();
    let mut ambient_light_averager = MovingAverage::<u16, u32, 16>::new();

    loop {
        do_sample_ticker.next().await;

        defmt::trace!("ADC measuring ambient");
        let val = adc.convert(&ch12, SampleTime::Cycles_480).await;
        let avg = ambient_light_averager.average(val);
        let ambient_light = AmbientLight::from_raw(avg);
        state_reading_ch
            .publish(AdcReading::AmbientLight(ambient_light))
            .await;
        ambient_reading_ch.send(ambient_light);

        defmt::trace!("ADC measuring throttle");
        let val = adc.convert(&ch13, SampleTime::Cycles_480).await;
        let avg = throttle_averager.average(val);
        let thr = Throttle::from_raw(avg);
        state_reading_ch.publish(AdcReading::Throttle(thr)).await;
        throttle_reading_ch.send(thr);
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
