use embassy_time::{Duration, Instant};

#[derive(PartialEq, Eq, defmt::Format, Clone)]
enum AveragerState {
    Started { at: Instant, sum: u64, num: u64 },
    Collected,
}

#[derive(PartialEq, Eq, defmt::Format, Clone)]
pub struct Averager {
    state: AveragerState,
}

impl Default for Averager {
    fn default() -> Self {
        Self {
            state: AveragerState::Collected,
        }
    }
}

impl Averager {
    pub fn feed(&mut self, value: u64) {
        match &mut self.state {
            AveragerState::Started { sum, num, .. } => {
                *sum = sum.saturating_add(value);
                *num = num.saturating_add(1);
            }
            AveragerState::Collected => {
                self.state = AveragerState::Started {
                    at: Instant::now(),
                    sum: value,
                    num: 1,
                }
            }
        }
    }

    pub fn take(&mut self) -> (Duration, u64) {
        match core::mem::replace(&mut self.state, AveragerState::Collected) {
            AveragerState::Started { at, sum, num } => {
                let dur = at.elapsed();
                let avg = sum.checked_div(num).unwrap_or(0);

                (dur, avg)
            }
            AveragerState::Collected => (Duration::MIN, 0),
        }
    }
}
