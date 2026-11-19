# Prompt cho Antigravity #1: tạo repo private `droneengage-rs`

> Ferb: copy toàn bộ phần dưới đường kẻ vào Antigravity. Khi Antigravity báo xong, gửi báo cáo lại cho Claude để review theo mục "Tiêu chí chấp nhận".

---

## Bối cảnh

Nhóm mình đang port các module C++ chạy trên companion computer của DroneEngage (`de_common`, `de_comm`, `de_mavlink`) sang Rust. Tác giả DroneEngage đã đồng ý cho làm việc này. Kế hoạch đầy đủ nằm ở `D:\Project\port_drone_rust\docs\PORT_PLAN_2026-11-19.md`; đọc mục 7 trước khi làm.

Nhiệm vụ lần này **chỉ là dựng khung repo và đẩy lên GitHub ở chế độ private**. Không viết code port, không chép code upstream.

## Việc cần làm

1. **Tạo thư mục repo** `D:\Project\port_drone_rust\droneengage-rs\`, rồi chạy `git init -b main` trong thư mục đó.
   - **Không** chạy `git init` ở `D:\Project\port_drone_rust\`.
   - **Không** di chuyển, sửa hay xoá bất kỳ thứ gì đang có trong `D:\Project\port_drone_rust\` (gồm `de_fork/`, `de_upstream_oct_2026/`, `.upstream_orig/`, `_archive/`).

2. **Tạo khung đúng như sau:**

   ```text
   droneengage-rs/
   ├── Cargo.toml
   ├── .gitignore
   ├── README.md
   ├── AGENTS.md
   ├── crates/
   │   ├── de_common/   (Cargo.toml, src/lib.rs)
   │   ├── de_mavlink/  (Cargo.toml, src/main.rs)
   │   └── de_comm/     (Cargo.toml, src/main.rs)
   ├── tools/.gitkeep
   ├── tests/fixtures/.gitkeep
   ├── deploy/.gitkeep
   └── docs/
       ├── PORT_PLAN_2026-11-19.md   (copy từ D:\Project\port_drone_rust\docs\)
       ├── UPSTREAM_SYNC.md
       └── license/.gitkeep
   ```

   Nội dung từng file:
   - `Cargo.toml` (workspace):
     ```toml
     [workspace]
     resolver = "2"
     members = ["crates/de_common", "crates/de_mavlink", "crates/de_comm"]

     [workspace.package]
     edition = "2021"
     rust-version = "1.88"
     publish = false
     ```
   - Mỗi crate có `Cargo.toml` dùng `edition.workspace = true`, `rust-version.workspace = true`, `publish.workspace = true`, version `0.1.0`, **chưa có dependency nào**.
     - `de_common` là lib, `src/lib.rs` để trống, chỉ có một dòng doc comment.
     - `de_mavlink` là bin, khai báo thêm `[[bin]] name = "de_ardupilot"`. `de_comm` là bin.
     - `main.rs` của hai bin chỉ in tên binary và version (`env!("CARGO_PKG_VERSION")`).
   - `.gitignore`: `/target`, `**/*.rs.bk`, `.env`, `*.pem`, `*.key`, `*.crt`, `*.log`, `.vscode/`, `.idea/`.
   - `README.md`: 5–10 dòng, gồm repo là gì, trạng thái "đang port, private", link tới `docs/PORT_PLAN_2026-11-19.md`.
   - `AGENTS.md`: chép nguyên khối này:
     ```markdown
     # AGENTS.md

     Quy ước cho mọi AI agent (Claude, Antigravity) làm việc trong repo này.

     - Ngôn ngữ: tài liệu tiếng Việt, code và comment trong code bằng tiếng Anh.
     - Mục tiêu tối thượng: tương thích giao thức với bản C++ upstream (JSON message, databus UDP/unix socket, file config). Khi phân vân, hành vi của bản C++ là đúng.
     - Mỗi file port từ C++ ghi ở đầu file nguồn gốc: repo upstream, đường dẫn file, commit SHA.
     - Không commit secret, key, cert, token, mật khẩu.
     - Chạy `cargo fmt` và `cargo clippy -- -D warnings` trước khi commit (khi đã có toolchain).
     - Không thêm dependency ngoài danh sách ở mục 4 của docs/PORT_PLAN_*.md khi chưa hỏi.
     - (Claude sẽ bổ sung quy ước chi tiết ở giai đoạn R0.)
     ```
   - `docs/UPSTREAM_SYNC.md`: có tiêu đề, một câu mô tả mục đích ("ghi lại các thay đổi lấy từ upstream: ngày, repo, commit, file, lý do"), và một bảng rỗng với các cột: Ngày | Repo upstream | Commit | Nội dung lấy về | Vào file nào.

3. **Không** tạo file `LICENSE`. Repo để private, license sẽ quyết sau.

4. **Kiểm tra build nếu được:** nếu WSL đã có `cargo`, chạy `cargo check` trong repo (đường dẫn WSL `/mnt/d/Project/port_drone_rust/droneengage-rs`). Nếu chưa có `cargo` thì **đừng cài**, chỉ ghi "chưa kiểm tra build" vào báo cáo.

5. **Commit đầu tiên:** `chore: initial workspace skeleton`.

6. **Tạo repo GitHub private và push:**
   - Dùng GitHub CLI: `gh repo create droneengage-rs --private --source . --remote origin --push`.
   - Nếu máy chưa có `gh`: cài bằng `winget install --id GitHub.cli`.
   - Nếu `gh` chưa đăng nhập: **dừng lại và nhờ Ferb tự chạy** `gh auth login`. Không bao giờ yêu cầu Ferb dán token hay mật khẩu vào chat, và không lưu token vào file nào.
   - Sau khi push xong, chạy `gh repo view droneengage-rs --json visibility,url` để xác nhận repo là `PRIVATE`.

## Tiêu chí chấp nhận (Claude sẽ review theo đúng danh sách này)

- [ ] `git status` sạch, có đúng 1 commit trên nhánh `main`.
- [ ] Cây thư mục đúng như mục 2, không có file thừa và không có code upstream.
- [ ] Không có `LICENSE`, không có file key/cert/secret nào.
- [ ] Repo GitHub có visibility = `PRIVATE`.
- [ ] Không có thay đổi nào ngoài `droneengage-rs/` trong `D:\Project\port_drone_rust\`.
- [ ] Ghi rõ kết quả `cargo check`, hoặc "chưa kiểm tra build".

## Báo cáo gửi lại

Gồm:
- URL repo và kết quả lệnh `gh repo view`.
- Output của `git log --oneline` và `git ls-files`.
- Kết quả `cargo check` (hoặc ghi "chưa kiểm tra build").
- Mọi chỗ đã làm khác với prompt này, kèm lý do.
