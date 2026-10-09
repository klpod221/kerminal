---
layout: home
title: Kerminal - Modern Terminal Emulator & SSH Manager
titleTemplate: false

hero:
  name: Kerminal
  text: Modern Terminal Emulator & SSH Manager
  tagline: A powerful, feature-rich terminal emulator with advanced SSH management, session recording & playback, multi-device sync, and enterprise-grade encryption.
  image:
    src: /logo.png
    alt: Kerminal
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/klpod221/kerminal

features:
  - icon: 💻
    title: Terminal Emulator
    details: Multiple tabs, WebGL-accelerated rendering, Sixel image support, clickable links, and built-in Command Palette.
  - icon: 📡
    title: SSH Management
    details: Profile organization, SSH key manager, proxy support, Jump Host Chains, and port forwarding (Local/Remote/Dynamic).
  - icon: 💾
    title: Session & File Management
    details: Record sessions in asciicast format, saved commands with syntax highlighting, and SFTP file transfer.
  - icon: 🔄
    title: Multi-Device Sync
    details: Sync via MySQL/PostgreSQL/MongoDB with AES-256-GCM encryption. Conflict resolution and auto-sync.
  - icon: 🔒
    title: Security First
    details: Master password protection, device-specific keys, OS keychain integration, and session auto-lock.
  - icon: 🎨
    title: Modern UI
    details: Customizable themes and fonts, native dark mode, keyboard shortcuts, and dashboard interface.
---

## 📸 Screenshots

### Dashboard
![Dashboard](/screenshots/Dashboard.png)

### Main Interface
![Main Interface](/screenshots/MainInterface.png)

### Demo
<video controls autoplay loop muted style="width: 100%; border-radius: 8px; margin-top: 16px;">
  <source src="/screencast/basic.webm" type="video/webm">
  Your browser does not support the video tag.
</video>

## 📥 Ready to Get Started?

Download Kerminal for your operating system.

### Quick Download

- **🐧 Linux**: [AppImage, deb, rpm](https://github.com/klpod221/kerminal/releases/latest)
- **🪟 Windows**: [exe, msi installer](https://github.com/klpod221/kerminal/releases/latest)
- **🍎 macOS**: [dmg (unsigned)](https://github.com/klpod221/kerminal/releases/latest)

::: warning macOS Users
App is unsigned. Run the following command after download:
```bash
xattr -rd com.apple.quarantine /path/to/Kerminal.app
```
[Learn more](https://github.com/klpod221/kerminal#-known-issues)
:::

### 🛠️ Alternative Installation
#### 📦 Flatpak (Linux)

Download the `.flatpak` file from the [Releases](https://github.com/klpod221/kerminal/releases/latest) page and run:
```bash
flatpak install /path/to/kerminal_*.flatpak
```

#### 🐧 Arch Linux (AUR)

```bash
yay -S kerminal
# or kerminal-bin for binary
```

#### ⚙️ Build from Source

[View full guide](/guide/development)

```bash
git clone https://github.com/klpod221/kerminal.git
cd kerminal && npm install
npm run tauri build
```
