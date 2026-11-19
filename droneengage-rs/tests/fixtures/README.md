# Fixtures

Traffic recorded from the C++ stack, used as conformance tests for the Rust modules. Line format: `tools/trace/README.md`.

## `cpp_upstream_tau0/` (2026-11-19)

| Item | Value |
|---|---|
| Source | Upstream C++, unpatched: communication `206b5a2`, mavlink `be5d8a5`, common `5fb11c2` (built with `git archive` from `de_upstream_oct_2026`) |
| Server / auth | Upstream Rust `20900c0` / `9274556` + P3/P4 (`/tmp/rv_rust`) |
| FC | 3 × ArduCopter 4.7.2 SITL |
| Scenario | `tools/sitl_gate/exp1.py`: arm, takeoff 15 m, Arrow d = 15 m, leader 250 m N then 250 m E, τ = 0 |
| Gate result | Window 20–65 s: 39.90 m and 39.84 m. Matches the B0 baseline of 39.67 ± 1.33 m |
| Volume | databus 7492 datagrams, fcb 6432 TCP segments, wss 8539 frames |

Databus ports: de_comm `60010/60020/60030`, de_mavlink `61013/61023/61033`. FC: TCP `5760/5770/5780`.

The `f` session keys in `de_auth` frames have been replaced with `REDACTED_SESSION_KEY`.
The full capture (pcap, logs) is outside the repo at `../traces/2026-11-19_cpp_tau0/`.
