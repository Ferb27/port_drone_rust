#!/usr/bin/env python3
"""TLS-terminating WebSocket relay that logs every frame (test rigs only).

de_comm / GCS --wss--> wss_tap (LISTEN, our cert) --wss--> comm server (TARGET)

Every frame is written to a JSONL file:
  {"t": <unix time>, "conn": <n>, "path": "/...", "dir": "c2s"|"s2c",
   "kind": "text"|"binary", "data": <str for text, base64 for binary>}

The cert/key must be one the clients already trust (in the SITL rig: the
fork's self-signed CN=127.0.0.1 airgap cert). Never point this at real users.

usage: wss_tap.py LISTEN_PORT TARGET_HOST TARGET_PORT CERT KEY OUT.jsonl
"""
import asyncio
import base64
import itertools
import json
import ssl
import sys
import time

import websockets

LISTEN_PORT, TARGET_HOST, TARGET_PORT = int(sys.argv[1]), sys.argv[2], int(sys.argv[3])
CERT, KEY, OUT = sys.argv[4], sys.argv[5], sys.argv[6]

out = open(OUT, "a", buffering=1)
conn_ids = itertools.count(1)


def log(conn, path, direction, msg):
    if isinstance(msg, bytes):
        rec = {"kind": "binary", "data": base64.b64encode(msg).decode()}
    else:
        rec = {"kind": "text", "data": msg}
    out.write(json.dumps({"t": round(time.time(), 6), "conn": conn, "path": path,
                          "dir": direction, **rec}, ensure_ascii=False) + "\n")


async def pump(src, dst, conn, path, direction):
    async for msg in src:
        log(conn, path, direction, msg)
        await dst.send(msg)


async def handle(client):
    conn = next(conn_ids)
    path = client.request.path
    up_ctx = ssl.create_default_context()
    up_ctx.check_hostname = False
    up_ctx.verify_mode = ssl.CERT_NONE
    url = f"wss://{TARGET_HOST}:{TARGET_PORT}{path}"
    log(conn, path, "meta", f"open {url}")
    try:
        async with websockets.connect(url, ssl=up_ctx, max_size=None, compression=None,
                                      ping_interval=None, open_timeout=10) as server:
            tasks = [asyncio.create_task(pump(client, server, conn, path, "c2s")),
                     asyncio.create_task(pump(server, client, conn, path, "s2c"))]
            done, pending = await asyncio.wait(tasks, return_when=asyncio.FIRST_COMPLETED)
            for t in pending:
                t.cancel()
    except Exception as e:  # noqa: BLE001 - log and drop, rig tool
        log(conn, path, "meta", f"error {type(e).__name__}: {e}")
    finally:
        log(conn, path, "meta", "close")


async def main():
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(CERT, KEY)
    async with websockets.serve(handle, "0.0.0.0", LISTEN_PORT, ssl=ctx, max_size=None,
                                compression=None, ping_interval=None):
        print(f"wss_tap :{LISTEN_PORT} -> {TARGET_HOST}:{TARGET_PORT}, log {OUT}", flush=True)
        await asyncio.Future()


asyncio.run(main())
