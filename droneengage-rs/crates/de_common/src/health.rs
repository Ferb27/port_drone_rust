// Ported from droneengage_common@5fb11c2: de_databus/de_module_health.cpp, de_module_health.hpp

use std::collections::VecDeque;
use std::time::Instant;

use crate::messages::{
    MODULE_HEALTH_STATUS_CRITICAL, MODULE_HEALTH_STATUS_OK, MODULE_HEALTH_STATUS_WARNING,
    MODULE_HEALTH_TREND_DOWN, MODULE_HEALTH_TREND_STABLE, MODULE_HEALTH_TREND_UP,
};

/// One memory health sample (sent as TYPE_AndruavMessage_MODULE_HEALTH_STATUS).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HealthSample {
    pub rss_mb: f64,
    pub peak_rss_mb: f64,
    pub vmswap_mb: f64,
    pub threads: i32,
    pub slope_mb_h: f64,
    pub trend: i32,
    pub status: i32,
    pub uptime_sec: u64,
}

/// Raw values from /proc/self/status.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ProcStatus {
    pub rss_mb: f64,
    pub vmpeak_mb: f64,
    pub vmswap_mb: f64,
    pub threads: i32,
}

/// Parse /proc/self/status text (VmRSS, VmPeak, VmSwap in kB; Threads).
pub fn parse_proc_self_status(text: &str) -> ProcStatus {
    let mut s = ProcStatus::default();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let (Some(key), Some(val)) = (it.next(), it.next()) else {
            continue;
        };
        match key {
            "VmRSS:" => s.rss_mb = val.parse::<u64>().unwrap_or(0) as f64 / 1024.0,
            "VmPeak:" => s.vmpeak_mb = val.parse::<u64>().unwrap_or(0) as f64 / 1024.0,
            "VmSwap:" => s.vmswap_mb = val.parse::<u64>().unwrap_or(0) as f64 / 1024.0,
            "Threads:" => s.threads = val.parse().unwrap_or(0),
            _ => {}
        }
    }
    s
}

#[derive(Debug)]
pub struct HealthMonitor {
    history: VecDeque<(u64, f64)>, // (t usec, rss_mb)
    history_samples: usize,
    max_rss_mb: f64,
    max_growth_mb_per_hour: f64,
    start: Option<Instant>,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self {
            history: VecDeque::new(),
            history_samples: 40,
            max_rss_mb: 500.0,
            max_growth_mb_per_hour: 20.0,
            start: None,
        }
    }
}

impl HealthMonitor {
    pub fn configure(
        &mut self,
        max_rss_mb: f64,
        max_growth_mb_per_hour: f64,
        history_samples: usize,
    ) {
        self.max_rss_mb = max_rss_mb;
        self.max_growth_mb_per_hour = max_growth_mb_per_hour;
        self.history_samples = history_samples;
    }

    /// Sample this process. `None` when /proc/self/status cannot be read.
    pub fn sample(&mut self) -> Option<HealthSample> {
        let text = std::fs::read_to_string("/proc/self/status").ok()?;
        Some(self.sample_from(parse_proc_self_status(&text), Instant::now()))
    }

    /// Pure part of `sample`, testable with fake readings.
    pub fn sample_from(&mut self, p: ProcStatus, now: Instant) -> HealthSample {
        let start = *self.start.get_or_insert(now);
        let now_usec = now.duration_since(start).as_micros() as u64;
        self.history.push_back((now_usec, p.rss_mb));
        while self.history.len() > self.history_samples {
            self.history.pop_front();
        }
        let slope = self.slope();
        let peak = self.history.iter().map(|&(_, r)| r).fold(0.0, f64::max);
        let trend = if slope > 1.0 {
            MODULE_HEALTH_TREND_UP
        } else if slope < -1.0 {
            MODULE_HEALTH_TREND_DOWN
        } else {
            MODULE_HEALTH_TREND_STABLE
        };
        let status = if p.rss_mb > self.max_rss_mb {
            MODULE_HEALTH_STATUS_CRITICAL
        } else if self.history.len() >= self.history_samples && slope > self.max_growth_mb_per_hour
        {
            MODULE_HEALTH_STATUS_WARNING
        } else {
            MODULE_HEALTH_STATUS_OK
        };
        HealthSample {
            rss_mb: p.rss_mb,
            peak_rss_mb: peak,
            vmswap_mb: p.vmswap_mb,
            threads: p.threads,
            slope_mb_h: slope,
            trend,
            status,
            uptime_sec: now_usec / 1_000_000,
        }
    }

    /// Least-squares slope of RSS (MB) over time (hours).
    fn slope(&self) -> f64 {
        let n = self.history.len();
        if n < 2 {
            return 0.0;
        }
        let t0 = self.history[0].0;
        let (mut sx, mut sy, mut sxy, mut sxx) = (0.0, 0.0, 0.0, 0.0);
        for &(t, y) in &self.history {
            let x = (t - t0) as f64 / 3_600_000_000.0;
            sx += x;
            sy += y;
            sxy += x * y;
            sxx += x * x;
        }
        let n = n as f64;
        let denom = n * sxx - sx * sx;
        if denom == 0.0 {
            return 0.0;
        }
        (n * sxy - sx * sy) / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn parse_status() {
        let p = parse_proc_self_status(
            "Name:\tx\nVmPeak:\t  2048 kB\nVmRSS:\t 6508 kB\nVmSwap:\t0 kB\nThreads:\t6\n",
        );
        assert_eq!(p.threads, 6);
        assert!((p.rss_mb - 6.35546875).abs() < 1e-12);
        assert!((p.vmpeak_mb - 2.0).abs() < 1e-12);
    }

    #[test]
    fn growing_memory_trends_up_and_warns_after_full_window() {
        let mut m = HealthMonitor::default();
        m.configure(500.0, 20.0, 3);
        let t0 = Instant::now();
        let mut last = HealthSample::default();
        for i in 0..3u64 {
            let p = ProcStatus {
                rss_mb: 10.0 + i as f64 * 10.0,
                ..Default::default()
            };
            last = m.sample_from(p, t0 + Duration::from_secs(i * 60));
        }
        assert_eq!(last.trend, MODULE_HEALTH_TREND_UP);
        assert_eq!(last.status, MODULE_HEALTH_STATUS_WARNING);
        assert_eq!(last.uptime_sec, 120);
        assert_eq!(last.peak_rss_mb, 30.0);
    }

    #[test]
    fn first_sample_is_stable_ok() {
        let mut m = HealthMonitor::default();
        let s = m.sample_from(
            ProcStatus {
                rss_mb: 6.0,
                threads: 6,
                ..Default::default()
            },
            Instant::now(),
        );
        assert_eq!(
            (s.trend, s.status, s.slope_mb_h, s.uptime_sec),
            (0, 0, 0.0, 0)
        );
    }
}
