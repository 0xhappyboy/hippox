//! Shared utilities for file system operations
//!
//! This module provides comprehensive file system utilities including
//! file operations, directory management, file hashing, and forensic analysis.
use crate::DriverError;
use crate::result::DriverResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File as StdFile;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};
/// File metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub modified: u64,
    pub created: u64,
    pub accessed: u64,
}
/// File hash result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashResult {
    pub path: String,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub blake3: Option<String>,
}
/// File integrity result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityResult {
    pub path: String,
    pub changed: bool,
    pub previous_hash: String,
    pub current_hash: String,
    pub action: String, // "added", "modified", "deleted", "unchanged"
}
/// Virus scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirusScanResult {
    pub path: String,
    pub infected: bool,
    pub virus_name: Option<String>,
    pub scan_time: String,
    pub file_size: u64,
}
/// Disk forensic result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicResult {
    pub path: String,
    pub file_type: String,
    pub magic_bytes: Option<String>,
    pub embedded_metadata: Vec<(String, String)>,
    pub suspicious: bool,
    pub suspicious_reasons: Vec<String>,
}
/// Validate and sanitize file path
pub fn validate_path(path: &str, base_dir: Option<&str>) -> DriverResult<PathBuf> {
    debug!("Validating path: {}, base_dir: {:?}", path, base_dir);
    let path_buf = PathBuf::from(path);
    if path_buf.components().any(|c| c == std::path::Component::ParentDir) {
        let err_msg = format!("Path traversal not allowed: {}", path);
        warn!("{}", err_msg);
        return Err(DriverError::validation("path", err_msg));
    }
    if let Some(base) = base_dir {
        let full_path = Path::new(base).join(&path_buf);
        match fs::canonicalize(&full_path) {
            Ok(canonicalized) => match fs::canonicalize(base) {
                Ok(base_canonical) => {
                    if !canonicalized.starts_with(base_canonical) {
                        let err_msg = format!("Path is outside base directory: {}", path);
                        warn!("{}", err_msg);
                        return Err(DriverError::validation("path", err_msg));
                    }
                    info!("Path validated: {:?}", canonicalized);
                    return Ok(canonicalized);
                }
                Err(e) => {
                    let err_msg = format!("Failed to canonicalize base dir: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            },
            Err(e) => {
                let err_msg = format!("Failed to canonicalize path: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    } else {
        info!("Path validated: {:?}", path_buf);
        return Ok(path_buf);
    }
}
/// Check if file exists
pub fn file_exists(path: &str) -> bool {
    let exists = Path::new(path).exists() && Path::new(path).is_file();
    debug!("File exists check for {}: {}", path, exists);
    return exists;
}
/// Check if directory exists
pub fn dir_exists(path: &str) -> bool {
    let exists = Path::new(path).exists() && Path::new(path).is_dir();
    debug!("Directory exists check for {}: {}", path, exists);
    return exists;
}
/// Ensure directory exists
pub fn ensure_dir(path: &str) -> DriverResult<()> {
    debug!("Ensuring directory exists: {}", path);
    let dir = Path::new(path);
    if !dir.exists() {
        match fs::create_dir_all(dir) {
            Ok(_) => {
                info!("Directory created: {}", path);
                return Ok(());
            }
            Err(e) => {
                let err_msg = format!("Failed to create directory {}: {}", path, e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    }
    info!("Directory already exists: {}", path);
    return Ok(());
}
/// Read file content as string
pub fn read_file_content(path: &str) -> DriverResult<String> {
    debug!("Reading file content: {}", path);
    match fs::read_to_string(path) {
        Ok(content) => {
            info!("File read successfully: {} ({} bytes)", path, content.len());
            return Ok(content);
        }
        Err(e) => {
            let err_msg = format!("Failed to read file {}: {}", path, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    }
}
/// Read file content as bytes
pub fn read_file_bytes(path: &str) -> DriverResult<Vec<u8>> {
    debug!("Reading file bytes: {}", path);
    let mut file = match StdFile::open(path) {
        Ok(f) => f,
        Err(e) => {
            let err_msg = format!("Failed to open file {}: {}", path, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    };
    let mut buffer = Vec::new();
    match file.read_to_end(&mut buffer) {
        Ok(_) => {
            info!("File bytes read successfully: {} ({} bytes)", path, buffer.len());
            return Ok(buffer);
        }
        Err(e) => {
            let err_msg = format!("Failed to read file bytes {}: {}", path, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    }
}
/// Write content to file
pub fn write_file_content(path: &str, content: &str, append: bool) -> DriverResult<()> {
    debug!("Writing file content: {}, append: {}", path, append);
    let path_obj = Path::new(path);
    if let Some(parent) = path_obj.parent() {
        let _ = ensure_dir(parent.to_str().unwrap())?;
    }
    if append {
        use std::fs::OpenOptions;
        let mut file = match OpenOptions::new().create(true).append(true).open(path_obj) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("Failed to open file for append {}: {}", path, e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        };
        match file.write_all(content.as_bytes()) {
            Ok(_) => {
                info!("Content appended to file: {}", path);
                return Ok(());
            }
            Err(e) => {
                let err_msg = format!("Failed to append to file {}: {}", path, e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    } else {
        match fs::write(path_obj, content) {
            Ok(_) => {
                info!("Content written to file: {}", path);
                return Ok(());
            }
            Err(e) => {
                let err_msg = format!("Failed to write file {}: {}", path, e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    }
}
/// Write bytes to file
pub fn write_file_bytes(path: &str, data: &[u8], append: bool) -> DriverResult<()> {
    debug!("Writing file bytes: {}, append: {}", path, append);
    let path_obj = Path::new(path);
    if let Some(parent) = path_obj.parent() {
        let _ = ensure_dir(parent.to_str().unwrap())?;
    }
    let mut file = if append {
        match fs::OpenOptions::new().create(true).append(true).open(path_obj) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("Failed to open file for append {}: {}", path, e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    } else {
        match StdFile::create(path_obj) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("Failed to create file {}: {}", path, e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    };
    match file.write_all(data) {
        Ok(_) => {
            info!("Bytes written to file: {}", path);
            return Ok(());
        }
        Err(e) => {
            let err_msg = format!("Failed to write bytes to file {}: {}", path, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    }
}
/// Get file metadata
pub fn get_file_metadata(path: &str) -> DriverResult<fs::Metadata> {
    debug!("Getting file metadata: {}", path);
    match fs::metadata(path) {
        Ok(metadata) => {
            info!("File metadata retrieved: {}", path);
            return Ok(metadata);
        }
        Err(e) => {
            let err_msg = format!("Failed to get metadata for {}: {}", path, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    }
}
/// Get detailed file metadata
pub fn get_detailed_metadata(path: &str) -> DriverResult<FileMetadata> {
    debug!("Getting detailed file metadata: {}", path);
    let path_obj = Path::new(path);
    let metadata = match fs::metadata(path_obj) {
        Ok(m) => m,
        Err(e) => {
            let err_msg = format!("Failed to get metadata for {}: {}", path, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    };
    let modified = match metadata.modified() {
        Ok(t) => match t.duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(e) => {
                let err_msg = format!("Failed to get modified time: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::internal(err_msg));
            }
        },
        Err(e) => {
            let err_msg = format!("Failed to get modified time: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
    let created = match metadata.created() {
        Ok(t) => match t.duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(e) => {
                let err_msg = format!("Failed to get created time: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::internal(err_msg));
            }
        },
        Err(e) => {
            let err_msg = format!("Failed to get created time: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
    let accessed = match metadata.accessed() {
        Ok(t) => match t.duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(e) => {
                let err_msg = format!("Failed to get accessed time: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::internal(err_msg));
            }
        },
        Err(e) => {
            let err_msg = format!("Failed to get accessed time: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
    let result = FileMetadata {
        path: path.to_string(),
        size: metadata.len(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        is_symlink: metadata.file_type().is_symlink(),
        modified,
        created,
        accessed,
    };
    info!("Detailed metadata retrieved for: {}", path);
    return Ok(result);
}
/// Calculate MD5 hash of file
pub fn calculate_md5(path: &str) -> DriverResult<String> {
    debug!("Calculating MD5 hash for: {}", path);
    let data = match read_file_bytes(path) {
        Ok(d) => d,
        Err(e) => {
            let err_msg = format!("Failed to read file for MD5: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let digest = md5::compute(&data);
    let result = format!("{:x}", digest);
    info!("MD5 hash calculated for: {}", path);
    return Ok(result);
}
/// Calculate SHA1 hash of file
pub fn calculate_sha1(path: &str) -> DriverResult<String> {
    debug!("Calculating SHA1 hash for: {}", path);
    let data = match read_file_bytes(path) {
        Ok(d) => d,
        Err(e) => {
            let err_msg = format!("Failed to read file for SHA1: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let hash = result.iter().map(|b| format!("{:02x}", b)).collect();
    info!("SHA1 hash calculated for: {}", path);
    return Ok(hash);
}
/// Calculate SHA256 hash of file
pub fn calculate_sha256(path: &str) -> DriverResult<String> {
    debug!("Calculating SHA256 hash for: {}", path);
    let data = match read_file_bytes(path) {
        Ok(d) => d,
        Err(e) => {
            let err_msg = format!("Failed to read file for SHA256: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let hash = result.iter().map(|b| format!("{:02x}", b)).collect();
    info!("SHA256 hash calculated for: {}", path);
    return Ok(hash);
}
/// Calculate SHA512 hash of file
pub fn calculate_sha512(path: &str) -> DriverResult<String> {
    debug!("Calculating SHA512 hash for: {}", path);
    let data = match read_file_bytes(path) {
        Ok(d) => d,
        Err(e) => {
            let err_msg = format!("Failed to read file for SHA512: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    use sha2::{Digest, Sha512};
    let mut hasher = Sha512::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let hash = result.iter().map(|b| format!("{:02x}", b)).collect();
    info!("SHA512 hash calculated for: {}", path);
    return Ok(hash);
}
/// Calculate all hashes for a file
pub fn calculate_all_hashes(path: &str) -> DriverResult<FileHashResult> {
    debug!("Calculating all hashes for: {}", path);
    let data = match read_file_bytes(path) {
        Ok(d) => d,
        Err(e) => {
            let err_msg = format!("Failed to read file for hashing: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    use sha1::{Digest as Sha1Digest, Sha1};
    use sha2::{Digest as Sha2Digest, Sha256, Sha512};
    // MD5
    let md5_digest = md5::compute(&data);
    let md5 = Some(format!("{:x}", md5_digest));
    // SHA1
    let mut sha1_hasher = Sha1::new();
    sha1_hasher.update(&data);
    let sha1_result = sha1_hasher.finalize();
    let sha1 = Some(sha1_result.iter().map(|b| format!("{:02x}", b)).collect());
    // SHA256
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(&data);
    let sha256_result = sha256_hasher.finalize();
    let sha256 = Some(sha256_result.iter().map(|b| format!("{:02x}", b)).collect());
    // SHA512
    let mut sha512_hasher = Sha512::new();
    sha512_hasher.update(&data);
    let sha512_result = sha512_hasher.finalize();
    let sha512 = Some(sha512_result.iter().map(|b| format!("{:02x}", b)).collect());
    // BLAKE3
    let blake3 = Some(blake3::hash(&data).to_string());
    let result = FileHashResult { path: path.to_string(), md5, sha1, sha256, sha512, blake3 };
    info!("All hashes calculated for: {}", path);
    return Ok(result);
}
/// Get file size in bytes
pub fn get_file_size(path: &str) -> DriverResult<u64> {
    debug!("Getting file size: {}", path);
    let metadata = match get_file_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            let err_msg = format!("Failed to get metadata for size: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    info!("File size: {} bytes for {}", metadata.len(), path);
    return Ok(metadata.len());
}
/// Calculate hash of file content (for integrity monitoring)
pub fn calculate_file_integrity_hash(path: &str) -> DriverResult<String> {
    debug!("Calculating integrity hash for: {}", path);
    return calculate_sha256(path);
}
/// Get directory size recursively
pub fn get_directory_size(path: &str) -> DriverResult<u64> {
    debug!("Getting directory size: {}", path);
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        let err_msg = format!("Path does not exist: {}", path);
        warn!("{}", err_msg);
        return Err(DriverError::validation("path", err_msg));
    }
    if path_obj.is_file() {
        return get_file_size(path);
    }
    let mut total_size = 0;
    for entry in walkdir::WalkDir::new(path_obj).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
    }
    info!("Directory size: {} bytes for {}", total_size, path);
    return Ok(total_size);
}
/// Copy file
pub fn copy_file(source: &str, destination: &str) -> DriverResult<u64> {
    debug!("Copying file: {} -> {}", source, destination);
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);
    if let Some(parent) = dest_path.parent() {
        let _ = ensure_dir(parent.to_str().unwrap())?;
    }
    match fs::copy(source_path, dest_path) {
        Ok(size) => {
            info!("File copied: {} -> {} ({} bytes)", source, destination, size);
            return Ok(size);
        }
        Err(e) => {
            let err_msg = format!("Failed to copy file {} -> {}: {}", source, destination, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    }
}
/// Copy directory recursively
pub fn copy_directory(source: &str, destination: &str) -> DriverResult<u64> {
    debug!("Copying directory: {} -> {}", source, destination);
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);
    if !source_path.exists() {
        let err_msg = format!("Source directory does not exist: {}", source);
        warn!("{}", err_msg);
        return Err(DriverError::validation("source", err_msg));
    }
    if !source_path.is_dir() {
        let err_msg = format!("Source is not a directory: {}", source);
        warn!("{}", err_msg);
        return Err(DriverError::validation("source", err_msg));
    }
    let _ = ensure_dir(dest_path.to_str().unwrap())?;
    let mut total_size = 0;
    for entry in match fs::read_dir(source_path) {
        Ok(e) => e,
        Err(e) => {
            let err_msg = format!("Failed to read directory {}: {}", source, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    } {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                let err_msg = format!("Failed to read directory entry: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        };
        let source_entry = entry.path();
        let dest_entry = dest_path.join(entry.file_name());
        if source_entry.is_dir() {
            total_size += match copy_directory(source_entry.to_str().unwrap(), dest_entry.to_str().unwrap()) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("Failed to copy subdirectory: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::execution(err_msg));
                }
            };
        } else {
            total_size += match copy_file(source_entry.to_str().unwrap(), dest_entry.to_str().unwrap()) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = format!("Failed to copy file: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::execution(err_msg));
                }
            };
        }
    }
    info!("Directory copied: {} -> {} ({} bytes)", source, destination, total_size);
    return Ok(total_size);
}
/// Delete file or directory
pub fn delete_path(path: &str, recursive: bool) -> DriverResult<()> {
    debug!("Deleting path: {}, recursive: {}", path, recursive);
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        info!("Path does not exist, skipping deletion: {}", path);
        return Ok(());
    }
    let result =
        if path_obj.is_dir() { if recursive { fs::remove_dir_all(path_obj) } else { fs::remove_dir(path_obj) } } else { fs::remove_file(path_obj) };
    match result {
        Ok(_) => {
            info!("Path deleted: {}", path);
            return Ok(());
        }
        Err(e) => {
            let err_msg = format!("Failed to delete path {}: {}", path, e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    }
}
/// Move file or directory
pub fn move_path(source: &str, destination: &str, overwrite: bool) -> DriverResult<()> {
    debug!("Moving path: {} -> {}, overwrite: {}", source, destination, overwrite);
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);
    if !source_path.exists() {
        let err_msg = format!("Source does not exist: {}", source);
        warn!("{}", err_msg);
        return Err(DriverError::validation("source", err_msg));
    }
    if dest_path.exists() && !overwrite {
        let err_msg = format!("Destination already exists: {}", destination);
        warn!("{}", err_msg);
        return Err(DriverError::validation("destination", err_msg));
    }
    if let Some(parent) = dest_path.parent() {
        let _ = ensure_dir(parent.to_str().unwrap())?;
    }
    if dest_path.exists() && overwrite {
        if dest_path.is_dir() {
            match fs::remove_dir_all(dest_path) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to remove existing directory: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
        } else {
            match fs::remove_file(dest_path) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to remove existing file: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
        }
    }
    match fs::rename(source_path, dest_path) {
        Ok(_) => {
            info!("Path moved: {} -> {}", source, destination);
            return Ok(());
        }
        Err(e) => {
            let err_msg = format!("Failed to move path: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    }
}
/// List directory contents
pub fn list_directory(path: &str, recursive: bool, show_hidden: bool) -> DriverResult<Vec<PathBuf>> {
    debug!("Listing directory: {}, recursive: {}, show_hidden: {}", path, recursive, show_hidden);
    let path_obj = Path::new(path);
    if !path_obj.exists() || !path_obj.is_dir() {
        let err_msg = format!("Directory does not exist: {}", path);
        warn!("{}", err_msg);
        return Err(DriverError::validation("path", err_msg));
    }
    let mut entries = Vec::new();
    if recursive {
        for entry in walkdir::WalkDir::new(path_obj).into_iter().filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            entries.push(entry.path().to_path_buf());
        }
    } else {
        for entry in match fs::read_dir(path_obj) {
            Ok(e) => e,
            Err(e) => {
                let err_msg = format!("Failed to read directory {}: {}", path, e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        } {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    let err_msg = format!("Failed to read directory entry: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            };
            let name = entry.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            entries.push(entry.path());
        }
    }
    info!("Listed {} entries in directory {}", entries.len(), path);
    return Ok(entries);
}
/// Check if file signature is valid (simplified)
pub fn verify_file_signature(path: &str, expected_signature: &str) -> DriverResult<bool> {
    debug!("Verifying file signature: {}", path);
    let current_hash = calculate_sha256(path)?;
    let valid = current_hash == expected_signature;
    info!("File signature verification for {}: {}", path, valid);
    return Ok(valid);
}
/// Detect file magic bytes
pub fn detect_magic_bytes(path: &str) -> DriverResult<Option<String>> {
    debug!("Detecting magic bytes for: {}", path);
    let data = match read_file_bytes(path) {
        Ok(d) => d,
        Err(e) => {
            let err_msg = format!("Failed to read file for magic bytes: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    if data.len() < 4 {
        info!("File too small for magic bytes detection: {}", path);
        return Ok(None);
    }
    let magic = &data[..4];
    let magic_hex = hex::encode(magic);
    // Common magic bytes
    let magic_map = [
        ("89504e47", "PNG image"),
        ("ffd8ffe0", "JPEG image"),
        ("ffd8ffe1", "JPEG image"),
        ("ffd8ffe2", "JPEG image"),
        ("47494638", "GIF image"),
        ("25504446", "PDF document"),
        ("504b0304", "ZIP archive"),
        ("1f8b0800", "GZIP archive"),
        ("7f454c46", "ELF executable"),
        ("4d5a9000", "PE executable"),
        ("23212f62", "Shell script"),
        ("efbbbf", "UTF-8 BOM"),
        ("3c3f786d", "XML document"),
        ("7b0d0a0a", "JSON document"),
        ("5b0d0a0a", "JSON array"),
    ];
    for (hex_str, description) in &magic_map {
        if magic_hex.starts_with(hex_str) {
            info!("Magic bytes detected for {}: {}", path, description);
            return Ok(Some(description.to_string()));
        }
    }
    info!("Unknown magic bytes for {}: 0x{}", path, magic_hex);
    return Ok(Some(format!("Unknown/Other (0x{})", magic_hex)));
}
/// Simple virus scan (signature-based)
pub fn scan_file_for_viruses(path: &str, signatures: &[&str]) -> DriverResult<VirusScanResult> {
    debug!("Scanning file for viruses: {}", path);
    let data = match read_file_bytes(path) {
        Ok(d) => d,
        Err(e) => {
            let err_msg = format!("Failed to read file for virus scan: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    };
    let file_size = data.len() as u64;
    let mut infected = false;
    let mut virus_name = None;
    // Convert data to hex for pattern matching
    let hex_data = hex::encode(&data);
    for signature in signatures {
        if hex_data.contains(signature) {
            infected = true;
            virus_name = Some(format!("Virus signature: {}", signature));
            break;
        }
    }
    let result = VirusScanResult { path: path.to_string(), infected, virus_name, scan_time: chrono::Local::now().to_string(), file_size };
    info!("Virus scan completed for {}: infected={}", path, infected);
    return Ok(result);
}
/// Perform forensic analysis on a file
pub fn perform_forensic_analysis(path: &str) -> DriverResult<ForensicResult> {
    debug!("Performing forensic analysis on: {}", path);
    let path_obj = Path::new(path);
    let metadata = match fs::metadata(path_obj) {
        Ok(m) => m,
        Err(e) => {
            let err_msg = format!("Failed to get metadata for forensic analysis: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::io(err_msg));
        }
    };
    let file_type = if metadata.is_dir() { "Directory" } else { "File" };
    let mut embedded_metadata = Vec::new();
    let mut suspicious = false;
    let mut suspicious_reasons = Vec::new();
    // Add basic metadata
    embedded_metadata.push(("Size".to_string(), format!("{} bytes", metadata.len())));
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
            embedded_metadata.push(("Modified".to_string(), format!("{}", duration.as_secs())));
        }
    }
    // Detect magic bytes
    let magic_bytes = if metadata.is_file() {
        match detect_magic_bytes(path) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to detect magic bytes: {}", e);
                None
            }
        }
    } else {
        None
    };
    // Check for suspicious patterns
    if metadata.is_file() {
        let data = match read_file_bytes(path) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to read file for suspicious patterns: {}", e);
                return Err(DriverError::execution(format!("Failed to read file: {}", e)));
            }
        };
        // Check for suspicious strings
        let data_str = String::from_utf8_lossy(&data);
        let suspicious_strings = ["eval(", "exec(", "base64_decode", "system(", "shell_exec"];
        for s in suspicious_strings {
            if data_str.contains(s) {
                suspicious_reasons.push(format!("Contains suspicious code pattern: {}", s));
                suspicious = true;
            }
        }
        // Check for binary file with weird permissions (Unix only)
        #[cfg(unix)]
        {
            if metadata.permissions().mode() & 0o111 != 0 {
                if !data_str.contains("ELF") && !data_str.contains("PE") {
                    suspicious_reasons.push("Executable file with unusual format".to_string());
                    suspicious = true;
                }
            }
        }
        // Check for large file with no extension
        if metadata.len() > 100_000_000 {
            if path_obj.extension().is_none() {
                suspicious_reasons.push("Large file with no extension".to_string());
                suspicious = true;
            }
        }
    }
    let result =
        ForensicResult { path: path.to_string(), file_type: file_type.to_string(), magic_bytes, embedded_metadata, suspicious, suspicious_reasons };
    info!("Forensic analysis completed for {}: suspicious={}", path, suspicious);
    return Ok(result);
}
/// Create a backup of a file or directory
pub fn create_backup(path: &str, backup_dir: &str) -> DriverResult<String> {
    debug!("Creating backup of: {} to {}", path, backup_dir);
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        let err_msg = format!("Path does not exist: {}", path);
        warn!("{}", err_msg);
        return Err(DriverError::validation("path", err_msg));
    }
    let _ = ensure_dir(backup_dir)?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let file_name = path_obj.file_name().unwrap_or_default();
    let backup_name = format!("{}_{}.backup", file_name.to_string_lossy(), timestamp);
    let backup_path = Path::new(backup_dir).join(backup_name);
    if path_obj.is_dir() {
        let _ = copy_directory(path, backup_path.to_str().unwrap())?;
    } else {
        let _ = copy_file(path, backup_path.to_str().unwrap())?;
    }
    info!("Backup created: {}", backup_path.to_string_lossy());
    return Ok(backup_path.to_string_lossy().to_string());
}
/// Pack logs into archive
pub fn pack_logs(source_dir: &str, destination: &str, archive_format: &str) -> DriverResult<String> {
    debug!("Packing logs from: {} to {} format: {}", source_dir, destination, archive_format);
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::fs::File;
    use tar::Builder;
    let source_path = Path::new(source_dir);
    if !source_path.exists() || !source_path.is_dir() {
        let err_msg = format!("Source directory does not exist: {}", source_dir);
        warn!("{}", err_msg);
        return Err(DriverError::validation("source_dir", err_msg));
    }
    let dest_parent = Path::new(destination).parent().unwrap_or(Path::new(""));
    let _ = ensure_dir(dest_parent.to_str().unwrap())?;
    let archive_path = match archive_format {
        "tar" => format!("{}.tar", destination),
        "tar.gz" | "tgz" => format!("{}.tar.gz", destination),
        "zip" => format!("{}.zip", destination),
        _ => {
            let err_msg = format!("Unsupported archive format: {}", archive_format);
            warn!("{}", err_msg);
            return Err(DriverError::validation("archive_format", err_msg));
        }
    };
    if archive_format == "zip" {
        use zip::ZipWriter;
        use zip::write::FileOptions;
        let file = match File::create(&archive_path) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("Failed to create archive file: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        };
        let mut zip = ZipWriter::new(file);
        let options: zip::write::FileOptions<'_, ()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Deflated).compression_level(Some(6));
        for entry in walkdir::WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let file_path = entry.path();
            let relative_path = match file_path.strip_prefix(source_path) {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = format!("Failed to strip prefix: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::internal(err_msg));
                }
            };
            match zip.start_file(relative_path.to_string_lossy(), options) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to start zip file: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
            let mut file = match File::open(file_path) {
                Ok(f) => f,
                Err(e) => {
                    let err_msg = format!("Failed to open file for zip: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            };
            let mut buffer = Vec::new();
            match file.read_to_end(&mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to read file for zip: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
            match zip.write_all(&buffer) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to write to zip: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
        }
        match zip.finish() {
            Ok(_) => {}
            Err(e) => {
                let err_msg = format!("Failed to finish zip: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    } else if archive_format == "tar" || archive_format == "tar.gz" || archive_format == "tgz" {
        let file = match File::create(&archive_path) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("Failed to create tar file: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        };
        let writer: Box<dyn Write> = if archive_format == "tar" { Box::new(file) } else { Box::new(GzEncoder::new(file, Compression::default())) };
        let mut tar_builder = Builder::new(writer);
        for entry in walkdir::WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let file_path = entry.path();
            let relative_path = match file_path.strip_prefix(source_path) {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = format!("Failed to strip prefix: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::internal(err_msg));
                }
            };
            let mut file = match File::open(file_path) {
                Ok(f) => f,
                Err(e) => {
                    let err_msg = format!("Failed to open file for tar: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            };
            match tar_builder.append_file(relative_path, &mut file) {
                Ok(_) => {}
                Err(e) => {
                    let err_msg = format!("Failed to append to tar: {}", e);
                    warn!("{}", err_msg);
                    return Err(DriverError::io(err_msg));
                }
            }
        }
        match tar_builder.finish() {
            Ok(_) => {}
            Err(e) => {
                let err_msg = format!("Failed to finish tar: {}", e);
                warn!("{}", err_msg);
                return Err(DriverError::io(err_msg));
            }
        }
    }
    info!("Logs packed to: {}", archive_path);
    return Ok(archive_path);
}
