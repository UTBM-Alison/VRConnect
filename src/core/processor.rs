// /src/core/processor.rs
// Module: core.processor
// Purpose: Main processor orchestrating data flow from input to outputs

use crate::config::Config;
use crate::domain::ProcessedData;
use crate::error::Result;
use crate::input::SocketIOServer;
use crate::output::{BleOutput, ConsoleOutput};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// ID SRS: SRS-MOD-PROCESSOR-001
/// Title: VitalProcessor
///
/// Description: VRConnect shall orchestrate the complete data processing pipeline
/// from Socket.IO input through transformation to multiple outputs with optional
/// debug logging.
///
/// Version: V1.0
pub struct VitalProcessor {
    config: Config,
    debug_file: Arc<RwLock<Option<std::fs::File>>>,
}

impl VitalProcessor {
    /// ID SRS: SRS-FN-PROCESSOR-001
    /// Title: new
    ///
    /// Description: VRConnect shall construct a VitalProcessor instance with
    /// configuration and initialize debug file if debug mode enabled.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `config` - Application configuration
    ///
    /// # Returns
    /// New VitalProcessor instance
    pub fn new(config: Config) -> Self {
        let debug_file = if config.debug_enabled {
            // Create debug file
            if let Ok(file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&config.debug_output_path)
            {
                Arc::new(RwLock::new(Some(file)))
            } else {
                log::error!("Failed to create debug file: {}", config.debug_output_path);
                Arc::new(RwLock::new(None))
            }
        } else {
            Arc::new(RwLock::new(None))
        };

        Self { config, debug_file }
    }

    /// ID SRS: SRS-FN-PROCESSOR-002
    /// Title: run
    ///
    /// Description: VRConnect shall execute the main processing loop, starting
    /// input server, creating outputs, and processing data until shutdown signal.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn run(&self) -> Result<()> {
        log::info!("Starting VitalProcessor...");

        // Create data channel
        let (tx, mut rx) = mpsc::unbounded_channel::<ProcessedData>();

        // Create console output
        let console_output = if self.config.output_console_enabled {
            Some(Arc::new(ConsoleOutput::new(
                self.config.output_console_verbose,
                self.config.output_console_colorized,
            )))
        } else {
            None
        };

        // Create BLE output
        let ble_output = if self.config.output_ble_enabled {
            log::warn!("⚠️  BLE Output: Waveform tracks excluded (MTU limit)");
            Some(Arc::new(
                BleOutput::new(
                    self.config.output_ble_device_name.clone(),
                    self.config.output_ble_service_uuid.clone(),
                )
                .await?,
            ))
        } else {
            None
        };

        // Start BLE server if enabled
        let ble_task = if let Some(ref ble) = ble_output {
            let ble_clone = ble.clone();
            Some(tokio::spawn(async move {
                if let Err(e) = ble_clone.start().await {
                    log::error!("BLE server error: {}", e);
                }
            }))
        } else {
            None
        };

        // Start Socket.IO input server
        let socketio_server = SocketIOServer::new(
            self.config.socketio_host.clone(),
            self.config.socketio_port,
            self.config.debug_enabled,
            self.debug_file.clone(),
        );

        let input_task = tokio::spawn(async move {
            if let Err(e) = socketio_server.start(tx).await {
                log::error!("Socket.IO server error: {}", e);
            }
        });

        log::info!("✓ VitalProcessor started successfully");

        // Processing loop
        let debug_file = self.debug_file.clone();
        let debug_enabled = self.config.debug_enabled;
        let ble_output_clone = ble_output.clone();
        let console_output_clone = console_output.clone();

        let processing_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                log::debug!("Processing data for device: {}", data.device_id);

                // Debug log processed data with ALL waveform points
                if debug_enabled {
                    Self::write_debug_data(&debug_file, &data).await;
                }

                // Output to console
                if let Some(ref console) = console_output_clone {
                    console.output(&data).await;
                }

                // Output to BLE (non-waveform only)
                if let Some(ref ble) = ble_output_clone {
                    if let Err(e) = ble.output(&data).await {
                        log::error!("BLE output error: {}", e);
                    }
                }
            }
        });

        // Wait for shutdown signal or task completion
        if let Some(ble_task) = ble_task {
            // With BLE enabled
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    log::info!("Shutdown signal received");
                }
                result = input_task => {
                    match result {
                        Ok(_) => log::info!("Socket.IO server stopped"),
                        Err(e) => log::error!("Socket.IO task panicked: {}", e),
                    }
                }
                result = processing_task => {
                    match result {
                        Ok(_) => log::info!("Processing task stopped"),
                        Err(e) => log::error!("Processing task panicked: {}", e),
                    }
                }
                result = ble_task => {
                    match result {
                        Ok(_) => log::info!("BLE server stopped"),
                        Err(e) => log::error!("BLE task panicked: {}", e),
                    }
                }
            }
        } else {
            // Without BLE
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    log::info!("Shutdown signal received");
                }
                result = input_task => {
                    match result {
                        Ok(_) => log::info!("Socket.IO server stopped"),
                        Err(e) => log::error!("Socket.IO task panicked: {}", e),
                    }
                }
                result = processing_task => {
                    match result {
                        Ok(_) => log::info!("Processing task stopped"),
                        Err(e) => log::error!("Processing task panicked: {}", e),
                    }
                }
            }
        }

        log::info!("✓ VitalProcessor stopped gracefully");
        Ok(())
    }

    /// ID SRS: SRS-FN-PROCESSOR-003
    /// Title: write_debug_data
    ///
    /// Description: VRConnect shall write complete processed data to debug file,
    /// including ALL waveform points for comprehensive data capture.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `debug_file` - Debug file handle
    /// * `data` - Processed data to log
    async fn write_debug_data(
        debug_file: &Arc<RwLock<Option<std::fs::File>>>,
        data: &ProcessedData,
    ) {
        if let Some(ref mut file) = *debug_file.write().await {
            // Header
            let _ = writeln!(file, "\n{}", "=".repeat(80));
            let _ = writeln!(file, "PROCESSED DATA - COMPLETE DUMP");
            let _ = writeln!(file, "{}", "=".repeat(80));
            let _ = writeln!(file, "Timestamp: {}", data.timestamp);
            let _ = writeln!(file, "Device ID: {}", data.device_id);
            let _ = writeln!(file, "Total Rooms: {}", data.rooms.len());
            let _ = writeln!(file, "Total Tracks: {}", data.all_tracks.len());
            let _ = writeln!(file, "{}", "=".repeat(80));

            // Process each room
            for room in &data.rooms {
                let _ = writeln!(
                    file,
                    "\n[ROOM] {} (Index: {})",
                    room.room_name, room.room_index
                );
                let _ = writeln!(file, "  Tracks in room: {}", room.tracks.len());
                let _ = writeln!(file, "{}", "-".repeat(80));

                for track in &room.tracks {
                    let _ = writeln!(file, "\n  [TRACK] {}", track.name);
                    let _ = writeln!(file, "    Type: {:?}", track.track_type);
                    let _ = writeln!(file, "    Room: {}", track.room_name);
                    let _ = writeln!(file, "    Unit: {}", track.unit);
                    let _ = writeln!(
                        file,
                        "    Timestamp: {}",
                        track.timestamp.format("%H:%M:%S%.3f")
                    );
                    let _ = writeln!(file, "    Display Value: {}", track.display_value);

                    // Raw value for numbers
                    if let Some(raw_val) = track.raw_value {
                        let _ = writeln!(file, "    Raw Value: {}", raw_val);
                    }

                    // Waveform statistics
                    if let Some(stats) = &track.waveform_stats {
                        let _ = writeln!(file, "    Waveform Stats:");
                        let _ = writeln!(file, "      Count: {}", stats.count);
                        let _ = writeln!(file, "      Min: {:.6}", stats.min);
                        let _ = writeln!(file, "      Max: {:.6}", stats.max);
                        let _ = writeln!(file, "      Avg: {:.6}", stats.avg);
                    }

                    // ALL WAVEFORM POINTS
                    if let Some(points) = &track.waveform_points {
                        let _ = writeln!(file, "    Waveform Points ({} total):", points.len());
                        let _ = write!(file, "      ");

                        for (i, point) in points.iter().enumerate() {
                            let _ = write!(file, "{:.6}", point);

                            // Formatting: 10 points per line
                            if (i + 1) % 10 == 0 && i + 1 < points.len() {
                                let _ = writeln!(file);
                                let _ = write!(file, "      ");
                            } else if i + 1 < points.len() {
                                let _ = write!(file, ", ");
                            }
                        }
                        let _ = writeln!(file);
                    }
                }
            }

            let _ = writeln!(file, "\n{}", "=".repeat(80));
            let _ = writeln!(file, "END OF DATA DUMP");
            let _ = writeln!(file, "{}\n", "=".repeat(80));

            // Flush to ensure data is written
            let _ = file.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        // TODO: Implement processor creation test
        assert!(true);
    }

    #[test]
    fn test_debug_file_initialization() {
        // TODO: Implement debug file init test
        assert!(true);
    }

    #[test]
    fn test_processor_lifecycle() {
        // TODO: Implement processor lifecycle test
        assert!(true);
    }

    #[test]
    fn test_write_debug_data() {
        // TODO: Implement debug data writing test
        assert!(true);
    }
}
