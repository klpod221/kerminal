---
layout: home
title: Kerminal - Modern Terminal Emulator & SSH Manager
titleTemplate: false

hero:
  name: Kerminal
  text: Modern Terminal Emulator & SSH Manager
  tagline: Terminal emulator mạnh mẽ với quản lý SSH nâng cao, ghi và phát lại session, đồng bộ đa thiết bị và mã hóa cấp doanh nghiệp.
  image:
    src: /logo.png
    alt: Kerminal
  actions:
    - theme: brand
      text: Bắt đầu
      link: /vi/guide/getting-started
    - theme: alt
      text: Xem trên GitHub
      link: https://github.com/klpod221/kerminal

features:
  - icon: 💻
    title: Terminal Emulator
    details: Đa màn hình (tabs, split panes), WebGL, hỗ trợ ảnh inline (Sixel), và Command Palette.
  - icon: 📡
    title: Quản lý SSH & Tunneling
    details: Tổ chức profile, quản lý khóa SSH, hỗ trợ Proxy, Jump Host Chains, và Port forwarding.
  - icon: 💾
    title: Session & Files
    details: Ghi hình session định dạng asciicast, thư viện lưu lệnh, và chuyển file SFTP.
  - icon: 🔄
    title: Đồng bộ đa thiết bị
    details: Sync qua MySQL/PostgreSQL/MongoDB, mã hóa AES-256-GCM.
  - icon: 🔒
    title: Bảo mật tối đa
    details: Bảo vệ bằng master password, khóa thiết bị, tích hợp OS keychain, và auto-lock.
  - icon: 🎨
    title: Giao diện hiện đại
    details: Tùy chỉnh theme, font chữ, Dark mode gốc, phím tắt linh hoạt, và Dashboard.
---

## 📸 Ảnh màn hình

### Dashboard
![Dashboard](/screenshots/Dashboard.png)

### Giao diện chính
![Main Interface](/screenshots/MainInterface.png)

### Demo
<video controls autoplay loop muted style="width: 100%; border-radius: 8px; margin-top: 16px;">
  <source src="/screencast/basic.webm" type="video/webm">
  Trình duyệt của bạn không hỗ trợ thẻ video.
</video>

## 📥 Sẵn sàng bắt đầu?

Tải xuống Kerminal cho hệ điều hành của bạn.

### Tải nhanh

- **🐧 Linux**: [AppImage, deb, rpm](https://github.com/klpod221/kerminal/releases/latest)
- **🪟 Windows**: [exe, msi installer](https://github.com/klpod221/kerminal/releases/latest)
- **🍎 macOS**: [dmg (unsigned)](https://github.com/klpod221/kerminal/releases/latest)

::: warning Người dùng macOS
Ứng dụng chưa được ký (unsigned). Chạy lệnh sau sau khi tải về:
```bash
xattr -rd com.apple.quarantine /path/to/Kerminal.app
```
[Tìm hiểu thêm](https://github.com/klpod221/kerminal#-known-issues)
:::

### 🛠️ Cài đặt khác
#### 📦 Flatpak (Linux)

Tải file `.flatpak` từ trang [Releases](https://github.com/klpod221/kerminal/releases/latest) và chạy lệnh:
```bash
flatpak install /path/to/kerminal_*.flatpak
```

#### 🐧 Arch Linux (AUR)

```bash
yay -S kerminal
# hoặc kerminal-bin cho bản binary
```

#### ⚙️ Build từ mã nguồn

[Xem hướng dẫn đầy đủ](/vi/guide/development)

```bash
git clone https://github.com/klpod221/kerminal.git
cd kerminal && npm install
npm run tauri build
```
