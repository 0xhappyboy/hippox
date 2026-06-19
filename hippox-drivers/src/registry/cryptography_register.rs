//! Cryptographic drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Cryptography;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "cryptography", feature = "all"))]
    {
        use crate::drivers::cryptography::*;

        // Base64 operations
        map.insert("base64_encode".to_string(), Arc::new(Base64EncodeDriver));
        map.insert("base64_decode".to_string(), Arc::new(Base64DecodeDriver));

        // Text hashing
        map.insert("hash_md5_text".to_string(), Arc::new(HashMd5TextDriver));
        map.insert(
            "hash_sha256_text".to_string(),
            Arc::new(HashSha256TextDriver),
        );
        map.insert(
            "hash_sha512_text".to_string(),
            Arc::new(HashSha512TextDriver),
        );
        map.insert("hash_hmac".to_string(), Arc::new(HashHmacDriver));

        // AES encryption
        map.insert("aes_encrypt".to_string(), Arc::new(AesEncryptDriver));
        map.insert("aes_decrypt".to_string(), Arc::new(AesDecryptDriver));

        // RSA operations
        map.insert("rsa_encrypt".to_string(), Arc::new(RsaEncryptDriver));
        map.insert("rsa_decrypt".to_string(), Arc::new(RsaDecryptDriver));
        map.insert("rsa_sign".to_string(), Arc::new(RsaSignDriver));
        map.insert("rsa_verify".to_string(), Arc::new(RsaVerifyDriver));

        // Generation
        map.insert("generate_random".to_string(), Arc::new(GenerateRandomDriver));
        map.insert("generate_uuid".to_string(), Arc::new(GenerateUuidDriver));
        map.insert(
            "generate_password".to_string(),
            Arc::new(GeneratePasswordDriver),
        );

        // Password hashing
        map.insert("password_hash".to_string(), Arc::new(PasswordHashDriver));
        map.insert("password_verify".to_string(), Arc::new(PasswordVerifyDriver));
    }
}
