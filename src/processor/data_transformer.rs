use crate::domain::{ProcessedData, ProcessedRoom, ProcessedTrack, TrackType, VitalData, WaveformStats};
use chrono::{DateTime, Utc};
use tracing::debug;

pub struct VitalDataTransformer;

impl VitalDataTransformer {
    pub fn new() -> Self {
        Self
    }

    pub fn transform(&self, vital_data: VitalData) -> ProcessedData {
        let rooms = vital_data
            .rooms
            .into_iter()
            .map(|room| self.transform_room(room))
            .collect();

        ProcessedData::new(vital_data.device_id, rooms)
    }

    fn transform_room(&self, room: crate::domain::VitalRoom) -> ProcessedRoom {
        let tracks = room
            .tracks
            .unwrap_or_default()
            .into_iter()
            .filter_map(|track| self.transform_track(track, &room.room_name))
            .collect();

        ProcessedRoom {
            room_id: room.room_id,
            room_name: room.room_name,
            tracks,
        }
    }

    fn transform_track(
        &self,
        track: crate::domain::VitalTrack,
        room_name: &str,
    ) -> Option<ProcessedTrack> {
        let records = track.records?;
        if records.is_empty() {
            return None;
        }

        let record = &records[0];
        let timestamp = DateTime::from_timestamp_millis(record.timestamp)
            .unwrap_or_else(Utc::now);

        match &record.value {
            serde_json::Value::Number(num) => {
                let value = num.as_f64()?;
                Some(ProcessedTrack {
                    track_id: track.track_id,
                    track_name: track.track_name,
                    room_name: room_name.to_string(),
                    track_type: TrackType::Number,
                    unit: track.unit,
                    display_value: format!("{:.3}", value),
                    raw_value: Some(value),
                    waveform_stats: None,
                    timestamp,
                })
            }
            serde_json::Value::Array(arr) => {
                let values: Vec<f64> = arr
                    .iter()
                    .filter_map(|v| v.as_f64())
                    .collect();

                if values.is_empty() {
                    Some(ProcessedTrack {
                        track_id: track.track_id,
                        track_name: track.track_name,
                        room_name: room_name.to_string(),
                        track_type: TrackType::Waveform,
                        unit: track.unit,
                        display_value: "0 points".to_string(),
                        raw_value: None,
                        waveform_stats: None,
                        timestamp,
                    })
                } else {
                    let stats = self.calculate_waveform_stats(&values);
                    Some(ProcessedTrack {
                        track_id: track.track_id,
                        track_name: track.track_name,
                        room_name: room_name.to_string(),
                        track_type: TrackType::Waveform,
                        unit: track.unit,
                        display_value: format!(
                            "{} points [min: {:.3}, max: {:.3}, avg: {:.3}]",
                            stats.count, stats.min, stats.max, stats.avg
                        ),
                        raw_value: None,
                        waveform_stats: Some(stats),
                        timestamp,
                    })
                }
            }
            other => {
                Some(ProcessedTrack {
                    track_id: track.track_id,
                    track_name: track.track_name,
                    room_name: room_name.to_string(),
                    track_type: TrackType::Other,
                    unit: track.unit,
                    display_value: format!("{}", other),
                    raw_value: None,
                    waveform_stats: None,
                    timestamp,
                })
            }
        }
    }

    fn calculate_waveform_stats(&self, values: &[f64]) -> WaveformStats {
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;

        WaveformStats {
            min,
            max,
            avg,
            count: values.len(),
        }
    }
}

impl Default for VitalDataTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for VitalDataTransformer {
    fn clone(&self) -> Self {
        Self::new()
    }
}
