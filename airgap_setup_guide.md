# Hướng dẫn Setup DroneEngage AirGap Server trên Raspberry Pi 4

> **Tài liệu chính thức:** https://cloud.ardupilot.org/srv-install-airgap.html  
> **GitHub Scripts:** https://github.com/DroneEngage/DroneEngage_ScriptWiki

---

## AirGap là gì?

AirGap Server là chế độ triển khai **hoàn toàn offline** của DroneEngage. Thay vì kết nối lên Cloud Server ở nước ngoài (gây delay ~1 giây), toàn bộ hệ thống server được cài đặt **ngay trên Raspberry Pi 4**, tạo thành một mạng LAN khép kín.

### So sánh 2 chế độ

| | Cloud Mode (đang dùng) | AirGap Mode |
|---|---|---|
| **Server** | Ở nước ngoài | Ngay trên Pi 4 |
| **Internet** | Bắt buộc cần | Không cần |
| **Delay** | ~500ms - 1.2s | **~10-50ms** ⚡ |
| **Phụ thuộc** | 4G/WiFi + Cloud | Chỉ cần WiFi nội bộ |
| **Bản đồ** | Tải online realtime | Phải cache trước |

### Kiến trúc mạng AirGap

```
                    ┌──────────────────────────────────────┐
                    │      RASPBERRY PI 4 (AirGap Server)  │
                    │                                      │
                    │  ┌──────────────────────────────┐    │
                    │  │  DroneEngage Services         │    │
                    │  │  • Authentication Server      │    │
                    │  │  • Communication Server       │    │
                    │  │  • WebClient (Port 8001)      │    │
                    │  └──────────────────────────────┘    │
                    │                                      │
                    │  ┌──────────────┐  ┌─────────────┐   │
                    │  │ WiFi AP      │  │ UART        │   │
                    │  │ (phát mạng)  │  │ (kết nối FC)│   │
                    │  └──────┬───────┘  └──────┬──────┘   │
                    └─────────┼─────────────────┼──────────┘
                              │ WiFi            │ Serial
                    ┌─────────▼────────┐  ┌─────▼──────────┐
                    │  Laptop/Tablet   │  │  Flight Ctrl   │
                    │  (User)          │  │  MicroAir H743 │
                    │                  │  │  (ArduCopter)   │
                    │  Truy cập:       │  └────────────────┘
                    │  http://airgap.  │
                    │  droneengage.com │
                    │  :8001           │
                    └──────────────────┘

          ⚡ Tất cả trong mạng LAN — KHÔNG qua Internet ⚡
```

---

## Yêu cầu phần cứng

| Thành phần | Yêu cầu |
|---|---|
| **Board** | Raspberry Pi 4 (4GB hoặc 8GB RAM) |
| **Hệ điều hành** | Raspberry Pi OS 64-bit (Bullseye trở lên) |
| **Ổ lưu trữ** | USB 3.0 Flash Drive (khuyến nghị, nhanh hơn SD card) |
| **Mạng** | WiFi onboard của Pi 4 (dùng làm Access Point) |
| **Kết nối ban đầu** | Cáp Ethernet (để SSH vào Pi lúc cài đặt) |

---

## Các bước Setup chi tiết

### Bước 1: Chuẩn bị Raspberry Pi

```bash
# 1. Flash Raspberry Pi OS 64-bit lên USB/SD card bằng Raspberry Pi Imager
# 2. Bật SSH trong cài đặt (hoặc tạo file "ssh" rỗng trong boot partition)
# 3. Cắm cáp Ethernet vào Pi để có Internet tạm thời (dùng để tải script)
# 4. Boot Pi và SSH vào
ssh pi@<địa_chỉ_IP_của_Pi>
```

### Bước 2: Đặt IP tĩnh cho Pi

```bash
sudo nano /etc/dhcpcd.conf
```

Thêm vào cuối file:
```
interface wlan0
static ip_address=192.168.1.161/24
static routers=192.168.1.1
static domain_name_servers=127.0.0.1
```

> **Lưu ý:** IP mặc định trong script là `192.168.1.161`. Nếu muốn đổi, phải sửa cả trong script trước khi chạy.

### Bước 3: Tải và chạy script cài đặt AirGap Server

Có **2 lựa chọn** tùy theo nhu cầu:

#### Lựa chọn A: AirGap + Access Point (KHUYẾN NGHỊ cho Swarm)

Chế độ này biến Pi thành **vừa Server vừa phát WiFi**. Laptop và các drone Follower (nếu có Pi riêng) đều kết nối vào WiFi này.

```bash
cd ~

# Tải script cài server + AP
wget https://raw.githubusercontent.com/DroneEngage/DroneEngage_ScriptWiki/refs/heads/main/server_installation/prepare-airgapped-server_w_ap.sh
chmod +x prepare-airgapped-server_w_ap.sh

# (TÙY CHỌN) Sửa IP tĩnh hoặc domain nếu cần
nano prepare-airgapped-server_w_ap.sh

# Chạy script cài đặt
sudo ./prepare-airgapped-server_w_ap.sh
```

Sau khi script chạy xong, tiếp tục cài Access Point:

```bash
# Tải script tạo Access Point
wget https://raw.githubusercontent.com/DroneEngage/DroneEngage_ScriptWiki/refs/heads/main/helper_scripts/create_ap.sh
chmod +x create_ap.sh

# Chạy script tạo AP
sudo ./create_ap.sh
```

#### Lựa chọn B: AirGap không có AP (dùng router WiFi riêng)

Nếu bạn đã có router WiFi riêng và muốn Pi chỉ làm server:

```bash
cd ~

# Tải script cài server (không có AP)
wget https://raw.githubusercontent.com/DroneEngage/DroneEngage_ScriptWiki/main/server_installation/prepare-airgapped-server.sh
chmod +x prepare-airgapped-server.sh

# Chạy script
sudo ./prepare-airgapped-server.sh
```

### Bước 4: Kiểm tra cài đặt thành công

Sau khi script chạy xong, Pi sẽ tự khởi động lại. Kiểm tra các thành phần:

```bash
# Kiểm tra các service đang chạy
sudo systemctl status droneengage_authenticator
sudo systemctl status droneengage_server
sudo systemctl status droneengage_webclient

# Kiểm tra dnsmasq (Local DNS)
sudo systemctl status dnsmasq
```

### Bước 5: Kết nối và truy cập

1. **Kết nối WiFi:** Dùng laptop/tablet kết nối vào WiFi AP của Pi
   - **SSID mặc định:** `DE_2025`
   - **Mật khẩu mặc định:** `1234567890`

2. **Mở trình duyệt:** Truy cập địa chỉ:
   ```
   http://airgap.droneengage.com:8001/index.html
   ```

3. **Đăng nhập:** Dùng tài khoản được tạo trong `droneengage_authenticator`

---

## Cấu trúc thư mục sau khi cài đặt

```
~/
├── droneengage_authenticator/    # Server xác thực
│   └── server.config             # ← Sửa mật khẩu ở đây
├── droneengage_server/           # Server giao tiếp chính
├── droneengage_webclient/        # Giao diện web GCS
├── map/
│   └── cachedMap/                # ← Bỏ bản đồ offline vào đây
└── ssl/                          # Chứng chỉ SSL tự tạo
```

---

## Cài đặt bản đồ Offline (Local Maps)

Vì AirGap không có Internet, bản đồ không thể tải online. Bạn cần cache bản đồ trước:

### Cách 1: Cache bằng tay
```bash
# Trước khi ngắt Internet, truy cập DroneEngage web và duyệt qua
# các khu vực bạn sẽ bay. Bản đồ sẽ được cache tự động.
# Sau đó copy thư mục cache vào:
~/map/cachedMap/
```

### Cách 2: Tải tile bản đồ bằng script
```bash
# Dùng các công cụ như Mobile Atlas Creator hoặc SAS.Planet
# để tải tile OpenStreetMap/Google Satellite của khu vực bay
# rồi copy vào ~/map/cachedMap/
```

---

## Setup cho Swarm (Nhiều drone)

Khi dùng AirGap cho hệ thống Swarm 3 drone của bạn:

### Kiến trúc Swarm qua AirGap

```
                    ┌──────────────────────────────┐
                    │  Pi 4 - DRONE 1 (LEADER)     │
                    │  • AirGap Server             │
                    │  • DroneEngage Companion      │
                    │  • WiFi AP (phát mạng)        │
                    │  • FC: MicroAir H743          │
                    └──────────┬───────────────────┘
                               │ WiFi AP
                 ┌─────────────┼─────────────┐
                 │             │             │
        ┌────────▼───┐  ┌─────▼─────┐  ┌────▼────────┐
        │ Pi 3+ -    │  │ Pi khác - │  │ Laptop/     │
        │ DRONE 2    │  │ DRONE 3   │  │ Tablet      │
        │ (Follower) │  │ (Follower)│  │ (GCS User)  │
        │            │  │           │  │             │
        │ Companion  │  │ Companion │  │ WebClient   │
        │ Client     │  │ Client    │  │ Browser     │
        │ FC: ???    │  │ FC: ???   │  │             │
        └────────────┘  └───────────┘  └─────────────┘
```

### Cấu hình quan trọng cho Swarm

```bash
# Trên mỗi drone, sửa file cấu hình DroneEngage Companion
# để trỏ về AirGap Server thay vì Cloud:

# File: ~/droneengage_companion/config.json (hoặc tương tự)
# Đổi server URL từ cloud sang IP nội bộ:
# "server": "ws://192.168.1.161:8001"   ← IP của Pi Leader

# Đảm bảo mỗi drone có SYSID_THISMAV khác nhau:
# Drone 1 (Leader): SYSID_THISMAV = 1
# Drone 2 (Follower): SYSID_THISMAV = 2
# Drone 3 (Follower): SYSID_THISMAV = 3
```

---

## Xử lý sự cố thường gặp

| Vấn đề | Nguyên nhân | Giải pháp |
|---|---|---|
| Không vào được web | dnsmasq chưa chạy | `sudo systemctl restart dnsmasq` |
| WiFi AP không phát | create_ap chưa cài đúng | Chạy lại `sudo ./create_ap.sh` |
| Drone không hiện trên bản đồ | Companion chưa trỏ về IP local | Sửa config trỏ về `192.168.1.161` |
| Bản đồ trắng | Chưa cache map offline | Tải tile bản đồ vào `~/map/cachedMap/` |
| Không SSH được sau khi bật AP | IP đã đổi sang static | Kết nối vào WiFi AP rồi SSH qua `192.168.1.161` |
| Lag khi 3 drone cùng hoạt động | Pi 4 quá tải | Giảm Stream Rate (`SRx_`) như đã cấu hình |

---

## Checklist trước khi ra hiện trường bay

- [ ] AirGap Server đã chạy ổn định trên Pi 4
- [ ] WiFi AP phát mạng bình thường
- [ ] Bản đồ khu vực bay đã được cache offline
- [ ] Laptop kết nối vào WiFi AP và mở được WebClient
- [ ] Tất cả drone hiện trên bản đồ với đúng System ID
- [ ] Đã test SITL simulation qua AirGap trước khi bay thật
- [ ] Pin đã sạc đầy (3S 2200mAh)
- [ ] GPS lock đủ vệ tinh (tối thiểu 8 sats, HDOP < 1.5)
- [ ] Radio Master Pocket đã bind với ELRS receiver

---

> **Tham khảo thêm:**
> - Tài liệu chính thức: https://cloud.ardupilot.org/srv-install-airgap.html
> - Video hướng dẫn: Tìm "DroneEngage AirGap" trên YouTube
> - GitHub Scripts: https://github.com/DroneEngage/DroneEngage_ScriptWiki
