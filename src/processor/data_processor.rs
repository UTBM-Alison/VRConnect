use crate::domain::VitalData;
use crate::error::Result;
use fancy_regex::Regex;
use tracing::debug;

#[derive(Clone)]
pub struct VitalDataProcessor {
    control_chars: Regex,
    nan_pattern: Regex,
    infinity_pattern: Regex,
    decimal_obj: Regex,
    decimal_arr: Regex,
}

impl VitalDataProcessor {
    pub fn new() -> Self {
        Self {
            control_chars: Regex::new(r"[\x00-\x1F\x7F]").unwrap(),
            // Exact Java patterns with lookahead/lookbehind
            nan_pattern: Regex::new(r"(?i)(?<!\w)[+\-]?nan(?!\w)").unwrap(),
            infinity_pattern: Regex::new(r"(?i)(?<!\w)[+\-]?(?:inf|infinity)(?!\w)").unwrap(),
            // Fix decimal separators
            decimal_obj: Regex::new(r#"(":\s*-?\d+),(\d+)"#).unwrap(),
            decimal_arr: Regex::new(r"(\[|,\s*)(-?\d+),(\d+)(?=[,\]])").unwrap(),
        }
    }

    pub fn process(&self, json_str: &str) -> Result<VitalData> {
        let cleaned = self.clean_json_string(json_str)?;
        let vital_data: VitalData = serde_json::from_str(&cleaned)?;
        
        debug!("Successfully parsed VitalData for device: {}", vital_data.vr_code);
        Ok(vital_data)
    }

    fn clean_json_string(&self, json_str: &str) -> Result<String> {
        // Remove control characters
        let mut cleaned = self.control_chars
            .replace_all(json_str, "")
            .to_string();

        // Replace NaN and Infinity with null using exact Java patterns
        cleaned = self.nan_pattern
            .replace_all(&cleaned, "null")
            .to_string();
        
        cleaned = self.infinity_pattern
            .replace_all(&cleaned, "null")
            .to_string();

        // Fix decimal separators - step 1: object values like "key": 123,456
        cleaned = self.decimal_obj
            .replace_all(&cleaned, "$1.$2")
            .to_string();

        // Fix decimal separators - step 2: array values like [123,456]
        cleaned = self.decimal_arr
            .replace_all(&cleaned, "$1$2.$3")
            .to_string();

        Ok(cleaned)
    }
}

impl Default for VitalDataProcessor {
    fn default() -> Self {
        Self::new()
    }
}
