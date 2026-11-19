#!/bin/bash
# R1 live interop: Rust de_common module (examples/hello_module) <-> real C++ de_comm.
# No SITL needed. Checks: registration (ID reply with party id), module -> server
# (1008 reaches the comm server through de_comm), server -> module (GCS 1005 reaches it).
#
# usage (WSL): interop_common.sh OUT_DIR
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
OUT=${1:?usage: interop_common.sh OUT_DIR}; mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
DE_COMM_BIN=${DE_COMM_BIN:-/tmp/de_up/comm/bin/de_comm}
SRV_DIR=${SRV_DIR:-/tmp/rv_rust/droneengage_server}
AUTH_DIR=${AUTH_DIR:-/tmp/rv_rust/droneengage_authenticator}
PY=${PY:-/tmp/rv_gate/venv/bin/python}
HELLO=${HELLO:-$ROOT/target/debug/examples/hello_module}

stop_all() { pkill -f "de_comm -c|hello_module|wss_tap.py|andruavmainserver|andruavauth" 2>/dev/null; sleep 1; }
stop_all
cd "$AUTH_DIR" && (setsid nohup ./target/release/andruavauth --config=server.config > "$OUT/auth.log" 2>&1 &)
sleep 3
cd "$SRV_DIR" && (de_public_port=9967 setsid nohup ./target/release/andruavmainserver --config=server.config > "$OUT/srv.log" 2>&1 &)
sleep 3
(setsid nohup "$PY" -I "$ROOT/tools/trace/wss_tap.py" 9967 127.0.0.1 9966 \
   "$SRV_DIR/ssl_local/ssl_airgap/domain.crt" "$SRV_DIR/ssl_local/ssl_airgap/domain.key" "$OUT/wss.jsonl" > "$OUT/tap.log" 2>&1 &)
sleep 1
mkdir -p "$OUT/u1"; cp "$HERE/units/u1/de_comm.json" "$OUT/u1/"
cd "$OUT/u1" && (setsid nohup "$DE_COMM_BIN" -c ./de_comm.json > comm.log 2>&1 &)
sleep 5
(setsid nohup "$HELLO" 127.0.0.1 60010 61099 HELLO_RS > "$OUT/hello.log" 2>&1 &)
sleep 8
# GCS asks everyone for their ID: de_comm forwards the 1005 to modules that subscribed to it.
cd "$HERE" && "$PY" -I -c "
import sys, time; sys.path.insert(0, '$HERE')
import gcs
g = gcs.GCS('single@airgap.droneengage.com', 'test', pid='gcs_interop')
time.sleep(1); g.req_id(); time.sleep(4)
print('units seen by GCS:', {k: v.get('UD') for k, v in g.units.items()})
" > "$OUT/gcs.log" 2>&1
sleep 3
stop_all

echo "== hello.log"; grep -E "started|RX|TX" "$OUT/hello.log" | head -12
echo "== checks"
party=$(grep -o '"sd":"[0-9]*"' "$OUT/hello.log" | head -1)
echo "registered (ID reply with party): ${party:-NO}"
echo "1005 from GCS received: $(grep -cE '"mt": ?1005' "$OUT/hello.log")"
echo "1008 from module at server: $(grep -c 'hello from rust module' "$OUT/wss.jsonl")"
grep -a -i "HELLO_RS\|module.*gen" "$OUT/u1/comm.log" | head -5
