use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Raw vital data received from VitalRecorder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalData {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub rooms: Vec<VitalRoom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalRoom {
    #[serde(rename = "roomId")]
    pub room_id: i32,
    #[serde(rename = "roomName")]
    pub room_name: String,
    pub tracks: Option<Vec<VitalTrack>>,
    #[serde(rename = "roomTimestamp")]
    pub room_timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalTrack {
    #[serde(rename = "trackId")]
    pub track_id: String,
    #[serde(rename = "trackName")]
    pub track_name: String,
    #[serde(rename = "trackType")]
    pub track_type: String,
    pub unit: String,
    #[serde(rename = "minValue")]
    pub min_value: Option<f64>,
    #[serde(rename = "maxValue")]
    pub max_value: Option<f64>,
    #[serde(rename = "trackTimestamp")]
    pub track_timestamp: Option<i64>,
    pub records: Option<Vec<VitalRecord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalRecord {
    pub value: serde_json::Value,
    pub timestamp: i64,
    #[serde(rename = "recordExtra")]
    pub record_extra: Option<serde_json::Value>,
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
    pub room_id: i32,
    pub room_name: String,
    pub tracks: Vec<ProcessedTrack>,
}

#[derive(Debug, Clone)]
pub struct ProcessedTrack {
    pub track_id: String,
    pub track_name: String,
    pub room_name: String,
    pub track_type: TrackType,
    pub unit: String,
    pub display_value: String,
    pub raw_value: Option<f64>,
    pub waveform_stats: Option<WaveformStats>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackType {
    Number,
    Waveform,
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
