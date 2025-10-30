// /src/input/decompressor.rs
// Module: input.decompressor
// Purpose: Automatic zlib decompression for Socket.IO binary data

use crate::error::{Result, VitalError};
use flate2::read::ZlibDecoder;
use std::io::Read;

/// ID SRS: SRS-MOD-DECOMPRESSOR-001
/// Title: VitalDataDecompressor
///
/// Description: VRConnect shall detect and decompress zlib-compressed data
/// automatically, supporting Socket.IO v4 binary indicators.
///
/// Version: V1.0
#[derive(Clone)]
pub struct VitalDataDecompressor;

impl VitalDataDecompressor {
    /// ID SRS: SRS-FN-DECOMPRESSOR-001
    /// Title: new
    ///
    /// Description: VRConnect shall construct a new VitalDataDecompressor instance.
    ///
    /// Version: V1.0
    ///
    /// # Returns
    /// New decompressor instance
    pub fn new() -> Self {
        Self
    }

    /// ID SRS: SRS-FN-DECOMPRESSOR-002
    /// Title: decompress
    ///
    /// Description: VRConnect shall detect compression format (Socket.IO v4 binary
    /// or direct zlib) and decompress data, returning original data if uncompressed.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `data` - Raw data bytes
    ///
    /// # Returns
    /// Decompressed data or original if not compressed
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(data.to_vec());
        }

        // Check for Socket.IO v4 binary indicator (0x04) followed by zlib header (0x78)
        if data.len() >= 2 && data[0] == 0x04 && data[1] == 0x78 {
            log::debug!("Detected Socket.IO v4 binary frame with zlib compression");
            return self.decompress_zlib(&data[1..]);
        }

        // Check for direct zlib header (0x78 0x9C or 0x78 0xDA or 0x78 0x01)
        if data[0] == 0x78 && data.len() >= 2 {
            let second_byte = data[1];
            if second_byte == 0x9C || second_byte == 0xDA || second_byte == 0x01 {
                log::debug!("Detected zlib compression");
                return self.decompress_zlib(data);
            }
        }

        // No compression detected
        log::debug!("No compression detected, returning original data");
        Ok(data.to_vec())
    }

    /// ID SRS: SRS-FN-DECOMPRESSOR-003
    /// Title: decompress_zlib
    ///
    /// Description: VRConnect shall decompress zlib-compressed data using
    /// flate2 decoder, returning decompressed bytes.
    ///
    /// Version: V1.0
    ///
    /// # Arguments
    /// * `data` - Zlib-compressed data
    ///
    /// # Returns
    /// Decompressed data or error
    fn decompress_zlib(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| VitalError::Decompression(format!("Zlib decompression failed: {}", e)))?;

        log::debug!(
            "Decompressed {} bytes to {} bytes",
            data.len(),
            decompressed.len()
        );
        Ok(decompressed)
    }
}

impl Default for VitalDataDecompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_uncompressed() {
        // TODO: Implement uncompressed data test
        assert!(true);
    }

    #[test]
    fn test_decompress_socketio_v4() {
        // TODO: Implement Socket.IO v4 binary frame test
        assert!(true);
    }

    #[test]
    fn test_decompress_direct_zlib() {
        // TODO: Implement direct zlib compression test
        assert!(true);
    }

    #[test]
    fn test_decompress_empty() {
        // TODO: Implement empty data test
        assert!(true);
    }

    #[test]
    fn test_decompress_invalid() {
        // TODO: Implement invalid compression test
        assert!(true);
    }
}
