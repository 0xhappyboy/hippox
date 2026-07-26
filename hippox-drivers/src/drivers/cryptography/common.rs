//! Shared utilities for cryptographic operations
use crate::DriverError;
use crate::result::DriverResult;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
/// Symmetric encryption algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SymmetricAlgorithm {
    Aes128Cbc,
    Aes256Cbc,
    Aes128Gcm,
    Aes256Gcm,
}
/// Asymmetric encryption algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AsymmetricAlgorithm {
    Rsa2048,
    Rsa3072,
    Rsa4096,
}
/// Password hashing algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PasswordHashAlgorithm {
    Bcrypt,
    Argon2id,
}
/// HMAC algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HmacAlgorithm {
    Sha256,
    Sha512,
}
/// Generate random bytes using getrandom (cryptographically secure)
pub fn generate_random_bytes(len: usize) -> DriverResult<Vec<u8>> {
    debug!("Generating {} random bytes", len);
    let mut bytes = vec![0u8; len];
    match getrandom::fill(&mut bytes) {
        Ok(_) => {
            info!("Generated {} random bytes", len);
            return Ok(bytes);
        }
        Err(e) => {
            let err_msg = format!("Failed to generate random bytes: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    }
}
/// Generate random hex string
pub fn generate_random_hex(len: usize) -> DriverResult<String> {
    debug!("Generating random hex string of length {}", len);
    let bytes = generate_random_bytes(len)?;
    let result = hex::encode(bytes);
    info!("Generated random hex string of length {}", len);
    return Ok(result);
}
/// Generate cryptographically secure random string
pub fn generate_random_string(length: usize) -> DriverResult<String> {
    debug!("Generating random string of length {}", length);
    let bytes = generate_random_bytes(length)?;
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let result: String = bytes.iter().map(|b| charset[(b % 62) as usize] as char).collect();
    info!("Generated random string of length {}", length);
    return Ok(result);
}
/// Generate salt for password hashing
pub fn generate_salt(len: usize) -> DriverResult<Vec<u8>> {
    debug!("Generating salt of length {}", len);
    return generate_random_bytes(len);
}
/// Derive key from password (PBKDF2)
pub fn derive_key(password: &str, salt: &[u8], iterations: u32, key_len: usize) -> DriverResult<Vec<u8>> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    debug!("Deriving key with iterations: {}, key_len: {}", iterations, key_len);
    let mut key = vec![0u8; key_len];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
    info!("Key derived successfully");
    return Ok(key);
}
/// Encode to Base64
pub fn to_base64(data: &[u8]) -> String {
    debug!("Encoding {} bytes to Base64", data.len());
    let result = STANDARD.encode(data);
    info!("Base64 encoding completed, output length: {}", result.len());
    return result;
}
/// Decode from Base64
pub fn from_base64(data: &str) -> DriverResult<Vec<u8>> {
    debug!("Decoding Base64 string of length {}", data.len());
    match STANDARD.decode(data) {
        Ok(result) => {
            info!("Base64 decoded successfully, output length: {}", result.len());
            return Ok(result);
        }
        Err(e) => {
            let err_msg = format!("Failed to decode Base64: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("base64", err_msg));
        }
    }
}
/// Encode to hex
pub fn to_hex(data: &[u8]) -> String {
    debug!("Encoding {} bytes to hex", data.len());
    let result = hex::encode(data);
    info!("Hex encoding completed, output length: {}", result.len());
    return result;
}
/// Decode from hex
pub fn from_hex(data: &str) -> DriverResult<Vec<u8>> {
    debug!("Decoding hex string of length {}", data.len());
    match hex::decode(data) {
        Ok(result) => {
            info!("Hex decoded successfully, output length: {}", result.len());
            return Ok(result);
        }
        Err(e) => {
            let err_msg = format!("Failed to decode hex: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("hex", err_msg));
        }
    }
}
/// Validate password strength
pub fn validate_password_strength(password: &str) -> DriverResult<()> {
    debug!("Validating password strength");
    if password.len() < 8 {
        let err_msg = "Password must be at least 8 characters long".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::validation("password", err_msg));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        let err_msg = "Password must contain at least one uppercase letter".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::validation("password", err_msg));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        let err_msg = "Password must contain at least one lowercase letter".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::validation("password", err_msg));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        let err_msg = "Password must contain at least one digit".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::validation("password", err_msg));
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        let err_msg = "Password must contain at least one special character".to_string();
        warn!("{}", err_msg);
        return Err(DriverError::validation("password", err_msg));
    }
    info!("Password strength validation passed");
    return Ok(());
}
/// AES encrypt with GCM mode (authenticated encryption)
pub fn aes_gcm_encrypt(key: &[u8], plaintext: &[u8], associated_data: Option<&[u8]>) -> DriverResult<(Vec<u8>, Vec<u8>)> {
    use aes_gcm::aead::{Aead, Payload};
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    debug!("AES GCM encrypt: plaintext_len={}, key_len={}", plaintext.len(), key.len());
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce_bytes = generate_random_bytes(12)?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let payload = if let Some(ad) = associated_data { Payload { msg: plaintext, aad: ad } } else { Payload { msg: plaintext, aad: &[] } };
    match cipher.encrypt(nonce, payload) {
        Ok(ciphertext) => {
            info!("AES GCM encryption successful");
            return Ok((nonce_bytes, ciphertext));
        }
        Err(e) => {
            let err_msg = format!("Encryption failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// AES decrypt with GCM mode
pub fn aes_gcm_decrypt(key: &[u8], nonce: &[u8], ciphertext: &[u8], associated_data: Option<&[u8]>) -> DriverResult<Vec<u8>> {
    use aes_gcm::aead::{Aead, Payload};
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    debug!("AES GCM decrypt: ciphertext_len={}, key_len={}", ciphertext.len(), key.len());
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    let payload = if let Some(ad) = associated_data { Payload { msg: ciphertext, aad: ad } } else { Payload { msg: ciphertext, aad: &[] } };
    match cipher.decrypt(nonce, payload) {
        Ok(plaintext) => {
            info!("AES GCM decryption successful, output length: {}", plaintext.len());
            return Ok(plaintext);
        }
        Err(e) => {
            let err_msg = format!("Decryption failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// AES encrypt with CBC mode
pub fn aes_cbc_encrypt(key: &[u8], plaintext: &[u8]) -> DriverResult<(Vec<u8>, Vec<u8>)> {
    use aes::cipher::{BlockModeEncrypt, KeyInit, KeyIvInit};
    use cbc::Encryptor;
    use cbc::cipher::block_padding::Pkcs7;
    debug!("AES CBC encrypt: plaintext_len={}, key_len={}", plaintext.len(), key.len());
    type Aes256CbcEnc = Encryptor<aes::Aes256>;
    let iv = generate_random_bytes(16)?;
    let cipher = match Aes256CbcEnc::new_from_slices(key, &iv) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Invalid key/IV: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("key", err_msg));
        }
    };
    let mut buffer = plaintext.to_vec();
    match cipher.encrypt_padded::<Pkcs7>(&mut buffer, plaintext.len()) {
        Ok(ciphertext) => {
            info!("AES CBC encryption successful");
            return Ok((iv, ciphertext.to_vec()));
        }
        Err(e) => {
            let err_msg = format!("Encryption failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// AES decrypt with CBC mode
pub fn aes_cbc_decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> DriverResult<Vec<u8>> {
    use aes::cipher::{BlockModeDecrypt, KeyInit, KeyIvInit};
    use cbc::Decryptor;
    use cbc::cipher::block_padding::Pkcs7;
    debug!("AES CBC decrypt: ciphertext_len={}, key_len={}", ciphertext.len(), key.len());
    type Aes256CbcDec = Decryptor<aes::Aes256>;
    let cipher = match Aes256CbcDec::new_from_slices(key, iv) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Invalid key/IV: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("key", err_msg));
        }
    };
    let mut buffer = ciphertext.to_vec();
    match cipher.decrypt_padded::<Pkcs7>(&mut buffer) {
        Ok(plaintext) => {
            info!("AES CBC decryption successful, output length: {}", plaintext.len());
            return Ok(plaintext.to_vec());
        }
        Err(e) => {
            let err_msg = format!("Decryption failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// Generate RSA key pair
pub fn generate_rsa_key_pair(bits: u32) -> DriverResult<(String, String)> {
    use rand_core::OsRng;
    use rsa::{
        RsaPrivateKey, RsaPublicKey,
        pkcs8::{EncodePrivateKey, EncodePublicKey},
    };
    debug!("Generating RSA key pair with {} bits", bits);
    let mut rng = OsRng;
    let private_key = match RsaPrivateKey::new(&mut rng, bits as usize) {
        Ok(k) => k,
        Err(e) => {
            let err_msg = format!("Failed to generate RSA key pair: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
    let public_key = RsaPublicKey::from(&private_key);
    let private_pem = match private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF) {
        Ok(p) => p,
        Err(e) => {
            let err_msg = format!("Failed to encode private key: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
    let public_pem = match public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF) {
        Ok(p) => p,
        Err(e) => {
            let err_msg = format!("Failed to encode public key: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::internal(err_msg));
        }
    };
    info!("RSA key pair generated with {} bits", bits);
    return Ok((private_pem.to_string(), public_pem.to_string()));
}
/// RSA encryption with public key
pub fn rsa_encrypt(public_key_pem: &str, data: &[u8]) -> DriverResult<Vec<u8>> {
    use rand_core::OsRng;
    use rsa::pkcs1v15::Pkcs1v15Encrypt;
    use rsa::traits::PaddingScheme;
    use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};
    debug!("RSA encrypt: data_len={}", data.len());
    let public_key = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(k) => k,
        Err(e) => {
            let err_msg = format!("Invalid public key: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("public_key", err_msg));
        }
    };
    let mut rng = OsRng;
    let padding = Pkcs1v15Encrypt;
    match padding.encrypt(&mut rng, &public_key, data) {
        Ok(encrypted) => {
            info!("RSA encryption successful, output length: {}", encrypted.len());
            return Ok(encrypted);
        }
        Err(e) => {
            let err_msg = format!("Encryption failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// RSA decryption with private key
pub fn rsa_decrypt(private_key_pem: &str, data: &[u8]) -> DriverResult<Vec<u8>> {
    use rand_core::OsRng;
    use rsa::pkcs1v15::Pkcs1v15Encrypt;
    use rsa::traits::PaddingScheme;
    use rsa::{RsaPrivateKey, pkcs8::DecodePrivateKey};
    debug!("RSA decrypt: data_len={}", data.len());
    let private_key = match RsaPrivateKey::from_pkcs8_pem(private_key_pem) {
        Ok(k) => k,
        Err(e) => {
            let err_msg = format!("Invalid private key: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("private_key", err_msg));
        }
    };
    let padding = Pkcs1v15Encrypt;
    match padding.decrypt(Some(&mut OsRng), &private_key, data) {
        Ok(decrypted) => {
            info!("RSA decryption successful, output length: {}", decrypted.len());
            return Ok(decrypted);
        }
        Err(e) => {
            let err_msg = format!("Decryption failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// RSA signature with private key
pub fn rsa_sign(private_key_pem: &str, data: &[u8]) -> DriverResult<Vec<u8>> {
    use ring::rand::SystemRandom;
    use ring::signature::{RSA_PKCS1_SHA256, RsaKeyPair};
    debug!("RSA sign: data_len={}", data.len());
    let pem = match pem::parse(private_key_pem) {
        Ok(p) => p,
        Err(e) => {
            let err_msg = format!("Invalid PEM: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("private_key", err_msg));
        }
    };
    let key_pair = match RsaKeyPair::from_pkcs8(pem.contents()) {
        Ok(k) => k,
        Err(e) => {
            let err_msg = format!("Invalid private key: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("private_key", err_msg));
        }
    };
    let rng = SystemRandom::new();
    let mut signature = vec![0; key_pair.public_modulus_len()];
    match key_pair.sign(&RSA_PKCS1_SHA256, &rng, data, &mut signature) {
        Ok(_) => {
            info!("RSA signature successful, output length: {}", signature.len());
            return Ok(signature);
        }
        Err(e) => {
            let err_msg = format!("Sign failed: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// RSA signature verification with public key
pub fn rsa_verify(public_key_pem: &str, data: &[u8], signature: &[u8]) -> DriverResult<bool> {
    use ring::signature::{RSA_PKCS1_2048_8192_SHA256, UnparsedPublicKey};
    debug!("RSA verify: data_len={}, signature_len={}", data.len(), signature.len());
    let pem = match pem::parse(public_key_pem) {
        Ok(p) => p,
        Err(e) => {
            let err_msg = format!("Invalid PEM: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("public_key", err_msg));
        }
    };
    let public_key = UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, pem.contents());
    match public_key.verify(data, signature) {
        Ok(()) => {
            info!("RSA signature verification successful");
            return Ok(true);
        }
        Err(_) => {
            info!("RSA signature verification failed");
            return Ok(false);
        }
    }
}
/// Hash password with bcrypt
pub fn bcrypt_hash(password: &str, cost: u32) -> DriverResult<String> {
    debug!("Bcrypt hash: password_len={}, cost={}", password.len(), cost);
    match bcrypt::hash(password, cost) {
        Ok(hashed) => {
            info!("Bcrypt hash successful");
            return Ok(hashed);
        }
        Err(e) => {
            let err_msg = format!("Failed to hash password: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// Verify password with bcrypt
pub fn bcrypt_verify(password: &str, hash: &str) -> DriverResult<bool> {
    debug!("Bcrypt verify");
    match bcrypt::verify(password, hash) {
        Ok(result) => {
            info!("Bcrypt verification completed: {}", result);
            return Ok(result);
        }
        Err(e) => {
            let err_msg = format!("Failed to verify password: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// Hash password with Argon2id
pub fn argon2_hash(password: &str) -> DriverResult<String> {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand_core::OsRng;
    debug!("Argon2 hash: password_len={}", password.len());
    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => {
            info!("Argon2 hash successful");
            return Ok(hash.to_string());
        }
        Err(e) => {
            let err_msg = format!("Failed to hash password: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::execution(err_msg));
        }
    }
}
/// Verify password with Argon2id
pub fn argon2_verify(password: &str, hash: &str) -> DriverResult<bool> {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    debug!("Argon2 verify");
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(e) => {
            let err_msg = format!("Invalid password hash: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("hash", err_msg));
        }
    };
    let argon2 = Argon2::default();
    let result = argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok();
    info!("Argon2 verification completed: {}", result);
    return Ok(result);
}
/// Generate HMAC SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> DriverResult<Vec<u8>> {
    use hmac::KeyInit;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    debug!("HMAC SHA256: key_len={}, data_len={}", key.len(), data.len());
    let mut mac = match Hmac::<Sha256>::new_from_slice(key) {
        Ok(m) => m,
        Err(e) => {
            let err_msg = format!("Invalid HMAC key: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("key", err_msg));
        }
    };
    mac.update(data);
    let result = mac.finalize().into_bytes().to_vec();
    info!("HMAC SHA256 generated, output length: {}", result.len());
    return Ok(result);
}
/// Generate HMAC SHA512
pub fn hmac_sha512(key: &[u8], data: &[u8]) -> DriverResult<Vec<u8>> {
    use hmac::KeyInit;
    use hmac::{Hmac, Mac};
    use sha2::Sha512;
    debug!("HMAC SHA512: key_len={}, data_len={}", key.len(), data.len());
    let mut mac = match Hmac::<Sha512>::new_from_slice(key) {
        Ok(m) => m,
        Err(e) => {
            let err_msg = format!("Invalid HMAC key: {}", e);
            warn!("{}", err_msg);
            return Err(DriverError::validation("key", err_msg));
        }
    };
    mac.update(data);
    let result = mac.finalize().into_bytes().to_vec();
    info!("HMAC SHA512 generated, output length: {}", result.len());
    return Ok(result);
}
