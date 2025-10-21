use crate::error::{Result, VitalError};
use crate::input::decompressor::VitalDataDecompressor;
use crate::processor::{VitalDataProcessor, VitalDataTransformer};
use crate::domain::ProcessedData;
use rust_socketio::{asynchronous::Client, Payload};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub struct SocketIOServerInput {
    url: String,
    decompressor: VitalDataDecompressor,
    processor: VitalDataProcessor,
    transformer: VitalDataTransformer,
}

impl SocketIOServerInput {
    pub fn new(url: String) -> Self {
        Self {
            url,
            decompressor: VitalDataDecompressor::new(),
            processor: VitalDataProcessor::new(),
            transformer: VitalDataTransformer::new(),
        }
    }

    pub async fn start(&self, tx: mpsc::UnboundedSender<ProcessedData>) -> Result<()> {
        info!("Connecting to Socket.IO server at {}", self.url);

        let decompressor = self.decompressor.clone();
        let processor = self.processor.clone();
        let transformer = self.transformer.clone();
        let tx = Arc::new(tx);

        let tx_clone = tx.clone();
        let callback = move |payload: Payload, _client: Client| {
            let decompressor = decompressor.clone();
            let processor = processor.clone();
            let transformer = transformer.clone();
            let tx = tx_clone.clone();

            async move {
                match Self::handle_payload(payload, &decompressor, &processor, &transformer).await {
                    Ok(processed_data) => {
                        if let Err(e) = tx.send(processed_data) {
                            error!("Failed to send processed data: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error processing payload: {}", e);
                    }
                }
            }
        };

        // Build Socket.IO client
        let client = Client::builder(&self.url)
            .on("vitalData", callback)
            .on("connect", |_, _| {
                async move {
                    info!("Connected to Socket.IO server");
                }
            })
            .on("disconnect", |_, _| {
                async move {
                    warn!("Disconnected from Socket.IO server");
                }
            })
            .on("error", |err, _| {
                async move {
                    error!("Socket.IO error: {:?}", err);
                }
            })
            .connect()
            .await
            .map_err(|e| VitalError::SocketIo(format!("Connection failed: {}", e)))?;

        info!("Socket.IO client started successfully");

        // Keep the connection alive
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| VitalError::Io(e))?;

        client
            .disconnect()
            .await
            .map_err(|e| VitalError::SocketIo(format!("Disconnect failed: {}", e)))?;

        Ok(())
    }

    async fn handle_payload(
        payload: Payload,
        decompressor: &VitalDataDecompressor,
        processor: &VitalDataProcessor,
        transformer: &VitalDataTransformer,
    ) -> Result<ProcessedData> {
        debug!("Received payload: {:?}", payload);

        let data = match payload {
            Payload::Binary(bytes) => bytes,
            Payload::String(s) => s.into_bytes(),
            _ => {
                return Err(VitalError::Processing("Unsupported payload type".to_string()));
            }
        };

        // Decompress if needed
        let decompressed = decompressor.decompress(&data)?;

        // Convert to string
        let json_str = String::from_utf8(decompressed)
            .map_err(|e| VitalError::Processing(format!("UTF-8 conversion failed: {}", e)))?;

        // Parse and process
        let vital_data = processor.process(&json_str)?;
        
        // Transform to processed data
        let processed_data = transformer.transform(vital_data);

        debug!("Successfully processed data for device: {}", processed_data.device_id);
        Ok(processed_data)
    }
}
