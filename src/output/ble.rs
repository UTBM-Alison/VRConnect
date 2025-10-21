use crate::domain::ProcessedData;
use crate::error::{Result, VitalError};
use bluer::{
    adv::Advertisement,
    gatt::local::{
        Application, Characteristic, CharacteristicNotify, CharacteristicNotifyMethod,
        CharacteristicRead, CharacteristicWrite, CharacteristicWriteMethod, Service,
    },
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

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

        let app = self.create_application().await?;
        let app_handle = adapter.serve_gatt_application(app).await?;
        info!("GATT application registered");

        let adv = Advertisement {
            service_uuids: vec![VITAL_SERVICE_UUID].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some(self.device_name.clone()),
            ..Default::default()
        };

        let adv_handle = adapter.advertise(adv).await?;
        info!("Advertising started as '{}'", self.device_name);

        tokio::signal::ctrl_c().await.map_err(|e| VitalError::Io(e))?;

        drop(adv_handle);
        drop(app_handle);
        info!("BLE GATT server stopped");

        Ok(())
    }

    async fn create_application(&self) -> Result<Application> {
        let data_buffer = self.data_buffer.clone();
        let _notify_enabled = self.notify_enabled.clone();

        let data_char = Characteristic {
            uuid: VITAL_DATA_CHARACTERISTIC_UUID,
            read: Some(CharacteristicRead {
                read: true,
                fun: Box::new(move |_req| {
                    let data_buffer = data_buffer.clone();
                    Box::pin(async move {
                        let buffer = data_buffer.read().await;
                        match buffer.as_ref() {
                            Some(data) => {
                                debug!("BLE read request: sending {} bytes", data.len());
                                Ok(data.clone())
                            }
                            None => {
                                debug!("BLE read request: no data available");
                                Ok(Vec::new())
                            }
                        }
                    })
                }),
                ..Default::default()
            }),
            notify: Some(CharacteristicNotify {
                notify: true,
                method: CharacteristicNotifyMethod::Io,
                ..Default::default()
            }),
            ..Default::default()
        };

        let control_char = Characteristic {
            uuid: VITAL_CONTROL_CHARACTERISTIC_UUID,
            write: Some(CharacteristicWrite {
                write: true,
                write_without_response: false,
                method: CharacteristicWriteMethod::Io,
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
        let json_data = self.serialize_data(data)?;
        *self.data_buffer.write().await = Some(json_data.clone());

        if *self.notify_enabled.read().await {
            debug!("Sending {} bytes via BLE notification", json_data.len());
        }

        Ok(())
    }

    fn serialize_data(&self, data: &ProcessedData) -> Result<Vec<u8>> {
        let simplified = serde_json::json!({
            "deviceId": data.device_id,
            "timestamp": data.timestamp.to_rfc3339(),
            "tracks": data.all_tracks.iter().map(|track| {
                serde_json::json!({
                    "name": track.name,
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
