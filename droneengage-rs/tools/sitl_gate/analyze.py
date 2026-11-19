import csv,math,sys,collections
rows=list(csv.DictReader(open(sys.argv[1])))
idx={1:int(sys.argv[2]),2:int(sys.argv[3])}  # sitl index -> follower_index
H=15.0
R=6371000.0
def en(lat,lon,lat0,lon0): return ((lon-lon0)*math.pi/180*R*math.cos(lat0*math.pi/180),(lat-lat0)*math.pi/180*R)
last={}; err=collections.defaultdict(list); t0=float(rows[0]["t"])
series=[]
for r in rows:
    i=int(r["id"]); last[i]=r
    if i==0 or 0 not in last: continue
    L=last[0]; la,lo=int(L["lat"])/1e7,int(L["lon"])/1e7
    vx,vy=int(L["vx"]),int(L["vy"])
    if vx*vx+vy*vy<2500: continue
    brg=math.atan2(vy,vx)  # bearing from north (vx north, vy east)
    k=idx[i]; left=(k%2==0); base=((k//2)+1)*H
    ang=brg+math.pi+(-math.pi/4 if left else math.pi/4)
    tn,te=base*math.cos(ang),base*math.sin(ang)
    fe,fn=en(int(r["lat"])/1e7,int(r["lon"])/1e7,la,lo)
    e=math.hypot(fe-te,fn-tn); t=float(r["t"])-t0
    err[i].append((t,e))
for i in sorted(err):
    es=sorted(e for _,e in err[i]); n=len(es)
    rms=math.sqrt(sum(e*e for e in es)/n)
    print(f"sitl{i} (idx {idx[i]}): n={n} mean={sum(es)/n:.2f} m RMS={rms:.2f} p50={es[n//2]:.2f} p95={es[int(n*.95)]:.2f} max={es[-1]:.2f}")
    # steady-state: windows
    for a,b in ((20,65),(100,125)):
        w=[e for t,e in err[i] if a<=t<b]
        if w: print(f"   window {a}-{b}s: mean={sum(w)/len(w):.2f} m  max={max(w):.2f}")
print("relative position in leader frame (along +fwd, cross +right), steady windows:")
last={}
acc=collections.defaultdict(list)
for r in rows:
    i=int(r["id"]); last[i]=r
    if i==0 or 0 not in last: continue
    L=last[0]; vx,vy=int(L["vx"]),int(L["vy"])
    if vx*vx+vy*vy<2500: continue
    t=float(r["t"])-t0
    if not (20<=t<65 or 100<=t<125): continue
    brg=math.atan2(vy,vx)
    fe,fn=en(int(r["lat"])/1e7,int(r["lon"])/1e7,int(L["lat"])/1e7,int(L["lon"])/1e7)
    along=fn*math.cos(brg)+fe*math.sin(brg); cross=-fn*math.sin(brg)+fe*math.cos(brg)
    acc[(i, 'A' if t<65 else 'B')].append((along,cross,math.hypot(vx,vy)/100))
for k,v in sorted(acc.items()):
    n=len(v); print(k, f"along={sum(a for a,_,_ in v)/n:.1f} cross={sum(c for _,c,_ in v)/n:.1f} leader_speed={sum(s for *_,s in v)/n:.2f} m/s")
print("ideal: along=-10.6, cross=-10.6 (idx0 left) / +10.6 (idx1 right)")
