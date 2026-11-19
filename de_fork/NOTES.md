# NOTES: `de_fork` (DE custom baseline B0 — v5.7.0-fork)

- **Ngày cập nhật:** 2026-11-19
- **Chủ sở hữu:** Ferb27
- **URL base:** <https://github.com/Ferb27/port_drone_rust>
- **Phiên bản:** `5.7.0-fork` (`DE_FORK=1`)
- **Mục đích:** Fork 6 repo DroneEngage để làm testbed & baseline **B0** cho đề tài NCKH (benchmark sai số formation theo tuổi dữ liệu trên ArduPilot). **Không** thêm velocity feed-forward vào follower (giữ nguyên hành vi B0: follower gửi waypoint qua `gotoGuidedPoint`, có LPF).
- **Giấy phép:** **MIT** (xem `../LICENSE` ở thư mục gốc; nguồn gốc DroneEngage từ Mohammad Hefny).
- **Upstream gốc:** Clone nguyên bản tại `.upstream_orig/` (không chỉnh sửa). Tất cả thay đổi ở `de_fork/` đều là **uncommitted patch**.

## Manifest Upstream Đã Pin

| Repo (`de_fork/`) | Upstream (`github.com/DroneEngage/`) | SHA đã pin | Số file (tracked) | LOC (ước lượng) | Ghi chú |
|---|---|---|---|---|---|
| `de_mavlink` | `droneengage_mavlink` | `be5d8a5` | 655 | 221,430 | Exclude `wiki/CFCBMaon::connectToFCB.md` khi checkout trên NTFS |
| `de_communication` | `droneengage_communication` | `206b5a2` | 129 | 37,952 | Rebrand + `DE_FORK=1` + `LICENSE` |
| `de_server` | `droneengage_server` | `53a092f` (v5.6.2, 2026-09-18) | 59 | 6,670 | Bản JS; đã reset HEAD về `53a092f`, P4 uncommitted |
| `de_authenticator` | `droneengage_authenticator` | `584d842` (2026-10-17) | 83 | 13,307 | Bản JS; đã reset HEAD về `584d842`, P3 uncommitted |
| `de_common` | `droneengage_common` | `5fb11c2` | 58 | 33,889 | Rebrand + `DE_FORK=1` + `LICENSE` |
| `de_webclient_react` | `droneengage_webclient_react` | `01710102` | 2,352 | 92,378 | Rebrand + `5.7.0-fork` + `LICENSE` |

## Bảng Tổng Hợp B0 / P1–P4 & Rebrand

| Hạng mục | Phạm vi | Nội dung chính | Trạng thái |
|---|---|---|---|
| **B0** | `de_mavlink` | Giữ nguyên baseline: **không** thêm velocity feed-forward; follower gửi waypoint qua `gotoGuidedPoint` + LPF (`TARGET_SMOOTHING_ALPHA`). | Giữ nguyên |
| **P1** | `de_mavlink` | Leader gắn `ms.q` (`uint32_t` seq) + `ms.t` (`uint32_t` ms từ boot FC, fallback `monotonic_ms`) vào msg `6503` (`SWARM_MAVLINK`). Follower lọc gói cũ/trùng bằng so wraparound `int32_t(seq - last) <= 0`. Timeout leader 5 s, gọi 10 Hz (`checkLeaderTimeout()`), reset LPF. | Uncommitted |
| **P2** | `de_mavlink` | Cài handler `FORMATION_VECTOR` (`= 3`) cho cả `CSwarmLeader` (`broadcastLeaderState()`) và `CSwarmFollower` (`updateFollowerInVectorFormation()`, dùng chung `resolveFormationBearing()` và LPF B0). | Uncommitted |
| **P3** | `de_authenticator` | Nâng `RATE_LIMIT_MAX_REQUESTS` trong `src/routes/js_router_agent.js` từ `10` → `1000` req/phút để chạy SITL hàng loạt. | Uncommitted |
| **P4** | `de_server` | Trả cổng công khai `de_public_port` (ép kiểu `parseInt(..., 10)`) trong `src/comm_server/js_andruav_comm_server.js` khi chạy sau proxy trễ, fallback `de_comm_server_port` rồi `server_port`. | Uncommitted |
| **Rebrand** | Cả 6 repo | Thống nhất sang chủ sở hữu `Ferb27` và giấy phép **MIT** tại `../LICENSE`. | Uncommitted |
| **AirGap / pnpm** | `de_server`, `de_authenticator` | Đổi `git+ssh://` → `https://` cho `droneengage_server_common`; thêm `body-parser` cho `pnpm`; cấu hình AirGap (`0.0.0.0`, `127.0.0.1`, `enable_storage_server: false`, `altcha_enabled: false`). | Uncommitted |
