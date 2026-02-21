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
        match self.minute_start {
            None => {
                self.minute_start = Some(now);
            }
            Some(start) if start.elapsed() >= MINUTE => {
                self.finalize_minute();
                self.minute_start = Some(now);
            }
            Some(_) => {}
        }
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
