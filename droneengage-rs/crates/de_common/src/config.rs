// Ported from droneengage_common@5fb11c2: de_databus/configFile.cpp, configFile.hpp,
// localConfigFile.cpp, localConfigFile.hpp
//
// C++ keeps these as process-wide singletons. Here they are plain structs;
// binaries wrap them in Arc<Mutex<..>> where they need sharing.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::{Map, Value};

use crate::helpers::remove_comments;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read config file {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write config file {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid JSON in {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

/// Pretty print like nlohmann `dump(4)`: 4-space indent, sorted keys.
fn dump_pretty4(v: &Value) -> String {
    let mut out = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut out, fmt);
    if serde::Serialize::serialize(v, &mut ser).is_err() {
        return String::new();
    }
    String::from_utf8(out).unwrap_or_default()
}

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Module config file (`de_mavlink.config.module.json` etc.).
///
/// Holds three layers like C++: `base` (what is on disk and what gets saved),
/// `local_overrides` (from the .local file), and `config` = base + overrides.
#[derive(Debug)]
pub struct ConfigFile {
    path: PathBuf,
    config: Value,
    base: Value,
    local_overrides: Value,
    last_write: Option<SystemTime>,
    update_pending: bool,
}

impl ConfigFile {
    /// Read and parse `path` (comments allowed). C++ exits the process on
    /// failure; we return the error and let the binary exit.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let config = Self::read_parse(&path)?;
        Ok(Self {
            last_write: mtime(&path),
            base: config.clone(),
            config,
            path,
            local_overrides: Value::Null,
            update_pending: false,
        })
    }

    fn read_parse(path: &Path) -> Result<Value, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_str(&remove_comments(&text)).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn json(&self) -> &Value {
        &self.config
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Re-read the file from disk, then re-apply the last local overrides.
    pub fn reload(&mut self) -> Result<(), ConfigError> {
        self.update_pending = false;
        self.config = Self::read_parse(&self.path)?;
        self.base = self.config.clone();
        if !self.local_overrides.is_null() {
            let o = self.local_overrides.clone();
            self.apply_local_overrides(&o);
        }
        self.last_write = mtime(&self.path);
        Ok(())
    }

    /// True once after `update_json`, or when the file's mtime changed.
    pub fn file_updated(&mut self) -> bool {
        if self.update_pending {
            self.update_pending = false;
            return true;
        }
        match mtime(&self.path) {
            Some(t) if Some(t) != self.last_write => {
                self.last_write = Some(t);
                true
            }
            _ => false,
        }
    }

    /// Apply `{"key": v, "a.b.c": v}` updates (dotted keys create nested
    /// objects) to both runtime and base config, then save.
    pub fn update_json(&mut self, json_text: &str) {
        self.update_pending = true;
        let update: Value = match serde_json::from_str(&remove_comments(json_text)) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "config: failed to parse update JSON");
                return;
            }
        };
        apply_update(&mut self.config, &update);
        apply_update(&mut self.base, &update);
        if let Err(e) = self.save() {
            tracing::error!(error = %e, "config: save failed");
        }
    }

    /// Deep-merge `local` over the base config (objects merge, other values replace).
    pub fn apply_local_overrides(&mut self, local: &Value) {
        self.local_overrides = local.clone();
        let empty = match local {
            Value::Null => true,
            Value::Object(m) => m.is_empty(),
            Value::Array(a) => a.is_empty(),
            _ => false,
        };
        if empty {
            return;
        }
        self.config = self.base.clone();
        deep_merge(&mut self.config, local);
        tracing::info!("config: applied local overrides");
    }

    /// Write `base` (not the overrides) with a timestamped backup of the old file.
    pub fn save(&mut self) -> Result<(), ConfigError> {
        if self.path.exists() {
            // DIVERGENCE: backup suffix uses UTC (C++ uses local time); no wire effect.
            let backup = PathBuf::from(format!("{}.bak_{}", self.path.display(), utc_stamp()));
            if let Err(e) = std::fs::copy(&self.path, &backup) {
                tracing::warn!(error = %e, backup = %backup.display(), "config: backup failed");
            }
        }
        std::fs::write(&self.path, dump_pretty4(&self.base)).map_err(|source| {
            ConfigError::Write {
                path: self.path.clone(),
                source,
            }
        })?;
        self.last_write = mtime(&self.path);
        Ok(())
    }
}

fn apply_update(target: &mut Value, update: &Value) {
    let Some(items) = update.as_object() else {
        return;
    };
    for (key, value) in items {
        if key.contains('.') {
            let parts: Vec<&str> = key.split('.').filter(|p| !p.is_empty()).collect();
            let Some((last, path)) = parts.split_last() else {
                tracing::error!(%key, "config: invalid key format");
                continue;
            };
            let mut node = &mut *target;
            for p in path {
                node = index_or_create(node, p);
            }
            *index_or_create(node, last) = value.clone();
        } else {
            *index_or_create(target, key) = value.clone();
        }
    }
}

/// nlohmann `operator[]` on a mutable value: null becomes an object, a
/// missing key is inserted as null.
fn index_or_create<'a>(v: &'a mut Value, key: &str) -> &'a mut Value {
    if !v.is_object() {
        *v = Value::Object(Map::new());
    }
    match v {
        Value::Object(m) => m.entry(key.to_string()).or_insert(Value::Null),
        _ => unreachable!("just made it an object"),
    }
}

fn deep_merge(target: &mut Value, overrides: &Value) {
    let Some(items) = overrides.as_object() else {
        *target = overrides.clone();
        return;
    };
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    for (k, v) in items {
        let slot = index_or_create(target, k);
        if slot.is_object() && v.is_object() {
            deep_merge(slot, v);
        } else {
            *slot = v.clone();
        }
    }
}

fn utc_stamp() -> String {
    let secs = crate::helpers::unix_time_sec();
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // civil-from-days (Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}{m:02}{d:02}_{:02}{:02}{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// The module's private `.local` file (module_key and other persisted values).
#[derive(Debug)]
pub struct LocalConfigFile {
    path: PathBuf,
    config: Value,
}

impl LocalConfigFile {
    /// Read `path`, creating it when missing. Trailing commas are tolerated.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let mut me = Self {
            path,
            config: Value::Null,
        };
        match std::fs::read_to_string(&me.path) {
            Ok(text) => {
                me.config = serde_json::from_str(&strip_trailing_commas(&remove_comments(&text)))
                    .map_err(|source| ConfigError::Parse {
                    path: me.path.clone(),
                    source,
                })?;
            }
            Err(_) => {
                tracing::info!(path = %me.path.display(), "local config missing, creating one");
                me.write()?;
            }
        }
        Ok(me)
    }

    pub fn json(&self) -> &Value {
        &self.config
    }

    /// Persist current values.
    pub fn apply(&self) -> Result<(), ConfigError> {
        self.write()
    }

    pub fn clear(&mut self) -> Result<(), ConfigError> {
        self.config = Value::Null;
        self.write()
    }

    fn write(&self) -> Result<(), ConfigError> {
        std::fs::write(&self.path, crate::json::dump(&self.config)).map_err(|source| {
            ConfigError::Write {
                path: self.path.clone(),
                source,
            }
        })
    }

    /// Missing key -> "" (C++ returns an empty string).
    pub fn get_string(&self, field: &str) -> String {
        self.config
            .get(field)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    }

    pub fn set_string(&mut self, field: &str, value: &str) {
        *index_or_create(&mut self.config, field) = Value::String(value.to_string());
    }

    /// Missing key -> 0xFFFFFFFF (C++ sentinel).
    pub fn get_numeric(&self, field: &str) -> u32 {
        match self.config.get(field) {
            None => 0xFFFF_FFFF,
            Some(v) => crate::json::as_cpp_int(v)
                .map(|i| i as u32)
                .unwrap_or(0xFFFF_FFFF),
        }
    }

    pub fn set_numeric(&mut self, field: &str, value: u32) {
        *index_or_create(&mut self.config, field) = Value::from(value);
    }

    /// Missing or non-numeric -> 0.0.
    pub fn get_double(&self, field: &str) -> f64 {
        self.config
            .get(field)
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    }

    pub fn set_double(&mut self, field: &str, value: f64) {
        *index_or_create(&mut self.config, field) = Value::from(value);
    }

    pub fn remove(&mut self, field: &str) {
        if let Value::Object(m) = &mut self.config {
            m.remove(field);
        }
    }
}

/// Drop a ',' when the next non-space char is '}' or ']'.
fn strip_trailing_commas(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    for i in 0..b.len() {
        if b[i] == b',' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < b.len() && (b[j] == b'}' || b[j] == b']') {
                continue;
            }
        }
        out.push(b[i]);
    }
    String::from_utf8(out).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn update_with_dotted_keys() {
        let mut v = json!({"a": 1, "n": {"x": 1}});
        apply_update(&mut v, &json!({"n.y.z": 2, "a": 3}));
        assert_eq!(v, json!({"a": 3, "n": {"x": 1, "y": {"z": 2}}}));
    }

    #[test]
    fn local_overrides_deep_merge_and_save_keeps_base() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("m.json");
        std::fs::write(&p, "{ // c\n \"a\": {\"x\":1,\"y\":2}, \"b\": 1 }").unwrap();
        let mut c = ConfigFile::load(&p).unwrap();
        c.apply_local_overrides(&json!({"a": {"y": 5}}));
        assert_eq!(c.json(), &json!({"a": {"x": 1, "y": 5}, "b": 1}));
        c.update_json(r#"{"b": 2}"#);
        let saved: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(saved, json!({"a": {"x": 1, "y": 2}, "b": 2}));
        assert!(c.file_updated());
    }

    #[test]
    fn pretty_dump_matches_nlohmann_dump4() {
        let v = json!({"b": [1, 2], "a": {}, "c": []});
        assert_eq!(
            dump_pretty4(&v),
            "{\n    \"a\": {},\n    \"b\": [\n        1,\n        2\n    ],\n    \"c\": []\n}"
        );
    }

    #[test]
    fn local_file_created_and_trailing_commas() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.local");
        let mut l = LocalConfigFile::load(&p).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "null");
        assert_eq!(l.get_numeric("k"), 0xFFFF_FFFF);
        l.set_string("module_key", "123");
        l.apply().unwrap();
        std::fs::write(&p, "{\"module_key\":\"123\", \"n\": 7,}").unwrap();
        let l = LocalConfigFile::load(&p).unwrap();
        assert_eq!(l.get_string("module_key"), "123");
        assert_eq!(l.get_numeric("n"), 7);
    }

    #[test]
    fn utc_stamp_format() {
        let s = utc_stamp();
        assert_eq!(s.len(), 15);
        assert_eq!(&s[8..9], "_");
    }
}
