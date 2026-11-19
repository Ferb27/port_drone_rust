# Traces: recording the C++ stack to use as conformance tests

Idea: run the C++ stack on SITL, record **all** traffic at three layers, then require the Rust modules to produce equivalent traffic in the same scenario.

| Layer | How it's recorded | File |
|---|---|---|
| Databus (module ↔ de_comm, UDP 60000–61999) | `tcpdump -i lo`, then `pcap2jsonl.py` | `trace.databus.jsonl` |
| FC (de_mavlink ↔ SITL, TCP 5760/5770/5780) | same pcap | `trace.fcb.jsonl` |
| WSS (de_comm/GCS ↔ comm server) | `wss_tap.py`: decrypts TLS with the rig's self-signed cert and logs plaintext frames | `wss.jsonl` |

Auth traffic (HTTPS :19408) isn't recorded yet. It will be added in R3 when porting de_comm.

## Line format

`*.databus.jsonl`, `*.fcb.jsonl`:

```json
{"proto":"udp","t":1763550000.123456,"src":"127.0.0.1:61013","dst":"127.0.0.1:60010","len":312,"data":"<base64>"}
```

`wss.jsonl`:

```json
{"t":1763550000.123456,"conn":1,"path":"/...","dir":"c2s","kind":"text","data":"{...}"}
```

`dir` is `c2s` (client → server), `s2c`, or `meta` (open/close/error). Binary frames are stored as base64.

## Tools

- `wss_tap.py LISTEN_PORT TARGET_HOST TARGET_PORT CERT KEY OUT.jsonl`: test rig only, never point it at real users.
- `pcap2jsonl.py IN.pcap OUT_PREFIX [PORT_MIN PORT_MAX] [FCB_PORTS]`

## Storage

- The full capture (`lo.pcap`, all logs) lives outside the repo, in `../traces/<date>_<scenario>/`.
- `tests/fixtures/` only holds compressed, reduced pieces (`.jsonl.gz`) that tests actually use.
