# Hướng dẫn phát triển

Hướng dẫn này bao gồm việc thiết lập môi trường phát triển để đóng góp cho Kerminal.

## Yêu cầu

Đảm bảo bạn đã cài đặt những thứ sau:

| Công cụ | Phiên bản | Mục đích |
|---------|-----------|----------|
| Node.js | 20+ | Phát triển frontend |
| Rust | Stable mới nhất | Backend (Tauri) |
| Tauri CLI | Mới nhất | Công cụ build |

### Cài đặt Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Cài đặt Tauri CLI

```bash
cargo install tauri-cli
```

### Dependencies theo nền tảng

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

Cài đặt [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) với workload C++.

## Bắt đầu

### Clone Repository

```bash
git clone https://github.com/klpod221/kerminal.git
cd kerminal
```

### Cài đặt Dependencies

```bash
npm install
```

### Chế độ phát triển

```bash
npm run tauri dev
```

Lệnh này sẽ:
1. Khởi động Vite dev server (hot reload)
2. Compile backend Rust
3. Mở cửa sổ Tauri

### Build production

```bash
npm run tauri build
```

Output sẽ ở `src-tauri/target/release/bundle/`.

## Cấu trúc dự án

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
│   └── tauri.conf.json     # Cấu hình Tauri
├── docs/                   # VitePress documentation
├── public/                 # Public assets
└── package.json            # Node.js dependencies
```

## Công nghệ chính

### Frontend

| Công nghệ | Mục đích |
|-----------|----------|
| Vue 3 | UI framework |
| TypeScript | Type safety |
| Pinia | State management |
| xterm.js | Terminal emulation |
| TailwindCSS | Styling |

### Backend

| Công nghệ | Mục đích |
|-----------|----------|
| Tauri v2 | Desktop framework |
| Tokio | Async runtime |
| russh | SSH implementation |
| SQLx | Database access |
| AES-GCM | Encryption |

## Code Style

### Frontend (TypeScript/Vue)

- Sử dụng Composition API với `<script setup>`
- Tuân theo Vue 3 style guide
- Sử dụng Prettier để format

```bash
npm run pretty
```

### Backend (Rust)

- Tuân theo Rust idioms
- Sử dụng `cargo fmt` để format
- Chạy `cargo clippy` để kiểm tra lints

```bash
cargo fmt
cargo clippy
```

## Testing

### Frontend Tests

```bash
# Sắp ra mắt
npm run test
```

### Backend Tests

```bash
cd src-tauri
cargo test
```

## Đóng góp

### Workflow

1. Fork repository
2. Tạo feature branch
   ```bash
   git checkout -b feature/tinh-nang-moi
   ```
3. Thực hiện thay đổi
4. Chạy tests và linting
5. Commit với message mô tả
   ```bash
   git commit -m 'Add tinh nang moi'
   ```
6. Push lên fork của bạn
   ```bash
   git push origin feature/tinh-nang-moi
   ```
7. Mở Pull Request

### Commit Messages

Tuân theo conventional commits:

- `feat:` Tính năng mới
- `fix:` Sửa lỗi
- `docs:` Tài liệu
- `style:` Định dạng
- `refactor:` Tái cấu trúc code
- `test:` Tests
- `chore:` Bảo trì

### Hướng dẫn Pull Request

1. Mô tả rõ ràng các thay đổi
2. Liên kết các issue liên quan
3. Bao gồm screenshot cho thay đổi UI
4. Cập nhật tài liệu nếu cần
5. Đảm bảo tất cả checks đều pass

## Debugging

### Frontend

Sử dụng Vue DevTools browser extension để debug Vue.

### Backend

Bật debug logging:

```bash
RUST_LOG=debug npm run tauri dev
```

### Network

Để debug SSH, bật verbose SSH output trong development builds.

## Build cho Release

### Tất cả nền tảng

```bash
npm run tauri build
```

### Nền tảng cụ thể

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

## Tài nguyên

- [Tài liệu Tauri](https://tauri.app/v1/guides/)
- [Tài liệu Vue 3](https://vuejs.org/)
- [Tài liệu xterm.js](https://xtermjs.org/)
- [Tài liệu russh](https://docs.rs/russh/latest/russh/)
