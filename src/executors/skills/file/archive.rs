//! # Archive Module
//!
//! This module provides skills for creating and extracting compressed archives
//! in various formats including ZIP, TAR (with gzip/bzip2 compression), and
//! standalone compression (gzip/bzip2).
//!
//! ## Available Skills
//!
//! - `ArchiveZipCreateSkill`: Create ZIP archives from files/directories
//! - `ArchiveZipExtractSkill`: Extract ZIP archives
//! - `ArchiveTarCreateSkill`: Create TAR archives (optionally compressed)
//! - `ArchiveTarExtractSkill`: Extract TAR archives (optionally compressed)
//! - `ArchiveCompressSkill`: Compress single files using gzip or bzip2
//!
//! ## Usage Examples
//!
//! ```rust,ignore
//! // Create a ZIP archive
//! let skill = ArchiveZipCreateSkill;
//! let params = HashMap::from([
//!     ("sources", json!(["/path/to/file1", "/path/to/dir"])),
//!     ("destination", json!("/path/to/archive.zip")),
//! ]);
//! let result = skill.execute(&params).await?;
//!
//! // Extract a TAR.GZ archive
//! let skill = ArchiveTarExtractSkill;
//! let params = HashMap::from([
//!     ("archive", json!("/path/to/archive.tar.gz")),
//!     ("destination", json!("/path/to/extract")),
//! ]);
//! let result = skill.execute(&params).await?;
//! ```

use anyhow::Result;
use bzip2::Compression as BzCompression;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use walkdir::WalkDir;
use zip::ZipWriter;
use zip::read::ZipArchive;
use zip::write::FileOptions;

use crate::executors::types::{Skill, SkillParameter};
use crate::executors::{file_exists, validate_path};

/// Skill for creating ZIP archives from files and directories.
///
/// This skill compresses multiple files and directories into a single ZIP archive.
/// It supports configurable compression levels and preserves directory structure
/// optionally.
///
/// # Parameters
///
/// * `sources` (required): Array of file or directory paths to include
/// * `destination` (required): Output path (must end with .zip)
/// * `compression_level` (optional): 0-9, default 6
/// * `preserve_paths` (optional): Keep directory structure, default true
///
/// # Example
///
/// ```json
/// {
///     "action": "archive_zip_create",
///     "parameters": {
///         "sources": ["/home/user/docs", "/home/user/readme.txt"],
///         "destination": "/home/user/backup.zip",
///         "compression_level": 9
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ArchiveZipCreateSkill;

#[async_trait::async_trait]
impl Skill for ArchiveZipCreateSkill {
    fn name(&self) -> &str {
        "archive_zip_create"
    }

    fn description(&self) -> &str {
        "Create a ZIP archive from files or directories"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compress multiple files or directories into a single ZIP archive. Provide source paths and the destination archive path."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "sources".to_string(),
                param_type: "array".to_string(),
                description: "Array of file or directory paths to include in the archive"
                    .to_string(),
                required: true,
                default: None,
                example: Some(json!(["/home/user/docs", "/home/user/notes.txt"])),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Path where to create the ZIP archive (should end with .zip)"
                    .to_string(),
                required: true,
                default: None,
                example: Some(json!("/home/user/archive.zip")),
                enum_values: None,
            },
            SkillParameter {
                name: "compression_level".to_string(),
                param_type: "integer".to_string(),
                description: "Compression level (0-9, where 0=none, 9=best compression)"
                    .to_string(),
                required: false,
                default: Some(Value::Number(6.into())),
                example: Some(Value::Number(9.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "preserve_paths".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to preserve directory structure in the archive".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "archive_zip_create",
            "parameters": {
                "sources": ["/home/user/documents", "/home/user/readme.txt"],
                "destination": "/home/user/backup.zip",
                "compression_level": 6,
                "preserve_paths": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully created ZIP archive at /home/user/backup.zip containing 15 files".to_string()
    }

    fn category(&self) -> &str {
        "archive"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let sources = parameters
            .get("sources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!("Missing or invalid 'sources' parameter (expected array)")
            })?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let compression_level = parameters
            .get("compression_level")
            .and_then(|v| v.as_u64())
            .unwrap_or(6)
            .min(9) as u16;
        let preserve_paths = parameters
            .get("preserve_paths")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let dest_path = validate_path(destination, None)?;
        if !destination.ends_with(".zip") {
            anyhow::bail!("Destination file must have .zip extension");
        }
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(&dest_path)?;
        let mut zip = ZipWriter::new(BufWriter::new(file));
        let options: FileOptions<()> = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(compression_level.into()));
        let mut file_count = 0;
        let mut total_size = 0;
        for source in sources {
            let source_path_str = source
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Source path must be a string"))?;
            let source_path = validate_path(source_path_str, None)?;
            if !file_exists(&source_path.to_string_lossy()) {
                anyhow::bail!("Source path not found: {}", source_path_str);
            }
            let base_path = if preserve_paths {
                source_path.parent().unwrap_or(&source_path).to_path_buf()
            } else {
                PathBuf::new()
            };
            if source_path.is_dir() {
                for entry in WalkDir::new(&source_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                {
                    let file_path = entry.path();
                    let relative_path: &Path = if preserve_paths {
                        file_path.strip_prefix(&base_path).unwrap_or(file_path)
                    } else {
                        let file_name = file_path.file_name().unwrap_or_default();
                        Path::new(file_name)
                    };
                    zip.start_file(relative_path.to_string_lossy(), options)?;
                    let mut file = File::open(file_path)?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    total_size += buffer.len();
                    zip.write_all(&buffer)?;
                    file_count += 1;
                }
            } else {
                let relative_path: &Path = if preserve_paths {
                    source_path.strip_prefix(&base_path).unwrap_or(&source_path)
                } else {
                    let file_name = source_path.file_name().unwrap_or_default();
                    Path::new(file_name)
                };
                zip.start_file(relative_path.to_string_lossy(), options)?;
                let mut file = File::open(&source_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                total_size += buffer.len();
                zip.write_all(&buffer)?;
                file_count += 1;
            }
        }
        zip.finish()?;
        Ok(format!(
            "Successfully created ZIP archive at {} containing {} file(s) (total size: {} bytes)",
            destination, file_count, total_size
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        if parameters
            .get("sources")
            .and_then(|v| v.as_array())
            .is_none()
        {
            anyhow::bail!("Missing or invalid 'sources' parameter (expected array)");
        }
        if parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .is_none()
        {
            anyhow::bail!("Missing 'destination' parameter");
        }
        Ok(())
    }
}

/// Skill for extracting ZIP archives to a destination directory.
///
/// This skill extracts all files from a ZIP archive, preserving directory structure.
/// It supports optional overwriting of existing files.
///
/// # Parameters
///
/// * `archive` (required): Path to the ZIP archive file
/// * `destination` (required): Directory where to extract the files
/// * `overwrite` (optional): Whether to overwrite existing files, default false
///
/// # Example
///
/// ```json
/// {
///     "action": "archive_zip_extract",
///     "parameters": {
///         "archive": "/home/user/backup.zip",
///         "destination": "/home/user/extracted",
///         "overwrite": true
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ArchiveZipExtractSkill;

#[async_trait::async_trait]
impl Skill for ArchiveZipExtractSkill {
    fn name(&self) -> &str {
        "archive_zip_extract"
    }

    fn description(&self) -> &str {
        "Extract a ZIP archive to a destination directory"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to extract files from a ZIP archive. Provide the archive path and the destination directory."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "archive".to_string(),
                param_type: "string".to_string(),
                description: "Path to the ZIP archive file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/archive.zip".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Directory where to extract the files".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/extracted".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "overwrite".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to overwrite existing files".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "archive_zip_extract",
            "parameters": {
                "archive": "/home/user/backup.zip",
                "destination": "/home/user/extracted",
                "overwrite": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully extracted 15 files from /home/user/backup.zip to /home/user/extracted"
            .to_string()
    }

    fn category(&self) -> &str {
        "archive"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let archive = parameters
            .get("archive")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'archive' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let overwrite = parameters
            .get("overwrite")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let archive_path = validate_path(archive, None)?;
        let dest_path = validate_path(destination, None)?;
        if !file_exists(&archive_path.to_string_lossy()) {
            anyhow::bail!("Archive file not found: {}", archive);
        }
        if !archive.ends_with(".zip") {
            anyhow::bail!("File must have .zip extension");
        }
        fs::create_dir_all(&dest_path)?;
        let file = File::open(&archive_path)?;
        let mut zip = ZipArchive::new(BufReader::new(file))?;
        let mut file_count = 0;
        let mut extracted_size = 0;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let entry_path = dest_path.join(entry.name());
            if entry.is_dir() {
                fs::create_dir_all(&entry_path)?;
                continue;
            }
            if entry_path.exists() && !overwrite {
                continue;
            }
            if let Some(parent) = entry_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&entry_path)?;
            let mut buffer = Vec::new();
            std::io::copy(&mut entry, &mut buffer)?;
            extracted_size += buffer.len();
            outfile.write_all(&buffer)?;
            file_count += 1;
        }
        Ok(format!(
            "Successfully extracted {} file(s) from {} to {} (total size: {} bytes)",
            file_count, archive, destination, extracted_size
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        if parameters.get("archive").and_then(|v| v.as_str()).is_none() {
            anyhow::bail!("Missing 'archive' parameter");
        }
        if parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .is_none()
        {
            anyhow::bail!("Missing 'destination' parameter");
        }
        Ok(())
    }
}

/// Skill for creating TAR archives (optionally with gzip/bzip2 compression).
///
/// This skill creates TAR archives that can optionally be compressed with gzip or bzip2.
/// It supports adding multiple files and directories while preserving directory structure.
///
/// # Parameters
///
/// * `sources` (required): Array of file or directory paths to include
/// * `destination` (required): Output path (.tar, .tar.gz, .tgz, .tar.bz2, .tbz2)
/// * `preserve_paths` (optional): Keep directory structure, default true
///
/// # Example
///
/// ```json
/// {
///     "action": "archive_tar_create",
///     "parameters": {
///         "sources": ["/home/user/documents", "/home/user/notes.txt"],
///         "destination": "/home/user/backup.tar.gz"
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ArchiveTarCreateSkill;

#[async_trait::async_trait]
impl Skill for ArchiveTarCreateSkill {
    fn name(&self) -> &str {
        "archive_tar_create"
    }

    fn description(&self) -> &str {
        "Create a TAR archive from files or directories (optionally compressed)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to create a TAR archive (optionally with gzip/bzip2 compression). Provide source paths and the destination archive path."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "sources".to_string(),
                param_type: "array".to_string(),
                description: "Array of file or directory paths to include in the archive".to_string(),
                required: true,
                default: None,
                example: Some(json!(["/home/user/data", "/home/user/config.json"])),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Path where to create the TAR archive (can end with .tar, .tar.gz, .tgz, .tar.bz2, .tbz2)".to_string(),
                required: true,
                default: None,
                example: Some(json!("/home/user/archive.tar.gz")),
                enum_values: None,
            },
            SkillParameter {
                name: "preserve_paths".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to preserve directory structure in the archive".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "archive_tar_create",
            "parameters": {
                "sources": ["/home/user/documents", "/home/user/notes.txt"],
                "destination": "/home/user/backup.tar.gz",
                "preserve_paths": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully created TAR archive at /home/user/backup.tar.gz containing 8 files"
            .to_string()
    }

    fn category(&self) -> &str {
        "archive"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let sources = parameters
            .get("sources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!("Missing or invalid 'sources' parameter (expected array)")
            })?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let preserve_paths = parameters
            .get("preserve_paths")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let dest_path = validate_path(destination, None)?;
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let dest_str = destination.to_lowercase();
        let is_gzip = dest_str.ends_with(".tar.gz") || dest_str.ends_with(".tgz");
        let is_bzip2 = dest_str.ends_with(".tar.bz2") || dest_str.ends_with(".tbz2");
        if !is_gzip && !is_bzip2 && !dest_str.ends_with(".tar") {
            anyhow::bail!("Destination must have .tar, .tar.gz/.tgz, or .tar.bz2/.tbz2 extension");
        }
        let file = File::create(&dest_path)?;
        let mut file_count = 0;
        let total_size = 0;
        let result: Result<String> = if is_gzip {
            let gz_encoder = GzEncoder::new(file, Compression::default());
            let mut tar_builder = Builder::new(gz_encoder);
            file_count = Self::add_to_tar(&mut tar_builder, sources, preserve_paths)?;
            tar_builder.finish()?;
            Ok(format!(
                "Successfully created GZIP-compressed TAR archive at {} containing {} file(s)",
                destination, file_count
            ))
        } else if is_bzip2 {
            let bz_encoder = BzEncoder::new(file, BzCompression::default());
            let mut tar_builder = Builder::new(bz_encoder);
            file_count = Self::add_to_tar(&mut tar_builder, sources, preserve_paths)?;
            tar_builder.finish()?;
            Ok(format!(
                "Successfully created BZIP2-compressed TAR archive at {} containing {} file(s)",
                destination, file_count
            ))
        } else {
            let mut tar_builder = Builder::new(file);
            file_count = Self::add_to_tar(&mut tar_builder, sources, preserve_paths)?;
            tar_builder.finish()?;
            Ok(format!(
                "Successfully created TAR archive at {} containing {} file(s)",
                destination, file_count
            ))
        };
        result
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        if parameters
            .get("sources")
            .and_then(|v| v.as_array())
            .is_none()
        {
            anyhow::bail!("Missing or invalid 'sources' parameter (expected array)");
        }
        if parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .is_none()
        {
            anyhow::bail!("Missing 'destination' parameter");
        }
        Ok(())
    }
}

impl ArchiveTarCreateSkill {
    /// Helper function to add files and directories to a TAR archive.
    ///
    /// Recursively walks through directories and adds all files to the archive builder.
    /// If preserve_paths is false, only file names are used (flattening the structure).
    fn add_to_tar<W: Write>(
        tar_builder: &mut Builder<W>,
        sources: &[Value],
        preserve_paths: bool,
    ) -> Result<usize> {
        let mut file_count = 0;
        for source in sources {
            let source_path_str = source
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Source path must be a string"))?;
            let source_path = validate_path(source_path_str, None)?;
            if !file_exists(&source_path.to_string_lossy()) {
                anyhow::bail!("Source path not found: {}", source_path_str);
            }
            let base_path = if preserve_paths {
                source_path.parent().unwrap_or(&source_path).to_path_buf()
            } else {
                PathBuf::new()
            };
            if source_path.is_dir() {
                for entry in WalkDir::new(&source_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                {
                    let file_path = entry.path();
                    let relative_path: &Path = if preserve_paths {
                        file_path.strip_prefix(&base_path).unwrap_or(file_path)
                    } else {
                        let file_name = file_path.file_name().unwrap_or_default();
                        Path::new(file_name)
                    };
                    tar_builder.append_file(relative_path, &mut File::open(file_path)?)?;
                    file_count += 1;
                }
            } else {
                let relative_path: &Path = if preserve_paths {
                    source_path.strip_prefix(&base_path).unwrap_or(&source_path)
                } else {
                    let file_name = source_path.file_name().unwrap_or_default();
                    Path::new(file_name)
                };
                tar_builder.append_file(relative_path, &mut File::open(&source_path)?)?;
                file_count += 1;
            }
        }
        Ok(file_count)
    }
}

// Helper trait extensions (unused, kept for potential future use)
trait TryIntoGz {
    fn try_into_gz(self) -> Option<GzEncoder<Vec<u8>>>;
}

trait TryIntoBz {
    fn try_into_bz(self) -> Option<BzEncoder<Vec<u8>>>;
}

impl TryIntoGz for GzEncoder<File> {
    fn try_into_gz(self) -> Option<GzEncoder<Vec<u8>>> {
        None
    }
}

impl TryIntoBz for BzEncoder<File> {
    fn try_into_bz(self) -> Option<BzEncoder<Vec<u8>>> {
        None
    }
}

/// Skill for extracting TAR archives (optionally with gzip/bzip2 compression).
///
/// This skill extracts files from TAR archives, automatically detecting and handling
/// gzip or bzip2 compression based on the file extension.
///
/// # Parameters
///
/// * `archive` (required): Path to the TAR archive file
/// * `destination` (required): Directory where to extract the files
/// * `overwrite` (optional): Whether to overwrite existing files, default false
///
/// # Supported Formats
///
/// - `.tar` - Uncompressed TAR
/// - `.tar.gz`, `.tgz` - Gzip-compressed TAR
/// - `.tar.bz2`, `.tbz2` - Bzip2-compressed TAR
///
/// # Example
///
/// ```json
/// {
///     "action": "archive_tar_extract",
///     "parameters": {
///         "archive": "/home/user/backup.tar.gz",
///         "destination": "/home/user/extracted",
///         "overwrite": true
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ArchiveTarExtractSkill;

#[async_trait::async_trait]
impl Skill for ArchiveTarExtractSkill {
    fn name(&self) -> &str {
        "archive_tar_extract"
    }

    fn description(&self) -> &str {
        "Extract a TAR archive (optionally compressed with gzip/bzip2) to a destination directory"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to extract files from a TAR archive. Provide the archive path and the destination directory. Supports .tar, .tar.gz, .tgz, .tar.bz2, .tbz2 formats."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "archive".to_string(),
                param_type: "string".to_string(),
                description: "Path to the TAR archive file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/archive.tar.gz".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Directory where to extract the files".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/extracted".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "overwrite".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to overwrite existing files".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "archive_tar_extract",
            "parameters": {
                "archive": "/home/user/backup.tar.gz",
                "destination": "/home/user/extracted",
                "overwrite": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully extracted 15 files from /home/user/backup.tar.gz to /home/user/extracted"
            .to_string()
    }

    fn category(&self) -> &str {
        "archive"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let archive = parameters
            .get("archive")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'archive' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let overwrite = parameters
            .get("overwrite")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let archive_path = validate_path(archive, None)?;
        let dest_path = validate_path(destination, None)?;
        if !file_exists(&archive_path.to_string_lossy()) {
            anyhow::bail!("Archive file not found: {}", archive);
        }
        fs::create_dir_all(&dest_path)?;
        let file = File::open(&archive_path)?;
        let archive_str = archive.to_lowercase();
        let reader: Box<dyn Read> =
            if archive_str.ends_with(".tar.gz") || archive_str.ends_with(".tgz") {
                Box::new(GzDecoder::new(file))
            } else if archive_str.ends_with(".tar.bz2") || archive_str.ends_with(".tbz2") {
                Box::new(BzDecoder::new(file))
            } else if archive_str.ends_with(".tar") {
                Box::new(file)
            } else {
                anyhow::bail!("Archive must have .tar, .tar.gz/.tgz, or .tar.bz2/.tbz2 extension");
            };
        let mut tar_archive = Archive::new(reader);
        let mut file_count = 0;
        let mut extracted_size = 0;
        for entry in tar_archive.entries()? {
            let mut entry = entry?;
            let entry_path = dest_path.join(entry.path()?);
            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(&entry_path)?;
                continue;
            }
            if entry_path.exists() && !overwrite {
                continue;
            }
            if let Some(parent) = entry_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&entry_path)?;
            let size = std::io::copy(&mut entry, &mut outfile)?;
            extracted_size += size;
            file_count += 1;
        }
        Ok(format!(
            "Successfully extracted {} file(s) from {} to {} (total size: {} bytes)",
            file_count, archive, destination, extracted_size
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        if parameters.get("archive").and_then(|v| v.as_str()).is_none() {
            anyhow::bail!("Missing 'archive' parameter");
        }
        if parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .is_none()
        {
            anyhow::bail!("Missing 'destination' parameter");
        }
        Ok(())
    }
}

/// Skill for compressing a single file using gzip or bzip2.
///
/// This skill compresses individual files using either gzip or bzip2 compression.
/// For multiple files or directories, use archive_tar_create or archive_zip_create instead.
///
/// # Parameters
///
/// * `source` (required): Path to the file to compress
/// * `destination` (optional): Output path (defaults to source + .gz or .bz2)
/// * `format` (optional): 'gzip' or 'bzip2', default 'gzip'
/// * `compression_level` (optional): 1-9, default 6
/// * `keep_original` (optional): Keep source file after compression, default false
///
/// # Example
///
/// ```json
/// {
///     "action": "archive_compress",
///     "parameters": {
///         "source": "/home/user/data.txt",
///         "format": "gzip",
///         "compression_level": 9,
///         "keep_original": true
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ArchiveCompressSkill;

#[async_trait::async_trait]
impl Skill for ArchiveCompressSkill {
    fn name(&self) -> &str {
        "archive_compress"
    }

    fn description(&self) -> &str {
        "Compress a single file using gzip or bzip2"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compress a single file using gzip or bzip2 compression. For compressing multiple files or directories, use archive_tar_create or archive_zip_create instead."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to compress".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/document.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description:
                    "Path where to create the compressed file (should end with .gz or .bz2)"
                        .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/home/user/document.txt.gz".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Compression format: 'gzip' or 'bzip2'".to_string(),
                required: false,
                default: Some(Value::String("gzip".to_string())),
                example: Some(Value::String("bzip2".to_string())),
                enum_values: Some(vec!["gzip".to_string(), "bzip2".to_string()]),
            },
            SkillParameter {
                name: "compression_level".to_string(),
                param_type: "integer".to_string(),
                description: "Compression level (1-9, where 1=fastest, 9=best compression)"
                    .to_string(),
                required: false,
                default: Some(Value::Number(6.into())),
                example: Some(Value::Number(9.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "keep_original".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to keep the original file after compression".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "archive_compress",
            "parameters": {
                "source": "/home/user/data.txt",
                "format": "gzip",
                "compression_level": 6,
                "keep_original": false
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully compressed /home/user/data.txt to /home/user/data.txt.gz (original size: 1024 bytes, compressed size: 512 bytes, ratio: 50.0%)".to_string()
    }

    fn category(&self) -> &str {
        "archive"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("gzip");
        let compression_level = parameters
            .get("compression_level")
            .and_then(|v| v.as_u64())
            .unwrap_or(6)
            .min(9)
            .max(1) as u32;
        let keep_original = parameters
            .get("keep_original")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                let path = Path::new(source);
                let extension = match format {
                    "gzip" => ".gz",
                    "bzip2" => ".bz2",
                    _ => ".gz",
                };
                Some(format!("{}{}", path.display(), extension))
            })
            .ok_or_else(|| anyhow::anyhow!("Could not determine destination path"))?;
        let source_path = validate_path(source, None)?;
        let dest_path = validate_path(&destination, None)?;
        if !file_exists(&source_path.to_string_lossy()) {
            anyhow::bail!("Source file not found: {}", source);
        }
        if source_path.is_dir() {
            anyhow::bail!("Source must be a file, not a directory");
        }
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let original_size = fs::metadata(&source_path)?.len();
        let mut source_file = File::open(&source_path)?;
        let mut source_data = Vec::new();
        source_file.read_to_end(&mut source_data)?;
        let compressed_data = match format {
            "gzip" => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::new(compression_level));
                encoder.write_all(&source_data)?;
                encoder.finish()?
            }
            "bzip2" => {
                let mut encoder = BzEncoder::new(Vec::new(), BzCompression::new(compression_level));
                encoder.write_all(&source_data)?;
                encoder.finish()?
            }
            _ => anyhow::bail!(
                "Unsupported compression format: {}. Use 'gzip' or 'bzip2'",
                format
            ),
        };
        let mut dest_file = File::create(&dest_path)?;
        dest_file.write_all(&compressed_data)?;
        let compressed_size = compressed_data.len() as u64;
        let ratio = if original_size > 0 {
            (compressed_size as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };
        if !keep_original {
            fs::remove_file(&source_path)?;
        }
        Ok(format!(
            "Successfully compressed {} to {} (original size: {} bytes, compressed size: {} bytes, ratio: {:.1}%)",
            source, destination, original_size, compressed_size, ratio
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        if parameters.get("source").and_then(|v| v.as_str()).is_none() {
            anyhow::bail!("Missing 'source' parameter");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_zip_create_and_extract() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();
        let file1_path = test_dir.join("test1.txt");
        let file2_path = test_dir.join("test2.txt");
        File::create(&file1_path)
            .unwrap()
            .write_all(b"Hello World 1")
            .unwrap();
        File::create(&file2_path)
            .unwrap()
            .write_all(b"Hello World 2")
            .unwrap();
        let zip_path = test_dir.join("test.zip");
        let extract_dir = test_dir.join("extract");
        let create_skill = ArchiveZipCreateSkill;
        let mut params = HashMap::new();
        params.insert(
            "sources".to_string(),
            json!([file1_path.to_str().unwrap(), file2_path.to_str().unwrap()]),
        );
        params.insert("destination".to_string(), json!(zip_path.to_str().unwrap()));
        let result = create_skill.execute(&params).await.unwrap();
        assert!(result.contains("Successfully created ZIP archive"));
        assert!(zip_path.exists());
        let extract_skill = ArchiveZipExtractSkill;
        let mut extract_params = HashMap::new();
        extract_params.insert("archive".to_string(), json!(zip_path.to_str().unwrap()));
        extract_params.insert(
            "destination".to_string(),
            json!(extract_dir.to_str().unwrap()),
        );
        extract_params.insert("overwrite".to_string(), json!(true));
        let extract_result = extract_skill.execute(&extract_params).await.unwrap();
        assert!(extract_result.contains("Successfully extracted"));
        assert!(extract_dir.join("test1.txt").exists());
        assert!(extract_dir.join("test2.txt").exists());
    }

    #[tokio::test]
    async fn test_compress_gzip() {
        let temp_dir = tempdir().unwrap();
        let source_file = temp_dir.path().join("test.txt");
        let content = b"This is test content for compression";
        File::create(&source_file)
            .unwrap()
            .write_all(content)
            .unwrap();
        let compress_skill = ArchiveCompressSkill;
        let mut params = HashMap::new();
        params.insert("source".to_string(), json!(source_file.to_str().unwrap()));
        params.insert("format".to_string(), json!("gzip"));
        params.insert("keep_original".to_string(), json!(true));
        let result = compress_skill.execute(&params).await.unwrap();
        assert!(result.contains("Successfully compressed"));
        let gz_file = temp_dir.path().join("test.txt.gz");
        assert!(gz_file.exists());
        assert!(source_file.exists());
        let gz_reader = GzDecoder::new(File::open(&gz_file).unwrap());
        let mut decompressed = String::new();
        std::io::BufReader::new(gz_reader)
            .read_to_string(&mut decompressed)
            .unwrap();
        assert_eq!(decompressed, String::from_utf8_lossy(content));
    }

    /// Test TAR archive creation and extraction with gzip compression
    #[tokio::test]
    async fn test_tar_create_and_extract_gzip() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path();
        let file1_path = test_dir.join("doc1.txt");
        let file2_path = test_dir.join("doc2.txt");
        let subdir = test_dir.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        let file3_path = subdir.join("doc3.txt");
        File::create(&file1_path)
            .unwrap()
            .write_all(b"Content of document 1")
            .unwrap();
        File::create(&file2_path)
            .unwrap()
            .write_all(b"Content of document 2")
            .unwrap();
        File::create(&file3_path)
            .unwrap()
            .write_all(b"Content of document 3 in subdirectory")
            .unwrap();
        let tar_path = test_dir.join("archive.tar.gz");
        let extract_dir = test_dir.join("extracted");
        let create_skill = ArchiveTarCreateSkill;
        let mut params = HashMap::new();
        params.insert(
            "sources".to_string(),
            json!([file1_path.to_str().unwrap(), subdir.to_str().unwrap()]),
        );
        params.insert("destination".to_string(), json!(tar_path.to_str().unwrap()));
        params.insert("preserve_paths".to_string(), json!(true));
        let create_result = create_skill.execute(&params).await.unwrap();
        assert!(create_result.contains("Successfully created"));
        assert!(tar_path.exists());
        let extract_skill = ArchiveTarExtractSkill;
        let mut extract_params = HashMap::new();
        extract_params.insert("archive".to_string(), json!(tar_path.to_str().unwrap()));
        extract_params.insert(
            "destination".to_string(),
            json!(extract_dir.to_str().unwrap()),
        );
        extract_params.insert("overwrite".to_string(), json!(true));
        let extract_result = extract_skill.execute(&extract_params).await.unwrap();
        assert!(extract_result.contains("Successfully extracted"));
        assert!(extract_dir.join("doc1.txt").exists());
        assert!(extract_dir.join("doc2.txt").exists());
        assert!(extract_dir.join("subdir/doc3.txt").exists());
        let extracted_content = fs::read_to_string(extract_dir.join("doc1.txt")).unwrap();
        assert_eq!(extracted_content, "Content of document 1");
    }

    /// Test bzip2 compression and decompression with compression ratio verification
    #[tokio::test]
    async fn test_compress_bzip2_with_ratio() {
        let temp_dir = tempdir().unwrap();
        let source_file = temp_dir.path().join("large.txt");
        let content = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(100);
        File::create(&source_file)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        let original_size = fs::metadata(&source_file).unwrap().len();
        let compress_skill = ArchiveCompressSkill;
        let mut params = HashMap::new();
        params.insert("source".to_string(), json!(source_file.to_str().unwrap()));
        params.insert("format".to_string(), json!("bzip2"));
        params.insert("compression_level".to_string(), json!(9));
        params.insert("keep_original".to_string(), json!(false));
        let result = compress_skill.execute(&params).await.unwrap();
        assert!(result.contains("Successfully compressed"));
        assert!(result.contains("ratio:"));
        assert!(!source_file.exists());
        let bz2_file = temp_dir.path().join("large.txt.bz2");
        assert!(bz2_file.exists());
        let compressed_size = fs::metadata(&bz2_file).unwrap().len();
        assert!(compressed_size < original_size);
        let bz2_reader = BzDecoder::new(File::open(&bz2_file).unwrap());
        let mut decompressed = String::new();
        std::io::BufReader::new(bz2_reader)
            .read_to_string(&mut decompressed)
            .unwrap();
        assert_eq!(decompressed, content);
    }
}
