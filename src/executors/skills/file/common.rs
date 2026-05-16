use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Validate and sanitize file path to prevent directory traversal attacks
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

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists() && Path::new(path).is_file()
}

/// Check if a directory exists
pub fn dir_exists(path: &str) -> bool {
    Path::new(path).exists() && Path::new(path).is_dir()
}

/// Ensure directory exists, create if not
pub fn ensure_dir(path: &str) -> Result<()> {
    let dir = Path::new(path);
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Get file extension
pub fn get_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

/// Read file content as string
pub fn read_file_content(path: &str) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

/// Write content to file
pub fn write_file_content(path: &str, content: &str, append: bool) -> Result<()> {
    if append {
        fs::write(path, content)?;
    } else {
        fs::write(path, content)?;
    }
    Ok(())
}

/// Get file metadata
pub fn get_file_metadata(path: &str) -> Result<fs::Metadata> {
    let metadata = fs::metadata(path)?;
    Ok(metadata)
}
