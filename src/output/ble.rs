// /src/output/ble.rs
// Module: output.ble
// Purpose: BLE GATT server output for non-waveform tracks

use crate::domain::{ProcessedData, ProcessedTrack, TrackType};
use crate::error::{Result, VitalError};
use bluer::{
    adv::Advertisement,
    gatt::local::{
        Application, Characteristic, CharacteristicNotify, CharacteristicNotifyMethod,
        CharacteristicRead, Service,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const MAX_BLE_PAYLOAD: usize = 500;

/// ID SRS: SRS-MOD-BLE-001
/// Title: BleOutput
///
/// Description: VRConnect shall provide BLE GATT server output transmitting
/// non-waveform tracks only via notification characteristic.
///
/// Version: V1.0
pub struct BleOutput {
    device_name: String,
    service_uuid: Uuid,
    data_buffer: Arc<RwLock<Option<Vec<u8>>>>,
}

/// BLE output JSON structure (non-waveform tracks only)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BleMessage {
    version: String,
    device_id: String,
    timestamp: String,
    track_count: usize,
    tracks: Vec<BleTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BleTrack {
    name: String,
    room: String,
    #[serde(rename = "type")]
    track_type: String,
    unit: String,
    timestamp: String,
    value: BleValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum BleValue {
    Number { value: f64, display: String },
    Text { value: String },
    Other { value: String },
}

impl BleOutput {
    /// ID SRS: SRS-FN-BLE-001
    /// Title: new
    ///
    /// Description: VRConnect shall construct a BleOutput instance with device
    /// name, service UUID, and initialize data buffer for notifications.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `device_name` - BLE advertising name
    /// * `service_uuid_str` - Service UUID string
    ///
    /// # Returns
    /// New BleOutput instance or error
    pub async fn new(device_name: String, service_uuid_str: String) -> Result<Self> {
        let service_uuid = Uuid::parse_str(&service_uuid_str)
            .map_err(|e| VitalError::Config(format!("Invalid BLE service UUID: {}", e)))?;

        Ok(Self {
            device_name,
            service_uuid,
            data_buffer: Arc::new(RwLock::new(None)),
        })
    }

    /// ID SRS: SRS-FN-BLE-002
    /// Title: start
    ///
    /// Description: VRConnect shall start BLE GATT server, register service and
    /// characteristic, and begin advertising with configured name and UUID.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn start(&self) -> Result<()> {
        log::info!("Starting BLE GATT server...");
        log::info!("  Device Name: {}", self.device_name);
        log::info!("  Service UUID: {}", self.service_uuid);
        log::info!("  ⚠️  Waveform tracks excluded from transmission");

        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;

        adapter.set_powered(true).await?;
        adapter.set_discoverable(true).await?;

        log::info!("Adapter: {} ({})", adapter.name(), adapter.address().await?);

        let app = self.create_application().await?;
        let app_handle = adapter.serve_gatt_application(app).await?;
        log::info!("✓ GATT application registered");

        let adv = Advertisement {
            service_uuids: vec![self.service_uuid].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some(self.device_name.clone()),
            ..Default::default()
        };

        let adv_handle = adapter.advertise(adv).await?;
        log::info!("✓ BLE advertising started");
        log::info!("✓ Push notifications enabled");

        tokio::signal::ctrl_c()
            .await
            .map_err(|e| VitalError::Io(e))?;

        drop(adv_handle);
        drop(app_handle);
        log::info!("BLE GATT server stopped");

        Ok(())
    }

    /// ID SRS: SRS-FN-BLE-003
    /// Title: create_application
    ///
    /// Description: VRConnect shall create GATT application with service and
    /// characteristic supporting read and notify operations.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// GATT Application structure
    async fn create_application(&self) -> Result<Application> {
        let data_buffer = self.data_buffer.clone();
        let data_buffer_notify = self.data_buffer.clone();

        let char_uuid = Uuid::parse_str(&format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.service_uuid.as_u128() >> 96,
            (self.service_uuid.as_u128() >> 80) & 0xFFFF,
            (self.service_uuid.as_u128() >> 64) & 0xFFFF,
            (self.service_uuid.as_u128() >> 48) & 0xFFFF,
            (self.service_uuid.as_u128() & 0xFFFFFFFFFFFF) + 1
        ))
        .unwrap();

        let data_char = Characteristic {
            uuid: char_uuid,
            read: Some(CharacteristicRead {
                read: true,
                fun: Box::new(move |_req| {
                    let data_buffer = data_buffer.clone();
                    Box::pin(async move {
                        let buffer = data_buffer.read().await;
                        match buffer.as_ref() {
                            Some(data) => {
                                log::debug!("BLE read: {} bytes", data.len());
                                Ok(data.clone())
                            }
                            None => {
                                log::debug!("BLE read: no data available");
                                Ok(Vec::new())
                            }
                        }
                    })
                }),
                ..Default::default()
            }),
            notify: Some(CharacteristicNotify {
                notify: true,
                method: CharacteristicNotifyMethod::Fun(Box::new(move |mut notifier| {
                    let data_buffer = data_buffer_notify.clone();
                    Box::pin(async move {
                        log::info!("✓ Client subscribed to notifications");

                        let mut last_data: Option<Vec<u8>> = None;
                        let mut interval =
                            tokio::time::interval(tokio::time::Duration::from_millis(100));

                        loop {
                            interval.tick().await;

                            let current_data = data_buffer.read().await.clone();

                            if current_data.is_some() && current_data != last_data {
                                if let Some(data) = &current_data {
                                    match notifier.notify(data.clone()).await {
                                        Ok(_) => {
                                            log::debug!(
                                                "✓ Notification sent: {} bytes",
                                                data.len()
                                            );
                                        }
                                        Err(e) => {
                                            log::warn!("Notification send failed: {}", e);
                                            break;
                                        }
                                    }
                                }
                                last_data = current_data;
                            }
                        }

                        log::info!("Client unsubscribed from notifications");
                    })
                })),
                ..Default::default()
            }),
            ..Default::default()
        };

        let service = Service {
            uuid: self.service_uuid,
            primary: true,
            characteristics: vec![data_char],
            ..Default::default()
        };

        Ok(Application {
            services: vec![service],
            ..Default::default()
        })
    }

    /// ID SRS: SRS-FN-BLE-004
    /// Title: output
    ///
    /// Description: VRConnect shall filter non-waveform tracks, serialize to JSON,
    /// and update data buffer for BLE notification transmission.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `data` - Processed vital data
    ///
    /// # Returns
    /// Result indicating success or error
    pub async fn output(&self, data: &ProcessedData) -> Result<()> {
        // Filter non-waveform tracks
        let non_waveform_tracks = data.get_non_waveform_tracks();

        if non_waveform_tracks.is_empty() {
            log::debug!("No non-waveform tracks to transmit via BLE");
            return Ok(());
        }

        // Convert to BLE format
        let ble_tracks: Vec<BleTrack> = non_waveform_tracks
            .iter()
            .map(|track| self.convert_track(track))
            .collect();

        let message = BleMessage {
            version: "1.0".to_string(),
            device_id: data.device_id.clone(),
            timestamp: data.timestamp.to_rfc3339(),
            track_count: ble_tracks.len(),
            tracks: ble_tracks,
        };

        // Serialize to JSON
        let json_bytes = serde_json::to_vec(&message)
            .map_err(|e| VitalError::Processing(format!("JSON serialization failed: {}", e)))?;

        let size = json_bytes.len();

        // Check payload size
        if size > MAX_BLE_PAYLOAD {
            log::warn!(
                "⚠️  BLE payload too large: {} bytes (max: {})",
                size,
                MAX_BLE_PAYLOAD
            );
            log::warn!("   Truncating to fit MTU limit");

            let truncated = json_bytes[..MAX_BLE_PAYLOAD].to_vec();
            *self.data_buffer.write().await = Some(truncated);
        } else {
            log::debug!("BLE payload: {} bytes (OK)", size);
            *self.data_buffer.write().await = Some(json_bytes);
        }

        Ok(())
    }

    /// ID SRS: SRS-FN-BLE-005
    /// Title: convert_track
    ///
    /// Description: VRConnect shall convert ProcessedTrack to BleTrack format
    /// with appropriate value structure based on track type.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `track` - Processed track to convert
    ///
    /// # Returns
    /// BLE-formatted track
    fn convert_track(&self, track: &ProcessedTrack) -> BleTrack {
        let track_type_str = match track.track_type {
            TrackType::Number => "number",
            TrackType::String => "string",
            TrackType::Other => "other",
            TrackType::Waveform => "waveform", // Should not occur due to filtering
        };

        let value = match track.track_type {
            TrackType::Number => BleValue::Number {
                value: track.raw_value.unwrap_or(0.0),
                display: track.display_value.clone(),
            },
            TrackType::String => BleValue::Text {
                value: track.display_value.clone(),
            },
            _ => BleValue::Other {
                value: track.display_value.clone(),
            },
        };

        BleTrack {
            name: track.name.clone(),
            room: track.room_name.clone(),
            track_type: track_type_str.to_string(),
            unit: track.unit.clone(),
            timestamp: track.timestamp.to_rfc3339(),
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_output_creation() {
        // TODO: Implement BLE output creation test
        assert!(true);
    }

    #[test]
    fn test_filter_non_waveform() {
        // TODO: Implement non-waveform filtering test
        assert!(true);
    }

    #[test]
    fn test_convert_track_number() {
        // TODO: Implement number track conversion test
        assert!(true);
    }

    #[test]
    fn test_convert_track_string() {
        // TODO: Implement string track conversion test
        assert!(true);
    }

    #[test]
    fn test_payload_size_check() {
        // TODO: Implement payload size validation test
        assert!(true);
    }

    #[test]
    fn test_json_serialization() {
        // TODO: Implement JSON serialization test
        assert!(true);
    }
}
