# Tổng quan kỹ thuật: NCKH swarm drone

Cập nhật 2026-11-19. Bối cảnh đầy đủ (đề tài, lịch, quyết định) có trong [`PROJECT_CONTEXT.md`](PROJECT_CONTEXT.md). File này chỉ tóm phần kỹ thuật.

## 1. Bài toán

Đội hình leader/follower nhiều UAV. Follower chỉ biết trạng thái leader qua bản tin bị **trễ τ** và gửi **thưa với chu kỳ T**, nên dữ liệu luôn có tuổi h ≈ τ + T/2.
Đề tài **benchmark độ chính xác đội hình theo tuổi dữ liệu** trên ArduPilot gốc. Mạng chỉ là đầu vào trễ được chèn có kiểm soát; nghiên cứu không tập trung vào bản thân mạng.
Mục tiêu nộp IEEE SMC 2027 (hạn 08/04/2027).

## 2. Kiến trúc hệ thống

```
 Leader UAV                         Server (AirGap / local)                Follower UAV
 ┌────────────┐  MAVLink  ┌───────────────┐   WSS/TLS   ┌───────────┐  WSS/TLS  ┌───────────────┐  MAVLink  ┌────────────┐
 │ ArduCopter │◄─────────►│ de_mavlink    │◄─(UDP)─►de_comm ◄────────►│ comm server   │◄────────►de_comm ◄─(UDP)─►│ de_mavlink    │◄─────────►│ ArduCopter │
 │ 4.7.2 (FC) │           │ (CSwarmLeader)│             │ :9966     │           │ (CSwarmFollower)│         │ 4.7.2 (FC) │
 └────────────┘           └───────────────┘             └─────┬─────┘           └───────────────┘           └────────────┘
                                                              │ S2S :19001
                                                        ┌─────┴─────┐
                                                        │ auth      │ HTTPS :19408 (login), admin :8089
                                                        └───────────┘
```

- **ArduPilot Copter 4.7.2 gốc**, không fork firmware. Ở SITL là binary tĩnh `/tmp/sitl/arducopter` trong WSL. Bay thật dùng MicroAir H743.
- **DroneEngage** (6 repo, tác giả Mohammad Hefny) là tầng truyền thông và điều phối swarm:
  - `droneengage_mavlink` (C++, binary `de_ardupilot`): cầu nối FC, logic swarm leader/follower.
  - `droneengage_communication` (C++, `de_comm`): client trên companion, nối tới server.
  - `droneengage_server`: comm server, định tuyến bản tin. Từ 30/10/2026 upstream là **Rust**, trước đó là Node.js.
  - `droneengage_authenticator`: login/account/S2S, cũng đã port sang **Rust**.
  - `droneengage_common` (C++ dùng chung), `droneengage_webclient_react` (GCS web).
- **Bản tin swarm:** 6503 `SWARM_MAVLINK` (leader gửi `GLOBAL_POSITION_INT` + `ATTITUDE` cho từng follower, đo được khoảng 1 Hz: 135 bản tin mỗi lượt gate). Upstream không có timestamp hay sequence.
- **Companion:** Raspberry Pi 4 (drone 5"), Pi 3+ (drone 7", còn thiếu FC).

## 3. Các bản code đang có

| Thư mục | Là gì | Trạng thái |
|---|---|---|
| `de_fork/` | Fork 6 repo = **baseline B0** + P1–P4 + rebrand (`5.7.0-fork`). Patch là thay đổi uncommitted, xem `NOTES.md` / `PATCHES.md` | Server/auth vẫn là bản **JS** (pin `53a092f` / `584d842`) |
| `de_upstream_oct_2026/` | Clone upstream mới (server `20900c0`, auth `9274556`, 4 repo còn lại trùng pin) + P3/P4 port sang Rust + rebrand `de_fork_*` | Uncommitted. `SUMMARY.md`, `report.txt`, `explain.txt`, `C4_router_6546.js` |
| `nckh_research/` | Lit review, verification, gate pilot, đề xuất v3, `de_sitl_tools/` (exp1/analyze/delayproxy) | |
| `_archive/gcs_clone_2026-06/` | GCS React/TS cũ | Không dùng |

### Patch của fork
| Patch | Nơi | Nội dung |
|---|---|---|
| P1 | de_mavlink | Leader gắn `ms.q` (uint32 seq) + `ms.t` (ms từ boot của FC) vào 6503. Follower bỏ gói cũ/trùng (so wraparound `int32(seq-last) <= 0`). Timeout leader 5 s (gọi 10 Hz) reset LPF |
| P2 | de_mavlink | Cài handler `FORMATION_VECTOR` (=3) cho cả 2 phía |
| P3 | auth | Rate limit login agent 10 → 1000/phút (để chạy SITL hàng loạt). JS: `js_router_agent.js`; Rust: `src/web/mod.rs:97` |
| P4 | server | Trả port công khai `de_public_port` (int) khi chạy sau proxy trễ, fallback `de_comm_server_port` rồi `server_port` |

**B0 giữ nguyên:** không có velocity feed-forward. Follower chỉ gửi waypoint qua `gotoGuidedPoint`, có LPF, nên tụt sau leader khoảng v × 8 s.

## 4. Testbed SITL (WSL Ubuntu 22.04 trên máy GTR)

- 3 × SITL Copter 4.7.2 (`-I0..2`, home lệch 0.00012° lon). Param bắt buộc: `MAV_SYSID`, `FRAME_CLASS 1`, `FRAME_TYPE 0`, `INS_ACC*OFFS/SCAL`, `WP_SPD 5`. Lưu ý 4.7 đã đổi tên `SYSID_THISMAV` và `WPNAV_SPEED`.
- auth :19408 (S2S :19001), server :9966, **delayproxy** :9967 (trễ mỗi chặng đọc từ `link.cfg`; 500 ms/chặng = 1 s đầu-cuối), 3 × `de_comm` + `de_ardupilot`.
- `exp1.py` (pymavlink): arm, takeoff 15 m, gán Arrow d = 15 m, leader bay 250 m Bắc rồi 250 m Đông (khoảng 4.8 m/s), log `GLOBAL_POSITION_INT` 10 Hz. `analyze.py`: sai số xác lập trong 2 cửa sổ 20–65 s và 100–125 s.
- Harness và các mẹo vặt (setsid, `timeout` wrapper cho SITL, gọi WSL qua PowerShell) có trong scratchpad `gate/` và memory `sitl-4-7-testbed`.

## 5. Kết quả đo đến nay

| Gate B0, n = 6 (3 lượt × 2 follower) | τ = 0 | τ = 1 s |
|---|---|---|
| 2026-11-09, SITL 4.6.3, cloud | 38.45 ± 2.33 m | 48.9 ± 21.1 m |
| 2026-11-18, SITL 4.7.2, fork JS | 39.44 ± 1.46 m | 44.85 ± 2.44 m |
| 2026-11-18, upstream Rust + P3/P4 + rebrand | 39.67 ± 1.33 m | 45.78 ± 1.53 m |

- Gate cũ còn đo P1R (dự báo theo lúc nhận) 1.70 / 6.53 m và P1 (dự báo theo lúc gửi) 1.61 / 1.73 m. Ý nghĩa: chỉ cần feed-forward là bỏ được sai số 38 m, còn bù theo send-time thì triệt được trễ nền. Kết quả này mới đúng cho đoạn bay thẳng.
- C4 trên server Rust: 6503/6546 đi qua nguyên vẹn. `outbound_guard` **bỏ** frame 6503 (latest_wins) khi buffer > 256 KiB, chứ không giữ frame mới nhất.

## 6. Bước kỹ thuật kế tiếp (S1, 30/11 – 20/12/2026)

- Baseline chính chuyển sang **ArduPilot FOLLOW 4.7.2 (B1)**. Node Python trên companion follower nhận 6503 rồi gửi `FOLLOW_TARGET` (#144, có vel/acc/rates), với `FOLL_OFS_TYPE=1`.
- Gắn timestamp GPS phía leader. Thêm khối chèn trễ/lấy mẫu thưa.
- Ghi 4 luồng log: leader thật, bản tin đã gửi, `FOLL` của firmware, follower thật.
- Viết script chạy hàng loạt. Chạy E0 → mốc G1 (H1: 10/10 lần chạy, sai số < 2 m khi không trễ).
- Sau đó: E1–E4 (quét τ, T, ω, d, lệch đồng hồ), bộ bù P1/P2 nếu qua G2, rồi bay thật 2 drone (S1b).

## 7. Rủi ro và nợ kỹ thuật

- **Bảo mật upstream:** private key TLS airgap cũ còn trong các tag public và khớp cert mặc định trên master. Fork dùng cert riêng `CN=127.0.0.1`. Auth mặc định có `session_secret` placeholder, access code `test`, admin terminal mở `0.0.0.0:8089`. Chỉ chạy trong LAN kín.
- **Giấy phép:** Toàn bộ repo thống nhất theo **MIT** (`LICENSE` tại gốc). Trước đó có trao đổi qua email của tác giả gốc Mohammad Hefny (16/11) và metadata Cargo upstream ghi MIT (auth) / UNLICENSED (server); lưu văn bản xác nhận vào `docs/license/`.
- **Fork chưa rebase** lên upstream Rust. Patch Rust đã sẵn và đã chạy gate, nhưng chưa test lại sau khi đổi tên `de_fork_*`, vì `cargo test` toàn bộ làm WSL bị restart.
- Gate mới chỉ có bay thẳng, cùng đồng hồ, không jitter/mất gói. Đóng góp khoa học phải đến từ đoạn rẽ, lấy mẫu thưa, lệch đồng hồ.
