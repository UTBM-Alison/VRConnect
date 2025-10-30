// /src/config/loader.rs
// Module: config.loader
// Purpose: Load configuration from environment files

use crate::config::Config;
use std::path::Path;

/// ID SRS: SRS-FN-LOADER-001
/// Title: load_from_file
///
/// Description: VRConnect shall load configuration from a .env file,
/// parsing key-value pairs and constructing a Config instance.
///
/// Version: V1.0
///
/// # Arguments
/// * `path` - Path to configuration file
///
/// # Returns
/// Result containing loaded Config or error
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Config, String> {
    // Load .env file
    dotenvy::from_path(path.as_ref())
        .map_err(|e| format!("Failed to load config file: {}", e))?;

    // Build config from environment variables
    let config = Config {
        config_file: None,
        socketio_host: std::env::var("SOCKETIO_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string()),
        socketio_port: std::env::var("SOCKETIO_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000),
        output_console_enabled: std::env::var("OUTPUT_CONSOLE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        output_console_verbose: std::env::var("OUTPUT_CONSOLE_VERBOSE")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false),
        output_console_colorized: std::env::var("OUTPUT_CONSOLE_COLORIZED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        output_ble_enabled: std::env::var("OUTPUT_BLE_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false),
        output_ble_device_name: std::env::var("OUTPUT_BLE_DEVICE_NAME")
            .unwrap_or_else(|_| "VitalConnect".to_string()),
        output_ble_service_uuid: std::env::var("OUTPUT_BLE_SERVICE_UUID")
            .unwrap_or_else(|_| "12345678-1234-5678-1234-567812345678".to_string()),
        debug_enabled: std::env::var("DEBUG_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false),
        debug_output_path: std::env::var("DEBUG_OUTPUT_PATH")
            .unwrap_or_else(|_| "./logs/debug.log".to_string()),
        log_level: std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "INFO".to_string()),
        log_dir: std::env::var("LOG_DIR")
            .unwrap_or_else(|_| "./logs".to_string()),
    };

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_file_success() {
        // TODO: Implement successful file load test
        assert!(true);
    }

    #[test]
    fn test_load_from_file_missing() {
        // TODO: Implement missing file test
        assert!(true);
    }

    #[test]
    fn test_load_from_file_invalid_format() {
        // TODO: Implement invalid format test
        assert!(true);
    }
}
