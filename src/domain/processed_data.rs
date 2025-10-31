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
        let all_tracks = rooms.iter().flat_map(|room| room.tracks.clone()).collect();

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
    use chrono::Utc;

    /// ID SRS: SRS-TEST-PDATA-001
    /// Title: Test ProcessedData creation
    ///
    /// Description: VRConnect shall create ProcessedData with device_id,
    /// rooms, and automatically flatten all tracks.
    ///
    /// Version: V1.0
    #[test]
    fn test_processed_data_new() {
        let track1 = ProcessedTrack {
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

        let room = ProcessedRoom {
            room_index: 0,
            room_name: "BED_01".to_string(),
            tracks: vec![track1.clone()],
        };

        let data = ProcessedData::new("VR-TEST".to_string(), vec![room]);

        assert_eq!(data.device_id, "VR-TEST");
        assert_eq!(data.rooms.len(), 1);
        assert_eq!(data.all_tracks.len(), 1);
        assert_eq!(data.all_tracks[0].name, "HR");
    }

    /// ID SRS: SRS-TEST-PDATA-002
    /// Title: Test ProcessedData with multiple rooms
    ///
    /// Description: VRConnect shall flatten tracks from all rooms into
    /// all_tracks vector.
    ///
    /// Version: V1.0
    #[test]
    fn test_processed_data_multiple_rooms() {
        let track1 = ProcessedTrack {
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

        let track2 = ProcessedTrack {
            name: "SpO2".to_string(),
            display_value: "98.000".to_string(),
            raw_value: Some(98.0),
            unit: "%".to_string(),
            timestamp: Utc::now(),
            room_index: 1,
            room_name: "BED_02".to_string(),
            track_index: 0,
            record_index: 0,
            track_type: TrackType::Number,
            waveform_stats: None,
            waveform_points: None,
        };

        let room1 = ProcessedRoom {
            room_index: 0,
            room_name: "BED_01".to_string(),
            tracks: vec![track1],
        };

        let room2 = ProcessedRoom {
            room_index: 1,
            room_name: "BED_02".to_string(),
            tracks: vec![track2],
        };

        let data = ProcessedData::new("VR-TEST".to_string(), vec![room1, room2]);

        assert_eq!(data.rooms.len(), 2);
        assert_eq!(data.all_tracks.len(), 2);
        assert_eq!(data.all_tracks[0].room_name, "BED_01");
        assert_eq!(data.all_tracks[1].room_name, "BED_02");
    }

    /// ID SRS: SRS-TEST-PDATA-003
    /// Title: Test ProcessedTrack with Number type
    ///
    /// Description: VRConnect shall create ProcessedTrack for numeric values
    /// with proper display formatting.
    ///
    /// Version: V1.0
    #[test]
    fn test_processed_track_number() {
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

        assert_eq!(track.track_type, TrackType::Number);
        assert_eq!(track.raw_value, Some(75.0));
        assert!(track.waveform_stats.is_none());
        assert!(track.waveform_points.is_none());
    }

    /// ID SRS: SRS-TEST-PDATA-004
    /// Title: Test ProcessedTrack with Waveform type
    ///
    /// Description: VRConnect shall create ProcessedTrack for waveform data
    /// with statistics and points array.
    ///
    /// Version: V1.0
    #[test]
    fn test_processed_track_waveform() {
        let points = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = WaveformStats {
            min: 1.0,
            max: 5.0,
            avg: 3.0,
            count: 5,
        };

        let track = ProcessedTrack {
            name: "ECG".to_string(),
            display_value: "5 points (1.000 to 5.000, avg: 3.000)".to_string(),
            raw_value: None,
            unit: "mV".to_string(),
            timestamp: Utc::now(),
            room_index: 0,
            room_name: "BED_01".to_string(),
            track_index: 0,
            record_index: 0,
            track_type: TrackType::Waveform,
            waveform_stats: Some(stats.clone()),
            waveform_points: Some(points.clone()),
        };

        assert_eq!(track.track_type, TrackType::Waveform);
        assert!(track.raw_value.is_none());
        assert!(track.waveform_stats.is_some());
        assert_eq!(track.waveform_points.as_ref().unwrap().len(), 5);

        let stored_stats = track.waveform_stats.unwrap();
        assert_eq!(stored_stats.min, 1.0);
        assert_eq!(stored_stats.max, 5.0);
        assert_eq!(stored_stats.avg, 3.0);
        assert_eq!(stored_stats.count, 5);
    }

    /// ID SRS: SRS-TEST-PDATA-005
    /// Title: Test ProcessedTrack with String type
    ///
    /// Description: VRConnect shall create ProcessedTrack for string values.
    ///
    /// Version: V1.0
    #[test]
    fn test_processed_track_string() {
        let track = ProcessedTrack {
            name: "ALARM_MSG".to_string(),
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

        assert_eq!(track.track_type, TrackType::String);
        assert_eq!(track.display_value, "HR High");
        assert!(track.raw_value.is_none());
    }

    /// ID SRS: SRS-TEST-PDATA-006
    /// Title: Test TrackType serialization
    ///
    /// Description: VRConnect shall serialize/deserialize TrackType enum
    /// in lowercase format.
    ///
    /// Version: V1.0
    #[test]
    fn test_track_type_serialization() {
        use serde_json;

        assert_eq!(
            serde_json::to_string(&TrackType::Number).unwrap(),
            "\"number\""
        );
        assert_eq!(
            serde_json::to_string(&TrackType::Waveform).unwrap(),
            "\"waveform\""
        );
        assert_eq!(
            serde_json::to_string(&TrackType::String).unwrap(),
            "\"string\""
        );
        assert_eq!(
            serde_json::to_string(&TrackType::Other).unwrap(),
            "\"other\""
        );
    }

    /// ID SRS: SRS-TEST-PDATA-007
    /// Title: Test WaveformStats serialization
    ///
    /// Description: VRConnect shall serialize/deserialize WaveformStats
    /// with min, max, avg, count fields.
    ///
    /// Version: V1.0
    #[test]
    fn test_waveform_stats_serialization() {
        let stats = WaveformStats {
            min: -0.5,
            max: 1.5,
            avg: 0.5,
            count: 100,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: WaveformStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.min, stats.min);
        assert_eq!(deserialized.max, stats.max);
        assert_eq!(deserialized.avg, stats.avg);
        assert_eq!(deserialized.count, stats.count);
    }

    /// ID SRS: SRS-TEST-PDATA-008
    /// Title: Test ProcessedData with empty rooms
    ///
    /// Description: VRConnect shall handle empty rooms vector correctly.
    ///
    /// Version: V1.0
    #[test]
    fn test_processed_data_empty_rooms() {
        let data = ProcessedData::new("VR-EMPTY".to_string(), vec![]);

        assert_eq!(data.device_id, "VR-EMPTY");
        assert_eq!(data.rooms.len(), 0);
        assert_eq!(data.all_tracks.len(), 0);
    }
}
