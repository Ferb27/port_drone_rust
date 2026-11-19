# Web UDP Telemetry — Research Notes

Source: https://cloud.ardupilot.org/webclient-udp-telemetry.html

## Overview

- Built into Communication Server (not a separate module)
- Creates 2 sockets: (1) receives telemetry from drone via UDP, (2) allows GCS connection via UDPCI
- Access WebClient → get UDP IP & port → connect Mission Planner via UDPCI
- **LVL** control: Level 0 (full rate) → Level 3 (lowest rate)

## Rate Control

JSON config in `de-config-mavlink` defines per-message delays per LVL:

```json
{
  "message_timeouts": {
    "24": [0, 800, 1000, 2000],    // GPS_RAW_INT
    "30": [0, 250, 1000, 2000],    // ATTITUDE
    "33": [0, 250, 1000, 2000],    // GLOBAL_POSITION_INT
    "65": [0, 500, 1000, 2000],    // RC_CHANNELS
    "74": [0, 500, 1000, 2000],    // VFR_HUD
    "147": [0, 250, 2000, 4000],   // BATTERY_STATUS
    // ... full list in source doc
  }
}
```

## Enabling

Set `"udp_proxy_enabled": true` in `de-config-mavlink` config. Optional fixed port:
```json
{
  "udp_proxy_enabled": true,
  "udp_proxy_fixed_port": 15412
}
```

## Notes

- UDP port stays fixed as long as Comm Server isn't restarted
- Pause/resume via power button in WebClient
