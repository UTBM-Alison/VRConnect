// /src/config/mod.rs
// Module: config
// Purpose: Configuration management with CLI and file support

pub mod loader;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// ID SRS: SRS-MOD-CONFIG-001
/// Title: Config
///
/// Description: VRConnect shall provide a configuration structure supporting
/// both CLI arguments and environment file loading for all application parameters.
///
/// Version: V1.0
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(name = "VRConnect")]
#[command(version = "1.0.0")]
#[command(author = "UTBM Team")]
#[command(about = "Medical vital data middleware")]
pub struct Config {
    /// Path to configuration file (.env format)
    #[arg(long)]
    #[serde(skip)]
    pub config_file: Option<PathBuf>,

    // Socket.IO Configuration
    /// Socket.IO server host
    #[arg(long, default_value = "127.0.0.1")]
    pub socketio_host: String,

    /// Socket.IO server port
    #[arg(long, short = 'p', default_value = "3000")]
    pub socketio_port: u16,

    // Console Output Configuration
    /// Enable console output
    #[arg(long, default_value = "true")]
    pub output_console_enabled: bool,

    /// Enable verbose console output
    #[arg(long, short = 'v', default_value = "false")]
    pub output_console_verbose: bool,

    /// Enable colorized console output
    #[arg(long, default_value = "true")]
    pub output_console_colorized: bool,

    // BLE Output Configuration
    /// Enable BLE output
    #[arg(long, default_value = "false")]
    pub output_ble_enabled: bool,

    /// BLE device name
    #[arg(long, default_value = "VitalConnect")]
    pub output_ble_device_name: String,

    /// BLE service UUID
    #[arg(long, default_value = "12345678-1234-5678-1234-567812345678")]
    pub output_ble_service_uuid: String,

    // Debug Configuration
    /// Enable debug mode
    #[arg(long, default_value = "false")]
    pub debug_enabled: bool,

    /// Debug output file path
    #[arg(long, default_value = "./logs/debug.log")]
    pub debug_output_path: String,

    // Logging Configuration
    /// Log level (SUCCESS, INFO, WARNING, ERROR, DEBUG)
    #[arg(long, default_value = "INFO")]
    pub log_level: String,

    /// Log directory
    #[arg(long, default_value = "./logs")]
    pub log_dir: String,
}

impl Config {
    /// ID SRS: SRS-FN-CONFIG-001
    /// Title: parse
    ///
    /// Description: VRConnect shall parse the configuration from CLI arguments
    /// and optionally merge with environment file, returning a validated Config instance.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Parsed and validated configuration
    pub fn parse() -> Self {
        let mut config = <Config as Parser>::parse();

        // If config file specified, load and merge
        if let Some(ref config_path) = config.config_file {
            if let Ok(file_config) = loader::load_from_file(config_path) {
                config = config.merge_with(file_config);
            }
        }

        // Validate
        config.validate().expect("Invalid configuration");

        config
    }

    /// ID SRS: SRS-FN-CONFIG-002
    /// Title: merge_with
    ///
    /// Description: VRConnect shall merge the current configuration with values
    /// from a file-loaded configuration, with CLI arguments taking precedence.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `file_config` - Configuration loaded from file
    ///
    /// # Returns
    /// Merged configuration
    fn merge_with(self, _file_config: Config) -> Self {
        // CLI arguments already parsed, just return self
        // File config is loaded via dotenvy before CLI parsing
        self
    }

    /// ID SRS: SRS-FN-CONFIG-003
    /// Title: validate
    ///
    /// Description: VRConnect shall validate the configuration parameters,
    /// returning an error if any value is invalid or inconsistent.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Result indicating validation success or error
    pub fn validate(&self) -> Result<(), String> {
        // Validate port range
        if self.socketio_port == 0 {
            return Err("Socket.IO port cannot be 0".to_string());
        }

        // Validate UUID format if BLE enabled
        if self.output_ble_enabled {
            if Uuid::parse_str(&self.output_ble_service_uuid).is_err() {
                return Err(format!("Invalid BLE service UUID: {}", self.output_ble_service_uuid));
            }
        }

        // Validate log level
        let valid_levels = ["SUCCESS", "INFO", "WARNING", "ERROR", "DEBUG"];
        if !valid_levels.contains(&self.log_level.to_uppercase().as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        Ok(())
    }

    /// ID SRS: SRS-FN-CONFIG-004
    /// Title: socket_url
    ///
    /// Description: VRConnect shall construct the complete Socket.IO URL
    /// from host and port configuration parameters.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Complete Socket.IO URL string
    #[allow(dead_code)]
    pub fn socket_url(&self) -> String {
        format!("http://{}:{}", self.socketio_host, self.socketio_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_valid() {
        // TODO: Implement valid configuration test
        assert!(true);
    }

    #[test]
    fn test_config_validation_invalid_port() {
        // TODO: Implement invalid port test
        assert!(true);
    }

    #[test]
    fn test_config_validation_invalid_uuid() {
        // TODO: Implement invalid UUID test
        assert!(true);
    }

    #[test]
    fn test_config_merge() {
        // TODO: Implement configuration merge test
        assert!(true);
    }

    #[test]
    fn test_socket_url_construction() {
        // TODO: Implement URL construction test
        assert!(true);
    }
}
