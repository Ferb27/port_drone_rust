# UDP Telemetry Commands — Reverse-Engineered

Source: JS bundle `main.7909d061.js` from cloud.ardupilot.org:8001

## Constants

| Constant | Value | Meaning |
|----------|-------|---------|
| `In` (message type) | **1005** | RemoteExecute — tất cả lệnh telemetry dùng type này |
| `Pn` (command code) | **108** | Telemetry control command |

## Commands (Web Client → Drone)

All via `API_sendCMD(partyID, 1005, payload)`

### Resume UDP
```
{ C: 108, Act: 3 }
```

### Pause UDP
```
{ C: 108, Act: 5 }
```

### Change Optimization Level
```
{ C: 108, Act: 4, LVL: 0-3 }
```
LVL 0=full rate, 3=lowest bandwidth.

### Stop Telemetry
```
{ C: 108, Act: 2 }
```

### Request Status
```
{ C: 1071 }
```

### Set UDP Client Port
```
{ C: 506, P: <port> }
```

## Response (Drone → Web Client) — Message Type 1071

```
{
  a:  string,   // udpProxy_ip
  p:  number,   // udpProxy_port
  o:  number,   // telemetry_level (0-3)
  en: boolean,  // udpProxy_active
  z:  boolean   // udpProxy_paused (optional)
}
```

## JS Source References

From `js_andruav_facade.js` (reconstructed):

```javascript
// API layer
static API_resumeTelemetry() { return { mt: 1005, ms: { C: 108, Act: 3 } } }
static API_pauseTelemetry()  { return { mt: 1005, ms: { C: 108, Act: 5 } } }

// Facade layer
API_adjustTelemetryDataRate(unit, level) {
  let payload = { C: 108, Act: 4 };
  if (level !== null && level !== -1) payload.LVL = level;
  API_sendCMD(unit.getPartyID(), 1005, payload);
}

API_requestUdpProxyStatus(unit) {
  API_sendCMD(unit.getPartyID(), 1005, { C: 1071 });
}
```

From `js_andruav_parser.js` (message handler for type 1071):
```javascript
case 1071:
  unit.m_Telemetry.m_udpProxy_ip = msg.a
  unit.m_Telemetry.m_udpProxy_port = msg.p
  unit.m_Telemetry.m_telemetry_level = msg.o
  unit.m_Telemetry.m_udpProxy_active = msg.en
  unit.m_Telemetry.m_udpProxy_paused = msg.hasOwnProperty('z') ? msg.z : false
  dispatch(EE_onProxyInfoUpdated, unit)
```
