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
    details: Hỗ trợ nhiều tab và chia màn hình, tích hợp shell gốc, render tăng tốc WebGL với hỗ trợ Unicode 11.
  - icon: 📡
    title: Quản lý SSH
    details: Tổ chức profile theo nhóm và màu sắc, xác thực bằng key, hỗ trợ proxy và jump host.
  - icon: 💾
    title: Ghi Session
    details: Ghi session theo định dạng asciicast với điều khiển phát lại. Thư viện lệnh với thay thế biến.
  - icon: 🔄
    title: Đồng bộ đa thiết bị
    details: Sync qua MySQL/PostgreSQL/MongoDB với mã hóa AES-256-GCM. Giải quyết xung đột và tự động đồng bộ.
  - icon: 🔒
    title: Bảo mật
    details: Bảo vệ bằng master password, khóa riêng cho từng thiết bị, tích hợp keychain và tự động khóa.
  - icon: 🎨
    title: Giao diện hiện đại
    details: Theme tối đẹp mắt, phím tắt, tùy chỉnh màu sắc và hiển thị trạng thái thời gian thực.
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
