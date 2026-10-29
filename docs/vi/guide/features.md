# Tính năng

Kerminal được trang bị đầy đủ các tính năng được thiết kế để nâng cao trải nghiệm terminal và đơn giản hóa quản lý SSH.

## 💻 Terminal Emulator

### Nhiều Tab & Chia màn hình
- Mở nhiều phiên terminal trong các tab
- Chia không gian làm việc theo chiều ngang hoặc dọc
- Kéo thả tab để sắp xếp lại
- Tích hợp shell gốc (bash, zsh, fish, PowerShell, v.v.)

### Render tăng tốc phần cứng Canvas 2D
- Render terminal tăng tốc phần cứng thông qua Canvas 2D
- Cuộn mượt mà và tiết kiệm tài nguyên, không bị giật lag compositor
- Hỗ trợ Unicode 11 với render emoji đúng
- Tùy chỉnh font và màu sắc

### Tính năng năng suất
- Tìm kiếm toàn văn trong output terminal
- Phát hiện liên kết có thể click
- Tích hợp clipboard (sao chép/dán)
- Phím tắt cho tất cả hành động phổ biến

### Sixel Graphics Protocol
- Hiển thị hình ảnh trực tiếp trong terminal
- Hỗ trợ các công cụ như `img2sixel`, `lsix`, `viu`, v.v.
- Render tăng tốc phần cứng qua xterm-addon-image
- Hoàn hảo để xem biểu đồ, sơ đồ và preview hình ảnh

## 📡 Quản lý SSH & Tunneling

### Tổ chức Profile
- Nhóm kết nối theo dự án, môi trường hoặc team
- Gán màu sắc cho profile để nhận dạng nhanh
- Thêm mô tả và ghi chú cho mỗi profile
- Import/export profile để backup hoặc chia sẻ

### Phương thức xác thực
- Xác thực bằng password
- Xác thực SSH key (RSA, Ed25519, ECDSA)
- Xác thực dựa trên certificate
- Hỗ trợ agent forwarding (sắp ra mắt)
- Hỗ trợ PKCS11 và Kerberos (sắp ra mắt)

### Quản lý SSH Key
- Tạo SSH key mới
- Import key hiện có
- Export key để backup
- Kiểm tra kết nối trước khi lưu

### Hỗ trợ Proxy
- HTTP proxy
- SOCKS4 proxy
- SOCKS5 proxy
- Phát hiện proxy tự động

### Jump Host Chain
- Kết nối qua nhiều bastion host
- Xác thực tự động tại mỗi hop
- Hiển thị chuỗi kết nối trực quan
- Lưu cấu hình multi-hop phức tạp

### Port Forwarding
- **Local Forwarding**: Truy cập dịch vụ remote trên máy local
- **Remote Forwarding**: Expose dịch vụ local ra remote
- **Dynamic SOCKS**: Tạo SOCKS proxy qua SSH
- Tự động khởi động forwarding khi kết nối
- Theo dõi trạng thái thời gian thực

## 💾 Lệnh đã lưu & Ghi Session

### Thư viện lệnh
- Lưu các lệnh thường dùng
- Tổ chức theo nhóm và danh mục
- Theo dõi thống kê sử dụng
- Đánh dấu yêu thích để truy cập nhanh
- Thay thế biến (ví dụ: `${username}`, `${hostname}`)

### Ghi Session
- Ghi session terminal theo định dạng asciicast v2
- Phát lại với điều khiển (play, pause, tốc độ)
- Export bản ghi để chia sẻ
- Tương thích với asciinema.org

## 🔄 Đồng bộ đa thiết bị & Bảo mật

### Đồng bộ Cloud
- Sync profile giữa các thiết bị
- Hỗ trợ MySQL, PostgreSQL và MongoDB
- Mã hóa AES-256-GCM cho tất cả dữ liệu sync
- Chiến lược giải quyết xung đột
- Quản lý thiết bị với thu hồi quyền

### Tự động Sync
- Đồng bộ hóa nền tự động
- Tùy chọn sync khi khởi động
- Kích hoạt sync thủ công
- Hiển thị trạng thái sync

## 🔒 Bảo mật

### Master Password
- Mã hóa tất cả dữ liệu nhạy cảm bằng master password
- Không bao giờ lưu trữ - chỉ giữ hash xác minh
- Yêu cầu khi khởi động (có thể cấu hình)

### Mã hóa riêng theo thiết bị
- Mỗi thiết bị có khóa mã hóa riêng
- Dữ liệu không thể truy cập từ thiết bị khác
- Dẫn xuất khóa an toàn với Argon2

### Tích hợp Keychain hệ thống
- Lưu master password trong keychain hệ thống
- Tự động mở khóa khi đăng nhập (tùy chọn)
- Lưu trữ thông tin xác thực an toàn

### Bảo mật Session
- Tự động khóa session sau khi không hoạt động
- Thời gian khóa có thể cấu hình
- Yêu cầu password để mở khóa

### Bảo vệ SSH Key
- Private key không bao giờ rời thiết bị không mã hóa
- Lưu trữ mã hóa khi nghỉ
- Hỗ trợ bảo vệ passphrase

## 🎨 Giao diện người dùng

### Theme tối hiện đại
- Dễ chịu cho mắt trong các phiên dài
- Ngôn ngữ thiết kế nhất quán
- Animation và transition mượt mà

### Tùy chỉnh
- Bảng màu tùy chỉnh
- Chọn font và kích thước
- Cài đặt độ trong suốt terminal
- Tùy chọn kiểu con trỏ

### Hiển thị trạng thái
- Trạng thái kết nối thời gian thực
- Trạng thái sync
- Hiển thị đang ghi
- Trạng thái port forwarding

### Phím tắt
- Điều hướng bàn phím toàn diện
- Phím tắt có thể tùy chỉnh
- Command palette nhanh
