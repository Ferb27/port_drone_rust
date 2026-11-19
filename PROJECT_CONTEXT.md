# PROJECT_CONTEXT: bối cảnh đầy đủ của đề tài NCKH swarm drone

> File này dành cho **bất kỳ AI hoặc người mới nào** cần hiểu project mà không có lịch sử trò chuyện trước đó. Đọc hết file này trước khi làm gì khác.
> Cập nhật lần cuối: **2026-11-18**. Nếu thông tin ở đây mâu thuẫn với file mới hơn trong `nckh_research/`, file mới hơn thắng (kiểm tra ngày trong tên file).
> Ngôn ngữ làm việc: tiếng Việt, giữ thuật ngữ kỹ thuật bằng tiếng Anh.

---

## 1. Tóm tắt trong 30 giây

- **Đây là gì:** đề tài nghiên cứu khoa học sinh viên (NCKH) của 3 sinh viên năm 3, mục tiêu ra **một bài báo hội nghị** rồi phát triển thành đồ án tốt nghiệp.
- **Đề tài (bản v3, 2026-11-16):**
  - VI: *Xây dựng nền tảng bay đội hình nhiều UAV dựa trên DroneEngage và ArduPilot, và đánh giá ảnh hưởng của độ trễ truyền thông tới độ chính xác đội hình.*
  - EN: *An Open Multi-UAV Formation Platform on DroneEngage and ArduPilot: Benchmarking Formation Accuracy under Communication Delay.*
- **Nơi nộp:** IEEE SMC 2027 (6-10/10/2027, TP.HCM, HCMUTE đăng cai). **Hạn nộp full paper: 2027-04-08 (AoE)**, regular paper tối đa 6 trang. Phương án hai: hội nghị IEEE trong nước/khu vực.
- **Cách làm:** ArduPilot Copter **4.7.2 bản gốc** (không fork firmware) chạy SITL, DroneEngage làm tầng truyền thông + điều phối swarm, nhóm chỉ viết thêm ở tầng **companion computer / MAVLink** (Python/pymavlink, Lua nếu cần).
- **Góc nhìn bắt buộc:** đây là bài **điều khiển / tự động hóa**. Mạng truyền thông chỉ là **đầu vào trễ cho trước** (chèn có kiểm soát), không nghiên cứu bản thân mạng.
- **Trạng thái hiện tại:** đã kiểm chứng DroneEngage (GO), đã chạy gate pilot SITL (GO), đề xuất v3 đã viết xong. **Chưa gửi email cho giảng viên hướng dẫn.** Giai đoạn kế tiếp là S1 (dựng nền tảng SITL, 30/11 - 20/12/2026).

---

## 2. Bài toán và ý tưởng khoa học

### 2.1 Bài toán
Follower chỉ biết trạng thái leader qua bản tin đến **trễ τ** và **thưa với chu kỳ T**, nên dữ liệu luôn có "tuổi" **h ≈ τ + T/2**. Thông tin đi một chiều leader → follower, nên trễ **nằm ngoài vòng phản hồi** của follower. Bài toán là **bám tham chiếu bị trễ**: câu hỏi là sai số đội hình tăng thế nào theo tuổi dữ liệu, **không phải** hệ có mất ổn định không.

> Lưu ý thuật ngữ: **không dùng "delay margin" hay "formation stability"** cho bài toán này (sai khái niệm, đã sửa khỏi đề xuất). Dùng "tracking-error bound theo tuổi dữ liệu" / "age budget".

Slot mong muốn: `s = p + R(ψ)·r`, với p, ψ là vị trí/hướng leader, r là offset đội hình, |r| = d.

### 2.2 Mô hình sai số tham chiếu (Bảng 1 trong đề xuất v3)
| Cách follower xử lý tuổi dữ liệu | Sai số tham chiếu xấp xỉ |
|---|---|
| Không dự báo | `v_s·h`, `v_s = |v + ω × R(ψ)r|`; quỹ đạo tròn: `2ρ·sin(ωh/2)` |
| Dự báo từ lúc **nhận** (giống FOLLOW 4.7) | `≈ v_s·τ₀` (phần trễ nền không thấy được) |
| Dự báo từ lúc **gửi**, tuyến tính | `≤ ½·v·ω·h² + 2d·sin(ωh/2)` |
| Dự báo từ lúc gửi, có gia tốc và ω | `≤ (1/6)·j_max·h³ + ½·d·α_max·h²` |
| Sai lệch đồng hồ δ | cộng thêm `≈ v_s·|δ|` |

Đây chỉ là sai số **tham chiếu**. Sai số thật còn gồm sai số bộ ước lượng mục tiêu trong firmware và độ trễ bám của position controller; thí nghiệm phải log riêng cả ba.

### 2.3 Phát hiện then chốt về ArduPilot FOLLOW (đọc mã nguồn)
- Copter **4.7.x** `AP_Follow` đã ngoại suy vị trí (v·dt + ½a·dt²), hướng (heading rate), và cộng `ω × r`, `α × r + ω × (ω × r)` cho offset. Các phần quay chỉ chạy khi **`FOLL_OFS_TYPE = 1`** và nhận **`FOLLOW_TARGET` (#144)** có rates. Đường `GLOBAL_POSITION_INT` không có heading rate/accel.
- Tuổi dữ liệu được tính qua **`JitterCorrection`** (cửa sổ 500 ms, tính lại sau 100 mẫu), lấy trễ nhỏ nhất quan sát được làm mốc 0 → **trễ nền τ₀ không được bù**, follower sẽ tụt ≈ `v·τ₀`. **Điều này mới là suy ra từ code, CHƯA đo** (là nội dung của mốc G2).
- Timestamp của `FOLLOW_TARGET` là ms từ lúc boot của máy gửi, không phải UTC → bộ bù phải tính tuổi theo GPS/UTC trên companion, và phải tránh **bù hai lần** (firmware ngoại suy thêm trên cùng khoảng thời gian).
- Vì vậy "ước lượng ω / cộng ω × r" **không phải đóng góp mới**. Baseline chính phải là **FOLLOW 4.7.2 (B1)**, không phải follower tự viết yếu hơn.

### 2.4 Đóng góp (đã reframe "benchmark-first" ngày 2026-11-11, chốt v3 ngày 2026-11-16)
1. **Đóng góp 1 (kỹ thuật):** nền tảng đội hình mở DroneEngage + ArduPilot 4.7.2 gốc: node đội hình trên companion (DroneEngage → `FOLLOW_TARGET`), khối chèn trễ, logging, script tự động, harness pymavlink thuần làm dự phòng/đối chứng.
2. **Đóng góp 2 (lõi khoa học):** benchmark có kiểm soát FOLLOW 4.7.2 dưới trễ nền, lấy mẫu thưa, khi rẽ; tách 3 lớp sai số; đối chiếu với mô hình giải tích ở 2.2 (mô hình là kiến thức nền, ghi công đúng, không coi là lý thuyết mới).
3. **Phụ thuộc kết quả:** bộ bù trên companion (tuổi theo GPS time, truyền trạng thái tới hiện tại rồi đưa vào FOLLOW gốc). **Chỉ làm nếu qua G2.**
4. **Kiểm chứng:** bay thật 2 drone (5" và 7") qua nền tảng. Bài hội nghị **không phụ thuộc** phần này.

Kết quả âm tính (FOLLOW 4.7.2 đã đủ tốt) vẫn viết thành bài được. **Không được trình bày như một phương pháp bù trễ mới**: mọi thành phần đều có prior art (xem mục 9).

### 2.5 Câu hỏi nghiên cứu (RQ) và giả thuyết
| RQ | Giả thuyết / tiêu chí |
|---|---|
| RQ1: nền tảng giữ được đội hình ổn định trên SITL và 2 drone thật? | H1: SITL 1 leader + 2 follower giữ đội hình **10/10 lần chạy**, sai số xác lập không trễ **< 2 m** trên đoạn thẳng |
| RQ2: FOLLOW 4.7.2 để sót bao nhiêu sai số khi có trễ nền τ₀? | H2: phần sót **> 3 sd** giữa các lần chạy ở τ₀ = 1 s và nằm trong **±30%** của `v_s·τ₀` |
| RQ3: mô hình 2.2 dự đoán đúng tới đâu khi đổi h, v, ω, d? | H3: lệch mô hình - số đo trong ±30%; chỉ ra được khi nào mô hình sai |
| RQ4: bộ bù giảm được bao nhiêu so với FOLLOW gốc, kể cả khi leader đổi chuyển động bất ngờ? | H4: giảm **≥ 50%** sai số do trễ ở τ₀ ≥ 0,5 s, không tăng p95/max ở ca dừng gấp, đảo chiều |

### 2.6 Biến thể so sánh và thí nghiệm
- **Biến thể:** B1 (FOLLOW 4.7.2 gốc, baseline chính), B1c (B1 + τ₀ hằng số hiệu chuẩn, vd nửa RTT, không đồng bộ đồng hồ), P1 (bù theo send-time GPS, tuyến tính), P2 (có gia tốc + ω, giới hạn tầm dự báo, mô hình kiểu CTRV), Oracle (không trễ, cận dưới). B0 (follower gốc DroneEngage) chỉ nhắc lại từ gate pilot.
- **Thí nghiệm:** E0 nền tảng (~20 run), E1 gate (B1, τ₀ ∈ {0; 0,5; 1} s, thẳng + tròn, 60 run), E2 quét τ₀ ∈ {0..2 s}, T ∈ {0,1; 0,5; 1} s, ω, d (~500), E3 5 biến thể × 4 kiểu quỹ đạo gồm ca "sai mô hình" (đổi ω, thẳng sang cua, dừng gấp, đảo chiều) (~600), E4 lệch đồng hồ δ ∈ {0; ±50; ±200} ms (~100), E5 bay thật.
- **Chỉ số/thống kê:** sai số cùng thời điểm và cross-track, RMSE/p95/max, tách dọc/ngang; sai số ước lượng firmware (log `FOLL`); ΔE = E(τ) − E(0); overshoot khi leader dừng; khoảng cách nhỏ nhất. Đơn vị thống kê là **lần chạy** (không phải follower), ≥ 10 lần/điều kiện, so sánh theo cặp, CI 95%, Wilcoxon signed-rank.
- **Lưu ý SITL:** leader dùng `CIRCLE` cần chỉnh `CIRCLE_OPTIONS` sang "face direction of travel"; FOLLOW dùng yaw leader còn DroneEngage dùng course. Kết quả phụ thuộc slot nằm trong hay ngoài vòng quay: **báo cả hai phía**.

---

## 3. Các quyết định đã chốt (và lý do)

| Ngày | Quyết định | Lý do |
|---|---|---|
| 2026-11-07 | Hướng **điều khiển** (topic brief v2), mạng chỉ là nhiễu/đầu vào trễ | Nhóm yếu về mạng, muốn phát triển thành đồ án tự động hóa, không muốn GVHD mảng mạng |
| 2026-11-07 | Dùng DroneEngage làm testbed/baseline trên ArduPilot SITL gốc; chỉ làm ở tầng companion/MAVLink (+Lua), **không fork firmware** | Tái lập được, giảm rủi ro, kết quả có ích cho cộng đồng ArduPilot |
| 2026-11-09 | Mọi tài liệu gửi người đọc phải đóng khung là bài điều khiển | Phản hồi của Ferb |
| 2026-11-10 | Cắt P3 (MPC/consensus) khỏi NCKH, để cho đồ án | Không đủ thời gian, làm loãng câu chuyện |
| 2026-11-10 | Baseline phải là FOLLOW **4.7.x** (không phải 4.6.3) | 4.7 đã có ω × r, dự báo; 4.6.3 thiếu |
| 2026-11-11 | Reframe **benchmark-first** | GPT round 2 + kiểm chứng: mọi thành phần bù trễ đều có prior art |
| 2026-11-16 | Nhắm **IEEE SMC 2027** với hệ thống hoàn chỉnh; tên đề tài phương án A (platform + benchmark); đề xuất **v3** thay v2 | Program Chair (phía VN) là PGS.TS. Vũ Văn Phong, người đứng đầu danh sách GVHD đề xuất |

---

## 4. Những gì đã làm và kết quả

### 4.1 Kiểm chứng DroneEngage (2026-11-07): **GO có điều kiện**
- Swarm leader-follower của DroneEngage chạy end-to-end với 3 ArduPilot SITL qua server tự host (AirGap, auth `account_storage_type: "single"`).
- Bản tin trạng thái leader **6503 (SWARM_MAVLINK)**, ~**1 Hz**, **không timestamp, không sequence**, follower không bỏ bản tin cũ, không timeout khi mất leader. Đây chính là khoảng trống.
- Chặng qua server là WebSocket/TLS trên TCP → mất gói thành **trễ head-of-line**, không mất bản tin.
- Follower gốc (B0) tụt sau **33-44 m** khi leader bay ~4,8 m/s, do chỉ gửi waypoint, **không có velocity feed-forward**.
- Formation: Thread, Arrow, Arrow_Dynamic có cài; Vector chỉ có trong enum.
- Container cloud không có `netem` → dùng **proxy trễ userspace** (`delayproxy.py`).
- SHA đã pin (droneengage_mavlink `be5d8a5`, communication `206b5a2`, server `a7283f8` v5.6.2, authenticator `40b25f4`, common `5fb11c2`, webclient_react `0171010`): xem báo cáo. ArduPilot khi đó: `Copter-4.6.3`.

### 4.2 Gate pilot SITL (2026-11-09): **GO**
Testbed: 3 SITL Copter 4.6.3 + DroneEngage tự host, đội hình mũi tên d = 15 m, leader 2 đoạn thẳng ~4,8 m/s, trễ đầu-cuối 1,0 s (đo lại: tuổi bản tin TB 1003 ms). n = 6 (3 lần × 2 follower).

| Follower | Trễ 0 | Trễ 1 s |
|---|---|---|
| B0 (gốc DroneEngage) | 38,45 ± 2,33 m | 48,9 ± 21,1 m |
| P1R (dự báo theo lúc nhận, giống FOLLOW) | 1,70 ± 0,12 m | 6,53 ± 0,20 m |
| P1 (dự báo theo lúc gửi) | 1,61 ± 0,05 m | 1,73 ± 0,14 m |

**Hạn chế (quan trọng):** chỉ bay thẳng vận tốc đều nên kết quả **tầm thường** (ΔE ≈ v·τ); cùng đồng hồ, không jitter/mất gói, FOLLOW 4.6.3. Đóng góp thật phải đến từ: **đoạn rẽ/tăng tốc, jitter, lấy mẫu thưa, lệch đồng hồ, observer P2**.

Mẹo testbed: auth agent giới hạn 10 lần/phút (đã nâng lên 1000 cục bộ); server cần env `de_public_port` (kiểu int) khi chạy qua proxy, không phải `de_comm_server_port`. Patch ở `nckh_research/de_sitl_tools/gate_2026-11-09/`.

### 4.3 Mô phỏng động học (2026-11-10)
Slot ngoài cua, ω = 0,2 rad/s, d = 15 m, τ = 1 s, T = 1 s: dự báo theo lúc nhận 8,62 m, tuyến tính theo lúc gửi 3,81 m, có ω 0,00 m. Slot trong cua đảo thứ tự (3,38 vs 3,70 m). Sai số tỷ lệ gần tuyến tính theo d.

### 4.4 Đánh giá phản biện (2026-11-10)
P1 trên đoạn thẳng là prior art (CACC: Xing, Ploeg & Nijmeijer, IEEE TVT 2020; Kilic & Meiboom AIAA 2015). Lỗi 38 m của B0 là lỗi cài đặt, không phải đóng góp khoa học. Lõi có giá trị: mô hình sai số khi **quay** (cánh tay đòn `d·ω·h`) + kiểm chứng trên autopilot gốc.

---

## 5. Trạng thái hiện tại và bước tiếp theo (tính đến 2026-11-18)

### 5.1 Lịch (đề xuất v3)
| Giai đoạn | Thời gian | Nội dung | Mốc |
|---|---|---|---|
| P0 | 16/11 - 29/11/2026 | Thống nhất đề tài với GVHD; xin xác nhận giấy phép DroneEngage bằng văn bản; chuyển testbed sang Copter 4.7.2 | Đề cương được duyệt |
| **S1** (nền tảng SITL) | **30/11 - 20/12/2026** | Node đội hình DroneEngage → `FOLLOW_TARGET`, timestamp phía leader, khối chèn trễ, logging, script tự động, harness pymavlink; chạy E0 | **G1 (20/12)**: H1 đạt |
| P1 | 21/12/2026 - 03/01/2027 | Chạy E1 | **G2 (03/01)**: H2 đạt → làm bộ bù; không đạt → bài tập trung nền tảng + benchmark + mô hình |
| P2 | 04/01 - 31/01/2027 | E2, đối chiếu mô hình | **G3 (31/01)**: khớp ±30% |
| P3 | 01/02 - 07/03/2027 | Cài P1/P2 (nếu qua G2), E3, E4 | Đủ số liệu |
| P4 | 08/03 - 24/03/2027 | Thống kê, hình, bản thảo | **G4 (24/03)**: bản thảo hoàn chỉnh |
| P5 | 25/03 - 08/04/2027 | GVHD góp ý, sửa, **nộp SMC 2027 (08/04/2027)** | |
| **S1b** (bay thật, song song) | 12/2026 - 03/2027 | Bench test RPi4 + DroneEngage với FC thật trên drone 5"; lắp FC cho drone 7"; thủ tục bay; E5 | Mục tiêu bay 2 drone trước **2027-03-15**, không bắt buộc cho bài |
| Đồ án | Sau NCKH | ≥ 3 drone thật, MPC/consensus, bản tạp chí (vd MDPI Drones) | |

(Bản kế hoạch chi tiết cũ 2026-11-10 có các gate khác ngày, vd G2 2026-12-06; **lịch của đề xuất v3 là bản hiện hành**.)

### 5.2 Việc đang chờ (chưa xong)
1. **Gửi email + đề xuất v3 cho GVHD PGS.TS. Vũ Văn Phong** (HCMUTE, ACIS Lab). **Chưa gửi.** Trong file docx còn ô trống phải điền: tên/thông tin 3 thành viên, học hàm học vị GV, email, SĐT.
2. **Xác nhận giấy phép DroneEngage bằng văn bản** từ tác giả Mohammad Hefny (lưu bản sao vào `docs/license/`); toàn bộ repo hiện tại phát hành dưới giấy phép **MIT** (`LICENSE` tại gốc).
3. Chuyển testbed SITL sang **Copter 4.7.2** và để nhóm tự chạy lại testbed trên máy của mình (mọi thứ trước đây chạy trong container cloud).
4. Thời điểm có **FC cho Drone 2**.
5. Chọn kết nối cho bay thật: **AirGap (LAN) hay cellular/4G**.
6. Câu hỏi quy chế: thành viên HCMUS có được tham gia NCKH sinh viên HCMUTE không (chưa xác minh).
7. Pháp lý bay: Nghị định 288/2025/NĐ-CP (hiệu lực 2025-11-05), drone ≥ 0,25 kg phải đăng ký, có thể cần chứng chỉ người điều khiển. Rủi ro lớn nhất cho bay thật (mới ở mức tin tức, chưa đọc văn bản gốc).

### 5.3 Bước kỹ thuật kế tiếp
Bắt đầu S1: viết node đội hình Python/pymavlink trên companion follower (nhận 6503 từ DroneEngage → gửi `FOLLOW_TARGET` có vận tốc, gia tốc, rates xuống FC), gắn timestamp GPS phía leader, khối chèn trễ/lấy mẫu, ghi 4 luồng log (trạng thái thật leader, bản tin đã gửi, ước lượng firmware `FOLL`, vị trí thật follower), script chạy hàng loạt. Tiêu chí G1: H1.

---

## 6. Nhóm, giảng viên, phần cứng

### 6.1 Nhóm (3 sinh viên năm 3)
| Thành viên | Nền tảng | Nhiệm vụ chính |
|---|---|---|
| **Ferb** (trưởng nhóm, chủ repo `Ferb27`) | Hệ thống nhúng / IoT, drone; biết C, Python; chủ yếu "vibe code"; định hướng chuyên ngành tự động hóa | Tích hợp DroneEngage + ArduPilot, node đội hình trên RPi4, khối chèn trễ, đồng bộ thời gian GPS, phần cứng drone, bay thử |
| Thành viên 2 | Tự động hóa / robotics + cơ khí; yếu phần mềm | Mô hình giải tích, nhận dạng độ trễ bám của position controller, thiết kế P2, cơ khí, lắp drone 7" |
| Thành viên 3 | CNTT (HCMUS), phần mềm thuần, AI | Tự động hóa thí nghiệm, đóng gói nền tảng tái lập, dữ liệu, thống kê, biểu đồ, kiểm thử |

Trường chủ quản đề tài: Khoa Điện - Điện tử, ĐH Công nghệ Kỹ thuật TP.HCM (HCMUTE).

### 6.2 Giảng viên hướng dẫn (đề xuất, chưa liên hệ)
- Chính: **PGS.TS. Vũ Văn Phong** (ACIS Lab; swarm mobile robot, observer; Program Chair phía VN của IEEE SMC 2027).
- UAV hằng ngày (tùy chọn): ThS. Nguyễn Anh Vũ. Dự phòng: PGS.TS. Lê Mỹ Hà, PGS.TS. Trần Đức Thiện.
- Nguồn danh sách: `references/Hướng nghiên cứu của Giảng viên Khoa Điện - Điện tử (cập nhật tháng 06_2026).xlsx`.

### 6.3 Phần cứng (chi tiết: `hardware_config.md`)
| | Trạng thái | Cấu hình chính |
|---|---|---|
| Drone 1 (5") | Có, sẵn sàng nhất | FC **MicoAir H743v2** (STM32H743), ESC Bluejay 45A, motor MT2204 2300kv, 3S 2200 mAh, GPS BZ251, RX ELRS RP1, companion **Raspberry Pi 4** (DroneEngage). Biên lực đẩy với RPi4 chưa kiểm |
| Drone 2 (7") | **Thiếu FC** | Khung F330, ESC Bluejay 60A, motor X2216 1100kv, 3S, GPS BZ251, RX ELRS Nano, Raspberry Pi 3+ |
| Drone 3 | Chưa lắp | |
| Tay điều khiển | Có | RadioMaster Pocket (ELRS) |

Firmware chính thức Copter 4.7.2 cho MicoAir743v2 được báo là có FOLLOW + Lua scripting (theo features.txt, chưa tự kiểm khi nạp).

---

## 7. Quy tắc làm việc (AI phải tuân thủ)

1. **Phân vai:** Claude là "bộ não" (lập kế hoạch, viết prompt, review, đối chiếu tiêu chí). **Antigravity (Google)** là "chân tay" thực thi code/file/triển khai; GPT dùng cho deep research web. Claude **không tự làm task code** trừ khi Ferb nói rõ "bạn làm luôn" / "bạn làm đi" (đã có ngoại lệ: kiểm chứng DroneEngage 2026-11-07, gate pilot 2026-11-09).
   - Quy trình chuẩn: Ferb chọn một hạng mục → Claude viết prompt đầy đủ ngữ cảnh cho Antigravity → Antigravity báo cáo → Claude review với tiêu chí chấp nhận.
2. **Trích dẫn từ GPT/AI khác coi là chưa kiểm chứng** cho tới khi mở được nguồn gốc. Ghi rõ mức đọc (tóm tắt / toàn văn / đọc code / đã chạy). Không suy ra "novelty" từ việc tìm không thấy.
3. **Đóng khung điều khiển**, mạng chỉ là đầu vào trễ.
4. **Mọi output nghiên cứu phải lưu cả trên máy Ferb** tại `D:\Project\clone_drone_engage\nckh_research\` (giữ cấu trúc, cập nhật `nckh_research/README.md`), không chỉ trên cloud. Không commit thư mục này vào git trừ khi được yêu cầu.
5. **Không ghi secret** (token, mật khẩu, `.env`, cookie) vào bất kỳ tài liệu nào.
6. Dùng ngày tuyệt đối (vd 2026-11-18). Giao tiếp: tiếng Việt, gọi người dùng là "bạn", tự xưng "tôi", góp ý thẳng khi đi sai hướng, hỏi lại khi yêu cầu mơ hồ.
7. Khi chạy thí nghiệm: **pin commit SHA** của DroneEngage và tag ArduPilot; giữ cùng quy ước timestamp giữa các biến thể.

---

## 8. Bản đồ thư mục `D:\Project\clone_drone_engage`

```text
clone_drone_engage/
├── PROJECT_CONTEXT.md          ← file này
├── README.md                   ← mục lục ngắn của workspace
├── hardware_config.md          ← cấu hình 2 drone + tay điều khiển
├── airgap_setup_guide.md       ← dựng DroneEngage chế độ AirGap (LAN) trên RPi4 (06/2026)
├── docs/PROJECT_SURVEY.md      ← khảo sát hiện trạng do Antigravity viết 2026-11-07 (mô tả cả GCS clone cũ)
├── nckh_research/              ← **TÀI LIỆU NGHIÊN CỨU CHÍNH** (bản sao của research/ trên Claude), có README.md chỉ mục
│   ├── advisor/                ← đề xuất đề tài .docx (v1 11-09, v2 11-11, **v3 11-16 = hiện hành**)
│   ├── de_sitl_tools/          ← script SITL + proxy trễ + dữ liệu kiểm chứng; gate_2026-11-09/ (CSV, patch, summary)
│   ├── plan_2026-11-10/        ← brief + báo cáo GPT vòng 1/2, search log, kinematic check
│   └── *.md                    ← các báo cáo (xem mục 9)
├── references/                 ← xlsx hướng nghiên cứu GV Khoa Điện - Điện tử, sơ đồ tìm tài liệu
├── _personal/                  ← CV, ghi chú nghề nghiệp (cá nhân, không liên quan đề tài)
└── _archive/gcs_clone_2026-06/ ← GCS clone React/TS cũ (06/2026), **ĐÃ BỎ, không dùng nữa**
```

Git: remote `github.com/Ferb27/clone_drone_engage`, nhánh `main`. Việc dọn thư mục ngày 2026-11-18 **chưa commit** (git sẽ thấy file bị xóa + untracked). Repo này **không chứa code nghiên cứu chính**; code nghiên cứu dựa trên DroneEngage upstream (`github.com/DroneEngage`) và ArduPilot (`github.com/ArduPilot/ardupilot`). Trên Claude, cùng tài liệu nằm ở `/mnt/project-files/research/`.

> Về `_archive/`: giai đoạn 06/2026 nhóm từng viết một GCS web (React 18 + TS + Zustand + Leaflet) clone DroneEngage webclient, có phản hồi từ một CTO về trễ 4G ~1 s và yêu cầu chuyển sang C++. Hướng đó đã dừng; đề tài hiện tại **không** dựa vào code này.

---

## 9. Tài liệu quan trọng (đọc theo thứ tự này)

| # | File (trong `nckh_research/`) | Vì sao đọc |
|---|---|---|
| 1 | `advisor/De_xuat_de_tai_NCKH_2026-11-16.docx` | **Bản đề xuất hiện hành (v3)**: tóm tắt, mô hình, RQ, thí nghiệm, lịch, phân công, rủi ro, tài liệu tham khảo |
| 2 | `droneengage_swarm_verification_2026-11-07.md` | Cách DroneEngage swarm hoạt động (file:line), SHA đã pin, cách dựng testbed |
| 3 | `gate_pilot_2026-11-09.md` + `de_sitl_tools/gate_2026-11-09/` | Kết quả SITL đầu tiên, script, patch |
| 4 | `critical_assessment_2026-11-10.md` | Phản biện: chỗ nào không mới, lõi học thuật nằm đâu |
| 5 | `plan_2026-11-10/gpt_report_round2_2026-11-11.md` | Prior art và lý do reframe benchmark-first |
| 6 | `literature_review_2026-11-07.md`, `similar_papers_benchmark_2026-11-09.md` | Tổng quan tài liệu, 13 bài tương tự; template tham khảo: Liang et al., IEEE RA-L 2025 (arXiv 2503.09148) |
| 7 | `topic_brief_v2_control_2026-11-07.md`, `feasibility_control_2026-11-07.md` | Lịch sử chuyển sang hướng điều khiển |
| 8 | `research_tooling_2026-11-07.md` | Công cụ hỗ trợ nghiên cứu |
| - | Kế hoạch chi tiết WP0-WP7 (Claude Doc, 2026-11-10): https://claude.ai/artifact/Hoppvn7qLRHFh7ChraFCPG | Kế hoạch cũ hơn v3, chưa xuất bản sao cục bộ |

Các công trình gần nhất cần trích và phân biệt: Aggarwal et al. NetGames 2004 (dead reckoning có timestamp); Kilic & Meiboom AIAA 2015 (đội hình, timestamp GPS, bay thẳng); Xing, Ploeg & Nijmeijer IEEE TVT 2020 (Smith predictor trong CACC); Bayuwindra et al. **IEEE TCST** 2020 (không phải T-ITS); Tripathi et al. WiSwarm INFOCOM 2023; Vera-Amaro et al. Drones 10(4):310, 2026 (LoRa, đối thủ gần nhất, không bù trễ); Ambroziak & Ciężkowski Sensors 2021; Wang et al. Machines 2021; ArduPilot Discourse thread 99433; PX4 Follow Target.

---

## 10. Thuật ngữ nhanh

| Ký hiệu | Nghĩa |
|---|---|
| τ, τ₀ | Trễ truyền leader → follower; τ₀ là trễ nền cố định |
| T | Chu kỳ lấy mẫu/gửi bản tin leader |
| h | Tuổi dữ liệu, h ≈ τ + T/2 |
| d, r | Khoảng cách / vector offset đội hình |
| ω, α | Tốc độ quay, gia tốc góc của leader |
| δ | Sai lệch đồng hồ giữa hai máy |
| 6503 | Bản tin SWARM_MAVLINK của DroneEngage (trạng thái leader, ~1 Hz) |
| `FOLLOW_TARGET` (#144) | Bản tin MAVLink đưa mục tiêu vào chế độ FOLLOW |
| SITL | Software-in-the-loop của ArduPilot |
| AirGap | Chế độ DroneEngage tự host trong LAN, không qua cloud |
| G1..G4 | Các mốc go/no-go trong lịch |
