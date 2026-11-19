# AGENTS.md

Conventions for every AI agent (Claude, Antigravity) and person working in this repo.
Docs are in Vietnamese. Code, comments, commit messages and identifiers are in English.

## 1. What this repo is

A Rust port of the companion-side modules of DroneEngage (`de_common`, `de_comm`, `de_mavlink`).
The plan, decisions and schedule are in `docs/PORT_PLAN_2026-11-19.md`. Read sections 0, 4, 6 before starting any task.

**Top-level goal: protocol compatibility with the C++ upstream.** A Rust module must be able to replace the C++ module while the rest of the stack stays C++. When unsure, the C++ behaviour is correct, even if it looks odd.

## 2. Upstream reference

- C++ source to port from: `../de_upstream_oct_2026/droneengage_{common,communication,mavlink}`.
  Pinned commits: common `5fb11c2`, communication `206b5a2`, mavlink `be5d8a5`.
- Rust server/auth (already ported upstream, same stack we use): `../de_upstream_oct_2026/droneengage_{server,authenticator}`.
- Never edit files under `../de_upstream_oct_2026/`, `../de_fork/`, `../.upstream_orig/`. They are read-only references.

Every Rust file ported from C++ starts with a header comment naming its sources:

```rust
// Ported from droneengage_common@5fb11c2: de_databus/udpClient.cpp, udpClient.hpp
```

When the port intentionally differs from C++ (bug fix, simplification), write `// DIVERGENCE: <why>` at that spot and add a line to `docs/DIVERGENCES.md`.

Anything taken from a newer upstream commit gets a row in `docs/UPSTREAM_SYNC.md`.

## 3. Layout

```text
crates/de_common/   library: databus transport, chunk protocol, message envelope, message ids, config
crates/de_mavlink/  binary `de_ardupilot`: flight controller bridge, swarm
crates/de_comm/     binary `de_comm`: auth, WSS to server, module broker
tools/              helper scripts (trace capture, message table generator, SITL gate)
tests/fixtures/     traces recorded from the C++ stack, used as conformance tests
deploy/             systemd units, install script, image build
docs/               plan, divergences, upstream sync, license evidence
```

Module structure inside a crate mirrors the C++ folder structure where it makes sense, so a reader can find the counterpart file.

## 4. Dependencies

Allowed without asking (keep versions in the workspace `Cargo.toml` under `[workspace.dependencies]`):
`tokio`, `serde`, `serde_json` (feature `float_roundtrip`; **never** `preserve_order`, see section 5), `mavlink` 0.21 (see plan section 4 for features), `tokio-tungstenite`, `rustls`, `tokio-rustls`, `webpki-roots`, `reqwest` (rustls, no default features), `tracing`, `tracing-subscriber`, `tracing-appender`, `clap`, `anyhow`, `thiserror`, `bytes`, `rppal`.

Dev-only, allowed: `pretty_assertions`, `tempfile`, `flate2`, `base64` (reading fixtures).

Anything else: ask first and explain why. No OpenSSL, no C libraries.

## 5. Code rules

- Edition 2021, MSRV 1.88. `unsafe` is forbidden unless a comment explains why and the reviewer agrees.
- Libraries (`de_common`) return typed errors (`thiserror`). Binaries may use `anyhow`.
- No `unwrap()`/`expect()` on data coming from the network, the FC or config files. Malformed input is logged and dropped, never a panic. A drone process must not crash because of one bad packet.
- JSON: keep field names exactly as C++ sends them (short keys like `"a"`, `"b"`), use `serde(rename)` instead of renaming the wire format.
- JSON key order: the C++ modules use plain `nlohmann::json`, whose objects are sorted maps, so every object they emit has **sorted keys**. `serde_json` without `preserve_order` (a `BTreeMap`) gives exactly the same order. Build messages as `serde_json::Value` (`json!` macro) and serialize with `de_common::json::dump`. Do not serialize structs directly for wire messages: struct field order is declaration order, not sorted. (Verified on 7191 recorded messages, `crates/de_common/tests/fixture_databus.rs`.)
- Exception: de_comm (C++) splices `"p"` and `"sd"` into forwarded messages as text, so messages coming *from* de_comm are not sorted. Only matters when porting de_comm (R3).
- Message type numbers come from the generated table (`de_common::messages`), never typed as bare literals.
- Async: one `tokio` runtime per binary. Don't block inside async code (no `std::thread::sleep`, no blocking file or socket I/O).
- Logs: `tracing`. `info` for state changes, `debug` for per-message events. Never log credentials or access codes.

## 6. Tests

- Unit tests next to the code. Port the C++ tests from `droneengage_common/test/` when porting the matching file.
- Conformance tests replay traces from `tests/fixtures/` (format described in `tools/trace/README.md`).
- Before committing: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p <crate you changed>`.
- **WSL warning:** a full parallel `cargo test` has crashed WSL before (custom kernel). Always use `-j 4` and test one crate at a time.

## 7. Build

Native: run `cargo build -j 4` inside WSL (`/mnt/d/Project/port_drone_rust/droneengage-rs`).
Raspberry Pi (Pi 3B+, Pi 4, 64-bit OS): `cargo zigbuild -j 4 --release --target aarch64-unknown-linux-gnu.2.36`.

## 8. Git

- Branch `main`. Small commits, one topic each. Message style: `type(scope): summary`, e.g. `feat(de_common): udp chunk reassembly`.
- Author: `Ferb27 <Ferb27@users.noreply.github.com>`, set locally in this repo. Never commit under a real name.
- No co-author trailers in commit messages.
- Never commit secrets: keys, certs, `.env`, tokens, access codes, real account names. Test configs use placeholders.
- Large captures (`*.pcap`) stay outside the repo in `../traces/`. Only reduced fixtures go in `tests/fixtures/`.

## 9. Handing work back

When an agent finishes a task, its report must include: what changed (files), test commands run and their results, anything done differently from the task description and why, and open questions.
