use crate::error::{Result, VitalError};
use crate::input::decompressor::VitalDataDecompressor;
use crate::processor::{VitalDataProcessor, VitalDataTransformer};
use crate::domain::ProcessedData;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

pub struct SocketIOServerInput {
    host: String,
    port: u16,
    decompressor: VitalDataDecompressor,
    processor: VitalDataProcessor,
    transformer: VitalDataTransformer,
}

impl SocketIOServerInput {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            decompressor: VitalDataDecompressor::new(),
            processor: VitalDataProcessor::new(),
            transformer: VitalDataTransformer::new(),
        }
    }

    pub async fn start(&self, tx: mpsc::UnboundedSender<ProcessedData>) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| VitalError::Io(e))?;

        info!("Socket.IO v4 WebSocket server started on port {}", self.port);
        info!("Configure Vital Recorder: SERVER_IP={}:{}", self.host, self.port);

        let tx = Arc::new(tx);
        let decompressor = Arc::new(self.decompressor.clone());
        let processor = Arc::new(self.processor.clone());
        let transformer = Arc::new(self.transformer.clone());

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let tx = tx.clone();
                    let decompressor = decompressor.clone();
                    let processor = processor.clone();
                    let transformer = transformer.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            tx,
                            decompressor,
                            processor,
                            transformer,
                        )
                        .await
                        {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        tx: Arc<mpsc::UnboundedSender<ProcessedData>>,
        decompressor: Arc<VitalDataDecompressor>,
        processor: Arc<VitalDataProcessor>,
        transformer: Arc<VitalDataTransformer>,
    ) -> Result<()> {
        info!("New Socket.IO v4 connection: /{}", addr);

        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| VitalError::SocketIo(format!("WebSocket handshake failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        // Send Socket.IO connection response (Engine.IO v4)
        let sid = uuid::Uuid::new_v4().to_string();
        let connection_response = format!(
            "0{{\"sid\":\"{}\",\"upgrades\":[],\"pingInterval\":25000,\"pingTimeout\":5000}}",
            sid
        );
        
        write
            .send(Message::Text(connection_response.clone()))
            .await
            .map_err(|e| VitalError::SocketIo(format!("Failed to send connection response: {}", e)))?;

        debug!("Sent connection response: {}", connection_response);

        let mut pending_binary_event: Option<String> = None;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received text message: {}", text);

                    if text.starts_with("2") {
                        // Engine.IO ping
                        debug!("Handling ping");
                        write
                            .send(Message::Text("3".to_string()))
                            .await
                            .map_err(|e| VitalError::SocketIo(format!("Failed to send pong: {}", e)))?;
                    } else if text.starts_with("40") {
                        // Socket.IO connect
                        debug!("Socket.IO namespace connected");
                    } else if text.starts_with("42") {
                        // Socket.IO event
                        let event_data = &text[2..];
                        debug!("Handling event: {}", event_data);

                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(event_data) {
                            if let Some(arr) = parsed.as_array() {
                                if let Some(event_name) = arr.get(0).and_then(|v| v.as_str()) {
                                    info!("Event received: {}", event_name);

                                    match event_name {
                                        "join_vr" => {
                                            if let Some(vr_code) = arr.get(1).and_then(|v| v.as_str()) {
                                                info!("VR joined: {}", vr_code);
                                            }
                                        }
                                        _ => {
                                            debug!("Unhandled event: {}", event_name);
                                        }
                                    }
                                }
                            }
                        }
                    } else if text.starts_with("451-") {
                        // Binary event placeholder
                        let placeholder_data = &text[4..];
                        debug!("Handling binary event placeholder: {}", placeholder_data);
                        pending_binary_event = Some(placeholder_data.to_string());
                    }
                }
                Ok(Message::Binary(data)) => {
                    debug!("Received binary message, length: {}", data.len());

                    if let Some(event_str) = pending_binary_event.take() {
                        info!(
                            "Processing binary event: {} with data length: {}",
                            event_str,
                            data.len()
                        );

                        match Self::process_data(&data, &decompressor, &processor, &transformer).await {
                            Ok(processed_data) => {
                                info!(
                                    "Successfully processed vital data: {} rooms, {} tracks",
                                    processed_data.rooms.len(),
                                    processed_data.all_tracks.len()
                                );

                                if let Err(e) = tx.send(processed_data) {
                                    error!("Failed to send processed data: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Error processing data: {}", e);
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Socket.IO connection closed: {}", addr);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    write
                        .send(Message::Pong(data))
                        .await
                        .map_err(|e| VitalError::SocketIo(format!("Failed to send pong: {}", e)))?;
                }
                Ok(Message::Pong(_)) => {}
                Ok(Message::Frame(_)) => {}
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        info!("Connection handler finished for {}", addr);
        Ok(())
    }

    async fn process_data(
        data: &[u8],
        decompressor: &VitalDataDecompressor,
        processor: &VitalDataProcessor,
        transformer: &VitalDataTransformer,
    ) -> Result<ProcessedData> {
        // Decompress if needed
        let decompressed = decompressor.decompress(data)?;
        debug!("Decompressed data length: {}", decompressed.len());

        // Convert to string
        let json_str = String::from_utf8(decompressed)
            .map_err(|e| VitalError::Processing(format!("UTF-8 conversion failed: {}", e)))?;

        // Parse and process
        let vital_data = processor.process(&json_str)?;
        
        // Transform to processed data
        let processed_data = transformer.transform(vital_data);

        Ok(processed_data)
    }
}
