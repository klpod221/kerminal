# Quản lý SSH

Tìm hiểu cách quản lý kết nối SSH hiệu quả với hệ thống quản lý profile mạnh mẽ của Kerminal.

## SSH Profiles

### Tạo Profile

1. Mở panel SSH Profiles (thanh bên)
2. Click nút **New Profile** ở dưới cùng của panel
3. Trong tab **Basic**, điền thông tin kết nối:
   - **Profile Name**: Tên mô tả
   - **Host**: Hostname hoặc địa chỉ IP
   - **Port**: Cổng SSH (mặc định: 22)
   - **Username**: Tên người dùng SSH
   - **Authentication Method**: Chọn Password hoặc SSH Key

### Nhóm Profile

Tổ chức kết nối bằng cách tạo nhóm:

1. Click nút **New Group** ở dưới cùng của panel SSH Profiles
2. Nhập tên nhóm
3. Kéo profile vào nhóm hoặc sử dụng nút **Add profile to group** trên header của nhóm

### Màu Profile

Gán màu cho profile để tổ chức trực quan:

1. Trong trình chỉnh sửa profile, chuyển đến tab **Terminal**
2. Sử dụng **Tab Color** picker để chọn màu
3. Màu sắc sẽ hiển thị dưới dạng chỉ báo ở sidebar và tab header

## Phương thức xác thực

### Xác thực bằng Password

Chỉ cần nhập password khi tạo profile. Password được mã hóa bằng AES-256-GCM.

::: tip Ghi chú bảo mật
Hãy cân nhắc sử dụng xác thực bằng key để bảo mật tốt hơn.
:::

### Xác thực SSH Key

1. Trong trình chỉnh sửa profile (tab **Basic**), chọn **SSH Key** làm Authentication Method
2. Chọn một key có sẵn từ dropdown
3. Hoặc click **Manage SSH Keys** để import/tạo key mới

### Quản lý SSH Key

Truy cập trình quản lý key thông qua liên kết **Manage SSH Keys** trong trình chỉnh sửa profile:

#### Tạo Key mới
```bash
# Kerminal sử dụng russh để tạo key
# Hỗ trợ RSA, Ed25519 và ECDSA
```

#### Import Key
- Import từ file
- Dán nội dung key trực tiếp
- Import từ clipboard

#### Export Key
- Export public key để cấu hình server
- Backup private key (được mã hóa)

## Cấu hình Proxy

Kết nối qua proxy khi không thể SSH trực tiếp.

1. Trong trình chỉnh sửa profile, chuyển đến tab **Network**
2. Tích chọn **Use Proxy**
3. Chọn **Proxy Type** (HTTP, SOCKS4, SOCKS5)

### HTTP Proxy

```
Host: proxy.example.com
Port: 8080
Username: (tùy chọn)
Password: (tùy chọn)
```

### SOCKS4/5 Proxy

```
Host: socks.example.com
Port: 1080
Username: (tùy chọn)
Password: (tùy chọn)
```

## Jump Hosts (Bastion)

Kết nối qua một hoặc nhiều bastion host để đến server đích.

### Jump đơn giản

```
Local → Bastion → Target
```

1. Tạo profile cho bastion host trước
2. Trong tab **Network** của profile đích:
3. Tích chọn **Use Jump Host**
4. Chọn profile bastion từ dropdown **Jump Host Profile**

### Chuỗi Jump

```
Local → Bastion1 → Bastion2 → Target
```

Thêm nhiều jump host theo thứ tự. Kerminal sẽ tự động:
- Xác thực tại mỗi hop
- Chuyển tiếp kết nối qua chuỗi
- Hiển thị đường dẫn kết nối trực quan

## Port Forwarding

### Local Port Forwarding

Truy cập dịch vụ remote trên máy local:

```
Local Port: 8080
Remote Host: localhost
Remote Port: 80
```

Bây giờ `localhost:8080` kết nối đến port 80 trên server remote.

**Trường hợp sử dụng:**
- Truy cập giao diện web sau firewall
- Kết nối đến database remote
- Sử dụng server phát triển remote

### Remote Port Forwarding

Expose dịch vụ local ra mạng remote:

```
Remote Port: 8080
Local Host: localhost
Local Port: 3000
```

Bây giờ port 8080 trên server remote kết nối đến port local 3000 của bạn.

**Trường hợp sử dụng:**
- Chia sẻ server phát triển local
- Kiểm tra webhook
- Expose dịch vụ tạm thời

### Dynamic Port Forwarding (SOCKS)

Tạo SOCKS proxy qua SSH:

```
Local Port: 1080
```

Cấu hình ứng dụng sử dụng `localhost:1080` làm SOCKS5 proxy.

**Trường hợp sử dụng:**
- Duyệt web qua mạng remote
- Truy cập tài nguyên nội bộ
- Vượt qua hạn chế mạng

### Tự động khởi động Forwarding

Bật "Auto-start" trên các quy tắc port forwarding để tự động thiết lập tunnel khi kết nối.

## Kiểm tra kết nối

Trước khi lưu profile, hãy kiểm tra kết nối:

1. Điền tất cả thông tin kết nối
2. Click **Test Connection**
3. Kerminal sẽ cố gắng kết nối và báo cáo:
   - Thành công với server fingerprint
   - Chi tiết lỗi xác thực
   - Lỗi mạng

## Import từ SSH Config

Kerminal tự động phân tích file cấu hình SSH local (`~/.ssh/config`) và cho phép bạn import các host thành profile.

1. Mở panel **SSH Profiles**
2. Tìm phần **From .ssh/config**
3. Click nút **Import** (biểu tượng Download)
4. Chọn các host bạn muốn import
5. Click **Import** để tạo profile cho các host đã chọn

## Sao lưu & Khôi phục (Backup & Restore)

Bảo vệ dữ liệu của bạn bằng cách tạo sao lưu đầy đủ các profile, key và cài đặt.

### Tạo Sao lưu (Backup)

1. Click nút **Backup & Restore** (biểu tượng Hộp lưu trữ) trên thanh công cụ
2. Trong phần **Export Backup**:
3. (Tùy chọn) Tích **Password Protection** để mã hóa backup bằng AES-256-GCM
4. Click **Export Backup**
5. Lưu file `.json` (hoặc `.kbak` nếu mã hóa)

### Khôi phục (Restore)

1. Click nút **Backup & Restore** (biểu tượng Hộp lưu trữ) trên thanh công cụ
2. Trong phần **Import Backup**:
3. Click **Select Backup File**
4. Chọn file backup của bạn
5. Nếu được mã hóa, nhập password khi được nhắc
6. Ứng dụng sẽ tải lại với dữ liệu được khôi phục

::: warning
Khôi phục backup sẽ cập nhật dữ liệu hiện có trùng khớp ID trong backup.
:::

## Thực hành tốt nhất

### Bảo mật

1. **Sử dụng xác thực bằng key** khi có thể
2. **Bảo vệ key** bằng passphrase
3. **Xoay vòng key** định kỳ
4. **Không chia sẻ profile** chứa thông tin xác thực

### Tổ chức

1. **Nhóm theo môi trường** (prod, staging, dev)
2. **Sử dụng tên mô tả** bao gồm mục đích
3. **Gán màu** để nhận dạng nhanh
4. **Thêm ghi chú** cho thông tin cụ thể của kết nối

### Backup

1. **Tạo backup** thường xuyên
2. **Mã hóa backup** với password mạnh
3. **Lưu trữ backup** an toàn (cloud mã hóa, password manager)
