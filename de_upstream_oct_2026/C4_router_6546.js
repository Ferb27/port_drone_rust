// C4: 3 kiểm tra mới của server Rust (upstream 20900c0 + fork P3/P4), chạy qua giao thức thật:
//   login HTTPS ở auth (/w/wl) -> WSS tới comm server -> de_auth -> gửi message dạng JSON {ty,sd,tg,mt,ms}.
// (Bản trong PROMPT_CLAUDE.md gửi frame raw TCP 0xDE.. tự chế: server không nói giao thức đó, nên mọi check sẽ "fail" vô nghĩa.)
//
//   node C4_router_6546.js --host 127.0.0.1 [--auth-port 19408] [--port 9967] [--direct-port 9966] > c4.txt
//   NODE_PATH=<thư mục có module ws>   (vd de_fork/de_server/node_modules)
//
// check1  6503 + 6546 AI_OBSERVATIONS: ms.q / ms.t (và obs) tới nơi nguyên vẹn qua header-scan routing.
// check2  64 lệnh hệ thống 9021 (UDP_PROXY_PEER) liên tiếp: server không rớt socket, đếm reply và thứ tự.
// check3  6503 latest_wins: người nhận ngừng đọc (pause socket, nối thẳng --direct-port vì proxy tự buffer),
//         người gửi nhồi > high-water 256 KiB; đếm frame bị bỏ và frame nào còn sống.
"use strict";
const https = require("https");
const WebSocket = require("ws");

const arg = (k, d) => { const i = process.argv.indexOf(k); return i > 0 ? process.argv[i + 1] : d; };
const HOST = arg("--host", "127.0.0.1");
const AUTH_PORT = +arg("--auth-port", 19408);
const PORT = arg("--port", null);            // null = dùng port server trả về lúc login (P4: de_public_port)
const DIRECT_PORT = +arg("--direct-port", 9966);
const ACC = arg("--acc", "single@airgap.droneengage.com");
const PWD = arg("--pwd", "test");
const HW = 262144;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function login() {
  const body = JSON.stringify({ acc: ACC, pwd: PWD, gr: "1", app: "de", ver: "5.0.0", ex: "C4", at: "g" });
  return new Promise((res, rej) => {
    const rq = https.request({ host: HOST, port: AUTH_PORT, path: "/w/wl/", method: "POST", rejectUnauthorized: false,
      headers: { "Content-Type": "application/json", "Content-Length": Buffer.byteLength(body) } }, (r) => {
      let d = ""; r.on("data", (c) => (d += c)); r.on("end", () => {
        try { const j = JSON.parse(d); j.e === 0 ? res(j.cs) : rej(new Error("auth: " + d)); } catch (e) { rej(e); } });
    });
    rq.on("error", rej); rq.end(body);
  });
}

async function party(pid, port) {
  const cs = await login();
  const p = port || cs.h;
  const ws = new WebSocket(`wss://${HOST}:${p}`, { rejectUnauthorized: false });
  const c = { pid, ws, port: p, rx: [], closed: false };
  ws.on("message", (m) => {
    let s = Buffer.isBuffer(m) ? m : Buffer.from(m);
    const z = s.indexOf(0); if (z >= 0) s = s.subarray(0, z);
    try { c.rx.push(JSON.parse(s.toString())); } catch (_) { /* binary payload */ }
  });
  ws.on("close", () => (c.closed = true));
  ws.on("error", () => (c.closed = true));
  await new Promise((r, j) => { ws.once("open", r); ws.once("error", j); });
  ws.send(JSON.stringify({ ty: "s", mt: "de_auth", f: cs.f, s: pid, at: "g" }));
  for (let i = 0; i < 50 && !c.rx.some((m) => m.mt === "de_auth_ack"); i++) await sleep(100);
  c.authed = c.rx.some((m) => m.mt === "de_auth_ack" && m.r === "ok");
  c.advertised = cs.h;
  return c;
}
const send = (a, b, mt, ms) => a.ws.send(JSON.stringify({ ty: "i", sd: a.pid, st: "w", tg: b.pid, mt, ms }));
const same = (x, y) => JSON.stringify(x) === JSON.stringify(y);

async function check1(out) {
  const a = await party("c4_a"), b = await party("c4_b");
  out.push(`login: advertised port=${a.advertised} (de_public_port), a.authed=${a.authed} b.authed=${b.authed}`);
  const m1 = { q: 4294967295, t: 123456 };                        // q ở biên uint32
  const m2 = { q: 7, t: 123457, obs: "AI_OBSERVATIONS" };
  send(a, b, 6503, m1); send(a, b, 6546, m2);
  await sleep(1500);
  const r1 = b.rx.find((m) => m.mt === 6503), r2 = b.rx.find((m) => m.mt === 6546);
  out.push(`check1 6503: ${r1 ? (same(r1.ms, m1) ? "PASS" : "FAIL ms=" + JSON.stringify(r1.ms)) : "FAIL không nhận được"}`);
  out.push(`check1 6546: ${r2 ? (same(r2.ms, m2) ? "PASS" : "FAIL ms=" + JSON.stringify(r2.ms)) : "FAIL không nhận được"}`);
  return { a, b };
}

async function check2(out, a) {
  const before = a.rx.length;
  for (let i = 0; i < 64; i++) a.ws.send(JSON.stringify({ ty: "s", sd: a.pid, mt: 9021, ms: { seq: i, act: "A_" + i } }));
  await sleep(2500);
  const rep = a.rx.slice(before).filter((m) => m.mt === 9021);
  const seqs = rep.map((m) => (m.ms && m.ms.seq !== undefined ? m.ms.seq : null)).filter((s) => s !== null);
  const ordered = seqs.every((s, i) => i === 0 || s > seqs[i - 1]);
  out.push(`check2 9021 x64: socket ${a.closed ? "BỊ ĐÓNG" : "còn sống"}, reply=${rep.length}, seq trả về=${seqs.length}, thứ tự ${ordered ? "đúng" : "SAI"}` +
    (rep[0] ? `, mẫu reply=${JSON.stringify(rep[0].ms).slice(0, 120)}` : ""));
  out.push(`check2: ${!a.closed && ordered ? "PASS" : "FAIL"} (không có relay thì server trả reject; PASS = không rớt socket + không đảo thứ tự)`);
}

async function check3(out, a) {
  const r = await party("c4_r", DIRECT_PORT);                     // nối thẳng, không qua proxy
  r.ws._socket.pause();                                           // ngừng đọc -> server buffer tăng
  const N = 100, PAD = "x".repeat(32 * 1024);                     // ~3.2 MiB > 256 KiB high-water, < 4 MiB hard cap
  for (let i = 0; i < N; i++) send(a, r, 6503, { q: i, t: i, pad: PAD });
  await sleep(3000);
  r.ws._socket.resume();
  await sleep(4000);
  const got = r.rx.filter((m) => m.mt === 6503).map((m) => m.ms.q);
  const dropped = N - got.length;
  out.push(`check3 6503 latest_wins: gửi=${N} nhận=${got.length} drop=${dropped} q đầu=${got[0]} q cuối=${got[got.length - 1]} receiver ${r.closed ? "BỊ terminate (hard cap)" : "còn sống"}`);
  out.push(`check3: ${dropped > 0 && !r.closed ? "PASS (latest_wins bỏ frame khi vượt high-water)" : dropped === 0 ? "FAIL (không drop: buffer không vượt high-water?)" : "FAIL"}` +
    `; frame cuối q=${N - 1} ${got.includes(N - 1) ? "còn" : "bị bỏ"} (guard bỏ frame khi buffer cao, không phải 'chỉ giữ frame mới nhất')`);
  r.ws.terminate();
}

async function run() {
  const out = [];
  try {
    const { a, b } = await check1(out);
    await check2(out, a);
    await check3(out, a);
    a.ws.close(); b.ws.close();
  } catch (e) { out.push("ERROR " + e.message); }
  console.log(out.join("\n"));
  process.exit(0);
}
module.exports = { run };
if (require.main === module) run();
