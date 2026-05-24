use once_cell::sync::OnceCell;

use at32f4xx_hal::{
    pac::{CRM, ERTC, PWC},
    rtc::Rtc,
};
use chrono::NaiveDateTime;

static RTC: OnceCell<
    embassy_sync::blocking_mutex::Mutex<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        Rtc,
    >,
> = OnceCell::new();

pub fn init_rtc(regs: ERTC, crm: &mut CRM, pwr: &mut PWC) {
    // HW has no lse (lext) attached
    let rtc = Rtc::new_lsi(regs, crm, pwr);

    RTC.set(embassy_sync::blocking_mutex::Mutex::new(rtc))
        .unwrap();
}

pub fn get_datetime() -> NaiveDateTime {
    unsafe { RTC.get().unwrap().lock_mut(|rtc| rtc.get_datetime()) }
}

pub fn set_datetime(dt: NaiveDateTime) {
    unsafe {
        RTC.get().unwrap().lock_mut(|rtc| _ = rtc.set_datetime(&dt));
    }
}
