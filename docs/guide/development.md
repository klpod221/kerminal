# Development Guide

This guide covers setting up a development environment for contributing to Kerminal.

## Prerequisites

Ensure you have the following installed:

| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | 20+ | Frontend development |
| Rust | Latest stable | Backend (Tauri) |
| Tauri CLI | Latest | Build tooling |

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Install Tauri CLI

```bash
cargo install tauri-cli
```

### Platform-Specific Dependencies

#### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

#### Linux (Arch Linux)

```bash
sudo pacman -S webkit2gtk \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    appmenu-gtk-module \
    libappindicator-gtk3 \
    librsvg
```

#### Linux (Fedora)

```bash
sudo dnf install webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libxdo-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

#### macOS

```bash
xcode-select --install
```

#### Windows

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload.

## Getting Started

### Clone Repository

```bash
git clone https://github.com/klpod221/kerminal.git
cd kerminal
```

### Install Dependencies

```bash
npm install
```

### Development Mode

```bash
npm run tauri dev
```

This will:
1. Start the Vite dev server (hot reload)
2. Compile the Rust backend
3. Launch the Tauri window

### Production Build

```bash
npm run tauri build
```

Output will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
kerminal/
├── src/                    # Vue 3 frontend
│   ├── components/         # Vue components
│   ├── stores/             # Pinia stores
│   ├── views/              # Page views
│   ├── composables/        # Vue composables
│   └── assets/             # Static assets
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   ├── models/         # Data models
│   │   ├── services/       # Business logic
│   │   └── main.rs         # Entry point
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── docs/                   # VitePress documentation
├── public/                 # Public assets
└── package.json            # Node.js dependencies
```

## Key Technologies

### Frontend

| Technology | Purpose |
|------------|---------|
| Vue 3 | UI framework |
| TypeScript | Type safety |
| Pinia | State management |
| xterm.js | Terminal emulation |
| TailwindCSS | Styling |

### Backend

| Technology | Purpose |
|------------|---------|
| Tauri v2 | Desktop framework |
| Tokio | Async runtime |
| russh | SSH implementation |
| SQLx | Database access |
| AES-GCM | Encryption |

## Code Style

### Frontend (TypeScript/Vue)

- Use Composition API with `<script setup>`
- Follow Vue 3 style guide
- Use Prettier for formatting

```bash
npm run pretty
```

### Backend (Rust)

- Follow Rust idioms
- Use `cargo fmt` for formatting
- Run `cargo clippy` for lints

```bash
cargo fmt
cargo clippy
```

## Testing

### Frontend Tests

```bash
# Coming soon
npm run test
```

### Backend Tests

```bash
cd src-tauri
cargo test
```

## Contributing

### Workflow

1. Fork the repository
2. Create a feature branch
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. Make your changes
4. Run tests and linting
5. Commit with descriptive message
   ```bash
   git commit -m 'Add amazing feature'
   ```
6. Push to your fork
   ```bash
   git push origin feature/amazing-feature
   ```
7. Open a Pull Request

### Commit Messages

Follow conventional commits:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `style:` Formatting
- `refactor:` Code refactoring
- `test:` Tests
- `chore:` Maintenance

### Pull Request Guidelines

1. Clear description of changes
2. Link related issues
3. Include screenshots for UI changes
4. Update documentation if needed
5. Ensure all checks pass

## Debugging

### Frontend

Use Vue DevTools browser extension for Vue debugging.

### Backend

Enable debug logging:

```bash
RUST_LOG=debug npm run tauri dev
```

### Network

For SSH debugging, enable verbose SSH output in development builds.

## Building for Release

### All Platforms

```bash
npm run tauri build
```

### Specific Platform

```bash
# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu

# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS (Intel)
npm run tauri build -- --target x86_64-apple-darwin

# macOS (Apple Silicon)
npm run tauri build -- --target aarch64-apple-darwin
```

## Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Vue 3 Documentation](https://vuejs.org/)
- [xterm.js Documentation](https://xtermjs.org/)
- [russh Documentation](https://docs.rs/russh/latest/russh/)
