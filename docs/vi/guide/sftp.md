# Trình Duyệt File SFTP

Truyền file an toàn với trình duyệt SFTP tích hợp của Kerminal.

## Mở SFTP

1. Kết nối đến SSH profile
2. Nhấn `Ctrl+Shift+F` hoặc nhấp biểu tượng thư mục trên thanh công cụ

::: tip
SFTP chỉ khả dụng cho kết nối SSH đang hoạt động.
:::

## Giao Diện

Trình duyệt file cung cấp giao diện quen thuộc:
- **Thanh đường dẫn** - Thư mục hiện tại, có thể chỉnh sửa
- **Danh sách file** - File và thư mục với chi tiết
- **Thanh công cụ** - Điều hướng, tải lên, tùy chọn xem

## Thao Tác File

| Hành động | Cách thực hiện |
|-----------|----------------|
| Điều hướng | Nhấp thư mục hoặc chỉnh sửa thanh đường dẫn |
| Quay lại | Nút `←` hoặc `Backspace` |
| Đi lên | Nhấp thư mục cha trong đường dẫn |
| Làm mới | Nhấp nút làm mới |
| Tải lên | Kéo thả hoặc nhấp Tải lên |
| Tải xuống | Nhấp chuột phải → Tải xuống |
| Thư mục mới | Nhấp chuột phải → Thư Mục Mới |
| Đổi tên | Nhấp chuột phải → Đổi Tên |
| Xóa | Nhấp chuột phải → Xóa |

## Thanh Đường Dẫn

- **Chỉnh sửa**: Nhấp thanh đường dẫn để gõ thủ công
- **Điều hướng**: Gõ đường dẫn và nhấn `Enter`
- **Tự động hoàn thành**: Dùng `Tab` để xem gợi ý
- **Hủy**: Nhấn `Escape`

## Điều Hướng Bằng Bàn Phím

| Phím | Hành động |
|------|-----------|
| `↑` / `↓` | Điều hướng file |
| `Enter` | Mở thư mục / Xem trước file |
| `Backspace` | Đi đến thư mục cha |
| `Ctrl+F` | Tìm kiếm trong thư mục |
| `Delete` | Xóa file đã chọn |
| `F2` | Đổi tên file đã chọn |
| `Ctrl+Shift+N` | Tạo thư mục mới |

## Kéo Thả

### Tải Lên
- Kéo file từ máy tính vào trình duyệt
- Chỉ báo tiến trình hiển thị trạng thái
- Hỗ trợ nhiều file

### Tải Xuống
- Kéo file từ trình duyệt ra desktop
- Hoặc nhấp chuột phải → Tải xuống

## Xem Trước File

Nhấp đúp file để xem trước:
- **File văn bản** - Xem với syntax highlighting
- **Hình ảnh** - Xem trước với điều khiển zoom
- **File khác** - Yêu cầu tải xuống

## Hàng Đợi Truyền

Khi truyền nhiều file:
- Xem tiến trình trong hàng đợi
- Hủy từng file đang truyền
- Tiếp tục truyền bị gián đoạn

## Thực Hành Tốt

1. **Dùng kéo thả** để tải nhanh
2. **Điều hướng bằng bàn phím** cho người dùng chuyên nghiệp
3. **Xem trước trước khi tải** cho file văn bản
4. **Tạo thư mục** để tổ chức file remote

