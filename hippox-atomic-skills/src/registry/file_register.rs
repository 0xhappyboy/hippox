// file_register.rs
//! File system skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::File;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "file", feature = "all"))]
    {
        use crate::skills::file::*;

        // Core file operations
        map.insert("file_read".to_string(), Arc::new(ReadFileSkill));
        map.insert("file_write".to_string(), Arc::new(WriteFileSkill));
        map.insert("file_delete".to_string(), Arc::new(DeleteFileSkill));
        map.insert("file_list".to_string(), Arc::new(ListDirectorySkill));
        map.insert("file_copy".to_string(), Arc::new(CopyFileSkill));

        // Hash operations
        map.insert("hash_md5".to_string(), Arc::new(HashMd5Skill));
        map.insert("hash_sha1".to_string(), Arc::new(HashSha1Skill));
        map.insert("hash_sha256".to_string(), Arc::new(HashSha256Skill));
        map.insert("hash_sha512".to_string(), Arc::new(HashSha512Skill));
        map.insert("hash_file".to_string(), Arc::new(HashFileSkill));

        // Archive operations (from archive.rs)
        map.insert(
            "archive_zip_create".to_string(),
            Arc::new(ArchiveZipCreateSkill),
        );
        map.insert(
            "archive_zip_extract".to_string(),
            Arc::new(ArchiveZipExtractSkill),
        );
        map.insert(
            "archive_tar_create".to_string(),
            Arc::new(ArchiveTarCreateSkill),
        );
        map.insert(
            "archive_tar_extract".to_string(),
            Arc::new(ArchiveTarExtractSkill),
        );
        map.insert(
            "archive_compress".to_string(),
            Arc::new(ArchiveCompressSkill),
        );

        // Security/Forensic operations (new)
        map.insert(
            "file_signature_verify".to_string(),
            Arc::new(FileSignatureSkill),
        );
        map.insert(
            "file_integrity_monitor".to_string(),
            Arc::new(FileIntegrityMonitorSkill),
        );
        map.insert("file_virus_scan".to_string(), Arc::new(VirusScanSkill));
        map.insert(
            "disk_forensic_analyze".to_string(),
            Arc::new(DiskForensicSkill),
        );
        map.insert("log_pack".to_string(), Arc::new(LogPackSkill));
    }
}
