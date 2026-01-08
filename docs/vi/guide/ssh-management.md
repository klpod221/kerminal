# Quản Lý SSH

Quản lý kết nối SSH với hệ thống profile mạnh mẽ của Kerminal.

## Tạo Profile

1. Mở panel SSH Profiles (`Ctrl+Shift+S`)
2. Nhấp **Profile Mới**
3. Điền thông tin kết nối:

| Trường | Mô tả |
|--------|-------|
| **Tên** | Tên hiển thị profile |
| **Host** | Hostname hoặc IP |
| **Port** | Cổng SSH (mặc định: 22) |
| **Username** | Tên người dùng SSH |
| **Xác thực** | Password hoặc SSH Key |

## Tổ Chức Profile

### Nhóm
1. Nhấp **Nhóm Mới**
2. Kéo profile vào nhóm
3. Thu gọn/mở rộng khi cần

### Màu Sắc
1. Sửa profile → tab **Terminal**
2. Chọn **Tab Color**
3. Màu hiển thị trong sidebar và tab

## Xác Thực

### Password
Nhập password khi tạo profile. Được mã hóa bằng AES-256-GCM.

::: tip
Nên dùng SSH key cho bảo mật tốt hơn.
:::

### SSH Key
1. Chọn **SSH Key** làm phương thức xác thực
2. Chọn key có sẵn hoặc nhấp **Quản Lý SSH Keys**

### Trình Quản Lý Key
- **Tạo mới**: RSA, Ed25519, ECDSA
- **Import**: Từ file hoặc clipboard
- **Export**: Public key cho server

## Tùy Chọn Mạng

### Proxy
1. Sửa profile → tab **Network**
2. Bật **Sử dụng Proxy**
3. Chọn loại: HTTP, SOCKS4, SOCKS5
4. Nhập chi tiết proxy

### Jump Host

Kết nối qua bastion server:

```
Local → Bastion → Target
```

1. Tạo profile bastion trước
2. Trong profile đích → tab **Network**
3. Bật **Sử dụng Jump Host**
4. Chọn profile bastion

**Chuỗi jump**: Thêm nhiều jump host theo thứ tự.

## Port Forwarding

### Local Forward
Truy cập dịch vụ remote cục bộ:
```
localhost:8080 → remote:80
```
Trường hợp dùng: Truy cập web UI, database

### Remote Forward
Expose dịch vụ local ra remote:
```
remote:8080 → localhost:3000
```
Trường hợp dùng: Chia sẻ dev server, test webhook

### Dynamic (SOCKS)
Tạo SOCKS proxy qua SSH:
```
Local port 1080 → SOCKS5 proxy
```
Trường hợp dùng: Duyệt web qua mạng remote

Bật **Auto-start** để thiết lập tunnel khi kết nối.

## Import & Export

### Từ ~/.ssh/config
1. Tìm phần **Từ .ssh/config**
2. Nhấp nút **Import**
3. Chọn host để import

### Backup & Restore
1. Nhấp **Backup & Restore** trên thanh công cụ
2. **Export**: Lưu profile dưới dạng JSON (bảo vệ password tùy chọn)
3. **Import**: Khôi phục từ file backup

## Kiểm Tra Kết Nối

Trước khi lưu:
1. Điền đầy đủ thông tin
2. Nhấp **Test Connection**
3. Xem kết quả: thành công, lỗi xác thực, hoặc lỗi mạng

## Thực Hành Tốt

### Bảo Mật
- Dùng SSH key thay vì password
- Bảo vệ key bằng passphrase
- Xoay key định kỳ

### Tổ Chức
- Nhóm theo môi trường (prod, staging, dev)
- Dùng tên mô tả
- Gán màu để nhận dạng nhanh

