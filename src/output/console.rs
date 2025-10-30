// /src/output/console.rs
// Module: output.console
// Purpose: Console output with compact and verbose modes

use crate::domain::{ProcessedData, ProcessedTrack, TrackType};

/// ID SRS: SRS-MOD-CONSOLE-001
/// Title: ConsoleOutput
///
/// Description: VRConnect shall provide console output with compact and verbose
/// modes, with optional colorization for improved readability.
///
/// Version: V1.0
pub struct ConsoleOutput {
    verbose: bool,
    _colorized: bool, // Keep for future use
}

impl ConsoleOutput {
    /// ID SRS: SRS-FN-CONSOLE-001
    /// Title: new
    ///
    /// Description: VRConnect shall construct a ConsoleOutput instance with
    /// verbosity and colorization configuration.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `verbose` - Enable verbose mode
    /// * `colorized` - Enable color output
    ///
    /// # Returns
    /// New ConsoleOutput instance
    pub fn new(verbose: bool, colorized: bool) -> Self {
        Self { 
            verbose,
            _colorized: colorized,
        }
    }

    /// ID SRS: SRS-FN-CONSOLE-002
    /// Title: output
    ///
    /// Description: VRConnect shall output ProcessedData to console in either
    /// compact or verbose format based on configuration.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `data` - Processed vital data to display
    pub async fn output(&self, data: &ProcessedData) {
        if self.verbose {
            self.output_verbose(data);
        } else {
            self.output_compact(data);
        }
    }

    /// ID SRS: SRS-FN-CONSOLE-003
    /// Title: output_compact
    ///
    /// Description: VRConnect shall display compact vital data summary showing
    /// timestamp, track count, and first 5 tracks with basic information.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `data` - Processed vital data
    fn output_compact(&self, data: &ProcessedData) {
        let header = format!(
            "[{}] 🏥 Vital Update - {} tracks",
            data.timestamp.format("%Y-%m-%dT%H:%M:%S%.9f"),
            data.all_tracks.len()
        );

        println!("{}", header);

        // Display first 5 tracks
        for track in data.all_tracks.iter().take(5) {
            self.print_track_compact(track);
        }

        // Show remaining count
        if data.all_tracks.len() > 5 {
            let remaining = data.all_tracks.len() - 5;
            println!("  ... and {} more tracks", remaining);
        }
    }

    /// ID SRS: SRS-FN-CONSOLE-004
    /// Title: output_verbose
    ///
    /// Description: VRConnect shall display detailed vital data including device
    /// info, room breakdown, and complete track details with ALL waveform points.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `data` - Processed vital data
    fn output_verbose(&self, data: &ProcessedData) {
        println!("\n{}", "═".repeat(60));
        println!("🏥 VITAL DATA UPDATE");
        println!("{}", "═".repeat(60));

        println!("Device: {}", data.device_id);
        println!(
            "Timestamp: {}",
            data.timestamp.format("%Y-%m-%d %H:%M:%S%.3f")
        );
        println!("Rooms: {}", data.rooms.len());
        println!("Total Tracks: {}", data.all_tracks.len());

        for room in &data.rooms {
            println!(
                "\nRoom: {} (Index: {})",
                room.room_name,
                room.room_index
            );

            for track in &room.tracks {
                self.print_track_verbose(track, "  ");
            }
        }

        println!("\n{}", "═".repeat(60));
    }

    /// ID SRS: SRS-FN-CONSOLE-005
    /// Title: print_track_compact
    ///
    /// Description: VRConnect shall print single track in compact format:
    /// name, value, unit, and room.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `track` - Track to display
    fn print_track_compact(&self, track: &ProcessedTrack) {
        println!(
            "  {}: {} {} ({})",
            track.name, track.display_value, track.unit, track.room_name
        );
    }

    /// ID SRS: SRS-FN-CONSOLE-006
    /// Title: print_track_verbose
    ///
    /// Description: VRConnect shall print single track in verbose format with
    /// all metadata, statistics, and ALL waveform points.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `track` - Track to display
    /// * `indent` - Indentation string
    fn print_track_verbose(&self, track: &ProcessedTrack, indent: &str) {
        println!("{}Track: {}", indent, track.name);
        println!("{}  Type: {:?}", indent, track.track_type);
        println!("{}  Value: {}", indent, track.display_value);
        println!("{}  Unit: {}", indent, track.unit);
        println!(
            "{}  Timestamp: {}",
            indent,
            track.timestamp.format("%H:%M:%S%.3f")
        );

        if let Some(stats) = &track.waveform_stats {
            println!(
                "{}  Stats: min={:.3}, max={:.3}, avg={:.3}, count={}",
                indent, stats.min, stats.max, stats.avg, stats.count
            );
        }

        // Print ALL waveform points in verbose mode
        if track.track_type == TrackType::Waveform {
            if let Some(points) = &track.waveform_points {
                println!("{}  Waveform Points ({} total):", indent, points.len());
                print!("{}    ", indent);
                
                for (i, point) in points.iter().enumerate() {
                    print!("{:.3}", point);
                    
                    // Format: 10 points per line
                    if (i + 1) % 10 == 0 && i + 1 < points.len() {
                        println!();
                        print!("{}    ", indent);
                    } else if i + 1 < points.len() {
                        print!(", ");
                    }
                }
                println!(); // Final newline
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_output_creation() {
        // TODO: Implement console creation test
        assert!(true);
    }

    #[test]
    fn test_compact_output() {
        // TODO: Implement compact output test
        assert!(true);
    }

    #[test]
    fn test_verbose_output() {
        // TODO: Implement verbose output test
        assert!(true);
    }

    #[test]
    fn test_track_formatting() {
        // TODO: Implement track formatting test
        assert!(true);
    }

    #[test]
    fn test_waveform_points_output() {
        // TODO: Implement waveform points console output test
        assert!(true);
    }
}
