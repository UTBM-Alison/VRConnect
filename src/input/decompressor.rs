use crate::error::{Result, VitalError};
use flate2::read::ZlibDecoder;
use std::io::Read;
use tracing::{debug, trace};

pub struct VitalDataDecompressor;

impl VitalDataDecompressor {
    pub fn new() -> Self {
        Self
    }

    /// Decompress data if it's compressed with zlib or has Socket.IO binary indicator
    /// Returns the decompressed data or the original data if not compressed
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(data.to_vec());
        }

        // Check for Socket.IO v4 binary indicator (0x04) followed by zlib header (0x78)
        if data.len() >= 2 && data[0] == 0x04 && data[1] == 0x78 {
            debug!("Detected Socket.IO v4 binary frame with zlib compression");
            return self.decompress_zlib(&data[1..]);
        }

        // Check for direct zlib header (0x78 0x9C or 0x78 0xDA or 0x78 0x01)
        if data[0] == 0x78 && data.len() >= 2 {
            let second_byte = data[1];
            if second_byte == 0x9C || second_byte == 0xDA || second_byte == 0x01 {
                debug!("Detected zlib compression");
                return self.decompress_zlib(data);
            }
        }

        // No compression detected, return as-is
        trace!("No compression detected, returning original data");
        Ok(data.to_vec())
    }

    fn decompress_zlib(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| VitalError::Decompression(format!("Zlib decompression failed: {}", e)))?;

        debug!("Decompressed {} bytes to {} bytes", data.len(), decompressed.len());
        Ok(decompressed)
    }
}

impl Default for VitalDataDecompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for VitalDataDecompressor {
    fn clone(&self) -> Self {
        Self::new()
    }
}
