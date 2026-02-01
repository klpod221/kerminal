# Installation

Kerminal is available for **Linux**, **Windows**, and **macOS**.

## Linux

### Arch Linux (AUR)

The easiest way to install on Arch Linux is through the AUR:

**Using an AUR helper (recommended):**
```bash
# Install binary package (faster)
yay -S kerminal-bin

# Or build from source
yay -S kerminal
```

**Manual installation:**
```bash
git clone https://aur.archlinux.org/kerminal-bin.git
cd kerminal-bin
makepkg -si
```

### Debian/Ubuntu

Download the `.deb` package from the [releases page](https://github.com/klpod221/kerminal/releases/latest):

```bash
# Download the latest .deb package
wget https://github.com/klpod221/kerminal/releases/latest/download/kerminal_x.x.x_amd64.deb

# Install
sudo dpkg -i kerminal_*.deb

# Fix any dependency issues
sudo apt-get install -f
```

### Fedora/RHEL

Download the `.rpm` package from the [releases page](https://github.com/klpod221/kerminal/releases/latest):

```bash
# Download the latest .rpm package
wget https://github.com/klpod221/kerminal/releases/latest/download/kerminal-x.x.x-1.x86_64.rpm

# Install
sudo rpm -i kerminal-*.rpm
```

### AppImage

For any Linux distribution:

```bash
# Download the AppImage
wget https://github.com/klpod221/kerminal/releases/latest/download/kerminal_x.x.x_amd64.AppImage

# Make it executable
chmod +x kerminal_*.AppImage

# Run
./kerminal_*.AppImage
```

## Windows

### Installer

1. Download the latest `.msi` or `.exe` installer from the [releases page](https://github.com/klpod221/kerminal/releases/latest)
2. Run the installer
3. Follow the installation wizard
4. Launch Kerminal from the Start menu

### Portable

1. Download the portable `.zip` from the [releases page](https://github.com/klpod221/kerminal/releases/latest)
2. Extract to your preferred location
3. Run `Kerminal.exe`

## macOS

::: warning Important Note
The macOS version is **not signed/notarized** due to Apple Developer Program restrictions ($99/year). You'll need to bypass Gatekeeper to run the app.
:::

### Installation

1. Download the latest `.dmg` from the [releases page](https://github.com/klpod221/kerminal/releases/latest)
2. Open the `.dmg` file
3. Drag Kerminal to the Applications folder

### Bypass Gatekeeper

After first launch attempt, run this command in Terminal:

```bash
xattr -rd com.apple.quarantine /Applications/Kerminal.app
```

Then try launching Kerminal again.

### Build from Source

If you prefer, you can build from source:

```bash
git clone https://github.com/klpod221/kerminal.git
cd kerminal
npm install
npm run tauri build
```

## Verifying Downloads

You can verify the integrity of downloaded files using the checksums provided in each release.

```bash
# Example for Linux
sha256sum -c kerminal_x.x.x_amd64.deb.sha256
```

## System Requirements

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| OS | Windows 10+, macOS 11+, Linux (kernel 5.0+) |
| RAM | 256 MB |
| Storage | 100 MB |
| Display | 1024x768 resolution |

### Recommended

| Component | Recommendation |
|-----------|----------------|
| RAM | 512 MB+ |
| Storage | SSD recommended |
| Display | 1920x1080+ |
| GPU | WebGL 2.0 capable |

## Next Steps

After installation, check out:

- [Getting Started](/guide/getting-started) - Introduction to Kerminal
- [Features](/guide/features) - Explore all features
- [SSH Management](/guide/ssh-management) - Set up your first SSH connection
