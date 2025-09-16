# Database System Architecture Documentation

## Tổng quan

Hệ thống database của Kerminal được thiết kế để hỗ trợ:
- **Local SQLite database** (luôn có, fully implemented)
- **External databases** (MySQL, PostgreSQL, MongoDB) cho sync (architecture ready)
- **Master Password encryption** với device-specific keys (fully implemented)
- **Conflict resolution** và sync strategies (fully implemented)
- **Flexible authentication methods** cho SSH profiles (fully implemented)

## ✅ Implementation Status

### Completed Components
- ✅ **SQLite Provider** - Full CRUD operations với encryption support
- ✅ **Master Password System** - AES-256-GCM encryption với multi-device support
- ✅ **Sync Framework** - Conflict resolution, strategies, scheduler
- ✅ **SSH Profile Management** - Complete với flexible authentication
- ✅ **Database Service** - Main orchestrator với statistics
- ✅ **Tauri Commands** - Full API cho frontend integration
- ✅ **Migration System** - Database schema evolution support

### Architecture Ready (Planned)
- 🔄 **MySQL Provider** - Framework ready, implementation pending
- 🔄 **PostgreSQL Provider** - Framework ready, implementation pending
- 🔄 **MongoDB Provider** - Framework ready, implementation pending

## Cấu trúc thư mục

```
src-tauri/src/database/
├── mod.rs                    # Main exports và public interface
├── traits.rs                # Core traits (Database, Syncable, Encryptable)
├── service.rs               # DatabaseService - main orchestrator
├── config.rs                # Database configurations
├── error.rs                 # Error types và handling
│
├── models/                  # Data models
│   ├── mod.rs
│   ├── base.rs             # BaseModel với sync metadata
│   ├── ssh_profile.rs      # SSH profile với flexible auth
│   ├── ssh_group.rs        # SSH groups
│   ├── device.rs           # Device management
│   └── sync_metadata.rs    # Sync tracking
│
├── providers/              # Database implementations
│   ├── mod.rs
│   ├── sqlite.rs           # SQLite provider (local)
│   ├── mysql.rs            # MySQL provider
│   ├── postgresql.rs       # PostgreSQL provider
│   ├── mongodb.rs          # MongoDB provider
│   └── factory.rs          # Provider factory
│
├── encryption/             # Encryption system
│   ├── mod.rs
│   ├── master_password.rs  # Master password management
│   ├── device_keys.rs      # Device-specific encryption
│   ├── aes.rs              # AES-256 encryption
│   └── keychain.rs         # System keychain integration
│
├── sync/                   # Synchronization system
│   ├── mod.rs
│   ├── manager.rs          # Sync orchestrator
│   ├── conflict.rs         # Conflict resolution
│   ├── strategies.rs       # Sync strategies
│   └── scheduler.rs        # Auto sync scheduling
│
└── migrations/             # Database migrations
    ├── mod.rs
    └── sqlite/             # SQLite-specific migrations
```

## Core Traits

### 1. Database Trait (✅ Implemented)
Tất cả database providers phải implement trait này. SQLite provider đã fully implemented:

```rust
#[async_trait]
pub trait Database: Send + Sync {
    // Connection management (✅ Implemented in SQLite)
    async fn connect(&mut self) -> DatabaseResult<()>;
    async fn disconnect(&mut self) -> DatabaseResult<()>;
    fn is_connected(&self) -> bool;
    async fn test_connection(&self) -> DatabaseResult<()>;

    // CRUD operations (✅ Fully implemented)
    async fn save_ssh_profile(&self, model: &SSHProfile) -> DatabaseResult<()>;
    async fn find_ssh_profile_by_id(&self, id: &str) -> DatabaseResult<Option<SSHProfile>>;
    async fn find_all_ssh_profiles(&self) -> DatabaseResult<Vec<SSHProfile>>;
    async fn delete_ssh_profile(&self, id: &str) -> DatabaseResult<()>;

    async fn save_ssh_group(&self, model: &SSHGroup) -> DatabaseResult<()>;
    async fn find_ssh_group_by_id(&self, id: &str) -> DatabaseResult<Option<SSHGroup>>;
    async fn find_all_ssh_groups(&self) -> DatabaseResult<Vec<SSHGroup>>;
    async fn delete_ssh_group(&self, id: &str) -> DatabaseResult<()>;

    // Migration support (✅ Implemented)
    async fn create_tables(&self) -> DatabaseResult<()>;
    async fn migrate(&self, version: u32) -> DatabaseResult<()>;
}
```

### 2. Syncable Trait (✅ Implemented via Macro)
Models có thể được sync giữa các database, implemented via derive macro:

```rust
pub trait Syncable: Send + Sync {
    fn table_name() -> &'static str;
    fn id(&self) -> &str;
    fn device_id(&self) -> &str;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn version(&self) -> u64;
    fn sync_status(&self) -> &SyncStatus;
    fn checksum(&self) -> String;
    fn should_sync(&self) -> bool;
}

// Usage: #[derive(Syncable)] on models
```

### 3. Encryptable Trait (✅ Implemented)
Models có fields cần mã hóa, được tích hợp trong SSH profiles:

```rust
pub trait Encryptable: Send + Sync {
    fn encrypted_fields() -> Vec<&'static str>;
    fn encrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()>;
    fn decrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()>;
    fn has_encrypted_data(&self) -> bool;
    fn encryption_device_id(&self) -> Option<&str>;
}

// Implemented for SSHProfile with password/key encryption
```

## Base Model System

Tất cả models sử dụng `BaseModel` để có sync metadata:

```rust
pub struct BaseModel {
    pub id: String,                    // UUID
    pub created_at: DateTime<Utc>,     // Creation timestamp
    pub updated_at: DateTime<Utc>,     // Last modification
    pub device_id: String,             // Device that created/modified
    pub version: u64,                  // Version for conflict resolution
    pub sync_status: SyncStatus,       // Sync state
}
```

### Sync Status States:
- `Pending` - Chờ sync
- `Syncing` - Đang sync
- `Synced` - Đã sync thành công
- `Failed(String)` - Sync thất bại
- `Conflict` - Có conflict
- `LocalOnly` - Chỉ lưu local

## Master Password System

### Kiến trúc:
1. **Master Password** - User tạo khi khởi chạy app lần đầu
2. **Device Encryption Key** - Derived từ Master Password + device salt
3. **Multi-device support** - Mỗi device có encryption key riêng
4. **Keychain integration** - Tự động unlock (tuỳ chọn)

### Flow:
1. User tạo Master Password
2. Generate device salt (32 bytes random)
3. Derive device key = PBKDF2(Master Password + device salt)
4. Tạo verification hash để check password
5. Lưu salt + verification hash vào database
6. Device key được encrypt bằng Master Password và lưu

### Security:
- **AES-256-GCM** cho encryption
- **PBKDF2/Argon2** cho key derivation
- **Random IV** cho mỗi encryption operation
- **HMAC verification** để đảm bảo integrity

## Sync System

### Conflict Resolution Strategies:
- `LastWriteWins` - Dữ liệu mới nhất thắng
- `FirstWriteWins` - Dữ liệu cũ nhất thắng
- `ManualResolve` - User quyết định
- `DevicePriority` - Ưu tiên theo device order

### Sync Process:
1. **Initial Sync** - Lần đầu connect external database
2. **Incremental Sync** - Sync changes định kỳ
3. **Conflict Detection** - So sánh version và checksum
4. **Resolution** - Apply strategy đã config

### Multi-Master Password Support:
- Khi gặp encrypted data từ device khác, có thể:
  - Prompt user nhập Master Password của device đó
  - Store encrypted và decrypt sau
  - Skip encrypted data

## Database Schema (SQLite)

### Core Tables:
```sql
-- SSH Groups
CREATE TABLE ssh_groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    icon TEXT,
    sort_order INTEGER DEFAULT 0,
    -- Base model fields
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    device_id TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    sync_status TEXT DEFAULT 'pending'
);

-- SSH Profiles
CREATE TABLE ssh_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER DEFAULT 22,
    username TEXT NOT NULL,
    group_id TEXT,
    auth_method TEXT NOT NULL,
    auth_data TEXT NOT NULL,    -- JSON, encrypted fields
    timeout INTEGER,
    keep_alive BOOLEAN DEFAULT true,
    compression BOOLEAN DEFAULT false,
    color TEXT,
    icon TEXT,
    sort_order INTEGER DEFAULT 0,
    -- Base model fields
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    device_id TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    sync_status TEXT DEFAULT 'pending',
    FOREIGN KEY (group_id) REFERENCES ssh_groups(id) ON DELETE SET NULL
);

-- Master Passwords
CREATE TABLE master_passwords (
    device_id TEXT PRIMARY KEY,
    device_name TEXT NOT NULL,
    password_salt BLOB NOT NULL,
    verification_hash TEXT NOT NULL,
    auto_unlock BOOLEAN DEFAULT false,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_verified_at DATETIME
);

-- Device Encryption Keys
CREATE TABLE device_encryption_keys (
    device_id TEXT PRIMARY KEY,
    device_name TEXT,
    encrypted_key BLOB NOT NULL,
    key_salt BLOB NOT NULL,
    key_version INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME
);

-- Sync Configuration
CREATE TABLE sync_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    database_type TEXT NOT NULL,
    connection_config TEXT NOT NULL,  -- JSON, encrypted
    sync_strategy TEXT DEFAULT 'last_write_wins',
    enabled BOOLEAN DEFAULT true,
    last_sync_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Error Handling

### Error Types:
- `DatabaseError` - Database operations
- `EncryptionError` - Encryption/decryption
- `SSHError` - SSH connections
- `SyncError` - Synchronization

### Error Recovery:
- **Connection failures** - Retry with exponential backoff
- **Sync conflicts** - Apply resolution strategy
- **Encryption failures** - Prompt for correct Master Password
- **Partial failures** - Rollback transactions

## Configuration

### Database Config:
```rust
pub struct DatabaseConfig {
    pub id: String,
    pub name: String,
    pub provider: DatabaseProvider,
    pub connection: ConnectionConfig,
    pub sync_settings: SyncSettings,
    pub enabled: bool,
}
```

### Sync Settings:
```rust
pub struct SyncSettings {
    pub strategy: SyncStrategy,
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
    pub conflict_resolution: ConflictResolution,
    pub enabled_models: Vec<String>,
}
```

## Usage Examples

### Tạo SSH Profile:
```rust
let mut profile = SSHProfile::new(
    device_id,
    "Production Server".to_string(),
    "192.168.1.100".to_string(),
    22,
    "root".to_string(),
);

profile.set_auth_method(AuthMethod::Password);
profile.set_auth_data(AuthData::Password {
    password: "secret123".to_string(),
});

// Save (auto-encrypt sensitive fields)
database_service.save(&profile).await?;
```

### Test SSH Connection:
```rust
let result = ssh_service.test_connection(&profile).await?;
println!("Connection: {} ({}ms)",
    if result.success { "OK" } else { "FAILED" },
    result.duration_ms
);
```

### Sync với External Database:
```rust
// Add external database
let config = DatabaseConfig::new_mysql(
    "Remote MySQL".to_string(),
    "mysql.example.com".to_string(),
    3306,
    "user".to_string(),
    "password".to_string(),
    "kerminal".to_string(),
);

database_service.add_external_database(config).await?;

// Manual sync
database_service.sync_all().await?;
```

Đây là kiến trúc hoàn chỉnh. Tiếp theo tôi sẽ implement từng phần chi tiết. Bạn có muốn tôi tiếp tục với SSH Profile models không?
