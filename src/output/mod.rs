// /src/output/mod.rs
// Module: output
// Purpose: Output modules for console and BLE

pub mod ble;
pub mod console;

pub use ble::BleOutput;
pub use console::ConsoleOutput;
