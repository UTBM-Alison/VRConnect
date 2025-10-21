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
        let header = format!("[{}] Device: {}", data.timestamp.format("%H:%M:%S"), data.device_id);
        
        if self.colorized {
            println!("{}", header.bright_cyan().bold());
        } else {
            println!("{}", header);
        }

        for track in &data.all_tracks {
            let line = format!(
                "  {} [{:?}]: {} {}",
                track.track_name, track.track_type, track.display_value, track.unit
            );

            if self.colorized {
                let colored_line = match track.track_type {
                    TrackType::Number => line.green(),
                    TrackType::Waveform => line.blue(),
                    TrackType::Other => line.yellow(),
                };
                println!("{}", colored_line);
            } else {
                println!("{}", line);
            }
        }
        println!();
    }

    fn output_verbose(&self, data: &ProcessedData) {
        let divider = "=".repeat(80);
        
        if self.colorized {
            println!("{}", divider.bright_cyan());
            println!("{}", format!("VITAL DATA UPDATE - {}", data.timestamp).bright_cyan().bold());
            println!("{}", divider.bright_cyan());
        } else {
            println!("{}", divider);
            println!("VITAL DATA UPDATE - {}", data.timestamp);
            println!("{}", divider);
        }

        if self.colorized {
            println!("{}: {}", "Device ID".bright_yellow(), data.device_id.bright_white().bold());
        } else {
            println!("Device ID: {}", data.device_id);
        }

        for room in &data.rooms {
            println!();
            if self.colorized {
                println!("{} {} (ID: {})", 
                    "Room:".bright_magenta().bold(), 
                    room.room_name.bright_white().bold(),
                    room.room_id
                );
            } else {
                println!("Room: {} (ID: {})", room.room_name, room.room_id);
            }

            for track in &room.tracks {
                self.output_track_verbose(track);
            }
        }

        if self.colorized {
            println!("{}", divider.bright_cyan());
        } else {
            println!("{}", divider);
        }
        println!();
    }

    fn output_track_verbose(&self, track: &ProcessedTrack) {
        let indent = "    ";
        
        if self.colorized {
            println!("{}{}: {}", indent, "Track".bright_blue().bold(), track.track_name.bright_white());
            println!("{}  {} {}", indent, "ID:".dimmed(), track.track_id.dimmed());
            println!("{}  {} {:?}", indent, "Type:".dimmed(), track.track_type);
            println!("{}  {} {}", indent, "Value:".bright_green(), track.display_value.bright_white().bold());
            
            if !track.unit.is_empty() {
                println!("{}  {} {}", indent, "Unit:".dimmed(), track.unit);
            }
            
            if let Some(stats) = &track.waveform_stats {
                println!("{}  {} {} points", indent, "Waveform:".bright_blue(), stats.count);
                println!("{}    Min: {:.3}, Max: {:.3}, Avg: {:.3}", 
                    indent, stats.min, stats.max, stats.avg);
            }
            
            println!("{}  {} {}", indent, "Timestamp:".dimmed(), 
                track.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
        } else {
            println!("{}Track: {}", indent, track.track_name);
            println!("{}  ID: {}", indent, track.track_id);
            println!("{}  Type: {:?}", indent, track.track_type);
            println!("{}  Value: {}", indent, track.display_value);
            
            if !track.unit.is_empty() {
                println!("{}  Unit: {}", indent, track.unit);
            }
            
            if let Some(stats) = &track.waveform_stats {
                println!("{}  Waveform: {} points", indent, stats.count);
                println!("{}    Min: {:.3}, Max: {:.3}, Avg: {:.3}", 
                    indent, stats.min, stats.max, stats.avg);
            }
            
            println!("{}  Timestamp: {}", indent, track.timestamp.format("%Y-%m-%d %H:%M:%S"));
        }
    }
}
