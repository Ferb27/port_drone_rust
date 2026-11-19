=== DRONE ENGAGE PROJECT — SESSION HANDOFF (22/06/2026) ===

Bạn đang tiếp quản session làm việc về đồ án tốt nghiệp "Hệ thống Drone Bầy Đàn Tự Hành dựa trên DroneEngage".

## BỐI CẢNH DỰ ÁN

Sinh viên Việt Nam, học Embedded → Robotics chuyển ngành. Biết C và Python căn bản. Đang làm đồ án tốt nghiệp về drone swarm.

Dự án: Clone Drone Engage — tự build web client + GCS riêng để điều khiển drone qua 4G/LTE. Dùng hạ tầng cloud của DroneEngage (Auth Server + Communication Server) nhưng tự viết client. Team 3 AI agent: Hermes (research), Antigravity (architecture), Codex (coding).

Repo: github.com/Ferb27/clone_drone_engage (private)
Thư mục: E:\project\clone_drone_engage\ (map sang WSL: /mnt/e/project/clone_drone_engage/)

## PHẦN CỨNG

- Drone 1 (5"): FC MicroAir H743v2, ESC Bluejay 45A, Motor MT 2204 2300kv, GPS BZ251, RX ELRS RP1, Pin 3S 2200mAh, Companion: Raspberry Pi 4
- Drone 2 (7"): ESC Bluejay 60A, Motor Sunsky X2216 1100kv, GPS BZ251, RX ELRS Nano, Pin 3S 2200mAh, Companion: Raspberry Pi 3+
- Drone 3: chưa có
- Transmitter: RadioMaster Pocket

## TRẠNG THÁI HIỆN TẠI

Phase 0-3 đã hoàn thành:
- Research toàn bộ DroneEngage protocol (Andruav WebSocket)
- React/TypeScript web client với Leaflet map, simulator 3 drone
- Swarm AI (Boids Reynolds: Separation + Cohesion + Alignment)
- Formation system (V-Shape, Line, Column, Circle)
- Mission Planning (multi-waypoint, auto-advance)
- E2E test (Playwright) — pass
- Auth flow test thành công với DroneEngage cloud server thật
- Đã push lên GitHub private repo

Codebase: ~7,881 dòng, 69 files, React 18 + Vite + TypeScript + Tailwind CSS + Zustand + Leaflet

## CTO FEEDBACK (file: cto_feedback_and_goals.md)

Vừa nhận được phản hồi từ CTO sau buổi phỏng vấn. Các điểm chính:

1. **C/C++ BẮT BUỘC** cho mọi dự án hoàn chỉnh (production). Python chỉ dùng để test nhanh/prototype. Web deploy được dùng JS/TS.
   → React frontend giữ lại. Backend logic (MAVLink handling, UDP routing, GCS engine) phải viết lại bằng C++.

2. **Mảng chuyên sâu:** IP-Based GCS Platform — trạm điều khiển drone qua mạng IP (WiFi/4G/5G) thay vì sóng radio truyền thống.

3. **3 vấn đề kỹ thuật cần giải quyết:**
   a) Auto Takeoff/Landing hoàn toàn tự động từ GCS
   b) Điều phối 3 drone bay đồng thời đến 3 vị trí khác nhau
   c) **Vấn đề delay ~1 giây khi điều khiển qua 4G** (đang gặp thực tế)

4. **Lộ trình 4 giai đoạn:** C++/MAVSDK (1-2 tháng) → IP-GCS cơ bản (2-3 tháng) → GCS Platform hoàn chỉnh (3-6 tháng) → Fleet Management (6+ tháng)

5. **Kiến trúc mục tiêu:**
   ```
   [React Web UI] ←WebSocket→ [C++ GCS Engine] ←UDP/MAVLink→ [Pi trên drone] ←UART→ [Flight Controller]
   ```

## VIỆC ĐÃ LÀM TRONG SESSION NÀY

1. Đọc + phân tích toàn bộ cto_feedback_and_goals.md
2. Đọc PROGRESS.md, hardware_config.md, README.md
3. Load 2 skills: drone-engage-andruav + drone-engage-reverse-engineer
4. Research:
   - MAVSDK C++ quickstart: cài .deb, build example takeoff_and_land, test với SITL
   - mavlink-router: build từ source, config UART→UDP, route MAVLink
   - Huawei E3372 USB 4G dongle: plug & play trên Pi, bỏ điện thoại hotspot
   - GCS career landscape: tên nghề, mức lương, tech stack, công ty tuyển dụng
5. Tạo PLAN_PHASE4.md — kế hoạch 2 track song song:
   - Track 1: Fix delay 4G (E3372 dongle + VPS relay)
   - Track 2: Setup C++ backend (MAVSDK + CMake + WebSocket server)
6. Cập nhật PROGRESS.md
7. Tạo research/gcs-career-landscape.md

## CÂU HỎI MỞ (CHƯA GIẢI QUYẾT)

Người phỏng vấn (CTO) nói ngành drone có **2 mảng chính**. Sinh viên thuộc về một mảng có tên tiếng Anh **bắt đầu bằng chữ "am"**. Không nhớ chính xác từ đó là gì.

Đã thử research nhưng chưa tìm ra. Các giả thuyết đã loại trừ: Air Mobility (bắt đầu bằng "ai"), Autonomous ("au"), Avionics ("av"), Embedded ("em"), Amphibious (đã check - không phải), Amateur (đã check - không phải).

CẦN LÀM: Tìm ra từ tiếng Anh trong ngành drone bắt đầu bằng "am" — đó là tên một trong hai mảng chính của ngành.

## FILE QUAN TRỌNG TRONG DỰ ÁN

- E:\project\clone_drone_engage\PLAN_PHASE4.md — kế hoạch phase 4 mới tạo
- E:\project\clone_drone_engage\cto_feedback_and_goals.md — feedback CTO
- E:\project\clone_drone_engage\PROGRESS.md — tiến độ dự án
- E:\project\clone_drone_engage\README.md — tổng quan dự án
- E:\project\clone_drone_engage\hardware_config.md — cấu hình phần cứng
- E:\project\clone_drone_engage\research\gcs-career-landscape.md — research về nghề GCS

## NEXT STEPS (theo PLAN_PHASE4.md)

Track 1 — Fix delay 4G:
- Mua Huawei E3372 USB 4G dongle (~400-600k VND)
- Cắm thẳng vào Pi, bỏ điện thoại hotspot
- Cài mavlink-router trên Pi
- Thuê VPS Singapore làm relay
- Đo latency trước/sau

Track 2 — C++ Backend:
- Cài MAVSDK C++ library trên WSL
- Tạo CMake project structure (gcs_engine/)
- Viết module MAVLink UDP bridge đầu tiên
- Test với ArduPilot SITL
- Tích hợp WebSocket server cho React UI

Tech stack mục tiêu: C++17, MAVSDK, Boost.Beast (WebSocket), CMake, mavlink-router
