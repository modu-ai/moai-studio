//! EXIF metadata extraction module (MS-2).
//!
//! SPEC-V3-016 MS-2: EXIF metadata panel (REQ-IV-040~045).
//! Uses `kamadak-exif` crate to extract camera and photo metadata from image files.
//! Graceful degradation: missing fields are returned as `None`.

use std::path::Path;

// @MX:ANCHOR: [AUTO] exif-data-struct
// @MX:REASON: [AUTO] REQ-IV-040. ExifData is the primary EXIF metadata container.
//   fan_in >= 3: image.rs load_image, render_exif_panel, unit tests.

/// EXIF metadata extracted from image file (REQ-IV-040).
///
/// All fields are optional to handle images without EXIF data gracefully.
#[derive(Debug, Clone, Default)]
pub struct ExifData {
    /// Camera manufacturer (e.g., "Canon", "Nikon", "Apple").
    pub camera_make: Option<String>,
    /// Camera model (e.g., "EOS R5", "iPhone 15 Pro").
    pub camera_model: Option<String>,
    /// Original capture date/time (e.g., "2024:01:15 14:30:22").
    pub datetime_original: Option<String>,
    /// Exposure time as string (e.g., "1/120").
    pub exposure_time: Option<String>,
    /// F-number as string (e.g., "f/2.8").
    pub f_number: Option<String>,
    /// ISO speed rating.
    pub iso: Option<u32>,
    /// Focal length in mm as string (e.g., "50.0").
    pub focal_length: Option<String>,
    /// Image width from EXIF (may differ from pixel width).
    pub image_width: Option<u32>,
    /// Image height from EXIF (may differ from pixel height).
    pub image_height: Option<u32>,
}

/// Extract EXIF metadata from image file at given path (REQ-IV-041).
///
/// Returns `None` if:
/// - File does not exist
/// - File cannot be read
/// - File has no EXIF data
/// - EXIF parsing fails for any reason
///
/// This function never panics. All errors are gracefully handled.
pub fn extract_exif(path: &Path) -> Option<ExifData> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(path).ok()?;
    let mut bufreader = BufReader::new(file);

    let exif_reader = exif::Reader::new();
    let exif_data = exif_reader.read_from_container(&mut bufreader).ok()?;

    Some(ExifData {
        camera_make: get_string_field(&exif_data, exif::Tag::Make),
        camera_model: get_string_field(&exif_data, exif::Tag::Model),
        datetime_original: get_string_field(&exif_data, exif::Tag::DateTimeOriginal),
        exposure_time: get_display_field(&exif_data, exif::Tag::ExposureTime),
        f_number: get_display_field(&exif_data, exif::Tag::FNumber),
        iso: get_uint_field(&exif_data, exif::Tag::PhotographicSensitivity),
        focal_length: get_display_field(&exif_data, exif::Tag::FocalLength),
        image_width: get_uint_field(&exif_data, exif::Tag::ImageWidth),
        image_height: get_uint_field(&exif_data, exif::Tag::ImageLength),
    })
}

/// Extract string value from an ASCII EXIF field.
fn get_string_field(exif_data: &exif::Exif, tag: exif::Tag) -> Option<String> {
    let field = exif_data.get_field(tag, exif::In::PRIMARY)?;
    // display_value().to_string() returns the value without unit for ASCII fields
    let val = field.display_value().to_string();
    // Strip surrounding quotes that display_value adds for ASCII strings
    let cleaned = val.trim_matches('"').to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

/// Extract display value from an EXIF field (with unit formatting).
fn get_display_field(exif_data: &exif::Exif, tag: exif::Tag) -> Option<String> {
    let field = exif_data.get_field(tag, exif::In::PRIMARY)?;
    let val = field.display_value().with_unit(exif_data).to_string();
    if val.is_empty() { None } else { Some(val) }
}

/// Extract uint value from an EXIF field.
fn get_uint_field(exif_data: &exif::Exif, tag: exif::Tag) -> Option<u32> {
    let field = exif_data.get_field(tag, exif::In::PRIMARY)?;
    field.value.get_uint(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── REQ-IV-040~045: EXIF metadata extraction tests ──

    #[test]
    fn extract_exif_nonexistent_file_returns_none() {
        let path = PathBuf::from("/nonexistent/image_that_does_not_exist.png");
        let result = extract_exif(&path);
        assert!(
            result.is_none(),
            "extract_exif should return None for nonexistent file"
        );
    }

    #[test]
    fn extract_exif_invalid_file_returns_none() {
        // Create a temp file with non-image content
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_path = temp_dir.path().join("not_an_image.jpg");
        std::fs::write(&invalid_path, b"this is not a real JPEG file content").unwrap();

        let result = extract_exif(&invalid_path);
        assert!(
            result.is_none(),
            "extract_exif should return None for invalid/non-EXIF file"
        );

        temp_dir.close().unwrap();
    }

    #[test]
    fn extract_exif_default_struct_has_none_fields() {
        let data = ExifData::default();
        assert!(data.camera_make.is_none());
        assert!(data.camera_model.is_none());
        assert!(data.datetime_original.is_none());
        assert!(data.exposure_time.is_none());
        assert!(data.f_number.is_none());
        assert!(data.iso.is_none());
        assert!(data.focal_length.is_none());
        assert!(data.image_width.is_none());
        assert!(data.image_height.is_none());
    }
}
