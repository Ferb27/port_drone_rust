# Nhận xét từ Giám đốc Công nghệ (CTO) & Mục tiêu phấn đấu

> **Ngày phỏng vấn:** 22/06/2026  
> **Mục đích:** Ghi nhận phản hồi từ CTO sau buổi phỏng vấn, xác định rõ hướng đi chuyên môn và mục tiêu phát triển.

---

## 1. Quy tắc bắt buộc về ngôn ngữ lập trình

> ⚠️ **BẮT BUỘC: Mọi dự án hoàn chỉnh phải được viết bằng C/C++**

| Mục đích | Ngôn ngữ cho phép |
|---|---|
| **Dự án hoàn chỉnh (Production)** | C/C++ **(BẮT BUỘC)** |
| **AI / Machine Learning** | C/C++ **(BẮT BUỘC)** — không được dùng Python |
| **Deploy Web** | Được dùng ngôn ngữ web (JS/TS/Python...) |
| **Test nhanh / Prototype** | Được dùng Python để thử nghiệm |

---

## 2. Mảng chuyên sâu được gợi ý: IP-Based GCS / Cellular-Connected UAV Platform

CTO gợi ý tập trung vào mảng **xây dựng nền tảng điều khiển drone (dạng Ground Control Station)** nhưng thay vì truyền tín hiệu qua các giao thức vô tuyến truyền thống (SiK Radio 433/915MHz, Telemetry), thì chuyển sang **truyền qua mạng IP (WiFi / 4G LTE / 5G)**.

### Tên gọi chính thức trong ngành

Mảng này trong ngành công nghiệp drone được gọi bằng nhiều tên, tùy theo góc nhìn:

| Tên gọi | Mô tả |
|---|---|
| **IP-Based Ground Control Station (IP-GCS)** | Trạm điều khiển mặt đất giao tiếp qua mạng IP thay vì sóng radio trực tiếp |
| **Cellular-Connected UAV System** | Hệ thống UAV kết nối qua mạng di động (4G/5G) |
| **BVLOS Communication Platform** | Nền tảng liên lạc cho bay ngoài tầm nhìn (Beyond Visual Line of Sight) |
| **Cloud-Connected / IoT-Integrated Drone Platform** | Drone như một thiết bị IoT giao tiếp với GCS qua Internet |
| **MAVLink-over-IP Telemetry Platform** | Nền tảng đóng gói và truyền tải gói tin MAVLink qua TCP/UDP trên mạng IP |

**→ Tên gọi ngắn gọn và chính xác nhất để giới thiệu bản thân:**  
> *"Tôi chuyên về phát triển **IP-Based GCS Platform** (Nền tảng điều khiển UAV qua mạng IP) cho các ứng dụng BVLOS sử dụng Cellular/WiFi."*

### Các sản phẩm/dự án tham khảo cùng mảng
- **QGroundControl (QGC):** GCS mã nguồn mở viết bằng C++/Qt, hỗ trợ custom build
- **MAVSDK (C++):** Thư viện C++ chuẩn công nghiệp để tương tác với MAVLink drone
- **DroneEngage:** Nền tảng GCS qua mạng (đang dùng trong dự án hiện tại)
- **UAVcast:** Dự án chuyên routing MAVLink qua 3G/4G/LTE
- **Elsight Halo / CUAV LTE-LINK:** Giải pháp phần cứng chuyên dụng cho BVLOS

---

## 3. Các vấn đề kỹ thuật CTO yêu cầu giải quyết

### 3.1 Cất cánh & Hạ cánh tự động (Auto Takeoff / Landing)
- Phải xử lý quy trình **Pre-Arm Check → Arm → Takeoff → Mission → RTL/Land** hoàn toàn tự động từ GCS
- Cần đảm bảo an toàn: kiểm tra GPS lock, battery level, EKF status trước khi cho phép Arm
- Xử lý edge case: mất tín hiệu khi đang hạ cánh, gió mạnh, hạ cánh lệch vị trí

### 3.2 Điều phối 3 drone bay đồng thời đến 3 vị trí khác nhau
- Upload mission riêng biệt cho từng drone dựa trên `SYSID_THISMAV` (System ID)
- Đồng bộ thời điểm cất cánh (Synchronized Takeoff) để tránh va chạm trên mặt đất
- Phân tầng độ cao (Altitude Layering) cho mỗi drone trong giai đoạn transit
- Giám sát realtime vị trí tất cả drone trên cùng 1 bản đồ GCS
- Xử lý Failsafe khi 1 trong 3 drone gặp sự cố (không ảnh hưởng 2 drone còn lại)

### 3.3 Vấn đề truyền tín hiệu qua 4G (ĐANG GẶP - CẦN GIẢI QUYẾT)

**Hiện trạng:** Bị delay ~1 giây khi điều khiển qua mạng 4G

#### Kiến trúc mạng hiện tại

```
                         ┌─────────────────────────────────────┐
                         │         ĐIỆN THOẠI (Hotspot 4G)     │
                         │  ┌───────────┐   ┌───────────────┐  │
                         │  │ WiFi AP   │   │ Modem 4G/LTE  │  │
                         │  │ (phát ra) │──▶│ (kết nối ra)  │  │
                         │  └─────┬─────┘   └───────┬───────┘  │
                         └────────┼─────────────────┼──────────┘
                                  │                 │
                    WiFi          │                 │  4G/LTE
                  ┌───────────────┤                 │
                  │               │                 │
          ┌───────▼───────┐  ┌───▼────────┐   ┌────▼──────────────────┐
          │  Raspberry Pi │  │ Laptop/PC  │   │  DroneEngage Cloud    │
          │  (Companion)  │  │ (User)     │   │  Server               │
          │               │  │            │   │                       │
          │ • DroneEngage │  │ Truy cập   │   │ • Routing telemetry   │
          │   Companion   │  │ Web quản lý│◀──│ • WebClient interface │
          │ • WiFi AP     │  │ DroneEngage│   │ • Mô phỏng SITL      │
          │   (local net) │  │            │   │ • Terminal Pi         │
          └───────┬───────┘  └────────────┘   └───────────────────────┘
                  │ UART
          ┌───────▼───────┐
          │ Flight Ctrl   │
          │ MicroAir H743 │
          │ (ArduCopter)  │
          └───────────────┘
```

**Chức năng đang hoạt động qua kết nối này:**
- ✅ Truy cập web quản lý DroneEngage
- ✅ Bật chế độ mô phỏng (SITL)
- ✅ Cập nhật firmware
- ✅ Xem hiệu năng hoạt động (Performance Monitor)
- ✅ Truy cập terminal của Pi 4 từ xa
- ⚠️ **Nhưng bị delay ~1 giây khi điều khiển realtime**

#### Phân tích nguyên nhân delay

Mỗi gói tin MAVLink phải đi qua **4 chặng (hop)** trước khi đến được giao diện điều khiển:

```
[FC H743] ──UART──▶ [Pi 4] ──WiFi──▶ [Điện thoại] ──4G──▶ [Cloud Server] ──4G──▶ [Điện thoại] ──WiFi──▶ [Laptop User]
  Hop 0 (~1ms)     Hop 1 (~5-20ms)   Hop 2 (~50-150ms)   Hop 3 (~50-150ms)   Hop 4 (~5-20ms)

Tổng Round-trip (khứ hồi): ~200-600ms mỗi chiều × 2 = ~400ms - 1.2s delay
```

**Các yếu tố gây cộng dồn delay:**

| Yếu tố | Mô tả | Delay thêm |
|---|---|---|
| **WiFi buffer trên điện thoại** | Điện thoại ưu tiên tiết kiệm pin, gom nhiều gói nhỏ lại rồi gửi 1 lần (Nagle-like behavior) | +50-200ms |
| **NAT traversal qua nhà mạng** | Nhà mạng VN thường dùng CGNAT (nhiều tầng NAT), gói tin phải xuyên qua nhiều lớp routing | +30-100ms |
| **Cloud Server ở nước ngoài** | DroneEngage server đặt ở nước ngoài, gói tin phải đi quốc tế rồi quay về | +100-300ms |
| **TCP retransmission** | Nếu dùng TCP, mất 1 gói → tất cả gói sau bị chặn chờ gửi lại (head-of-line blocking) | +100-500ms (đột biến) |

#### Tại sao các dự án khác dùng 4G/WiFi mà vẫn ổn?

Vì họ **loại bỏ được các chặng thừa** trong chuỗi trên:

| Họ làm gì | Kết quả |
|---|---|
| Cắm **USB LTE Modem thẳng** vào Pi | Bỏ hop WiFi → Pi, giảm 1 chặng + bỏ buffer điện thoại |
| Dùng **VPS relay server** gần (Singapore/VN) | Bỏ hop đi Cloud nước ngoài, giảm ~200ms |
| Dùng **UDP** thay TCP cho MAVLink | Không bị head-of-line blocking |
| Dùng **mavlink-router** trên Pi | Ưu tiên gói MAVLink ở tầng OS, không bị queue chung với traffic khác |

#### Giải pháp đề xuất (theo thứ tự ưu tiên)

| # | Giải pháp | Kiến trúc mới | Delay dự kiến |
|---|---|---|---|
| 1 | **USB 4G Dongle cắm trực tiếp vào Pi** | `FC → Pi → LTE Modem → VPS → User` (bỏ điện thoại hoàn toàn) | ~100-300ms |
| 2 | **Dedicated LTE Router** | `FC → Pi → WiFi → Router 4G → VPS → User` (thay điện thoại bằng router chuyên dụng có buffer nhỏ) | ~150-350ms |
| 3 | **AirGap Local Server (đã có sẵn!)** | `FC → Pi → WiFi AP → User` (0 chặng qua Internet, mọi thứ trong LAN) | **~10-50ms** ⚡ |
| 4 | **VPS Relay + UDP** | Thuê VPS Singapore, dùng UDP cho MAVLink, TCP cho web | Giảm thêm ~50-100ms |
| 5 | **ArduPilot High Latency Mode** | Bật `HIGH_LATENCY2`: gửi 1 gói tổng hợp thay vì hàng chục stream | Chống nghẽn tốt |

> **Lưu ý:** Giải pháp AirGap Local Server (số 3) là thứ bạn **đã triển khai** trong dự án Swarm hiện tại. Khi bay thực tế tại hiện trường, nếu không cần Internet, đây là phương án có delay thấp nhất vì mọi dữ liệu chỉ chạy trong mạng WiFi nội bộ của Pi.

---

## 4. Lộ trình học tập & Công nghệ cần nắm

### Giai đoạn 1: Nền tảng C++ cho Drone (1-2 tháng)
- [ ] Thành thạo **MAVSDK C++** — thư viện chuẩn công nghiệp để giao tiếp MAVLink bằng C++
- [ ] Đọc và hiểu source code **MAVLink C Library (c_library_v2)** — header-only C library
- [ ] Làm quen **CMake** build system (dùng cho mọi dự án C++ trong hệ sinh thái drone)
- [ ] Chạy thử ví dụ `fly_multiple_drones.cpp` từ MAVSDK trên SITL

### Giai đoạn 2: Xây dựng IP-GCS cơ bản (2-3 tháng)
- [ ] Tích hợp **USB 4G LTE Modem** vào Raspberry Pi, cấu hình mạng trên Linux
- [ ] Viết chương trình C++ nhận/gửi MAVLink qua **UDP socket** (không dùng Python)
- [ ] Triển khai **mavlink-router** trên Pi để route tín hiệu FC ↔ GCS qua IP
- [ ] Dựng **VPS relay server** để làm trung gian kết nối (xử lý NAT traversal)

### Giai đoạn 3: GCS Platform hoàn chỉnh (3-6 tháng)
- [ ] Nghiên cứu **QGroundControl source code** (C++/Qt/QML) để hiểu kiến trúc GCS chuẩn
- [ ] Xây dựng **Custom GCS** bằng C++/Qt với các tính năng:
  - Bản đồ realtime hiển thị vị trí nhiều drone
  - Upload mission riêng cho từng drone
  - Auto Takeoff/Landing sequence
  - Failsafe monitoring dashboard
- [ ] Tích hợp **video streaming** (GStreamer) qua đường truyền cellular
- [ ] Tối ưu latency và xử lý mất gói tin (packet loss recovery)

### Giai đoạn 4: Fleet Management & Swarm nâng cao (6+ tháng)
- [ ] Xây dựng hệ thống **Synchronized Multi-drone Mission** (bay 3+ drone cùng lúc)
- [ ] Implement **Collision Avoidance** bằng C++ cho multi-drone
- [ ] Tích hợp AI inference bằng C++ (TensorRT C++ API thay vì Python)
- [ ] Nghiên cứu **5G / MQTT / WebRTC** cho next-gen drone communication

---

## 5. Tài liệu tham khảo

| Tài liệu | Link |
|---|---|
| MAVSDK C++ Guide | https://mavsdk.mavlink.io/main/en/cpp/ |
| MAVLink C Library | https://mavlink.io/en/mavgen_usage/ |
| QGroundControl Dev Guide | https://dev.qgroundcontrol.com/master/en/ |
| ArduPilot 4G/LTE Telemetry | https://ardupilot.org/copter/docs/common-4g-lte-telemetry.html |
| mavlink-router | https://github.com/mavlink-router/mavlink-router |
| UAVcast (4G routing) | https://docs.uavmatrix.com/ |

---

> **Ghi chú cá nhân:** Mảng IP-Based GCS Platform là giao điểm giữa Embedded Systems, Networking và UAV — đúng với thế mạnh hiện tại (Embedded & IoT + ArduPilot). Tập trung vào C++ và MAVSDK, lấy QGroundControl làm hình mẫu để xây dựng nền tảng riêng.
