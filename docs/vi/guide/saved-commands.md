# Lệnh Đã Lưu

Xây dựng thư viện các lệnh thường dùng để truy cập nhanh và tái sử dụng.

## Tạo Lệnh

### Thêm Lệnh Mới

1. Mở Saved Commands với `Ctrl+Alt+C`
2. Nhấp **Lệnh Mới**
3. Điền chi tiết:

| Trường | Mô tả |
|--------|-------|
| **Tên** | Tên mô tả (vd: "Deploy Production") |
| **Lệnh** | Lệnh thực tế để chạy |
| **Nhóm** | Danh mục để tổ chức |
| **Mô tả** | Ghi chú tùy chọn |

### Lưu Nhanh

Nhấp chuột phải trong terminal và chọn **Lưu Lệnh** để lưu lệnh vừa thực thi.

## Sử Dụng Biến

Lệnh hỗ trợ thay thế biến động với cú pháp `{{biến}}`:

```bash
ssh {{user}}@{{host}} -p {{port}}
```

Khi thực thi, bạn sẽ được nhắc điền mỗi biến.

### Mẫu Biến Thường Dùng

```bash
# Kết nối SSH
ssh {{user}}@{{server}}

# Lệnh Docker
docker exec -it {{container}} bash

# Thao tác Git
git clone git@github.com:{{org}}/{{repo}}.git

# Thao tác file
scp {{file}} {{user}}@{{host}}:{{path}}
```

## Thực Thi Lệnh

| Hành động | Kết quả |
|-----------|---------|
| **Nhấp đơn** | Chèn vào terminal đang active |
| **Nhấp đúp** | Chèn và thực thi |
| **Nhấp chuột phải** | Hiện menu ngữ cảnh |

### Tùy Chọn Menu Ngữ Cảnh

- Thực thi trong terminal hiện tại
- Thực thi trong tab mới
- Chỉnh sửa lệnh
- Sao chép vào clipboard
- Xóa

## Tổ Chức Lệnh

### Nhóm

- Tạo nhóm để phân loại lệnh
- Kéo lệnh giữa các nhóm
- Thu gọn/mở rộng nhóm

### Yêu Thích

- Đánh dấu sao lệnh để truy cập nhanh
- Lệnh yêu thích hiện ở đầu

### Tìm Kiếm

- Dùng thanh tìm kiếm để lọc lệnh
- Tìm trong tên, lệnh, và mô tả

## Thống Kê Sử Dụng

Kerminal theo dõi tần suất bạn dùng mỗi lệnh:
- Xem số lần sử dụng trên mỗi lệnh
- Sắp xếp theo dùng nhiều nhất
- Xác định ứng viên cho yêu thích

## Import & Export

### Export Lệnh

1. Nhấp chuột phải nhóm → Export
2. Lưu dưới dạng file JSON

### Import Lệnh

1. Nhấp Import trong panel
2. Chọn file JSON
3. Chọn merge hoặc thay thế

## Mẹo

1. **Dùng nhóm** - Tổ chức theo dự án, môi trường, hoặc mục đích
2. **Biến cho linh hoạt** - Làm lệnh tái sử dụng được
3. **Tên mô tả** - Dễ tìm sau này
4. **Đánh dấu yêu thích** - Truy cập nhanh lệnh thường dùng
5. **Giữ cập nhật** - Xóa lệnh lỗi thời

