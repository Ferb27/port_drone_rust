# Phase 4: Kế Hoạch Triển Khai

> Ngày lập: 22/06/2026
> Dựa trên: CTO feedback, trạng thái hiện tại (PROGRESS.md), research mới nhất

---

## Bối Cảnh

CTO yêu cầu:
- **C/C++ bắt buộc** cho production code (MAVLink, UDP routing, GCS engine)
- React/TS frontend giữ lại — OK vì là "Deploy Web"
- **Vấn đề delay 4G ~1 giây** cần giải quyết gấp
- Mục tiêu dài hạn: IP-Based GCS Platform tự build

Kiến trúc mục tiêu:
```
[React Web UI] ←WebSocket→ [C++ GCS Engine] ←UDP/MAVLink→ [Pi trên drone] ←UART→ [FC]
```

---

## TRACK 1: Fix Delay 4G (Ưu tiên CAO NHẤT)

### Mục tiêu
Giảm delay từ ~1 giây xuống ~100-300ms bằng cách bỏ điện thoại hotspot, cắm thẳng USB 4G dongle vào Pi.

### Hiện trạng vs Mục tiêu

```
HIỆN TẠI (6 hop):
FC → Pi → WiFi → Điện thoại → 4G → Cloud → 4G → Điện thoại → WiFi → Laptop
Delay: ~1 giây

MỤC TIÊU (3-4 hop):
FC → Pi → USB 4G Dongle → VPS Relay → Laptop (qua internet)
Delay: ~100-300ms
```

### Bước 1: Mua USB 4G Dongle

| Thiết bị | Ghi chú | Giá tham khảo |
|----------|---------|---------------|
| **Huawei E3372** | Plug & play trên Pi, không cần driver | ~400-600k |
| Huawei E3370 | Cần wvdial để dial-up | ~300-400k |
| SIM 4G Vietnamobile/Viettel | Data plan riêng cho drone | ~50-100k/tháng |

Khuyến nghị: **Huawei E3372** — cắm vào là Pi nhận ngay, tạo interface `eth1` hoặc `wwan0`.

### Bước 2: Cấu Hình Pi

**2a. Kiểm tra dongle được nhận:**
```bash
lsusb | grep Huawei
# Expected: 12d1:1506 Huawei Technologies Co., Ltd. Modem/Networkcard

dmesg | tail -20
# Tìm interface mới (eth1, wwan0, hoặc ppp0)
```

**2b. Với E3372 (NCM/ECM mode - plug & play):**
```bash
# Dongle tự tạo interface eth1, cần DHCP:
sudo dhclient eth1
# Hoặc cấu hình tự động qua /etc/network/interfaces
```

**2c. Với E3370 (cần wvdial):**
```bash
sudo apt install wvdial
# File /etc/wvdial.conf đã có sẵn, test:
sudo wvdial
# Auto-connect: systemd service + udev rule
```

### Bước 3: Cài mavlink-router trên Pi

```bash
sudo apt install git meson ninja-build pkg-config gcc g++ systemd
git clone https://github.com/mavlink-router/mavlink-router.git
cd mavlink-router
git submodule update --init --recursive
meson setup build .
ninja -C build
sudo ninja -C build install
```

**Config `/etc/mavlink-router/main.conf`:**
```ini
[General]
TcpServerPort=5760
ReportStats=false
MavlinkDialect=common

[UartEndpoint serial0]
Device=/dev/serial0
Baud=921600

[UdpEndpoint drone_engage]
Mode=normal
Address=127.0.0.1
Port=14550

[UdpEndpoint vps_relay]
Mode=normal
Address=<VPS_IP>
Port=14550
```

Route: `FC → UART → mavlink-router → (1) localhost:14550 cho DroneEngage + (2) VPS:14550 cho GCS từ xa`

### Bước 4: VPS Relay Server

Thuê VPS Singapore (low latency về VN ~30-50ms):
- **Vultr / DigitalOcean Singapore:** ~$6/tháng
- Hoặc **VPS Việt Nam** (VietIDC, Nhân Hòa) nếu cần latency thấp hơn nữa

**Cài đặt trên VPS:** UDP proxy đơn giản bằng `socat` hoặc `mavlink-router`:
```bash
# Option A: socat (đơn giản nhất)
socat UDP4-LISTEN:14550,fork,reuseaddr UDP4:<drone_ip>:14550

# Option B: mavlink-router (mạnh hơn, có filter)
mavlink-routerd -e <gcs_ip>:14550 0.0.0.0:14550
```

### Bước 5: Đo Latency & So Sánh

```bash
# Trước khi fix
ping -c 10 <cloud_server>

# Sau khi fix (qua 4G dongle)
ping -c 10 <vps_ip>

# Test MAVLink round-trip: gửi HEARTBEAT, đo thời gian nhận response
```

---

## TRACK 2: C++ GCS Backend Setup

### Mục tiêu
Tạo nền tảng C++ cho IP-GCS Platform — thay thế dần logic Python/TS hiện tại.

### Bước 1: Cài MAVSDK C++

**Trên WSL (dev environment):**
```bash
# Ubuntu 24.04
wget https://github.com/mavlink/MAVSDK/releases/download/v3.0.0/libmavsdk-dev_3.0.0_ubuntu24.04_amd64.deb
sudo dpkg -i libmavsdk-dev_3.0.0_ubuntu24.04_amd64.deb
```

**Trên Raspberry Pi (target):**
```bash
# ARM64
wget https://github.com/mavlink/MAVSDK/releases/download/v3.0.0/libmavsdk-dev_3.0.0_ubuntu24.04_arm64.deb
sudo dpkg -i libmavsdk-dev_3.0.0_ubuntu24.04_arm64.deb
```

### Bước 2: Tạo C++ Project Structure

```
gcs_engine/                    ← C++ backend mới
├── CMakeLists.txt
├── src/
│   ├── main.cpp               ← Entry point
│   ├── gcs_server.h/cpp       ← WebSocket server cho React UI
│   ├── mavlink_bridge.h/cpp   ← Kết nối MAVLink (UDP) đến drone
│   ├── drone_manager.h/cpp    ← Quản lý nhiều drone
│   └── mission_engine.h/cpp   ← Auto takeoff/landing, mission logic
├── tests/
│   └── test_mavlink.cpp
└── README.md
```

### Bước 3: CMakeLists.txt Mẫu

```cmake
cmake_minimum_required(VERSION 3.16)
project(gcs_engine VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Tìm MAVSDK
find_package(MAVSDK REQUIRED)

# Boost (cho WebSocket server)
find_package(Boost REQUIRED COMPONENTS system)

add_executable(gcs_engine
    src/main.cpp
    src/gcs_server.cpp
    src/mavlink_bridge.cpp
    src/drone_manager.cpp
    src/mission_engine.cpp
)

target_link_libraries(gcs_engine
    MAVSDK::mavsdk
    Boost::system
    pthread
)
```

### Bước 4: Module Đầu Tiên — Kết Nối MAVLink UDP

File `src/mavlink_bridge.cpp` đầu tiên:

```cpp
#include <mavsdk/mavsdk.h>
#include <mavsdk/plugins/telemetry/telemetry.h>
#include <iostream>

int main() {
    mavsdk::Mavsdk mavsdk;
    
    // Kết nối đến drone qua UDP (SITL hoặc mavlink-router trên Pi)
    auto conn = mavsdk.add_any_connection("udpin://0.0.0.0:14540");
    if (conn != mavsdk::ConnectionResult::Success) {
        std::cerr << "Connection failed" << std::endl;
        return 1;
    }
    
    // Chờ drone xuất hiện
    auto system = mavsdk.first_autopilot(3.0);
    if (!system) {
        std::cerr << "No drone found" << std::endl;
        return 1;
    }
    
    // Subscribe telemetry
    auto telemetry = mavsdk::Telemetry{system.value()};
    telemetry.subscribe_position([](mavsdk::Telemetry::Position pos) {
        std::cout << "Drone at: " << pos.latitude_deg 
                  << ", " << pos.longitude_deg 
                  << ", " << pos.relative_altitude_m << "m" << std::endl;
    });
    
    std::this_thread::sleep_for(std::chrono::seconds(30));
    return 0;
}
```

### Bước 5: Build & Test Với SITL

```bash
# Terminal 1: Chạy ArduPilot SITL
sim_vehicle.py -v ArduCopter --console --map

# Terminal 2: Build + chạy C++ app
cd gcs_engine
cmake -Bbuild -S.
cmake --build build -j4
./build/gcs_engine
```

---

## TRACK 3: Tích Hợp React Frontend ↔ C++ Backend

### Kiến trúc

```
[React UI] ←WebSocket (JSON) → [C++ GCS Engine] ←UDP/MAVLink→ [Pi + Drone]
```

- React UI giữ nguyên hiện tại (map, mission planning, swarm display)
- C++ backend nhận lệnh từ React qua WebSocket → chuyển thành MAVLink command → gửi đến drone
- C++ backend nhận telemetry từ drone → chuyển thành JSON → gửi lên React

### WebSocket Protocol (JSON)

```json
// React → C++: Arm drone
{"cmd": "arm", "drone_id": 1}

// React → C++: Fly to
{"cmd": "goto", "drone_id": 1, "lat": 10.85, "lng": 106.77, "alt": 50}

// C++ → React: Telemetry update
{"type": "telemetry", "drone_id": 1, "lat": 10.851, "lng": 106.772, "alt": 49.5, "battery": 87}

// C++ → React: Status
{"type": "status", "drone_id": 1, "armed": true, "mode": "GUIDED"}
```

---

## LỘ TRÌNH THỰC HIỆN

| Tuần | Track 1 (Fix 4G) | Track 2 (C++ Backend) |
|------|-------------------|----------------------|
| **1** | Mua E3372, test cắm Pi, lên mạng | Cài MAVSDK, chạy takeoff_and_land SITL |
| **2** | Cài mavlink-router, config UART→UDP | Viết mavlink_bridge.cpp, test với SITL |
| **3** | Thuê VPS, setup UDP relay | Viết drone_manager + mission_engine |
| **4** | Đo latency, so sánh trước/sau | Tích hợp WebSocket server (Boost.Beast) |
| **5** | End-to-end test: GCS → VPS → Pi → FC | Kết nối React UI với C++ backend |
| **6** | Auto takeoff/landing test qua 4G | Multi-drrone coordination test |

---

## TÀI LIỆU THAM KHẢO

| Chủ đề | Link |
|--------|------|
| MAVSDK C++ Quickstart | https://mavsdk.mavlink.io/main/en/cpp/quickstart.html |
| MAVSDK API Reference | https://mavsdk.mavlink.io/main/en/cpp/api_reference/ |
| mavlink-router GitHub | https://github.com/mavlink-router/mavlink-router |
| 4G modem trên Pi | https://bellergy.com/3-installing-4g-usb-modem-to-raspberry-pi/ |
| mavlink-router setup | https://bellergy.com/6-install-and-setup-mavlink-router/ |
| ArduPilot SITL | https://ardupilot.org/dev/docs/setting-up-sitl-on-linux.html |

---

*Next action: Bắt đầu Track 1 Bước 1 — đặt mua Huawei E3372 + SIM 4G.*
