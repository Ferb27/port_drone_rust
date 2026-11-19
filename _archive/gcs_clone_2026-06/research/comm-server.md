# Communication Server — Research Notes

Source: https://cloud.ardupilot.org/srv-communication.html

## Overview

- Acts like a chatting server — relays messages between units and webclients
- Maintains persistent WebSocket to authenticator (s2s)
- Source: github.com/DroneEngage/droneengage_communication

## Config (server.config)

- `public_host`: "airgap.droneengage.com" — the domain/IP visible to clients
- `server_sid`: unique fixed number, allows multiple comm servers
- `server_port`: default 9966
- `enable_SSL`: true
- `s2s_ws_target_ip:port`: points to Auth Server (127.0.0.1:19408)
- `allow_fake_SSL`: true — skip SSL validation when needed
- `command_plugin`: "./js_andruav_command_processing" — plugin for processing commands
- `allow_udpproxy_fixed_port`: true — enables Web UDP Telemetry

## Shared Auth Diagram

![Auth Sequence Diagram](https://cloud.ardupilot.org/_images/seq_diagram_authentication.png)

1. Authenticator listens on port 19408 (HTTP)
2. Communicator listens on port 9966 (WebSocket)
3. s2s WebSocket between them (internal, 127.0.0.1)
4. Client POSTs to Auth → gets temp key → connects to Comm via WS
