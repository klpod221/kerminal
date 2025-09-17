use crate::database::error::{EncryptionError, EncryptionResult};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::engine::general_purpose;
use base64::Engine;
use rand::RngCore;

/// AES-256-GCM encryption service
pub struct AESEncryption;

#[allow(dead_code)]
impl AESEncryption {
    /// Encrypt data using AES-256-GCM
    pub fn encrypt(key: &[u8; 32], data: &[u8]) -> EncryptionResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;

        // Generate random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Combine nonce + ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt(key: &[u8; 32], encrypted_data: &[u8]) -> EncryptionResult<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(EncryptionError::InvalidFormat);
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;

        // Extract nonce và ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }

    /// Encrypt string và return base64 encoded result
    pub fn encrypt_string(key: &[u8; 32], data: &str) -> EncryptionResult<String> {
        let encrypted = Self::encrypt(key, data.as_bytes())?;
        Ok(general_purpose::STANDARD.encode(encrypted))
    }

    /// Decrypt base64 encoded string
    pub fn decrypt_string(key: &[u8; 32], encrypted_data: &str) -> EncryptionResult<String> {
        let encrypted_bytes = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|_e| EncryptionError::InvalidFormat)?;

        let decrypted = Self::decrypt(key, &encrypted_bytes)?;

        String::from_utf8(decrypted)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
    }

    /// Generate a random 256-bit key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Generate a random salt
    pub fn generate_salt() -> [u8; 32] {
        Self::generate_key() // Same as key generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = AESEncryption::generate_key();
        let data = b"Hello, World!";

        let encrypted = AESEncryption::encrypt(&key, data).unwrap();
        let decrypted = AESEncryption::decrypt(&key, &encrypted).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = AESEncryption::generate_key();
        let data = "Hello, World!";

        let encrypted = AESEncryption::encrypt_string(&key, data).unwrap();
        let decrypted = AESEncryption::decrypt_string(&key, &encrypted).unwrap();

        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_invalid_key() {
        let data = b"Hello, World!";
        let result = AESEncryption::encrypt(&[0u8; 32], data); // Valid key size
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_data() {
        let key = AESEncryption::generate_key();
        let data = b"Hello, World!";

        let mut encrypted = AESEncryption::encrypt(&key, data).unwrap();
        encrypted[20] ^= 1; // Tamper with data

        let result = AESEncryption::decrypt(&key, &encrypted);
        assert!(result.is_err());
    }
}
