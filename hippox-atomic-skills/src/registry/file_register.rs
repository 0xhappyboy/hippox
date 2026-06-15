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

        map.insert("file_read".to_string(), Arc::new(ReadFileSkill));
        map.insert("file_write".to_string(), Arc::new(WriteFileSkill));
        map.insert("file_delete".to_string(), Arc::new(DeleteFileSkill));
        map.insert("file_list".to_string(), Arc::new(ListDirectorySkill));
        map.insert("file_copy".to_string(), Arc::new(CopyFileSkill));
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
    }
}
