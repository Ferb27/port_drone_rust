#!/usr/bin/env python3
"""Convert a loopback pcap from the SITL rig into JSONL trace files.

Writes, next to OUT_PREFIX:
  <prefix>.databus.jsonl  one line per UDP datagram on the module databus
  <prefix>.fcb.jsonl      one line per TCP segment payload between de_mavlink and SITL
Each line: {"t", "proto", "src", "dst", "len", "data"(base64)}

Raw TCP segments are kept as-is (MAVLink frames may span segments); the Rust
side re-parses the byte stream per (src, dst) direction.

usage: pcap2jsonl.py IN.pcap OUT_PREFIX [DATABUS_PORT_MIN DATABUS_PORT_MAX] [FCB_PORTS,...]
"""
import base64
import json
import socket
import sys

import dpkt

pcap_path, prefix = sys.argv[1], sys.argv[2]
db_lo = int(sys.argv[3]) if len(sys.argv) > 3 else 60000
db_hi = int(sys.argv[4]) if len(sys.argv) > 4 else 61999
fcb_ports = {int(p) for p in (sys.argv[5] if len(sys.argv) > 5 else "5760,5770,5780").split(",")}

db = open(prefix + ".databus.jsonl", "w")
fcb = open(prefix + ".fcb.jsonl", "w")
counts = {"databus": 0, "fcb": 0, "other": 0}

with open(pcap_path, "rb") as f:
    reader = dpkt.pcap.Reader(f)
    link = reader.datalink()
    for ts, buf in reader:
        if link == dpkt.pcap.DLT_NULL:
            ip = dpkt.loopback.Loopback(buf).data
        elif link == dpkt.pcap.DLT_LINUX_SLL:
            ip = dpkt.sll.SLL(buf).data
        else:
            ip = dpkt.ethernet.Ethernet(buf).data
        if not isinstance(ip, dpkt.ip.IP):
            counts["other"] += 1
            continue
        l4 = ip.data
        src_ip, dst_ip = socket.inet_ntoa(ip.src), socket.inet_ntoa(ip.dst)
        payload = bytes(l4.data) if hasattr(l4, "data") else b""
        if not payload:
            continue
        rec = {"t": round(ts, 6), "src": f"{src_ip}:{l4.sport}", "dst": f"{dst_ip}:{l4.dport}",
               "len": len(payload), "data": base64.b64encode(payload).decode()}
        if isinstance(l4, dpkt.udp.UDP) and (db_lo <= l4.sport <= db_hi or db_lo <= l4.dport <= db_hi):
            db.write(json.dumps({"proto": "udp", **rec}) + "\n")
            counts["databus"] += 1
        elif isinstance(l4, dpkt.tcp.TCP) and (l4.sport in fcb_ports or l4.dport in fcb_ports):
            fcb.write(json.dumps({"proto": "tcp", **rec}) + "\n")
            counts["fcb"] += 1
        else:
            counts["other"] += 1

print(json.dumps(counts))
