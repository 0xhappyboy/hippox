//! Shared utilities for file system operations

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File as StdFile;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
pub fn validate_path(path: &str, base_dir: Option<&str>) -> Result<PathBuf> {
    let path_buf = PathBuf::from(path);
    if path_buf
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        anyhow::bail!("Path traversal not allowed: {}", path);
    }
    if let Some(base) = base_dir {
        let full_path = Path::new(base).join(&path_buf);
        let canonicalized = fs::canonicalize(&full_path)?;
        let base_canonical = fs::canonicalize(base)?;
        if !canonicalized.starts_with(base_canonical) {
            anyhow::bail!("Path is outside base directory: {}", path);
        }
        Ok(canonicalized)
    } else {
        Ok(path_buf)
    }
}

/// Check if file exists
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists() && Path::new(path).is_file()
}

/// Check if directory exists
pub fn dir_exists(path: &str) -> bool {
    Path::new(path).exists() && Path::new(path).is_dir()
}

/// Ensure directory exists
pub fn ensure_dir(path: &str) -> Result<()> {
    let dir = Path::new(path);
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Read file content as string
pub fn read_file_content(path: &str) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

/// Read file content as bytes
pub fn read_file_bytes(path: &str) -> Result<Vec<u8>> {
    let mut file = StdFile::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Write content to file
pub fn write_file_content(path: &str, content: &str, append: bool) -> Result<()> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        ensure_dir(parent.to_str().unwrap())?;
    }
    if append {
        use std::fs::OpenOptions;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(content.as_bytes())?;
    } else {
        fs::write(path, content)?;
    }
    Ok(())
}

/// Write bytes to file
pub fn write_file_bytes(path: &str, data: &[u8], append: bool) -> Result<()> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        ensure_dir(parent.to_str().unwrap())?;
    }
    let mut file = if append {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?
    } else {
        StdFile::create(path)?
    };
    file.write_all(data)?;
    Ok(())
}

/// Get file metadata
pub fn get_file_metadata(path: &str) -> Result<fs::Metadata> {
    let metadata = fs::metadata(path)?;
    Ok(metadata)
}

/// Get detailed file metadata
pub fn get_detailed_metadata(path: &str) -> Result<FileMetadata> {
    let path_obj = Path::new(path);
    let metadata = fs::metadata(path_obj)?;
    Ok(FileMetadata {
        path: path.to_string(),
        size: metadata.len(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        is_symlink: metadata.file_type().is_symlink(),
        modified: metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs(),
        created: metadata.created()?.duration_since(UNIX_EPOCH)?.as_secs(),
        accessed: metadata.accessed()?.duration_since(UNIX_EPOCH)?.as_secs(),
    })
}

/// Calculate MD5 hash of file
pub fn calculate_md5(path: &str) -> Result<String> {
    let data = read_file_bytes(path)?;
    let digest = md5::compute(&data);
    Ok(format!("{:x}", digest))
}

/// Calculate SHA1 hash of file
pub fn calculate_sha1(path: &str) -> Result<String> {
    use sha1::{Digest, Sha1};
    let data = read_file_bytes(path)?;
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(result.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Calculate SHA256 hash of file
pub fn calculate_sha256(path: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let data = read_file_bytes(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(result.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Calculate SHA512 hash of file
pub fn calculate_sha512(path: &str) -> Result<String> {
    use sha2::{Digest, Sha512};
    let data = read_file_bytes(path)?;
    let mut hasher = Sha512::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(result.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Calculate all hashes for a file
pub fn calculate_all_hashes(path: &str) -> Result<FileHashResult> {
    use sha1::{Digest as Sha1Digest, Sha1};
    use sha2::{Digest as Sha2Digest, Sha256, Sha512};
    let data = read_file_bytes(path)?;
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
    Ok(FileHashResult {
        path: path.to_string(),
        md5,
        sha1,
        sha256,
        sha512,
        blake3,
    })
}

/// Calculate hash of file content (for integrity monitoring)
pub fn calculate_file_integrity_hash(path: &str) -> Result<String> {
    calculate_sha256(path)
}

/// Get file size in bytes
pub fn get_file_size(path: &str) -> Result<u64> {
    let metadata = get_file_metadata(path)?;
    Ok(metadata.len())
}

/// Get directory size recursively
pub fn get_directory_size(path: &str) -> Result<u64> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        anyhow::bail!("Path does not exist: {}", path);
    }
    if path_obj.is_file() {
        return get_file_size(path);
    }
    let mut total_size = 0;
    for entry in walkdir::WalkDir::new(path_obj)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
    }
    Ok(total_size)
}

/// Copy file
pub fn copy_file(source: &str, destination: &str) -> Result<u64> {
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);
    if let Some(parent) = dest_path.parent() {
        ensure_dir(parent.to_str().unwrap())?;
    }
    let size = fs::copy(source_path, dest_path)?;
    Ok(size)
}

/// Copy directory recursively
pub fn copy_directory(source: &str, destination: &str) -> Result<u64> {
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);
    if !source_path.exists() {
        anyhow::bail!("Source directory does not exist: {}", source);
    }
    if !source_path.is_dir() {
        anyhow::bail!("Source is not a directory: {}", source);
    }
    ensure_dir(dest_path.to_str().unwrap())?;
    let mut total_size = 0;
    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let source_entry = entry.path();
        let dest_entry = dest_path.join(entry.file_name());
        if source_entry.is_dir() {
            total_size +=
                copy_directory(source_entry.to_str().unwrap(), dest_entry.to_str().unwrap())?;
        } else {
            total_size += copy_file(source_entry.to_str().unwrap(), dest_entry.to_str().unwrap())?;
        }
    }
    Ok(total_size)
}

/// Delete file or directory
pub fn delete_path(path: &str, recursive: bool) -> Result<()> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Ok(());
    }
    if path_obj.is_dir() {
        if recursive {
            fs::remove_dir_all(path_obj)?;
        } else {
            fs::remove_dir(path_obj)?;
        }
    } else {
        fs::remove_file(path_obj)?;
    }
    Ok(())
}

/// Move file or directory
pub fn move_path(source: &str, destination: &str, overwrite: bool) -> Result<()> {
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);
    if !source_path.exists() {
        anyhow::bail!("Source does not exist: {}", source);
    }
    if dest_path.exists() && !overwrite {
        anyhow::bail!("Destination already exists: {}", destination);
    }
    if let Some(parent) = dest_path.parent() {
        ensure_dir(parent.to_str().unwrap())?;
    }
    if dest_path.exists() && overwrite {
        if dest_path.is_dir() {
            fs::remove_dir_all(dest_path)?;
        } else {
            fs::remove_file(dest_path)?;
        }
    }
    fs::rename(source_path, dest_path)?;
    Ok(())
}

/// List directory contents
pub fn list_directory(path: &str, recursive: bool, show_hidden: bool) -> Result<Vec<PathBuf>> {
    let path_obj = Path::new(path);
    if !path_obj.exists() || !path_obj.is_dir() {
        anyhow::bail!("Directory does not exist: {}", path);
    }
    let mut entries = Vec::new();
    if recursive {
        for entry in walkdir::WalkDir::new(path_obj)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let name = entry.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            entries.push(entry.path().to_path_buf());
        }
    } else {
        for entry in fs::read_dir(path_obj)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            entries.push(entry.path());
        }
    }
    Ok(entries)
}

/// Check if file signature is valid (simplified)
pub fn verify_file_signature(path: &str, expected_signature: &str) -> Result<bool> {
    let current_hash = calculate_sha256(path)?;
    Ok(current_hash == expected_signature)
}

/// Detect file magic bytes
pub fn detect_magic_bytes(path: &str) -> Result<Option<String>> {
    let data = read_file_bytes(path)?;
    if data.len() < 4 {
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
            return Ok(Some(description.to_string()));
        }
    }

    Ok(Some(format!("Unknown/Other (0x{})", magic_hex)))
}

/// Simple virus scan (signature-based)
pub fn scan_file_for_viruses(path: &str, signatures: &[&str]) -> Result<VirusScanResult> {
    let data = read_file_bytes(path)?;
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

    Ok(VirusScanResult {
        path: path.to_string(),
        infected,
        virus_name,
        scan_time: chrono::Local::now().to_string(),
        file_size,
    })
}

/// Perform forensic analysis on a file
pub fn perform_forensic_analysis(path: &str) -> Result<ForensicResult> {
    let path_obj = Path::new(path);
    let metadata = fs::metadata(path_obj)?;
    let file_type = if metadata.is_dir() {
        "Directory"
    } else {
        "File"
    };

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
        detect_magic_bytes(path)?
    } else {
        None
    };

    // Check for suspicious patterns
    if metadata.is_file() {
        let data = read_file_bytes(path)?;
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

    Ok(ForensicResult {
        path: path.to_string(),
        file_type: file_type.to_string(),
        magic_bytes,
        embedded_metadata,
        suspicious,
        suspicious_reasons,
    })
}

/// Create a backup of a file or directory
pub fn create_backup(path: &str, backup_dir: &str) -> Result<String> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        anyhow::bail!("Path does not exist: {}", path);
    }

    ensure_dir(backup_dir)?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let file_name = path_obj.file_name().unwrap_or_default();
    let backup_name = format!("{}_{}.backup", file_name.to_string_lossy(), timestamp);
    let backup_path = Path::new(backup_dir).join(backup_name);

    if path_obj.is_dir() {
        copy_directory(path, backup_path.to_str().unwrap())?;
    } else {
        copy_file(path, backup_path.to_str().unwrap())?;
    }
    Ok(backup_path.to_string_lossy().to_string())
}

/// Pack logs into archive
pub fn pack_logs(source_dir: &str, destination: &str, archive_format: &str) -> Result<String> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::fs::File;
    use tar::Builder;
    let source_path = Path::new(source_dir);
    if !source_path.exists() || !source_path.is_dir() {
        anyhow::bail!("Source directory does not exist: {}", source_dir);
    }
    ensure_dir(
        Path::new(destination)
            .parent()
            .unwrap_or(Path::new(""))
            .to_str()
            .unwrap(),
    )?;
    let archive_path = match archive_format {
        "tar" => format!("{}.tar", destination),
        "tar.gz" | "tgz" => format!("{}.tar.gz", destination),
        "zip" => format!("{}.zip", destination),
        _ => anyhow::bail!("Unsupported archive format: {}", archive_format),
    };
    if archive_format == "zip" {
        use zip::ZipWriter;
        use zip::write::FileOptions;
        let file = File::create(&archive_path)?;
        let mut zip = ZipWriter::new(file);
        let options: zip::write::FileOptions<'_, ()> = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));
        for entry in walkdir::WalkDir::new(source_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(source_path)?;
            zip.start_file(relative_path.to_string_lossy(), options)?;
            let mut file = File::open(file_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
        zip.finish()?;
    } else if archive_format == "tar" || archive_format == "tar.gz" || archive_format == "tgz" {
        let file = File::create(&archive_path)?;
        let writer: Box<dyn Write> = if archive_format == "tar" {
            Box::new(file)
        } else {
            Box::new(GzEncoder::new(file, Compression::default()))
        };
        let mut tar_builder = Builder::new(writer);
        for entry in walkdir::WalkDir::new(source_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(source_path)?;
            tar_builder.append_file(relative_path, &mut File::open(file_path)?)?;
        }
        tar_builder.finish()?;
    }
    Ok(archive_path)
}
