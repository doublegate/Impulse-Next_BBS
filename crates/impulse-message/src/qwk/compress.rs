//! QWK packet compression and decompression
//!
//! Handles ZIP archive operations for QWK packets.

use super::error::{QwkError, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Compress files into a QWK ZIP packet
pub struct QwkCompressor {
    zip: ZipWriter<File>,
}

impl QwkCompressor {
    /// Create a new QWK packet compressor
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path)?;
        let zip = ZipWriter::new(file);
        Ok(Self { zip })
    }

    /// Add a file to the QWK packet
    pub fn add_file(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let options: zip::write::FileOptions<()> = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        self.zip.start_file(name, options)?;
        self.zip.write_all(data)?;
        Ok(())
    }

    /// Finish writing the QWK packet
    pub fn finish(self) -> Result<()> {
        self.zip.finish()?;
        Ok(())
    }
}

/// Decompress a QWK ZIP packet
pub struct QwkDecompressor {
    zip: ZipArchive<File>,
}

impl QwkDecompressor {
    /// Open a QWK packet for decompression
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let zip = ZipArchive::new(file)?;
        Ok(Self { zip })
    }

    /// Get the number of files in the packet
    pub fn file_count(&self) -> usize {
        self.zip.len()
    }

    /// Get the list of file names in the packet
    pub fn file_names(&self) -> Vec<String> {
        (0..self.zip.len())
            .filter_map(|i| {
                self.zip
                    .name_for_index(i)
                    .and_then(|res| res.ok())
                    .map(|s| s.into_owned())
            })
            .collect()
    }

    /// Check if a specific file exists in the packet
    pub fn has_file(&mut self, name: &str) -> bool {
        self.zip.by_name(name).is_ok()
    }

    /// Extract a file from the packet
    pub fn extract_file(&mut self, name: &str) -> Result<Vec<u8>> {
        let mut file = self
            .zip
            .by_name(name)
            .map_err(|_| QwkError::MissingFile(name.to_string()))?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    /// Extract all files from the packet to a directory
    pub fn extract_all<P: AsRef<Path>>(&mut self, dest_dir: P) -> Result<()> {
        let dest_dir = dest_dir.as_ref();

        for i in 0..self.zip.len() {
            let mut file = self.zip.by_index(i)?;
            let name = file.name()?;
            let file_path = dest_dir.join(name.as_ref());

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut dest_file = File::create(file_path)?;
            std::io::copy(&mut file, &mut dest_file)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compress_decompress_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.qwk");

        // Compress
        let mut compressor = QwkCompressor::new(&zip_path).unwrap();
        compressor.add_file("TEST.TXT", b"Hello, World!").unwrap();
        compressor.finish().unwrap();

        // Decompress
        let mut decompressor = QwkDecompressor::open(&zip_path).unwrap();
        assert_eq!(decompressor.file_count(), 1);
        assert!(decompressor.has_file("TEST.TXT"));

        let data = decompressor.extract_file("TEST.TXT").unwrap();
        assert_eq!(data, b"Hello, World!");
    }

    #[test]
    fn test_compress_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.qwk");

        // Compress multiple files
        let mut compressor = QwkCompressor::new(&zip_path).unwrap();
        compressor.add_file("CONTROL.DAT", b"Control data").unwrap();
        compressor.add_file("DOOR.ID", b"Door ID").unwrap();
        compressor
            .add_file("MESSAGES.DAT", b"Message data")
            .unwrap();
        compressor.finish().unwrap();

        // Decompress and verify
        let decompressor = QwkDecompressor::open(&zip_path).unwrap();
        assert_eq!(decompressor.file_count(), 3);

        let names = decompressor.file_names();
        assert!(names.contains(&"CONTROL.DAT".to_string()));
        assert!(names.contains(&"DOOR.ID".to_string()));
        assert!(names.contains(&"MESSAGES.DAT".to_string()));
    }

    #[test]
    fn test_extract_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.qwk");

        // Create packet with one file
        let mut compressor = QwkCompressor::new(&zip_path).unwrap();
        compressor.add_file("TEST.TXT", b"data").unwrap();
        compressor.finish().unwrap();

        // Try to extract non-existent file
        let mut decompressor = QwkDecompressor::open(&zip_path).unwrap();
        let result = decompressor.extract_file("MISSING.TXT");
        assert!(matches!(result, Err(QwkError::MissingFile(_))));
    }

    #[test]
    fn test_extract_all() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.qwk");
        let extract_dir = temp_dir.path().join("extracted");

        // Create packet
        let mut compressor = QwkCompressor::new(&zip_path).unwrap();
        compressor.add_file("FILE1.TXT", b"content1").unwrap();
        compressor.add_file("FILE2.TXT", b"content2").unwrap();
        compressor.finish().unwrap();

        // Extract all
        let mut decompressor = QwkDecompressor::open(&zip_path).unwrap();
        decompressor.extract_all(&extract_dir).unwrap();

        // Verify files exist
        assert!(extract_dir.join("FILE1.TXT").exists());
        assert!(extract_dir.join("FILE2.TXT").exists());

        // Verify contents
        let content1 = std::fs::read(extract_dir.join("FILE1.TXT")).unwrap();
        assert_eq!(content1, b"content1");
    }

    #[test]
    fn test_has_file() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.qwk");

        let mut compressor = QwkCompressor::new(&zip_path).unwrap();
        compressor.add_file("EXISTS.TXT", b"data").unwrap();
        compressor.finish().unwrap();

        let mut decompressor = QwkDecompressor::open(&zip_path).unwrap();
        assert!(decompressor.has_file("EXISTS.TXT"));
        assert!(!decompressor.has_file("MISSING.TXT"));
    }
}
