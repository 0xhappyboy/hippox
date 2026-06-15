//! Mathematics skills registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Math;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "math", feature = "all"))]
    {
        use crate::skills::*;
        use crate::skills::math::*;
        
        map.insert("math_calculator".to_string(), Arc::new(CalculatorSkill));
        map.insert("math_power".to_string(), Arc::new(PowerSkill));
        map.insert("math_statistics".to_string(), Arc::new(StatisticsSkill));
        map.insert("math_unit_converter".to_string(), Arc::new(UnitConverterSkill));
        map.insert("hash_md5".to_string(), Arc::new(HashMd5Skill));
        map.insert("hash_sha256".to_string(), Arc::new(HashSha256Skill));
        map.insert("hash_sha512".to_string(), Arc::new(HashSha512Skill));
        map.insert("hash_file".to_string(), Arc::new(HashFileSkill));
        map.insert("base64_encode".to_string(), Arc::new(Base64EncodeSkill));
        map.insert("base64_decode".to_string(), Arc::new(Base64DecodeSkill));
        map.insert("random_number".to_string(), Arc::new(RandomNumberSkill));
        map.insert("random_string".to_string(), Arc::new(RandomStringSkill));
        map.insert("random_uuid".to_string(), Arc::new(RandomUuidSkill));
        map.insert("random_password".to_string(), Arc::new(RandomPasswordSkill));
    }
}