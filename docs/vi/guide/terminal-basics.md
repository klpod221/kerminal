# Cơ Bản Terminal

Tìm hiểu những điều cơ bản khi làm việc với terminal emulator của Kerminal.

## Bắt Đầu Nhanh

1. **Mở Kerminal** - Mở với workspace terminal sẵn sàng sử dụng
2. **Xem Tour** - Nhấn `Ctrl+Shift+/` để xem tour hướng dẫn tính năng
3. **Command Palette** - Nhấn `Ctrl+Shift+P` để khám phá tất cả lệnh

::: tip Người dùng macOS
Thay `Ctrl` bằng `Cmd (⌘)` cho tất cả phím tắt.
:::

## Làm Việc Với Tab

### Tạo Tab

| Hành động | Cách thực hiện |
|-----------|----------------|
| Terminal local mới | `Ctrl+T` hoặc nhấp nút `+` |
| Kết nối SSH | Nhấp đúp profile trong ngăn SSH |
| Nhân đôi tab | Nhấp chuột phải → Nhân đôi |

### Quản Lý Tab

- **Chuyển**: Nhấp hoặc `Ctrl+1` đến `Ctrl+9`
- **Sắp xếp**: Kéo thả
- **Đóng**: `Ctrl+W` hoặc nhấp `×`
- **Đóng các tab khác**: Nhấp chuột phải → Đóng Tab Khác

### Menu Ngữ Cảnh Tab

Nhấp chuột phải tab để:
- Đổi tên / Đổi màu
- Nhân đôi / Chia sang panel mới
- Đóng / Đóng Khác / Đóng Tất Cả

## Chia Panel

### Tạo Chia

| Phím tắt | Kết quả |
|----------|---------|
| `Ctrl+K` | Chia dọc (cạnh nhau) |
| `Ctrl+L` | Chia ngang (xếp chồng) |

### Điều Hướng & Thay Đổi Kích Thước

- **Focus panel**: `Ctrl+←` / `Ctrl+→` hoặc nhấp trực tiếp
- **Thay đổi kích thước**: Kéo thanh phân chia
- **Đặt lại kích thước**: Nhấp đúp thanh phân chia
- **Đóng**: Gõ `exit` hoặc nhấp chuột phải → Đóng

## Tìm Kiếm

### Tìm Trong Terminal

1. Nhấn `Ctrl+Shift+F` hoặc `F3`
2. Gõ từ khóa tìm kiếm
3. `Enter` = tìm tiếp, `Shift+Enter` = tìm trước đó
4. `Escape` để đóng

**Tùy chọn**: Phân biệt hoa thường, Regex, Từ nguyên vẹn

### Lịch Sử Lệnh

1. Nhấn `Ctrl+H`
2. Gõ để lọc lịch sử
3. Chọn và nhấn `Enter` để thực thi

## Mẹo & Thực Hành Tốt

### Năng Suất

- **Command Palette** (`Ctrl+Shift+P`) - Truy cập nhanh mọi thứ
- **Học phím tắt** - Xem [Phím Tắt](/vi/guide/keyboard-shortcuts)
- **Mã màu tab** - Đỏ cho prod, xanh cho dev
- **Đổi tên tab** - Nhấp chuột phải → Đổi tên cho rõ ràng

### Mẹo Bố Cục

- `Ctrl+K` × 3 = bố cục 3 cột
- `Ctrl+L` × 2 = bố cục 3 hàng
- Kết hợp để tạo bố cục lưới phức tạp

