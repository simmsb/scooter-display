// - UART5
// - rx: GPIOD 2 (0x4) (`UART5_RX`)
// - tx: GPIOC 12 (0x1000) (`UART5_TX`)
// - 9600 baud

use at32f4xx_hal::{exti::ExtiInput, gpio::Pin, uart::Serial5};
use butt_head::{ButtHead, ServiceTiming};
use deku::DekuContainerRead as _;
use embassy_executor::SendSpawner;
use embassy_futures::select::{self, select};
use embassy_sync::{blocking_mutex, pubsub::PubSubChannel, watch::Watch};
use embassy_time::{Duration, Instant, WithTimeout};
use embedded_io_async::Read as _;

use crate::buttons_proto::{self, ButtonParser, Buttons};

pub static BUTTON_STATE_WATCH: Watch<
    blocking_mutex::raw::CriticalSectionRawMutex,
    buttons_proto::Buttons,
    4,
> = Watch::new();

#[derive(Copy, Clone, defmt::Format, PartialEq, Eq, PartialOrd, Ord)]
pub struct BHDuration(pub Duration);

#[derive(Copy, Clone, defmt::Format, PartialEq, Eq, PartialOrd, Ord)]
pub struct BHInstant(pub Instant);

impl butt_head::TimeDuration for BHDuration {
    const ZERO: Self = Self(Duration::MIN);

    fn as_millis(&self) -> u64 {
        self.0.as_millis()
    }

    fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }

    fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.checked_sub(other.0).unwrap_or(Duration::MIN))
    }
}

impl butt_head::TimeInstant for BHInstant {
    type Duration = BHDuration;

    fn duration_since(&self, earlier: Self) -> Self::Duration {
        BHDuration(self.0.duration_since(earlier.0))
    }

    fn checked_add(self, duration: Self::Duration) -> Option<Self> {
        self.0.checked_add(duration.0).map(Self)
    }

    fn checked_sub(self, duration: Self::Duration) -> Option<Self> {
        self.0.checked_sub(duration.0).map(Self)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, defmt::Format)]
pub enum Button {
    Up,
    Down,
    Confirm,
    Power,
}

pub static BUTTON_EVENTS: PubSubChannel<
    blocking_mutex::raw::CriticalSectionRawMutex,
    (Button, butt_head::Event<BHDuration, BHInstant>),
    4,
    4,
    1,
> = PubSubChannel::new();

pub fn start_buttons(spawner: SendSpawner, uart: Serial5, power_button: ExtiInput<Pin<'A', 1>, 1>) {
    spawner.spawn(buttons_rx(uart, power_button).unwrap());
    spawner.spawn(buttons_eventer().unwrap());
}

#[embassy_executor::task]
async fn buttons_rx(rx: Serial5, power_button: ExtiInput<Pin<'A', 1>, 1>) {
    buttons_rx_(rx, power_button).await;
}

async fn buttons_rx_(rx: Serial5, mut power_button: ExtiInput<Pin<'A', 1>, 1>) {
    let (_, mut rx) = rx.split();
    let sender = BUTTON_STATE_WATCH.sender();

    defmt::info!("buttons RX startup");

    let mut buttons = Buttons::empty();

    loop {
        let mut buf = [0; 4];

        match select(rx.read_exact(&mut buf), power_button.wait_for_any_edge()).await {
            select::Either::First(r) => {
                if r.is_err() {
                    // XXX: this error case probably doesn't happen
                    defmt::warn!("read error button rx");
                    continue;
                };

                if (buf[0] ^ buf[2] != 0xff)
                    || (buf[1] ^ buf[3] != 0xff)
                    || (buf[0] & 0b11000000 != 0)
                    || (buf[1] & 0b11111110 != 0)
                {
                    defmt::warn!("buttons xor didn't match, or invalid: {}", buf);

                    // on framing error, read until we have nothing
                    let _ = async {
                        let _ = rx.read_exact(&mut buf).await;
                    }
                    .with_timeout(Duration::from_millis(10))
                    .await;

                    continue;
                }

                let parsed = match ButtonParser::from_bytes((&buf[..2], 0)) {
                    Ok(((_, _), buttons)) => buttons,
                    Err(e) => {
                        defmt::warn!(
                            "Button parse error: {} (in: {})",
                            defmt::Debug2Format(&e),
                            buf
                        );
                        continue;
                    }
                };

                buttons.update_from_uart(parsed);
            }
            select::Either::Second(_) => {
                buttons.set(Buttons::POWER, power_button.is_low());
            }
        }

        sender.send_if_modified(|x| {
            if *x != Some(buttons) {
                *x = Some(buttons);

                true
            } else {
                false
            }
        });

        defmt::trace!("Buttons update: {}", defmt::Debug2Format(&buttons));
    }
}

#[embassy_executor::task]
async fn buttons_eventer() {
    buttons_eventer_().await
}

async fn buttons_eventer_() {
    let mut buttons_changes = BUTTON_STATE_WATCH.receiver().unwrap();
    let button_events = BUTTON_EVENTS.publisher().unwrap();

    static CONFIG: butt_head::Config<BHDuration> = butt_head::Config {
        active_low: false,
        // don't do multi-clicks
        click_timeout: BHDuration(Duration::MIN),
        hold_delay: BHDuration(Duration::from_millis(700)),
        hold_interval: BHDuration(Duration::from_millis(1000)),
        max_click_count: None,
    };

    static CONFIG_POWER: butt_head::Config<BHDuration> = butt_head::Config {
        active_low: false,
        // don't do multi-clicks
        click_timeout: BHDuration(Duration::MIN),
        hold_delay: BHDuration(Duration::from_millis(700)),
        hold_interval: BHDuration(Duration::from_secs(60)),
        max_click_count: None,
    };

    let mut up_state = ButtHead::new(&CONFIG);
    let mut down_state = ButtHead::new(&CONFIG);
    let mut confirm_state = ButtHead::new(&CONFIG);
    let mut power_state = ButtHead::new(&CONFIG_POWER);

    let handle_update_result =
        |upd: butt_head::UpdateResult<BHDuration, BHInstant>,
         next_service: &mut ServiceTiming<BHDuration>| {
            *next_service = next_service.min(upd.next_service);

            upd.event
        };

    let mut next_service = ServiceTiming::<BHDuration>::Idle;

    loop {
        let buttons = buttons_changes
            .changed()
            .with_timeout(match next_service {
                ServiceTiming::Immediate => Duration::MIN,
                ServiceTiming::Delay(d) => d.0,
                ServiceTiming::Idle => Duration::from_secs(600),
            })
            .await
            .ok()
            .unwrap_or_else(|| buttons_changes.try_get().unwrap_or(Buttons::empty()));

        let now = BHInstant(Instant::now());
        next_service = ServiceTiming::Idle;

        if let Some(evt) = handle_update_result(
            up_state.update(buttons.contains(Buttons::UP), now),
            &mut next_service,
        ) {
            button_events.publish((Button::Up, evt)).await;
        }

        if let Some(evt) = handle_update_result(
            down_state.update(buttons.contains(Buttons::DOWN), now),
            &mut next_service,
        ) {
            button_events.publish((Button::Down, evt)).await;
        }

        if let Some(evt) = handle_update_result(
            confirm_state.update(buttons.contains(Buttons::CONFIRM), now),
            &mut next_service,
        ) {
            button_events.publish((Button::Confirm, evt)).await;
        }

        if let Some(evt) = handle_update_result(
            power_state.update(buttons.contains(Buttons::POWER), now),
            &mut next_service,
        ) {
            button_events.publish((Button::Power, evt)).await;
        }
    }
}
