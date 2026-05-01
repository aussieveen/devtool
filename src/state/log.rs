use chrono::{DateTime, Local, TimeDelta};

const MAX_ENTRIES: usize = 500;
const EXPIRY_HOURS: i64 = 3;

// ── Log source constants ──────────────────────────────────────────────────────

pub mod log_source {
    pub const APP: &str = "App";
    pub const JIRA: &str = "Jira";
    pub const SERVICE_STATUS: &str = "Service Status";
    pub const TOKEN_GENERATOR: &str = "Token Generator";
}

// ── Activity feed ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ActivityEntry {
    pub timestamp: DateTime<Local>,
    pub source: String,
    pub message: String,
}

// ── App log ───────────────────────────────────────────────────────────────────

/// RFC 5424 severity levels.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

impl LogLevel {
    pub fn label(&self) -> &'static str {
        match self {
            LogLevel::Emergency => "EMERG ",
            LogLevel::Alert => "ALERT ",
            LogLevel::Critical => "CRIT  ",
            LogLevel::Error => "ERROR ",
            LogLevel::Warning => "WARN  ",
            LogLevel::Notice => "NOTICE",
            LogLevel::Info => "INFO  ",
            LogLevel::Debug => "DEBUG ",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppLogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub source: String,
    pub title: String,
    pub detail: Option<String>,
}

// ── LogEntry (dispatch struct, no timestamp) ──────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    pub level: LogLevel,
    pub source: String,
    pub title: String,
    pub detail: Option<String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, source: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            level,
            source: source.into(),
            title: title.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

// ── Log state ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogsItem {
    Activity,
    AppLog,
}

pub struct LogState {
    pub activity: Vec<ActivityEntry>,
    pub app_log: Vec<AppLogEntry>,
    pub selected_item: LogsItem,
    /// Number of activity entries the user has already seen (used for unread dot).
    pub activity_seen_count: usize,
}

impl LogState {
    pub fn new() -> Self {
        Self {
            activity: Vec::new(),
            app_log: Vec::new(),
            selected_item: LogsItem::Activity,
            activity_seen_count: 0,
        }
    }

    pub fn push_activity(&mut self, source: String, message: String) {
        self.activity.push(ActivityEntry {
            timestamp: Local::now(),
            source,
            message,
        });
        if self.activity.len() > MAX_ENTRIES {
            self.activity.remove(0);
            self.activity_seen_count = self.activity_seen_count.saturating_sub(1);
        }
    }

    pub fn push_log(&mut self, entry: LogEntry) {
        self.app_log.push(AppLogEntry {
            timestamp: Local::now(),
            level: entry.level,
            source: entry.source,
            title: entry.title,
            detail: entry.detail,
        });
        if self.app_log.len() > MAX_ENTRIES {
            self.app_log.remove(0);
        }
    }

    pub fn has_unread_activity(&self) -> bool {
        self.activity.len() > self.activity_seen_count
    }

    pub fn mark_activity_seen(&mut self) {
        self.activity_seen_count = self.activity.len();
    }

    /// Returns activity entries newer than 3 hours, newest first.
    pub fn visible_activity(&self) -> Vec<&ActivityEntry> {
        let cutoff = Local::now() - TimeDelta::hours(EXPIRY_HOURS);
        self.activity
            .iter()
            .rev()
            .filter(|e| e.timestamp > cutoff)
            .collect()
    }

    /// Returns log entries newer than 3 hours, newest first.
    pub fn visible_log(&self) -> Vec<&AppLogEntry> {
        let cutoff = Local::now() - TimeDelta::hours(EXPIRY_HOURS);
        self.app_log
            .iter()
            .rev()
            .filter(|e| e.timestamp > cutoff)
            .collect()
    }

    pub fn select_logs(&mut self) {
        self.selected_item = LogsItem::AppLog;
    }

    pub fn select_activity(&mut self) {
        self.selected_item = LogsItem::Activity;
    }
}
