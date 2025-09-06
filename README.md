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

## 🔧 Development

### Project Structure

```text
kerminal/
├── build/                          # Build resources and assets
│   ├── entitlements.mac.plist     # macOS entitlements for code signing
│   ├── icon.icns                  # macOS application icon
│   ├── icon.ico                   # Windows application icon
│   └── icon.png                   # Application icon (PNG format)
│
├── resources/                      # Static application resources
│   └── icon.png                   # Application icon resource
│
├── src/                           # Main source code directory
│   ├── main/                      # Electron main process (Backend)
│   │   ├── app.ts                 # Main application entry point
│   │   ├── index.ts               # Electron main process initialization
│   │   ├── ipc-handlers.ts        # IPC communication handlers
│   │   │
│   │   ├── base/                  # Base classes and abstractions
│   │   │   └── base-service.ts    # Base service class for all services
│   │   │
│   │   ├── interfaces/            # TypeScript interfaces and contracts
│   │   │   ├── application.interface.ts    # Application-wide interfaces
│   │   │   ├── ssh.interface.ts           # SSH-related interfaces
│   │   │   ├── sync.interface.ts          # Synchronization interfaces
│   │   │   ├── syncable-storage.interface.ts  # Storage sync interfaces
│   │   │   └── terminal.interface.ts      # Terminal-related interfaces
│   │   │
│   │   ├── services/              # Core business logic services
│   │   │   ├── mongodb-service.ts         # MongoDB connection service
│   │   │   ├── saved-command-service.ts   # Saved commands management
│   │   │   ├── ssh-connection-service.ts  # SSH connection handling
│   │   │   ├── ssh-connection.ts          # SSH connection implementation
│   │   │   ├── ssh-profile-service.ts     # SSH profile management
│   │   │   ├── ssh-tunnel-service.ts      # SSH tunneling service
│   │   │   ├── sync-manager.ts            # Data synchronization manager
│   │   │   ├── sync-service.ts            # Synchronization service
│   │   │   ├── system-info.ts             # System information service
│   │   │   ├── terminal-buffer-manager.ts # Terminal buffer management
│   │   │   ├── terminal-manager.ts        # Terminal session management
│   │   │   └── window-manager.ts          # Application window management
│   │   │
│   │   ├── storage/               # Data persistence layer
│   │   │   ├── base-storage.ts            # Base storage class
│   │   │   ├── saved-command-storage.ts   # Saved commands storage
│   │   │   ├── ssh-connection-storage.ts  # SSH connections storage
│   │   │   ├── ssh-group-storage.ts       # SSH groups storage
│   │   │   ├── ssh-profile-storage.ts     # SSH profiles storage
│   │   │   ├── ssh-tunnel-storage.ts      # SSH tunnels storage
│   │   │   └── sync-config-storage.ts     # Sync configuration storage
│   │   │
│   │   ├── types/                 # TypeScript type definitions
│   │   │   ├── main.ts            # Main process type definitions
│   │   │   └── ssh.ts             # SSH-related type definitions
│   │   │
│   │   ├── utils/                 # Main process utilities
│   │   │   └── logger.ts          # Logging utility
│   │   │
│   │   └── validators/            # Data validation modules
│   │       └── ssh-config-validator.ts    # SSH configuration validator
│   │
│   ├── preload/                   # Electron preload scripts (Security layer)
│   │   ├── index.d.ts             # Preload type definitions
│   │   └── index.ts               # Main preload script for IPC bridge
│   │
│   ├── renderer/                  # Frontend application (Vue.js)
│   │   ├── index.html             # Main HTML template
│   │   └── src/
│   │       ├── App.vue            # Root Vue component
│   │       ├── main.ts            # Vue application entry point
│   │       ├── env.d.ts           # Environment type definitions
│   │       │
│   │       ├── assets/            # Static frontend assets
│   │       │   ├── fonts/         # Custom fonts
│   │       │   ├── images/        # Images and icons
│   │       │   └── styles/        # CSS/SCSS stylesheets
│   │       │
│   │       ├── components/        # Vue components
│   │       │   ├── Dashboard.vue              # Main dashboard component
│   │       │   ├── KeyboardShortcutsModal.vue # Keyboard shortcuts modal
│   │       │   ├── Panel.vue                  # Split panel component
│   │       │   ├── PanelManager.vue           # Panel management component
│   │       │   ├── SavedCommandDrawer.vue     # Saved commands drawer
│   │       │   ├── SavedCommandModal.vue      # Saved command modal
│   │       │   ├── SSHGroupModal.vue          # SSH group modal
│   │       │   ├── SSHProfileDrawer.vue       # SSH profile drawer
│   │       │   ├── SSHProfileModal.vue        # SSH profile modal
│   │       │   ├── SSHTunnelManager.vue       # SSH tunnel manager
│   │       │   ├── SSHTunnelModal.vue         # SSH tunnel modal
│   │       │   ├── SyncSettingsModal.vue      # Sync settings modal
│   │       │   ├── Terminal.vue               # Terminal component
│   │       │   ├── TerminalManager.vue        # Terminal manager component
│   │       │   ├── TopBar.vue                 # Top navigation bar
│   │       │   └── ui/                        # Reusable UI components
│   │       │
│   │       ├── composables/       # Vue composition functions
│   │       │   ├── useTopBarState.ts  # Top bar state management
│   │       │   └── useValidation.ts   # Form validation composable
│   │       │
│   │       ├── services/          # Frontend services
│   │       │   ├── keyboard-shortcut-service.ts   # Keyboard shortcuts
│   │       │   └── terminal-buffer-manager.ts     # Terminal buffer management
│   │       │
│   │       ├── types/             # Frontend type definitions
│   │       │   ├── components.ts      # Component-related types
│   │       │   ├── keyboard.ts        # Keyboard-related types
│   │       │   ├── modals.ts          # Modal-related types
│   │       │   ├── panel.ts           # Panel-related types
│   │       │   ├── splitpanes.d.ts    # Split panes type definitions
│   │       │   ├── ssh.ts             # SSH-related types
│   │       │   ├── sync.ts            # Synchronization types
│   │       │   ├── system.ts          # System-related types
│   │       │   └── ui.ts              # UI-related types
│   │       │
│   │       └── utils/             # Frontend utilities
│   │           ├── clipboard.ts       # Clipboard operations
│   │           ├── debounce.ts        # Debounce utility
│   │           ├── formatter.ts       # Data formatting utilities
│   │           ├── message.ts         # Message handling utility
│   │           └── ...                # Additional utility files
│   │
│   └── shared/                    # Shared code between main and renderer
│       ├── index.ts               # Shared exports
│       └── types/                 # Shared type definitions
│           ├── application.ts         # Application-wide types
│           ├── index.ts               # Type exports
│           ├── ssh.ts                 # SSH-related shared types
│           ├── sync.ts                # Synchronization shared types
│           └── terminal.ts            # Terminal-related shared types
│
├── dev-app-update.yml             # Auto-updater configuration for development
├── electron-builder.yml           # Electron builder configuration
├── electron.vite.config.ts        # Vite configuration for Electron
├── eslint.config.mjs              # ESLint configuration
├── package.json                   # Project dependencies and scripts
├── tsconfig.json                  # TypeScript configuration (main)
├── tsconfig.node.json             # TypeScript configuration (Node.js)
├── tsconfig.web.json              # TypeScript configuration (Web/Renderer)
├── LICENSE                        # MIT License
└── README.md                      # Project documentation
```

#### Architecture Overview

**Electron Multi-Process Architecture:**

- **Main Process**: Manages application lifecycle, system integration, and core services
- **Renderer Process**: Handles the user interface using Vue.js
- **Preload Scripts**: Secure bridge between main and renderer processes

**Key Design Patterns:**

- **Service-Oriented Architecture**: Modular services for different functionalities
- **Storage Abstraction**: Base storage class with specific implementations
- **Interface-Driven Development**: Clear contracts between modules
- **Composable Architecture**: Vue 3 composition API for reactive state management

### Tech Stack

- **Frontend**: Vue.js 3, Tailwind CSS, Lucide Icons
- **Backend**: Electron, Node.js, TypeScript
- **Terminal**: xterm.js with various addons
- **SSH**: ssh2 library for secure connections with proxy support
- **SSH Tunneling**: Comprehensive port forwarding (Local, Remote, Dynamic/SOCKS)
- **Proxy Support**: HTTP, SOCKS4, SOCKS5, and SSH Jump Host proxies
- **Storage**: Secure local file-based storage with optional MongoDB synchronization
- **Database**: MongoDB for cross-device data synchronization

## 🤝 Contributing

I appreciate your interest in contributing to Kerminal!

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and test thoroughly
4. Commit your changes: `git commit -m 'Add amazing feature'`
5. Push to the branch: `git push origin feature/amazing-feature`
6. Open a Pull Request

### Code Style

- Follow the existing code style
- Use TypeScript for type safety
- Add tests for new features
- Update documentation as needed

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👨‍💻 Author

**klpod221** (Bùi Thanh Xuân)

- Website: [klpod221.com](https://klpod221.com)
- GitHub: [@klpod221](https://github.com/klpod221)
- Email: [klpod221@gmail.com](mailto:klpod221@gmail.com)

## 🙏 Acknowledgments

- [xterm.js](https://xtermjs.org/) - For the excellent terminal emulation
- [ssh2](https://github.com/mscdex/ssh2) - For SSH connectivity
- [Electron](https://electronjs.org/) - For cross-platform desktop apps
- [Vue.js](https://vuejs.org/) - For the reactive frontend framework
- [MongoDB](https://www.mongodb.com/) - For reliable data synchronization across devices

## 📊 Project Status

Kerminal is actively maintained and under continuous development. We regularly add new features and improvements based on user feedback.

Check the [Releases page](https://github.com/klpod221/kerminal/releases) for the latest version.

---

<div align="center">
    <p>Made with ❤️ by <a href="https://github.com/klpod221">klpod221</a></p>
    <p>If you find Kerminal useful, please consider giving it a ⭐ on GitHub!</p>
</div>

---

_This README was crafted with assistance from an AI model._
