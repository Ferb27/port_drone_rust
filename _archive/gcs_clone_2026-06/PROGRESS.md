# Tiến Trình Dự Án — Clone Drone Engage

Đồ án tốt nghiệp: **Hệ thống Drone Bầy Đàn Tự Hành dựa trên DroneEngage**
Team: Hermes (Research) · Antigravity (Architecture) · Codex (Coding)
Repo: https://github.com/Ferb27/clone_drone_engage

---

## Tổng Quan Tiến Độ

| Phase | Mô tả | Trạng thái |
|-------|-------|------------|
| Phase 0 | Research & Docs | ✅ Hoàn thành |
| Phase 1 | React Skeleton + Auth Flow | ✅ Hoàn thành |
| Phase 2 | Simulator + Swarm AI + UI | ✅ Hoàn thành |
| Phase 3 | Mission Planning + E2E Test | ✅ Hoàn thành |
| Phase 4 | Thực chiến (Pi thật / Cloud) | 🔜 Đang triển khai |

---

## Timeline Chi Tiết

### 2026-06-07 — Khởi Tạo Dự Án (Phase 0-2)

| Giờ | Agent | Hành động |
|-----|-------|-----------|
| 17:40 | Hermes | Research toàn bộ DroneEngage docs + GitHub repos. Viết README.md (747 dòng). Tạo system prompts cho 3 agent. |
| 17:45 | Codex | Đọc docs, khởi tạo React Vite TypeScript project trong `src/client/`. |
| 17:47 | Codex | Pin React 18.3.1 (không dùng React 19). Cài dependencies: Zustand, React Router v6, Leaflet, Tailwind CSS. |
| 18:23 | Antigravity | Thiết kế `docs/architecture/overview.md` — Tech stack, folder structure, data flow. User duyệt plan. |
| 18:53 | Antigravity | Tạo `config/.env` với credentials. Viết `scripts/test_auth.mjs` test kết nối Auth Server. |
| 19:00 | Codex | Tạo skeleton: `App.tsx`, `main.tsx`, `AuthPage.tsx`, `authStore.ts`, types, logger. Build + lint passed. |
| 19:15 | Codex | Tạo toàn bộ cấu trúc thư mục Feature-Sliced. |
| 19:27 | Antigravity | Thiết kế `docs/architecture/data-flow.md`. Auth flow: HTTP POST → temp key → WebSocket. |
| 19:32 | Antigravity | Đảm nhận vai trò Codex (Codex hết token). Viết auth flow: `authService.ts`, `AuthPage.tsx` hoàn chỉnh. |
| 19:37 | Antigravity | **Reverse-engineer Auth Server endpoint.** Download JS bundle 2.3MB từ cloud.ardupilot.org. |
| 19:42 | Antigravity | ⚡ **Phát hiện:** Endpoint thật = `POST /w/wl/` (không phải `/wl/0`). Field names: `acc`, `pwd`, `gr`, `app`, `ver`, `ex`, `at`. |
| 19:43 | Antigravity | **XÁC THỰC THÀNH CÔNG!** Server trả về `e:0`, temp key, session ID. Sửa `authService.ts` với endpoint chính xác. |
| 19:46 | Antigravity | Fix lỗi "upgrade Mobile App" — đổi version field lên 5.0.0. |
| 19:52 | Antigravity | Lên kế hoạch WebSocket + Leaflet Map. Quyết định thêm Simulator vì không có Pi thật. |
| 19:53 | Antigravity | Code `ProtocolManager.ts` + `droneStore.ts` + `DroneMap.tsx`. Simulator bay quanh SPKT (10.850, 106.772). |
| 20:13 | Antigravity | Clone UI DroneEngage gốc (từ ảnh user cung cấp). Layout Grid: Map trái, Control Panel phải. |
| 20:25 | Antigravity | Thêm logic bay: Arm, Takeoff, Land, RTL, Guided + Fly Here. |
| 20:32 | Antigravity | **Tính năng Click Map → Fly Here.** Polyline lộ trình + Marker đích đến. |
| 20:41 | Antigravity | **Swarm AI Phase 1.** 3 drone, Group Commands (Arm All, Takeoff All), Leader-Follower V-Shape. |
| 20:50 | Antigravity | **Đại tu UI.** Đập bỏ layout cũ. Chuyển sang fullscreen Map + Glassmorphism widgets (TopToolbar, FloatingHUD, ActionPanel). |
| 20:58 | Antigravity | Đổi bản đồ sang Esri Satellite (giống Mission Planner). Thêm MISSION SETTINGS panel. |
| 21:02 | Antigravity | Fix lỗi zoom limit của Leaflet với tile vệ tinh. |
| 21:06 | Antigravity | Nâng cấp HUD compass sang Compass Rose chuyên nghiệp. |
| 21:07 | Antigravity | Drone xoay mũi về hướng đích trước khi bay. Icon drone có mũi nhọn trắng chỉ hướng. |

### 2026-06-08 — Mission Planning + Swarm AI (Phase 3)

| Giờ | Agent | Hành động |
|-----|-------|-----------|
| 19:08 | Antigravity | Tạo `LOGGING_GUIDELINES.md` cho các AI assistant tiếp quản dự án. |
| 19:42 | Antigravity | Fix lỗi không drag/zoom bản đồ do UI layer chặn chuột. |
| 19:46 | Antigravity | **Multi-Waypoint Mission Planning.** Thêm `missionWaypoints[]` vào Store. Vẽ Polyline + Marker đánh số. |
| 19:50 | Antigravity | Hoàn thiện Mission Control panel (Add to Mission → Start Mission). Drone tự động bốc waypoint tiếp theo. |
| 19:52 | Antigravity | Tăng tốc độ xoay mũi drone lên 30°/tick cho mượt hơn. |
| 19:58 | Antigravity | **Collision Avoidance + Debug Console.** Sửa lỗi overshoot, thêm lực đẩy (Repulsion), khóa lệnh Fly của Follower. |
| 19:59 | Antigravity | Tạo `logStore.ts` + `DebugConsole.tsx`. Console log real-time ở góc màn hình. |
| 20:03 | Antigravity | Chỉnh vị trí DebugConsole ra bottom-center, không đè FloatingHUD. |
| 20:06 | Antigravity | Nâng cấp Collision Avoidance: bán kính ~15m, lực đẩy tỷ lệ nghịch, phanh gấp nếu <5m. |
| 20:10 | Antigravity | Nghiên cứu Swarm AI algorithms. Đề xuất Virtual Structure + Boids Organic Flocking. |
| 20:15 | Antigravity | **Formation System + Flock Mode.** Dropdown FORM (V-Shape, Line, Column, Circle). Nút FLOCK MODE. |
| 20:17 | Antigravity | Code Boids Reynolds: Separation + Cohesion + Alignment. Build thành công! |
| 20:21 | Antigravity | Fix xung đột Collision Avoidance vs Flocking. Vô hiệu hóa Hard Collision, để Boids xử lý Soft Separation. |
| 20:27 | Antigravity | **Viết lại toàn bộ thuật toán Flock.** Alignment thực sự + lực hút về nhà để bầy không bay mất. |
| 20:27 | Antigravity | Fix Deadlock Formation — gỡ `forwardMoveDist = 0`, nhân lực đẩy x2. |
| 20:34 | Antigravity | **Headless Test Boids.** Node.js script 5.000 ticks, 5 drone gán sát nhau. Kết quả: 0 crashes, khoảng cách gần nhất 7.87m. |
| 09:16 | Antigravity | **Playwright E2E Test.** `swarm.spec.ts` — login bypass, simulator, flock mode, stress test collision. Kết quả: Pass 100%. |

### 2026-06-17 — Git + GitHub Push

| Giờ | Agent | Hành động |
|-----|-------|-----------|
| 07:42 | Hermes | Review toàn bộ project structure. Khởi tạo Git repo, tạo `.gitignore`. |
| 07:46 | Hermes | Commit đầu tiên: 69 files, 7.881 dòng. Push lên GitHub private repo `Ferb27/clone_drone_engage`. |

---

## Những Gì Đã Hoàn Thành

### Research & Documentation
- README.md: 747 dòng tổng hợp toàn bộ DroneEngage docs + protocol
- 5 research files: auth-server, comm-server, rpi-admin-tools, udp-telemetry, udp-telemetry-commands
- 2 architecture docs: overview.md, data-flow.md
- Logging guidelines cho các AI assistant

### Web Client (`src/client/`)
- **Framework:** React 18 + Vite + TypeScript + Tailwind CSS
- **State:** Zustand (4 stores: auth, drone, telemetry, log)
- **Map:** Leaflet + react-leaflet (Esri Satellite tiles)
- **Auth:** Reverse-engineered endpoint `/w/wl/` — xác thực thật với DroneEngage cloud server thành công

### Core Features
| Tính năng | File chính | Dòng code |
|-----------|-----------|-----------|
| Protocol Manager + Simulator | `ProtocolManager.ts` | 558 |
| Drone Map + Collision Detection | `DroneMap.tsx` | 330 |
| Action Panel (điều khiển) | `ActionPanel.tsx` | 166 |
| Auth Service + Auth Page | `authService.ts` + `AuthPage.tsx` | 126 + ~200 |
| Drone Store (Zustand) | `droneStore.ts` | 131 |
| Telemetry Control | `TelemetryControlPanel.tsx` | — |

### Swarm AI Algorithms
- **Virtual Structure:** V-Shape, Line, Column, Circle formations
- **Boids Organic Flocking:** Reynolds algorithm (Separation + Cohesion + Alignment)
- **Collision Avoidance:** Pixel-based (<40px), Hard & Soft modes
- **Mission Planning:** Multi-waypoint với auto-advance

### Testing
- **Headless Boids Test:** 5.000 ticks, 5 drones → 0 crashes, min distance 7.87m
- **Playwright E2E:** `swarm.spec.ts` — login bypass → simulator → flock mode → 300s stress test → Pass

---

## Phần Chưa Làm (Next Steps)

| Ưu tiên | Task | Mô tả |
|---------|------|-------|
| 🔴 Cao | Kết nối Pi thật | Test WebSocket với Raspberry Pi + ArduPilot |
| 🔴 Cao | WebRTC Video | Tích hợp camera stream từ drone |
| 🟡 Trung | GamePad | HTML5 Gamepad API để bay bằng tay cầm |
| 🟡 Trung | UDP Telemetry | Forward telemetry ra Mission Planner/QGC |
| 🟡 Trung | Geo-Fence | Hiển thị + chỉnh sửa geo-fence trên map |
| 🟢 Thấp | i18n | Đa ngôn ngữ (Anh/Việt) |
| 🟢 Thấp | Mobile | Responsive cho điện thoại |
| 🟢 Thấp | Dark/Light theme | Theme switcher |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 18 + Vite + TypeScript |
| Styling | Tailwind CSS 4 |
| State | Zustand |
| Map | Leaflet + react-leaflet 4 |
| Router | React Router v6 |
| Protocol | Andruav WebSocket (reverse-engineered) |
| Testing | Playwright + Vitest |
| Build | `npm run build` → `dist/` |

---
### 2026-06-22 — Phase 4 Kickoff (CTO Feedback)

| Giờ | Agent | Hành động |
|-----|-------|-----------|
| 18:40 | Hermes | Đọc CTO feedback. Phân tích yêu cầu C++ mandate, IP-GCS focus, 4G delay issue. |
| 19:05 | Hermes | Research 4G dongle (Huawei E3372), MAVSDK C++, mavlink-router setup. |
| 19:20 | Hermes | Viết PLAN_PHASE4.md — 2 track song song: Fix 4G Delay + C++ Backend Setup. |

### Key Decisions

- **C/C++ cho backend:** MAVLink handling, UDP routing, GCS engine viết bằng C++/MAVSDK
- **React/TS cho frontend:** Giữ nguyên UI hiện tại, giao tiếp với C++ backend qua WebSocket
- **Fix 4G gấp:** Mua Huawei E3372 USB dongle cắm thẳng Pi, bỏ điện thoại hotspot
- **VPS relay:** Thuê VPS Singapore làm trung gian (thay vì qua cloud DroneEngage nước ngoài)

---
*Last updated: 2026-06-22*
*Team: Hermes (DeepSeek V4 Pro) · Antigravity (Opus 4.6) · Codex (GPT 5.5)*
