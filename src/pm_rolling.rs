use esp_hal::time::{Duration, Instant};

const WINDOW_MINUTES: usize = 24 * 60;
const MINUTE: Duration = Duration::from_secs(60);

#[derive(Clone, Copy)]
pub struct PmAtmAverages {
    pub pm1_0: u16,
    pub pm2_5: u16,
    pub pm10: u16,
}

pub struct Pm24hRollingAverage {
    pm1_window: [u16; WINDOW_MINUTES],
    pm25_window: [u16; WINDOW_MINUTES],
    pm10_window: [u16; WINDOW_MINUTES],
    window_len: usize,
    window_pos: usize,
    sum_pm1: u64,
    sum_pm25: u64,
    sum_pm10: u64,
    minute_start: Option<Instant>,
    minute_sum_pm1: u32,
    minute_sum_pm25: u32,
    minute_sum_pm10: u32,
    minute_count: u16,
}

impl Pm24hRollingAverage {
    pub const fn new() -> Self {
        Self {
            pm1_window: [0; WINDOW_MINUTES],
            pm25_window: [0; WINDOW_MINUTES],
            pm10_window: [0; WINDOW_MINUTES],
            window_len: 0,
            window_pos: 0,
            sum_pm1: 0,
            sum_pm25: 0,
            sum_pm10: 0,
            minute_start: None,
            minute_sum_pm1: 0,
            minute_sum_pm25: 0,
            minute_sum_pm10: 0,
            minute_count: 0,
        }
    }

    pub fn update(&mut self, pm1: u16, pm25: u16, pm10: u16, now: Instant) -> PmAtmAverages {
        self.roll_minute_if_needed(now);

        self.minute_sum_pm1 = self.minute_sum_pm1.saturating_add(pm1 as u32);
        self.minute_sum_pm25 = self.minute_sum_pm25.saturating_add(pm25 as u32);
        self.minute_sum_pm10 = self.minute_sum_pm10.saturating_add(pm10 as u32);
        self.minute_count = self.minute_count.saturating_add(1);

        self.current_average()
    }

    fn roll_minute_if_needed(&mut self, now: Instant) {
        let start = match self.minute_start {
            None => {
                self.minute_start = Some(now);
                return;
            }
            Some(s) => s,
        };

        let elapsed = start.elapsed();
        if elapsed < MINUTE {
            return;
        }

        // Finalize whatever accumulated in the current minute bucket.
        self.finalize_minute();

        // For each additional minute that passed with no sensor data, evict the
        // corresponding oldest stored bucket so the window stays time-correct
        // across gaps (sensor disconnects, power interruptions, etc.).
        let missed = missed_bucket_count(elapsed, self.window_len);
        for _ in 0..missed {
            self.evict_oldest();
        }

        self.minute_start = Some(now);
    }

    // Remove the oldest minute bucket from the ring buffer without adding new
    // data. Used to advance the window during periods with no sensor readings.
    fn evict_oldest(&mut self) {
        if self.window_len == 0 {
            return;
        }
        let oldest = (self.window_pos + WINDOW_MINUTES - self.window_len) % WINDOW_MINUTES;
        self.sum_pm1 = self.sum_pm1.saturating_sub(self.pm1_window[oldest] as u64);
        self.sum_pm25 = self
            .sum_pm25
            .saturating_sub(self.pm25_window[oldest] as u64);
        self.sum_pm10 = self
            .sum_pm10
            .saturating_sub(self.pm10_window[oldest] as u64);
        self.window_len -= 1;
    }

    fn finalize_minute(&mut self) {
        if self.minute_count == 0 {
            return;
        }

        let count = self.minute_count as u32;
        let pm1_mean = ((self.minute_sum_pm1 + count / 2) / count) as u16;
        let pm25_mean = ((self.minute_sum_pm25 + count / 2) / count) as u16;
        let pm10_mean = ((self.minute_sum_pm10 + count / 2) / count) as u16;
        self.push_minute(pm1_mean, pm25_mean, pm10_mean);

        self.minute_sum_pm1 = 0;
        self.minute_sum_pm25 = 0;
        self.minute_sum_pm10 = 0;
        self.minute_count = 0;
    }

    fn push_minute(&mut self, pm1: u16, pm25: u16, pm10: u16) {
        if self.window_len == WINDOW_MINUTES {
            let idx = self.window_pos;
            self.sum_pm1 = self.sum_pm1.saturating_sub(self.pm1_window[idx] as u64);
            self.sum_pm25 = self.sum_pm25.saturating_sub(self.pm25_window[idx] as u64);
            self.sum_pm10 = self.sum_pm10.saturating_sub(self.pm10_window[idx] as u64);
        } else {
            self.window_len += 1;
        }

        self.pm1_window[self.window_pos] = pm1;
        self.pm25_window[self.window_pos] = pm25;
        self.pm10_window[self.window_pos] = pm10;
        self.sum_pm1 = self.sum_pm1.saturating_add(pm1 as u64);
        self.sum_pm25 = self.sum_pm25.saturating_add(pm25 as u64);
        self.sum_pm10 = self.sum_pm10.saturating_add(pm10 as u64);
        self.window_pos = (self.window_pos + 1) % WINDOW_MINUTES;
    }

    fn current_average(&self) -> PmAtmAverages {
        if self.minute_count == 0 {
            return self.stored_average();
        }

        let count = self.minute_count as u32;
        let curr_pm1 = ((self.minute_sum_pm1 + count / 2) / count) as u64;
        let curr_pm25 = ((self.minute_sum_pm25 + count / 2) / count) as u64;
        let curr_pm10 = ((self.minute_sum_pm10 + count / 2) / count) as u64;
        let denom = (self.window_len as u64) + 1;

        PmAtmAverages {
            pm1_0: ((self.sum_pm1 + curr_pm1 + denom / 2) / denom) as u16,
            pm2_5: ((self.sum_pm25 + curr_pm25 + denom / 2) / denom) as u16,
            pm10: ((self.sum_pm10 + curr_pm10 + denom / 2) / denom) as u16,
        }
    }

    fn stored_average(&self) -> PmAtmAverages {
        if self.window_len == 0 {
            return PmAtmAverages {
                pm1_0: 0,
                pm2_5: 0,
                pm10: 0,
            };
        }

        let denom = self.window_len as u64;
        PmAtmAverages {
            pm1_0: ((self.sum_pm1 + denom / 2) / denom) as u16,
            pm2_5: ((self.sum_pm25 + denom / 2) / denom) as u16,
            pm10: ((self.sum_pm10 + denom / 2) / denom) as u16,
        }
    }
}

fn elapsed_complete_minutes(elapsed: Duration) -> usize {
    // Count the number of complete minutes that elapsed. Using a comparison
    // loop avoids depending on Duration::to_millis() availability.
    let mut elapsed_mins: usize = 1;
    while elapsed >= MINUTE * ((elapsed_mins + 1) as u32) && elapsed_mins < WINDOW_MINUTES {
        elapsed_mins += 1;
    }
    elapsed_mins
}

fn missed_bucket_count(elapsed: Duration, window_len: usize) -> usize {
    debug_assert!(elapsed >= MINUTE);

    // When the gap is strictly greater than the full window, evict everything
    // in the ring (including any entry just finalized above) so no pre-gap
    // data survives. For shorter gaps, keep the just-finalized newest entry.
    if elapsed > MINUTE * WINDOW_MINUTES as u32 {
        return window_len;
    }

    elapsed_complete_minutes(elapsed)
        .saturating_sub(1)
        .min(window_len.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minutes(count: u32) -> Duration {
        MINUTE * count
    }

    #[test]
    fn exact_one_minute_gap_only_finalizes_current_bucket() {
        assert_eq!(missed_bucket_count(minutes(1), 5), 0);
    }

    #[test]
    fn exact_full_window_gap_keeps_newest_bucket() {
        assert_eq!(
            missed_bucket_count(minutes(WINDOW_MINUTES as u32), WINDOW_MINUTES),
            WINDOW_MINUTES - 1
        );
    }

    #[test]
    fn gap_longer_than_full_window_evicts_everything() {
        assert_eq!(
            missed_bucket_count(minutes(WINDOW_MINUTES as u32) + Duration::from_secs(1), 12),
            12
        );
    }

    #[test]
    fn short_history_gap_does_not_evict_just_finalized_bucket() {
        assert_eq!(missed_bucket_count(minutes(WINDOW_MINUTES as u32), 10), 9);
    }

    #[test]
    fn evict_oldest_removes_wrapped_ring_head() {
        let mut rolling = Pm24hRollingAverage::new();
        rolling.window_pos = WINDOW_MINUTES - 1;

        rolling.push_minute(1, 10, 100);
        rolling.push_minute(2, 20, 200);
        rolling.push_minute(3, 30, 300);

        rolling.evict_oldest();

        assert_eq!(rolling.window_len, 2);
        assert_eq!(rolling.sum_pm1, 5);
        assert_eq!(rolling.sum_pm25, 50);
        assert_eq!(rolling.sum_pm10, 500);
        assert_eq!(rolling.stored_average().pm1_0, 3);
        assert_eq!(rolling.stored_average().pm2_5, 25);
        assert_eq!(rolling.stored_average().pm10, 250);
    }
}
