use crate::RollingTimer;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
static T0: Lazy<Instant> = Lazy::new(|| Instant::now());

#[derive(Default, Clone)]
pub struct Timer<const TPS: u32>;

impl<const TPS: u32> Timer<TPS> {
    // TODO: evaluate if we're losing precision here
    const NANOS_PER_TICK: u128 = 1_000_000_000 / (TPS as u128);

    pub fn new() -> Self {
        Timer
    }
}

impl<const TPS: u32> RollingTimer for Timer<TPS> {
    type Tick = u32;
    const TICKS_PER_SECOND: Self::Tick = TPS;

    fn is_initialized(&self) -> bool {
        true
    }

    fn get_ticks(&self) -> Self::Tick {
        let ticks = Instant::now()
            .checked_duration_since(*T0)
            .unwrap_or_else(|| Duration::from_secs(0));

        let tnanos = ticks.as_nanos();
        let div = tnanos / Self::NANOS_PER_TICK;
        (div & 0xFFFF_FFFF) as u32
    }
}

#[cfg(test)]
mod test {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    use super::Timer;
    use crate::RollingTimer;

    #[test]
    fn sanity_test_1m() {
        let timer: Timer<1_000_000> = Timer::new();
        let start_gh = timer.get_ticks();
        let start = Instant::now();
        sleep(Duration::from_secs(1));
        let stop_gh = timer.millis_since(start_gh);
        let stop = start.elapsed();

        assert!((stop >= Duration::from_millis(998)) && (stop <= Duration::from_millis(1002)));
        assert!((stop_gh >= 998) && (stop_gh <= 1002));
    }

    #[test]
    fn sanity_test_1k() {
        let timer: Timer<1_000> = Timer::new();
        let start_gh = timer.get_ticks();
        let start = Instant::now();
        sleep(Duration::from_secs(1));
        let stop_gh = timer.millis_since(start_gh);
        let stop = start.elapsed();

        assert!((stop >= Duration::from_millis(998)) && (stop <= Duration::from_millis(1002)));
        assert!((stop_gh >= 998) && (stop_gh <= 1002));
    }

    #[test]
    fn sanity_test_330() {
        let timer: Timer<330> = Timer::new();
        let start_gh = timer.get_ticks();
        let start = Instant::now();
        sleep(Duration::from_secs(1));
        let stop_gh = timer.millis_since(start_gh);
        let stop = start.elapsed();

        assert!((stop >= Duration::from_millis(998)) && (stop <= Duration::from_millis(1002)));
        assert!((stop_gh >= 998) && (stop_gh <= 1002));
    }
}
