// Ported from droneengage_common@5fb11c2: de_databus/de_cpu_monitor.cpp, de_cpu_monitor.hpp

use serde_json::{json, Value};

/// "cpu_monitor" block of a module config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuMonitorConfig {
    pub enabled: bool,
    pub super_busy_percent: i32,
    pub free_percent: i32,
    pub sustain_s: i32,
    pub report_s: i32,
    pub summary_s: i32,
}

impl Default for CpuMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            super_busy_percent: 90,
            free_percent: 70,
            sustain_s: 5,
            report_s: 10,
            summary_s: 60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuBusyState {
    Free = 0,
    SuperBusy = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuTransition {
    None,
    BecameSuperBusy,
    BecameFree,
}

/// Aggregate "cpu " line of /proc/stat.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CpuTimes {
    pub total: u64,
    pub idle_all: u64,
}

/// utime + stime from /proc/self/stat (jiffies).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProcCpuTimes {
    pub busy_jiffies: u64,
}

pub fn parse_proc_stat_cpu(text: &str) -> Option<CpuTimes> {
    let line = text.lines().find(|l| l.starts_with("cpu "))?;
    // C++ reads up to 16 numbers and stops at the first non-number.
    let values: Vec<u64> = line[4..]
        .split_whitespace()
        .map_while(|t| t.parse().ok())
        .take(16)
        .collect();
    if values.len() < 4 {
        return None;
    }
    Some(CpuTimes {
        total: values.iter().sum(),
        idle_all: values[3] + values.get(4).copied().unwrap_or(0),
    })
}

/// Busy percentage between two samples; -1 when the counter did not advance.
pub fn cpu_usage_percent(prev: &CpuTimes, cur: &CpuTimes) -> f64 {
    if cur.total <= prev.total {
        return -1.0;
    }
    let total = cur.total - prev.total;
    let idle = cur.idle_all.wrapping_sub(prev.idle_all);
    let busy = total.saturating_sub(idle);
    100.0 * busy as f64 / total as f64
}

pub fn parse_proc_self_stat_cpu(text: &str) -> Option<ProcCpuTimes> {
    let after = &text[text.rfind(')')? + 1..];
    let mut it = after.split_whitespace();
    it.next()?; // state
    let values: Vec<u64> = it.map_while(|t| t.parse().ok()).take(16).collect();
    if values.len() < 12 {
        return None;
    }
    Some(ProcCpuTimes {
        busy_jiffies: values[10] + values[11],
    })
}

pub fn process_cpu_usage_percent(
    prev: &ProcCpuTimes,
    cur: &ProcCpuTimes,
    elapsed_ms: u64,
    ticks_per_second: i64,
) -> f64 {
    if elapsed_ms == 0 || ticks_per_second <= 0 || cur.busy_jiffies <= prev.busy_jiffies {
        return -1.0;
    }
    let busy_s = (cur.busy_jiffies - prev.busy_jiffies) as f64 / ticks_per_second as f64;
    100.0 * busy_s / (elapsed_ms as f64 / 1000.0)
}

/// Read the optional "cpu_monitor" object; invalid thresholds fall back to defaults.
pub fn parse_cpu_monitor_config(module_config: &Value) -> CpuMonitorConfig {
    let mut cfg = CpuMonitorConfig::default();
    let Some(src) = module_config.get("cpu_monitor").filter(|v| v.is_object()) else {
        return cfg;
    };
    if let Some(b) = src.get("enabled").and_then(Value::as_bool) {
        cfg.enabled = b;
    }
    let num = |k: &str| src.get(k).and_then(crate::json::as_cpp_int);
    if let Some(v) = num("super_busy_percent") {
        cfg.super_busy_percent = v;
    }
    if let Some(v) = num("free_percent") {
        cfg.free_percent = v;
    }
    if let Some(v) = num("sustain_s") {
        cfg.sustain_s = v;
    }
    if let Some(v) = num("report_s") {
        cfg.report_s = v;
    }
    if let Some(v) = num("summary_s") {
        cfg.summary_s = v;
    }
    let bad = !(1..=100).contains(&cfg.super_busy_percent)
        || !(0..=100).contains(&cfg.free_percent)
        || cfg.sustain_s < 1
        || cfg.report_s < 1
        || cfg.summary_s < 0;
    if bad || cfg.free_percent >= cfg.super_busy_percent {
        tracing::warn!("cpu_monitor: invalid thresholds, falling back to defaults");
        return CpuMonitorConfig::default();
    }
    cfg
}

fn read_text(env_override: &str, default: &str) -> Option<String> {
    let path = std::env::var(env_override).ok().filter(|p| !p.is_empty());
    std::fs::read_to_string(path.as_deref().unwrap_or(default)).ok()
}

pub fn read_proc_stat_cpu() -> Option<CpuTimes> {
    parse_proc_stat_cpu(&read_text("DE_PROC_STAT_PATH", "/proc/stat")?)
}

pub fn read_proc_self_stat_cpu() -> Option<ProcCpuTimes> {
    parse_proc_self_stat_cpu(&read_text("DE_PROC_SELF_STAT_PATH", "/proc/self/stat")?)
}

/// `$DE_BOARD_ID`, else /etc/machine-id, else the hostname.
pub fn board_id() -> String {
    if let Ok(id) = std::env::var("DE_BOARD_ID") {
        if !id.is_empty() {
            return id;
        }
    }
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        let id = id.lines().next().unwrap_or("").to_string();
        if !id.is_empty() {
            return id;
        }
    }
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|h| h.trim().to_string())
        .unwrap_or_default()
}

/// Payload of TYPE_AndruavMessage_MODULE_CPU_STATUS (6547).
pub fn build_cpu_status_json(
    board_id: &str,
    board_usage: i32,
    process_usage: i32,
    board_state: i32,
    report_event: i32,
) -> Value {
    json!({
        "bd": board_id,
        "u": board_usage.clamp(0, 100),
        "pu": process_usage.clamp(0, 100),
        "s": board_state,
        "ev": report_event,
    })
}

/// Busy/free hysteresis on the board CPU usage.
#[derive(Debug, Clone)]
pub struct CpuMonitor {
    config: CpuMonitorConfig,
    state: CpuBusyState,
    prev: Option<CpuTimes>,
    last_usage: f64,
    pending_dir: i32,
    pending_since_ms: u64,
}

impl CpuMonitor {
    pub fn new(config: CpuMonitorConfig) -> Self {
        Self {
            config,
            state: CpuBusyState::Free,
            prev: None,
            last_usage: -1.0,
            pending_dir: 0,
            pending_since_ms: 0,
        }
    }

    pub fn configure(&mut self, config: CpuMonitorConfig) {
        self.config = config;
        self.pending_dir = 0;
        self.pending_since_ms = 0;
    }

    pub fn state(&self) -> CpuBusyState {
        self.state
    }

    pub fn last_usage_percent(&self) -> f64 {
        self.last_usage
    }

    pub fn config(&self) -> &CpuMonitorConfig {
        &self.config
    }

    pub fn update_from_times(&mut self, cur: CpuTimes, now_ms: u64) -> CpuTransition {
        let Some(prev) = self.prev else {
            self.prev = Some(cur);
            return CpuTransition::None;
        };
        let usage = cpu_usage_percent(&prev, &cur);
        if usage < 0.0 {
            return CpuTransition::None;
        }
        self.prev = Some(cur);
        self.last_usage = usage;
        self.on_usage(usage, now_ms)
    }

    fn on_usage(&mut self, usage: f64, now_ms: u64) -> CpuTransition {
        let mut want = 0;
        if self.state == CpuBusyState::Free && usage >= f64::from(self.config.super_busy_percent) {
            want = 1;
        }
        if self.state == CpuBusyState::SuperBusy && usage <= f64::from(self.config.free_percent) {
            want = -1;
        }
        if want == 0 {
            self.pending_dir = 0;
            self.pending_since_ms = 0;
            return CpuTransition::None;
        }
        if self.pending_dir != want {
            self.pending_dir = want;
            self.pending_since_ms = now_ms;
            return CpuTransition::None;
        }
        let held = now_ms.saturating_sub(self.pending_since_ms);
        if held < self.config.sustain_s as u64 * 1000 {
            return CpuTransition::None;
        }
        self.pending_dir = 0;
        self.pending_since_ms = 0;
        if want > 0 {
            self.state = CpuBusyState::SuperBusy;
            CpuTransition::BecameSuperBusy
        } else {
            self.state = CpuBusyState::Free;
            CpuTransition::BecameFree
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_stat_parsing() {
        let t = parse_proc_stat_cpu("cpu  10 0 10 70 10 0 0 0 0 0\ncpu0 1 2 3 4\n").unwrap();
        assert_eq!(
            t,
            CpuTimes {
                total: 100,
                idle_all: 80
            }
        );
        assert!(parse_proc_stat_cpu("cpu0 1 2 3 4\n").is_none());
    }

    #[test]
    fn proc_self_stat_handles_spaces_in_comm() {
        let s = "42 (de ardu pilot) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15";
        // after state: 1 2 3 4 5 6 7 8 9 10 [11]=utime [12]=stime
        assert_eq!(parse_proc_self_stat_cpu(s).unwrap().busy_jiffies, 11 + 12);
    }

    #[test]
    fn hysteresis_needs_sustain() {
        let mut m = CpuMonitor::new(CpuMonitorConfig::default());
        let mut total = 0;
        let mut idle = 0;
        let mut step = |m: &mut CpuMonitor, busy_pct: u64, t: u64| {
            total += 100;
            idle += 100 - busy_pct;
            m.update_from_times(
                CpuTimes {
                    total,
                    idle_all: idle,
                },
                t,
            )
        };
        assert_eq!(step(&mut m, 0, 0), CpuTransition::None); // seed
        assert_eq!(step(&mut m, 95, 1000), CpuTransition::None); // pending
        assert_eq!(step(&mut m, 95, 5000), CpuTransition::None); // 4 s
        assert_eq!(step(&mut m, 95, 6000), CpuTransition::BecameSuperBusy);
        assert_eq!(m.state(), CpuBusyState::SuperBusy);
    }

    #[test]
    fn bad_thresholds_fall_back() {
        let c = parse_cpu_monitor_config(&json!({"cpu_monitor": {"free_percent": 95}}));
        assert_eq!(c, CpuMonitorConfig::default());
        let c =
            parse_cpu_monitor_config(&json!({"cpu_monitor": {"enabled": false, "report_s": 3}}));
        assert!(!c.enabled);
        assert_eq!(c.report_s, 3);
    }

    #[test]
    fn status_json_clamps() {
        assert_eq!(
            crate::json::dump(&build_cpu_status_json("GTR", 120, -3, 0, 0)),
            r#"{"bd":"GTR","ev":0,"pu":0,"s":0,"u":100}"#
        );
    }
}
