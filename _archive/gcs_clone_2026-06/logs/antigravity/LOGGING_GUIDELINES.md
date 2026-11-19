# Hướng dẫn ghi Log dành cho AI (AI Logging Guidelines)

File này được tạo ra để hướng dẫn các AI assistant tiếp quản dự án về cách ghi nhật ký (logging) một cách chuẩn xác và thống nhất. Hãy đọc kỹ và tuân thủ định dạng này khi làm việc.

## 1. Mục đích của việc ghi Log
Trong dự án này, việc ghi lại lịch sử phát triển là **bắt buộc** để:
- Người dùng (User) có thể theo dõi chính xác AI đã làm gì, sửa file nào, vào lúc mấy giờ.
- Các AI assistant ở phiên làm việc sau (hoặc sau khi server restart) có thể đọc file log để nắm bắt bối cảnh (context) và tiếp tục công việc mà không bị đứt đoạn.

## 2. Vị trí lưu Log
- Toàn bộ file log phải được lưu trong thư mục: `logs/antigravity/`
- Tên file log phải tuân theo định dạng ngày tháng: `session_YYYY-MM-DD.md` (Ví dụ: `session_2026-06-08.md`).
- Nếu chuyển sang ngày mới, hãy tạo một file `session_YYYY-MM-DD.md` mới.

## 3. Định dạng Log (Log Format)
Mỗi hành động phải được ghi trên một dòng mới dưới dạng danh sách (bullet point) theo cấu trúc sau:

```markdown
- `[TIMESTAMP HH:MM]` **[TAG]** Mô tả chi tiết hành động.
```

### Các thành phần:
- **`TIMESTAMP HH:MM`**: Thời gian thực hiện hành động theo giờ địa phương của User (dựa trên thẻ `<ADDITIONAL_METADATA>` được cung cấp trong prompt). Định dạng 24h (VD: `21:06`).
- **`[TAG]`**: Phân loại hành động bằng một trong các tag sau:
  - **`[PLANNING]`**: Lên kế hoạch, tạo file `implementation_plan.md` hoặc đề xuất giải pháp.
  - **`[RESEARCH]`**: Tìm kiếm web, phân tích tài liệu, đọc file hiện có để tìm hiểu hệ thống.
  - **`[CODE]`**: Viết code mới, sửa lỗi, tái cấu trúc (refactor) hoặc xoá file. (Cần ghi rõ file nào được sửa).
  - **`[VERIFY]`**: Chạy lệnh kiểm tra, build code (`npm run build`), chạy test hoặc người dùng đã xác nhận chạy thành công.
  - **`[BUGFIX]`**: Sửa một lỗi cụ thể do người dùng báo cáo hoặc tự phát hiện.
  - **`[DOCS]`**: Viết tài liệu (walkthrough, README, hoặc chính file hướng dẫn này).

## 4. Ví dụ Mẫu (Examples)
```markdown
- `[TIMESTAMP 20:41]` **[RESEARCH]** User yêu cầu tìm hiểu về GCS cho Swarm. Tiến hành Search Web về "drone swarm ground control station UI features".
- `[TIMESTAMP 20:50]` **[PLANNING]** Lên kế hoạch sửa lỗi Fly Here chỉ nhận Drone 1 và Đại tu giao diện sang phong cách QGroundControl.
- `[TIMESTAMP 20:51]` **[CODE]** Đưa `selectedDroneId` vào `droneStore.ts` để đồng bộ chọn Drone giữa UI và Map. Sửa lại logic trong `DroneMap.tsx`.
- `[TIMESTAMP 20:54]` **[VERIFY]** Build thành công. Lỗi Fly Here đa phương tiện đã fix xong. Cập nhật `task.md`.
```

## 5. Quy tắc Bắt buộc (Golden Rules)
1. **Ghi log liên tục**: Sau khi hoàn thành một cụm task hoặc trước khi kết thúc một lượt trả lời (turn), hãy dùng tool `replace_file_content` (chế độ append) để thêm log vào file của ngày hôm đó. Không bao giờ được phép quên!
2. **Chi tiết nhưng súc tích**: Ghi rõ tên file bị thay đổi (VD: `DroneMap.tsx`), nhưng không cần dán lại toàn bộ code vào log.
3. **Tiếng Việt**: Sử dụng ngôn ngữ Tiếng Việt để viết log cho đồng nhất với ngôn ngữ giao tiếp của dự án.
