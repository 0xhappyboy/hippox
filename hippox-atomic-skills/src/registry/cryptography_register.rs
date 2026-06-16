//! Cryptographic skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Cryptography;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    #[cfg(any(feature = "cryptography", feature = "all"))]
    {
        use crate::skills::{
            Base64DecodeSkill, Base64EncodeSkill, HashFileSkill, HashMd5Skill, HashSha256Skill,
            HashSha512Skill,
        };

        map.insert("hash_md5".to_string(), Arc::new(HashMd5Skill));
        map.insert("hash_sha256".to_string(), Arc::new(HashSha256Skill));
        map.insert("hash_sha512".to_string(), Arc::new(HashSha512Skill));
        map.insert("hash_file".to_string(), Arc::new(HashFileSkill));
        map.insert("base64_encode".to_string(), Arc::new(Base64EncodeSkill));
        map.insert("base64_decode".to_string(), Arc::new(Base64DecodeSkill));
    }
}
