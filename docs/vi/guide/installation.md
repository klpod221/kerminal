# Cài đặt

Kerminal có sẵn cho **Linux**, **Windows** và **macOS**.

## Linux

### Arch Linux (AUR)

Cách dễ nhất để cài đặt trên Arch Linux là thông qua AUR:

**Sử dụng AUR helper (khuyến nghị):**
```bash
# Cài đặt gói binary (nhanh hơn)
yay -S kerminal-bin

# Hoặc build từ source
yay -S kerminal
```

**Cài đặt thủ công:**
```bash
git clone https://aur.archlinux.org/kerminal-bin.git
cd kerminal-bin
makepkg -si
```

### Debian/Ubuntu

Tải gói `.deb` từ [trang releases](https://github.com/klpod221/kerminal/releases/latest):

```bash
# Tải gói .deb mới nhất
wget https://github.com/klpod221/kerminal/releases/latest/download/kerminal_x.x.x_amd64.deb

# Cài đặt
sudo dpkg -i kerminal_*.deb

# Sửa lỗi dependency nếu có
sudo apt-get install -f
```

### Fedora/RHEL

Tải gói `.rpm` từ [trang releases](https://github.com/klpod221/kerminal/releases/latest):

```bash
# Tải gói .rpm mới nhất
wget https://github.com/klpod221/kerminal/releases/latest/download/kerminal-x.x.x-1.x86_64.rpm

# Cài đặt
sudo rpm -i kerminal-*.rpm
```

### AppImage

Cho mọi bản phân phối Linux:

```bash
# Tải AppImage
wget https://github.com/klpod221/kerminal/releases/latest/download/kerminal_x.x.x_amd64.AppImage

# Cấp quyền thực thi
chmod +x kerminal_*.AppImage

# Chạy
./kerminal_*.AppImage
```

## Windows

### Trình cài đặt

1. Tải file `.msi` hoặc `.exe` installer mới nhất từ [trang releases](https://github.com/klpod221/kerminal/releases/latest)
2. Chạy trình cài đặt
3. Làm theo hướng dẫn cài đặt
4. Khởi chạy Kerminal từ menu Start

### Portable

1. Tải file `.zip` portable từ [trang releases](https://github.com/klpod221/kerminal/releases/latest)
2. Giải nén đến vị trí bạn muốn
3. Chạy `Kerminal.exe`

## macOS

::: warning Lưu ý quan trọng
Phiên bản macOS **chưa được ký/notarize** do hạn chế của Apple Developer Program ($99/năm). Bạn cần bypass Gatekeeper để chạy ứng dụng.
:::

### Cài đặt

1. Tải file `.dmg` mới nhất từ [trang releases](https://github.com/klpod221/kerminal/releases/latest)
2. Mở file `.dmg`
3. Kéo Kerminal vào thư mục Applications

### Bypass Gatekeeper

Sau lần khởi chạy đầu tiên thất bại, chạy lệnh này trong Terminal:

```bash
xattr -rd com.apple.quarantine /Applications/Kerminal.app
```

Sau đó thử khởi chạy Kerminal lại.

### Build từ Source

Nếu bạn muốn, có thể build từ source:

```bash
git clone https://github.com/klpod221/kerminal.git
cd kerminal
npm install
npm run tauri build
```

## Xác minh tải xuống

Bạn có thể xác minh tính toàn vẹn của file tải xuống bằng checksum được cung cấp trong mỗi release.

```bash
# Ví dụ cho Linux
sha256sum -c kerminal_x.x.x_amd64.deb.sha256
```

## Yêu cầu hệ thống

### Yêu cầu tối thiểu

| Thành phần | Yêu cầu |
|------------|---------|
| Hệ điều hành | Windows 10+, macOS 11+, Linux (kernel 5.0+) |
| RAM | 256 MB |
| Bộ nhớ | 100 MB |
| Màn hình | Độ phân giải 1024x768 |

### Khuyến nghị

| Thành phần | Khuyến nghị |
|------------|-------------|
| RAM | 512 MB+ |
| Bộ nhớ | SSD khuyến nghị |
| Màn hình | 1920x1080+ |
| GPU | Hỗ trợ WebGL 2.0 |

## Bước tiếp theo

Sau khi cài đặt, xem:

- [Bắt đầu](/vi/guide/getting-started) - Giới thiệu về Kerminal
- [Tính năng](/vi/guide/features) - Khám phá tất cả tính năng
- [Quản lý SSH](/vi/guide/ssh-management) - Thiết lập kết nối SSH đầu tiên
