use crate::domain::*;
use chrono::{TimeZone, Utc};
use tracing::debug;

#[derive(Clone)]
pub struct VitalDataTransformer;

impl VitalDataTransformer {
    pub fn new() -> Self {
        Self
    }

    pub fn transform(&self, vital_data: VitalData) -> ProcessedData {
        let mut processed_rooms = Vec::new();
        let mut all_tracks = Vec::new();

        debug!("Transforming data with {} rooms", vital_data.rooms.len());

        if !vital_data.rooms.is_empty() {
            for (room_index, room) in vital_data.rooms.iter().enumerate() {
                let room_name = room.room_name.clone()
                    .unwrap_or_else(|| format!("Room {}", room_index));
                
                let mut room_tracks = Vec::new();

                // Now tracks is a Vec, not Option<Vec>
                debug!("Room {} has {} tracks", room_index, room.tracks.len());
                
                for (track_index, track) in room.tracks.iter().enumerate() {
                    // Now records is a Vec, not Option<Vec>
                    debug!("Track {} has {} records", track_index, track.records.len());
                    
                    for (record_index, record) in track.records.iter().enumerate() {
                        let processed_track = self.process_track(
                            track,
                            record,
                            room_index as i32,
                            &room_name,
                            track_index as i32,
                            record_index as i32,
                        );
                        
                        room_tracks.push(processed_track.clone());
                        all_tracks.push(processed_track);
                    }
                }

                processed_rooms.push(ProcessedRoom {
                    room_index: room_index as i32,
                    room_name: room_name.clone(),
                    tracks: room_tracks,
                });
            }
        }

        debug!("Transformation complete: {} total tracks", all_tracks.len());

        ProcessedData {
            device_id: vital_data.vr_code,
            rooms: processed_rooms,
            all_tracks,
            timestamp: Utc::now(),
        }
    }

    fn process_track(
        &self,
        track: &VitalTrack,
        record: &VitalRecord,
        room_index: i32,
        room_name: &str,
        track_index: i32,
        record_index: i32,
    ) -> ProcessedTrack {
        let track_name = track.name.clone()
            .or_else(|| track.display_name.clone())
            .unwrap_or_else(|| format!("Track-{}-{}", room_index, track_index));
        
        let track_type_str = track.track_type.as_deref().unwrap_or("other");
        let unit = track.unit.clone().unwrap_or_default();

        let (track_type, display_value, raw_value, waveform_stats) = 
            self.process_value(&record.value, track_type_str);

        let timestamp = record.get_effective_timestamp()
            .and_then(|ts| Utc.timestamp_millis_opt(ts).single())
            .unwrap_or_else(Utc::now);

        ProcessedTrack {
            name: track_name,
            display_value,
            raw_value,
            unit,
            timestamp,
            room_index,
            room_name: room_name.to_string(),
            track_index,
            record_index,
            track_type,
            waveform_stats,
        }
    }

    fn process_value(
        &self,
        value: &serde_json::Value,
        track_type: &str,
    ) -> (TrackType, String, Option<f64>, Option<WaveformStats>) {
        match value {
            serde_json::Value::Number(n) => {
                let num = n.as_f64().unwrap_or(0.0);
                (
                    TrackType::Number,
                    format!("{:.3}", num),
                    Some(num),
                    None,
                )
            }
            serde_json::Value::Array(arr) if track_type == "wav" => {
                let numbers: Vec<f64> = arr
                    .iter()
                    .filter_map(|v| v.as_f64())
                    .collect();

                if numbers.is_empty() {
                    return (TrackType::Waveform, "0 points".to_string(), None, None);
                }

                let min = numbers.iter().copied().fold(f64::INFINITY, f64::min);
                let max = numbers.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let sum: f64 = numbers.iter().sum();
                let avg = sum / numbers.len() as f64;
                let count = numbers.len();

                let stats = WaveformStats { min, max, avg, count };
                let display = format!(
                    "{} points ({:.3} to {:.3}, avg: {:.3})",
                    count, min, max, avg
                );

                (TrackType::Waveform, display, None, Some(stats))
            }
            serde_json::Value::String(s) => {
                (TrackType::String, s.clone(), None, None)
            }
            _ => {
                (TrackType::Other, value.to_string(), None, None)
            }
        }
    }
}

impl Default for VitalDataTransformer {
    fn default() -> Self {
        Self::new()
    }
}
