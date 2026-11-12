//! Application log sink (decision 23, .dojo/CONTEXT.md).
//!
//! Code depends on the `LogSink` interface, never a destination. All records are JSON Lines.
//! If a remote sink is ever added, it must wrap a local-file fallback (dojo-principles → Logging).

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One structured log record. `ts` is UTC; `fields` carries event-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub ts: String,
    pub level: String,
    pub event: String,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub fields: Value,
}

/// Where records go. All options are JSON Lines; implementations must be thread-safe.
pub trait LogSink: Send + Sync {
    fn write(&self, record: &LogRecord) -> std::io::Result<()>;
}

/// JSON Lines text file (decision 23's chosen destination).
pub struct JsonlFileSink {
    path: std::path::PathBuf,
    lock: std::sync::Mutex<File>,
}

impl JsonlFileSink {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { path: path.to_path_buf(), lock: std::sync::Mutex::new(file) })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl LogSink for JsonlFileSink {
    fn write(&self, record: &LogRecord) -> std::io::Result<()> {
        let mut line = serde_json::to_string(record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        line.push('\n');
        let mut f = self.lock.lock().expect("log sink mutex poisoned");
        f.write_all(line.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_valid_jsonl_lines() {
        let dir = std::env::temp_dir().join(format!("koma-log-test-{}", std::process::id()));
        let path = dir.join("server.log");
        std::fs::remove_file(&path).unwrap_or(());
        let sink = JsonlFileSink::open(&path).unwrap();

        sink.write(&LogRecord {
            ts: "2026-01-01T00:00:00Z".into(),
            level: "info".into(),
            event: "boot".into(),
            fields: serde_json::json!({ "pid": 1 }),
        })
        .unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let line = text.lines().next().unwrap();
        let parsed: LogRecord = serde_json::from_str(line).unwrap();
        assert_eq!(parsed.event, "boot");
        assert_eq!(parsed.fields["pid"], 1);
        std::fs::remove_dir_all(dir).unwrap_or(());
    }
}
