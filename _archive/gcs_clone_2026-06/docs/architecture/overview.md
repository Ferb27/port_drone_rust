# Architecture Overview

**Date:** 2026-06-07
**Author:** Antigravity
**Status:** Approved

Tài liệu này định nghĩa kiến trúc tổng thể cho web client của Clone Drone Engage, phục vụ mục đích điều khiển drone bầy đàn (swarm) dựa trên giao thức Andruav.

## 1. Tech Stack
Dự án web client được xây dựng theo kiến trúc Single Page Application (SPA) nhằm mang lại trải nghiệm mượt mà, realtime.

- **Framework:** React 18 + Vite.
- **Ngôn ngữ:** TypeScript (bắt buộc, giúp type-safe cho các payload WebSocket phức tạp).
- **State Management:** Zustand.
- **Routing:** React Router v6.
- **Styling:** Tailwind CSS + Radix UI (hoặc Headless UI).
- **Map / GIS:** Leaflet + react-leaflet.
- **Networking:**
  - `fetch` API cho các tác vụ HTTP REST (giao tiếp Auth Server).
  - `WebSocket` API thuần cho giao tiếp realtime (giao tiếp Communication Server).
- **Testing:** Vitest + React Testing Library.

## 2. Directory Structure (`src/client`)

Dự án sử dụng Feature-Sliced Design / Domain-Driven structure để đảm bảo tính module hóa và dễ bảo trì:

```text
src/client/
├── assets/          # Static files (images, icons)
├── components/      # UI Components dùng chung (Button, Modal, Card)
├── features/        # Các module tính năng độc lập, chứa logic và UI riêng
│   ├── auth/        # Logic Login, parse temp key
│   ├── map/         # Bản đồ Leaflet, hiển thị Drone Markers, Geofence
│   ├── swarm/       # Bảng điều khiển bầy đàn, formation setup
│   └── telemetry/   # Hiển thị thông số GPS, Battery, HUD
├── core/            # Logic lõi, không phụ thuộc UI
│   ├── protocol/    # Andruav Message parsing và builder (1002, 1004, 1010...)
│   ├── webrtc/      # Logic xử lý streaming camera
│   └── gamepad/     # Xử lý HTML5 Gamepad API
├── hooks/           # Custom React Hooks (vd: useWebSocket, useDrone)
├── store/           # Zustand stores (chia nhỏ: authStore, droneStore, commStore)
├── types/           # TypeScript Interfaces & Enums (định nghĩa chuẩn Andruav)
├── utils/           # Helper functions (toán học, format tọa độ, logger)
├── App.tsx          # Root Component & Providers
└── main.tsx         # Entry point
```

## 3. Data Flow & State Management

**Vấn đề:** Giao thức Andruav gửi dữ liệu Telemetry (GPS, IMU, Battery) liên tục qua WebSocket, nếu quản lý state không tốt sẽ dẫn đến re-render toàn bộ ứng dụng gây giật lag.

**Giải pháp (với Zustand):**
1. **Tiếp nhận:** `Protocol Module` (nằm trong `core/protocol`) duy trì một Singleton WebSocket connection. Khi có message tới, nó parse JSON.
2. **Cập nhật State:** `Protocol Module` không trả dữ liệu qua React Context, mà gọi trực tiếp action cập nhật của Zustand Store (ví dụ: `useDroneStore.getState().updateDroneLocation(id, lat, lng)`).
3. **Hiển thị (Re-render tối ưu):** Các React Component (như `DroneMarker` trên bản đồ) sẽ *subscribe* trực tiếp vào một node dữ liệu cụ thể trong Zustand store (chỉ lấy đúng tọa độ của Drone ID đó). Nhờ vậy, khi tọa độ đổi, chỉ component `DroneMarker` đó re-render, còn lại toàn bộ màn hình đứng yên.

## 4. Dual-Mode Connection Flow

Web client phải hỗ trợ 2 chế độ: Local (AirGap) và Cloud.
- **Bước 1:** Gửi POST HTTP request tới `Auth Server` (Port 19408).
- **Bước 2:** Nhận về JSON chứa địa chỉ IP/Port của `Communication Server` (Port 9966) và một `temp_login_key`.
- **Bước 3:** Mở WebSocket connection tới Communication Server, gửi package đăng nhập chứa `temp_login_key` + `partyID`.
- **Bước 4:** Bắt đầu nhận message 1004 (Unit Identity) từ các drone trên mạng.

*(Flow chi tiết sẽ được vẽ trong tài liệu `data-flow.md` ở bước sau).*
