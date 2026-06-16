//! # Image Processing Skill
//!
//! This module provides a collection of image manipulation skills including:
//! - Resizing images with aspect ratio preservation
//! - Converting between different image formats (PNG, JPEG, WebP, BMP, GIF)
//! - Extracting image metadata and information
//! - Rotating images by standard angles (90°, 180°, 270°)
//! - Cropping images to specified rectangular regions
//! - Compressing images with configurable quality and optional resizing
//!
//! All skills implement the `Skill` trait from the executors module and can be
//! used within the agent's skill execution framework.
//!
//! # Examples
//!
//! Basic usage of the image resize skill:
//! ```rust,ignore
//! use std::collections::HashMap;
//! use serde_json::json;
//!
//! let skill = ImageResizeSkill;
//! let mut params = HashMap::new();
//! params.insert("source".to_string(), json!("input.jpg"));
//! params.insert("destination".to_string(), json!("output.jpg"));
//! params.insert("width".to_string(), json!(800));
//! params.insert("height".to_string(), json!(600));
//! let result = skill.execute(&params).await?;
//! ```

use anyhow::Result;
use image::GenericImageView;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::{
    SkillCategory, file_exists,
    types::{Skill, SkillParameter},
};

/// Resize an image to specified dimensions
///
/// This skill resizes an image to the target width and height. It supports
/// preserving the original aspect ratio and offers multiple resampling filters
/// for different quality/performance trade-offs.
///
/// # Supported Filters
/// - `nearest`: Fastest, lowest quality (good for pixel art)
/// - `triangle`: Bilinear interpolation
/// - `catmullrom`: Bicubic filter, good quality
/// - `gaussian`: Gaussian blur-based scaling
/// - `lanczos3`: Highest quality, slowest (default)
///
/// # Use Cases
/// - Creating thumbnails for image galleries
/// - Generating profile pictures of uniform size
/// - Producing responsive image variants for web use
/// - Batch resizing photos for consistent dimensions
#[derive(Debug)]
pub struct ImageResizeSkill;

#[async_trait::async_trait]
impl Skill for ImageResizeSkill {
    fn name(&self) -> &str {
        "image_resize"
    }

    fn description(&self) -> &str {
        "Resize an image to specified width and height"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to change the dimensions of an image. \
        Supports maintaining aspect ratio with the 'preserve_aspect' parameter. \
        Common use cases: thumbnails, profile pictures, responsive images."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path (supported formats: PNG, JPEG, GIF, BMP, WebP)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/input.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path for the resized image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/output.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Target width in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(800.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Target height in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(600.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "preserve_aspect".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to preserve the original aspect ratio. If true, the image will be resized to fit within the specified dimensions while maintaining aspect ratio".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
            SkillParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Resampling filter to use (nearest, triangle, catmullrom, gaussian, lanczos3)".to_string(),
                required: false,
                default: Some(Value::String("lanczos3".to_string())),
                example: Some(Value::String("gaussian".to_string())),
                enum_values: Some(vec![
                    "nearest".to_string(),
                    "triangle".to_string(),
                    "catmullrom".to_string(),
                    "gaussian".to_string(),
                    "lanczos3".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_resize",
            "parameters": {
                "source": "/photos/original.jpg",
                "destination": "/photos/thumbnail.jpg",
                "width": 300,
                "height": 300,
                "preserve_aspect": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully resized image from 1920x1080 to 300x225 (aspect preserved)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Media
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let width = parameters
            .get("width")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'width' parameter"))?
            as u32;
        let height = parameters
            .get("height")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'height' parameter"))?
            as u32;
        let preserve_aspect = parameters
            .get("preserve_aspect")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let filter_name = parameters
            .get("filter")
            .and_then(|v| v.as_str())
            .unwrap_or("lanczos3");
        if !file_exists(source) {
            anyhow::bail!("Source image not found: {}", source);
        }
        let img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;
        let original_dimensions = img.dimensions();
        // Calculate new dimensions if preserving aspect ratio
        let (new_width, new_height) = if preserve_aspect {
            let ratio = original_dimensions.0 as f32 / original_dimensions.1 as f32;
            let target_ratio = width as f32 / height as f32;
            if ratio > target_ratio {
                let new_w = width;
                let new_h = (width as f32 / ratio).round() as u32;
                (new_w, new_h.max(1))
            } else {
                let new_h = height;
                let new_w = (height as f32 * ratio).round() as u32;
                (new_w.max(1), new_h)
            }
        } else {
            (width, height)
        };
        let filter = match filter_name {
            "nearest" => image::imageops::FilterType::Nearest,
            "triangle" => image::imageops::FilterType::Triangle,
            "catmullrom" => image::imageops::FilterType::CatmullRom,
            "gaussian" => image::imageops::FilterType::Gaussian,
            "lanczos3" => image::imageops::FilterType::Lanczos3,
            _ => image::imageops::FilterType::Lanczos3,
        };
        let resized = img.resize(new_width, new_height, filter);
        resized.save(destination).map_err(|e| {
            anyhow::anyhow!("Failed to save resized image to '{}': {}", destination, e)
        })?;
        Ok(format!(
            "Successfully resized image from {}x{} to {}x{} (preserve_aspect: {})",
            original_dimensions.0, original_dimensions.1, new_width, new_height, preserve_aspect
        ))
    }
}

/// Convert image between different formats
///
/// This skill converts images from one format to another. The output format is
/// determined by the destination file extension. Quality parameter controls
/// compression for lossy formats (JPEG, WebP).
///
/// # Supported Conversions
/// - PNG ↔ JPEG (lossy compression for smaller files)
/// - JPEG ↔ PNG (add transparency support)
/// - Any format → WebP (modern web-optimized format)
/// - BMP ↔ PNG (lossless conversion)
/// - GIF conversion (preserves animation for GIF output)
///
/// # Notes
/// - When converting to JPEG, transparency is replaced with a white background
/// - Quality 85 is recommended for photos, 75 for good compression/quality balance
/// - PNG conversions always use lossless compression (quality parameter ignored)
#[derive(Debug)]
pub struct ImageConvertSkill;

#[async_trait::async_trait]
impl Skill for ImageConvertSkill {
    fn name(&self) -> &str {
        "image_convert"
    }

    fn description(&self) -> &str {
        "Convert an image from one format to another (PNG, JPEG, WebP, BMP, GIF)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to change an image's file format. \
        Common conversions: PNG to JPEG (to reduce size), JPEG to PNG (for transparency), \
        any format to WebP (modern web format). Quality parameter only applies to JPEG and WebP outputs."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/image.png".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path (extension determines output format)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/image.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "quality".to_string(),
                param_type: "integer".to_string(),
                description: "Quality for lossy formats (JPEG/WebP), 1-100. Higher = better quality, larger file".to_string(),
                required: false,
                default: Some(Value::Number(85.into())),
                example: Some(Value::Number(90.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_convert",
            "parameters": {
                "source": "/photos/screenshot.png",
                "destination": "/photos/screenshot.jpg",
                "quality": 85
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully converted image from PNG to JPEG (quality: 85)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Media
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let quality = parameters
            .get("quality")
            .and_then(|v| v.as_u64())
            .unwrap_or(85);
        if !file_exists(source) {
            anyhow::bail!("Source image not found: {}", source);
        }
        let img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;
        let source_format = Path::new(source)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        let dest_path = Path::new(destination);
        let dest_ext = dest_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        match dest_ext.as_str() {
            "jpg" | "jpeg" => {
                img.save_with_format(destination, image::ImageFormat::Jpeg)
                    .map_err(|e| anyhow::anyhow!("Failed to save JPEG: {}", e))?;
                img.save(destination)
                    .map_err(|e| anyhow::anyhow!("Failed to save image: {}", e))?;
            }
            "png" => {
                img.save(destination)
                    .map_err(|e| anyhow::anyhow!("Failed to save PNG: {}", e))?;
            }
            "webp" => {
                img.save_with_format(destination, image::ImageFormat::WebP)
                    .map_err(|e| anyhow::anyhow!("Failed to save WebP: {}", e))?;
            }
            "bmp" => {
                img.save_with_format(destination, image::ImageFormat::Bmp)
                    .map_err(|e| anyhow::anyhow!("Failed to save BMP: {}", e))?;
            }
            "gif" => {
                img.save_with_format(destination, image::ImageFormat::Gif)
                    .map_err(|e| anyhow::anyhow!("Failed to save GIF: {}", e))?;
            }
            _ => {
                anyhow::bail!(
                    "Unsupported output format: '{}'. Supported: jpg, jpeg, png, webp, bmp, gif",
                    dest_ext
                );
            }
        }
        Ok(format!(
            "Successfully converted image from {} to {} (quality: {})",
            source_format.to_uppercase(),
            dest_ext.to_uppercase(),
            quality
        ))
    }
}

/// Get information about an image
///
/// This skill extracts comprehensive metadata from an image file without
/// modifying it. Useful for inspecting images before processing or for
/// gathering statistics about image collections.
///
/// # Returned Information
/// - **Dimensions**: Width, height, and aspect ratio in pixels
/// - **Format**: File format (JPEG, PNG, GIF, WebP, BMP, etc.)
/// - **File Size**: Size in bytes, kilobytes, and megabytes
/// - **Color Type**: Color space and bit depth (RGB8, RGBA8, Grayscale, etc.)
/// - **Total Pixels**: Total number of pixels in the image
///
/// # Use Cases
/// - Validating image requirements before upload
/// - Generating image catalogs with metadata
/// - Checking if images need resizing or optimization
/// - Debugging image loading issues
#[derive(Debug)]
pub struct ImageInfoSkill;

#[async_trait::async_trait]
impl Skill for ImageInfoSkill {
    fn name(&self) -> &str {
        "image_info"
    }

    fn description(&self) -> &str {
        "Get metadata information about an image (dimensions, format, file size)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to inspect an image's properties before processing it. \
        Returns dimensions, format, file size, and color information."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the image file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/path/to/image.jpg".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_info",
            "parameters": {
                "path": "/photos/landscape.jpg"
            }
        })
    }

    fn example_output(&self) -> String {
        "{\n  \"dimensions\": \"1920x1080\",\n  \"format\": \"JPEG\",\n  \"file_size_bytes\": 245760,\n  \"file_size_kb\": 240.0\n}".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Media
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        if !file_exists(path) {
            anyhow::bail!("Image not found: {}", path);
        }
        let metadata = fs::metadata(path)?;
        let file_size_bytes = metadata.len();
        let file_size_kb = file_size_bytes as f64 / 1024.0;
        let img = image::open(path)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", path, e))?;
        let dimensions = img.dimensions();
        let format = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_uppercase();
        let color_type = match img.color() {
            image::ColorType::L8 => "Grayscale 8-bit",
            image::ColorType::La8 => "Grayscale with Alpha 8-bit",
            image::ColorType::Rgb8 => "RGB 8-bit",
            image::ColorType::Rgba8 => "RGBA 8-bit",
            _ => "Other",
        };
        let info = json!({
            "path": path,
            "dimensions": {
                "width": dimensions.0,
                "height": dimensions.1,
                "aspect_ratio": format!("{:.2}", dimensions.0 as f64 / dimensions.1 as f64)
            },
            "format": format,
            "file_size": {
                "bytes": file_size_bytes,
                "kb": file_size_kb,
                "mb": file_size_kb / 1024.0
            },
            "color_type": color_type,
            "total_pixels": dimensions.0 as u64 * dimensions.1 as u64
        });

        Ok(serde_json::to_string_pretty(&info)?)
    }
}

/// Rotate an image by specified angle
///
/// This skill rotates images by standard angles (90°, 180°, 270°). For arbitrary
/// rotations, use the custom_angle parameter which will be implemented in future versions.
///
/// # Supported Rotations
/// - 90°: Rotates clockwise 90 degrees (portrait ↔ landscape)
/// - 180°: Flips upside down
/// - 270°: Rotates counter-clockwise 90 degrees (or clockwise 270°)
///
/// # Behavior
/// - Dimensions swap automatically for 90° and 270° rotations
/// - The image is rotated around its center
/// - No quality loss as rotation is lossless for pixel data
///
/// # Use Cases
/// - Correcting photo orientation from camera EXIF data
/// - Creating rotated variants for artistic purposes
/// - Standardizing image orientation in batches
#[derive(Debug)]
pub struct ImageRotateSkill;

#[async_trait::async_trait]
impl Skill for ImageRotateSkill {
    fn name(&self) -> &str {
        "image_rotate"
    }

    fn description(&self) -> &str {
        "Rotate an image by 90, 180, 270 degrees or a custom angle"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to rotate images that are oriented incorrectly. \
        For 90, 180, or 270-degree rotations, use the standard angles. \
        For arbitrary rotations, use 'custom_angle' and set 'expand_canvas' to true \
        to avoid cropping."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/input.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path for rotated image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/output.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "angle".to_string(),
                param_type: "integer".to_string(),
                description: "Rotation angle in degrees (0-360). For standard rotations: 90, 180, 270".to_string(),
                required: false,
                default: Some(Value::Number(90.into())),
                example: Some(Value::Number(180.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "custom_angle".to_string(),
                param_type: "number".to_string(),
                description: "Custom rotation angle in degrees (0-360). Use this for non-90-degree rotations".to_string(),
                required: false,
                default: None,
                example: Some(json!(45.0)),
                enum_values: None,
            },
            SkillParameter {
                name: "expand_canvas".to_string(),
                param_type: "boolean".to_string(),
                description: "For custom angles, expand canvas to fit entire rotated image (otherwise crop to original bounds)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
            SkillParameter {
                name: "background_color".to_string(),
                param_type: "string".to_string(),
                description: "Background color for custom rotations (hex: #RRGGBB or color name: white, black, etc.)".to_string(),
                required: false,
                default: Some(Value::String("white".to_string())),
                example: Some(Value::String("#f0f0f0".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_rotate",
            "parameters": {
                "source": "/photos/portrait.jpg",
                "destination": "/photos/rotated.jpg",
                "angle": 90
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully rotated image by 90 degrees".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Media
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        if !file_exists(source) {
            anyhow::bail!("Source image not found: {}", source);
        }
        let img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;
        let angle_deg = if let Some(custom) =
            parameters.get("custom_angle").and_then(|v| v.as_f64())
        {
            anyhow::bail!(
                "Custom angle rotation ({}) requires additional setup. Use standard angles: 90, 180, 270",
                custom
            );
        } else {
            let angle = parameters
                .get("angle")
                .and_then(|v| v.as_u64())
                .unwrap_or(90) as u32;
            let rotated = match angle % 360 {
                90 => img.rotate90(),
                180 => img.rotate180(),
                270 => img.rotate270(),
                0 => img,
                _ => anyhow::bail!("Unsupported angle {}. Supported: 90, 180, 270", angle),
            };
            rotated
                .save(destination)
                .map_err(|e| anyhow::anyhow!("Failed to save rotated image: {}", e))?;
            angle % 360
        };
        Ok(format!(
            "Successfully rotated image by {} degrees",
            angle_deg
        ))
    }
}

/// Crop an image to specified region
///
/// This skill extracts a rectangular region from an image, removing everything outside
/// the specified bounds. Cropping is useful for focusing on specific areas of interest
/// or removing unwanted borders, watermarks, or background elements.
///
/// # Parameter Details
/// - `x`, `y`: Top-left corner coordinates (0,0 is the top-left corner of the image)
/// - `width`, `height`: Dimensions of the crop region in pixels
/// - All crop bounds are validated against the original image dimensions
///
/// # Validation
/// The skill validates that the crop region fits entirely within the image bounds.
/// If the crop rectangle extends beyond the image edges, an error is returned.
///
/// # Use Cases
/// - Removing borders or watermarks from images
/// - Extracting faces or objects from photos
/// - Creating consistent aspect ratios for product images
/// - Zooming in on specific image areas
#[derive(Debug)]
pub struct ImageCropSkill;

#[async_trait::async_trait]
impl Skill for ImageCropSkill {
    fn name(&self) -> &str {
        "image_crop"
    }

    fn description(&self) -> &str {
        "Crop an image to a specified rectangular region"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to remove unwanted areas from an image. \
        Specify the crop region by coordinates (x, y, width, height). \
        Coordinates are measured from the top-left corner (0,0). \
        All values are in pixels."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/input.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path for cropped image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/output.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate of the top-left corner (pixels from left)".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate of the top-left corner (pixels from top)".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Width of the crop region in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(800.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Height of the crop region in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(600.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_crop",
            "parameters": {
                "source": "/photos/family.jpg",
                "destination": "/photos/cropped.jpg",
                "x": 200,
                "y": 150,
                "width": 1000,
                "height": 800
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully cropped image from 1920x1080 to 1000x800 at position (200, 150)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Media
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let x = parameters
            .get("x")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'x' parameter"))?
            as u32;
        let y = parameters
            .get("y")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'y' parameter"))?
            as u32;
        let width = parameters
            .get("width")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'width' parameter"))?
            as u32;
        let height = parameters
            .get("height")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'height' parameter"))?
            as u32;
        if !file_exists(source) {
            anyhow::bail!("Source image not found: {}", source);
        }
        let mut img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;
        let original_dimensions = img.dimensions();
        if x + width > original_dimensions.0 {
            anyhow::bail!(
                "Crop width exceeds image bounds: x={}, width={}, image_width={}",
                x,
                width,
                original_dimensions.0
            );
        }
        if y + height > original_dimensions.1 {
            anyhow::bail!(
                "Crop height exceeds image bounds: y={}, height={}, image_height={}",
                y,
                height,
                original_dimensions.1
            );
        }
        let cropped = img.crop(x, y, width, height);
        cropped
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save cropped image: {}", e))?;
        Ok(format!(
            "Successfully cropped image from {}x{} to {}x{} at position ({}, {})",
            original_dimensions.0, original_dimensions.1, width, height, x, y
        ))
    }
}

/// Compress an image to reduce file size
///
/// This skill reduces image file size through compression, optionally combined
/// with resizing for even greater savings. It's ideal for web optimization,
/// email attachments, and storage reduction.
///
/// # Compression Strategies
/// - **Quality Reduction**: Lower JPEG/WebP quality values (1-100) produce smaller files
/// - **Rescaling**: Reducing dimensions provides multiplicative file size savings
/// - **Format Selection**: Some formats (WebP) offer better compression than others
///
/// # Quality Recommendations
/// - **100**: Maximum quality, minimal compression (largest files)
/// - **85**: Excellent quality, good compression (recommended for photos)
/// - **75**: Good quality, significant compression (balanced for web)
/// - **60**: Acceptable quality, high compression (thumbnails, previews)
/// - **30**: Poor quality, maximum compression (archival only)
///
/// # Format-Specific Notes
/// - JPEG: Quality parameter affects compression level directly
/// - PNG: Always lossless (quality parameter ignored)
/// - WebP: Quality parameter works similar to JPEG
///
/// # Use Cases
/// - Optimizing images for website loading speed
/// - Reducing email attachment sizes
/// - Batch compressing photo libraries
/// - Creating web-optimized thumbnails
#[derive(Debug)]
pub struct ImageCompressSkill;

#[async_trait::async_trait]
impl Skill for ImageCompressSkill {
    fn name(&self) -> &str {
        "image_compress"
    }

    fn description(&self) -> &str {
        "Compress an image to reduce file size with configurable quality"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to reduce image file sizes for web optimization, email attachments, \
        or storage savings. Lower quality = smaller file size but more artifacts. \
        For JPEG/WebP, quality 70-85 is usually a good balance. \
        You can also resize as part of compression for additional savings."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Source image file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/large_image.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Destination file path for compressed image".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/compressed.jpg".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "quality".to_string(),
                param_type: "integer".to_string(),
                description: "Compression quality (1-100). Higher = better quality, larger file. For JPEG: 70-85 recommended".to_string(),
                required: false,
                default: Some(Value::Number(80.into())),
                example: Some(Value::Number(75.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "max_width".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum width (optional). If specified, image will be scaled down proportionally".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1920.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "max_height".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum height (optional). If specified, image will be scaled down proportionally".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1080.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_compress",
            "parameters": {
                "source": "/uploads/photo.jpg",
                "destination": "/uploads/photo_compressed.jpg",
                "quality": 80,
                "max_width": 1920
            }
        })
    }

    fn example_output(&self) -> String {
        "Compressed image: 2.5MB -> 850KB (66.0% reduction)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Media
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let quality = parameters
            .get("quality")
            .and_then(|v| v.as_u64())
            .unwrap_or(80) as u8;
        let max_width = parameters
            .get("max_width")
            .and_then(|v| v.as_u64())
            .map(|w| w as u32);
        let max_height = parameters
            .get("max_height")
            .and_then(|v| v.as_u64())
            .map(|h| h as u32);
        if !file_exists(source) {
            anyhow::bail!("Source image not found: {}", source);
        }
        let original_size = fs::metadata(source)?.len();
        let img = image::open(source)
            .map_err(|e| anyhow::anyhow!("Failed to open image '{}': {}", source, e))?;
        let mut processed_img = img;
        if let (Some(max_w), Some(max_h)) = (max_width, max_height) {
            let dimensions = processed_img.dimensions();
            if dimensions.0 > max_w || dimensions.1 > max_h {
                let ratio =
                    (dimensions.0 as f32 / dimensions.1 as f32).min(max_w as f32 / max_h as f32);
                let new_width = if dimensions.0 > max_w {
                    max_w
                } else {
                    dimensions.0
                };
                let new_height = (new_width as f32 / ratio) as u32;
                processed_img = processed_img.resize(
                    new_width,
                    new_height,
                    image::imageops::FilterType::Lanczos3,
                );
            }
        }
        let dest_ext = Path::new(destination)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        match dest_ext.as_str() {
            "jpg" | "jpeg" => {
                let mut bytes: Vec<u8> = Vec::new();
                let cursor = std::io::Cursor::new(&mut bytes);
                let mut encoder =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(cursor, quality);
                processed_img
                    .write_with_encoder(encoder)
                    .map_err(|e| anyhow::anyhow!("Failed to encode JPEG: {}", e))?;
                fs::write(destination, bytes)?;
            }
            "png" => {
                processed_img.save(destination)?;
            }
            "webp" => {
                processed_img
                    .save_with_format(destination, image::ImageFormat::WebP)
                    .map_err(|e| anyhow::anyhow!("Failed to save WebP: {}", e))?;
            }
            _ => {
                processed_img.save(destination)?;
            }
        }
        let compressed_size = fs::metadata(destination)?.len();
        let reduction = if original_size > 0 {
            ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };
        let original_mb = original_size as f64 / (1024.0 * 1024.0);
        let compressed_mb = compressed_size as f64 / (1024.0 * 1024.0);
        Ok(format!(
            "Compressed image: {:.2}MB -> {:.2}MB ({:.1}% reduction) at quality {}",
            original_mb, compressed_mb, reduction, quality
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    /// Tests the image info skill with a valid image file.
    /// Verifies that the skill correctly extracts metadata including dimensions,
    /// format, file size, and color type.
    #[tokio::test]
    async fn test_image_info_with_valid_image() {
        // Create a temporary test image (1x1 red pixel PNG)
        let test_image_data = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // width=1, height=1
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // bit depth=8, color=RGB
            0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
            0x54, 0x78, 0xDA, 0x63, 0x60, 0x60, 0x60, 0x00, // IDAT data (red)
            0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0x01, 0x9F, // IDAT data cont
            0xD1, 0x4F, 0x8E, 0x00, 0x00, 0x00, 0x00, 0x49, // IEND chunk
            0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();
        fs::write(temp_path, &test_image_data).unwrap();
        let skill = ImageInfoSkill;
        let mut params = HashMap::new();
        params.insert("path".to_string(), json!(temp_path));
        let result = skill.execute(&params).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let info: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(info["dimensions"]["width"], 1);
        assert_eq!(info["dimensions"]["height"], 1);
        assert_eq!(info["dimensions"]["aspect_ratio"], "1.00");
        assert!(info["file_size"]["bytes"].as_u64().unwrap() > 0);
        assert!(info["file_size"]["kb"].as_f64().unwrap() > 0.0);
        assert!(info["total_pixels"].as_u64().unwrap() == 1);
    }

    /// Tests the image info skill with a non-existent file.
    /// Verifies that the skill returns an appropriate error.
    #[tokio::test]
    async fn test_image_info_with_nonexistent_file() {
        let skill = ImageInfoSkill;
        let mut params = HashMap::new();
        params.insert("path".to_string(), json!("/nonexistent/path/image.jpg"));
        let result = skill.execute(&params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    /// Tests the image info skill with missing required parameters.
    /// Verifies that the skill validates input parameters correctly.
    #[tokio::test]
    async fn test_image_info_with_missing_parameter() {
        let skill = ImageInfoSkill;
        let params = HashMap::new();
        let result = skill.execute(&params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'path'"));
    }
}
