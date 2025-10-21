use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Raw vital data received from VitalRecorder
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

// Custom deserializer for flexible ID field
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

// Custom deserializer for flexible timestamp (can be int or float)
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
    pub fn get_effective_timestamp(&self) -> Option<i64> {
        self.timestamp.or(self.time)
    }
}

/// Processed data ready for output
#[derive(Debug, Clone)]
pub struct ProcessedData {
    pub device_id: String,
    pub rooms: Vec<ProcessedRoom>,
    pub all_tracks: Vec<ProcessedTrack>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ProcessedRoom {
    pub room_index: i32,
    pub room_name: String,
    pub tracks: Vec<ProcessedTrack>,
}

#[derive(Debug, Clone)]
pub struct ProcessedTrack {
    pub name: String,
    pub display_value: String,
    pub raw_value: Option<f64>,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub room_index: i32,
    pub room_name: String,
    pub track_index: i32,
    pub record_index: i32,
    pub track_type: TrackType,
    pub waveform_stats: Option<WaveformStats>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackType {
    Number,
    Waveform,
    String,
    Other,
}

#[derive(Debug, Clone)]
pub struct WaveformStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub count: usize,
}

impl ProcessedData {
    pub fn new(device_id: String, rooms: Vec<ProcessedRoom>) -> Self {
        let all_tracks = rooms
            .iter()
            .flat_map(|room| room.tracks.clone())
            .collect();

        Self {
            device_id,
            rooms,
            all_tracks,
            timestamp: Utc::now(),
        }
    }
}
