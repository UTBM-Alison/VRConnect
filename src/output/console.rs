use crate::domain::{ProcessedData, ProcessedTrack, TrackType};
use colored::*;

pub struct ConsoleVitalOutput {
    verbose: bool,
    colorized: bool,
}

impl ConsoleVitalOutput {
    pub fn new(verbose: bool, colorized: bool) -> Self {
        Self { verbose, colorized }
    }

    pub async fn output(&self, data: &ProcessedData) {
        if self.verbose {
            self.output_verbose(data);
        } else {
            self.output_compact(data);
        }
    }

    fn output_compact(&self, data: &ProcessedData) {
        let header = format!(
            "[{}] 🏥 Vital Update - {} tracks",
            data.timestamp.format("%Y-%m-%dT%H:%M:%S%.9f"),
            data.all_tracks.len()
        );

        if self.colorized {
            println!("{}", header.bright_cyan().bold());
        } else {
            println!("{}", header);
        }

        // Show first 5 tracks
        for track in data.all_tracks.iter().take(5) {
            let line = format!(
                "  {}: {} {} ({})",
                // FIXED: Use 'name' instead of 'track_name'
                track.name, track.display_value, track.unit, track.room_name
            );

            if self.colorized {
                println!("{}", line);
            } else {
                println!("{}", line);
            }
        }

        if data.all_tracks.len() > 5 {
            let remaining = data.all_tracks.len() - 5;
            let line = format!("  ... and {} more tracks", remaining);
            
            if self.colorized {
                println!("{}", line.dimmed());
            } else {
                println!("{}", line);
            }
        }
    }

    fn output_verbose(&self, data: &ProcessedData) {
        if self.colorized {
            println!("\n{}", "═".repeat(60).bright_cyan());
            println!("{}", "🏥 VITAL DATA UPDATE".bright_cyan().bold());
            println!("{}", "═".repeat(60).bright_cyan());
        } else {
            println!("\n{}", "═".repeat(60));
            println!("🏥 VITAL DATA UPDATE");
            println!("{}", "═".repeat(60));
        }

        println!("Device: {}", data.device_id);
        println!("Timestamp: {}", data.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"));
        println!("Rooms: {}", data.rooms.len());
        println!("Total Tracks: {}", data.all_tracks.len());

        for room in &data.rooms {
            if self.colorized {
                println!(
                    "\n{} {} (Index: {})",
                    "Room:".bright_blue().bold(),
                    // FIXED: Use 'room_index' instead of 'room_id'
                    room.room_name.bright_white(),
                    room.room_index
                );
            } else {
                println!("\nRoom: {} (Index: {})", room.room_name, room.room_index);
            }

            for track in &room.tracks {
                self.print_track_verbose(track, "  ");
            }
        }

        if self.colorized {
            println!("\n{}", "═".repeat(60).bright_cyan());
        } else {
            println!("\n{}", "═".repeat(60));
        }
    }

    fn print_track_verbose(&self, track: &ProcessedTrack, indent: &str) {
        if self.colorized {
            // FIXED: Use 'name' instead of 'track_name'
            println!("{}{}: {}", indent, "Track".bright_blue().bold(), track.name.bright_white());
            println!("{}  {} {:?}", indent, "Type:".dimmed(), track.track_type);
            println!("{}  {} {}", indent, "Value:".dimmed(), track.display_value.bright_green());
            println!("{}  {} {}", indent, "Unit:".dimmed(), track.unit);
            println!("{}  {} {}", indent, "Timestamp:".dimmed(), track.timestamp.format("%H:%M:%S%.3f"));
            
            if let Some(stats) = &track.waveform_stats {
                println!("{}  {} min={:.3}, max={:.3}, avg={:.3}, count={}", 
                    indent,
                    "Stats:".dimmed(),
                    stats.min, stats.max, stats.avg, stats.count
                );
            }
        } else {
            // FIXED: Use 'name' instead of 'track_name'
            println!("{}Track: {}", indent, track.name);
            println!("{}  Type: {:?}", indent, track.track_type);
            println!("{}  Value: {}", indent, track.display_value);
            println!("{}  Unit: {}", indent, track.unit);
            println!("{}  Timestamp: {}", indent, track.timestamp.format("%H:%M:%S%.3f"));
            
            if let Some(stats) = &track.waveform_stats {
                println!("{}  Stats: min={:.3}, max={:.3}, avg={:.3}, count={}", 
                    indent, stats.min, stats.max, stats.avg, stats.count
                );
            }
        }
    }
}
