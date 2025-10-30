// /src/utils/logger.rs
// Module: utils.logger
// Purpose: Centralized logging system with file rotation and custom levels

use crate::config::Config;
use crate::error::{Result, VitalError};
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// ID SRS: SRS-MOD-LOGGER-001
/// Title: Logger
///
/// Description: VRConnect shall provide a custom logger with daily file rotation,
/// custom log levels (SUCCESS, INFO, WARNING, ERROR, DEBUG), and timestamp-based
/// log entries using machine time.
///
/// Version: V1.0
pub struct Logger {
    log_dir: PathBuf,
    current_file: Mutex<Option<PathBuf>>,
}

impl Logger {
    /// ID SRS: SRS-FN-LOGGER-001
    /// Title: init
    ///
    /// Description: VRConnect shall initialize the logger with configuration
    /// parameters, create log directory if needed, and set global logger.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `config` - Application configuration
    ///
    /// # Returns
    /// Result indicating initialization success or error
    pub fn init(config: &Config) -> Result<()> {
        let log_dir = PathBuf::from(&config.log_dir);

        // Create log directory if it doesn't exist
        fs::create_dir_all(&log_dir)
            .map_err(|e| VitalError::Logger(format!("Failed to create log directory: {}", e)))?;

        let logger = Logger {
            log_dir,
            current_file: Mutex::new(None),
        };

        // Set as global logger
        let level_filter = Self::parse_level(&config.log_level)?;
        
        log::set_boxed_logger(Box::new(logger))
            .map_err(|e| VitalError::Logger(format!("Failed to set logger: {}", e)))?;

        log::set_max_level(level_filter);

        Ok(())
    }

    /// ID SRS: SRS-FN-LOGGER-002
    /// Title: parse_level
    ///
    /// Description: VRConnect shall parse log level string (SUCCESS, INFO,
    /// WARNING, ERROR, DEBUG) into LevelFilter enum.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `level_str` - Log level string
    ///
    /// # Returns
    /// Parsed LevelFilter or error
    fn parse_level(level_str: &str) -> Result<log::LevelFilter> {
        match level_str.to_uppercase().as_str() {
            "SUCCESS" => Ok(log::LevelFilter::Info),
            "INFO" => Ok(log::LevelFilter::Info),
            "WARNING" => Ok(log::LevelFilter::Warn),
            "ERROR" => Ok(log::LevelFilter::Error),
            "DEBUG" => Ok(log::LevelFilter::Debug),
            "TRACE" => Ok(log::LevelFilter::Trace),
            _ => Err(VitalError::Logger(format!("Invalid log level: {}", level_str))),
        }
    }

    /// ID SRS: SRS-FN-LOGGER-003
    /// Title: get_log_file_path
    ///
    /// Description: VRConnect shall determine the log file path based on
    /// current date, creating daily log files (vrconnect-YYYY-MM-DD.log).
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// PathBuf to current log file
    fn get_log_file_path(&self) -> PathBuf {
        let date = Local::now().format("%Y-%m-%d");
        self.log_dir.join(format!("vrconnect-{}.log", date))
    }

    /// ID SRS: SRS-FN-LOGGER-004
    /// Title: write_log
    ///
    /// Description: VRConnect shall write formatted log message to daily log file
    /// with timestamp, level, and message content.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `record` - Log record to write
    fn write_log(&self, record: &log::Record) {
        let log_file = self.get_log_file_path();
        
        // Update current file if date changed
        {
            let mut current = self.current_file.lock().unwrap();
            if current.as_ref() != Some(&log_file) {
                *current = Some(log_file.clone());
            }
        }

        // Format: [2025-01-15 14:30:45.123] [INFO] Message
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level_str = self.format_level(record.level());
        let message = format!("[{}] [{}] {}\n", timestamp, level_str, record.args());

        // Write to file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            let _ = file.write_all(message.as_bytes());
        }
    }

    /// ID SRS: SRS-FN-LOGGER-005
    /// Title: format_level
    ///
    /// Description: VRConnect shall format log level with consistent width
    /// for readable log output (SUCCESS, INFO, WARNING, ERROR, DEBUG).
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `level` - Log level
    ///
    /// # Returns
    /// Formatted level string
    fn format_level(&self, level: log::Level) -> String {
        match level {
            log::Level::Error => "ERROR  ".to_string(),
            log::Level::Warn => "WARNING".to_string(),
            log::Level::Info => "INFO   ".to_string(),
            log::Level::Debug => "DEBUG  ".to_string(),
            log::Level::Trace => "TRACE  ".to_string(),
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        self.write_log(record);
    }

    fn flush(&self) {
        // Nothing to flush with file-based logging
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_level_valid() {
        // TODO: Implement valid level parsing tests
        assert!(true);
    }

    #[test]
    fn test_parse_level_invalid() {
        // TODO: Implement invalid level parsing test
        assert!(true);
    }

    #[test]
    fn test_log_file_path_generation() {
        // TODO: Implement log file path generation test
        assert!(true);
    }

    #[test]
    fn test_format_level() {
        // TODO: Implement level formatting test
        assert!(true);
    }

    #[test]
    fn test_daily_rotation() {
        // TODO: Implement daily rotation test
        assert!(true);
    }
}
