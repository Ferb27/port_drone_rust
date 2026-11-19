# Intentional differences from the C++ version

Every place where the Rust port deliberately behaves differently from C++ upstream (bug fix, simplification, security). Each one also has a `// DIVERGENCE:` comment in the code.

| Date | Crate / file | C++ behaviour | Rust behaviour | Why | Affects the wire? |
|---|---|---|---|---|---|
| 2026-11-19 | de_common / json (`serde_json`) | nlohmann (Grisu2) prints doubles, sometimes with 17 digits or the other tie-break digit | ryu prints the shortest round-trip form | Different float printing library | Number text differs in 3/7191 recorded messages (e.g. `...563` vs `...562`); **parsed value identical**. No effect on JSON receivers |
| 2026-11-19 | de_common / config.rs `save` | Backup `*.bak_YYYYmmdd_HHMMSS` in local time | Same format, UTC | Avoids a time-zone crate | No |
| 2026-11-19 | de_common / module.rs health loop | Process CPU % uses `sysconf(_SC_CLK_TCK)` | Assumes 100 | USER_HZ is 100 on every Linux we target; avoids libc | No (same value) |
| 2026-11-19 | de_common / module.rs `init` | `inet_addr` only accepts a numeric IPv4 | Also resolves host names (first IPv4) | Convenience | No for numeric IPs (all existing configs) |
| 2026-11-19 | de_mavlink / fcb.rs `tick` | 10 ms loop = `sleep(10 ms)` + work, so it drifts (~13 ms per tick measured) | `tokio::time::interval(10 ms)`, no drift | Intended rates | Periodic messages (9102, 6502, 1004, 6518) come ~20-30% more often, i.e. at the rate C++ intends |
| 2026-11-19 | de_mavlink / vehicle.rs | `GPS_RAW_INT` case falls through into `GPS2_RAW` and `HIGHRES_IMU` decoding (missing `break`) | Each message decoded into its own struct | C++ bug, garbage in unused state | No (GPS2 is only forwarded when a real GPS2_RAW arrives, in both) |
| 2026-11-19 | de_mavlink / fcb.rs mode change | Also notifies DE-Pilot (sends a 1008 "pilot operations disabled") | Not sent | DE-Pilot is R2b | Missing 1008 messages until R2b |
| 2026-11-19 | de_mavlink / fcb.rs ACK | Error text from `CMavlinkHelper::getACKError` | `cmd N result M` | Helper not ported (R2b) | 1008 text differs for rejected commands |
| 2026-11-19 | de_mavlink / fcb.rs UDP proxy | Opens a UDP socket and forwards MAVLink when the server enables it | Reports `en:false` | Forwarding is R2b | Smart telemetry over UDP not available yet |
| 2026-11-19 | de_mavlink / mav.rs UDP FCB link | `UDPPort(ip, port)` | `udpin:ip:port` | C++ semantics not checked yet | Only for `fcb_connection_uri.type = udp` (gate uses tcp) |
| 2026-11-19 | de_mavlink / fcb.rs `sendMissionCurrent` | MISSION_COUNT from the waypoint manager | All-zero MISSION_COUNT | Waypoint manager is R2b | 6502 content differs while no mission is loaded only in the count field |
