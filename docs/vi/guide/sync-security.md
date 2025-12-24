# Đồng bộ & Bảo mật

Tìm hiểu về tính năng đồng bộ hóa đa thiết bị và bảo mật của Kerminal.

## Đồng bộ đa thiết bị

Kerminal cho phép bạn sync SSH profile, lệnh đã lưu và cài đặt giữa nhiều thiết bị.

### Database hỗ trợ

| Database | Mô tả |
|----------|-------|
| MySQL | Database quan hệ đầy đủ tính năng |
| PostgreSQL | Database mã nguồn mở nâng cao |
| MongoDB | Database tài liệu NoSQL |

### Thiết lập Sync

1. Click nút **Sync Manager** (biểu tượng Đám mây) trên thanh công cụ
2. Chọn loại database
3. Nhập thông tin kết nối:
   - Host
   - Port
   - Tên database
   - Username
   - Password
4. Click **Test Connection**
5. Bật sync

### Cấu hình MongoDB Atlas

Kerminal hỗ trợ MongoDB Atlas để đồng bộ. Ánh xạ connection string của bạn vào các trường:

**Ví dụ Connection String:**
`mongodb+srv://user:pass@cluster0.abcde.mongodb.net/?retryWrites=true&w=majority`

**Nhập trong Kerminal:**
- **Protocol**: `mongodb+srv`
- **Host**: `cluster0.abcde.mongodb.net`
- **Port**: `27017` (Mặc định)
- **Username**: `user`
- **Password**: `pass`
- **Options**: `retryWrites=true&w=majority`

### Những gì được Sync

- ✅ SSH profile và nhóm
- ✅ Lệnh đã lưu
- ✅ Cấu hình port forwarding
- ✅ Cài đặt ứng dụng
- ❌ SSH private key (không bao giờ sync)
- ❌ Master password

### Hành vi Sync

#### Tự động Sync
Bật đồng bộ hóa tự động:
- Sync khi khởi động
- Sync khi thay đổi profile
- Khoảng thời gian sync nền

#### Sync thủ công
Kích hoạt sync thủ công:
- Click nút sync trong thanh trạng thái
- Sử dụng phím tắt
- Sync Manager > Settings > Sync Now

### Giải quyết xung đột

Khi cùng một mục được sửa đổi trên nhiều thiết bị:

| Chiến lược | Mô tả |
|------------|-------|
| **Last Write Wins** | Thay đổi gần nhất được giữ |
| **Ask** | Hỏi người dùng chọn |
| **Keep Local** | Luôn ưu tiên thay đổi local |
| **Keep Remote** | Luôn ưu tiên thay đổi remote |

### Quản lý thiết bị

Quản lý các thiết bị đã kết nối:

1. Mở **Sync Manager** (biểu tượng Đám mây)
2. Đi đến tab **Devices**
3. Xem tất cả thiết bị đã đăng ký
4. Thu hồi quyền truy cập cho thiết bị bị mất/đánh cắp
5. Đổi tên thiết bị để nhận dạng

## Kiến trúc bảo mật

### Mã hóa dữ liệu

Tất cả dữ liệu nhạy cảm được mã hóa trước khi lưu trữ:

```
Dữ liệu → Mã hóa AES-256-GCM → Lưu trữ mã hóa
```

**Dữ liệu được bảo vệ bao gồm:**
- SSH password
- Passphrase private key
- Thông tin xác thực sync
- Biến lệnh đã lưu

### Master Password

Master password là chìa khóa cho tất cả dữ liệu mã hóa:

- **Không bao giờ lưu trữ** - chỉ giữ hash xác minh
- **Không thể khôi phục** - nếu quên, dữ liệu mã hóa sẽ mất
- **Được sử dụng để dẫn xuất** khóa mã hóa qua Argon2

#### Đặt Master Password

1. Lần khởi chạy đầu tiên sẽ yêu cầu master password
2. Hoặc click **Master Password Settings** (biểu tượng Khiên) trên thanh công cụ
3. Nhập password mạnh
4. Xác nhận password
5. Tùy chọn lưu trong keychain hệ thống

#### Thay đổi Master Password

1. Click **Master Password Settings** (biểu tượng Khiên)
2. Click **Change Master Password**
3. Nhập password hiện tại
4. Nhập password mới
5. Tất cả dữ liệu được mã hóa lại với key mới

### Dẫn xuất Key

Kerminal sử dụng **Argon2id** để dẫn xuất key:

```
Master Password + Salt → Argon2id → Encryption Key
```

Tham số Argon2 được điều chỉnh cho bảo mật:
- Memory: 64 MB
- Iterations: 3
- Parallelism: 4

### Key riêng theo thiết bị

Mỗi thiết bị có mã hóa riêng ngăn truy cập dữ liệu từ các thiết bị khác:

1. Thiết bị tạo cặp key riêng khi chạy lần đầu
2. Dữ liệu sync được mã hóa với key riêng của thiết bị
3. Các thiết bị khác không thể giải mã nếu không có trao đổi key đúng cách

### Mã hóa dữ liệu Sync

Dữ liệu sync lên cloud database luôn được mã hóa:

```
Dữ liệu Local → Mã hóa với Master Key → Sync lên Database
```

Server không bao giờ thấy không mã hóa:
- Password
- Dữ liệu private key
- Cấu hình nhạy cảm

## Bảo mật Session

### Tự động khóa

Tự động khóa Kerminal sau khi không hoạt động:

1. Click **Master Password Settings** (biểu tượng Khiên)
2. Bật **Auto-Lock**
3. Đặt thời gian chờ (ví dụ: 5 phút)
4. Chọn hành vi khóa:
   - Đóng tất cả kết nối
   - Giữ kết nối (chỉ khóa UI)

### Mở khóa

Khi bị khóa, nhập master password để mở khóa.

### Tích hợp Keychain

Lưu master password trong keychain hệ thống để tự động mở khóa:

| Nền tảng | Keychain |
|----------|----------|
| Windows | Windows Credential Manager |
| macOS | Keychain Access |
| Linux | Secret Service (GNOME Keyring, KWallet) |

**Bật tự động mở khóa:**
1. Click **Master Password Settings** (biểu tượng Khiên)
2. Bật **Store in Keychain**
3. Nhập master password khi được yêu cầu

## Bảo mật SSH Key

### Lưu trữ Private Key

Private key được lưu trữ với mã hóa:

```
Private Key → Mã hóa với Device Key → Lưu trữ an toàn
```

### Passphrase của Key

Khi import key được bảo vệ bằng passphrase:
- Passphrase có thể được lưu mã hóa
- Hoặc hỏi mỗi lần (bảo mật hơn)

### Không bao giờ Export

Private key không bao giờ:
- Sync lên cloud
- Export mà không có hành động rõ ràng
- Ghi log hoặc truyền đi

## Thực hành tốt nhất

### Master Password

1. **Sử dụng password mạnh** (12+ ký tự, chữ hoa thường, số, ký hiệu)
2. **Không sử dụng lại** password từ dịch vụ khác
3. **Cân nhắc dùng passphrase** (dễ nhớ hơn, vẫn an toàn)
4. **Lưu backup** trong password manager an toàn

### Bảo mật Sync

1. **Sử dụng TLS/SSL** cho kết nối database
2. **Password database mạnh**
3. **Kiểm tra thiết bị thường xuyên** - thu hồi thiết bị không xác định
4. **Mã hóa backup database**

### Quản lý Key

1. **Sử dụng Ed25519** cho key mới (bảo mật hơn, nhanh hơn)
2. **Bảo vệ bằng passphrase**
3. **Key khác nhau** cho các môi trường khác nhau
4. **Xoay vòng định kỳ** (khuyến nghị hàng năm)

### Tổng quát

1. **Cập nhật Kerminal** để có bản vá bảo mật
2. **Khóa khi rời** máy tính
3. **Xem xét thiết bị kết nối** thường xuyên
4. **Backup file export mã hóa** an toàn
