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
    
    #[serde(rename = "dt", default, deserialize_with = "deserialize_flexible_timestamp")]
    pub timestamp: Option<i64>,
    
    #[serde(rename = "time", default, deserialize_with = "deserialize_flexible_timestamp")]
    pub time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalEvent {
    #[serde(rename = "dt", default, deserialize_with = "deserialize_flexible_timestamp")]
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

    #[test]
    fn test_vital_data_deserialization() {
        // TODO: Implement VitalData deserialization test
        assert!(true);
    }

    #[test]
    fn test_flexible_id_string() {
        // TODO: Implement flexible ID with string test
        assert!(true);
    }

    #[test]
    fn test_flexible_id_int() {
        // TODO: Implement flexible ID with integer test
        assert!(true);
    }

    #[test]
    fn test_flexible_timestamp_int() {
        // TODO: Implement flexible timestamp with integer test
        assert!(true);
    }

    #[test]
    fn test_flexible_timestamp_float() {
        // TODO: Implement flexible timestamp with float test
        assert!(true);
    }

    #[test]
    fn test_get_effective_timestamp() {
        // TODO: Implement effective timestamp extraction test
        assert!(true);
    }
}
