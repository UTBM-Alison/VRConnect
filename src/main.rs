mod config;
mod core;
mod domain;
mod error;
mod input;
mod output;
mod processor;

use clap::Parser;
use config::Config;
use core::VitalProcessor;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let config = Config::parse();

    // Initialize logging
    let log_level = match config.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // Print banner
    print_banner(&config);

    // Create and run processor
    let processor = VitalProcessor::new(config);
    
    if let Err(e) = processor.run().await {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}

fn print_banner(config: &Config) {
    println!("\n{}", "═".repeat(60));
    println!("  VitalConnect - Rust Edition");
    println!("  Real-time Vital Data Processing");
    println!("{}", "═".repeat(60));
    println!("  Socket.IO:  {}", config.socket_url());
    println!("  BLE Output: {}", if config.ble_enabled { "Enabled" } else { "Disabled" });
    if config.ble_enabled {
        println!("  BLE Name:   {}", config.ble_device_name);
    }
    println!("  Verbose:    {}", config.verbose);
    println!("  Colorized:  {}", config.colorized);
    println!("  Log Level:  {}", config.log_level);
    println!("{}\n", "═".repeat(60));
}
