#!/bin/bash
# Run one gate scenario (tau = 0) on 3 ArduCopter SITL + DroneEngage and record traffic.
#
# Records:
#   lo.pcap               all loopback traffic on the databus UDP ports and the FC TCP ports
#   wss.jsonl             plaintext WebSocket frames (unit/GCS <-> comm server), via wss_tap.py
#   gate.csv / gate.ana   leader/follower positions and formation error (same as the old gate)
#   *.databus.jsonl, *.fcb.jsonl   converted from lo.pcap by pcap2jsonl.py
#
# Usage (inside WSL): capture.sh OUT_DIR
# Override binaries with env vars, e.g. DE_COMM_BIN=... DE_MAV_BIN=... (Rust builds later).
#
# Needs: sudo for tcpdump, the Rust server/auth builds, a python venv with pymavlink,
# websocket-client, websockets>=13, dpkt. See tools/sitl_gate/README.md.
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
TRACE=$HERE/../trace
OUT=${1:?usage: capture.sh OUT_DIR}
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)

DE_COMM_BIN=${DE_COMM_BIN:-/tmp/de_up/comm/bin/de_comm}
DE_MAV_BIN=${DE_MAV_BIN:-/tmp/de_up/mav/bin/de_ardupilot}
SITL_BIN=${SITL_BIN:-/tmp/sitl/arducopter}
SRV_DIR=${SRV_DIR:-/tmp/rv_rust/droneengage_server}
AUTH_DIR=${AUTH_DIR:-/tmp/rv_rust/droneengage_authenticator}
PY=${PY:-/tmp/rv_gate/venv/bin/python}
FLY_ACC=${FLY_ACC:-single@airgap.droneengage.com}   # upstream airgap default test account
FLY_AC=${FLY_AC:-test}

stop_all() {
  pkill -f "de_ardupilot|de_comm|arducopter|wss_tap.py|delayproxy.py|andruavmainserver|andruavauth" 2>/dev/null
  sudo -n pkill -f "tcpdump -i lo" 2>/dev/null
  sleep 2
}
stop_all
for b in "$DE_COMM_BIN" "$DE_MAV_BIN" "$SITL_BIN" "$SRV_DIR/target/release/andruavmainserver" "$AUTH_DIR/target/release/andruavauth" "$PY"; do
  [ -x "$b" ] || { echo "missing: $b"; exit 1; }
done
echo "comm=$DE_COMM_BIN mav=$DE_MAV_BIN" > "$OUT/binaries.txt"
sha256sum "$DE_COMM_BIN" "$DE_MAV_BIN" >> "$OUT/binaries.txt"

# 1. auth + comm server (server announces public port 9967 = the tap)
cd "$AUTH_DIR" && (setsid nohup ./target/release/andruavauth --config=server.config > "$OUT/auth.log" 2>&1 &)
sleep 3
cd "$SRV_DIR" && (de_public_port=9967 setsid nohup ./target/release/andruavmainserver --config=server.config > "$OUT/srv.log" 2>&1 &)
sleep 3

# 2. TLS-terminating tap in place of the delay proxy
(setsid nohup "$PY" -I "$TRACE/wss_tap.py" 9967 127.0.0.1 9966 \
   "$SRV_DIR/ssl_local/ssl_airgap/domain.crt" "$SRV_DIR/ssl_local/ssl_airgap/domain.key" \
   "$OUT/wss.jsonl" > "$OUT/tap.log" 2>&1 &)
sleep 1

# 3. packet capture: databus UDP 60000-61999, FC TCP 5760/5770/5780
sudo -n true || { echo "need passwordless sudo for tcpdump"; exit 1; }
(sudo -n setsid nohup tcpdump -i lo -s 0 -U -w "$OUT/lo.pcap" \
   "udp portrange 60000-61999 or tcp port 5760 or tcp port 5770 or tcp port 5780" > "$OUT/tcpdump.log" 2>&1 &)
sleep 2

# 4. three units: de_comm first
for i in 1 2 3; do
  W=$OUT/u$i; mkdir -p "$W"; cp "$HERE/units/u$i/de_comm.json" "$HERE/units/u$i/de_mavlink.json" "$W/"
  cd "$W" && (setsid nohup "$DE_COMM_BIN" -c ./de_comm.json > comm.log 2>&1 &); sleep 1
done
sleep 6
echo "COMM_CONNECTED=$(grep -a -h 'Communication Server Connected' "$OUT"/u*/comm.log | wc -l)"

# 5. SITL x3 (ArduCopter 4.7.2 param names) + de_mavlink x3
for i in 0 1 2; do
  d=$OUT/sitl$i; mkdir -p "$d"; LON=$(python3 -c "print(149.165230+$i*0.00012)")
  printf "MAV_SYSID %d\nFRAME_CLASS 1\nFRAME_TYPE 0\nWP_SPD 5\n" $((i+1)) > "$d/id.parm"
  for s in "" 2; do for a in X Y Z; do
    printf "INS_ACC%sOFFS_%s 0.001\nINS_ACC%sSCAL_%s 1.001\n" "$s" $a "$s" $a >> "$d/id.parm"; done; done
  cd "$d" && rm -f eeprom.bin
  (timeout 3600 "$SITL_BIN" --model + --speedup 1 -I$i --home -35.363261,$LON,584,0 --defaults "$d/id.parm" > sitl.log 2>&1 &)
done
sleep 3
for i in 1 2 3; do cd "$OUT/u$i" && (setsid nohup "$DE_MAV_BIN" -c ./de_mavlink.json -b ./de_mavlink.local > mav.log 2>&1 &); done
sleep 45

# 6. scenario: arm, takeoff, Arrow formation, leader 250 m N then 250 m E
ACC=$FLY_ACC AC=$FLY_AC "$PY" -I "$HERE/exp1.py" "$OUT/gate.csv" > "$OUT/exp.log" 2>&1
python3 -I "$HERE/analyze.py" "$OUT/gate.csv" 0 1 > "$OUT/gate.ana" 2>&1

# 7. stop and convert
stop_all
"$PY" -I "$TRACE/pcap2jsonl.py" "$OUT/lo.pcap" "$OUT/trace"
ls -la "$OUT"; tail -8 "$OUT/gate.ana"; wc -l "$OUT"/*.jsonl
