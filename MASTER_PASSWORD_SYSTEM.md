# Master Password System Documentation

## Tổng quan

Master Password system trong Kerminal được thiết kế để:
- Bảo vệ dữ liệu nhạy cảm (passwords, private keys) bằng AES-256-GCM encryption
- Hỗ trợ multi-device với device-specific encryption keys
- Tích hợp với system keychain cho auto-unlock
- Conflict resolution khi sync giữa devices có master password khác nhau

## Kiến trúc Security

### 1. Master Password Flow
```
User Master Password
    ↓ (PBKDF2/Argon2 + Salt)
Device-Specific Key (256-bit)
    ↓ (AES-256-GCM)
Encrypted Data
```

### 2. Components

#### MasterPasswordManager
- Quản lý master password lifecycle
- Device authentication và verification
- Multi-device password support

#### DeviceKeyManager
- Quản lý device-specific encryption keys
- Key derivation từ master passwords
- Memory-based key cache

#### AESEncryption
- AES-256-GCM encryption/decryption
- Random IV generation
- Authenticated encryption

#### KeychainManager
- System keychain integration
- Auto-unlock support
- Secure storage cho passwords và keys

## Database Schema

### Master Passwords Table
```sql
CREATE TABLE master_passwords (
    device_id TEXT PRIMARY KEY,
    device_name TEXT NOT NULL,
    password_salt BLOB NOT NULL,        -- 32 bytes random salt
    verification_hash TEXT NOT NULL,    -- Argon2 hash for verification
    auto_unlock BOOLEAN DEFAULT false,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_verified_at DATETIME
);
```

### Device Encryption Keys Table
```sql
CREATE TABLE device_encryption_keys (
    device_id TEXT PRIMARY KEY,
    device_name TEXT,
    encrypted_key BLOB NOT NULL,        -- Device key encrypted with current master password
    key_salt BLOB NOT NULL,             -- Salt used for key derivation
    key_version INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME
);
```

## Security Features

### 1. Key Derivation
```rust
// PBKDF2 với 100,000 iterations
pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
    password.as_bytes(),
    salt,           // 32 bytes random
    100_000,        // iterations
    &mut key,       // 32 bytes output
);
```

### 2. Password Verification
```rust
// Argon2 cho password hashing
let argon2 = Argon2::default();
let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
```

### 3. AES-256-GCM Encryption
```rust
// Format: [12-byte IV][Encrypted Data][16-byte Auth Tag]
let mut result = Vec::with_capacity(12 + ciphertext.len());
result.extend_from_slice(&nonce_bytes);  // IV
result.extend_from_slice(&ciphertext);   // Data + Auth tag
```

## Multi-Device Support

### Scenario: Device A có password "123", Device B có password "456"

#### 1. Initial Setup Device A:
```rust
// Device A creates master password
let entry_a = manager.create_master_password(
    "Device A".to_string(),
    "123",
    &config,
)?;

// Device A encrypts SSH profile
let encrypted_profile = manager.encrypt_with_device(profile_data, Some("device_a"))?;
```

#### 2. Device B encounters encrypted data:
```rust
// Device B tries to decrypt
match manager.decrypt_with_device(&encrypted_data, Some("device_a")) {
    Ok(data) => {
        // Success - Device B has Device A's key
    },
    Err(EncryptionError::UnknownDeviceKey(_)) => {
        // Prompt user for Device A's master password
        let device_a_password = prompt_user_for_device_password("device_a")?;

        // Add Device A's key to Device B
        manager.add_device_key(
            "device_a".to_string(),
            "Device A".to_string(),
            &device_a_password,
            &device_a_entry,
        )?;

        // Retry decryption
        let data = manager.decrypt_with_device(&encrypted_data, Some("device_a"))?;
    }
}
```

### 3. Fallback Strategy:
```rust
// Try to decrypt với bất kỳ device key nào
match manager.try_decrypt_with_any_device(&encrypted_data) {
    Ok((data, device_id)) => {
        println!("Decrypted with device: {}", device_id);
    },
    Err(_) => {
        // Store encrypted, decrypt later khi có key
        store_encrypted_for_later_resolution(&encrypted_data)?;
    }
}
```

## Auto-Unlock với Keychain

### 1. Enable Auto-Unlock:
```rust
let config = MasterPasswordConfig {
    auto_unlock: true,
    use_keychain: true,
    session_timeout_minutes: Some(60),
    ..Default::default()
};

let entry = manager.create_master_password(device_name, password, &config)?;
```

### 2. Auto-Unlock Process:
```rust
// At app startup
if let Ok(true) = manager.try_auto_unlock(&current_device_id) {
    // Successfully unlocked từ keychain
    println!("Auto-unlocked successfully");
} else {
    // Prompt user for master password
    let password = prompt_master_password()?;
    manager.verify_master_password(&password, &stored_entry)?;
}
```

### 3. Session Management:
```rust
// Check session timeout
if let Some(timeout) = config.session_timeout_minutes {
    if manager.is_session_expired(timeout) {
        manager.clear_all_keys();
        // Require re-authentication
    }
}
```

## Error Handling

### 1. Error Types:
```rust
pub enum EncryptionError {
    EncryptionFailed(String),
    DecryptionFailed(String),
    InvalidKey(String),
    KeyDerivationFailed(String),
    MasterPasswordVerificationFailed,
    UnknownDeviceKey(String),
    KeychainError(String),
    InvalidFormat,
}
```

### 2. Recovery Strategies:
```rust
// Encryption failure
match encrypt_result {
    Err(EncryptionError::InvalidKey(_)) => {
        // Re-derive key từ password
        manager.verify_master_password(&password, &entry)?;
        retry_encrypt()?;
    },
    Err(EncryptionError::UnknownDeviceKey(device_id)) => {
        // Prompt for device password
        prompt_device_password(&device_id)?;
    },
    Err(e) => return Err(e.into()),
}
```

## Usage Examples

### 1. First-time Setup:
```rust
// Setup wizard
let master_password = prompt_new_master_password()?;
let device_name = get_device_name()?;

let config = MasterPasswordConfig {
    auto_unlock: true,
    use_keychain: true,
    require_on_startup: true,
    session_timeout_minutes: Some(60),
};

let entry = master_password_manager.create_master_password(
    device_name,
    &master_password,
    &config,
)?;

// Save entry to database
database.save_master_password_entry(&entry).await?;
```

### 2. Daily Usage:
```rust
// App startup
let stored_entry = database.get_master_password_entry(&device_id).await?;

if stored_entry.auto_unlock {
    if !master_password_manager.try_auto_unlock(&device_id)? {
        // Keychain failed, prompt user
        let password = prompt_master_password()?;
        master_password_manager.verify_master_password(&password, &stored_entry)?;
    }
} else {
    // Always prompt
    let password = prompt_master_password()?;
    master_password_manager.verify_master_password(&password, &stored_entry)?;
}
```

### 3. Encrypting SSH Profile:
```rust
// Create SSH profile với password
let mut profile = SSHProfile::new(device_id, name, host, port, username);
profile.set_authentication(AuthMethod::Password, AuthData::Password {
    password: "secret123".to_string(),
});

// Encryption được handle tự động trong save process
database_service.save(&profile).await?;
```

### 4. Cross-Device Sync:
```rust
// Device B receives encrypted data từ Device A
let sync_result = sync_manager.sync_from_external_database().await;

match sync_result {
    Ok(report) => {
        if report.undecryptable_records > 0 {
            // Có data không decrypt được
            show_device_password_prompt(report.unknown_device_ids)?;
        }
    },
    Err(SyncError::EncryptionError(e)) => {
        // Handle encryption errors
        handle_sync_encryption_error(e)?;
    }
}
```

## Security Best Practices

### 1. Password Requirements:
- Minimum 8 characters
- Mix of uppercase, lowercase, numbers, symbols
- Not common passwords
- Different from system passwords

### 2. Key Management:
- Keys cleared from memory sau timeout
- No keys stored in plain text
- Keychain entries protected by OS
- Regular key rotation option

### 3. Audit Trail:
- Log authentication attempts
- Track device access
- Monitor sync activities
- Alert on suspicious activities

### 4. Recovery Options:
- Master password reset (với data loss warning)
- Export/import encrypted data
- Emergency access codes
- Device deauthorization

Đây là documentation chi tiết cho Master Password system. Bạn có muốn tôi tiếp tục implement SQLite provider không?
