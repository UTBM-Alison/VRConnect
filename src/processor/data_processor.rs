use crate::domain::VitalData;
use crate::error::{Result, VitalError};
use regex::Regex;
use std::sync::OnceLock;
use tracing::{debug, trace};

static NAN_INFINITY_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct VitalDataProcessor;

impl VitalDataProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process raw JSON string: clean and parse into VitalData
    pub fn process(&self, json_str: &str) -> Result<VitalData> {
        let cleaned = self.clean_json(json_str);
        trace!("Cleaned JSON: {}", cleaned);

        let vital_data: VitalData = serde_json::from_str(&cleaned)?;
        debug!("Successfully parsed VitalData for device: {}", vital_data.device_id);

        Ok(vital_data)
    }

    /// Clean JSON by:
    /// - Removing control characters
    /// - Replacing NaN/Infinity with null
    /// - Normalizing decimal separators (comma to dot)
    fn clean_json(&self, json_str: &str) -> String {
        // Remove control characters except for newlines and tabs that might be in strings
        let mut cleaned = json_str
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect::<String>();

        // Replace NaN and Infinity variants with null
        let regex = NAN_INFINITY_REGEX.get_or_init(|| {
            Regex::new(r"(?i)\b(NaN|[-+]?Infinity)\b").expect("Invalid regex")
        });
        cleaned = regex.replace_all(&cleaned, "null").to_string();

        // Normalize decimal separator: replace comma with dot in numeric contexts
        cleaned = self.normalize_decimals(&cleaned);

        cleaned
    }

    fn normalize_decimals(&self, s: &str) -> String {
        // Replace patterns like "123,456" with "123.456" when followed by digits
        let re = Regex::new(r"(\d+),(\d+)").expect("Invalid regex");
        re.replace_all(s, "$1.$2").to_string()
    }
}

impl Default for VitalDataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for VitalDataProcessor {
    fn clone(&self) -> Self {
        Self::new()
    }
}
