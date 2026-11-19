import sys,time,os,json,math,threading,csv; sys.path.insert(0,os.path.dirname(os.path.abspath(__file__)))
from pymavlink import mavutil
from gcs import GCS
OUT=sys.argv[1] if len(sys.argv)>1 else "/tmp/rv_gate/exp1.csv"
FORM=int(os.environ.get("FORM","2"))
mv=[mavutil.mavlink_connection(f"tcp:127.0.0.1:{5762+10*i}",source_system=200+i) for i in range(3)]
for i,m in enumerate(mv):
    m.wait_heartbeat(timeout=20); m.target_system=i+1; m.target_component=1
print("hb ok")
def armed(m):
    t=time.time()
    while time.time()-t<2:
        hb=m.recv_match(type="HEARTBEAT",blocking=True,timeout=1)
        if hb and hb.get_srcSystem()==m.target_system and hb.get_srcComponent()==1: return bool(hb.base_mode & 128)
    return False
def guided_arm_takeoff(m,alt):
    m.set_mode("GUIDED"); time.sleep(1)
    for _ in range(40):
        m.arducopter_arm(); time.sleep(1)
        if armed(m): break
    print("armed", m.target_system, armed(m))
    m.mav.command_long_send(m.target_system,m.target_component,mavutil.mavlink.MAV_CMD_NAV_TAKEOFF,0,0,0,0,0,0,0,alt)
for m in mv:
    m.mav.param_set_send(m.target_system,m.target_component,b"WPNAV_SPEED",500,mavutil.mavlink.MAV_PARAM_TYPE_REAL32)
for m in mv: guided_arm_takeoff(m,15)
print("takeoff sent"); time.sleep(20)
g=GCS(os.environ["ACC"],os.environ["AC"]); time.sleep(1); g.req_id(); time.sleep(3)
byud={v["UD"]:k for k,v in g.units.items()}; L,F2,F3=byud["drone1"],byud["drone2"],byud["drone3"]
g.send(L,1056,{"a":FORM,"b":L,"h":15,"v":0}); time.sleep(2)
for f in (F2,F3): g.send(f,1054,{"a":-1,"b":L,"c":f,"f":1}); time.sleep(2)
time.sleep(5)
pos={}; stop=False
def logger():
    w=csv.writer(open(OUT,"w")); w.writerow(["t","id","lat","lon","ralt","vx","vy"])
    while not stop:
        for i,m in enumerate(mv):
            while True:
                msg=m.recv_match(type="GLOBAL_POSITION_INT",blocking=False)
                if msg is None: break
                w.writerow([f"{time.time():.3f}",i,msg.lat,msg.lon,msg.relative_alt,msg.vx,msg.vy])
        time.sleep(0.02)
for m in mv: m.mav.request_data_stream_send(m.target_system,m.target_component,mavutil.mavlink.MAV_DATA_STREAM_POSITION,10,1)
th=threading.Thread(target=logger,daemon=True); th.start()
t0=time.time(); print("T0",t0)
# leader: go 250 m north then 250 m east
lead=mv[0]
def goto(dn,de):
    msg=None
    while msg is None: msg=lead.recv_match(type="GLOBAL_POSITION_INT",blocking=True,timeout=2)
    lat=-35.363261+dn/111111.0; lon=149.165230+de/(111111.0*math.cos(math.radians(-35.36)))
    lead.mav.set_position_target_global_int_send(0,lead.target_system,lead.target_component,6,0b110111111000,int(lat*1e7),int(lon*1e7),15,0,0,0,0,0,0,0,0)
time.sleep(10); goto(250,0); time.sleep(60); goto(250,250); time.sleep(60)
stop=True; time.sleep(0.5); print("T1",time.time())
