use crate::config::Config;
use crate::domain::ProcessedData;
use crate::error::Result;
use crate::input::SocketIOServerInput;
use crate::output::{BleVitalOutput, ConsoleVitalOutput};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

pub struct VitalProcessor {
    config: Config,
    stats: Arc<RwLock<ProcessorStats>>,
}

#[derive(Debug, Default)]
struct ProcessorStats {
    start_time: Option<Instant>,
    data_received: u64,
    last_data: Option<ProcessedData>,
}

impl VitalProcessor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(ProcessorStats::default())),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting VitalConnect processor...");
        info!("Configuration: {:?}", self.config);

        // Initialize stats
        {
            let mut stats = self.stats.write().await;
            stats.start_time = Some(Instant::now());
        }

        // Create channel for data flow
        let (tx, mut rx) = mpsc::unbounded_channel::<ProcessedData>();

        // Create outputs
        let console_output = ConsoleVitalOutput::new(self.config.verbose, self.config.colorized);
        let ble_output = if self.config.ble_enabled {
            Some(Arc::new(BleVitalOutput::new(self.config.ble_device_name.clone())))
        } else {
            None
        };

        // Start BLE server if enabled
        let ble_task = if let Some(ble) = ble_output.clone() {
            let ble = ble.clone();
            Some(tokio::spawn(async move {
                if let Err(e) = ble.start().await {
                    tracing::error!("BLE server error: {}", e);
                }
            }))
        } else {
            None
        };

        // Start Socket.IO input
        let socketio_input = SocketIOServerInput::new(self.config.socket_url());
        let input_task = {
            let tx = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = socketio_input.start(tx).await {
                    tracing::error!("Socket.IO input error: {}", e);
                }
            })
        };

        // Process incoming data
        let stats = self.stats.clone();
        let ble_output_clone = ble_output.clone();
        let processing_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                debug!("Processing data for device: {}", data.device_id);

                // Update stats
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.data_received += 1;
                    stats_guard.last_data = Some(data.clone());
                }

                // Output to console
                console_output.output(&data).await;

                // Output to BLE if enabled
                if let Some(ble) = &ble_output_clone {
                    if let Err(e) = ble.output(&data).await {
                        tracing::error!("BLE output error: {}", e);
                    }
                }
            }
        });

        // Wait for tasks (they run until Ctrl+C)
        tokio::select! {
            _ = input_task => {
                info!("Socket.IO input task completed");
            }
            _ = processing_task => {
                info!("Processing task completed");
            }
            _ = async {
                if let Some(task) = ble_task {
                    let _ = task.await;
                }
            } => {
                info!("BLE task completed");
            }
        }

        self.print_stats().await;
        Ok(())
    }

    async fn print_stats(&self) {
        let stats = self.stats.read().await;
        
        info!("=== VitalConnect Statistics ===");
        
        if let Some(start_time) = stats.start_time {
            let uptime = start_time.elapsed();
            info!("Uptime: {:.2}s", uptime.as_secs_f64());
        }
        
        info!("Data packets received: {}", stats.data_received);
        
        if let Some(last_data) = &stats.last_data {
            info!("Last device ID: {}", last_data.device_id);
            info!("Last update: {}", last_data.timestamp);
            info!("Rooms processed: {}", last_data.rooms.len());
            info!("Tracks processed: {}", last_data.all_tracks.len());
        }
        
        info!("================================");
    }
}
