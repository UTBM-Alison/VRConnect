// /src/output/mod.rs
// Module: output
// Purpose: Output modules for console and BLE

pub mod console;
pub mod ble;

pub use console::ConsoleOutput;
pub use ble::BleOutput;
