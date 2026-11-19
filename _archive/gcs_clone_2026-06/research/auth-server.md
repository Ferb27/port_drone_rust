# Authentication Server — Research Notes

Source: https://cloud.ardupilot.org/srv-authentication.html

## Overview

- Validates GCS and units against predefined accounts
- Returns JSON with comm server address/port + temp login key (NO HTTP redirect)
- Written in Node.js
- Source: github.com/DroneEngage/droneegnage_authenticator

## Auth Flow

```
Client → POST /wl/<groupName> → Auth Server (port 19408)
Auth Server → requests temp key from Communication Server (via s2s WebSocket)
Auth Server ← JSON { cs: { g: host, h: port, f: temp_key }, sid: session_id } → Client
Client → WebSocket wss://host:port → sends temp_key + partyID
```

## Config (server.config)

- `server_port`: default 19408
- `account_storage_type`: single | file | db
- `single_account_user_name`: e.g. single@airgap.droneengage.com
- `single_account_access_code`: e.g. "test"
- `db_users`: JSON file path for file-based storage
- `s2s_ws_listening_ip/port`: WebSocket connection to Communication Server (default 127.0.0.1:19001)
- `enable_SSL`: true/false

## DB Users File Format (file storage)

```json
{
    "user@email.com": {
        "sid": 1,
        "pwd": "0001",
        "isadmin": true,
        "prm": "0xffffffff"
    }
}
```

Users with same `sid` share access to same drones (multi-login).
