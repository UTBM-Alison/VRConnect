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
    /// Title: generate_char_uuid
    ///
    /// Description: VRConnect shall generate characteristic UUID by incrementing
    /// service UUID by 1.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Characteristic UUID
    fn generate_char_uuid(&self) -> Uuid {
        Uuid::parse_str(&format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.service_uuid.as_u128() >> 96,
            (self.service_uuid.as_u128() >> 80) & 0xFFFF,
            (self.service_uuid.as_u128() >> 64) & 0xFFFF,
            (self.service_uuid.as_u128() >> 48) & 0xFFFF,
            (self.service_uuid.as_u128() & 0xFFFFFFFFFFFF) + 1
        ))
        .expect("Generated UUID should always be valid")
    }

    /// ID SRS: SRS-FN-BLE-003
    /// Title: create_advertisement_config
    ///
    /// Description: VRConnect shall create advertisement configuration with
    /// service UUIDs and device name.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Tuple of (service_uuids, local_name, discoverable)
    fn create_advertisement_config(&self) -> (Vec<Uuid>, Option<String>, Option<bool>) {
        (
            vec![self.service_uuid],
            Some(self.device_name.clone()),
            Some(true),
        )
    }

    /// ID SRS: SRS-FN-BLE-004
    /// Title: log_startup_info
    ///
    /// Description: VRConnect shall log BLE server startup information.
    ///
    /// Version: V1.0
    fn log_startup_info(&self) {
        log::info!("Starting BLE GATT server...");
        log::info!("  Device Name: {}", self.device_name);
        log::info!("  Service UUID: {}", self.service_uuid);
        log::info!("  ⚠️  Waveform tracks excluded from transmission");
    }

    /// ID SRS: SRS-FN-BLE-005
    /// Title: start
    ///
    /// Description: VRConnect shall start BLE GATT server, register service and
    /// characteristic, and begin advertising with configured name and UUID.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Result indicating success or error
    #[cfg(not(tarpaulin_include))] // Hardware-dependent, cannot unit test
    pub async fn start(&self) -> Result<()> {
        self.log_startup_info();

        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;

        adapter.set_powered(true).await?;
        adapter.set_discoverable(true).await?;

        log::info!("Adapter: {} ({})", adapter.name(), adapter.address().await?);

        let app = self.create_application().await?;
        let app_handle = adapter.serve_gatt_application(app).await?;
        log::info!("✓ GATT application registered");

        let (service_uuids, local_name, discoverable) = self.create_advertisement_config();
        let adv = Advertisement {
            service_uuids: service_uuids.into_iter().collect(),
            discoverable,
            local_name,
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

    /// ID SRS: SRS-FN-BLE-006
    /// Title: create_application
    ///
    /// Description: VRConnect shall create GATT application with service and
    /// characteristic supporting read and notify operations.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// GATT Application structure
    #[cfg(not(tarpaulin_include))] // Hardware-dependent, cannot unit test
    async fn create_application(&self) -> Result<Application> {
        let data_buffer = self.data_buffer.clone();
        let data_buffer_notify = self.data_buffer.clone();
        let char_uuid = self.generate_char_uuid();

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

    /// ID SRS: SRS-FN-BLE-007
    /// Title: create_ble_message
    ///
    /// Description: VRConnect shall create BleMessage from filtered tracks.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `data` - Processed vital data
    /// * `ble_tracks` - Converted BLE tracks
    ///
    /// # Returns
    /// BleMessage structure
    fn create_ble_message(&self, data: &ProcessedData, ble_tracks: Vec<BleTrack>) -> BleMessage {
        BleMessage {
            version: "1.0".to_string(),
            device_id: data.device_id.clone(),
            timestamp: data.timestamp.to_rfc3339(),
            track_count: ble_tracks.len(),
            tracks: ble_tracks,
        }
    }

    /// ID SRS: SRS-FN-BLE-008
    /// Title: check_payload_size
    ///
    /// Description: VRConnect shall check payload size and return truncated
    /// version if exceeds MAX_BLE_PAYLOAD.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `json_bytes` - Serialized JSON bytes
    ///
    /// # Returns
    /// Tuple of (final_bytes, was_truncated)
    fn check_payload_size(&self, json_bytes: Vec<u8>) -> (Vec<u8>, bool) {
        let size = json_bytes.len();

        if size > MAX_BLE_PAYLOAD {
            log::warn!(
                "⚠️  BLE payload too large: {} bytes (max: {})",
                size,
                MAX_BLE_PAYLOAD
            );
            log::warn!("   Truncating to fit MTU limit");

            let truncated = json_bytes[..MAX_BLE_PAYLOAD].to_vec();
            (truncated, true)
        } else {
            log::debug!("BLE payload: {} bytes (OK)", size);
            (json_bytes, false)
        }
    }

    /// ID SRS: SRS-FN-BLE-009
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

        // Create message
        let message = self.create_ble_message(data, ble_tracks);

        // Serialize to JSON
        let json_bytes = serde_json::to_vec(&message)
            .map_err(|e| VitalError::Processing(format!("JSON serialization failed: {}", e)))?;

        // Check size and truncate if needed
        let (final_bytes, _was_truncated) = self.check_payload_size(json_bytes);

        // Update buffer
        *self.data_buffer.write().await = Some(final_bytes);

        Ok(())
    }

    /// ID SRS: SRS-FN-BLE-010
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
    use crate::domain::{ProcessedData, ProcessedRoom, ProcessedTrack, TrackType, WaveformStats};
    use chrono::Utc;

    /// ID SRS: SRS-TEST-BLE-001
    /// Title: Test BleOutput creation
    ///
    /// Description: VRConnect shall create BleOutput with device name and service UUID.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_ble_output_creation() {
        let result = BleOutput::new(
            "TestDevice".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await;

        assert!(result.is_ok());
        let ble = result.unwrap();
        assert_eq!(ble.device_name, "TestDevice");
    }

    /// ID SRS: SRS-TEST-BLE-002
    /// Title: Test BleOutput with invalid UUID
    ///
    /// Description: VRConnect shall return error for invalid service UUID.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_ble_output_invalid_uuid() {
        let result = BleOutput::new("TestDevice".to_string(), "INVALID-UUID".to_string()).await;

        assert!(result.is_err());
    }

    /// ID SRS: SRS-TEST-BLE-003
    /// Title: Test generate_char_uuid
    ///
    /// Description: VRConnect shall generate characteristic UUID by incrementing
    /// service UUID.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_generate_char_uuid() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let char_uuid = ble.generate_char_uuid();

        // Characteristic UUID should be service UUID + 1
        assert_ne!(char_uuid, ble.service_uuid);
        assert_eq!(char_uuid.as_u128(), ble.service_uuid.as_u128() + 1);
    }

    /// ID SRS: SRS-TEST-BLE-004
    /// Title: Test create_advertisement_config
    ///
    /// Description: VRConnect shall create correct advertisement configuration.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_create_advertisement_config() {
        let ble = BleOutput::new(
            "MyDevice".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let (service_uuids, local_name, discoverable) = ble.create_advertisement_config();

        assert_eq!(service_uuids.len(), 1);
        assert_eq!(service_uuids[0], ble.service_uuid);
        assert_eq!(local_name, Some("MyDevice".to_string()));
        assert_eq!(discoverable, Some(true));
    }

    /// ID SRS: SRS-TEST-BLE-005
    /// Title: Test log_startup_info
    ///
    /// Description: VRConnect shall log startup information without errors.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_log_startup_info() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        // Should not panic
        ble.log_startup_info();
    }

    /// ID SRS: SRS-TEST-BLE-006
    /// Title: Test convert_track for number type
    ///
    /// Description: VRConnect shall convert Number track to BleTrack format.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_convert_track_number() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let track = ProcessedTrack {
            name: "HR".to_string(),
            display_value: "75.000".to_string(),
            raw_value: Some(75.0),
            unit: "bpm".to_string(),
            timestamp: Utc::now(),
            room_index: 0,
            room_name: "BED_01".to_string(),
            track_index: 0,
            record_index: 0,
            track_type: TrackType::Number,
            waveform_stats: None,
            waveform_points: None,
        };

        let ble_track = ble.convert_track(&track);

        assert_eq!(ble_track.name, "HR");
        assert_eq!(ble_track.room, "BED_01");
        assert_eq!(ble_track.track_type, "number");
        assert_eq!(ble_track.unit, "bpm");

        match ble_track.value {
            BleValue::Number { value, display } => {
                assert_eq!(value, 75.0);
                assert_eq!(display, "75.000");
            }
            _ => panic!("Expected Number value"),
        }
    }

    /// ID SRS: SRS-TEST-BLE-007
    /// Title: Test convert_track for string type
    ///
    /// Description: VRConnect shall convert String track to BleTrack format.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_convert_track_string() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let track = ProcessedTrack {
            name: "ALARM".to_string(),
            display_value: "HR High".to_string(),
            raw_value: None,
            unit: "".to_string(),
            timestamp: Utc::now(),
            room_index: 0,
            room_name: "BED_01".to_string(),
            track_index: 0,
            record_index: 0,
            track_type: TrackType::String,
            waveform_stats: None,
            waveform_points: None,
        };

        let ble_track = ble.convert_track(&track);

        assert_eq!(ble_track.track_type, "string");

        match ble_track.value {
            BleValue::Text { value } => {
                assert_eq!(value, "HR High");
            }
            _ => panic!("Expected Text value"),
        }
    }

    /// ID SRS: SRS-TEST-BLE-008
    /// Title: Test convert_track for Other type
    ///
    /// Description: VRConnect shall convert Other track type to BleTrack.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_convert_track_other() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let track = ProcessedTrack {
            name: "UNKNOWN".to_string(),
            display_value: "Some value".to_string(),
            raw_value: None,
            unit: "".to_string(),
            timestamp: Utc::now(),
            room_index: 0,
            room_name: "BED_01".to_string(),
            track_index: 0,
            record_index: 0,
            track_type: TrackType::Other,
            waveform_stats: None,
            waveform_points: None,
        };

        let ble_track = ble.convert_track(&track);

        assert_eq!(ble_track.track_type, "other");

        match ble_track.value {
            BleValue::Other { value } => {
                assert_eq!(value, "Some value");
            }
            _ => panic!("Expected Other value"),
        }
    }

    /// ID SRS: SRS-TEST-BLE-009
    /// Title: Test convert_track for Waveform type
    ///
    /// Description: VRConnect shall handle waveform track conversion
    /// (though they should be filtered).
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_convert_track_waveform() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let track = ProcessedTrack {
            name: "ECG".to_string(),
            display_value: "110 points".to_string(),
            raw_value: None,
            unit: "mV".to_string(),
            timestamp: Utc::now(),
            room_index: 0,
            room_name: "BED_01".to_string(),
            track_index: 0,
            record_index: 0,
            track_type: TrackType::Waveform,
            waveform_stats: None,
            waveform_points: None,
        };

        let ble_track = ble.convert_track(&track);

        // Should still convert even though it shouldn't normally happen
        assert_eq!(ble_track.track_type, "waveform");
    }

    /// ID SRS: SRS-TEST-BLE-010
    /// Title: Test create_ble_message
    ///
    /// Description: VRConnect shall create BleMessage with correct structure.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_create_ble_message() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let room = ProcessedRoom {
            room_index: 0,
            room_name: "BED_01".to_string(),
            tracks: vec![ProcessedTrack {
                name: "HR".to_string(),
                display_value: "75.000".to_string(),
                raw_value: Some(75.0),
                unit: "bpm".to_string(),
                timestamp: Utc::now(),
                room_index: 0,
                room_name: "BED_01".to_string(),
                track_index: 0,
                record_index: 0,
                track_type: TrackType::Number,
                waveform_stats: None,
                waveform_points: None,
            }],
        };

        let data = ProcessedData::new("VR-TEST".to_string(), vec![room]);

        let ble_track = ble.convert_track(&data.all_tracks[0]);
        let message = ble.create_ble_message(&data, vec![ble_track]);

        assert_eq!(message.version, "1.0");
        assert_eq!(message.device_id, "VR-TEST");
        assert_eq!(message.track_count, 1);
        assert_eq!(message.tracks.len(), 1);
    }

    /// ID SRS: SRS-TEST-BLE-011
    /// Title: Test check_payload_size within limit
    ///
    /// Description: VRConnect shall not truncate payload within size limit.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_check_payload_size_ok() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let small_payload = vec![0u8; 100];
        let (result, truncated) = ble.check_payload_size(small_payload.clone());

        assert_eq!(result.len(), 100);
        assert!(!truncated);
    }

    /// ID SRS: SRS-TEST-BLE-012
    /// Title: Test check_payload_size exceeds limit
    ///
    /// Description: VRConnect shall truncate payload exceeding MAX_BLE_PAYLOAD.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_check_payload_size_truncate() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let large_payload = vec![0u8; 1000];
        let (result, truncated) = ble.check_payload_size(large_payload);

        assert_eq!(result.len(), MAX_BLE_PAYLOAD);
        assert!(truncated);
    }

    /// ID SRS: SRS-TEST-BLE-013
    /// Title: Test filter non-waveform tracks
    ///
    /// Description: VRConnect shall output only non-waveform tracks via BLE.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_filter_non_waveform() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let room = ProcessedRoom {
            room_index: 0,
            room_name: "BED_01".to_string(),
            tracks: vec![
                ProcessedTrack {
                    name: "HR".to_string(),
                    display_value: "75.000".to_string(),
                    raw_value: Some(75.0),
                    unit: "bpm".to_string(),
                    timestamp: Utc::now(),
                    room_index: 0,
                    room_name: "BED_01".to_string(),
                    track_index: 0,
                    record_index: 0,
                    track_type: TrackType::Number,
                    waveform_stats: None,
                    waveform_points: None,
                },
                ProcessedTrack {
                    name: "ECG".to_string(),
                    display_value: "110 points".to_string(),
                    raw_value: None,
                    unit: "mV".to_string(),
                    timestamp: Utc::now(),
                    room_index: 0,
                    room_name: "BED_01".to_string(),
                    track_index: 1,
                    record_index: 0,
                    track_type: TrackType::Waveform,
                    waveform_stats: Some(WaveformStats {
                        min: -1.0,
                        max: 1.0,
                        avg: 0.0,
                        count: 110,
                    }),
                    waveform_points: Some(vec![0.0; 110]),
                },
            ],
        };

        let data = ProcessedData::new("VR-TEST".to_string(), vec![room]);

        // Should succeed and filter out waveform
        let result = ble.output(&data).await;
        assert!(result.is_ok());
    }

    /// ID SRS: SRS-TEST-BLE-014
    /// Title: Test JSON serialization
    ///
    /// Description: VRConnect shall serialize BleMessage to valid JSON.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_json_serialization() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        let room = ProcessedRoom {
            room_index: 0,
            room_name: "BED_01".to_string(),
            tracks: vec![ProcessedTrack {
                name: "HR".to_string(),
                display_value: "75.000".to_string(),
                raw_value: Some(75.0),
                unit: "bpm".to_string(),
                timestamp: Utc::now(),
                room_index: 0,
                room_name: "BED_01".to_string(),
                track_index: 0,
                record_index: 0,
                track_type: TrackType::Number,
                waveform_stats: None,
                waveform_points: None,
            }],
        };

        let data = ProcessedData::new("VR-TEST".to_string(), vec![room]);

        // Output should create JSON in buffer
        let result = ble.output(&data).await;
        assert!(result.is_ok());

        // Verify buffer has data
        let buffer = ble.data_buffer.read().await;
        assert!(buffer.is_some());

        // Verify it's valid JSON
        let json_bytes = buffer.as_ref().unwrap();
        let json_str = String::from_utf8(json_bytes.clone()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["version"], "1.0");
        assert_eq!(parsed["device_id"], "VR-TEST");
        assert_eq!(parsed["track_count"], 1);
    }

    /// ID SRS: SRS-TEST-BLE-015
    /// Title: Test payload size check in output
    ///
    /// Description: VRConnect shall truncate large payloads in output method.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_payload_size_check() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        // Create many tracks to exceed payload limit
        let mut tracks = Vec::new();
        for i in 0..100 {
            tracks.push(ProcessedTrack {
                name: format!("TRACK_{}", i),
                display_value: "0".repeat(100), // Large value
                raw_value: Some(i as f64),
                unit: "unit".to_string(),
                timestamp: Utc::now(),
                room_index: 0,
                room_name: "BED_01".to_string(),
                track_index: i,
                record_index: 0,
                track_type: TrackType::String,
                waveform_stats: None,
                waveform_points: None,
            });
        }

        let room = ProcessedRoom {
            room_index: 0,
            room_name: "BED_01".to_string(),
            tracks,
        };

        let data = ProcessedData::new("VR-TEST".to_string(), vec![room]);

        let result = ble.output(&data).await;
        assert!(result.is_ok());

        // Verify payload was truncated
        let buffer = ble.data_buffer.read().await;
        assert!(buffer.is_some());
        let size = buffer.as_ref().unwrap().len();
        assert!(size <= MAX_BLE_PAYLOAD);
    }

    /// ID SRS: SRS-TEST-BLE-016
    /// Title: Test output with empty non-waveform tracks
    ///
    /// Description: VRConnect shall return early when no non-waveform
    /// tracks are present.
    ///
    /// Version: V1.0
    #[tokio::test]
    async fn test_output_empty_non_waveform() {
        let ble = BleOutput::new(
            "Test".to_string(),
            "12345678-1234-5678-1234-567812345678".to_string(),
        )
        .await
        .unwrap();

        // Create data with only waveform tracks
        let room = ProcessedRoom {
            room_index: 0,
            room_name: "BED_01".to_string(),
            tracks: vec![ProcessedTrack {
                name: "ECG".to_string(),
                display_value: "110 points".to_string(),
                raw_value: None,
                unit: "mV".to_string(),
                timestamp: Utc::now(),
                room_index: 0,
                room_name: "BED_01".to_string(),
                track_index: 0,
                record_index: 0,
                track_type: TrackType::Waveform,
                waveform_stats: None,
                waveform_points: None,
            }],
        };

        let data = ProcessedData::new("VR-TEST".to_string(), vec![room]);

        // Should succeed but not write to buffer
        let result = ble.output(&data).await;
        assert!(result.is_ok());

        // Buffer should still be None (early return)
        let buffer = ble.data_buffer.read().await;
        assert!(buffer.is_none());
    }
}
