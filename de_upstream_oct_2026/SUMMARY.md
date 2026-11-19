# Upstream DroneEngage, đợt commit tháng 10–11/2026, so với pin `de_fork`

Ngày fetch: 2026-11-18. Thư mục này chứa 6 clone mới (checkout `origin/master`) và `patches/`.
`de_fork/` **không bị động tới**: chưa rebase, pin cũ giữ nguyên.

## Tóm tắt

| Repo | Pin fork | `origin/master` | Commit mới | Version | Tag mới |
|---|---|---|---|---|---|
| droneengage_mavlink | `be5d8a5` | `be5d8a5` (2026-10-09) | 0 | không đổi | không |
| droneengage_communication | `206b5a2` | `206b5a2` (2026-10-17) | 0 | không đổi | không |
| droneengage_common | `5fb11c2` | `5fb11c2` (2026-10-24) | 0 | không đổi | không |
| droneengage_webclient_react | `0171010` | `01710102` (2026-10-17) | 0 | không đổi | không |
| droneengage_server | `53a092f` | `20900c0` (2026-11-13) | **74** (09: 2, 10: 40, 11: 32) | 5.6.2 → **6.0.2** (Cargo) | không (tag mới nhất vẫn là `ver_5.4.3`, 08/2026) |
| droneengage_authenticator | `584d842` | `9274556` (2026-11-13) | **25** (10: 9, 11: 16) | 7.7.0 → **7.7.3** (Cargo) | không (tag mới nhất `ver_7.4.0`, 08/2026) |

Các nhánh khác (`release`, `pr_*`, `sep_2025`, …) đều cũ hơn (≤ 08/2026) và rẽ nhánh, không phải đợt mới.

**Thay đổi lớn nhất:** server và auth đã được **port sang Rust** vào 2026-10-30 (`9622500`, `a4f1796`). `package.json` bị xoá, thay bằng `Cargo.toml`. Các file JS mà P3/P4 vá không còn tồn tại.

## droneengage_server: 74 commit, `53a092f..20900c0`

`git diff --stat`: 124 file, +35187/−7258 (72 thêm, 42 xoá, 10 sửa).

| Loại | File | +/− | Nội dung |
|---|---|---|---|
| Rust `.rs` | 41 | +10475/−0 | Server viết lại bằng Rust: hub, router, chat, s2s (Ed25519/HMAC), udp relay client |
| JS | 40 | +20472/−7060 | Xoá server JS cũ; vendor plugin JS cũ (`command_processing`, `aisstream`, lib mavlink20) |
| config/scripts | 21 | +880/−57 | `server.config` (udp_relay, outbound_guard, s2s_shared_secret_file), `test.server.config`, `gl0041.config`, `deployment/*`, `.env` của plugin |
| docs | 11 | +1777/−98 | wiki UdpRelay, PluginEvents, benchmark Node vs Rust, LIVE_TEST_ON_RPI |
| khác | 11 | +1583/−43 | Cargo.lock, bench loadgen, file op SAR |

Commit theo ngày (rút gọn: những commit cùng chủ đề trong ngày gộp thành một dòng):
- 09-24 `1d1df61` outbound backpressure guard và sender blocking
- 09-30 `dcee538` Phase 4A message constants, roster latest_wins, clock ts
- 10-01 `7ebc545` mission list/save passthrough
- 10-02 `1861638` never-queue list cho fleet coordination
- 10-08 `2333e8c` `99b0f77` `fac2937` `eaf257b` CI pure-gates (git+ssh → https trên runner, `SERVER_COMMON_PAT`)
- 10-14 `a8a5be3` pin server_common v1.0.1; `ec9259a` storage S2S HMAC, bỏ no-auth fallthrough
- 10-15 `8c74f7a` msg 6546 AI_OBSERVATIONS; `4347e6f` dọn latest_wins
- 10-17 `7d3a54c` forward load-mission replies verbatim
- 10-21 `04b0997` e2e config (no auth, no storage); `9ef8941` bug batch comm_server; `8d531b6` wiki PluginEvents
- 10-22 `2653d5b` … `7a4d42c` (17 commit) UDP relay client, multi-relay failover, UDP proxy key theo (account, unit), các commit merge
- 10-23 `33ed82f` merge relay vào master
- 10-24 `cb778cc` op mẫu SAR forest
- 10-30 `9622500` **port comm server sang Rust**; `74c7dc3` `2d187aa` `8e2f41c` `da78477` docs và server_common
- 10-31 `93dada2` `d3b26a9` AGENTS.md
- 11-01 `5e88f82` vendor legacy JS plugins
- 11-02 `63070c3` … `27dbb74` (10 commit) **bỏ hẳn UDP proxy built-in**, benchmark harness, tối ưu hot path (header scan, batch frames, plugin workers)
- 11-03 `0682303` `06359d2` docs perf; `446f299` `96d1d48` 9021 actions; `dfa8c8d` cảnh báo relay thiếu secret; `2ee39eb`; `37ca942` benchmark Pi 4
- 11-05/06 `1f76660` `d0566bb` `6574a68` `668c659` `245a1e2` udp relay token và tests
- 11-09 `14b4961` test.server.config
- 11-10 `baa5884` v6.0.1; 11-11 `d748ed6` log WebRTC/1059; 11-12 `b89f955` v6.0.2
- 11-13 `b1422aa` `bb08c08` `182e462` `20900c0` public-readiness (.gitignore key, scrub LAN IP và đường dẫn, link repo public)

## droneengage_authenticator: 25 commit, `584d842..9274556`

`git diff --stat`: 115 file, +15196/−10779 (44 thêm, 40 xoá, 10 sửa, 21 rename).
Thành phần: Rust 31 file +9673; JS 37 file +34/−10369 (xoá auth JS); config/scripts 7 file +3359/−52 (chủ yếu `package-lock.json`, `ecosystem.local.config.js`, `test.server.config`, `scripts/*`).

- 10-14 `0a4abc6` cap độ dài accountName, pin server_common v1.0.1; `4bffa63` regenerate cần ownership proof, recovery code
- 10-18 `e7c3f1f` merge `backupDT/master`
- 10-19 `5686dd5` ecosystem.local.config.js (PM2)
- 10-22 `769b6f4` admin dashboard UDP relay status
- 10-24 `a2d6943` gitignore
- 10-30 `a4f1796` **port authenticator sang Rust**
- 10-31 `279049e` `b403a1d` AGENTS.md
- 11-01 `8c84fe0` server_common master, **license MIT**; `c5932fc` cargo source override
- 11-02 `b351122` UDP proxy rows; `cf800ec` package-lock.json
- 11-08 `07c42ef`; 11-09 `a650241` test.server.config
- 11-10 `76fb04d` cookie name theo install; `24d7f59` v7.7.1
- 11-11 `d1e2b1b` permission gửi dạng u32
- 11-12 `533d588` production/shadow cutover scripts, v7.7.3
- 11-13 `2c795b4` `3086e26` `0dbf9b9` `f104086` `6b1328c` `9274556` public-readiness

## Thứ nhạy (mỗi dòng: gì, tại sao lộ)

- **Private key TLS airgap** `ssl_local/ssl_airgap/domain.key`: commit `7573b36` (server, 07-13) và `3a9f4a0` (auth, 07-14) vẫn truy cập được qua tag public `ver_5.0.0…ver_5.4.2` (server) và `ver_6.0.0…ver_7.3.1` (auth). Lộ vì commit "public-readiness" chỉ thêm `.gitignore` chứ không xoá lịch sử. Key này **khớp public key** với `domain.crt` trên master (CN=airgap.local, hết hạn 2035), nên mọi bản cài airgap dùng cert mặc định đều có thể bị MITM.
- **Private key S2S Ed25519** `ssl_local/DE_ServerComm_private.pem`: commit `76417da` (auth, 07-14), nằm trong tag `ver_6.0.0…ver_7.3.1`. Lộ cùng lý do trên. Public key trên master **không khớp**, tức đã rotate, nên rủi ro chỉ còn với các bản cài cũ.
- **Fork không dính:** `de_fork/*/ssl_local/ssl_airgap/domain.key` có sha256 `a9408a09…`, khác key bị lộ (`97c28571…`). Cert của fork là self-signed `CN=127.0.0.1`, khớp public key với key của fork. `DE_ServerComm_public.pem` của fork trùng upstream (khác hash file chỉ do CRLF). Fork không có `DE_ServerComm_private.pem`.
- **Env default** `src/plugins/command_processing/.env` (server, `5e88f82`): commit cả file `.env` (ZMQ `tcp://127.0.0.1:5555`, thư mục log/download). Không có secret, nhưng là file env bị track lẽ ra phải ignore.
- **Default secret (auth `server.config`, không đổi từ pin, fork thừa hưởng):** `session_secret "change-this-secret-in-production"`, `single_account_access_code "test"`, `webadmin_terminal_enabled true` trên `webadmin_listening_ip 0.0.0.0:8089`. Lộ vì là giá trị mặc định công khai: ai vào được LAN là có admin terminal. Fork còn tắt altcha, nên càng hở.
- **Port/host:** `test.server.config` và `deployment/server.gl0041.config` thêm port 9967 và relay 9100. Commit `bb08c08` scrub IP LAN `10.x` và đường dẫn cá nhân, nhưng chúng **vẫn còn trong các commit trước đó** của lịch sử public.
- **CI secret:** workflow dùng `SERVER_COMMON_PAT` (`eaf257b`). Chỉ là tên secret, không lộ giá trị.
- **License metadata:** Cargo của server ghi `license = "UNLICENSED"`, của auth ghi `"MIT"`, cả hai không có file LICENSE. Điều này mâu thuẫn với grant The Unlicense riêng cho nhóm, nên cần giữ email grant làm bằng chứng.

## Áp patch fork lên bản mới (thử trong `patches/`, không commit)

| Patch | `git apply --check` lên master | Ghi chú |
|---|---|---|
| P1+P2 `P1P2_de_mavlink.patch` | **OK** | mavlink không đổi, áp nguyên văn |
| P3 JS `P3_de_authenticator_js.patch` | FAIL | `src/routes/js_router_agent.js` đã bị xoá |
| P3 Rust (port) `P3_de_authenticator_rust.patch` | OK (chưa compile) | `src/web/mod.rs:97` `agent_login_limit: SlidingLimiter::new(10)` → `1000` (+1/−1) |
| P4 JS `P4_de_server_js.patch` | FAIL | `src/comm_server/js_andruav_comm_server.js` đã bị xoá |
| P4 Rust (port) `P4_de_server_rust.patch` | OK (chưa compile) | Thêm `Env.public_port` và `public_port_value()` (parse int, fallback `de_comm_server_port`, rồi `server_port`), dùng cho login reply `comm_server.rs:38` và watchdog `server.rs:119`. Port listen vẫn dùng `server_port_value()` (+13/−2). Rust upstream vẫn trả `de_comm_server_port` dạng **String**, tức lỗi kiểu dữ liệu mà P4 sửa vẫn còn |
| Rebrand webclient | OK | |
| Rebrand server/auth | FAIL | Không còn `package.json`; cần làm lại trên `Cargo.toml` (name/version) và `server.config` (conflict ở dòng 26 do có thêm block udp_relay) |

### Re-check C2/C3

- **C2 PASS, không đổi:** mavlink, communication và common giống hệt pin, nên follower B0 không đổi một byte nào. Không có velocity feed-forward.
- **C3 PASS, không đổi** (`fcb_swarm_leader.cpp:19–24`, `fcb_swarm_follower.cpp:453`). Lưu ý mới ở tầng transport: server Rust đưa **6503 vào `outbound_guard.latest_wins_message_types`** (`server.config`). Khi socket vượt high-water 256 KB, server chỉ giữ frame 6503 mới nhất. Điều này hợp với bộ lọc `q` của P1, nhưng làm đổi thống kê mất gói khi đo trễ. Ngoài ra router Rust định tuyến theo header scan (`5c8cf0c`), nên cần kiểm chứng `ms.q`/`ms.t` đi qua nguyên vẹn.

## Gate τ=0/1 s

Đã chạy lại trên stack mới, sau khi người dùng đồng ý cài rustup (cargo 1.100.0, cài trong WSL).

Thiết lập:
- Build server Rust tại `20900c0` kèm P4 port, auth Rust tại `9274556` kèm P3 port, server_common `c22630b`. `cargo build --release` exit 0; `cargo test` pass (server lib 3/3, auth router_security 2/2).
- Config AirGap giống fork: `server_ip 0.0.0.0`, `public_host 127.0.0.1`, storage off, `udp_relay []`, `command_plugin []`, altcha off, cert của fork.
- de_comm và de_ardupilot của fork (B0+P1), SITL 4.7.2, leader 4.88 m/s, proxy trễ đặt `de_public_port=9967`. Cả 3 de_comm đều kết nối qua proxy, chứng tỏ P4 port hoạt động.

| Stack | τ=0 (n=6) | τ=1 s (n=6) |
|---|---|---|
| Server/auth **Rust** (upstream mới + P3/P4) | **39.06 ± 2.23 m** (36.3–43.0) | **45.59 ± 1.74 m** (43.0–47.9) |
| Server/auth JS (pin fork) | 39.44 ± 1.46 | 44.85 ± 2.44 |
| Gate 2026-11-09 (SITL 4.6.3) | 38.45 ± 2.33 | 48.9 ± 21.1 |

Ở τ=0, kết quả **PASS** dải 38–44 m. Ở τ=1 s, sai số cao hơn biên trên 1.6 m, giống bản JS. Stack Rust **không làm đổi hành vi B0**: chênh lệch so với JS nằm trong 1 sd. `latest_wins` của 6503 không kích hoạt, vì ở 10 Hz không chạm high-water.

Kết quả lưu trong WSL: `/tmp/rv_gate/out` (Rust), `out_js_5ms` (JS), `out_9ms` (lần chạy sai tốc độ). Script nằm trong scratchpad `gate/` (`setup_rust.sh`, `all_rust.sh`).

## Cập nhật: áp P3/P4 Rust + rebrand vào clone (2026-11-18, người dùng yêu cầu)

Sửa trực tiếp trên working tree của `droneengage_server` và `droneengage_authenticator` trong thư mục này (chưa commit). `de_fork/` vẫn giữ nguyên.

| Repo | File | Thay đổi |
|---|---|---|
| server | `src/server.rs` | P4: `Env.public_port` (`de_public_port`) và `public_port_value()` (parse int, fallback `de_comm_server_port`, rồi `server_port`). Watchdog dùng hàm này ở `serverPort`. +12/−1 |
| server | `src/comm_server.rs:38` | P4: login reply dùng `public_port_value()`. +1/−1 |
| server | `Cargo.toml:2-3` | `name = "de_server"`, `version = "5.7.0-fork"`. `[lib] andruavserver` và bin `andruavmainserver` giữ nguyên |
| server | `server.config` | `server_ip "0.0.0.0"`, `public_host "127.0.0.1"`, `enable_storage_server false`. Block `udp_relay` (dòng 18–22) và `command_plugin` **giữ nguyên upstream**: bỏ qua conflict, đúng yêu cầu |
| auth | `src/web/mod.rs:97` | P3: `agent_login_limit: SlidingLimiter::new(1000)`. +1/−1 |
| auth | `Cargo.toml:2-3` | `name = "de_authenticator"`, `version = "5.7.0-fork"`. `[lib] de_auth_server` và bin `andruavauth` giữ nguyên |
| auth | `server.config` | `altcha_enabled false`. `account_storage_type single`, `server_ip 0.0.0.0` và S2S `127.0.0.1` vốn đã là mặc định upstream |

**Build** (WSL, bản copy `/tmp/rv_rust`): cả hai `cargo build --release` exit 0. Cargo.lock tự cập nhật tên package.

**cargo test:**
- server: 88/88 pass. Cần có cert trong `ssl_local/ssl_airgap/`, mà upstream không ship key, nên tôi dùng cert của fork.
- auth: 34/35 pass. Test fail là `web_app::admin_login_page_and_csrf` (`tests/web_app.rs:93`, kỳ vọng cookie `_csrf=`). Test này **cũng fail trên upstream sạch `9274556`**, khả năng do `76fb04d` đổi tên cookie. Không liên quan patch.

**Gate B0** trên stack Rust đã rebrand, dùng đúng `server.config` của repo, không override:

| Stack | τ=0 (n=6) | τ=1 s (n=6) |
|---|---|---|
| Rust + P3/P4 + rebrand config | **39.67 ± 1.33 m** (37.8–41.5) | **45.78 ± 1.53 m** (43.7–48.2) |
| Rust + P3/P4 (config tạm, relay/plugin tắt) | 39.06 ± 2.23 | 45.59 ± 1.74 |
| JS pin fork | 39.44 ± 1.46 | 44.85 ± 2.44 |

Ở τ=0, kết quả **PASS** dải 38–44 m. Ở τ=1 s, sai số cao hơn biên trên khoảng 1.8 m, giống mọi stack trước. B0 không đổi. Auth log báo `UDP relay DE_UdpRelay_1 status: ERROR` vì không có relay nào chạy ở 9100. Điều này không ảnh hưởng tới swarm, vì 6503 không đi qua UDP relay.
