# Sync & Security

Learn about Kerminal's multi-device synchronization and security features.

## Multi-Device Sync

Kerminal allows you to synchronize your SSH profiles, saved commands, and settings across multiple devices.

### Supported Databases

| Database | Description |
|----------|-------------|
| MySQL | Full-featured relational database |
| PostgreSQL | Advanced open-source database |
| MongoDB | NoSQL document database |

### Setting Up Sync

1. Click the **Sync Manager** button (Cloud icon) in the top bar
2. Choose your database type
3. Enter connection details:
   - Host
   - Port
   - Database name
   - Username
   - Password
4. Click **Test Connection**
5. Enable sync

### Setting Up MongoDB Atlas

Kerminal supports MongoDB Atlas for sync. Map your connection string to the fields:

**Connection String Example:**
`mongodb+srv://user:pass@cluster0.abcde.mongodb.net/?retryWrites=true&w=majority`

**Enter in Kerminal:**
- **Protocol**: `mongodb+srv`
- **Host**: `cluster0.abcde.mongodb.net`
- **Port**: `27017` (Default)
- **Username**: `user`
- **Password**: `pass`
- **Options**: `retryWrites=true&w=majority`

### What Gets Synced

- ✅ SSH profiles and groups
- ✅ Saved commands
- ✅ Port forwarding configurations
- ✅ Application settings
- ❌ SSH private keys (never synced)
- ❌ Master password

### Sync Behavior

#### Auto-Sync
Enable automatic synchronization:
- Sync on startup
- Sync on profile changes
- Background sync interval

#### Manual Sync
Trigger sync manually:
- Click the sync button in status bar
- Use keyboard shortcut
- Sync Manager > Settings > Sync Now

### Conflict Resolution

When the same item is modified on multiple devices:

| Strategy | Description |
|----------|-------------|
| **Last Write Wins** | Most recent change is kept |
| **Ask** | Prompt user to choose |
| **Keep Local** | Always prefer local changes |
| **Keep Remote** | Always prefer remote changes |

### Device Management

Manage connected devices:

1. Open **Sync Manager** (Cloud icon)
2. Go to the **Devices** tab
3. View all registered devices
4. Revoke access for lost/stolen devices
5. Rename devices for identification

## Security Architecture

### Encryption at Rest

All sensitive data is encrypted before storage:

```
Data → AES-256-GCM Encryption → Encrypted Storage
```

**Protected data includes:**
- SSH passwords
- Private key passphrases
- Sync credentials
- Saved command variables

### Master Password

The master password is the key to all encrypted data:

- **Never stored** - only a verification hash is kept
- **Cannot be recovered** - if forgotten, encrypted data is lost
- **Used to derive** encryption keys via Argon2

#### Setting Master Password

1. First launch prompts for master password
2. Or click **Master Password Settings** (Shield icon) in the top bar
3. Enter a strong password
4. Confirm the password
5. Optionally store in system keychain

#### Changing Master Password

1. Click **Master Password Settings** (Shield icon)
2. Click **Change Master Password**
3. Enter current password
4. Enter new password
5. All data is re-encrypted with new key

### Key Derivation

Kerminal uses **Argon2id** for key derivation:

```
Master Password + Salt → Argon2id → Encryption Key
```

Argon2 parameters are tuned for security:
- Memory: 64 MB
- Iterations: 3
- Parallelism: 4

### Device-Specific Keys

Each device has unique encryption that prevents data access from other devices:

1. Device generates unique key pair on first run
2. Sync data is encrypted with device-specific key
3. Other devices cannot decrypt without proper key exchange

### Synced Data Encryption

Data synced to cloud databases is always encrypted:

```
Local Data → Encrypt with Master Key → Sync to Database
```

The server never sees unencrypted:
- Passwords
- Private key data
- Sensitive configuration

## Session Security

### Auto-Lock

Automatically lock Kerminal after inactivity:

1. Click **Master Password Settings** (Shield icon)
2. Enable **Auto-Lock**
3. Set timeout (e.g., 5 minutes)
4. Choose lock behavior:
   - Close all connections
   - Keep connections (lock UI only)

### Unlock

When locked, enter your master password to unlock.

### Keychain Integration

Store master password in system keychain for auto-unlock:

| Platform | Keychain |
|----------|----------|
| Windows | Windows Credential Manager |
| macOS | Keychain Access |
| Linux | Secret Service (GNOME Keyring, KWallet) |

**Enable auto-unlock:**
1. Click **Master Password Settings** (Shield icon)
2. Enable **Store in Keychain**
3. Enter master password when prompted

## SSH Key Security

### Private Key Storage

Private keys are stored with encryption:

```
Private Key → Encrypt with Device Key → Secure Storage
```

### Key Passphrases

When importing passphrase-protected keys:
- Passphrase can be stored encrypted
- Or prompted each time (more secure)

### Never Exported

Private keys are never:
- Synced to cloud
- Exported without explicit action
- Logged or transmitted

## Best Practices

### Master Password

1. **Use a strong password** (12+ characters, mixed case, numbers, symbols)
2. **Don't reuse** passwords from other services
3. **Consider a passphrase** (easier to remember, still secure)
4. **Store backup** in a secure password manager

### Sync Security

1. **Use TLS/SSL** for database connections
2. **Strong database passwords**
3. **Regular device audit** - revoke unknown devices
4. **Encrypt database backups**

### Key Management

1. **Use Ed25519** for new keys (more secure, faster)
2. **Protect with passphrase**
3. **Different keys** for different environments
4. **Regular rotation** (yearly recommended)

### General

1. **Keep Kerminal updated** for security patches
2. **Lock when away** from computer
3. **Review connected devices** regularly
4. **Backup encrypted exports** securely
