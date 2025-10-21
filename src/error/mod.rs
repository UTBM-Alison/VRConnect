use thiserror::Error;

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

    #[error("BLE error: {0}")]
    Bluetooth(#[from] bluer::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Data processing error: {0}")]
    Processing(String),

    #[error("Regex error: {0}")]
    Regex(#[from] fancy_regex::Error),
}

pub type Result<T> = std::result::Result<T, VitalError>;
