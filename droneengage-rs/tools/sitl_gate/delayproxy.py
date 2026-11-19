# Order-preserving TCP delay proxy emulating a cellular link (per direction):
# delay = D + U(-J,J); with prob P a chunk incurs an extra RTO (TCP retransmission, head-of-line).
import asyncio, random, sys, time, os
LH,LP,TH,TP=sys.argv[1],int(sys.argv[2]),sys.argv[3],int(sys.argv[4])
def cfg():
    try: d,j,p,rto=[float(x) for x in open(os.environ.get("LINK_CFG","/tmp/rv_gate/link.cfg")).read().split()]
    except Exception: d,j,p,rto=0,0,0,0.3
    return d/1000,j/1000,p,rto
async def pipe(r,w):
    last=0.0; q=asyncio.Queue()
    async def sender():
        while True:
            t,data=await q.get()
            if data is None: w.close(); return
            dt=t-time.monotonic()
            if dt>0: await asyncio.sleep(dt)
            w.write(data); await w.drain()
    s=asyncio.create_task(sender())
    while True:
        data=await r.read(65536)
        if not data: await q.put((0,None)); break
        d,j,p,rto=cfg(); now=time.monotonic()
        t=now+max(0,d+random.uniform(-j,j))
        if random.random()<p: t+=rto*random.uniform(1,2)
        last=max(last,t); await q.put((last,data))
    await s
async def handle(cr,cw):
    sr,sw=await asyncio.open_connection(TH,TP)
    await asyncio.gather(pipe(cr,sw),pipe(sr,cw),return_exceptions=True)
async def main():
    srv=await asyncio.start_server(handle,LH,LP); print("proxy",LH,LP,"->",TH,TP,flush=True)
    async with srv: await srv.serve_forever()
asyncio.run(main())
