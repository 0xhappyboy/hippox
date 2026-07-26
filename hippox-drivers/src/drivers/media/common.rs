//! Shared utilities for media processing
//!
//! This module provides common utility functions used across media processing
//! drivers including format detection, file size calculation, and value clamping.
use std::path::Path;
use tracing::debug;
use crate::DriverResult;
/// Gets image format from file extension
///
/// # Arguments
/// * `path` - File path
///
/// # Returns
/// * `Option<image::ImageFormat>` - Image format if recognized
pub fn get_format_from_extension(path: &str) -> Option<image::ImageFormat> {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase())?;
    let format = match ext.as_str() {
        "jpg" | "jpeg" => Some(image::ImageFormat::Jpeg),
        "png" => Some(image::ImageFormat::Png),
        "webp" => Some(image::ImageFormat::WebP),
        "bmp" => Some(image::ImageFormat::Bmp),
        "gif" => Some(image::ImageFormat::Gif),
        "ico" => Some(image::ImageFormat::Ico),
        "tif" | "tiff" => Some(image::ImageFormat::Tiff),
        "avif" => Some(image::ImageFormat::Avif),
        _ => None,
    };
    debug!("Detected format for {}: {:?}", path, format);
    return format;
}
/// Gets file size in bytes
///
/// # Arguments
/// * `path` - File path
///
/// # Returns
/// * `DriverResult<u64>` - File size in bytes
pub fn get_file_size(path: &str) -> DriverResult<u64> {
    let size = std::fs::metadata(path).map_err(|e| crate::DriverError::execution(format!("Failed to get file size: {}", e)))?.len();
    return Ok(size);
}
/// Formats file size for display
///
/// # Arguments
/// * `bytes` - File size in bytes
///
/// # Returns
/// * `String` - Human-readable file size
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{} B", bytes);
    } else if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    } else if bytes < 1024 * 1024 * 1024 {
        return format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0));
    } else {
        return format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    }
}
/// Clamps a value between min and max
///
/// # Arguments
/// * `value` - Value to clamp
/// * `min` - Minimum allowed value
/// * `max` - Maximum allowed value
///
/// # Returns
/// * `T` - Clamped value
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        return min;
    } else if value > max {
        return max;
    } else {
        return value;
    }
}
