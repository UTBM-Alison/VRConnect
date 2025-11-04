// /src/domain/vital_data.rs
// Module: domain.vital_data
// Purpose: Raw vital data structures from VitalRecorder

use serde::{Deserialize, Serialize};

/// ID SRS: SRS-MOD-VITALDATA-001
/// Title: VitalData
///
/// Description: VRConnect shall define structures to deserialize raw vital data
/// received from VitalRecorder via Socket.IO with flexible field types.
///
/// Version: V1.0

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalData {
    #[serde(rename = "vrcode")]
    pub vr_code: String,
    pub rooms: Vec<VitalRoom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalRoom {
    #[serde(rename = "seqid")]
    pub seq_id: Option<i32>,

    #[serde(rename = "roomname")]
    pub room_name: Option<String>,

    #[serde(rename = "trks", default)]
    pub tracks: Vec<VitalTrack>,

    #[serde(rename = "evts", default)]
    pub events: Vec<VitalEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalTrack {
    #[serde(rename = "id", default, deserialize_with = "deserialize_flexible_id")]
    pub id: Option<String>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "type")]
    pub track_type: Option<String>,

    #[serde(rename = "unit")]
    pub unit: Option<String>,

    #[serde(rename = "montype")]
    pub mon_type: Option<String>,

    #[serde(rename = "dname")]
    pub display_name: Option<String>,

    #[serde(rename = "srate")]
    pub sample_rate: Option<f64>,

    #[serde(rename = "recs", default)]
    pub records: Vec<VitalRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalRecord {
    #[serde(rename = "val")]
    pub value: serde_json::Value,

    #[serde(
        rename = "dt",
        default,
        deserialize_with = "deserialize_flexible_timestamp"
    )]
    pub timestamp: Option<i64>,

    #[serde(
        rename = "time",
        default,
        deserialize_with = "deserialize_flexible_timestamp"
    )]
    pub time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalEvent {
    #[serde(
        rename = "dt",
        default,
        deserialize_with = "deserialize_flexible_timestamp"
    )]
    pub timestamp: Option<i64>,

    #[serde(rename = "msg")]
    pub message: Option<String>,
}

impl VitalRecord {
    /// ID SRS: SRS-FN-VITALRECORD-001
    /// Title: get_effective_timestamp
    ///
    /// Description: VRConnect shall extract the effective timestamp from a record,
    /// preferring 'timestamp' field over 'time' field.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Optional timestamp in milliseconds
    pub fn get_effective_timestamp(&self) -> Option<i64> {
        self.timestamp.or(self.time)
    }
}

/// ID SRS: SRS-FN-DESERIALIZE-001
/// Title: deserialize_flexible_id
///
/// Description: VRConnect shall deserialize ID field accepting both String
/// and Integer types, converting integers to strings.
///
/// Version: V1.0
fn deserialize_flexible_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FlexibleId {
        String(String),
        Int(i64),
    }

    let value = Option::<FlexibleId>::deserialize(deserializer)?;
    Ok(value.map(|v| match v {
        FlexibleId::String(s) => s,
        FlexibleId::Int(i) => i.to_string(),
    }))
}

/// ID SRS: SRS-FN-DESERIALIZE-002
/// Title: deserialize_flexible_timestamp
///
/// Description: VRConnect shall deserialize timestamp field accepting both
/// Integer and Float types, converting floats to integers.
///
/// Version: V1.0
fn deserialize_flexible_timestamp<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FlexibleTimestamp {
        Int(i64),
        Float(f64),
    }

    let value = Option::<FlexibleTimestamp>::deserialize(deserializer)?;
    Ok(value.map(|v| match v {
        FlexibleTimestamp::Int(i) => i,
        FlexibleTimestamp::Float(f) => f as i64,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// ID SRS: SRS-TEST-DOM-001
    /// Title: Test VitalData deserialization
    ///
    /// Description: VRConnect shall correctly deserialize VitalData from JSON
    /// containing vrcode and rooms array.
    ///
    /// Version: V1.0
    #[test]
    fn test_vital_data_deserialize() {
        let json = json!({
            "vrcode": "VR-12345",
            "rooms": []
        });

        let vital_data: VitalData = serde_json::from_value(json).unwrap();
        assert_eq!(vital_data.vr_code, "VR-12345");
        assert_eq!(vital_data.rooms.len(), 0);
    }

    /// ID SRS: SRS-TEST-DOM-002
    /// Title: Test VitalRoom deserialization
    ///
    /// Description: VRConnect shall correctly deserialize VitalRoom with
    /// seqid, roomname, tracks, and events.
    ///
    /// Version: V1.0
    #[test]
    fn test_vital_room_deserialize() {
        let json = json!({
            "seqid": 1,
            "roomname": "BED_01",
            "trks": [],
            "evts": []
        });

        let room: VitalRoom = serde_json::from_value(json).unwrap();
        assert_eq!(room.seq_id, Some(1));
        assert_eq!(room.room_name, Some("BED_01".to_string()));
        assert_eq!(room.tracks.len(), 0);
        assert_eq!(room.events.len(), 0);
    }

    /// ID SRS: SRS-TEST-DOM-003
    /// Title: Test VitalTrack deserialization with flexible ID
    ///
    /// Description: VRConnect shall deserialize VitalTrack with ID as either
    /// string or integer using flexible_id deserializer.
    ///
    /// Version: V1.0
    #[test]
    fn test_vital_track_flexible_id() {
        // Test with string ID
        let json_str = json!({
            "id": "track-123",
            "name": "ECG",
            "type": "wav",
            "unit": "mV",
            "recs": []
        });

        let track: VitalTrack = serde_json::from_value(json_str).unwrap();
        assert_eq!(track.id, Some("track-123".to_string()));

        // Test with integer ID
        let json_int = json!({
            "id": 456,
            "name": "HR",
            "type": "num",
            "unit": "bpm",
            "recs": []
        });

        let track: VitalTrack = serde_json::from_value(json_int).unwrap();
        assert_eq!(track.id, Some("456".to_string()));
    }

    /// ID SRS: SRS-TEST-DOM-004
    /// Title: Test VitalRecord with flexible timestamp
    ///
    /// Description: VRConnect shall deserialize VitalRecord timestamps as
    /// either integer or float using flexible_timestamp deserializer.
    ///
    /// Version: V1.0
    #[test]
    fn test_vital_record_flexible_timestamp() {
        // Test with integer timestamp
        let json_int = json!({
            "val": 75.0,
            "dt": 1234567890,
            "time": 1234567890
        });

        let record: VitalRecord = serde_json::from_value(json_int).unwrap();
        assert_eq!(record.timestamp, Some(1234567890));
        assert_eq!(record.time, Some(1234567890));

        // Test with float timestamp
        let json_float = json!({
            "val": 75.0,
            "dt": 1234567890.5,
            "time": 1234567890.5
        });

        let record: VitalRecord = serde_json::from_value(json_float).unwrap();
        assert_eq!(record.timestamp, Some(1234567890));
        assert_eq!(record.time, Some(1234567890));
    }

    /// ID SRS: SRS-TEST-DOM-005
    /// Title: Test VitalRecord get_effective_timestamp
    ///
    /// Description: VRConnect shall return dt field if present, otherwise
    /// fall back to time field.
    ///
    /// Version: V1.0
    #[test]
    fn test_get_effective_timestamp() {
        // Test with both dt and time
        let record1 = VitalRecord {
            value: json!(75.0),
            timestamp: Some(1000),
            time: Some(2000),
        };
        assert_eq!(record1.get_effective_timestamp(), Some(1000));

        // Test with only time
        let record2 = VitalRecord {
            value: json!(75.0),
            timestamp: None,
            time: Some(2000),
        };
        assert_eq!(record2.get_effective_timestamp(), Some(2000));

        // Test with neither
        let record3 = VitalRecord {
            value: json!(75.0),
            timestamp: None,
            time: None,
        };
        assert_eq!(record3.get_effective_timestamp(), None);
    }

    /// ID SRS: SRS-TEST-DOM-006
    /// Title: Test VitalEvent deserialization
    ///
    /// Description: VRConnect shall correctly deserialize VitalEvent with
    /// timestamp and message.
    ///
    /// Version: V1.0
    #[test]
    fn test_vital_event_deserialize() {
        let json = json!({
            "dt": 1234567890,
            "msg": "Alarm triggered"
        });

        let event: VitalEvent = serde_json::from_value(json).unwrap();
        assert_eq!(event.timestamp, Some(1234567890));
        assert_eq!(event.message, Some("Alarm triggered".to_string()));
    }

    /// ID SRS: SRS-TEST-DOM-007
    /// Title: Test complete VitalData structure
    ///
    /// Description: VRConnect shall deserialize complete VitalData with
    /// nested rooms, tracks, records, and events.
    ///
    /// Version: V1.0
    #[test]
    fn test_complete_vital_data() {
        let json = json!({
            "vrcode": "VR-TEST",
            "rooms": [
                {
                    "seqid": 0,
                    "roomname": "BED_01",
                    "trks": [
                        {
                            "id": "1",
                            "name": "HR",
                            "type": "num",
                            "unit": "bpm",
                            "recs": [
                                {
                                    "val": 75.0,
                                    "dt": 1234567890
                                }
                            ]
                        }
                    ],
                    "evts": [
                        {
                            "dt": 1234567890,
                            "msg": "Test event"
                        }
                    ]
                }
            ]
        });

        let vital_data: VitalData = serde_json::from_value(json).unwrap();
        assert_eq!(vital_data.vr_code, "VR-TEST");
        assert_eq!(vital_data.rooms.len(), 1);
        assert_eq!(vital_data.rooms[0].tracks.len(), 1);
        assert_eq!(vital_data.rooms[0].tracks[0].records.len(), 1);
        assert_eq!(vital_data.rooms[0].events.len(), 1);
    }

    /// ID SRS: SRS-TEST-DOM-008
    /// Title: Test array value in VitalRecord
    ///
    /// Description: VRConnect shall handle array values for waveform data.
    ///
    /// Version: V1.0
    #[test]
    fn test_vital_record_array_value() {
        let json = json!({
            "val": [1.0, 2.0, 3.0, 4.0, 5.0],
            "dt": 1234567890
        });

        let record: VitalRecord = serde_json::from_value(json).unwrap();
        assert!(record.value.is_array());

        let arr = record.value.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_f64().unwrap(), 1.0);
        assert_eq!(arr[4].as_f64().unwrap(), 5.0);
    }
}
