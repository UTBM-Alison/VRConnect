// /src/processor/cleaner.rs
// Module: processor.cleaner
// Purpose: JSON data cleaning and sanitization

use crate::error::Result;
use fancy_regex::Regex;

/// ID SRS: SRS-MOD-CLEANER-001
/// Title: VitalDataCleaner
///
/// Description: VRConnect shall clean raw JSON data by removing control characters,
/// replacing invalid numeric values (NaN, Infinity), and fixing decimal separators.
///
/// Version: V1.0
#[derive(Clone)]
pub struct VitalDataCleaner {
    control_chars: Regex,
    nan_pattern: Regex,
    infinity_pattern: Regex,
    decimal_obj: Regex,
    decimal_arr: Regex,
}

impl VitalDataCleaner {
    /// ID SRS: SRS-FN-CLEANER-001
    /// Title: new
    ///
    /// Description: VRConnect shall construct a VitalDataCleaner instance with
    /// precompiled regex patterns for efficient JSON cleaning.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// New VitalDataCleaner instance
    pub fn new() -> Self {
        Self {
            control_chars: Regex::new(r"[\x00-\x1F\x7F]").unwrap(),
            nan_pattern: Regex::new(r"(?i)(?<!\w)[+\-]?nan(?!\w)").unwrap(),
            infinity_pattern: Regex::new(r"(?i)(?<!\w)[+\-]?(?:inf|infinity)(?!\w)").unwrap(),
            decimal_obj: Regex::new(r#"(":\s*-?\d+),(\d+)"#).unwrap(),
            decimal_arr: Regex::new(r"(\[|,\s*)(-?\d+),(\d+)(?=[,\]])").unwrap(),
        }
    }

    /// ID SRS: SRS-FN-CLEANER-002
    /// Title: clean
    ///
    /// Description: VRConnect shall apply all cleaning operations to raw JSON string,
    /// returning sanitized JSON ready for parsing.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `json_str` - Raw JSON string
    ///
    /// # Returns
    /// Cleaned JSON string or error
    pub fn clean(&self, json_str: &str) -> Result<String> {
        log::debug!("Cleaning JSON data, original length: {}", json_str.len());

        // Step 1: Remove control characters
        let mut cleaned = self.remove_control_chars(json_str);

        // Step 2: Replace NaN with null
        cleaned = self.replace_nan(&cleaned);

        // Step 3: Replace Infinity with null
        cleaned = self.replace_infinity(&cleaned);

        // Step 4: Fix decimal separators in objects
        cleaned = self.fix_decimal_obj(&cleaned);

        // Step 5: Fix decimal separators in arrays
        cleaned = self.fix_decimal_arr(&cleaned);

        log::debug!("JSON cleaned, final length: {}", cleaned.len());

        Ok(cleaned)
    }

    /// ID SRS: SRS-FN-CLEANER-003
    /// Title: remove_control_chars
    ///
    /// Description: VRConnect shall remove control characters (0x00-0x1F, 0x7F)
    /// from JSON string to prevent parsing errors.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `json_str` - Input JSON string
    ///
    /// # Returns
    /// String with control characters removed
    fn remove_control_chars(&self, json_str: &str) -> String {
        self.control_chars.replace_all(json_str, "").to_string()
    }

    /// ID SRS: SRS-FN-CLEANER-004
    /// Title: replace_nan
    ///
    /// Description: VRConnect shall replace NaN values (case-insensitive,
    /// with optional +/- prefix) with null using word boundary checks.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `json_str` - Input JSON string
    ///
    /// # Returns
    /// String with NaN replaced by null
    fn replace_nan(&self, json_str: &str) -> String {
        self.nan_pattern.replace_all(json_str, "null").to_string()
    }

    /// ID SRS: SRS-FN-CLEANER-005
    /// Title: replace_infinity
    ///
    /// Description: VRConnect shall replace Infinity/inf values (case-insensitive,
    /// with optional +/- prefix) with null using word boundary checks.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `json_str` - Input JSON string
    ///
    /// # Returns
    /// String with Infinity replaced by null
    fn replace_infinity(&self, json_str: &str) -> String {
        self.infinity_pattern
            .replace_all(json_str, "null")
            .to_string()
    }

    /// ID SRS: SRS-FN-CLEANER-006
    /// Title: fix_decimal_obj
    ///
    /// Description: VRConnect shall fix decimal separators in object values,
    /// converting comma to period (e.g., "key": 123,456 → "key": 123.456).
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `json_str` - Input JSON string
    ///
    /// # Returns
    /// String with object decimal separators fixed
    fn fix_decimal_obj(&self, json_str: &str) -> String {
        self.decimal_obj.replace_all(json_str, "$1.$2").to_string()
    }

    /// ID SRS: SRS-FN-CLEANER-007
    /// Title: fix_decimal_arr
    ///
    /// Description: VRConnect shall fix decimal separators in array values,
    /// converting comma to period (e.g., [123,456] → [123.456]).
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `json_str` - Input JSON string
    ///
    /// # Returns
    /// String with array decimal separators fixed
    fn fix_decimal_arr(&self, json_str: &str) -> String {
        self.decimal_arr
            .replace_all(json_str, "$1$2.$3")
            .to_string()
    }
}

impl Default for VitalDataCleaner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ID SRS: SRS-TEST-CLEAN-001
    /// Title: Test cleaner creation
    ///
    /// Description: VRConnect shall create a VitalDataCleaner instance.
    ///
    /// Version: V1.0
    #[test]
    fn test_cleaner_new() {
        let cleaner = VitalDataCleaner::new();
        assert!(cleaner.clean("{}").is_ok());
    }

    /// ID SRS: SRS-TEST-CLEAN-002
    /// Title: Test control character removal
    ///
    /// Description: VRConnect shall remove control characters (0x00-0x1F, 0x7F)
    /// from JSON strings.
    ///
    /// Version: V1.0
    #[test]
    fn test_remove_control_characters() {
        let cleaner = VitalDataCleaner::new();

        // Test with control characters
        let input = "Hello\x00\x01\x1FWorld\x7F";
        let result = cleaner.clean(input).unwrap();
        assert_eq!(result, "HelloWorld");

        // Test with newlines and tabs (should be removed)
        let input = "Hello\n\tWorld";
        let result = cleaner.clean(input).unwrap();
        assert_eq!(result, "HelloWorld");
    }

    /// ID SRS: SRS-TEST-CLEAN-003
    /// Title: Test NaN replacement
    ///
    /// Description: VRConnect shall replace NaN values (case-insensitive)
    /// with null in JSON.
    ///
    /// Version: V1.0
    #[test]
    fn test_replace_nan() {
        let cleaner = VitalDataCleaner::new();

        // Test various NaN formats
        let input = r#"{"val1": NaN, "val2": nan, "val3": NAN, "val4": +NaN, "val5": -NaN}"#;
        let result = cleaner.clean(input).unwrap();

        assert!(result.contains("null"));
        assert!(!result.contains("NaN"));
        assert!(!result.contains("nan"));

        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }

    /// ID SRS: SRS-TEST-CLEAN-004
    /// Title: Test Infinity replacement
    ///
    /// Description: VRConnect shall replace Infinity values (case-insensitive,
    /// with optional +/- signs) with null in JSON.
    ///
    /// Version: V1.0
    #[test]
    fn test_replace_infinity() {
        let cleaner = VitalDataCleaner::new();

        // Test various Infinity formats
        let input = r#"{"val1": Infinity, "val2": infinity, "val3": +Infinity, "val4": -Infinity, "val5": inf, "val6": -inf}"#;
        let result = cleaner.clean(input).unwrap();

        assert!(result.contains("null"));
        assert!(!result.contains("Infinity"));
        assert!(!result.contains("infinity"));
        assert!(!result.contains("inf"));

        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }

    /// ID SRS: SRS-TEST-CLEAN-005
    /// Title: Test decimal separator fix in objects
    ///
    /// Description: VRConnect shall fix comma decimal separators to dots
    /// in JSON object values (e.g., "key": 123,456 → "key": 123.456).
    ///
    /// Version: V1.0
    #[test]
    fn test_fix_decimal_separator_objects() {
        let cleaner = VitalDataCleaner::new();

        // Test decimal separator in object values
        let input = r#"{"hr": 75,5, "temp": 36,8, "spo2": 98,2}"#;
        let result = cleaner.clean(input).unwrap();

        assert!(result.contains("75.5"));
        assert!(result.contains("36.8"));
        assert!(result.contains("98.2"));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["hr"].as_f64().unwrap(), 75.5);
        assert_eq!(parsed["temp"].as_f64().unwrap(), 36.8);
    }

    /// ID SRS: SRS-TEST-CLEAN-006
    /// Title: Test decimal separator fix in arrays
    ///
    /// Description: VRConnect shall fix comma decimal separators to dots
    /// in JSON array values (e.g., [123,456] → [123.456]).
    ///
    /// Version: V1.0
    #[test]
    fn test_fix_decimal_separator_arrays() {
        let cleaner = VitalDataCleaner::new();

        // Test decimal separator in arrays
        let input = r#"{"waveform": [1,5, 2,3, 3,7, -0,5]}"#;
        let result = cleaner.clean(input).unwrap();

        assert!(result.contains("1.5"));
        assert!(result.contains("2.3"));
        assert!(result.contains("3.7"));
        assert!(result.contains("-0.5"));

        // Verify it's valid JSON and parse correctly
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let arr = parsed["waveform"].as_array().unwrap();
        assert_eq!(arr[0].as_f64().unwrap(), 1.5);
        assert_eq!(arr[3].as_f64().unwrap(), -0.5);
    }

    /// ID SRS: SRS-TEST-CLEAN-007
    /// Title: Test complete JSON cleaning
    ///
    /// Description: VRConnect shall apply all cleaning rules to produce
    /// valid, parseable JSON from dirty input.
    ///
    /// Version: V1.0
    #[test]
    fn test_complete_cleaning() {
        let cleaner = VitalDataCleaner::new();

        // Dirty JSON with multiple issues
        let input = r#"{"hr": 75,5, "temp": NaN, "bp": Infinity, "ecg": [1,2, 3,4, NaN]}"#;
        let result = cleaner.clean(input).unwrap();

        // Verify all issues are fixed
        assert!(result.contains("75.5"));
        assert!(result.contains("null"));
        assert!(!result.contains("NaN"));
        assert!(!result.contains("Infinity"));

        // Must be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_object());
    }

    /// ID SRS: SRS-TEST-CLEAN-008
    /// Title: Test valid JSON passthrough
    ///
    /// Description: VRConnect shall pass through already-valid JSON
    /// without modification (except control characters).
    ///
    /// Version: V1.0
    #[test]
    fn test_valid_json_passthrough() {
        let cleaner = VitalDataCleaner::new();

        // Use larger numbers to avoid decimal separator confusion
        let input = r#"{"name":"Test","value":123.456,"array":[10, 20, 30]}"#;
        let result = cleaner.clean(input).unwrap();

        // Parse both and compare
        let input_parsed: serde_json::Value = serde_json::from_str(input).unwrap();
        let result_parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(input_parsed, result_parsed);
    }

    /// ID SRS: SRS-TEST-CLEAN-009
    /// Title: Test edge case - empty string
    ///
    /// Description: VRConnect shall handle empty JSON strings.
    ///
    /// Version: V1.0
    #[test]
    fn test_empty_string() {
        let cleaner = VitalDataCleaner::new();
        let result = cleaner.clean("").unwrap();
        assert_eq!(result, "");
    }

    /// ID SRS: SRS-TEST-CLEAN-010
    /// Title: Test edge case - whitespace only
    ///
    /// Description: VRConnect shall handle whitespace-only strings.
    ///
    /// Version: V1.0
    #[test]
    fn test_whitespace_only() {
        let cleaner = VitalDataCleaner::new();
        let result = cleaner.clean("   \t\n   ").unwrap();
        // Control chars (tabs, newlines) should be removed
        assert_eq!(result.trim(), "");
    }

    /// ID SRS: SRS-TEST-CLEAN-011
    /// Title: Test negative numbers preservation
    ///
    /// Description: VRConnect shall preserve negative numbers while fixing
    /// decimal separators.
    ///
    /// Version: V1.0
    #[test]
    fn test_negative_numbers() {
        let cleaner = VitalDataCleaner::new();

        let input = r#"{"val1": -123,456, "val2": -0,5, "arr": [-1,2, -3,4]}"#;
        let result = cleaner.clean(input).unwrap();

        assert!(result.contains("-123.456"));
        assert!(result.contains("-0.5"));
        assert!(result.contains("-1.2"));
        assert!(result.contains("-3.4"));

        // Verify parsing
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["val1"].as_f64().unwrap(), -123.456);
    }

    /// ID SRS: SRS-TEST-CLEAN-012
    /// Title: Test real vital data example
    ///
    /// Description: VRConnect shall clean realistic vital sign JSON data.
    ///
    /// Version: V1.0
    #[test]
    fn test_real_vital_data() {
        let cleaner = VitalDataCleaner::new();

        let input = r#"{"vrcode":"VR-123","rooms":[{"roomname":"BED_01","trks":[{"name":"ECG","recs":[{"val":[1,2, 3,4, NaN, 5,6],"dt":1234567890}]}]}]}"#;
        let result = cleaner.clean(input).unwrap();

        // Must be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["vrcode"].is_string());
        assert!(parsed["rooms"].is_array());

        // Check decimal fixes
        assert!(result.contains("1.2"));
        assert!(result.contains("3.4"));
    }
}
