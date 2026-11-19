# Progress Log — Clone Drone Engage

Đồ án: Hệ thống Drone Bầy Đàn Tự Hành dựa trên DroneEngage

---

## 2026-06-07: Khởi tạo dự án

- **Đã làm:** 
  - Research toàn bộ DroneEngage docs (cloud.ardupilot.org) và GitHub repos (github.com/DroneEngage)
  - Viết README.md tổng hợp (protocol, commands, config, kiến trúc)
  - Thiết lập system prompts cho 3 agent (Hermes, Antigravity, Codex)
  - Tạo cấu trúc thư mục dự án
- **Kết quả:** README.md hoàn chỉnh 20KB+, sẵn sàng bắt đầu Phase 1
- **Vấn đề:** Không có
- **File:** README.md, research/*.md

---

## 2026-06-07: Milestone: Khởi tạo React Vite Web Client Skeleton

- **Đã làm:**
  - Khởi tạo project React Vite TypeScript trong `src/client/`.
  - Cài dependencies theo kiến trúc: Zustand, React Router, Tailwind CSS, React Leaflet, Leaflet.
  - Pin React về `18.3.1` và React Router về v6 để khớp `docs/architecture/overview.md`.
  - Cấu hình Tailwind với Vite plugin.
  - Tạo cấu trúc thư mục trong `src/client/src/` theo Feature-Sliced / Domain-Driven structure.
  - Thay demo Vite bằng route shell và placeholder `AuthPage`.
  - Tạo log riêng cho Codex tại `logs/codex/`.
- **Kết quả:**
  - `npm.cmd run build` thành công.
  - `npm.cmd run lint` thành công.
  - Vite dev server chạy tại `http://127.0.0.1:5173` và trả HTTP 200.
- **Vấn đề gặp phải:**
  - `npm.ps1` trên máy trỏ tới đường dẫn npm CLI bị thiếu; đã dùng `npm.cmd`.
  - Sandbox chặn npm ghi cache vào `AppData`; npm install/create cần quyền escalated.
  - `react-leaflet@5.0.0` yêu cầu React 19 nên đã dùng `react-leaflet@4.2.1` để giữ React 18.
  - In-app Browser không khởi động được do lỗi quyền `EPERM` khi runtime truy cập `C:\Users\phamt\AppData`; đã xác minh bằng HTTP request trong terminal.
- **File đã thay đổi:**
  - `src/client/package.json`
  - `src/client/package-lock.json`
  - `src/client/vite.config.ts`
  - `src/client/src/App.tsx`
  - `src/client/src/main.tsx`
  - `src/client/src/index.css`
  - `src/client/src/features/auth/AuthPage.tsx`
  - `src/client/src/store/authStore.ts`
  - `src/client/src/types/andruav.ts`
  - `src/client/src/utils/logger.ts`
  - `logs/codex/*.md`

## 2026-06-07: Thiết kế Kiến trúc Tổng thể (Phase 1)

- **Đã làm:**
  - Thiết kế cấu trúc Architecture Overview (Tech Stack, Folder Structure, State Management bằng Zustand)
  - Ghi nhận thông tin đăng nhập và lưu file `.env`
  - Viết script NodeJS test kết nối Auth Server để chuẩn bị chạy thử nghiệm
- **Kết quả:** Bản thiết kế kiến trúc hoàn tất và được duyệt. Sẵn sàng test kết nối mạng.
- **Vấn đề:** Không có
- **File:** docs/architecture/overview.md, config/.env, scripts/test_auth.mjs

---

## 2026-06-07: Thiết kế Data Flow & Cập nhật Task

- **Đã làm:**
  - Xác nhận React Vite Skeleton do Codex build đã chạy thành công và hiển thị giao diện.
  - Thiết kế chi tiết luồng dữ liệu (Data-flow) cho Đăng nhập và WebSocket (docs/architecture/data-flow.md).
  - Cập nhật file cấu trúc `task.md` cho Phase 2.
- **Kết quả:** Skeleton hoàn thiện. Tài liệu Data-flow đã sẵn sàng để Codex bắt đầu code giao diện Auth và module Protocol.
- **Vấn đề:** Do Auth Server endpoint thay đổi tuỳ cài đặt, thống nhất đẩy URL này vào file `.env` theo biến `VITE_AUTH_SERVER_URL`.
- **File:** logs/progress.md, logs/antigravity/*, docs/architecture/data-flow.md, task.md

---

## 2026-06-07: Milestone: Xây dựng Login Page & Auth Flow (Phase 2)

- **Đã làm:**
  - Antigravity đảm nhận vai trò Codex (do Codex hết token).
  - Tạo file `.env` cho Vite chứa `VITE_AUTH_SERVER_URL` và `VITE_AUTH_GROUP_NAME`.
  - Tạo TypeScript types cho Auth flow (`types/auth.ts`, `vite-env.d.ts`).
  - Tạo `authService.ts` — Service layer tách biệt khỏi UI, xử lý HTTP POST lên Auth Server.
  - Mở rộng `authStore.ts` (Zustand) — thêm `commServer`, `isLoading`, `error` state.
  - Viết lại `AuthPage.tsx` — Giao diện Login hoàn chỉnh (dark-mode, glassmorphism, spinner, error).
  - Tạo `DashboardPage.tsx` — Trang hiển thị sau đăng nhập, cho thấy CommServer info.
  - Cập nhật `App.tsx` — Thêm route `/dashboard` với ProtectedRoute guard.
- **Kết quả:** `npm run build` passed. Dev server chạy tại `http://127.0.0.1:5173`.
- **Vấn đề:** Auth Server endpoint `/wl/0` trả về 404 — cần Hermes tiếp tục research endpoint chính xác.
- **File đã tạo/sửa:**
  - `src/client/.env` (NEW)
  - `src/client/src/vite-env.d.ts` (NEW)
  - `src/client/src/types/auth.ts` (NEW)
  - `src/client/src/features/auth/authService.ts` (NEW)
  - `src/client/src/features/auth/AuthPage.tsx` (MODIFIED)
  - `src/client/src/features/dashboard/DashboardPage.tsx` (NEW)
  - `src/client/src/store/authStore.ts` (MODIFIED)
  - `src/client/src/App.tsx` (MODIFIED)

---

## 2026-06-08: Reverse-engineer UDP Telemetry Commands

- **Đã làm:**
  - Download + phân tích JS bundle `main.7909d061.js` (2.3MB) từ cloud.ardupilot.org:8001
  - Xác định message type 1005 (RemoteExecute) + command code 108 dùng cho telemetry control
  - Extract đầy đủ payload: Resume (Act:3), Pause (Act:5), Change LVL (Act:4+LVL), Stop (Act:2)
  - Xác định message type 1071 cho UDP proxy status response từ drone
- **Kết quả:** Document `research/udp-telemetry-commands.md` hoàn chỉnh.
- **Vấn đề:** Không có
- **File:** research/udp-telemetry-commands.md

---
