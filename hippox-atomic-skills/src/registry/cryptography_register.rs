//! Cryptographic skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Cryptography;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "cryptography", feature = "all"))]
    {
        use crate::skills::cryptography::*;

        // Base64 operations
        map.insert("base64_encode".to_string(), Arc::new(Base64EncodeSkill));
        map.insert("base64_decode".to_string(), Arc::new(Base64DecodeSkill));

        // Text hashing
        map.insert("hash_md5_text".to_string(), Arc::new(HashMd5TextSkill));
        map.insert(
            "hash_sha256_text".to_string(),
            Arc::new(HashSha256TextSkill),
        );
        map.insert(
            "hash_sha512_text".to_string(),
            Arc::new(HashSha512TextSkill),
        );
        map.insert("hash_hmac".to_string(), Arc::new(HashHmacSkill));

        // AES encryption
        map.insert("aes_encrypt".to_string(), Arc::new(AesEncryptSkill));
        map.insert("aes_decrypt".to_string(), Arc::new(AesDecryptSkill));

        // RSA operations
        map.insert("rsa_encrypt".to_string(), Arc::new(RsaEncryptSkill));
        map.insert("rsa_decrypt".to_string(), Arc::new(RsaDecryptSkill));
        map.insert("rsa_sign".to_string(), Arc::new(RsaSignSkill));
        map.insert("rsa_verify".to_string(), Arc::new(RsaVerifySkill));

        // Generation
        map.insert("generate_random".to_string(), Arc::new(GenerateRandomSkill));
        map.insert("generate_uuid".to_string(), Arc::new(GenerateUuidSkill));
        map.insert(
            "generate_password".to_string(),
            Arc::new(GeneratePasswordSkill),
        );

        // Password hashing
        map.insert("password_hash".to_string(), Arc::new(PasswordHashSkill));
        map.insert("password_verify".to_string(), Arc::new(PasswordVerifySkill));
    }
}
