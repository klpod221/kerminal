# Ghi Phiên Làm Việc

Ghi và phát lại phiên terminal để tài liệu hóa, đào tạo, hoặc debug.

## Ghi Phiên

### Bắt Đầu Ghi

1. Nhấp nút **Ghi** (●) trên thanh công cụ terminal
2. Chỉ báo màu đỏ xác nhận đang ghi
3. Toàn bộ input và output terminal được ghi lại

### Dừng Ghi

1. Nhấp nút **Dừng** (■)
2. Nhập tên cho bản ghi
3. Bản ghi được lưu cục bộ

::: tip
Đặt tên mô tả như "cai-dat-nginx-2024" để dễ nhận dạng.
:::

## Phát Bản Ghi

### Điều Khiển Phát Lại

1. Mở panel **Recordings** từ ngăn kéo
2. Nhấp bản ghi để phát

| Điều khiển | Hành động |
|------------|-----------|
| **Play/Pause** | Bắt đầu hoặc tạm dừng |
| **Timeline** | Nhấp để tua đến điểm bất kỳ |
| **Speed** | Điều chỉnh từ 0.5x đến 4x |
| **Restart** | Nhảy về đầu |

### Phím Tắt (khi phát lại)

| Phím | Hành động |
|------|-----------|
| `Space` | Play/Pause |
| `←` / `→` | Tua lùi/tiến |
| `Home` | Nhảy về đầu |
| `End` | Nhảy về cuối |

## Quản Lý Bản Ghi

### Tổ Chức

- **Đổi tên**: Nhấp chuột phải → Đổi Tên
- **Xóa**: Nhấp chuột phải → Xóa
- **Tìm kiếm**: Dùng thanh tìm kiếm để lọc

### Xuất

Bản ghi sử dụng định dạng **asciicast v2**:

1. Nhấp chuột phải bản ghi → Xuất
2. Lưu file `.cast`

Tương thích với:
- [asciinema.org](https://asciinema.org) để chia sẻ
- Lệnh `asciinema play`
- Bất kỳ trình phát asciicast nào

## Trường Hợp Sử Dụng

### Tài Liệu Hóa
Ghi quy trình cài đặt để chia sẻ với team.

### Đào Tạo
Tạo hướng dẫn hiển thị chuỗi lệnh và output dự kiến.

### Debug
Ghi lại phiên có vấn đề để phân tích sau.

### Demo
Ghi demo cho thuyết trình hoặc README.

## Mẹo

1. **Lên kế hoạch** - Biết bạn muốn demo gì
2. **Giữ ngắn gọn** - Tập trung vào task cụ thể
3. **Lệnh rõ ràng** - Tránh lỗi gõ và sửa
4. **Thêm mô tả** - Đặt tên bản ghi có ý nghĩa
5. **Xuất để chia sẻ** - Định dạng asciicast hoạt động mọi nơi

