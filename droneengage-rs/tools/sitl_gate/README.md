# SITL gate: 3 ArduCopter + DroneEngage on WSL

Moved from the gate pilot harness (2026-11-09, re-run 2026-11-18). Runs in WSL Ubuntu 22.04.

| File | Role |
|---|---|
| `capture.sh OUT_DIR` | Runs the full stack plus one τ = 0 scenario, records traces (see `../trace/README.md`), computes the formation error |
| `exp1.py` | pymavlink: arm, takeoff 15 m, Arrow formation d = 15 m, leader flies 250 m North then 250 m East (~4.8 m/s), logs `GLOBAL_POSITION_INT` at 10 Hz |
| `gcs.py` | Minimal GCS: logs in to auth, connects WSS, sends swarm commands (1056, 1054) |
| `analyze.py CSV 0 1` | Steady-state error in the 20–65 s and 100–125 s windows |
| `delayproxy.py` | TCP proxy that adds delay (reads `LINK_CFG`), used when measuring τ > 0 |
| `units/u{1,2,3}/` | de_comm and de_mavlink configs for 3 units (upstream airgap default account `single@airgap.droneengage.com` / `test`) |

## Requirements

- `/tmp/sitl/arducopter`: static ArduCopter 4.7.2 binary.
- Rust server/auth built in `/tmp/rv_rust/` (upstream + P3/P4), with the fork's cert in `ssl_local/ssl_airgap/`.
- Python venv with `pymavlink websocket-client "websockets>=13" dpkt`.
- Passwordless `sudo` (for tcpdump).
- C++ binaries default to `/tmp/de_up/{comm,mav}/bin/`. Override with `DE_COMM_BIN`, `DE_MAV_BIN`.

## ArduCopter 4.7 gotchas

- `SYSID_THISMAV` → `MAV_SYSID`, `WPNAV_SPEED` → `WP_SPD` (m/s). Without them all SITL instances are sysid 1 and the leader flies ~9 m/s.
- Without `copter.parm` you must set `FRAME_CLASS 1`, `FRAME_TYPE 0`, `INS_ACC*OFFS/SCAL`, otherwise it won't arm.
- Start SITL with `(timeout 3600 arducopter ... &)`. Start servers with `setsid`, or they die with SIGHUP when `wsl.exe` exits.
- From Windows, run through `wsl -e bash -lc '...'`.

## Reference result

Upstream B0 baseline at τ = 0: **39.67 ± 1.33 m** (2026-11-18, Rust server + P3/P4).
