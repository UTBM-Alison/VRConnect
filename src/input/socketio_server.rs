// /src/input/socketio_server.rs
// Module: input.socketio_server
// Purpose: Socket.IO v4 WebSocket server for vital data reception

use crate::domain::ProcessedData;
use crate::error::{Result, VitalError};
use crate::input::decompressor::VitalDataDecompressor;
use crate::processor::{VitalDataCleaner, VitalDataTransformer};
use futures_util::{SinkExt, StreamExt};
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// ID SRS: SRS-MOD-SOCKETIO-001
/// Title: SocketIOServer
///
/// Description: VRConnect shall implement a Socket.IO v4 compatible WebSocket
/// server receiving vital data, with automatic decompression and processing.
///
/// Version: V1.0
pub struct SocketIOServer {
    host: String,
    port: u16,
    debug_enabled: bool,
    debug_file: Arc<RwLock<Option<File>>>,
    decompressor: VitalDataDecompressor,
    cleaner: VitalDataCleaner,
    transformer: VitalDataTransformer,
}

impl SocketIOServer {
    /// ID SRS: SRS-FN-SOCKETIO-001
    /// Title: new
    ///
    /// Description: VRConnect shall construct a SocketIOServer instance with
    /// host, port, debug configuration, and data processing components.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `host` - Server bind address
    /// * `port` - Server port
    /// * `debug_enabled` - Enable debug logging
    /// * `debug_file` - Debug file handle
    ///
    /// # Returns
    /// New SocketIOServer instance
    pub fn new(
        host: String,
        port: u16,
        debug_enabled: bool,
        debug_file: Arc<RwLock<Option<File>>>,
    ) -> Self {
        Self {
            host,
            port,
            debug_enabled,
            debug_file,
            decompressor: VitalDataDecompressor::new(),
            cleaner: VitalDataCleaner::new(),
            transformer: VitalDataTransformer::new(),
        }
    }

    /// ID SRS: SRS-FN-SOCKETIO-002
    /// Title: start
    ///
    /// Description: VRConnect shall start the Socket.IO WebSocket server,
    /// accepting connections and processing incoming vital data.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `tx` - Channel sender for processed data
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn start(&self, tx: mpsc::UnboundedSender<ProcessedData>) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| VitalError::Io(e))?;

        log::info!("Socket.IO v4 WebSocket server listening on {}", addr);
        log::info!("✓ Socket.IO server started");

        let tx = Arc::new(tx);
        let decompressor = Arc::new(self.decompressor.clone());
        let cleaner = Arc::new(self.cleaner.clone());
        let transformer = Arc::new(self.transformer.clone());
        let debug_file = self.debug_file.clone();
        let debug_enabled = self.debug_enabled;

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let tx = tx.clone();
                    let decompressor = decompressor.clone();
                    let cleaner = cleaner.clone();
                    let transformer = transformer.clone();
                    let debug_file = debug_file.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            tx,
                            decompressor,
                            cleaner,
                            transformer,
                            debug_enabled,
                            debug_file,
                        )
                        .await
                        {
                            log::error!("Connection error from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// ID SRS: SRS-FN-SOCKETIO-003
    /// Title: handle_connection
    ///
    /// Description: VRConnect shall handle individual WebSocket connection,
    /// performing Socket.IO handshake and processing messages.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `stream` - TCP stream
    /// * `addr` - Client address
    /// * `tx` - Data channel sender
    /// * `decompressor` - Decompressor instance
    /// * `cleaner` - Data cleaner instance
    /// * `transformer` - Data transformer instance
    /// * `debug_enabled` - Debug mode flag
    /// * `debug_file` - Debug file handle
    ///
    /// # Returns
    /// Result indicating success or error
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        tx: Arc<mpsc::UnboundedSender<ProcessedData>>,
        decompressor: Arc<VitalDataDecompressor>,
        cleaner: Arc<VitalDataCleaner>,
        transformer: Arc<VitalDataTransformer>,
        debug_enabled: bool,
        debug_file: Arc<RwLock<Option<File>>>,
    ) -> Result<()> {
        log::info!("New Socket.IO v4 connection from {}", addr);

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
            .map_err(|e| {
                VitalError::SocketIo(format!("Failed to send connection response: {}", e))
            })?;

        log::debug!("Sent connection response to {}", addr);

        // Debug log
        if debug_enabled {
            if let Some(ref mut file) = *debug_file.write().await {
                let _ = writeln!(
                    file,
                    "\n=== SOCKETIO CONNECTION ===\nClient: {}\nSID: {}\n",
                    addr, sid
                );
            }
        }

        let mut pending_binary_event: Option<String> = None;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    log::debug!("Received text message from {}: {}", addr, text);

                    // Debug log
                    if debug_enabled {
                        if let Some(ref mut file) = *debug_file.write().await {
                            let _ = writeln!(file, "\n=== TEXT MESSAGE ===\n{}\n", text);
                        }
                    }

                    if text.starts_with("2") {
                        // Engine.IO ping
                        log::debug!("Handling ping from {}", addr);
                        write
                            .send(Message::Text("3".to_string()))
                            .await
                            .map_err(|e| {
                                VitalError::SocketIo(format!("Failed to send pong: {}", e))
                            })?;
                    } else if text.starts_with("40") {
                        // Socket.IO connect
                        log::debug!("Socket.IO namespace connected: {}", addr);
                    } else if text.starts_with("42") {
                        // Socket.IO event
                        let event_data = &text[2..];

                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(event_data) {
                            if let Some(arr) = parsed.as_array() {
                                if let Some(event_name) = arr.get(0).and_then(|v| v.as_str()) {
                                    log::info!("Event '{}' received from {}", event_name, addr);

                                    if event_name == "join_vr" {
                                        if let Some(vr_code) = arr.get(1).and_then(|v| v.as_str()) {
                                            log::info!("VR joined: {}", vr_code);
                                        }
                                    }
                                }
                            }
                        }
                    } else if text.starts_with("451-") {
                        // Binary event placeholder
                        let placeholder_data = &text[4..];
                        log::debug!(
                            "Binary event placeholder from {}: {}",
                            addr,
                            placeholder_data
                        );
                        pending_binary_event = Some(placeholder_data.to_string());
                    }
                }
                Ok(Message::Binary(data)) => {
                    log::debug!(
                        "Received binary message from {}, length: {}",
                        addr,
                        data.len()
                    );

                    // Debug log raw binary
                    if debug_enabled {
                        if let Some(ref mut file) = *debug_file.write().await {
                            let _ = writeln!(
                                file,
                                "\n=== BINARY MESSAGE ===\nLength: {} bytes\nFirst 16 bytes: {:02X?}\n",
                                data.len(),
                                &data[..data.len().min(16)]
                            );
                        }
                    }

                    if pending_binary_event.take().is_some() {
                        match Self::process_data(
                            &data,
                            &decompressor,
                            &cleaner,
                            &transformer,
                            debug_enabled,
                            &debug_file,
                        )
                        .await
                        {
                            Ok(processed_data) => {
                                log::info!(
                                    "Successfully processed vital data: {} rooms, {} tracks",
                                    processed_data.rooms.len(),
                                    processed_data.all_tracks.len()
                                );

                                if let Err(e) = tx.send(processed_data) {
                                    log::error!("Failed to send processed data: {}", e);
                                }
                            }
                            Err(e) => {
                                log::error!("Error processing data from {}: {}", addr, e);
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    log::info!("Socket.IO connection closed: {}", addr);
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
                    log::warn!("WebSocket error from {}: {}", addr, e);
                    break;
                }
            }
        }

        log::info!("Connection handler finished for {}", addr);
        Ok(())
    }

    /// ID SRS: SRS-FN-SOCKETIO-004
    /// Title: process_data
    ///
    /// Description: VRConnect shall process binary data through decompression,
    /// cleaning, and transformation pipeline, with optional debug logging.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `data` - Raw binary data
    /// * `decompressor` - Decompressor instance
    /// * `cleaner` - Data cleaner instance
    /// * `transformer` - Data transformer instance
    /// * `debug_enabled` - Debug mode flag
    /// * `debug_file` - Debug file handle
    ///
    /// # Returns
    /// Processed vital data or error
    async fn process_data(
        data: &[u8],
        decompressor: &VitalDataDecompressor,
        cleaner: &VitalDataCleaner,
        transformer: &VitalDataTransformer,
        debug_enabled: bool,
        debug_file: &Arc<RwLock<Option<File>>>,
    ) -> Result<ProcessedData> {
        // Step 1: Decompress
        let decompressed = decompressor.decompress(data)?;
        log::debug!("Decompressed data length: {}", decompressed.len());

        // Debug log decompressed
        if debug_enabled {
            if let Some(ref mut file) = *debug_file.write().await {
                let _ = writeln!(
                    file,
                    "\n=== DECOMPRESSED DATA ===\nLength: {} bytes\n",
                    decompressed.len()
                );
            }
        }

        // Step 2: Convert to string
        let json_str = String::from_utf8(decompressed)
            .map_err(|e| VitalError::Processing(format!("UTF-8 conversion failed: {}", e)))?;

        // Debug log raw JSON
        if debug_enabled {
            if let Some(ref mut file) = *debug_file.write().await {
                let _ = writeln!(file, "\n=== RAW JSON ===\n{}\n", json_str);
            }
        }

        // Step 3: Clean JSON
        let cleaned_json = cleaner.clean(&json_str)?;

        // Debug log cleaned JSON
        if debug_enabled {
            if let Some(ref mut file) = *debug_file.write().await {
                let _ = writeln!(file, "\n=== CLEANED JSON ===\n{}\n", cleaned_json);
            }
        }

        // Step 4: Parse to VitalData
        let vital_data: crate::domain::VitalData = serde_json::from_str(&cleaned_json)?;

        // Step 5: Transform to ProcessedData
        let processed_data = transformer.transform(vital_data);

        // Debug log processed structure
        if debug_enabled {
            if let Some(ref mut file) = *debug_file.write().await {
                let _ = writeln!(
                    file,
                    "\n=== TRANSFORMATION COMPLETE ===\nDevice: {}\nRooms: {}\nTracks: {}\n",
                    processed_data.device_id,
                    processed_data.rooms.len(),
                    processed_data.all_tracks.len()
                );
            }
        }

        Ok(processed_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socketio_server_creation() {
        // TODO: Implement server creation test
        assert!(true);
    }

    #[test]
    fn test_connection_handshake() {
        // TODO: Implement handshake test
        assert!(true);
    }

    #[test]
    fn test_ping_pong() {
        // TODO: Implement ping/pong test
        assert!(true);
    }

    #[test]
    fn test_binary_event_processing() {
        // TODO: Implement binary event test
        assert!(true);
    }

    #[test]
    fn test_process_data_pipeline() {
        // TODO: Implement data processing pipeline test
        assert!(true);
    }
}
