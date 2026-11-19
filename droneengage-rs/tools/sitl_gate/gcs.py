# Minimal DroneEngage GCS client: login via auth, connect to comm server, send swarm commands.
import json, ssl, sys, time, threading, urllib.request, os
import websocket
AUTH="https://127.0.0.1:19408/w/wl/"
ctx=ssl._create_unverified_context()
def login(acc, pwd):
    body=json.dumps({"acc":acc,"pwd":pwd,"gr":"1","app":"de","ver":"5.0.0","ex":"DRONE ENGAGE Web Client","at":"g"}).encode()
    r=urllib.request.urlopen(urllib.request.Request(AUTH,data=body,headers={"Content-Type":"application/json"}),context=ctx,timeout=10)
    d=json.loads(r.read()); assert d.get("e")==0, d
    return d["cs"]
class GCS:
    def __init__(self, acc, pwd, pid="gcs_test"):
        cs=login(acc,pwd); self.pid=pid; self.units={}
        self.ws=websocket.create_connection(f"wss://{cs['g']}:{cs['h']}",sslopt={"cert_reqs":ssl.CERT_NONE})
        self.ws.send(json.dumps({"ty":"s","mt":"de_auth","f":cs["f"],"s":pid,"at":"g"}))
        threading.Thread(target=self._rx,daemon=True).start()
    def _rx(self):
        while True:
            try: m=self.ws.recv()
            except Exception as e: print("rx end",e); return
            if isinstance(m,bytes): m=m.split(b"\0")[0].decode(errors="ignore")
            try: j=json.loads(m)
            except: continue
            if j.get("mt")==1004: self.units[j.get("sd")]=j.get("ms",{})
            elif j.get("mt") in (1056,1054,1058,1002,1001) or j.get("ty")=="s": print("RX",str(j)[:220])
    def send(self, tg, mt, ms):
        self.ws.send(json.dumps({"ty":"i","sd":self.pid,"st":"w","tg":tg,"mt":mt,"ms":ms}))
    def req_id(self):
        self.ws.send(json.dumps({"ty":"g","sd":self.pid,"st":"w","mt":1005,"ms":{"C":1004}}))
