# Kerminal - Advanced SSH Terminal Manager

**Kerminal** là một ứng dụng quản lý SSH terminals hiện đại được xây dựng với Tauri + Vue + TypeScript. Ứng dụng cung cấp một giao diện đẹp mắt và tính năng bảo mật cao để quản lý các kết nối SSH.

## ✨ Tính năng chính

### 🔐 Bảo mật & Encryption
- **Master Password System** với AES-256-GCM encryption
- **Multi-device support** với device-specific encryption keys
- **System keychain integration** cho auto-unlock
- **Secure storage** cho passwords và private keys

### 🗄️ Database System
- **Local SQLite database** (luôn khả dụng)
- **External databases sync** (MySQL, PostgreSQL, MongoDB)
- **Conflict resolution** với nhiều strategies
- **Full CRUD operations** cho SSH profiles và groups

### 🔄 Sync & Multi-Device
- **Cross-device synchronization** với encryption support
- **Conflict resolution strategies**: LastWriteWins, FirstWriteWins, Manual
- **Background sync scheduler** với configurable intervals
- **Multi-master password support** cho teams

### 🖥️ Terminal Management
- **Multiple SSH connections** trong một giao diện
- **SSH profiles & groups** để tổ chức kết nối
- **Flexible authentication** (Password, SSH Key, SSH Agent, Kerberos)
- **Connection testing** và monitoring

## 🏗️ Kiến trúc

### Frontend (Vue 3 + TypeScript)
- **Modern Vue 3** với Composition API
- **TypeScript** cho type safety
- **Responsive design** với CSS animations
- **Component-based architecture**

### Backend (Rust + Tauri)
- **Tauri framework** cho desktop app development
- **SQLite** cho local database
- **Multi-provider database system** cho external sync
- **Comprehensive encryption system**

### Database Architecture
```
Local SQLite (Always Available)
    ├── SSH Profiles (encrypted passwords/keys)
    ├── SSH Groups (organization)
    ├── Master Passwords (device-specific)
    └── Sync Metadata (conflict resolution)

External Databases (Optional Sync)
    ├── MySQL/PostgreSQL/MongoDB
    ├── Cross-device synchronization
    └── Team collaboration
```

## 🚀 Cài đặt & Development

### Prerequisites
- **Node.js** (v18+)
- **Rust** (latest stable)
- **Tauri CLI**

### Setup
```bash
# Clone repository
git clone https://github.com/klpod221/kerminal-tauri.git
cd kerminal-tauri

# Install frontend dependencies
npm install

# Install Tauri CLI
npm install -g @tauri-apps/cli

# Development mode
npm run tauri dev

# Build production
npm run tauri build
```

## 📖 Documentation

### Kiến trúc & Thiết kế
- [Database Architecture](./DATABASE_ARCHITECTURE.md) - Chi tiết về hệ thống database
- [Master Password System](./MASTER_PASSWORD_SYSTEM.md) - Bảo mật và encryption

### API Documentation
- **Tauri Commands** - Frontend-backend communication
- **Database Providers** - Multi-database support
- **Sync System** - Cross-device synchronization

## 🛠️ Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 🔧 Configuration

### Database Configuration
```rust
// Local SQLite (auto-configured)
let local_config = DatabaseConfig::new_sqlite("/path/to/kerminal.db");

// External MySQL (for sync)
let mysql_config = DatabaseConfig::new_mysql(
    "sync_db", "mysql.example.com", 3306,
    "user", "password", "kerminal"
);
```

### Master Password Setup
```rust
let config = MasterPasswordConfig {
    auto_unlock: true,
    use_keychain: true,
    session_timeout_minutes: Some(60),
    require_on_startup: true,
};
```

## 🤝 Contributing

Chúng tôi hoan nghênh mọi contribution! Vui lòng:

1. Fork repository
2. Tạo feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Mở Pull Request

## 📝 License

Dự án này được phát hành dưới [MIT License](LICENSE).

## 👤 Author

**Bùi Thanh Xuân (klpod221)**
- Website: [klpod221.com](https://klpod221.com)
- GitHub: [@klpod221](https://github.com/klpod221)
- Email: klpod221@gmail.com

---

*Kerminal - Making SSH management simple, secure, and beautiful.*
