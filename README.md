# NCKH swarm drone (workspace)

Thư mục làm việc cho đề tài NCKH: đội hình multi-UAV dùng DroneEngage + ArduPilot SITL dưới độ trễ truyền thông.
Code nghiên cứu chính dựa trên DroneEngage upstream, không nằm ở repo này.

**Bắt đầu từ [`PROJECT_CONTEXT.md`](PROJECT_CONTEXT.md)**: tổng hợp toàn bộ bối cảnh đề tài (mục tiêu, quyết định, trạng thái, tài liệu, quy tắc làm việc) cho người hoặc AI mới.

## Cấu trúc

| Thư mục / file | Nội dung |
|---|---|
| `PROJECT_CONTEXT.md` | Bối cảnh đầy đủ của project, đọc đầu tiên. |
| `nckh_research/` | Bản sao các kết quả nghiên cứu (lit review, verification, gate pilot, đề xuất đề tài...). Xem `nckh_research/README.md`. |
| `docs/PROJECT_SURVEY.md` | Khảo sát hiện trạng project (2026-11-07). |
| `hardware_config.md` | Cấu hình phần cứng drone. |
| `airgap_setup_guide.md` | Hướng dẫn dựng DroneEngage chế độ AirGap. |
| `references/` | Tài liệu tham khảo: danh sách hướng nghiên cứu giảng viên Khoa Điện - Điện tử, sơ đồ cách tìm tài liệu. |
| `_personal/` | CV và ghi chú định hướng nghề nghiệp (cá nhân). |
| `_archive/gcs_clone_2026-06/` | Bản GCS clone (React/TS) cũ, giai đoạn 06/2026, không còn dùng. `node_modules` đã xóa, chạy `npm install` trong `src/client` nếu cần chạy lại. |

Dọn lại ngày 2026-11-18.

## Giấy phép (License)

- [LICENSE](LICENSE) (MIT License)
