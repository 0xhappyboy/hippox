//! File system drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::File;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "file", feature = "all"))]
    {
        use crate::drivers::file::*;

        // Core file operations
        map.insert("file_read".to_string(), Arc::new(ReadFileDriver));
        map.insert("file_write".to_string(), Arc::new(WriteFileDriver));
        map.insert("file_delete".to_string(), Arc::new(DeleteFileDriver));
        map.insert("file_list".to_string(), Arc::new(ListDirectoryDriver));
        map.insert("file_copy".to_string(), Arc::new(CopyFileDriver));

        // Hash operations
        map.insert("hash_md5".to_string(), Arc::new(HashMd5Driver));
        map.insert("hash_sha1".to_string(), Arc::new(HashSha1Driver));
        map.insert("hash_sha256".to_string(), Arc::new(HashSha256Driver));
        map.insert("hash_sha512".to_string(), Arc::new(HashSha512Driver));
        map.insert("hash_file".to_string(), Arc::new(HashFileDriver));

        // Archive operations (from archive.rs)
        map.insert(
            "archive_zip_create".to_string(),
            Arc::new(ArchiveZipCreateDriver),
        );
        map.insert(
            "archive_zip_extract".to_string(),
            Arc::new(ArchiveZipExtractDriver),
        );
        map.insert(
            "archive_tar_create".to_string(),
            Arc::new(ArchiveTarCreateDriver),
        );
        map.insert(
            "archive_tar_extract".to_string(),
            Arc::new(ArchiveTarExtractDriver),
        );
        map.insert(
            "archive_compress".to_string(),
            Arc::new(ArchiveCompressDriver),
        );

        // Security/Forensic operations (new)
        map.insert(
            "file_signature_verify".to_string(),
            Arc::new(FileSignatureDriver),
        );
        map.insert(
            "file_integrity_monitor".to_string(),
            Arc::new(FileIntegrityMonitorDriver),
        );
        map.insert("file_virus_scan".to_string(), Arc::new(VirusScanDriver));
        map.insert(
            "disk_forensic_analyze".to_string(),
            Arc::new(DiskForensicDriver),
        );
        map.insert("log_pack".to_string(), Arc::new(LogPackDriver));
    }
}
