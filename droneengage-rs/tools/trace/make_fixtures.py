#!/usr/bin/env python3
"""Turn a capture directory (from tools/sitl_gate/capture.sh) into repo fixtures.

Writes tests/fixtures/<NAME>/{databus,fcb,wss}.jsonl.gz and meta.json.
Session keys in de_auth frames ("f") are replaced, so nothing usable lands in git.

usage: make_fixtures.py CAPTURE_DIR NAME [REPO_ROOT]
"""
import gzip
import json
import os
import re
import sys

cap, name = sys.argv[1], sys.argv[2]
root = sys.argv[3] if len(sys.argv) > 3 else os.path.join(os.path.dirname(__file__), "..", "..")
dst = os.path.join(root, "tests", "fixtures", name)
os.makedirs(dst, exist_ok=True)

SESSION = re.compile(r'("f"\s*:\s*")[0-9a-fA-F]{16,}(")')


def scrub(line):
    return SESSION.sub(r"\1REDACTED_SESSION_KEY\2", line)


def gz(src, out, fn=lambda s: s):
    n = 0
    with open(src) as i, gzip.open(out, "wt", compresslevel=9) as o:
        for line in i:
            rec = json.loads(line)
            if rec.get("kind") == "text" and isinstance(rec.get("data"), str):
                rec["data"] = fn(rec["data"])
            o.write(json.dumps(rec, ensure_ascii=False) + "\n")
            n += 1
    return n


counts = {
    "databus": gz(os.path.join(cap, "trace.databus.jsonl"), os.path.join(dst, "databus.jsonl.gz")),
    "fcb": gz(os.path.join(cap, "trace.fcb.jsonl"), os.path.join(dst, "fcb.jsonl.gz")),
    "wss": gz(os.path.join(cap, "wss.jsonl"), os.path.join(dst, "wss.jsonl.gz"), scrub),
}

with gzip.open(os.path.join(dst, "wss.jsonl.gz"), "rt") as f:
    assert not re.search(r'"f\\?"\s*:\s*\\?"[0-9a-f]{16,}', f.read()), "session key left in wss fixture"

meta = {
    "name": name,
    "lines": counts,
    "binaries": open(os.path.join(cap, "binaries.txt")).read().strip().splitlines(),
    "gate": open(os.path.join(cap, "gate.ana")).read().strip().splitlines()[:6],
}
json.dump(meta, open(os.path.join(dst, "meta.json"), "w"), indent=1, ensure_ascii=False)
print(json.dumps(counts))
