use crate::domain::ProcessedData;
use crate::error::{Result, VitalError};
use bluer::{
    adv::Advertisement,
    gatt::local::{
        Application, Characteristic, CharacteristicNotify, CharacteristicNotifyMethod,
        CharacteristicWrite, CharacteristicWriteMethod, Service,
    },
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

// Custom UUIDs for VitalConnect GATT service
const VITAL_SERVICE_UUID: Uuid = Uuid::from_u128(0x12345678_1234_5678_1234_567812345678);
const VITAL_DATA_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x12345678_1234_5678_1234_567812345679);
const VITAL_CONTROL_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x12345678_1234_5678_1234_56781234567A);

pub struct BleVitalOutput {
    device_name: String,
    data_buffer: Arc<RwLock<Option<Vec<u8>>>>,
    notify_enabled: Arc<RwLock<bool>>,
}

impl BleVitalOutput {
    pub fn new(device_name: String) -> Self {
        Self {
            device_name,
            data_buffer: Arc::new(RwLock::new(None)),
            notify_enabled: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting BLE GATT server...");

        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        
        adapter.set_powered(true).await?;
        adapter.set_discoverable(true).await?;
        
        info!("Adapter: {} ({})", adapter.name(), adapter.address().await?);

        // Create GATT application
        let app = self.create_application().await?;

        // Register GATT application
        let app_handle = adapter.serve_gatt_application(app).await?;
        info!("GATT application registered");

        // Start advertising
        let adv = Advertisement {
            service_uuids: vec![VITAL_SERVICE_UUID].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some(self.device_name.clone()),
            ..Default::default()
        };

        let adv_handle = adapter.advertise(adv).await?;
        info!("BLE advertising started as '{}'", self.device_name);
        info!("Service UUID: {}", VITAL_SERVICE_UUID);

        // Keep the service running
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down BLE server...");
            }
        }

        drop(adv_handle);
        drop(app_handle);
        
        Ok(())
    }

    async fn create_application(&self) -> Result<Application> {
        let data_buffer = self.data_buffer.clone();
        let notify_enabled = self.notify_enabled.clone();

        // Data characteristic - for sending vital data to clients
        let data_char = Characteristic {
            uuid: VITAL_DATA_CHARACTERISTIC_UUID,
            read: Some(CharacteristicWrite {
                write: true,
                write_without_response: false,
                method: CharacteristicWriteMethod::Io,
                ..Default::default()
            }),
            notify: Some(CharacteristicNotify {
                notify: true,
                method: CharacteristicNotifyMethod::Io,
                ..Default::default()
            }),
            ..Default::default()
        };

        // Control characteristic - for receiving commands from clients
        let control_char = Characteristic {
            uuid: VITAL_CONTROL_CHARACTERISTIC_UUID,
            write: Some(CharacteristicWrite {
                write: true,
                write_without_response: false,
                method: CharacteristicWriteMethod::Fun(Box::new(move |new_value, _req| {
                    let notify_enabled = notify_enabled.clone();
                    Box::pin(async move {
                        if let Some(&command) = new_value.first() {
                            match command {
                                0x01 => {
                                    *notify_enabled.write().await = true;
                                    info!("Client enabled notifications");
                                }
                                0x00 => {
                                    *notify_enabled.write().await = false;
                                    info!("Client disabled notifications");
                                }
                                _ => {
                                    warn!("Unknown control command: 0x{:02X}", command);
                                }
                            }
                        }
                        Ok(())
                    })
                })),
                ..Default::default()
            }),
            ..Default::default()
        };

        let service = Service {
            uuid: VITAL_SERVICE_UUID,
            primary: true,
            characteristics: vec![data_char, control_char],
            ..Default::default()
        };

        Ok(Application {
            services: vec![service],
            ..Default::default()
        })
    }

    pub async fn output(&self, data: &ProcessedData) -> Result<()> {
        // Serialize the data to JSON
        let json_data = self.serialize_data(data)?;
        
        // Store in buffer
        *self.data_buffer.write().await = Some(json_data.clone());

        // If notifications are enabled, try to send
        if *self.notify_enabled.read().await {
            debug!("Sending {} bytes via BLE notification", json_data.len());
        }

        Ok(())
    }

    fn serialize_data(&self, data: &ProcessedData) -> Result<Vec<u8>> {
        // Create a simplified JSON structure for BLE transmission
        let simplified = serde_json::json!({
            "deviceId": data.device_id,
            "timestamp": data.timestamp.to_rfc3339(),
            "tracks": data.all_tracks.iter().map(|track| {
                serde_json::json!({
                    "name": track.track_name,
                    "room": track.room_name,
                    "value": track.display_value,
                    "unit": track.unit,
                    "type": format!("{:?}", track.track_type),
                })
            }).collect::<Vec<_>>()
        });

        let json_str = serde_json::to_string(&simplified)?;
        Ok(json_str.into_bytes())
    }
}
