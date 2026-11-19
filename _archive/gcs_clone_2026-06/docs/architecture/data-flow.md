# Authentication & WebSocket Data Flow

**Date:** 2026-06-07
**Author:** Antigravity
**Status:** Approved

Tài liệu này định nghĩa cách luồng dữ liệu chạy trong dự án khi người dùng đăng nhập và hệ thống kết nối WebSocket nhận Telemetry.

## 1. Luồng Xác Thực (Authentication Flow)

Do API Endpoint của hệ thống DroneEngage có thể thay đổi (giữa AirGap và Cloud), hệ thống sẽ lấy cấu hình từ `.env`:
`VITE_AUTH_SERVER_URL`

### Các bước:
1. Người dùng truy cập `AuthPage.tsx`. Nhập `Email` và `Access Code`.
2. Component gọi hàm `login(email, password)` từ `services/authService.ts`.
3. `authService` dùng `fetch` POST dữ liệu lên `VITE_AUTH_SERVER_URL`.
4. Nếu HTTP 200, lấy Object `cs` (Communication Server info) và `sid` từ JSON.
5. `authService` gọi action `setAuthInfo(cs, sid)` của `useAuthStore` (Zustand).
6. React Router điều hướng người dùng khỏi `AuthPage.tsx` và vào `Dashboard.tsx`.

## 2. Luồng Khởi Tạo WebSocket (Connection Flow)

Để tránh render lại UI không cần thiết, logic WebSocket KHÔNG được đặt trực tiếp trong React Component.
Sẽ có một class `ProtocolManager` đóng vai trò Singleton.

### Các bước:
1. Trong `Dashboard.tsx`, dùng `useEffect` (chạy 1 lần) để lắng nghe sự thay đổi của `useAuthStore`.
2. Khi `useAuthStore` có `cs` (host, port, temp_key), gọi hàm `ProtocolManager.connect(cs)`.
3. `ProtocolManager` tạo kết nối `new WebSocket("wss://<host>:<port>")`.
4. Ngay khi `onopen`, gửi gói tin JSON Authentication Message (msg 1004) kèm theo `temp_key` (`cs.f`) để server cho phép nhận dữ liệu.

## 3. Luồng Dữ Liệu Real-time (Telemetry Routing)

**Nguyên tắc cốt lõi:** Tuyệt đối không lưu các biến thay đổi tần số cao (tọa độ GPS, IMU) vào Context chung của React.

### Cơ chế:
1. Khi có message mới từ server, hàm `onmessage` của `ProtocolManager` được kích hoạt.
2. Hệ thống kiểm tra trường `_ty`.
   - Nếu `_ty == 'c'` và `messageType == 1002` (GPS):
     `ProtocolManager` gọi hàm `useDroneStore.getState().updateGPS(partyID, lat, lng)`.
   - Nếu `_ty == 'c'` và `messageType == 1004` (Identity):
     `ProtocolManager` gọi hàm `useDroneStore.getState().addOrUpdateDrone(partyID, info)`.
3. Component `DroneMarker` (trên Leaflet Map) dùng hooks của Zustand để subscribe trực tiếp vào đúng `partyID` của nó.
```tsx
// Ví dụ subscribe tối ưu trong component:
const { lat, lng } = useDroneStore(state => state.drones[myId].gps);
```
Chỉ có component `DroneMarker` đó bị re-render khi GPS của nó thay đổi.
