/// File common module
///
/// This module provides reusable file system utilities that can be used by file-related skills.
///
/// # Examples
///
/// ## Validate and sanitize file path
///
/// ```rust
/// use crate::executors::utils::File;
///
/// let safe_path = File::validate_path("documents/readme.txt", None)?;
/// ```
///
/// ## Check if file exists
///
/// ```rust
/// use crate::executors::utils::File;
///
/// if File::file_exists("/tmp/test.txt") {
///     let content = File::read_file_content("/tmp/test.txt")?;
///     println!("{}", content);
/// }
/// ```
///
/// ## Read and write files
///
/// ```rust
/// use crate::executors::utils::File;
///
/// // Ensure directory exists
/// File::ensure_dir("/tmp/myapp/logs")?;
///
/// // Write file
/// File::write_file_content("/tmp/myapp/logs/app.log", "Hello World", false)?;
///
/// // Read file
/// let content = File::read_file_content("/tmp/myapp/logs/app.log")?;
/// ```
///
/// ## Get file metadata
///
/// ```rust
/// use crate::executors::utils::File;
///
/// let metadata = File::get_file_metadata("/tmp/test.txt")?;
/// println!("File size: {} bytes", metadata.len());
/// ```
///
/// ## Copy files and directories
///
/// ```rust
/// use crate::executors::utils::File;
///
/// // Copy single file
/// File::copy("/tmp/source.txt", "/tmp/dest.txt", false)?;
///
/// // Copy directory recursively
/// File::copy("/tmp/source_dir", "/tmp/dest_dir", true)?;
/// ```
///
/// ## Delete files and directories
///
/// ```rust
/// use crate::executors::utils::File;
///
/// // Delete single file
/// File::delete("/tmp/test.txt", false)?;
///
/// // Delete directory recursively
/// File::delete("/tmp/myapp", true)?;
/// ```
///
/// ## List directory contents
///
/// ```rust
/// use crate::executors::utils::File;
///
/// let entries = File::list_directory("/tmp", false)?;
/// for entry in entries {
///     println!("{}", entry.display());
/// }
/// ```
///
/// ## Get file/directory size
///
/// ```rust
/// use crate::executors::utils::File;
///
/// let file_size = File::get_file_size("/tmp/test.txt")?;
/// let dir_size = File::get_directory_size("/tmp/myapp")?;
/// ```
///
/// ## Safe path operations
///
/// ```rust
/// use crate::executors::utils::File;
///
/// let normalized = File::normalize_path("/home/user/../user/docs/./file.txt")?;
/// let safe_joined = File::join_safe("/home/user", "docs/file.txt")?;
/// let relative = File::get_relative_path("/home/user/docs/file.txt", "/home/user")?;
/// ```
///
/// ## Move files
///
/// ```rust
/// use crate::executors::utils::File;
///
/// File::move_path("/tmp/source.txt", "/tmp/dest.txt", true)?;
/// ```
///
/// ## Create temporary directory
///
/// ```rust
/// use crate::executors::utils::File;
///
/// let temp_dir = File::create_temp_dir("myapp_")?;
/// ```
use anyhow::Result;
use std::fs;
use std::fs::File as StdFile;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Validate and sanitize file path to prevent directory traversal attacks
///
/// # Arguments
/// * `path` - The path to validate
/// * `base_dir` - Optional base directory to restrict access to
///
/// # Returns
/// * `Result<PathBuf>` - The validated and sanitized path
///
/// # Errors
/// Returns an error if:
/// - Path contains directory traversal sequences (`..`)
/// - Path is outside the base directory (if base_dir is provided)
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
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `bool` - True if the path exists and is a file
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists() && Path::new(path).is_file()
}

/// Check if a directory exists
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `bool` - True if the path exists and is a directory
pub fn dir_exists(path: &str) -> bool {
    Path::new(path).exists() && Path::new(path).is_dir()
}

/// Ensure directory exists, create if not
///
/// # Arguments
/// * `path` - Directory path to ensure
///
/// # Returns
/// * `Result<()>` - Ok if directory exists or was created successfully
pub fn ensure_dir(path: &str) -> Result<()> {
    let dir = Path::new(path);
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Get file extension
///
/// # Arguments
/// * `path` - File path
///
/// # Returns
/// * `Option<String>` - The file extension in lowercase, or None if no extension
pub fn get_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

/// Read file content as string
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// * `Result<String>` - The file content as a string
pub fn read_file_content(path: &str) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

/// Write content to file
///
/// # Arguments
/// * `path` - Path to the file to write
/// * `content` - Content to write
/// * `append` - If true, append to the file; if false, overwrite
///
/// # Returns
/// * `Result<()>` - Ok if write was successful
pub fn write_file_content(path: &str, content: &str, append: bool) -> Result<()> {
    let path = Path::new(path);

    // Ensure parent directory exists
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

/// Get file metadata
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// * `Result<fs::Metadata>` - The file metadata
pub fn get_file_metadata(path: &str) -> Result<fs::Metadata> {
    let metadata = fs::metadata(path)?;
    Ok(metadata)
}

/// Copy file or directory
///
/// # Arguments
/// * `source` - Source path
/// * `destination` - Destination path
/// * `recursive` - If true and source is a directory, copy recursively
///
/// # Returns
/// * `Result<u64>` - Total number of bytes copied
pub fn copy(source: &str, destination: &str, recursive: bool) -> Result<u64> {
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);

    if source_path.is_dir() {
        if !recursive {
            anyhow::bail!("Cannot copy directory without recursive flag");
        }
        copy_directory(source_path, dest_path)
    } else {
        copy_file(source_path, dest_path)
    }
}

/// Copy a single file
///
/// # Arguments
/// * `source` - Source file path
/// * `destination` - Destination file path
///
/// # Returns
/// * `Result<u64>` - Number of bytes copied
fn copy_file(source: &Path, destination: &Path) -> Result<u64> {
    // Ensure destination directory exists
    if let Some(parent) = destination.parent() {
        ensure_dir(parent.to_str().unwrap())?;
    }

    let size = fs::copy(source, destination)?;
    Ok(size)
}

/// Copy a directory recursively
///
/// # Arguments
/// * `source` - Source directory path
/// * `destination` - Destination directory path
///
/// # Returns
/// * `Result<u64>` - Total number of bytes copied
fn copy_directory(source: &Path, destination: &Path) -> Result<u64> {
    if !source.exists() {
        anyhow::bail!("Source directory does not exist: {}", source.display());
    }

    if !source.is_dir() {
        anyhow::bail!("Source is not a directory: {}", source.display());
    }

    // Create destination directory
    ensure_dir(destination.to_str().unwrap())?;

    let mut total_size = 0;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if file_type.is_dir() {
            total_size += copy_directory(&source_path, &dest_path)?;
        } else {
            total_size += copy_file(&source_path, &dest_path)?;
        }
    }

    Ok(total_size)
}

/// Delete file or directory
///
/// # Arguments
/// * `path` - Path to delete
/// * `recursive` - If true and path is a directory, delete recursively
///
/// # Returns
/// * `Result<()>` - Ok if deletion was successful
pub fn delete(path: &str, recursive: bool) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        if recursive {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_dir(path)?;
        }
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

/// List directory contents
///
/// # Arguments
/// * `path` - Directory path to list
/// * `recursive` - If true, list all files recursively
///
/// # Returns
/// * `Result<Vec<PathBuf>>` - List of paths in the directory
pub fn list_directory(path: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    let path = Path::new(path);

    if !path.exists() || !path.is_dir() {
        anyhow::bail!("Directory does not exist: {}", path.display());
    }

    let mut entries = Vec::new();

    if recursive {
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            entries.push(entry.path().to_path_buf());
        }
    } else {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            entries.push(entry.path());
        }
    }

    Ok(entries)
}

/// Get file size in bytes
///
/// # Arguments
/// * `path` - File path
///
/// # Returns
/// * `Result<u64>` - File size in bytes
pub fn get_file_size(path: &str) -> Result<u64> {
    let metadata = get_file_metadata(path)?;
    Ok(metadata.len())
}

/// Get directory size (sum of all files recursively)
///
/// # Arguments
/// * `path` - Directory path
///
/// # Returns
/// * `Result<u64>` - Total size in bytes
pub fn get_directory_size(path: &str) -> Result<u64> {
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    if path.is_file() {
        return get_file_size(path.to_str().unwrap());
    }

    let mut total_size = 0;

    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
    }

    Ok(total_size)
}

/// Read file content as bytes
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// * `Result<Vec<u8>>` - File content as byte vector
pub fn read_file_bytes(path: &str) -> Result<Vec<u8>> {
    let mut file = StdFile::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Write byte array to file
///
/// # Arguments
/// * `path` - Path to the file to write
/// * `data` - Byte array to write
/// * `append` - If true, append to the file; if false, overwrite
///
/// # Returns
/// * `Result<()>` - Ok if write was successful
pub fn write_file_bytes(path: &str, data: &[u8], append: bool) -> Result<()> {
    let path = Path::new(path);

    // Ensure parent directory exists
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

/// Check if path is a directory
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `bool` - True if path exists and is a directory
pub fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
}

/// Check if path is a file
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `bool` - True if path exists and is a file
pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

/// Get file name (without path)
///
/// # Arguments
/// * `path` - File path
///
/// # Returns
/// * `Option<String>` - File name or None
pub fn get_file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

/// Get file stem (name without extension)
///
/// # Arguments
/// * `path` - File path
///
/// # Returns
/// * `Option<String>` - File stem or None
pub fn get_file_stem(path: &str) -> Option<String> {
    Path::new(path)
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

/// Normalize path (resolve . and .. components)
///
/// # Arguments
/// * `path` - Path to normalize
///
/// # Returns
/// * `Result<PathBuf>` - Normalized path
pub fn normalize_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(c) => {
                components.push(c);
            }
            _ => {}
        }
    }

    let mut result = PathBuf::new();
    for component in components {
        result.push(component);
    }

    Ok(result)
}

/// Safe path join (prevents path traversal)
///
/// # Arguments
/// * `base` - Base directory
/// * `sub` - Sub path to join
///
/// # Returns
/// * `Result<PathBuf>` - Joined and validated path
pub fn join_safe(base: &str, sub: &str) -> Result<PathBuf> {
    let base_path = Path::new(base);
    let sub_path = Path::new(sub);

    // Check if sub contains path traversal
    if sub_path
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        anyhow::bail!("Path traversal not allowed: {}", sub);
    }

    let full_path = base_path.join(sub_path);

    // Normalize and check if still under base directory
    let canonicalized = fs::canonicalize(&full_path)
        .or_else(|_| Ok::<PathBuf, anyhow::Error>(full_path.clone()))?;

    let base_canonical = fs::canonicalize(base_path)
        .or_else(|_| Ok::<PathBuf, anyhow::Error>(base_path.to_path_buf()))?;

    if !canonicalized.starts_with(base_canonical) {
        anyhow::bail!("Path escapes base directory: {}", sub);
    }

    Ok(canonicalized)
}

/// Move file or directory
///
/// # Arguments
/// * `source` - Source path
/// * `destination` - Destination path
/// * `overwrite` - If true, overwrite existing destination
///
/// # Returns
/// * `Result<()>` - Ok if move was successful
pub fn move_path(source: &str, destination: &str, overwrite: bool) -> Result<()> {
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);

    if !source_path.exists() {
        anyhow::bail!("Source does not exist: {}", source);
    }

    // If destination exists and overwrite is not allowed, return error
    if dest_path.exists() && !overwrite {
        anyhow::bail!("Destination already exists: {}", destination);
    }

    // Ensure destination parent directory exists
    if let Some(parent) = dest_path.parent() {
        ensure_dir(parent.to_str().unwrap())?;
    }

    // If destination exists and overwrite is allowed, delete it first
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

/// Create a temporary directory
///
/// # Arguments
/// * `prefix` - Prefix for the temporary directory name
///
/// # Returns
/// * `Result<PathBuf>` - Path to the created temporary directory
pub fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join(prefix);

    // Add random suffix to avoid conflicts
    let suffix: String = (0..8)
        .map(|_| format!("{:x}", rand::random::<u8>()))
        .collect();
    let temp_dir = temp_dir.with_file_name(format!("{}_{}", prefix, suffix));

    ensure_dir(temp_dir.to_str().unwrap())?;
    Ok(temp_dir)
}

/// Get relative path from base directory
///
/// # Arguments
/// * `path` - Full path
/// * `base` - Base directory
///
/// # Returns
/// * `Result<String>` - Relative path as string
pub fn get_relative_path(path: &str, base: &str) -> Result<String> {
    let path = Path::new(path);
    let base = Path::new(base);

    let relative = path
        .strip_prefix(base)
        .map_err(|_| anyhow::anyhow!("Path is not under base directory"))?;

    Ok(relative.to_string_lossy().to_string())
}

/// Check if two paths are the same file
///
/// # Arguments
/// * `path1` - First path
/// * `path2` - Second path
///
/// # Returns
/// * `Result<bool>` - True if paths point to the same file
pub fn is_same_file(path1: &str, path2: &str) -> Result<bool> {
    let path1 = Path::new(path1);
    let path2 = Path::new(path2);

    if !path1.exists() || !path2.exists() {
        return Ok(false);
    }

    let metadata1 = fs::metadata(path1)?;
    let metadata2 = fs::metadata(path2)?;

    // Compare file IDs (Unix) or use canonicalization
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(metadata1.ino() == metadata2.ino() && metadata1.dev() == metadata2.dev())
    }

    #[cfg(not(unix))]
    {
        let canonical1 = fs::canonicalize(path1)?;
        let canonical2 = fs::canonicalize(path2)?;
        Ok(canonical1 == canonical2)
    }
}

/// Get the absolute path
///
/// # Arguments
/// * `path` - Path to make absolute
///
/// # Returns
/// * `Result<PathBuf>` - Absolute path
pub fn absolute_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    // Try to canonicalize, but fall back to normalized path
    Ok(fs::canonicalize(&absolute).unwrap_or_else(|_| absolute))
}

/// Create an empty file and all parent directories
///
/// # Arguments
/// * `path` - Path to the file to create
///
/// # Returns
/// * `Result<()>` - Ok if file was created successfully
pub fn touch(path: &str) -> Result<()> {
    let path = Path::new(path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir(parent.to_str().unwrap())?;
    }

    // Create or update file timestamp
    if path.exists() {
        let file = StdFile::open(path)?;
        file.set_modified(std::time::SystemTime::now())?;
    } else {
        StdFile::create(path)?;
    }

    Ok(())
}

/// Read file line by line
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// * `Result<Vec<String>>` - Lines of the file as strings
pub fn read_lines(path: &str) -> Result<Vec<String>> {
    let content = read_file_content(path)?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

/// Append line to file
///
/// # Arguments
/// * `path` - Path to the file
/// * `line` - Line to append
///
/// # Returns
/// * `Result<()>` - Ok if line was appended
pub fn append_line(path: &str, line: &str) -> Result<()> {
    write_file_content(path, &format!("{}\n", line), true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_path() {
        let result = validate_path("safe/path.txt", None);
        assert!(result.is_ok());

        let result = validate_path("../unsafe.txt", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path traversal"));
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        assert!(!file_exists(test_file.to_str().unwrap()));

        fs::write(&test_file, "content").unwrap();
        assert!(file_exists(test_file.to_str().unwrap()));
    }

    #[test]
    fn test_ensure_dir() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path().join("nested/deep/dir");

        assert!(!test_dir.exists());
        ensure_dir(test_dir.to_str().unwrap()).unwrap();
        assert!(test_dir.exists());
    }

    #[test]
    fn test_read_write_file() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = "Hello, World!";

        write_file_content(test_file.to_str().unwrap(), content, false).unwrap();
        let read_content = read_file_content(test_file.to_str().unwrap()).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_copy_file() {
        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&source, "content").unwrap();

        copy(source.to_str().unwrap(), dest.to_str().unwrap(), false).unwrap();
        assert!(dest.exists());
    }

    #[test]
    fn test_copy_directory() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("file1.txt"), "content1").unwrap();
        fs::write(source_dir.join("file2.txt"), "content2").unwrap();

        copy(
            source_dir.to_str().unwrap(),
            dest_dir.to_str().unwrap(),
            true,
        )
        .unwrap();
        assert!(dest_dir.exists());
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("file2.txt").exists());
    }

    #[test]
    fn test_delete() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        fs::write(&test_file, "content").unwrap();
        assert!(test_file.exists());

        delete(test_file.to_str().unwrap(), false).unwrap();
        assert!(!test_file.exists());
    }

    #[test]
    fn test_list_directory() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();

        let entries = list_directory(temp_dir.path().to_str().unwrap(), false).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_get_file_size() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = "Hello";

        fs::write(&test_file, content).unwrap();
        let size = get_file_size(test_file.to_str().unwrap()).unwrap();
        assert_eq!(size, content.len() as u64);
    }

    #[test]
    fn test_normalize_path() {
        let normalized = normalize_path("/home/user/../user/docs/./file.txt").unwrap();
        assert_eq!(normalized.to_str().unwrap(), "/home/user/docs/file.txt");
    }

    #[test]
    fn test_join_safe() {
        let result = join_safe("/home/user", "docs/file.txt");
        assert!(result.is_ok());

        let result = join_safe("/home/user", "../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_move_path() {
        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&source, "content").unwrap();

        move_path(source.to_str().unwrap(), dest.to_str().unwrap(), false).unwrap();
        assert!(!source.exists());
        assert!(dest.exists());
    }

    #[test]
    fn test_get_relative_path() {
        let relative = get_relative_path("/home/user/docs/file.txt", "/home/user").unwrap();
        assert_eq!(relative, "docs/file.txt");
    }

    #[test]
    fn test_touch() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        touch(test_file.to_str().unwrap()).unwrap();
        assert!(test_file.exists());
    }

    #[test]
    fn test_read_lines() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        fs::write(&test_file, "line1\nline2\nline3").unwrap();
        let lines = read_lines(test_file.to_str().unwrap()).unwrap();
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_append_line() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        write_file_content(test_file.to_str().unwrap(), "line1\n", false).unwrap();
        append_line(test_file.to_str().unwrap(), "line2").unwrap();

        let content = read_file_content(test_file.to_str().unwrap()).unwrap();
        assert_eq!(content, "line1\nline2\n");
    }
}
