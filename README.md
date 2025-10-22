# This branch has been marked as archived and is now kept for reference purposes only.

<div align="center">
    <h1>Kerminal: Modern Terminal with SSH, Tunneling & Cross-Device Sync</h1>
    <p>A powerful, cross-platform terminal application with advanced SSH management, comprehensive tunneling capabilities, and seamless data synchronization across all your devices.</p>
    <img src="https://img.shields.io/github/last-commit/klpod221/kerminal?style=for-the-badge&color=74c7ec&labelColor=111827" alt="Last Commit">
    <img src="https://img.shields.io/github/stars/klpod221/kerminal?style=for-the-badge&color=facc15&labelColor=111827" alt="GitHub Stars">
    <img src="https://img.shields.io/github/repo-size/klpod221/kerminal?style=for-the-badge&color=a78bfa&labelColor=111827" alt="Repo Size">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge&color=34d399&labelColor=111827" alt="License">
</div>

## 📝 Description

**Kerminal** is a modern terminal application that combines powerful SSH management, comprehensive tunneling capabilities, and cross-device synchronization. Built with Electron and Vue.js, it provides an intuitive interface for developers and system administrators.

**Key Features:**

- **SSH Management** - Organize profiles with groups and secure proxy support
- **Port Tunneling** - Visual tunnel management with real-time monitoring
- **Cross-Device Sync** - MongoDB-powered synchronization across devices
- **Modern Interface** - Split panels, keyboard shortcuts, and dark themes

Perfect for managing cloud infrastructure, remote development, and multi-server environments.

## Table of Contents

- [📝 Description](#-description)
- [Table of Contents](#table-of-contents)
- [📸 Screenshots](#-screenshots)
  - [Main Interface](#main-interface)
  - [Dashboard](#dashboard)
  - [SSH Profiles](#ssh-profiles)
  - [Saved Commands](#saved-commands)
  - [SSH Tunneling](#ssh-tunneling)
  - [Sync Settings](#sync-settings)
- [✨ Features](#-features)
  - [🖥️ **Modern Terminal**](#️-modern-terminal)
  - [🔐 **SSH Management**](#-ssh-management)
  - [🌐 **Port Tunneling**](#-port-tunneling)
  - [☁️ **Cross-Device Sync**](#️-cross-device-sync)
  - [⚡ **Automation \& Tools**](#-automation--tools)
- [🚀 Installation Guide](#-installation-guide)
  - [System Requirements](#system-requirements)
  - [Option 1: Download Pre-built Packages (Recommended)](#option-1-download-pre-built-packages-recommended)
    - [Windows](#windows)
    - [macOS](#macos)
    - [Linux](#linux)
  - [Installation Instructions](#installation-instructions)
    - [Windows Installation](#windows-installation)
    - [macOS Installation](#macos-installation)
    - [Linux Installation](#linux-installation)
  - [Option 2: Build from Source](#option-2-build-from-source)
    - [Prerequisites](#prerequisites)
    - [Build Instructions](#build-instructions)
- [🐞 Known Issues](#-known-issues)
- [🗺️ Roadmap \& Checklist](#️-roadmap--checklist)
- [Contributors ✨](#contributors-)
  - [How to Contribute](#how-to-contribute)
  - [🙏 Our Valued Contributors](#-our-valued-contributors)
- [📄 License](#-license)
- [👨‍💻 Author](#-author)
- [📊 Project Status](#-project-status)

## 📸 Screenshots

### Main Interface

![Kerminal Main Interface](screenshots/main-interface.png)

_Main interface showing terminal tabs, SSH profiles, and split panels_

### Dashboard

![Dashboard](screenshots/dashboard.png)

_Dashboard with quick access, recent connections, and machine info_

### SSH Profiles

![SSH Profiles](screenshots/ssh-profiles.png)

_Manage SSH profiles with groups and proxy settings_

### Saved Commands

![Saved Commands](screenshots/saved-commands.png)

_Store and quickly access your frequently used commands_

### SSH Tunneling

![SSH Tunneling](screenshots/ssh-tunneling.png)

_Visual SSH tunnel management with real-time status_

### Sync Settings

![Sync Settings](screenshots/sync-settings.png)

_Manage synchronization data with MongoDB_

## ✨ Features

### 🖥️ **Modern Terminal**

- Multi-tab and split panel support
- Customizable keyboard shortcuts
- Beautiful dark theme optimized for coding
- Cross-platform (Windows, macOS, Linux)

### 🔐 **SSH Management**

- Profile organization with groups
- HTTP, SOCKS, and Jump Host proxy support
- Secure credential storage
- One-click server connections

### 🌐 **Port Tunneling**

- Local, remote, and dynamic (SOCKS) forwarding
- Visual tunnel management with real-time status
- Auto-start tunnels on launch
- Color-coded status indicators

### ☁️ **Cross-Device Sync**

- MongoDB-powered synchronization
- Real-time sync with configurable intervals
- Intelligent conflict resolution
- Seamless data migration

### ⚡ **Automation & Tools**

- Saved commands with quick execution
- Command history across sessions
- Auto-updates and export/import
- Advanced search functionality

## 🚀 Installation Guide

### System Requirements

- **Windows**: Windows 10 or higher (64-bit)
- **macOS**: macOS 10.15 (Catalina) or higher
- **Linux**: Ubuntu 18.04+, Debian 10+, Arch Linux, or equivalent distributions

### Option 1: Download Pre-built Packages (Recommended)

Download the appropriate installer for your operating system from the [Releases page](https://github.com/klpod221/kerminal/releases).

> **Note**: Replace `{version}` with the actual version number (e.g., `0.5.0`) from the latest release.

#### Windows

- **kerminal-{version}-setup.exe** - Windows installer (recommended)
- **kerminal\_{version}\_amd64.deb** - For Windows Subsystem for Linux (WSL)

#### macOS

- **kerminal-{version}.dmg** - macOS disk image
- **kerminal-{version}.AppImage** - Portable AppImage (alternative)

#### Linux

- **kerminal** - Arch Linux (AUR)
- **kerminal\_{version}\_amd64.deb** - Debian/Ubuntu package
- **kerminal-{version}.AppImage** - Portable AppImage
- **kerminal-{version}.tar.gz** - Compressed archive

### Installation Instructions

#### Windows Installation

1. **Using Windows Installer (Recommended)**:

   ```bash
   # Download and run the installer
   ./kerminal-{version}-setup.exe
   ```

   - Double-click the downloaded `.exe` file
   - Follow the installation wizard
   - Kerminal will be added to your Start Menu and Desktop

2. **Using WSL/Debian Package**:

   ```bash
   # Install using dpkg
   sudo dpkg -i kerminal_{version}_amd64.deb

   # Fix dependencies if needed
   sudo apt-get install -f
   ```

#### macOS Installation

1. **Using DMG (Recommended)**:

   ```bash
   # Mount the disk image
   open kerminal-{version}.dmg
   ```

   - Drag Kerminal to your Applications folder
   - Launch from Applications or Spotlight

2. **Using AppImage**:

   ```bash
   # Make executable and run
   chmod +x kerminal-{version}.AppImage
   ./kerminal-{version}.AppImage
   ```

#### Linux Installation

1. **Using AUR (Arch Linux)**:

   ```bash
   # Using yay AUR helper
   yay -S kerminal

   # Using paru AUR helper
   paru -S kerminal

   # Manual installation from AUR
   git clone https://aur.archlinux.org/kerminal.git
   cd kerminal
   makepkg -si
   ```

2. **Using Debian Package (Ubuntu/Debian)**:

   ```bash
   # Install the package
   sudo dpkg -i kerminal_{version}_amd64.deb

   # Install dependencies
   sudo apt-get install -f

   # Launch Kerminal
   kerminal
   ```

3. **Using AppImage (Universal)**:

   ```bash
   # Make executable
   chmod +x kerminal-{version}.AppImage

   # Run directly
   ./kerminal-{version}.AppImage

   # Optional: Move to applications directory
   sudo mv kerminal-{version}.AppImage /usr/local/bin/kerminal
   ```

4. **Using Tar Archive**:

   ```bash
   # Extract the archive
   tar -xzf kerminal-{version}.tar.gz
   cd kerminal-{version}

   # Run the application
   ./kerminal
   ```

### Option 2: Build from Source

For developers or users who want to build from source:

#### Prerequisites

- **Node.js** 22.0.0 or higher
- **Git** for cloning the repository
- **npm** or **yarn** package manager

#### Build Instructions

```bash
# Clone the repository
git clone https://github.com/klpod221/kerminal.git
cd kerminal

# Install dependencies
npm install

# Development mode (hot reload)
npm run dev

# Build for production
npm run build

# Build for specific platforms
npm run build:win    # Windows
npm run build:mac    # macOS
npm run build:linux  # Linux
```

## 🐞 Known Issues

- Error with password authentication when update to lastest from older version (1.0.6 and below). You may need to re-enter your password or recreate your profile.
- Cannot launch on MacOS because MacOS blocks apps from unidentified developers (code signing in progress). You can bypass this by right-clicking the app and selecting "Open", then confirming or launch it from terminal using `xattr -d com.apple.quarantine /path/to/Kerminal.app`. (More info: [Issue #2](https://github.com/klpod221/kerminal/issues/2))
- Not have auto-update feature.
- Not save the ssh key (only save the path to the key)

If you encounter a bug, please create a new issue on [GitHub Issues](https://github.com/klpod221/kerminal/issues).

## 🗺️ Roadmap & Checklist

- [ ] **Electron based with Vue.js front-end**
  - [x] Initial setup with Electron and Vue.js
  - [x] Basic terminal functionality with xterm.js
  - [x] SSH connection support using ssh2
  - [x] Profile management with local storage (JSON file)
  - [x] Basic UI with tabs and split panels
  - [x] Dashboard with recent connections and machine info
  - [x] SSH proxy support (HTTP, SOCKS, Jump Host)
  - [x] SSH command saved commands
  - [x] SSH tunneling (Local, Remote, Dynamic/SOCKS)
  - [x] Cross-device sync using MongoDB
  - [x] Encrypt sensitive data with AES-256
  - [x] Add keyboard shortcuts support
  - [ ] Add syntax highlighting for bash
  - [ ] Add auto-update feature for all platforms
  - [ ] Encrypt and securely store SSH keys instead of just paths
  - [ ] Add more keyboard shortcuts and allow custom mappings
  - [ ] Add more themes and customization options
  - [ ] Support 2FA/MFA for SSH connections
  - [ ] Implement session recording and playback
  - [ ] Add support for SFTP file transfers
  - [ ] Implement advanced search across profiles and commands
  - [ ] Fix password authentication issue when updating from versions 1.0.6 and below
  - [ ] Implement code signing for macOS to avoid unidentified developer issues

- [ ] **Rewrite backend in Rust**
  - [ ] Rewrite backend logic in Rust for performance and security
  - [ ] Create Node.js bindings to interface with Rust backend
  - [ ] Ensure all existing features work seamlessly with Rust backend
  - [ ] Optimize SSH and tunneling performance using Rust libraries
  - [ ] Test cross-platform compatibility with Rust backend
  - [ ] Update build and deployment processes for Rust integration

- [ ] **Allow front-end to run in browser (self-hosted)**
  - [ ] Choose a suitable framework for browser compatibility
  - [ ] Implement WebAssembly (WASM) for performance-critical parts
  - [ ] Ensure secure handling of SSH connections in the browser
  - [ ] Test browser compatibility across major browsers
  - [ ] Optimize UI/UX for web usage
  - [ ] Implement cloud sync and storage options for web users
  - [ ] Ensure seamless integration with Rust backend via WebAssembly

## Contributors ✨

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

Please read our [Contributing Guidelines](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.

### How to Contribute

1. **Find an Issue:** Ensure the bug or feature you want to work on is not already being addressed at [issues](https://github.com/klpod221/kerminal/issues).
2. **Fork the Project:** Create your own copy of the repository.
3. **Create a Feature Branch:** `git checkout -b feature/AmazingFeature`
4. **Commit your Changes:** `git commit -m 'Add some AmazingFeature'`
5. **Push to the Branch:** `git push origin feature/AmazingFeature`
6. **Open a Pull Request:** Submit your changes for review.

### 🙏 Our Valued Contributors

A huge thank you to all the wonderful people who have contributed to this project:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Abhishekrajpurohit"><img src="https://avatars.githubusercontent.com/u/71376117?v=4?s=100" width="100px;" alt="Abhishek Rajpurohit"/><br /><sub><b>Abhishek Rajpurohit</b></sub></a><br /><a href="https://github.com/klpod221/kerminal/commits?author=Abhishekrajpurohit" title="Code">💻</a> <a href="https://github.com/klpod221/kerminal/issues?q=author%3AAbhishekrajpurohit" title="Bug reports">🐛</a></td>
    </tr>
  </tbody>
</table>
<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->
<!-- ALL-CONTRIBUTORS-LIST:END -->

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👨‍💻 Author

**klpod221** (Bùi Thanh Xuân)

- Website: [klpod221.com](https://klpod221.com)
- GitHub: [@klpod221](https://github.com/klpod221)
- Email: [klpod221@gmail.com](mailto:klpod221@gmail.com)

## 📊 Project Status

Kerminal is actively maintained and under continuous development.

> **Please Note:** Currently, this is a side project maintained solely by me. Therefore, progress may be inconsistent and is highly dependent on my personal schedule.

Check the [Releases page](https://github.com/klpod221/kerminal/releases) for the latest version.

---

<div align="center">
    <p>Made with ❤️ by <a href="https://github.com/klpod221">klpod221</a></p>
    <p>If you find Kerminal useful, please consider giving it a ⭐ on GitHub!</p>
</div>

---

_This README was crafted with assistance from an AI model._
