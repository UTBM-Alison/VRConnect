use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "VitalConnect")]
#[command(about = "Real-time vital data processing with Socket.IO and BLE output", long_about = None)]
pub struct Config {
    /// Socket.IO server host
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Socket.IO server port
    #[arg(long, short = 'p', default_value = "3000")]  // Added short option -p
    pub port: u16,

    /// Enable verbose console output
    #[arg(long, short = 'v', default_value = "false")]  // Added short option -v
    pub verbose: bool,

    /// Enable colorized console output
    #[arg(long, default_value = "true")]
    pub colorized: bool,

    /// Enable BLE output
    #[arg(long, default_value = "true")]
    pub ble_enabled: bool,

    /// BLE device name to advertise
    #[arg(long, default_value = "VitalConnect")]
    pub ble_device_name: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,
}

impl Config {
    pub fn socket_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}
