<div align="center">
    <h1>Kerminal: Modern Terminal Emulator & SSH Manager</h1>
    <p>A powerful, feature-rich terminal emulator with advanced SSH management, multi-device sync, and enterprise-grade encryption built with Tauri + Vue 3.</p>
    <img src="https://img.shields.io/github/last-commit/klpod221/kerminal?style=for-the-badge&color=74c7ec&labelColor=111827" alt="Last Commit">
    <img src="https://img.shields.io/github/stars/klpod221/kerminal?style=for-the-badge&color=facc15&labelColor=111827" alt="GitHub Stars">
    <img src="https://img.shields.io/github/repo-size/klpod221/kerminal?style=for-the-badge&color=a78bfa&labelColor=111827" alt="Repo Size">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge&color=34d399&labelColor=111827" alt="License">
    <br/>
    <a href="https://github.com/sponsors/klpod221">
        <img src="https://img.shields.io/badge/Sponsor-GitHub-ea4aaa?style=for-the-badge&logo=github&labelColor=111827" alt="GitHub Sponsors">
    </a>
    <a href="https://www.buymeacoffee.com/klpod221">
        <img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black&labelColor=111827" alt="Buy Me a Coffee">
    </a>
</div>

## 📝 Description

**Kerminal** is a modern, high-performance terminal emulator that combines the power of a full-featured local terminal with advanced SSH connection management. Built with security-first architecture using Tauri (Rust) for native performance and Vue 3 for a responsive UI, Kerminal offers everything from basic terminal operations to complex SSH workflows with encrypted profile management, tunneling, and multi-device synchronization—all in a beautiful native desktop application.

Perfect for developers, DevOps engineers, system administrators, and anyone who lives in the terminal and values security, organization, and productivity.

## 🚀 Table Of Content

- [📝 Description](#-description)
- [🚀 Table Of Content](#-table-of-content)
- [📸 Screenshots](#-screenshots)
  - [Dashboard](#dashboard)
  - [Main Interface](#main-interface)
- [✨ Features](#-features)
  - [💻 Terminal Emulator](#-terminal-emulator)
  - [📡 SSH Management \& Tunneling](#-ssh-management--tunneling)
  - [💾 Saved Commands \& Session Recording](#-saved-commands--session-recording)
  - [🔄 Multi-Device Sync \& Security](#-multi-device-sync--security)
  - [🎨 User Interface](#-user-interface)
- [Installation Guide](#installation-guide)
  - [Arch Linux (install from AUR)](#arch-linux-install-from-aur)
  - [Other Platforms (Windows, macOS, Linux)](#other-platforms-windows-macos-linux)
- [🚀 Development](#-development)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Project Structure](#project-structure)
  - [Key Technologies](#key-technologies)
- [🔒 Security Considerations](#-security-considerations)
- [🤝 Contributing](#-contributing)
- [❗ Known Issues](#-known-issues)
- [📝 License](#-license)
- [👤 Author](#-author)
- [🙏 Acknowledgments](#-acknowledgments)
- [📮 Support](#-support)
- [🗺️ Roadmap](#️-roadmap)

## 📸 Screenshots

### Dashboard

![Dashboard](public/screenshots/Dashboard.png)

### Main Interface

![Kerminal Main Interface](public/screenshots/MainInterface.png)

## ✨ Features

### 💻 Terminal Emulator
- Multiple tabs and split panes, native shell integration (bash, zsh, fish, PowerShell, etc.)
- WebGL-accelerated rendering with Unicode 11 support
- Search, clickable links, clipboard integration

### 📡 SSH Management & Tunneling
- Profile organization with groups, colors, and descriptions
- Authentication: password and keys (certificate, Kerberos, PKCS11, agent coming soon)
- SSH key manager with import/export, connection testing, proxy support (HTTP, SOCKS4/5)
- Port forwarding (Local/Remote/Dynamic) with auto-start and status monitoring

### 💾 Saved Commands & Session Recording
- Command library with groups, usage tracking, favorites, and variable substitution
- Record sessions in `asciicast` format with playback controls and export capabilities

### 🔄 Multi-Device Sync & Security
- Sync via MySQL/PostgreSQL/MongoDB with AES-256-GCM encryption
- Conflict resolution strategies, device management, auto-sync
- Master password protection, device-specific keys, keychain integration, auto-lock sessions

### 🎨 User Interface
- Modern dark theme, keyboard shortcuts, customizable colors, real-time status indicators

## Installation Guide

### Arch Linux (install from AUR)

- Using an AUR helper (e.g., yay):
```bash
yay -S kerminal # or kerminal-bin
```

- Manually:
```bash
git clone https://aur.archlinux.org/kerminal.git # or kerminal-bin.git
cd kerminal
makepkg -si
```

### Other Platforms (Windows, macOS, Linux)

1. Download the latest release from the [Releases](https://github.com/klpod221/kerminal/releases/latest) page
2. Follow the installation instructions for your operating system

## 🚀 Development

### Prerequisites
- **Node.js** (v20 or higher)
- **Rust** (latest stable)
- **Tauri CLI**: `cargo install tauri-cli`

### Installation

1. Clone the repository
```bash
git clone https://github.com/klpod221/kerminal.git
cd kerminal
```

2. Install dependencies
```bash
npm install
```

3. Run in development mode
```bash
npm run tauri dev
```

4. Build for production
```bash
npm run tauri build
```

The application will be available in `src-tauri/target/release/bundle/`.

### Project Structure

- **Frontend**: Vue 3 with Composition API, Pinia stores, TypeScript
- **Backend**: Rust with Tauri v2, async/await with Tokio
- **Terminal**: xterm.js with WebGL renderer and addons
- **Recording**: asciicast v2 format with asciinema-player for playback
- **SSH**: russh library for SSH protocol implementation
- **Database**: SQLx for SQL databases, MongoDB driver for NoSQL
- **Encryption**: AES-GCM with Argon2 key derivation

### Key Technologies

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Frontend | Vue 3 + TypeScript | Reactive UI framework |
| State | Pinia | Centralized state management |
| Backend | Rust + Tauri v2 | Native performance and security |
| SSH | russh | SSH protocol implementation |
| Terminal | xterm.js | Terminal emulation |
| Recording | asciinema-player | Session playback |
| Database | SQLite, MySQL, PostgreSQL, MongoDB | Local and sync storage |
| Encryption | AES-256-GCM + Argon2 | Data encryption and key derivation |

## 🔒 Security Considerations

- All sensitive data encrypted at rest with AES-256-GCM
- Master password never stored, only verification hash
- Device-specific encryption keys prevent data access from other devices
- SSH private keys never leave the device unencrypted
- Sync data encrypted before transmission
- Automatic session locking after inactivity
- Platform keychain integration for secure auto-unlock

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## ❗ Known Issues

- Limited support for some SSH authentication methods
- MacOS version is **not signed/notarized yet** due to Apple Developer Program restrictions (it takes **99 USD/year!**) So please build from source if you want to use on MacOS or run unsigned app with `xattr -rd com.apple.quarantine /path/to/Kerminal.app` after first launch.
- Android version is currently not working (some how it works on my device (Xiaomi Redmi Note 12) but not on other devices).

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👤 Author

**Bùi Thanh Xuân (klpod221)**

- Website: [klpod221.com](https://klpod221.com)
- GitHub: [@klpod221](https://github.com/klpod221)
- Email: [klpod221@gmail.com](mailto:klpod221@gmail.com)

## 🙏 Acknowledgments

- **Tauri** - For the amazing Rust-based desktop framework
- **Vue 3** - For the reactive and performant frontend framework
- **xterm.js** - For the excellent terminal emulator
- **asciinema-player** - For the powerful terminal session player
- **russh** - For the robust SSH implementation in Rust
- **Lucide** - For the beautiful icon set

## 📮 Support

If you encounter any issues or have questions:

1. Check existing [Issues](https://github.com/klpod221/kerminal/issues)
2. Create a new issue with detailed information
3. Contact via email: klpod221@gmail.com

## 🗺️ Roadmap

- [x] Custom terminal themes and color schemes
- [x] Custom terminal font settings
- [x] Syntax highlighting for saved commands
- [x] Session recording and playback (asciicast format)
- [x] SFTP file transfer integration
- [ ] Support graphic protocol
- [ ] More SSH Authentication Methods
- [ ] Plugin system for extensions
- [ ] AI Agent for auto-completion, suggestion, chat and automation tasks
- [ ] Cloud backup integration
- [ ] Web-based version
- [ ] Mobile app companion

---

<div align="center">
    <p>Made with ❤️ by klpod221</p>
    <p>⭐ Star this repository if you find it helpful!</p>
</div>
