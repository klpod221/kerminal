<div align="center">
    <h1>Kerminal: Terminal with SSH Profile Support</h1>
    <p>A modern, feature-rich terminal application built with Electron and Vue.js, offering seamless SSH connections and command management.</p>
    <img src="https://img.shields.io/github/last-commit/klpod221/kerminal?style=for-the-badge&color=74c7ec&labelColor=111827" alt="Last Commit">
    <img src="https://img.shields.io/github/stars/klpod221/kerminal?style=for-the-badge&color=facc15&labelColor=111827" alt="GitHub Stars">
    <img src="https://img.shields.io/github/repo-size/klpod221/kerminal?style=for-the-badge&color=a78bfa&labelColor=111827" alt="Repo Size">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge&color=34d399&labelColor=111827" alt="License">
</div>

## 📝 Description

**Kerminal** is a modern terminal application that combines the power of a traditional terminal with advanced SSH connection management, command automation, and cross-device synchronization features. Built with Electron and Vue.js, it provides a sleek, user-friendly interface for developers and system administrators who work with multiple remote servers.

Key highlights include secure SSH profile management, saved command functionality, MongoDB-powered cross-device synchronization, and a beautiful dark-themed interface optimized for productivity. Whether you're managing cloud infrastructure, developing on remote servers, or collaborating across multiple devices, Kerminal streamlines your workflow and keeps your configurations synchronized.

## ✨ Features

### 🖥️ **Modern Terminal Experience**

- **Multi-tab Support**: Work with multiple terminal sessions simultaneously
- **Beautiful Dark Theme**: Eye-friendly interface optimized for long coding sessions
- **Advanced Terminal Emulation**: Powered by xterm.js with full feature support
- **Cross-platform**: Available for Windows, macOS, and Linux

### 🔐 **SSH Connection Management**

- **SSH Profiles**: Create and manage multiple SSH connection profiles
- **Group Organization**: Organize your SSH connections into logical groups
- **Secure Storage**: Encrypted storage of SSH credentials and configurations
- **Quick Connect**: One-click connection to your favorite servers

### ⚡ **Command Automation**

- **Saved Commands**: Store and quickly execute frequently used commands
- **Command History**: Access your command history across sessions
- **Custom Scripts**: Execute complex command sequences with a single click

### ☁️ **Data Synchronization**

- **MongoDB Sync**: Synchronize your SSH profiles and saved commands across devices
- **Real-time Sync**: Automatic synchronization with configurable intervals (5-3600 seconds)
- **Conflict Resolution**: Intelligent merging of changes from multiple devices
- **Data Migration**: Seamless migration from local storage to cloud storage
- **Cross-device Access**: Access your configurations from any device with Kerminal

### 🛠️ **Developer-Friendly**

- **Auto-updates**: Seamless application updates via electron-updater
- **Customizable Interface**: Tailored experience for different workflows
- **Export/Import**: Backup and share your configurations
- **Search Functionality**: Quickly find connections and commands

## 🚀 Getting Started

### Prerequisites

- **Node.js** 20
- **Git** for cloning the repository

### Installation

#### Option 1: Download Release (Recommended)

Download the latest release from the [Releases page](https://github.com/klpod221/kerminal/releases).

#### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/klpod221/kerminal.git
cd kerminal

# Install dependencies
npm install

# Development mode
npm run dev

# Build for production
npm run build

# Build for specific platforms
npm run build:win    # Windows
npm run build:mac    # macOS
npm run build:linux  # Linux
```

## ⚙️ Configuration

### MongoDB Sync Setup

Kerminal supports optional MongoDB synchronization to keep your SSH profiles and saved commands in sync across multiple devices.

#### MongoDB Requirements

- MongoDB Atlas account (recommended) or self-hosted MongoDB instance
- MongoDB connection URI with read/write permissions

#### Setup Steps

1. **Open Sync Settings**: Click the sync icon in the top bar
2. **Configure Connection**:
   - Enter your MongoDB URI (e.g., `mongodb+srv://user:pass@cluster.mongodb.net`)
   - Set the database name (default: `kerminal`)
   - Configure sync interval (5-3600 seconds, default: 30 seconds)
3. **Test Connection**: Use the "Test Connection" button to verify your setup
4. **Save Configuration**: Click "Save Configuration" to enable sync

#### MongoDB URI Examples

```bash
# MongoDB Atlas
mongodb+srv://username:password@cluster0.example.mongodb.net

# Self-hosted MongoDB
mongodb://username:password@localhost:27017

# MongoDB with additional options
mongodb://user:pass@host:port/database?authSource=admin
```

#### Data Migration

If you have existing local data, Kerminal will automatically offer to migrate it to MongoDB when you first enable sync.

## 🔧 Development

### Project Structure

```text
src/
├── main/           # Electron main process
│   ├── services/   # Core services (SSH, Terminal, Storage)
│   ├── storage/    # Data persistence layer
│   └── utils/      # Utilities and helpers
├── preload/        # Electron preload scripts
└── renderer/       # Vue.js frontend
    └── src/
        ├── components/  # Vue components
        ├── composables/ # Vue composition functions
        └── utils/       # Frontend utilities
```

### Tech Stack

- **Frontend**: Vue.js 3, Tailwind CSS, Lucide Icons
- **Backend**: Electron, Node.js, TypeScript
- **Terminal**: xterm.js with various addons
- **SSH**: ssh2 library for secure connections
- **Storage**: Secure local file-based storage with optional MongoDB synchronization
- **Database**: MongoDB for cross-device data synchronization

### Scripts

```bash
npm run dev         # Start development server
npm run build       # Build for production
npm run lint        # Run linter
npm run format      # Format code with Prettier
npm run typecheck   # Type checking
```

## 📦 Building

Kerminal uses electron-builder for creating distributable packages:

```bash
# Build for all platforms
npm run build:win
npm run build:mac
npm run build:linux

# Create unpacked directory (for testing)
npm run build:unpack
```

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

Current version: **v0.4.0**

Kerminal is actively maintained and under continuous development. We regularly add new features and improvements based on user feedback.

---

<div align="center">
    <p>Made with ❤️ by <a href="https://github.com/klpod221">klpod221</a></p>
    <p>If you find Kerminal useful, please consider giving it a ⭐ on GitHub!</p>
</div>
