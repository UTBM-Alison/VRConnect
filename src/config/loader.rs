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
    dotenvy::from_path(path.as_ref()).map_err(|e| format!("Failed to load config file: {}", e))?;

    // Build config from environment variables
    let config = Config {
        config_file: None,
        socketio_host: std::env::var("SOCKETIO_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
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
        log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string()),
        log_dir: std::env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string()),
    };

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to clear all config-related env vars
    fn clear_env_vars() {
        let vars = [
            "SOCKETIO_HOST",
            "SOCKETIO_PORT",
            "OUTPUT_CONSOLE_ENABLED",
            "OUTPUT_CONSOLE_VERBOSE",
            "OUTPUT_CONSOLE_COLORIZED",
            "OUTPUT_BLE_ENABLED",
            "OUTPUT_BLE_DEVICE_NAME",
            "OUTPUT_BLE_SERVICE_UUID",
            "DEBUG_ENABLED",
            "DEBUG_OUTPUT_PATH",
            "LOG_LEVEL",
            "LOG_DIR",
        ];
        for var in &vars {
            std::env::remove_var(var);
        }
    }

    /// ID SRS: SRS-TEST-LOADER-001
    /// Title: Test successful config file loading
    ///
    /// Description: VRConnect shall load configuration from valid .env file.
    ///
    /// Version: V1.0
    #[test]
    #[serial]
    fn test_load_from_file_success() {
        clear_env_vars();

        let mut temp_file = NamedTempFile::new().unwrap();

        // Write test config
        writeln!(temp_file, "SOCKETIO_HOST=192.168.1.1").unwrap();
        writeln!(temp_file, "SOCKETIO_PORT=5000").unwrap();
        writeln!(temp_file, "OUTPUT_CONSOLE_VERBOSE=true").unwrap();
        writeln!(temp_file, "LOG_LEVEL=DEBUG").unwrap();
        temp_file.flush().unwrap();

        let result = load_from_file(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.socketio_host, "192.168.1.1");
        assert_eq!(config.socketio_port, 5000);
        assert!(config.output_console_verbose);
        assert_eq!(config.log_level, "DEBUG");

        clear_env_vars();
    }

    /// ID SRS: SRS-TEST-LOADER-002
    /// Title: Test missing config file handling
    ///
    /// Description: VRConnect shall return error when config file doesn't exist.
    ///
    /// Version: V1.0
    #[test]
    #[serial]
    fn test_load_from_file_missing() {
        clear_env_vars();
        let result = load_from_file("/nonexistent/path/config.env");
        assert!(result.is_err());
        clear_env_vars();
    }

    /// ID SRS: SRS-TEST-LOADER-003
    /// Title: Test config file with comments and invalid lines
    ///
    /// Description: VRConnect shall load valid entries and ignore
    /// comments and malformed lines in .env files.
    ///
    /// Version: V1.0
    #[test]
    #[serial]
    fn test_load_from_file_with_comments() {
        clear_env_vars();

        let mut temp_file = NamedTempFile::new().unwrap();

        // Write config with comments (dotenvy supports # comments)
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "SOCKETIO_PORT=6000").unwrap();
        writeln!(temp_file, "# Another comment").unwrap();
        writeln!(temp_file, "LOG_LEVEL=WARN").unwrap();
        temp_file.flush().unwrap();

        let result = load_from_file(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.socketio_port, 6000);
        assert_eq!(config.log_level, "WARN");
        clear_env_vars();
    }

    /// ID SRS: SRS-TEST-LOADER-004
    /// Title: Test partial config loading
    ///
    /// Description: VRConnect shall load partial config and use defaults
    /// for missing values.
    ///
    /// Version: V1.0
    #[test]
    #[serial]
    fn test_load_from_file_partial() {
        clear_env_vars();

        let mut temp_file = NamedTempFile::new().unwrap();

        // Only specify some values
        writeln!(temp_file, "SOCKETIO_PORT=8080").unwrap();
        temp_file.flush().unwrap();

        let result = load_from_file(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.socketio_port, 8080);
        // Others should have defaults
        assert_eq!(config.socketio_host, "127.0.0.1");

        clear_env_vars();
    }

    /// ID SRS: SRS-TEST-LOADER-005
    /// Title: Test boolean value parsing
    ///
    /// Description: VRConnect shall correctly parse boolean values from config.
    ///
    /// Version: V1.0
    #[test]
    #[serial]
    fn test_load_boolean_values() {
        clear_env_vars();

        let mut temp_file = NamedTempFile::new().unwrap();

        writeln!(temp_file, "OUTPUT_CONSOLE_ENABLED=false").unwrap();
        writeln!(temp_file, "OUTPUT_BLE_ENABLED=true").unwrap();
        writeln!(temp_file, "DEBUG_ENABLED=true").unwrap();
        temp_file.flush().unwrap();

        let result = load_from_file(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.output_console_enabled);
        assert!(config.output_ble_enabled);
        assert!(config.debug_enabled);

        clear_env_vars();
    }

    /// ID SRS: SRS-TEST-LOADER-006
    /// Title: Test environment variable override
    ///
    /// Description: VRConnect shall allow environment variables to override
    /// file values.
    ///
    /// Version: V1.0
    #[test]
    #[serial]
    fn test_env_var_override() {
        clear_env_vars();

        std::env::set_var("SOCKETIO_PORT", "9999");

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "SOCKETIO_PORT=5000").unwrap();
        temp_file.flush().unwrap();

        let result = load_from_file(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        // Env var should override file value
        assert_eq!(config.socketio_port, 9999);

        clear_env_vars();
    }

    /// ID SRS: SRS-TEST-LOADER-007
    /// Title: Test all config fields loading
    ///
    /// Description: VRConnect shall load all configuration fields from file.
    ///
    /// Version: V1.0
    #[test]
    #[serial]
    fn test_load_all_fields() {
        clear_env_vars();

        let mut temp_file = NamedTempFile::new().unwrap();

        writeln!(temp_file, "SOCKETIO_HOST=10.0.0.1").unwrap();
        writeln!(temp_file, "SOCKETIO_PORT=7000").unwrap();
        writeln!(temp_file, "OUTPUT_CONSOLE_ENABLED=true").unwrap();
        writeln!(temp_file, "OUTPUT_CONSOLE_VERBOSE=true").unwrap();
        writeln!(temp_file, "OUTPUT_CONSOLE_COLORIZED=false").unwrap();
        writeln!(temp_file, "OUTPUT_BLE_ENABLED=true").unwrap();
        writeln!(temp_file, "OUTPUT_BLE_DEVICE_NAME=TestDevice").unwrap();
        writeln!(
            temp_file,
            "OUTPUT_BLE_SERVICE_UUID=12345678-1234-5678-1234-567812345678"
        )
        .unwrap();
        writeln!(temp_file, "DEBUG_ENABLED=true").unwrap();
        writeln!(temp_file, "DEBUG_OUTPUT_PATH=/tmp/debug.log").unwrap();
        writeln!(temp_file, "LOG_LEVEL=TRACE").unwrap();
        writeln!(temp_file, "LOG_DIR=/var/log/test").unwrap();
        temp_file.flush().unwrap();

        let result = load_from_file(temp_file.path());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.socketio_host, "10.0.0.1");
        assert_eq!(config.socketio_port, 7000);
        assert!(config.output_console_enabled);
        assert!(config.output_console_verbose);
        assert!(!config.output_console_colorized);
        assert!(config.output_ble_enabled);
        assert_eq!(config.output_ble_device_name, "TestDevice");
        assert!(config.debug_enabled);
        assert_eq!(config.debug_output_path, "/tmp/debug.log");
        assert_eq!(config.log_level, "TRACE");
        assert_eq!(config.log_dir, "/var/log/test");

        clear_env_vars();
    }
}
