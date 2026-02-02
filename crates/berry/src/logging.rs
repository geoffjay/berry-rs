//! Logging configuration for the Berry system.
//!
//! Uses tracing with tracing-subscriber for structured logging.

use std::env;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Log format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Human-readable text format (default)
    #[default]
    Text,
    /// JSON format for machine processing
    Json,
}

impl LogFormat {
    /// Parse log format from environment variable BERRY_LOG_FORMAT.
    pub fn from_env() -> Self {
        match env::var("BERRY_LOG_FORMAT")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "json" => LogFormat::Json,
            _ => LogFormat::Text,
        }
    }
}

/// Configuration for logging initialization.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level filter (e.g., "info", "debug", "berry=debug,tower=warn")
    pub level: String,
    /// Output format
    pub format: LogFormat,
    /// Whether to include file/line info in logs
    pub include_location: bool,
    /// Whether to include target (module path) in logs
    pub include_target: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: env::var("BERRY_LOG").unwrap_or_else(|_| "info".to_string()),
            format: LogFormat::from_env(),
            include_location: false,
            include_target: true,
        }
    }
}

impl LogConfig {
    /// Create a new log configuration with the specified level.
    pub fn with_level(level: impl Into<String>) -> Self {
        Self {
            level: level.into(),
            ..Default::default()
        }
    }

    /// Set the log format.
    pub fn format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable file/line location in logs.
    pub fn with_location(mut self) -> Self {
        self.include_location = true;
        self
    }
}

/// Initialize the logging system with default configuration.
///
/// Uses BERRY_LOG environment variable for level (defaults to "info")
/// and BERRY_LOG_FORMAT for format (defaults to "text").
pub fn init() {
    init_with_config(LogConfig::default());
}

/// Initialize the logging system with the given configuration.
pub fn init_with_config(config: LogConfig) {
    let filter = EnvFilter::try_new(&config.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    match config.format {
        LogFormat::Text => {
            let subscriber = tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_file(config.include_location)
                        .with_line_number(config.include_location)
                        .with_target(config.include_target),
                );
            if tracing::subscriber::set_global_default(subscriber).is_err() {
                // Subscriber already set, which is fine
            }
        }
        LogFormat::Json => {
            let subscriber = tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().json());
            if tracing::subscriber::set_global_default(subscriber).is_err() {
                // Subscriber already set, which is fine
            }
        }
    }
}

/// Initialize logging for tests (captures output).
#[cfg(test)]
pub fn init_test() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_default() {
        // Clear env var for test - unsafe in Rust 2024 due to potential data races
        // SAFETY: Tests run single-threaded in this context
        unsafe { env::remove_var("BERRY_LOG_FORMAT") };
        assert_eq!(LogFormat::from_env(), LogFormat::Text);
    }

    #[test]
    fn test_log_config_default() {
        // SAFETY: Tests run single-threaded in this context
        unsafe {
            env::remove_var("BERRY_LOG");
            env::remove_var("BERRY_LOG_FORMAT");
        }
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, LogFormat::Text);
    }

    #[test]
    fn test_log_config_builder() {
        let config = LogConfig::with_level("debug")
            .format(LogFormat::Json)
            .with_location();

        assert_eq!(config.level, "debug");
        assert_eq!(config.format, LogFormat::Json);
        assert!(config.include_location);
    }
}
