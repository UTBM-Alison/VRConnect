// /src/error/mod.rs
// Module: error
// Purpose: Centralized error handling for the application

use thiserror::Error;

/// ID SRS: SRS-MOD-ERROR-001
/// Title: VitalError
///
/// Description: VRConnect shall provide a unified error type covering all
/// application domains (IO, parsing, network, BLE) with clear error messages.
///
/// Version: V1.0
#[derive(Error, Debug)]
pub enum VitalError {
    #[error("Decompression error: {0}")]
    Decompression(String),

    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Socket.IO error: {0}")]
    SocketIo(String),

    #[error("Bluetooth error: {0}")]
    Bluetooth(#[from] bluer::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Data processing error: {0}")]
    Processing(String),

    #[error("Regex error: {0}")]
    Regex(#[from] fancy_regex::Error),

    #[error("Logger error: {0}")]
    Logger(String),

}

/// ID SRS: SRS-UTIL-ERROR-001
/// Title: Result
///
/// Description: VRConnect shall define a standard Result type alias
/// using VitalError for consistent error handling across the application.
///
/// Version: V1.0
pub type Result<T> = std::result::Result<T, VitalError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        // TODO: Implement error display formatting tests
        assert!(true);
    }

    #[test]
    fn test_error_conversion() {
        // TODO: Implement error conversion tests (from io::Error, etc.)
        assert!(true);
    }
}
