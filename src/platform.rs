#[cfg(feature = "app")]
pub fn can_is_alive() -> bool {
    use embassy_time::Duration;

    crate::can::LAST_SEEN_CAN_MESSAGE.lock(|c| c.elapsed() < Duration::from_secs(1))
}

#[cfg(feature = "sim")]
pub fn can_is_alive() -> bool {
    crate::sim::can_is_alive()
}

#[cfg(feature = "app")]
pub fn trigger_shutdown() {
    crate::scram::trigger_controller_shutdown();
}

#[cfg(feature = "sim")]
pub fn trigger_shutdown() {}

#[cfg(feature = "app")]
pub fn current_time() -> chrono::NaiveTime {
    crate::rtc::get_datetime().time()
}

#[cfg(feature = "sim")]
pub fn current_time() -> chrono::NaiveTime {
    chrono::Local::now().time()
}
