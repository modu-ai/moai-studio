//! Image decoding module (MS-1).
//!
//! Provides `ImageData` struct and `decode_image` function using the `image` crate.
//! Decodes PNG, JPEG, GIF, WebP, BMP, ICO formats into RGBA pixel buffer.

use std::path::{Path, PathBuf};

/// Decoded image data stored in ImageViewer (REQ-IV-001).
///
/// Contains RGBA pixel buffer, dimensions, and source path.
/// The pixels are in RGBA8 format (4 bytes per pixel).
pub struct ImageData {
    /// RGBA pixel buffer (width * height * 4 bytes).
    pub pixels: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Source file path.
    pub path: PathBuf,
    /// File size in bytes (REQ-IV-040).
    pub file_size: u64,
}

impl ImageData {
    /// Create new ImageData from decoded pixel buffer.
    pub fn new(pixels: Vec<u8>, width: u32, height: u32, path: PathBuf, file_size: u64) -> Self {
        Self {
            pixels,
            width,
            height,
            path,
            file_size,
        }
    }

    /// Get total number of pixels.
    pub fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Get expected buffer size (width * height * 4).
    pub fn expected_buffer_size(&self) -> usize {
        (self.width as usize) * (self.height as usize) * 4
    }
}

/// Image decoding error types (REQ-IV-003).
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image decode error: {0}")]
    Decode(#[from] image::ImageError),

    #[error("File too large: {path:?} ({bytes} bytes, max 200MB)")]
    TooLarge { path: PathBuf, bytes: u64 },

    #[error("Unsupported image format: {0:?}")]
    UnsupportedFormat(PathBuf),
}

// @MX:ANCHOR: [AUTO] decode-image-function
// @MX:REASON: [AUTO] REQ-IV-002. decode_image is the primary image decoding entry point.
//   fan_in >= 3: lib.rs handle_open_file, unit tests (success/error cases),
//   future MS-2 EXIF integration.

/// Decode image file into ImageData (REQ-IV-002).
///
/// Supported formats: PNG, JPEG, GIF (first frame), WebP, BMP, ICO (REQ-IV-004).
///
/// # Errors
///
/// Returns `ImageError` if:
/// - File cannot be read (I/O error)
/// - Image format is unsupported or corrupt
/// - File exceeds 200MB limit
pub fn decode_image(path: &Path) -> Result<ImageData, ImageError> {
    use std::fs;

    // Check file size (200MB limit from viewer/mod.rs MAX_FILE_BYTES)
    const MAX_FILE_BYTES: u64 = 200 * 1024 * 1024;
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len();
    if file_size > MAX_FILE_BYTES {
        return Err(ImageError::TooLarge {
            path: path.to_path_buf(),
            bytes: file_size,
        });
    }

    // Decode image using image crate
    let img = image::open(path)?;
    let rgba = img.to_rgba8();

    Ok(ImageData::new(
        rgba.to_vec(),
        img.width(),
        img.height(),
        path.to_path_buf(),
        file_size,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── RED Phase Tests: These will fail until we implement decode_image ──

    #[test]
    fn image_data_creation() {
        let pixels = vec![0u8; 100 * 100 * 4];
        let data = ImageData::new(pixels.clone(), 100, 100, PathBuf::from("test.png"), 1024);
        assert_eq!(data.width, 100);
        assert_eq!(data.height, 100);
        assert_eq!(data.pixels.len(), 100 * 100 * 4);
        assert_eq!(data.pixel_count(), 10000);
        assert_eq!(data.expected_buffer_size(), 100 * 100 * 4);
    }

    #[test]
    fn decode_nonexistent_file_returns_io_error() {
        let result = decode_image(Path::new("/nonexistent/image.png"));
        assert!(matches!(result, Err(ImageError::Io(_))));
    }

    #[test]
    fn decode_invalid_image_returns_decode_error() {
        // Create a temporary file with invalid image data
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_path = temp_dir.path().join("invalid.png");
        std::fs::write(&invalid_path, b"not an image").unwrap();

        let result = decode_image(&invalid_path);
        assert!(matches!(result, Err(ImageError::Decode(_))));

        // Cleanup
        temp_dir.close().unwrap();
    }
}
