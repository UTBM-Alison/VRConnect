// /src/domain/processed_data.rs
// Module: domain.processed_data
// Purpose: Processed vital data structures for output

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ID SRS: SRS-MOD-PROCESSEDDATA-001
/// Title: ProcessedData
///
/// Description: VRConnect shall define structures for processed vital data
/// ready for output, including computed statistics and type classification.
///
/// Version: V1.0

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    pub device_id: String,
    pub rooms: Vec<ProcessedRoom>,
    pub all_tracks: Vec<ProcessedTrack>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedRoom {
    pub room_index: i32,
    pub room_name: String,
    pub tracks: Vec<ProcessedTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedTrack {
    pub name: String,
    pub display_value: String,
    pub raw_value: Option<f64>,
    pub unit: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub room_index: i32,
    pub room_name: String,
    pub track_index: i32,
    pub record_index: i32,
    pub track_type: TrackType,
    pub waveform_stats: Option<WaveformStats>,
    pub waveform_points: Option<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackType {
    Number,
    Waveform,
    String,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub count: usize,
}

impl ProcessedData {
    /// ID SRS: SRS-FN-PROCESSEDDATA-001
    /// Title: new
    ///
    /// Description: VRConnect shall construct a ProcessedData instance from
    /// device ID and rooms, flattening all tracks for easy access.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `device_id` - VitalRecorder device identifier
    /// * `rooms` - Vector of processed rooms
    ///
    /// # Returns
    /// New ProcessedData instance
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

    /// ID SRS: SRS-FN-PROCESSEDDATA-002
    /// Title: get_non_waveform_tracks
    ///
    /// Description: VRConnect shall filter and return only non-waveform tracks
    /// (Number, String, Other types) for BLE transmission.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// Vector of non-waveform tracks
    pub fn get_non_waveform_tracks(&self) -> Vec<&ProcessedTrack> {
        self.all_tracks
            .iter()
            .filter(|track| track.track_type != TrackType::Waveform)
            .collect()
    }
}

impl ProcessedTrack {
    /// ID SRS: SRS-FN-PROCESSEDTRACK-001
    /// Title: is_waveform
    ///
    /// Description: VRConnect shall determine if a track contains waveform data.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// True if track is waveform type
    #[allow(dead_code)]
    pub fn is_waveform(&self) -> bool {
        self.track_type == TrackType::Waveform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processed_data_new() {
        // TODO: Implement ProcessedData construction test
        assert!(true);
    }

    #[test]
    fn test_get_non_waveform_tracks() {
        // TODO: Implement non-waveform filtering test
        assert!(true);
    }

    #[test]
    fn test_is_waveform() {
        // TODO: Implement waveform detection test
        assert!(true);
    }

    #[test]
    fn test_track_type_serialization() {
        // TODO: Implement TrackType serialization test
        assert!(true);
    }
}
