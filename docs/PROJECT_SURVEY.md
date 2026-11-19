# BÁO CÁO KHẢO SÁT TOÀN DIỆN DỰ ÁN "CLONE_DRONE_ENGAGE"
## Đánh Giá Hiện Trạng Kỹ Thuật, Thuật Toán Bầy Đàn & Tiềm Năng Nghiên Cứu Khoa Học

> **Người thực hiện:** Research Assistant (Antigravity System)  
> **Thời điểm khảo sát:** Tháng 11/2026 (Local timestamp: 2026-11-07)  
> **Chế độ khảo sát:** READ-ONLY toàn bộ cây thư mục và mã nguồn  
> **Mục tiêu khảo sát:** Phục vụ tái khởi động dự án để làm Nghiên cứu Khoa học (NCKH) và công bố bài báo khoa học (Scientific Paper).

---

## MỤC LỤC

1. [Cấu Trúc Cây Thư Mục & Phân Tích Dung Lượng](#1-cấu-trúc-cây-thư-mục--phân-tích-dung-lượng)
2. [Khảo Sát Tài Liệu, Cấu Hình, Thư Viện & Lịch Sử Git](#2-khảo-sát-tài-liệu-cấu-hình-thư-viện--lịch-sử-git)
3. [Kiến Trúc Hệ Thống & Cơ Chế Giao Tiếp](#3-kiến-trúc-hệ-thống--cơ-chế-giao-tiếp)
4. [Phân Tích Thuật Toán Cốt Lõi (Core Algorithms & Code Audit)](#4-phân-tích-thuật-toán-cốt-lõi-core-algorithms--code-audit)
5. [Mô Phỏng, Thí Nghiệm & Khả Năng Tái Lập (Reproducibility)](#5-mô-phỏng-thí-nghiệm--khả-năng-tái-lập-reproducibility)
6. [Đánh Giá Chất Lượng Mã Nguồn & Tình Trạng Kỹ Thuật](#6-đánh-giá-chất-lượng-mã-nguồn--tình-trạng-kỹ-thuật)
7. [Đánh Giá Hướng Nghiên Cứu Khoa Học & Đề Xuất Đóng Góp (Novelty)](#7-đánh-giá-hướng-nghiên-cứu-khoa-học--đề-xuất-đóng-góp-novelty)
8. [Danh Sách Câu Hỏi Trọng Yếu Cần Chủ Dự Án Phản Hồi](#8-danh-sách-câu-hỏi-trọng-yếu-cần-chủ-dự-án-phản-hồi)

---

## 1. CẤU TRÚC CÂY THƯ MỤC & PHÂN TÍCH DUNG LƯỢNG

### 1.1 Cây thư mục (Độ sâu 3 levels, đã lọc bỏ cache/binary)

```text
clone_drone_engage/
├── config/
│   └── .env                               # Cấu hình biến môi trường cục bộ
├── docs/
│   └── architecture/
│       ├── data-flow.md                   # Luồng xác thực & WebSocket sequence
│       └── overview.md                    # Kiến trúc Web Client & Zustand stores
├── logs/
│   ├── antigravity/
│   │   ├── LOGGING_GUIDELINES.md          # Hướng dẫn ghi log cho AI agent
│   │   ├── session_2026-06-07.md          # Log thiết kế & Reverse Engineering Auth
│   │   └── session_2026-06-08.md          # Log Swarm AI, Boids, Collision Avoidance
│   ├── codex/
│   │   ├── codex_2026-06-07_log.md        # Log coding giai đoạn skeleton
│   │   ├── codex_2026-06-07_summary.md    # Tóm tắt kết quả code
│   │   ├── session_2026-06-07_goals.md    # Mục tiêu ngày 07/06
│   │   ├── vite-dev.err.log               # Log lỗi runtime Vite dev server
│   │   └── vite-dev.out.log               # Log stdout Vite dev server
│   └── progress.md                        # Nhật ký tiến trình dự án ban đầu
├── research/
│   ├── auth-server.md                     # Tài liệu API Auth Server (port 19408)
│   ├── comm-server.md                     # Tài liệu Comm Server WebSocket (port 9966)
│   ├── gcs-career-landscape.md            # Báo cáo thị trường nghề nghiệp GCS UAV
│   ├── rpi-admin-tools.md                 # Tài liệu WiFi Manager & Cockpit trên RPi
│   ├── udp-telemetry-commands.md          # Danh mục lệnh MAVLink rate control (msg 1005)
│   └── udp-telemetry.md                   # Tài liệu Web UDP Telemetry & UDPCI
├── scratch/
│   └── test_ws.js                         # Script Node.js test raw WebSocket kết nối Comm Server
├── scripts/
│   └── test_auth.mjs                      # Script kiểm tra HTTP POST Auth Server
├── src/
│   ├── client/                            # Web Client SPA (React 18 + TS + Tailwind)
│   │   ├── public/                        # Static assets (favicon, icons)
│   │   ├── src/
│   │   │   ├── assets/                    # Hình ảnh UI (hero banner)
│   │   │   ├── components/                # UI components dùng chung (stub)
│   │   │   ├── core/                      # ProtocolManager, Gamepad stub, WebRTC stub
│   │   │   ├── features/                  # Auth, Dashboard, Map, Swarm, Telemetry
│   │   │   ├── hooks/                     # Custom hooks (stub)
│   │   │   ├── store/                     # Zustand stores: auth, drone, telemetry, log
│   │   │   ├── types/                     # TypeScript types (Andruav envelope, Auth)
│   │   │   ├── utils/                     # EventEmitter, client logger
│   │   │   ├── App.tsx                    # React Root routing (/login, /dashboard)
│   │   │   ├── main.tsx                   # Entry point DOM mount
│   │   │   └── index.css                  # Tailwind styles
│   │   ├── tests/
│   │   │   └── e2e/
│   │   │       └── swarm.spec.ts          # Playwright test 300s stress test
│   │   ├── package.json                   # Khai báo dependency NPM
│   │   ├── playwright.config.ts           # Cấu hình kiểm thử E2E
│   │   ├── simulation_log.txt             # Kết quả chạy test Boids độc lập (5.000 ticks)
│   │   ├── test_swarm.ts                  # Thuật toán Boids chạy headless Node.js
│   │   └── vite.config.ts                 # Cấu hình Vite build tool
│   └── server/                            # Thư mục rỗng (Dự định chứa C++ Backend)
├── airgap_setup_guide.md                  # Hướng dẫn setup DroneEngage AirGap LAN trên RPi4
├── cto_feedback_and_goals.md              # Nhận xét từ CTO: C++ mandate, delay 4G, IP-GCS
├── hardware_config.md                     # Danh mục linh kiện 2 drone vật lý (5" và 7")
├── new_cv.md                              # CV ứng viên (Phạm Hoàng Trung Kiên - HCMUTE)
├── old_cv_text.txt                        # Bản trích xuất text CV cũ
├── PLAN_PHASE4.md                         # Kế hoạch Phase 4: Fix delay 4G & dựng C++ GCS
├── PROGRESS.md                            # Tổng kết tiến độ toàn dự án qua các Phase
├── README.md                              # Bách khoa toàn thư tài liệu DroneEngage & Swarm
└── SESSION_HANDOFF_2026-06-22.md          # Biên bản bàn giao ca làm việc kỹ thuật
```

### 1.2 Phân tích các file và thư mục dung lượng lớn / Đáng lưu ý

| Tên File / Thư mục | Kích thước xấp xỉ | Bản chất kỹ thuật & Ý nghĩa trong dự án |
|---|---|---|
| `temp_pypdf/` | ~1.4 MB (hơn 100 files) | **ĐÃ XÁC MINH:** Mã nguồn giải nén của thư viện Python `pypdf` (kèm file nhị phân compiled `.pyc`). Thư mục này do chủ dự án copy vào để phục vụ script parse CV (`new_cv.md`, `old_cv_text.txt`). Hoàn toàn **không liên quan** đến thuật toán điều khiển drone. |
| `src/client/playwright-report/` | ~530 KB | Báo cáo HTML kết quả kiểm thử E2E tự động từ Playwright. Chứa dữ liệu trace của bài test giả lập bầy drone. |
| `src/client/dist/assets/index-*.js` | ~365 KB | Bản build JavaScript đã bundle và minify của Web Client GCS. |
| `src/client/package-lock.json` | ~114 KB | Khóa phiên bản dependency cụ thể cho môi trường Node.js. |
| `logs/codex/vite-dev.err.log` | ~36 KB | File text log lưu các cảnh báo và lỗi compile của server Vite trong quá trình dev. |

---

## 2. KHẢO SÁT TÀI LIỆU, CẤU HÌNH, THƯ VIỆN & LỊCH SỬ GIT

### 2.1 Đánh giá tài liệu nội bộ (Documentation Audit)
Dự án có hệ thống tài liệu rất chi tiết, phản ánh rõ quá trình phát triển từ một dự án sinh viên đến định hướng chuyên nghiệp:
- **`README.md` (776 dòng):** Chứa toàn bộ thông tin tổng hợp về hệ sinh thái **DroneEngage** (tác giả Mohammad Hefny - ArduPilot Cloud), phân tích kiến trúc Dual-Mode (Cloud vs AirGap), bảng mã hóa Message Types của giao thức Andruav (1002 GPS, 1004 Identity, 1005 Remote Command, 1010 Flight Control, 1035 Mission, 1071 UDP Proxy...).
- **`cto_feedback_and_goals.md` & `PLAN_PHASE4.md`:** Biên bản ghi nhận ý kiến từ CTO phỏng vấn ngày 22/06/2026. Phản ánh nút thắt kỹ thuật lớn nhất: **Điều khiển bầy đàn qua mạng 4G thương mại bị delay ~1.000 ms (1 giây)** do kiến trúc 6-hop (FC → Pi → WiFi AP → Mobile Hotspot 4G → Cloud Nước Ngoài → Mobile 4G → Laptop). Đồng thời đặt ra quy tắc bắt buộc chuyển đổi dần core backend sang **C/C++ (MAVSDK)**.
- **`hardware_config.md`:** Khảo sát cho thấy cấu hình phần cứng hiện có:
  + **Drone 1 (5 inch):** Khung Mark 4, Flight Controller MicroAir H743v2 (STM32H743), ESC Bluejay 45A, Motor MT2204 2300kv, GPS BZ251, RX ELRS RP1, Pin 3S 2200mAh, Companion Computer **Raspberry Pi 4**.
  + **Drone 2 (7 inch):** Khung F330, ESC Bluejay 60A, Motor Sunsky X2216 1100kv, GPS BZ251, RX ELRS Nano, Companion Computer **Raspberry Pi 3+** (Hiện tại đang thiếu mạch Flight Controller).
  + **Drone 3:** Chưa lắp ráp phần cứng.
  + **Transmitter:** RadioMaster Pocket (ELRS).

### 2.2 Cấu hình môi trường & Thư viện phụ thuộc
- **Frontend Stack (`src/client/package.json`):**
  + Runtime: Node.js (đã kiểm tra tương thích Node v24.19.0).
  + Core Framework: React `18.3.1` (cố định React 18, không lên React 19 để tương thích Leaflet), React DOM `18.3.1`.
  + Trình biên dịch & Bundler: Vite `8.0.12`, TypeScript `~6.0.2`.
  + Giao diện & Bản đồ: Tailwind CSS `4.3.0`, Leaflet `1.9.4`, `react-leaflet 4.2.1`.
  + Quản lý trạng thái (State Management): Zustand `5.0.14`.
  + Kiểm thử: `@playwright/test 1.60.0`.
- **Cấu hình mạng (`config/.env` & `src/client/.env`):**
  + Biến `VITE_AUTH_SERVER_URL`: Trỏ tới server xác thực của DroneEngage (mặc định `https://cloud.ardupilot.org:19408` hoặc server AirGap `http://192.168.1.161:19408`).
- **Launch Files & Robotic Framework:**
  + **ĐÃ XÁC MINH TỪ CODE:** Hiện tại trong repo **hoàn toàn KHÔNG có launch file của ROS/ROS2** (`.launch.py`), không có file mô hình SDF/URDF của Gazebo, không có workspace PX4/ArduPilot SITL tích hợp trực tiếp trong git. Mọi logic mô phỏng hiện tại được thực thi thuần túy trong trình duyệt (In-browser JavaScript Engine).

### 2.3 Lịch sử Git & Trạng thái Working Tree
- **Nhánh hiện tại:** `main` (Up to date với `origin/main` tại repo `Ferb27/clone_drone_engage`).
- **Lịch sử Commit (Toàn bộ 4 commit trên repo):**
  + `b2cd5f4` (HEAD) - *docs: tune down Phase 3 to 60% - UI done, algorithms need real implementation*
  + `33f44b4` - *docs: update README - reflect actual progress (Phase 0-3 done, Phase 4-5 next)*
  + `804449b` - *docs: add PROGRESS.md - full project timeline and status*
  + `8536465` - *feat: initial commit*
- **Trạng thái Uncommitted:** Working directory đang có **hơn 60 files bị thay đổi (modified) và 9 files untracked** (chứa toàn bộ code triển khai UI, Simulator, Swarm Boids, E2E test). Toàn bộ phần code này nằm ở working tree chưa commit.

---

## 3. KIẾN TRÚC HỆ THỐNG & CƠ CHẾ GIAO TIẾP

### 3.1 Ngôn ngữ & Nền tảng thực tế

| Phân hệ | Hiện trạng trong Codebase | Mục tiêu thiết kế (Tài liệu) |
|---|---|---|
| **Ground Control Station (GCS)** | TypeScript, React 18, Zustand, Leaflet | Giữ nguyên React UI, bổ sung C++ GCS Engine |
| **Swarm Controller Logic** | TypeScript (chạy giả lập 1000ms tick trên client) | C++ chạy phân tán trên Raspberry Pi hoặc GCS Backend |
| **Companion Computer** | Chưa có code trong repo (dùng Image DroneEngage RPi đóng gói sẵn) | Raspberry Pi OS, C++ modules (`de_comm`, `de_mavlink`) |
| **Flight Controller** | ArduCopter firmware (trên MicroAir H743v2) | ArduCopter MAVLink Protocol |
| **Giao tiếp liên drone** | WebSocket relay trung tâm qua Cloud Server | WebSocket Cloud Relay hoặc UDP Ad-hoc P2P |

### 3.2 Luồng Dữ Liệu (Data Flow) Đã Xác Minh Từ Mã Nguồn

```text
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      BƯỚC 1: XÁC THỰC (HTTP REST)                     │
 │  GCS Web Client  ──POST /w/wl/──▶  Auth Server (Port 19408)           │
 │  (email, pwd, ver="5.0.0")                                             │
 │                  ◀──JSON {cs, sid}─ (Host Comm Server, TempKey f)       │
 └──────────────────────────────────┬─────────────────────────────────────┘
                                    │
                                    ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │                  BƯỚC 2: KẾT NỐI WEBSOCKET PERSISTENT                  │
 │  GCS Web Client  ──WSS Connect──▶  Communication Server (Port 9966)    │
 │  URL: wss://host:port/?f=tempKey&s=partyID&at=g                        │
 └──────────────────────────────────┬─────────────────────────────────────┘
                                    │
                                    ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │                  BƯỚC 3: ĐIỀU PHỐI VÀ TELEMETRY REAL-TIME              │
 │                                                                        │
 │  [Drone 1 / Companion] ──UDP/MAVLink──▶ [Comm Server]                  │
 │                                             │                          │
 │                                      WebSocket Relay                   │
 │                                             │                          │
 │                                             ▼                          │
 │                                    ProtocolManager.ts                  │
 │                                    (Singleton Client)                  │
 │                                             │                          │
 │                     ┌───────────────────────┴───────────────────────┐  │
 │                     ▼                                               ▼  │
 │             useDroneStore (Zustand)                       useLogStore  │
 │           (lat, lng, heading, alt...)                    (Debug Term)  │
 │                     │                                                  │
 │      ┌──────────────┴──────────────┐                                   │
 │      ▼                             ▼                                   │
 │  DroneMap.tsx                FloatingHUD.tsx                           │
 │ (Leaflet Marker,             (Compass Rose,                            │
 │  Collision Alert)             Alt, Spd, State)                         │
 └────────────────────────────────────────────────────────────────────────┘
```

- **Cấu trúc gói tin Envelope (Andruav Protocol):**
  Xác minh tại [`src/client/src/core/ProtocolManager.ts#L109-L115`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L109-L115):
  ```typescript
  const envelope = {
    _ty: "c",             // 'c' = Communication message, 's' = System message
    i: targetPartyID,     // ID drone mục tiêu (hoặc '_AGN_' để broadcast tới toàn bầy)
    messageType: 1005,    // 1005 = Remote Command
    sender: sessionId,    // ID của GCS Web
    msg: msgPayload       // Lệnh cụ thể: Arm ({C:100}), Takeoff ({C:103, Act:1})...
  };
  ```

- **Giao thức giữa các Drone (Inter-drone Communication):**
  + **ĐÃ XÁC MINH TỪ CODE:** Hệ thống hiện tại **KHÔNG dùng mạng Mesh Ad-hoc trực tiếp giữa các drone (P2P Wi-Fi hay RF)**. Mọi trao đổi thông tin giữa các drone hoặc từ GCS đến bầy đàn đều là **mô hình hình sao (Star Topology)** thông qua WebSocket Relay của Communication Server.
  + **Hạn chế đối với NCKH:** Cơ chế này chịu ảnh hưởng trực tiếp của băng thông mạng di động và độ trễ Round-Trip Time (RTT) của server trung gian. Nếu mất kết nối Internet, bầy đàn mất liên lạc hoàn toàn trừ khi chuyển sang dùng AirGap Server trên mạng WiFi nội bộ của Pi.

---

## 4. PHÂN TÍCH THUẬT TOÁN CỐT LÕI (CORE ALGORITHMS & CODE AUDIT)

Khảo sát chi tiết từng thuật toán trong mã nguồn thực tế tại [`ProtocolManager.ts`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts) và [`test_swarm.ts`](file:///d:/Project/clone_drone_engage/src/client/test_swarm.ts):

### 4.1 Thuật toán Điều khiển Đội hình Cấu trúc Ảo (Virtual Structure / Leader-Follower)
- **Vị trí trong mã nguồn:** [`src/client/src/core/ProtocolManager.ts#L389-L432`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L389-L432).
- **Ý tưởng thuật toán:**
  Một drone được chỉ định làm Leader (thông qua UI selector). Các drone còn lại (Followers) sẽ tính toán tọa độ mục tiêu mong muốn $(\text{destLat}, \text{destLng})$ dựa trên:
  1. Tọa độ thực của Leader $(\text{lat}_L, \text{lng}_L)$.
  2. Góc mũi (Heading) của Leader $\psi_L$.
  3. Khoảng cách lệch phía sau ($\text{back}$) và khoảng cách lệch sang phải ($\text{right}$) theo hệ trục tọa độ của Leader.
- **Bốn hình thái đội hình (Formations):**
  + **V-Shape (Hình chữ V):** [`ProtocolManager.ts#L400-L403`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L400-L403):
    $$\text{back} = \lceil \text{index} / 2 \rceil \times 20\,\text{m}, \quad \text{right} = (-1)^{\text{index}} \times \lceil \text{index} / 2 \rceil \times 20\,\text{m}$$
  + **Line (Hàng ngang):** [`ProtocolManager.ts#L404-L407`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L404-L407): $\text{back} = 0$, dãn cách sang 2 bên theo bước $20\,\text{m}$.
  + **Column (Hàng dọc):** [`ProtocolManager.ts#L408-L411`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L408-L411): $\text{right} = 0, \text{back} = \text{index} \times 20\,\text{m}$.
  + **Circle (Vòng tròn quanh Leader):** [`ProtocolManager.ts#L412-L417`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L412-L417):
    $$\theta = \frac{360^\circ}{N_{\text{followers}}} \times (\text{index} - 1), \quad \text{back} = 30 \cos\theta, \quad \text{right} = 30 \sin\theta$$
- **Chuyển đổi hệ tọa độ phẳng sang GPS (WGS84 quy đổi cục bộ):**
  Trích dẫn [`ProtocolManager.ts#L420-L432`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L420-L432):
  ```typescript
  const leadHeadingRad = leaderSimData.heading * (Math.PI / 180)
  const backRad = leadHeadingRad + Math.PI
  const rightRad = leadHeadingRad + (Math.PI / 2)
  const mToDeg = 0.000009 // Hằng số xấp xỉ: 1 mét ~ 0.000009 độ vĩ tuyến

  const backOffsetLat = Math.cos(backRad) * back * mToDeg
  const backOffsetLng = Math.sin(backRad) * back * mToDeg
  const rightOffsetLat = Math.cos(rightRad) * right * mToDeg
  const rightOffsetLng = Math.sin(rightRad) * right * mToDeg

  destLat = leaderSimData.lat + backOffsetLat + rightOffsetLat
  destLng = leaderSimData.lng + backOffsetLng + rightOffsetLng
  ```
- **Nhận xét chuyên môn:** Thuật toán tính toán vị trí hình học thuần túy (Kinematic Position Offset). Chưa tích hợp bộ điều khiển bù sai số động học (như PID, LQR, hay Feedback Linearization) để bù trễ khi Leader đổi hướng đột ngột, dẫn đến hiện tượng Followers bị "văng" khi quay vòng gắt.

---

### 4.2 Thuật toán Bầy Đàn Tự Tổ Chức (Boids Organic Flocking - Reynolds Model)
- **Vị trí trong mã nguồn:** [`src/client/src/core/ProtocolManager.ts#L318-L376`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L318-L376) và bản kiểm thử độc lập tại [`src/client/test_swarm.ts#L44-L111`](file:///d:/Project/clone_drone_engage/src/client/test_swarm.ts#L44-L111).
- **Ý tưởng:** Mô phỏng tập tính bầy chim (Craig Reynolds, 1987) kết hợp lực neo trọng tâm để bầy không phân tán ra vô tận.
- **Công thức và Code thực thi:**
  1. **Lực tách rời (Separation):**
     Đẩy các cá thể ra xa khi khoảng cách $< 0.001^\circ$ (~$110\,\text{m}$ trong simulator web hoặc $16\,\text{m}$ trong `test_swarm.ts`):
     Trích dẫn [`ProtocolManager.ts#L338-L343`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L338-L343):
     $$\vec{F}_{\text{sep}} = \sum_{j \neq i} \frac{\vec{x}_i - \vec{x}_j}{\|\vec{x}_i - \vec{x}_j\|} \times \left( \frac{R_{\text{safe}} - \|\vec{x}_i - \vec{x}_j\|}{R_{\text{safe}}} \right)$$
  2. **Lực liên kết (Cohesion):**
     Hút cá thể về phía trọng tâm tọa độ của các cá thể lân cận trong bán kính lân cận ($R_{\text{neighbor}} \approx 330\,\text{m}$):
     $$\vec{F}_{\text{coh}} = \left( \frac{1}{N} \sum_{j=1}^N \vec{x}_j \right) - \vec{x}_i$$
  3. **Lực đồng hướng (Alignment):**
     Điều chỉnh hướng bay theo trung bình cộng hướng bay của các cá thể xung quanh:
     $$\bar{\psi} = \frac{1}{N} \sum_{j=1}^N \psi_j, \quad \vec{F}_{\text{align}} = [\cos\bar{\psi}, \sin\bar{\psi}]^T \times W_{\text{align}}$$
  4. **Lực neo gốc (Centering / Homing Force):**
     Kéo nhẹ các cá thể về điểm xuất phát gốc HCMUTE để giữ bầy trong khung nhìn bản đồ:
     $$\vec{F}_{\text{home}} = (\vec{x}_{\text{origin}} - \vec{x}_i) \times 0.02$$
  5. **Tổng hợp vector di chuyển:**
     Trích dẫn [`ProtocolManager.ts#L368-L369`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L368-L369):
     $$\vec{x}_{i,\text{dest}} = \vec{x}_i + 0.1 \vec{F}_{\text{coh}} + 0.00015 \vec{F}_{\text{sep}} + \vec{F}_{\text{align}} + \vec{F}_{\text{home}}$$

---

### 4.3 Thuật toán Tránh Va Chạm Hai Tầng (Two-Tier Collision Avoidance)
Hệ thống kết hợp cả thuật toán tránh va chạm trên mô hình toán học và cơ chế giám sát đồ họa:

- **Tầng 1: Lực đẩy trượt tiếp tuyến (Algorithmic Repulsion & Tangential Slide):**
  Vị trí: [`src/client/src/core/ProtocolManager.ts#L470-L510`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L470-L510).
  Khi hai drone tiếp cận trong bán kính cảnh giới $R_{\text{safe}} = 0.0012^\circ$ (~$130\,\text{m}$), thuật toán sẽ:
  + Giảm vận tốc tịnh tiến xuống 50%: `forwardMoveDist *= 0.5`.
  + Tạo lực đẩy trực tiếp (Radial Repulsion): $\vec{F}_{\text{push}} = \frac{\Delta \vec{x}}{\|\Delta \vec{x}\|} \times F_{\text{repel}}$.
  + Tạo lực trượt tiếp tuyến góc $90^\circ$ (Tangential Slide Vector) để lách qua nhau, tránh hiện tượng đối đầu trực diện gây nghẽn (Deadlock):
    ```typescript
    // Vector trượt ngang (Tangential) để lượn qua nhau
    const slideLat = -(dLng / otherDist) * (repelStrength * 0.8)
    const slideLng =  (dLat / otherDist) * (repelStrength * 0.8)
    ```
- **Tầng 2: Giám sát va chạm pixel trên GCS (UI Pixel-Based Collision Detection):**
  Vị trí: [`src/client/src/features/dashboard/DroneMap.tsx#L183-L228`](file:///d:/Project/clone_drone_engage/src/client/src/features/dashboard/DroneMap.tsx#L183-L228).
  Sử dụng hàm chuyển đổi không gian của Leaflet `map.latLngToContainerPoint([lat, lng])` để chuyển tọa độ địa lý thành pixel trên màn hình. Kích thước icon là $40 \times 40\,\text{px}$. Nếu khoảng cách giữa 2 tâm icon $< 40\,\text{px}$, hệ thống hiển thị cảnh báo đỏ nhấp nháy trên HUD và kích hoạt sự kiện vi phạm an toàn.

---

### 4.4 Lập Kế Hoạch Đường Bay Đa Điểm (Multi-Waypoint Mission Planning)
- **Vị trí:** [`src/client/src/store/droneStore.ts#L89-L123`](file:///d:/Project/clone_drone_engage/src/client/src/store/droneStore.ts#L89-L123) và [`ProtocolManager.ts#L177-L192, L520-L526`](file:///d:/Project/clone_drone_engage/src/client/src/core/ProtocolManager.ts#L177-L192).
- **Cơ chế:** Cho phép người dùng click trên bản đồ Leaflet để nạp mảng tọa độ `missionWaypoints[]`. Khi drone bay đến gần waypoint hiện tại trong phạm vi sai số quy định (`dist < 0.00002^\circ` ~ $2.2\,\text{m}$), máy trạng thái tự động gọi `popWaypoint(partyID)` để kích hoạt waypoint kế tiếp cho đến khi hoàn tất lộ trình.

---

### 4.5 Phân Định Chức Năng: Task Allocation, Engagement & Tên Dự Án
- **ĐÃ XÁC MINH TỪ MÃ NGUỒN:**
  + Mặc dù dự án mang tên **"clone_drone_engage"**, nhưng trong toàn bộ mã nguồn **CHƯA CÓ** thuật toán bám đuổi (Pursuit), đánh chặn (Interception), giao tranh đối kháng (Engagement), hoặc phân bổ nhiệm vụ tối ưu động (Dynamic Task Allocation như CBBA, Hungarian Algorithm, Auction Algorithm).
  + **Nguồn gốc tên gọi:** Tên dự án xuất phát từ việc **"clone lại nền tảng mã nguồn mở DroneEngage"** của tác giả Mohammad Hefny (hệ thống trạm điều khiển ArduPilot qua Internet), chứ không phải hệ thống chiến đấu quân sự tự hành. Đây là điểm mấu chốt cần phân biệt rõ ràng khi viết thuyết minh đề tài nghiên cứu khoa học.

---

## 5. MÔ PHỎNG, THÍ NGHIỆM & KHẢ NĂNG TÁI LẬP (REPRODUCIBILITY)

### 5.1 Các Kịch Bản Mô Phỏng Hiện Có Trong Dự Án
1. **Kịch bản Web Simulator (Tương tác trực quan):**
   - Khởi chạy dev server và click nút **"START SWARM SIM"** trên thanh công cụ.
   - 3 drone ảo (`sim_drone_1`, `sim_drone_2`, `sim_drone_3`) xuất hiện tại khuôn viên HCMUTE (10.850145, 106.771661).
   - Thao tác: Chọn Leader, đổi đội hình (V-Shape, Line, Column, Circle), bật Flock Mode, click bản đồ để chỉ định vị trí bay (Fly to here / Add to Mission).
2. **Kịch bản Thử Nghiệm Headless (Kiểm thử tải thuật toán):**
   - File thực thi: [`src/client/test_swarm.ts`](file:///d:/Project/clone_drone_engage/src/client/test_swarm.ts).
   - Mô phỏng 5 drone xuất phát cực gần nhau (chênh lệch $0.00005^\circ \approx 5.5\,\text{m}$) và bay tự do theo mô hình Boids trong suốt 5.000 chu kỳ (ticks) với tần số 10Hz (tương đương 500 giây bay liên tục).
3. **Kịch bản E2E Stress Test (Playwright):**
   - File thực thi: [`src/client/tests/e2e/swarm.spec.ts`](file:///d:/Project/clone_drone_engage/src/client/tests/e2e/swarm.spec.ts).
   - Tự động hóa: Bypass Login → Start Sim → Điều khiển 3 drone bay giao thoa vào cùng một tọa độ hẹp để gây va chạm → Kích hoạt Flock Mode → Đo lường lỗi màn hình và lỗi va chạm trong 300 giây.

### 5.2 Các Chỉ Số (Metrics) Đang Được Ghi Nhận
- **Khoảng cách tối thiểu (Minimum Distance Observed):** Khoảng cách gần nhất giữa hai cá thể bất kỳ trong suốt quá trình bay (đơn vị: mét).
- **Số lần đâm va chạm vật lý (Total Crashes):** Số lần khoảng cách giữa 2 drone rơi vào mức nguy hiểm $< 5.0\,\text{m}$.
- **Cảnh báo đè icon đồ họa (UI Pixel Collision):** Tần số xuất hiện cảnh báo khoảng cách pixel $< 40\,\text{px}$.
- **Độ trễ khứ hồi đường truyền (Network Latency - RTT):** Đo đạc định tính trong tài liệu kiến trúc (Cloud: ~1.000 ms, AirGap dự kiến: ~20-50 ms).

### 5.3 Kết Quả Chạy Thử Nghiệm Thực Tế (Verification Run)
Chúng tôi đã chạy thử nghiệm trực tiếp các lệnh test trong môi trường dòng lệnh (đảm bảo tính toàn vẹn, không chỉnh sửa file code):

1. **Chạy kiểm thử Headless Swarm:**
   ```powershell
   npx tsx src/client/test_swarm.ts
   ```
   - **Kết quả xuất ra màn hình:**
     ```text
     Simulation finished. Min Dist: 7.87m, Crashes: 0
     ```
   - **Đánh giá:** TÁI LẬP THÀNH CÔNG 100%. Khớp hoàn toàn với log lưu trữ tại `src/client/simulation_log.txt`. Thuật toán tách rời (Separation) hoạt động ổn định trong không gian 2D lý tưởng.

2. **Kiểm tra biên dịch TypeCheck toàn dự án:**
   ```powershell
   cd src/client; npx tsc -b
   ```
   - **Kết quả:** Phát hiện **01 lỗi biên dịch TypeScript**!
     ```text
     src/features/auth/AuthPage.tsx(165,58): error TS2322: Type 'number' is not assignable to type 'string'.
     ```
   - **Nguyên nhân chính xác:** Tại dòng 165 của file [`AuthPage.tsx`](file:///d:/Project/clone_drone_engage/src/client/src/features/auth/AuthPage.tsx#L165), nút bypass login gọi hàm `setAuthInfo({ f: 'mock', g: 'localhost', h: 8080 }, 'dev_session')` truyền `h: 8080` (number), trong khi interface `CommServerInfo` tại [`types/auth.ts:15`](file:///d:/Project/clone_drone_engage/src/client/src/types/auth.ts#L15) quy định trường `h` (port) phải là kiểu `string`.

---

## 6. ĐÁNH GIÁ CHẤT LƯỢNG MÃ NGUỒN & TÌNH TRẠNG KỸ THUẬT

### 6.1 Bảng Phân Loại Tình Trạng Tính Năng

| Phân hệ / Tính năng | Hiện trạng | Mức độ hoàn thiện | Chi tiết kỹ thuật |
|---|---|:---:|---|
| **Giao diện GCS (Dashboard UI)** | Hoàn thiện tốt | 95% | Giao diện hiện đại, Glassmorphism, Leaflet map tải mượt, HUD hiển thị la bàn xoay trực quan. |
| **Xác thực Auth Server thật** | Hoàn thiện | 90% | Reverse-engineer thành công endpoint `/w/wl/` của DroneEngage, nhận diện đúng session ID và temp key. |
| **Bộ thu phát WebSocket** | Tốt trên lý thuyết | 70% | Đã thiết kế envelope Andruav chuẩn, tuy nhiên chưa có cơ chế Auto-Reconnect và chưa test thực tế với server tải cao. |
| **Mô phỏng Đội hình (Virtual Structure)** | Mức cơ bản | 60% | Mới mô phỏng vị trí hình học 2D, chưa mô phỏng quán tính vật lý (6-DOF) và khí động học cánh quạt. |
| **Mô phỏng Flocking (Boids)** | Khá | 75% | Đã tuning tham số để tránh va chạm tốt trong 2D, có lực hồi tâm chống mất bầy. |
| **C++ GCS Engine / Backend** | Chưa triển khai | 0% | Thư mục `src/server/` hoàn toàn rỗng. Kế hoạch MAVSDK mới chỉ nằm trên tài liệu `PLAN_PHASE4.md`. |
| **Tích hợp Camera WebRTC** | Chưa triển khai | 5% | Mới có file stub [`webrtc/index.ts`](file:///d:/Project/clone_drone_engage/src/client/src/core/webrtc/index.ts) rỗng. |
| **Điều khiển Gamepad API** | Chưa triển khai | 5% | Mới có file stub [`gamepad/index.ts`](file:///d:/Project/clone_drone_engage/src/client/src/core/gamepad/index.ts) rỗng. |
| **Phần cứng UAV thực tế** | Dở dang | 40% | Đã có 1 drone 5" hoàn chỉnh (MicroAir H743 + RPi4); drone 2 thiếu FC; chưa từng bay bầy thực tế ở hiện trường. |

### 6.2 Nợ Kỹ Thuật (Technical Debt) & Vấn Đề Cần Xử Lý
1. **Lỗi TypeCheck TypeScript:** Lỗi tại `AuthPage.tsx:165` khiến lệnh `npm run build` (`tsc -b && vite build`) bị chặn. Cần sửa `h: '8080'` hoặc định nghĩa `h: string | number`.
2. **Thiếu Unit Test cho Frontend:** Toàn bộ thư mục `src/client` chỉ có 1 file E2E và 1 file script Node.js. Chưa có các unit test với Vitest/Jest kiểm thử riêng cho các hàm toán học chuyển đổi tọa độ (`mToDeg`, vector rotation, boids summation).
3. **Mô phỏng Kinematic thay vì Dynamic:** Vận tốc và hướng mũi trong Simulator web được cập nhật bằng cách cộng đại số tọa độ mỗi giây một lần (1Hz tick). Drone thực tế trong không gian 3D bị chi phối bởi phương trình vi phân phi tuyến (Nonlinear Quadrotor Dynamics), lực cản gió, hiệu ứng ground effect và độ bão hòa lực nâng của động cơ.
4. **Vấn đề trễ truyền thông 4G chưa có giải pháp trên code:** Mã nguồn chưa cài đặt bộ ước lượng trạng thái (State Estimator / Extended Kalman Filter) hoặc bộ bù trễ (Smith Predictor / Dead Reckoning) để xử lý tình trạng packet drop hay lag ~1 giây khi bay qua sóng 4G.

---

## 7. ĐÁNH GIÁ HƯỚNG NGHIÊN CỨU KHOA HỌC & ĐỀ XUẤT ĐÓNG GÓP (NOVELTY)

### 7.1 Khoảng Trống Tri Thức So Với Các Công Bố Khoa Học (Literature Gap)
Khi khảo sát các bài báo khoa học gần đây về Swarm Drone trên các tạp chí uy tín (IEEE Transactions on Robotics, IEEE T-IV, Elsevier Journal of Field Robotics, IEEE Access, ICUAS):
- **Khoảng trống 1 (Lý thuyết vs Thực tế mạng viễn thông):** Đại đa số các nghiên cứu học thuật về Swarm Formation/Consensus giả định hệ thống truyền thông lý tưởng (độ trễ bằng 0, không rớt gói, hoặc trễ cố định rất nhỏ qua mạng LAN/ROS2 DDS). Rất ít công trình giải quyết bài toán: **Điều khiển bầy đàn đồng thuận khi kết nối qua mạng di động tế bào (Cellular 4G/LTE/5G) với độ trễ biến thiên bất đối xứng (Time-varying asymmetric delay) và mất gói ngẫu nhiên (Packet Dropout).**
- **Khoảng trống 2 (GCS Platform cho BVLOS đa tác tử):** Các trạm mặt đất hiện nay (QGroundControl, Mission Planner) chủ yếu thiết kế cho mô hình điều khiển 1-1 (1 trạm - 1 drone). Các giải pháp thương mại cho Swarm (như DroneShowSoftware của UgCS) đều đóng mã nguồn và chi phí rất đắt. Việc xây dựng một nền tảng GCS nguồn mở qua IP có khả năng quản lý động trạng thái bầy đàn và tự thích nghi tốc độ truyền telemetry là một đóng góp kỹ thuật có giá trị thực tiễn cao.

### 7.2 Các Thí Nghiệm Cần Bổ Sung Để Đủ Dữ Liệu Đăng Bài Báo
Để có một bài báo khoa học đạt chuẩn bình duyệt (Peer-review), không thể chỉ dùng kết quả từ script TypeScript 2D hiện tại. Cần bổ sung các thí nghiệm sau:
1. **Thí nghiệm Benchmark Mạng Di Động (Network Characterization):**
   - Thu thập số liệu thực tế về RTT, Jitter, Packet Delivery Ratio (PDR) của đường truyền 4G/LTE tại Việt Nam (Viettel, Vinaphone) khi truyền gói MAVLink.
   - So sánh định lượng giữa chế độ Cloud Relay và chế độ AirGap LAN cục bộ trên RPi4.
2. **Thí nghiệm Mô Phỏng HIL / SITL Độ Trung Thực Cao:**
   - Dựng cụm mô phỏng ArduPilot SITL đa drone (Multi-vehicle SITL) kết hợp công cụ Linux `tc` (Traffic Control / NetEm) để bơm trễ nhân tạo (ví dụ: inject delay 200ms, 500ms, 1.000ms; loss 5%, 10%).
   - Đo đạc sai số bám đội hình (Tracking Error $\text{RMSE}$) và khoảng cách an toàn nhỏ nhất giữa các drone dưới tác động của trễ mạng.
3. **Thí nghiệm Thực Địa (Outdoor Field Experiment):**
   - Tối thiểu 2 drone vật lý bay đồng thời ngoài trời dưới chế độ tự hành đồng bộ (Synchronized Auto Mission).

---

### 7.3 Đề Xuất 3 Hướng Đề Tài Nghiên Cứu Kèm Đánh Giá Tính Khả Thi

---

#### 🌟 HƯỚNG 1 (KHUYẾN NGHỊ CAO NHẤT - TÍNH KHẢ THI VƯỢT TRỘI)
**Tên đề tài:**  
*“A Cellular-Connected Multi-UAV Swarm Architecture with Delay-Adaptive Consensus Control under Bandwidth-Constrained MAVLink Telemetry”*  
*(Kiến trúc điều khiển bầy đàn UAV qua mạng di động tế bào với thuật toán đồng thuận thích nghi trễ truyền thông trên giao thức MAVLink giới hạn băng thông)*

- **Đóng góp mới (Novelty):**
  1. Đề xuất cơ chế điều tiết băng thông thích nghi (Adaptive Telemetry Rate Control - dựa trên các mức LVL 0-3 đã có trong tài liệu dự án) kết hợp bộ lọc Dead-Reckoning bù trễ mạng 4G.
  2. Xây dựng thuật toán điều khiển đồng thuận đội hình (Consensus-based Formation) chứng minh được tính ổn định Lyapunov khi độ trễ mạng biến thiên $0 < \tau(t) \le \tau_{\max}$.
- **Tận dụng từ dự án:** Tận dụng toàn bộ hạ tầng nghiên cứu về DroneEngage, các lệnh rate control trong file `udp-telemetry-commands.md`, và cấu hình RPi4/AirGap đã có sẵn.
- **Tính khả thi:** **Rất Cao (90%)**. Hoàn toàn có thể kiểm chứng bằng ArduPilot SITL kết hợp bay kiểm chứng 2 drone thực tế. Phù hợp đăng tại các hội nghị uy tín như **IEEE RIVF, IEEE ICUAS, VCCA** hoặc tạp chí trong nước có uy tín (REV Journal on Electronics and Communications).

---

#### 🚀 HƯỚNG 2 (THIÊN VỀ HỆ THỐNG & NỀN TẢNG GCS)
**Tên đề tài:**  
*“Design and Implementation of an IP-Based Ground Control Platform for Autonomous Multi-Drone Coordination in Beyond-Visual-Line-of-Sight (BVLOS) Operations”*  
*(Thiết kế và hiện thực nền tảng trạm điều khiển mặt đất trên nền IP phục vụ điều phối bầy drone tự hành ngoài tầm nhìn BVLOS)*

- **Đóng góp mới (Novelty):**
  1. Kiến trúc phân tán phân tầng: Web GUI tương tác thời gian thực kết hợp C++ GCS Engine định tuyến MAVLink phân luồng.
  2. Cơ chế chuyển trạng thái đội hình linh hoạt (Dynamic Formation Switching) giữa Cấu trúc ảo (Virtual Structure) và Bầy đàn hữu cơ (Organic Flocking) khi gặp không gian hẹp.
- **Tận dụng từ dự án:** Tận dụng 100% mã nguồn giao diện web hiện có (`DroneMap.tsx`, `ActionPanel.tsx`, `FloatingHUD.tsx`), chỉ cần phát triển thêm C++ backend như kế hoạch `PLAN_PHASE4.md`.
- **Tính khả thi:** **Khá Cao (75%)**. Thiên về bài báo ứng dụng hệ thống (Systems/Applied Engineering Paper).

---

#### 🎯 HƯỚNG 3 (THIÊN VỀ THUẬT TOÁN ĐIỀU KHIỂN & TRÁNH VA CHẠM ĐỐI KHÁNG)
**Tên đề tài:**  
*“Distributed Velocity-Obstacle Swarm Navigation and Target Encircling for Heterogeneous UAVs with Cellular Communication Latency”*  
*(Điều hướng bầy đàn phân tán và bao vây mục tiêu dùng chướng ngại vận tốc (VO) cho bầy UAV dị thể trong điều kiện trễ truyền thông di động)*

- **Đóng góp mới (Novelty):**
  1. Nâng cấp thuật toán tránh va chạm tiếp tuyến hiện tại lên thuật toán Chướng ngại vận tốc tương hỗ (ORCA - Optimal Reciprocal Collision Avoidance) trong không gian 3D.
  2. Bổ sung bài toán "Engagement / Pursuit": Bầy drone tự động chia vùng bao vây một mục tiêu di động (Target Encircling).
- **Tận dụng từ dự án:** Tận dụng ý tưởng vector trượt tiếp tuyến tại `ProtocolManager.ts#L491-L496` và cấu hình phần cứng 2 drone dị thể (1 con 5" và 1 con 7").
- **Tính khả thi:** **Trung bình (60%)**. Yêu cầu khối lượng toán học giải tích cao và cần bổ sung thêm module mô phỏng mục tiêu di động.

---

## 8. DANH SÁCH CÂU HỎI TRỌNG YẾU CẦN CHỦ DỰ ÁN PHẢN HỒI

Để hoàn thiện kế hoạch khởi động lại dự án và chuẩn bị đề cương NCKH chính xác, chủ dự án cần làm rõ các câu hỏi sau:

1. **Mục tiêu công bố khoa học:**
   - Dự án hướng tới bài báo cấp độ nào: Hội nghị trong nước (VCCA, REV-ECIT), Hội nghị quốc tế thuộc IEEE (RIVF, ICUAS, ASR), hay Tạp chí quốc tế Scopus/ISI (IEEE Access, Sensors, Unmanned Systems)?
   - Thời hạn nộp bài (Deadline dự kiến) là khi nào?
2. **Kế hoạch phần cứng thực tế:**
   - Chiếc Drone 2 (7 inch) hiện đang thiếu Flight Controller, dự kiến khi nào sẽ bổ sung (MicroAir H743 hay loại FC nào khác)?
   - Có kế hoạch trang bị chiếc Drone thứ 3 hay chỉ tập trung thí nghiệm bầy 2 drone vật lý kết hợp N drone ảo trong mô phỏng SITL?
   - Đã mua USB 4G Dongle (Huawei E3372) như kế hoạch Phase 4 chưa?
3. **Môi trường Server & Hạ tầng:**
   - Nghiên cứu sẽ tập trung vào chế độ **AirGap LAN nội bộ** (WiFi cục bộ của RPi4 - delay thấp) hay chế độ **Cellular 4G/LTE qua Cloud/VPS** (BVLOS đường dài)?
   - Bạn đã có VPS đặt tại Việt Nam/Singapore để thử nghiệm làm Relay server chưa?
4. **Phân bổ công nghệ (C++ vs TypeScript):**
   - Theo phản hồi từ CTO, C/C++ là bắt buộc cho production. Đối với bài báo NCKH, bạn muốn:
     + *Phương án A:* Giữ nguyên Web Client (React/TS) làm GCS giám sát, chỉ viết thuật toán swarm cốt lõi bằng C++ chạy trên Raspberry Pi / ROS2.
     + *Phương án B:* Viết lại toàn bộ hệ thống bằng C++/Qt (tương tự custom QGroundControl).
5. **Thuật ngữ chuyên ngành bí ẩn:**
   - Trong biên bản bàn giao có nhắc đến từ tiếng Anh bắt đầu bằng chữ **"am"** mà CTO đã đề cập khi nói về 2 mảng chính của ngành drone. Bạn đã nhớ ra hoặc hỏi lại được từ đó là gì chưa? (Gợi ý phân tích: Có thể là *Amateur UAV* đối lập với *Commercial/Enterprise*, hay *Autonomous Mobility*, hay *Air Mobility*?).

---
*(Báo cáo khảo sát được đóng gói hoàn chỉnh tại `docs/PROJECT_SURVEY.md`)*
