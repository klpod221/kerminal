# Features

Kerminal comes packed with features designed to enhance your terminal experience and streamline SSH management.

## 💻 Terminal Emulator

### Multiple Tabs & Split Panes
- Open multiple terminal sessions in tabs
- Split your workspace horizontally or vertically
- Drag and drop tabs to reorganize
- Native shell integration (bash, zsh, fish, PowerShell, etc.)

### WebGL-Accelerated Rendering
- Hardware-accelerated terminal rendering
- Smooth scrolling and animations
- Unicode 11 support with proper emoji rendering
- Customizable fonts and colors

### Productivity Features
- Full-text search within terminal output
- Clickable links detection
- Clipboard integration (copy/paste)
- Keyboard shortcuts for all common actions

## 📡 SSH Management & Tunneling

### Profile Organization
- Group connections by project, environment, or team
- Color-code profiles for quick identification
- Add descriptions and notes to each profile
- Import/export profiles for backup or sharing

### Authentication Methods
- Password authentication
- SSH key authentication (RSA, Ed25519, ECDSA)
- Certificate-based authentication
- Agent forwarding support (coming soon)
- PKCS11 and Kerberos support (coming soon)

### SSH Key Manager
- Generate new SSH keys
- Import existing keys
- Export keys for backup
- Test connections before saving

### Proxy Support
- HTTP proxy
- SOCKS4 proxy
- SOCKS5 proxy
- Automatic proxy detection

### Jump Host Chain
- Connect through multiple bastion hosts
- Automatic authentication at each hop
- Visual chain representation
- Save complex multi-hop configurations

### Port Forwarding
- **Local Forwarding**: Access remote services locally
- **Remote Forwarding**: Expose local services remotely
- **Dynamic SOCKS**: Create a SOCKS proxy through SSH
- Auto-start forwarding on connection
- Real-time status monitoring

## 💾 Saved Commands & Session Recording

### Command Library
- Save frequently used commands
- Organize with groups and categories
- Track usage statistics
- Mark favorites for quick access
- Variable substitution (e.g., `${username}`, `${hostname}`)

### Session Recording
- Record terminal sessions in asciicast v2 format
- Playback with controls (play, pause, speed)
- Export recordings for sharing
- Compatible with asciinema.org

## 🔄 Multi-Device Sync & Security

### Cloud Synchronization
- Sync profiles across devices
- Support for MySQL, PostgreSQL, and MongoDB
- AES-256-GCM encryption for all synced data
- Conflict resolution strategies
- Device management with revocation

### Auto-Sync
- Automatic background synchronization
- Sync on startup option
- Manual sync trigger
- Sync status indicators

## 🔒 Security

### Master Password
- Encrypt all sensitive data with a master password
- Never stored - only verification hash is kept
- Required on startup (configurable)

### Device-Specific Encryption
- Each device has unique encryption keys
- Data cannot be accessed from other devices
- Secure key derivation with Argon2

### Platform Keychain Integration
- Store master password in system keychain
- Auto-unlock on login (optional)
- Secure credential storage

### Session Security
- Automatic session locking after inactivity
- Configurable lock timeout
- Require password to unlock

### SSH Key Protection
- Private keys never leave device unencrypted
- Encrypted storage at rest
- Passphrase protection support

## 🎨 User Interface

### Modern Dark Theme
- Easy on the eyes for long sessions
- Consistent design language
- Smooth animations and transitions

### Customization
- Custom color schemes
- Font selection and sizing
- Terminal opacity settings
- Cursor style options

### Status Indicators
- Real-time connection status
- Sync status
- Recording indicator
- Port forwarding status

### Keyboard Shortcuts
- Comprehensive keyboard navigation
- Customizable shortcuts
- Quick command palette
