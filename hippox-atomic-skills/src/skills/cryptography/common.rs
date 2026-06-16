//! Shared utilities for cryptographic operations

use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

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
pub fn generate_random_bytes(len: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; len];
    getrandom::fill(&mut bytes)
        .map_err(|e| anyhow::anyhow!("Failed to generate random bytes: {}", e))?;
    Ok(bytes)
}

/// Generate random hex string
pub fn generate_random_hex(len: usize) -> Result<String> {
    let bytes = generate_random_bytes(len)?;
    Ok(hex::encode(bytes))
}

/// Generate cryptographically secure random string
pub fn generate_random_string(length: usize) -> Result<String> {
    let bytes = generate_random_bytes(length)?;
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let result: String = bytes
        .iter()
        .map(|b| charset[(b % 62) as usize] as char)
        .collect();
    Ok(result)
}

/// Generate salt for password hashing
pub fn generate_salt(len: usize) -> Result<Vec<u8>> {
    generate_random_bytes(len)
}

/// Derive key from password (PBKDF2)
pub fn derive_key(password: &str, salt: &[u8], iterations: u32, key_len: usize) -> Result<Vec<u8>> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    let mut key = vec![0u8; key_len];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
    Ok(key)
}

/// Encode to Base64
pub fn to_base64(data: &[u8]) -> String {
    STANDARD.encode(data)
}

/// Decode from Base64
pub fn from_base64(data: &str) -> Result<Vec<u8>> {
    Ok(STANDARD.decode(data)?)
}

/// Encode to hex
pub fn to_hex(data: &[u8]) -> String {
    hex::encode(data)
}

/// Decode from hex
pub fn from_hex(data: &str) -> Result<Vec<u8>> {
    Ok(hex::decode(data)?)
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<()> {
    if password.len() < 8 {
        anyhow::bail!("Password must be at least 8 characters long");
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        anyhow::bail!("Password must contain at least one uppercase letter");
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        anyhow::bail!("Password must contain at least one lowercase letter");
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        anyhow::bail!("Password must contain at least one digit");
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        anyhow::bail!("Password must contain at least one special character");
    }
    Ok(())
}

/// AES encrypt with GCM mode (authenticated encryption)
pub fn aes_gcm_encrypt(
    key: &[u8],
    plaintext: &[u8],
    associated_data: Option<&[u8]>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    use aes_gcm::aead::{Aead, Payload};
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce_bytes = generate_random_bytes(12)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = if let Some(ad) = associated_data {
        Payload {
            msg: plaintext,
            aad: ad,
        }
    } else {
        Payload {
            msg: plaintext,
            aad: &[],
        }
    };

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    Ok((nonce_bytes, ciphertext))
}

/// AES decrypt with GCM mode
pub fn aes_gcm_decrypt(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    associated_data: Option<&[u8]>,
) -> Result<Vec<u8>> {
    use aes_gcm::aead::{Aead, Payload};
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    let payload = if let Some(ad) = associated_data {
        Payload {
            msg: ciphertext,
            aad: ad,
        }
    } else {
        Payload {
            msg: ciphertext,
            aad: &[],
        }
    };

    let plaintext = cipher
        .decrypt(nonce, payload)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

/// AES encrypt with CBC mode
pub fn aes_cbc_encrypt(key: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    use aes::cipher::{BlockModeEncrypt, KeyInit, KeyIvInit};
    use cbc::Encryptor;
    use cbc::cipher::block_padding::Pkcs7;

    type Aes256CbcEnc = Encryptor<aes::Aes256>;
    let iv = generate_random_bytes(16)?;
    let cipher = Aes256CbcEnc::new_from_slices(key, &iv)
        .map_err(|e| anyhow::anyhow!("Invalid key/IV: {}", e))?;
    let mut buffer = plaintext.to_vec();
    let ciphertext = cipher
        .encrypt_padded::<Pkcs7>(&mut buffer, plaintext.len())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
    Ok((iv, ciphertext.to_vec()))
}

/// AES decrypt with CBC mode
pub fn aes_cbc_decrypt(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    use aes::cipher::{BlockModeDecrypt, KeyInit, KeyIvInit};
    use cbc::Decryptor;
    use cbc::cipher::block_padding::Pkcs7;

    type Aes256CbcDec = Decryptor<aes::Aes256>;
    let cipher = Aes256CbcDec::new_from_slices(key, iv)
        .map_err(|e| anyhow::anyhow!("Invalid key/IV: {}", e))?;
    let mut buffer = ciphertext.to_vec();
    let plaintext = cipher
        .decrypt_padded::<Pkcs7>(&mut buffer)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    Ok(plaintext.to_vec())
}

/// Generate RSA key pair  
pub fn generate_rsa_key_pair(bits: u32) -> Result<(String, String)> {
    use rand_core::OsRng;
    use rsa::{
        RsaPrivateKey, RsaPublicKey,
        pkcs8::{EncodePrivateKey, EncodePublicKey},
    };
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, bits as usize)
        .map_err(|e| anyhow::anyhow!("Failed to generate RSA key pair: {}", e))?;
    let public_key = RsaPublicKey::from(&private_key);
    let private_pem = private_key
        .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
        .map_err(|e| anyhow::anyhow!("Failed to encode private key: {}", e))?;
    let public_pem = public_key
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .map_err(|e| anyhow::anyhow!("Failed to encode public key: {}", e))?;
    Ok((private_pem.to_string(), public_pem.to_string()))
}

/// RSA encryption with public key 
pub fn rsa_encrypt(public_key_pem: &str, data: &[u8]) -> Result<Vec<u8>> {
    use rand_core::OsRng;
    use rsa::pkcs1v15::Pkcs1v15Encrypt;
    use rsa::traits::PaddingScheme;
    use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};

    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)
        .map_err(|e| anyhow::anyhow!("Invalid public key: {}", e))?;

    let mut rng = OsRng;
    let padding = Pkcs1v15Encrypt;
    let encrypted = padding
        .encrypt(&mut rng, &public_key, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    Ok(encrypted)
}

/// RSA decryption with private key  
pub fn rsa_decrypt(private_key_pem: &str, data: &[u8]) -> Result<Vec<u8>> {
    use rand_core::OsRng;
    use rsa::pkcs1v15::Pkcs1v15Encrypt;
    use rsa::traits::PaddingScheme;
    use rsa::traits::PublicKeyParts;
    use rsa::{RsaPrivateKey, pkcs8::DecodePrivateKey};

    let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;

    let padding = Pkcs1v15Encrypt;
    let decrypted = padding
        .decrypt(Some(&mut OsRng), &private_key, data)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(decrypted)
}

/// RSA signature with private key  
pub fn rsa_sign(private_key_pem: &str, data: &[u8]) -> Result<Vec<u8>> {
    use ring::rand::SystemRandom;
    use ring::signature::{RSA_PKCS1_SHA256, RsaKeyPair};

    let pem = pem::parse(private_key_pem).map_err(|e| anyhow::anyhow!("Invalid PEM: {}", e))?;

    let key_pair = RsaKeyPair::from_pkcs8(pem.contents())
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;

    let rng = SystemRandom::new();
    let mut signature = vec![0; key_pair.public_modulus_len()];

    key_pair
        .sign(&RSA_PKCS1_SHA256, &rng, data, &mut signature)
        .map_err(|e| anyhow::anyhow!("Sign failed: {}", e))?;

    Ok(signature)
}

/// RSA signature verification with public key  
pub fn rsa_verify(public_key_pem: &str, data: &[u8], signature: &[u8]) -> Result<bool> {
    use ring::signature::{RSA_PKCS1_2048_8192_SHA256, UnparsedPublicKey};

    let pem = pem::parse(public_key_pem).map_err(|e| anyhow::anyhow!("Invalid PEM: {}", e))?;

    let public_key = UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, pem.contents());

    match public_key.verify(data, signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Hash password with bcrypt
pub fn bcrypt_hash(password: &str, cost: u32) -> Result<String> {
    let hashed = bcrypt::hash(password, cost)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
    Ok(hashed)
}

/// Verify password with bcrypt
pub fn bcrypt_verify(password: &str, hash: &str) -> Result<bool> {
    let result = bcrypt::verify(password, hash)
        .map_err(|e| anyhow::anyhow!("Failed to verify password: {}", e))?;
    Ok(result)
}

/// Hash password with Argon2id
pub fn argon2_hash(password: &str) -> Result<String> {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand_core::OsRng;

    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
    Ok(hash.to_string())
}

/// Verify password with Argon2id
pub fn argon2_verify(password: &str, hash: &str) -> Result<bool> {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHash, PasswordVerifier};

    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;

    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate HMAC SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    use hmac::KeyInit;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("Invalid HMAC key: {}", e))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Generate HMAC SHA512
pub fn hmac_sha512(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    use hmac::KeyInit;
    use hmac::{Hmac, Mac};
    use sha2::Sha512;

    let mut mac = Hmac::<Sha512>::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("Invalid HMAC key: {}", e))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}
