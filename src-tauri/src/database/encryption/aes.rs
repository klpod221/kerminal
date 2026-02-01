use crate::database::error::{EncryptionError, EncryptionResult};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::Rng;

/// AES-256-GCM encryption service
pub struct AESEncryption;

impl AESEncryption {
    /// Encrypt data using AES-256-GCM
    pub fn encrypt(key: &[u8; 32], data: &[u8]) -> EncryptionResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;

        let nonce_bytes: [u8; 12] = OsRng.gen();
        let nonce = &Nonce::from(nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

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

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = &Nonce::from(*<&[u8; 12]>::try_from(nonce_bytes).unwrap());

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }

    /// Generate a random salt
    pub fn generate_salt() -> [u8; 32] {
        OsRng.gen()
    }
}
