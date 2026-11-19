# Clone Drone Engage — Web Client (Đồ án tốt nghiệp)

## Team

| Agent | Model | Vai trò |
|-------|-------|---------|
| **Antigravity** | Opus 4.6 / Gemini 3.1 Pro | Kiến trúc, thiết kế, logic phức tạp |
| **Codex** | GPT 5.5 medium / high | Viết code chính |
| **Hermes** | DeepSeek V4 Pro | Research, web browser, crawl docs, tìm API endpoints, feed info |

## Mục tiêu

**Đồ án tốt nghiệp: Hệ thống Drone Bầy Đàn Tự Hành**

Xây dựng web client riêng cho **DroneEngage** — không chỉ dùng lại web client có sẵn, mà phát triển dựa trên bộ khung hạ tầng của DroneEngage (Authentication Server, Communication Server, Andruav WebSocket protocol). Hệ thống phục vụ điều khiển drone bầy đàn (swarm) qua 4G/LTE.

- Web client tự viết, giao tiếp với server cloud của DroneEngage qua Andruav WebSocket protocol
- Tham khảo source code DroneEngage để hiểu protocol, viết lại theo cách riêng
- Project cá nhân đồ án tốt nghiệp

---

## System Prompts cho 3 Agent

Các system prompt dưới đây được dùng khi spawn agent qua `delegate_task` hoặc tmux.
Mỗi agent có vai trò riêng nhưng cùng tuân thủ quy tắc chung của đồ án.

### Quy tắc chung (áp dụng cho cả 3 agent)

```
=== THESIS RULES (ALL AGENTS) ===

1. NO HALLUCINATION — Đây là đồ án tốt nghiệp, yêu cầu kỹ thuật cao.
   - KHÔNG được bịa ra API endpoint, function, hay config không tồn tại
   - Nếu không chắc chắn về một thông tin, phải ghi rõ "CHƯA XÁC MINH: ..."
   - Luôn trích dẫn nguồn: file nào, dòng nào, docs nào

2. LOG EVERYTHING — Mọi hành động phải được log để debug.
   - Tạo file log riêng: logs/<agent_name>_<date>.log
   - Log format: [TIMESTAMP] [ACTION] detail
   - Log cả lỗi và thành công
   - Code sinh ra phải có error handling đầy đủ, không nuốt exception

3. PROGRESS TRACKING — Ghi lại tiến độ để báo cáo đồ án.
   - Mỗi session bắt đầu: ghi goals vào logs/session_<date>_goals.md
   - Mỗi milestone hoàn thành: ghi vào logs/progress.md với format:
     ## [DATE] Milestone: <tên>
     - Đã làm: ...
     - Kết quả: ...
     - Vấn đề gặp phải: ...
     - File đã thay đổi: ...
   - Dùng git commit message rõ ràng, có ý nghĩa cho báo cáo

4. PROJECT CONTEXT
   - Đồ án: Hệ thống Drone Bầy Đàn Tự Hành dựa trên DroneEngage
   - Mục tiêu: Web client riêng, không dùng lại code DroneEngage
   - Infrastructure: Auth Server (port 19408), Comm Server (port 9966), Andruav WebSocket
   - Hạ tầng DroneEngage là nền tảng, web client của mình là phần phát triển mới
   - Repos tham khảo: github.com/DroneEngage/*
   - Docs: cloud.ardupilot.org/de-index.html

5. CODE QUALITY
   - Code phải có comment giải thích logic
   - Function/variable name rõ ràng, tự document
   - Viết unit test cho các module quan trọng
   - Không copy-paste nguyên đoạn từ source DroneEngage
```

### Hermes (Research Agent)

```
=== HERMES SYSTEM PROMPT ===

Bạn là HERMES — Research Agent trong team 3 người xây dựng web client
cho DroneEngage (đồ án tốt nghiệp: Hệ thống Drone Bầy Đàn Tự Hành).

VAI TRÒ:
- Nghiên cứu protocol, crawl documentation, tìm API endpoints
- Đọc source code DroneEngage (github.com/DroneEngage/*) để hiểu logic
- Cung cấp thông tin chính xác cho Antigravity (architecture) và Codex (coding)
- KHÔNG viết production code — chỉ viết script test/research

QUY TẮC RIÊNG:
- Mọi thông tin đưa ra phải có nguồn: URL docs, file path, line number
- Phân biệt rõ: "ĐÃ XÁC MINH" vs "SUY ĐOÁN" vs "CHƯA RÕ"
- Khi đọc code DroneEngage: mô tả logic, không copy-paste code
- Lưu research findings vào research/<topic>.md
- Nếu docs hoặc code mâu thuẫn: ghi rõ cả hai và đánh dấu cần xác minh

OUTPUT FORMAT:
- Mỗi research session: tạo file research/session_<date>_<topic>.md
- Format: mục tiêu → phương pháp → findings (có source) → kết luận → open questions
- Gửi summary cho 2 agent còn lại qua file docs/feed/<topic>_for_antigravity.md
  hoặc docs/feed/<topic>_for_codex.md

TOOLS: web_search, web_extract, browser_navigate, browser_snapshot,
        read_file (trên DroneEngage repos), search_files
```

### Antigravity (Architecture Agent)

```
=== ANTIGRAVITY SYSTEM PROMPT ===

Bạn là ANTIGRAVITY — Architecture & Design Agent trong team 3 người
xây dựng web client cho DroneEngage (đồ án tốt nghiệp).

VAI TRÒ:
- Thiết kế kiến trúc tổng thể web client
- Quyết định tech stack, component tree, data flow
- Review code từ Codex, đảm bảo đúng kiến trúc
- Phân tích protocol từ research của Hermes, thiết kế module tương ứng
- Giải quyết các vấn đề logic phức tạp

QUY TẮC RIÊNG:
- Mọi thiết kế phải dựa trên protocol ĐÃ XÁC MINH từ Hermes
- KHÔNG thiết kế dựa trên giả định chưa kiểm chứng
- Mỗi quyết định kiến trúc phải có rationale rõ ràng
- Phải chỉ ra trade-off của từng lựa chọn
- Thiết kế phải có sequence diagram hoặc flow chart cho các flow quan trọng

OUTPUT:
- docs/architecture/overview.md — kiến trúc tổng thể
- docs/architecture/components/<component>.md — thiết kế từng component
- docs/architecture/data-flow.md — luồng dữ liệu
- docs/architecture/decisions.md — ADR (Architecture Decision Records)
- Mỗi file phải có: mục đích, dependencies, interface, state management

REVIEW PROCESS:
- Code từ Codex → review theo checklist: đúng protocol, đúng kiến trúc, error handling, test coverage
- Nếu reject: ghi rõ lý do + đề xuất sửa
- Nếu approve: ghi REVIEW PASSED + signature
- Lưu review vào docs/reviews/<feature>_review.md
```

### Codex (Coding Agent)

```
=== CODEX SYSTEM PROMPT ===

Bạn là CODEX — Coding Agent trong team 3 người xây dựng web client
cho DroneEngage (đồ án tốt nghiệp: Hệ thống Drone Bầy Đàn Tự Hành).

VAI TRÒ:
- Viết code cho web client dựa trên thiết kế từ Antigravity
- Implement WebSocket client, UI components, state management
- Viết unit test, integration test
- Fix bug được phát hiện trong quá trình review

QUY TẮC RIÊNG:
- TUYỆT ĐỐI KHÔNG bịa API endpoint, message type, hay protocol
  * Chỉ dùng các message types ĐÃ ĐƯỢC Hermes xác minh (có trong docs/protocol/)
  * Nếu cần endpoint mới → hỏi Hermes research trước
  * Mọi WebSocket message gửi đi phải khớp với Andruav protocol đã document

- CHỈ viết code dựa trên spec từ Antigravity
  * Không tự ý thay đổi kiến trúc
  * Nếu thấy vấn đề → báo Antigravity, không tự sửa

- CODE STANDARDS:
  * Mỗi file phải có header comment: purpose, dependencies, author (Codex), date
  * Function phải có JSDoc đầy đủ
  * Error handling: try-catch + log + user-friendly message
  * Không console.log bừa bãi — dùng logger module
  * Viết test cho mọi function quan trọng (Jest/Vitest)

- GIT:
  * Commit message format: type(scope): description
  * Types: feat, fix, refactor, test, docs, chore
  * Mỗi commit là một đơn vị logic nhỏ, dễ review
  * Branch naming: feat/<feature-name>, fix/<bug-name>

- TRƯỚC KHI CODE:
  * Đọc spec từ Antigravity trong docs/architecture/
  * Kiểm tra protocol từ Hermes trong docs/protocol/
  * Confirm đã hiểu rõ input/output/edge cases
  * Nếu spec chưa rõ → hỏi lại, không đoán

OUTPUT:
- src/client/<module>/ — code chính
- src/client/<module>/__tests__/ — unit test
- Sau mỗi session: ghi summary vào logs/codex_<date>_summary.md
  * File đã tạo/sửa
  * Test result
  * Vấn đề gặp phải
  * Câu hỏi cho Antigravity/Hermes
```

---

## DroneEngage là gì?

DroneEngage là hệ thống companion computer cloud-based cho drone chạy ArduPilot. Nó gồm:

1. **Raspberry Pi** gắn trên drone — chạy các module C++: communication, MAVLink bridge, camera WebRTC
2. **Cloud Server** — Authentication Server + Communication Server (WebSocket relay)
3. **Web Client** — browser-based GCS để điều khiển drone từ xa qua 4G/LTE

Tác giả: **Mohammad Hefny** (GitHub: HefnySco, Telegram: t.me/droneengage)
Website: https://droneengage.com

### Dual-Mode Architecture

```
MODE 1: LOCAL (AirGap) — không cần internet
  Pi phát WiFi AP: SSID=DE_ADMIN, password=droneengage
  IP Pi: 192.168.9.1
  Admin Cockpit: https://192.168.9.1:9090

MODE 2: CLOUD — qua internet (4G/LTE)
  Pi → WebSocket → Communication Server (airgap.droneengage.com:9966)
  Web Client → browser → cùng Communication Server
  Authentication Server (port 19408) cấp temp key
```

---

## GitHub Repositories (chính thức)

Tất cả repo thuộc tổ chức **https://github.com/DroneEngage** — tác giả duy nhất: HefnySco (Mohammad Hefny).

| Repository | Language | Stars | Commits | Purpose |
|------------|----------|-------|---------|---------|
| [droneengage_webclient_react](https://github.com/DroneEngage/droneengage_webclient_react) | JavaScript 89.5% | 8 | 746 | Web Client SPA — React + Bootstrap 5 + Leaflet |
| [droneengage_communication](https://github.com/DroneEngage/droneengage_communication) | C++ 98.5% | 7 | 427 | Main Communication Module (chạy trên Pi) — WebSocket client/server relay |
| [droneengage_mavlink](https://github.com/DroneEngage/droneengage_mavlink) | C 92.6% | 14 | 639 | MAVLink plugin — bridge giữa ArduPilot flight controller và comm module |
| [droneengage_camera](https://github.com/DroneEngage/droneengage_camera) | C++ 94.6% | 7 | 59 | WebRTC Camera Module — stream video từ Pi lên web client |
| [droneegnage_authenticator](https://github.com/DroneEngage/droneegnage_authenticator) | JavaScript 98.8% | 4 | 82 | Authentication Server — Node.js, quản lý user/vehicle, cấp temp key |
| [droneengage_ap_wiki](https://github.com/DroneEngage/droneengage_ap_wiki) | HTML 77.6% | 3 | 65 | Documentation source (Sphinx) → cloud.ardupilot.org |
| [droneengage_databus](https://github.com/DroneEngage/droneengage_databus) | Python/C++/JS | 4 | 80 | Plugin SDK — template cho C++, Python, Node.js modules |

**Latest activity:** Feb 2026. Low public engagement (3-14 stars). Single maintainer.

---

## Documentation (cloud.ardupilot.org)

Trang chủ docs: **https://cloud.ardupilot.org/de-index.html**

### Web Client & Features

| Page | URL |
|------|-----|
| Web Client Overview | https://cloud.ardupilot.org/webclient-whatis.html |
| Web Client Usage | https://cloud.ardupilot.org/de-web-client.html |
| GamePad Navigation | https://cloud.ardupilot.org/webclient-gamepad.html |
| UDP Telemetry Forwarding | https://cloud.ardupilot.org/webclient-udp-telemetry.html |
| SWARM Interface | https://cloud.ardupilot.org/webclient-swarm.html |
| Servo Channels Control | https://cloud.ardupilot.org/webclient-servo.html |
| Configuration (js_siteConfig.js) | https://cloud.ardupilot.org/webclient-configuration.html |

### Camera & Telemetry

| Page | URL |
|------|-----|
| Camera Module | https://cloud.ardupilot.org/de-camera.html |
| Telemetry | https://cloud.ardupilot.org/de-telemetry.html |

### Advanced Features

| Page | URL |
|------|-----|
| Advanced Features Index | https://cloud.ardupilot.org/de-advanced.html |
| Geo-Fencing (server-side) | https://cloud.ardupilot.org/de-geo-fencing.html |
| RC Blocking | https://cloud.ardupilot.org/de-tx-block.html |
| TX Freeze | https://cloud.ardupilot.org/de-tx-freeze.html |
| SWARM Operations | https://cloud.ardupilot.org/de-swarm.html |

### Plugins

| Page | URL |
|------|-----|
| Plugins Overview | https://cloud.ardupilot.org/de-plugins.html |
| SDR Plugin | https://cloud.ardupilot.org/de-plugin-sdr.html |
| GPIO Plugin | https://cloud.ardupilot.org/de-plugin-gpio.html |
| Sound Plugin | https://cloud.ardupilot.org/de-plugin-sound.html |

### Backend Servers

| Page | URL |
|------|-----|
| Authentication Server | https://cloud.ardupilot.org/srv-authentication.html |
| Communication Server | https://cloud.ardupilot.org/srv-communication.html |

### Installation & Admin

| Page | URL |
|------|-----|
| Installation Overview | https://cloud.ardupilot.org/de-install.html |
| Unit Installation | https://cloud.ardupilot.org/de-install-unit.html |
| Ready-Made RPI Image | https://cloud.ardupilot.org/de-software-installation_download.html |
| RPI Admin Tools | https://cloud.ardupilot.org/de-rpi-image-tools.html |
| - WiFi Manager | https://cloud.ardupilot.org/de-rpi-image-tools-wifi.html |
| - Account Setup | https://cloud.ardupilot.org/de-rpi-image-tools-account.html |
| - App Management | https://cloud.ardupilot.org/de-rpi-image-tools-bash.html |
| - Module Updater | https://cloud.ardupilot.org/de-rpi-image-module-updater.html |
| - File Navigator | https://cloud.ardupilot.org/de-rpi-image-tools-navigator.html |
| - Terminal | https://cloud.ardupilot.org/de-rpi-image-tools-terminal.html |
| FAQ | https://cloud.ardupilot.org/de-faq.html |
| Contributing | https://cloud.ardupilot.org/de-contributing.html |

---

## Andruav WebSocket Protocol

### Authentication Flow (3 bước)

```
Bước 1: POST https://airgap.droneengage.com:19408/wl/<groupName>
  Body: { email, password, GCS_ID, app_name, app_ver, ... }

Bước 2: Response JSON:
  {
    "cs": {
      "g": "airgap.droneengage.com",   // Communication Server host
      "h": "9966",                     // Communication Server port
      "f": "<temp_login_key>"          // CONST_CS_LOGIN_TEMP_KEY
    },
    "sid": "<session_id>"
  }

Bước 3: WebSocket connect wss://airgap.droneengage.com:9966
  Gửi temp key + partyID để authenticate
```

**Important:** Auth Server và Comm Server là 2 server riêng biệt.
Auth Server trả về JSON (không redirect), client tự mở WebSocket đến Comm Server.

### Message Envelope

```javascript
// System command (no target)
{ "_ty": "s", "msg": { ... } }

// Communication — group broadcast
{ "_ty": "c", "g": "<groupId>", "messageType": <int>, "sender": "...", ... }

// Communication — individual message to specific unit
{ "_ty": "c", "i": "<partyID>", "messageType": <int>, "sender": "...", ... }
```

### Routing Constants

```javascript
CMDTYPE_SYS   = 's'     // System command
CMDTYPE_COMM  = 'c'     // Communication command
CMD_COMM_GROUP      = 'g'   // Group broadcast target
CMD_COMM_INDIVIDUAL = 'i'   // Individual target
CMD_SYS_PING        = 'ping'
CONST_TARGETS_GCS   = '_GCS_'    // Target all GCS units
CONST_TARGETS_DRONES = '_AGN_'   // Target all drone agents
```

### Andruav Message Type IDs

| ID   | Description | Notes |
|------|-------------|-------|
| 1002 | GPS Position | Lat/lng/alt from drone |
| 1003 | Power/Battery | Voltage, current, remaining |
| 1004 | Unit Identity | `unitName`, `VehicleType`, `m_IsGCS`, `dv` (DE version), `av` (Andruav version), `Permissions`, nested `HomeLocation` |
| 1005 | Remote Command | Execute command on drone |
| 1006 | Image/Still Photo | Binary image data from camera |
| 1007 | Telemetry Data | Flight data (attitude, velocity, etc.) |
| 1008 | Error Message | Error reporting |
| 1010 | Flight Control | Arm/Disarm (`AC: 1`/`AC: 0`), mode change |
| 1012 | Camera List | Available camera enumeration |
| 1013 | IMU Data | Raw IMU (binary) |
| 1014 | Video Frame | Real-time video data |
| 1016 | IMU Statistics | Aggregated IMU metrics |
| 1020 | Drone Report | Mission status report |
| 1022 | Home Location | Home point coordinates |
| 1023 | Geofence | Geofence definition |
| 1035 | Waypoints/Mission | Mission upload/download |

### Binary Message Types

```
CONST_TYPE_AndruavBinaryMessage_Mavlink = 0x2712  // MAVLink packet
IMG                                         // Image data
SDR_SPECTRUM                                // SDR spectrum data
```

Binary format: JSON header as byte array + binary payload concatenated.

---

## Commands Reference (Web Client → Drone)

### Arm / Disarm
```
API_sendCMD(partyID, 1010, { "AC": 1 })   // Arm
API_sendCMD(partyID, 1010, { "AC": 0 })   // Disarm
```

### Flight Modes
```
API_do_FlightMode(unit, mode)
// Modes: GUIDED, AUTO, RTL, LOITER, LAND, ALT_HOLD, STABILIZE
```

### Navigation
```
API_do_FlyHere(lat, lng, alt, velocities)
API_do_CircleHere(lat, lng, alt, radius, turns)
API_do_ChangeAltitude(unit, altitude)
API_do_YAW(unit, degrees, turnRate, clockwise, isRelative)
API_do_ChangeSpeed2(speed, isGroundSpeed, throttle, isRelative)
API_do_Land(unit)
```

### Waypoints / Mission
```
API_uploadWayPoints(unit, eraseFirst, textMission)    // MAVLink text format
API_uploadDEMission(unit, eraseFirst, jsonMission)     // JSON format
API_do_StartMissionFrom(unit, missionNumber)           // 0-based
API_requestWayPoints(unit, enableFCB)
```

### GamePad / RC
```
API_sendRXChannels: { 'R': rudder, 'T': throttle, 'A': aileron, 'E': elevator }
// Axes normalized [-1, 1] → mapped to [0, 1000]
API_engageRX(unit)
API_disengageRX(unit)
API_engageGamePad(unit)
```

### Camera
```
streamVideo(unit, OnOff, number, channel)
takeImage2(target, cameraSource, count, interval, distance)
recordVideo(target, trackId, OnOff)
zoomCamera(target, cameraName, isZoomIn, zoomValue, zoomStep)
```

### Servo / Gimbal
```
API_do_ServoChannel(channel, value)
API_do_GimbalCtrl(pitch, roll, yaw, isAbsolute)
```

### Swarm / P2P / SDR
```
makeSwarm(unit, formationID, horizDist, vertDist)
requestFromDroneToFollowAnother(unit, slaveIndex, leaderPartyID, do_follow)
requestP2P(unit)
requestSDR(unit)
```

---

## Web Client Configuration

File chính: `src/js/js_siteConfig.js` — có thể override bằng `public/config.json`.

### Server Connection
| Field | Default |
|-------|---------|
| `CONST_PROD_MODE_IP` | `airgap.droneengage.com` |
| `CONST_PROD_MODE_PORT` | `19408` |
| `CONST_TITLE` | `Drone Engage` |

### Map
| Field | Default |
|-------|---------|
| `CONST_MAP_LEAFLET_URL` | Mapbox satellite tiles |

### Feature Toggles (set `true` để disable)
| Key | Effect |
|-----|--------|
| `DISABLE_UNIT_NAMING` | Chặn rename unit |
| `DISABLE_UDPPROXY_UPDATE` | Chặn update UDP proxy |
| `DISABLE_SWARM` | Ẩn swarm GUI |
| `DISABLE_SWARM_DESTINATION_PONTS` | Ẩn follower destination icons |
| `DISABLE_P2P` | Ẩn P2P module |
| `DISABLE_SDR` | Ẩn SDR module |
| `DISABLE_GPIO` | Ẩn GPIO module |
| `DISABLE_VOICE` | Ẩn voice module |

### WebRTC ICE Servers
```javascript
CONST_ICE_SERVERS = [
  { urls: 'turn:cloud.ardupilot.org', credential: '1234', username: 'andruav_ap' },
  { urls: "stun:stun1.l.google.com:19302" },
]
```

---

## Raspberry Pi RPI Image

### Download
Pre-built images: https://cloud.ardupilot.org/downloads/RPI_Full_Images/droneengage_rpi/

Hỗ trợ: RPI-Zero W, RPI-Zero 2 W, RPI-4, RPI-5

### Default AP Mode
- SSID: `DE_ADMIN`
- Password: `droneengage`
- Admin URL: `https://192.168.9.1:9090`
- SSH: pi / raspberry

### Admin Cockpit Tools
| Tool | Purpose |
|------|---------|
| WiFi Manager | Cấu hình WLAN hoặc bật AP mode |
| Account Setup | Nhập username + access code, set server URL |
| App Management | Start/stop services, enable autostart, chạy simulator |
| Module Updater | Update từng module (de_comm, de_mavlink, de_camera, de_rpi_gpio, de_tracking, de_sdr) |
| File Navigator | Duyệt và sửa config files |
| Terminal | Linux shell trên Pi |

### Recovery
Nếu mất kết nối WiFi: tạo file `activate_ap.txt` trong boot partition của SD card → reboot → Pi sẽ bật AP mode.

### Key Config File Paths
```
/home/pi/drone_engage/de_comm/de_comm.config.module.json
/home/pi/drone_engage/de_mavlink/de_mavlink.config.module.json
/home/pi/drone_engage/de_camera/de_camera.config.module.json
/home/pi/simulator/sim_de_mavlink_instances/de_comm.1.config.module.json
/home/pi/simulator/sim_de_mavlink_instances/de_comm.2.config.module.json
```

---

## Telemetry & UDP Proxy

### Smart Telemetry Levels
| LVL | Rate |
|-----|------|
| 0 | Full rate |
| 1 | Light optimization |
| 2 | Moderate optimization |
| 3 | Maximum optimization (lowest bandwidth) |

### UDP Telemetry Forwarding
Cho phép kết nối Mission Planner/QGroundControl qua UDP:

```json
// de_mavlink.config.module.json
{
  "udp_proxy_enabled": true,
  "udp_proxy_fixed_port": 15412
}
```

### Per-Message Rate Control (LVL-based)
Mỗi MAVLink message ID có 4 giá trị delay (ms) cho LVL 0-3.
Ví dụ: GPS_RAW_INT (msg 24) → `[0, 800, 1000, 2000]`

---

## Authentication Server Config

File: `server.config` (Node.js)

```json
{
  "server_id": "AndruavAuth",
  "server_ip": "0.0.0.0",
  "server_port": 19408,
  "account_storage_type": "single",    // single | file | db
  "single_account_user_name": "single@airgap.droneengage.com",
  "single_account_access_code": "test",
  "db_users": "./db_users.db",
  "s2s_ws_listening_ip": "127.0.0.1",
  "s2s_ws_listening_port": "19001",
  "enable_SSL": true
}
```

DB Users file format (khi `account_storage_type=file`):
```json
{
  "user1@email.com": { "sid": 1, "pwd": "0001", "isadmin": true, "prm": "0xffffffff" },
  "user2@email.com": { "sid": 1, "pwd": "drone", "isadmin": true, "prm": "0xffffffff" }
}
```
Users cùng `sid` → share cùng drones.

---

## Communication Server Config

File: `server.config` (Node.js)

```json
{
  "server_id": "DE_Lap",
  "server_ip": "::",
  "public_host": "airgap.droneengage.com",
  "server_sid": 0,
  "server_port": 9966,
  "enable_SSL": true,
  "s2s_ws_target_ip": "127.0.0.1",
  "s2s_ws_target_port": "19408",
  "allow_udpproxy_fixed_port": true,
  "command_plugin": "./js_andruav_command_processing"
}
```

**S2S flow:** Communication Server duy trì WebSocket persistent đến Authentication Server.
Khi user/drone auth, Auth Server yêu cầu Comm Server tạo temp key. Client dùng key đó kết nối WebSocket đến Comm Server.

---

## Plugin System (DataBus)

DroneEngage có hệ thống plugin qua DataBus protocol (UDP-based):

```
Plugin (C++/Python/Node) → UDP → de_comm Communicator (port 60000) → Cloud
```

Các plugin có sẵn:
- **de_mavlink** — MAVLink bridge (quan trọng nhất)
- **de_camera** — WebRTC video streaming
- **de_rpi_gpio** — GPIO control
- **de_tracking** — Object tracking
- **de_sdr** — Software Defined Radio
- **de_sound** — Audio playback

SDK: https://github.com/DroneEngage/droneengage_databus

---

## Safety Features

| Feature | Description |
|---------|-------------|
| **RC Blocking** | Pilot ngoài field gạt switch TX → chặn toàn bộ lệnh từ GCS, trả quyền điều khiển về TX. Kênh cấu hình trong `rc_block_channel` (PWM > 1800 = block) |
| **TX Freeze** | Giữ nguyên giá trị throttle/servo hiện tại khi drone bay xa ngoài tầm TX |
| **Geo-Fencing** | Server-side (không phải ArduPilot fence). Green = fly zone, Red = no-fly zone. Quản lý qua Map Editor: https://droneengage.com:8021/mapeditor.html |

---

## Web Client URLs

| URL | Purpose |
|-----|---------|
| https://cloud.ardupilot.org:8001/webclient.html | Main Web Client |
| https://droneengage.com:8021/webclient.html | Alternative Web Client |
| https://droneengage.com:8021/mapeditor.html | Geo-Fence & Mission Editor |
| https://cloud.ardupilot.org:8001/accounts | Account Registration |

---

## Hardware Requirements (RPI Image)

| Use Case | Board | Notes |
|----------|-------|-------|
| Telemetry Only | RPI-Zero W | Lightest, no video |
| Telemetry + Single Camera | RPI-Zero 2 W | Best balance |
| Multi-Camera | RPI-4 / RPI-5 | Most powerful |

Connection to flight controller: Serial (UART), USB, UDP, TCP, Bluetooth/WiFi

---

## Tech Stack (Web Client gốc)

- **React** — UI framework
- **Bootstrap 5** — CSS
- **Leaflet** — Map (Mapbox satellite tiles)
- **WebSocket** — Real-time communication
- **WebRTC** — Video streaming (TURN/STUN)
- **HTML5 Gamepad API** — Controller support
- **i18next** — Đa ngôn ngữ

Dev: `npm install && npm run start` → https://localhost:3000
Build: `npm run build` → `build/`

---

## Kế hoạch Build (Clone Project)

### Phase 0: Research & Protocol Testing ✅ 100%
- [x] Reverse-engineer Auth Server endpoint (`POST /w/wl/`) từ JS bundle gốc
- [x] Verify message flow: login → nhận temp key → WebSocket → nhận 1004 (identity)
- [x] Document toàn bộ Andruav protocol (15 message types) + UDP telemetry commands
- [x] 5 research files: auth-server, comm-server, rpi-admin-tools, udp-telemetry × 2
- [x] Script test kết nối Auth Server (`scripts/test_auth.mjs`)

### Phase 1: React App Skeleton ✅ 100%
- [x] React 18 + Vite + TypeScript + Tailwind CSS 4
- [x] Login page → POST auth server, nhận temp key (endpoint `/w/wl/` đã xác minh)
- [x] WebSocket client (ProtocolManager.ts) → kết nối communication server
- [x] Map view (Leaflet + Esri Satellite) hiển thị vị trí drone
- [x] Zustand stores: authStore, droneStore, telemetryStore, logStore
- [x] Architecture docs: overview.md, data-flow.md

### Phase 2: Core Controls + Simulator ✅ 100%
- [x] Arm/Disarm button (msgType 1005, C:100/C:101)
- [x] Flight mode selector (Guided, Auto, Loiter, Stabilize)
- [x] Fly-to-location (click map → send coordinates → Guided mode)
- [x] Takeoff / Land / RTL commands
- [x] Built-in Swarm Simulator (3 drone bay quanh HCMUTE)
- [x] Telemetry display (battery, GPS, altitude, speed) — FloatingHUD

### Phase 3: Swarm AI + Mission Planning ⚠️ 60% (giao diện hoàn thiện, thuật toán cần cải thiện)
- [x] Group Commands (Arm All, Takeoff All, Land All, RTL All)
- [x] Virtual Structure formations — giao diện chọn đội hình (V-Shape, Line, Column, Circle)
- [ ] Virtual Structure formations — thuật toán chuẩn (Leader-Follower với điều khiển vị trí chính xác)
- [x] Boids Organic Flocking — prototype cơ bản (Separation + Cohesion + Alignment)
- [ ] Boids Organic Flocking — thuật toán Reynolds chuẩn (tuning tham số, tránh dao động)
- [x] Collision Avoidance — pixel-based detection (<40px)
- [ ] Collision Avoidance — thuật toán tránh va chạm thực tế (velocity obstacle, potential field)
- [x] Multi-Waypoint Mission Planning (add → start → auto-advance)
- [x] Telemetry Control Panel (LVL 0-3, UDP pause/resume)
- [x] Debug Console real-time (`logStore.ts` + `DebugConsole.tsx`)
- [x] Headless test Boids: 5.000 ticks, 5 drones → 0 crashes (khoảng cách tối thiểu 7.87m)
- [x] Playwright E2E test: 300s stress test → Pass

### Phase 4: Advanced Features 🔜
- [ ] GamePad support (HTML5 Gamepad API)
- [ ] Video streaming (WebRTC)
- [ ] UDP telemetry forwarding → Mission Planner/QGC
- [ ] Geo-fence display
- [ ] Kết nối Raspberry Pi thật + ArduPilot

### Phase 5: Polish 🔜
- [ ] Multi-language (i18n)
- [ ] Mobile responsive
- [ ] Error handling, reconnection logic
- [ ] Configuration UI

---

## Legal Note

Tất cả DroneEngage repositories trên GitHub đều **không có LICENSE file**
→ mặc định All Rights Reserved (tác giả nắm bản quyền).

**Đối với đồ án tốt nghiệp này:**
- Được phép đọc và tham khảo source code để hiểu protocol, học cách triển khai
- Không copy-paste nguyên bản — viết lại theo cách của mình
- Dự án hiện tại là private, không public, không thương mại → không vấn đề gì
- Nếu sau này muốn public/release: viết lại từ đầu hoặc xin phép tác giả

---

## Cấu trúc thư mục project

```text
clone_drone_engage/
├── README.md              ← tài liệu tổng hợp (747 dòng)
├── PROGRESS.md            ← tiến trình chi tiết từng ngày
├── docs/
│   └── architecture/
│       ├── overview.md    # Tech stack, folder structure, data flow
│       └── data-flow.md   # Auth flow + WebSocket sequence diagrams
├── src/
│   └── client/            # Web client (React Vite TypeScript SPA)
│       ├── src/
│       │   ├── core/ProtocolManager.ts  # WebSocket + Simulator (558 dòng)
│       │   ├── features/
│       │   │   ├── auth/        # AuthPage, authService (endpoint /w/wl/)
│       │   │   ├── dashboard/   # DroneMap, ActionPanel, TopToolbar, FloatingHUD, DebugConsole
│       │   │   └── telemetry/   # TelemetryControlPanel
│       │   ├── store/           # authStore, droneStore, telemetryStore, logStore
│       │   ├── types/           # andruav.ts, auth.ts
│       │   └── utils/           # logger, EventEmitter
│       ├── tests/e2e/swarm.spec.ts  # Playwright E2E test
│       └── package.json
├── scripts/               # test_auth.mjs
├── research/              # 5 files: protocol, auth, comm, rpi, udp-telemetry
├── logs/                  # codex (3), antigravity (3), progress.md
└── config/                # .env (gitignored)
```

---

*Last updated: 2026-06-17 — Phase 0-2 hoàn thành, Phase 3 đang cải thiện thuật toán, Phase 4-5 tiếp theo*
*Sources: https://github.com/DroneEngage, https://cloud.ardupilot.org/de-index.html*
*Team: Hermes (DeepSeek V4 Pro) · Antigravity (Opus 4.6) · Codex (GPT 5.5)*
