// Ported from droneengage_mavlink@be5d8a5: mavlink_sdk/src/mavlink_sdk.cpp,
// mavlink_communicator.cpp, tcp_client_port.cpp, mavlink_command.cpp (the commands R2a uses)
//
// The flight-controller link. Frame I/O is done by the `mavlink` crate.

// ArduPilot still relies on REQUEST_DATA_STREAM (deprecated in the MAVLink XML).
#![allow(deprecated)]

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use mavlink::dialects::ardupilotmega::{self as apm, MavMessage};
use mavlink::{AsyncMavConnection, MavHeader};
use serde_json::Value;
use tokio::sync::RwLock;

/// Sender id de_mavlink uses for commands (C++ GCS_SYSID / component 190).
pub const GCS_SYSID: u8 = 255;
pub const GCS_COMPID: u8 = 190;
pub const MAV_COMP_ID_CAMERA: u8 = 100;

type Conn = mavlink::AsyncConnection<MavMessage>;

/// Process-wide MAVLink v2 sequence counter (C++ encodes everything on channel 0).
static SEQ: AtomicU8 = AtomicU8::new(0);

pub fn next_seq() -> u8 {
    SEQ.fetch_add(1, Ordering::Relaxed)
}

/// Serialize one message as a MAVLink v2 frame (payload zero-truncated, like c_library_v2).
pub fn encode(system_id: u8, component_id: u8, msg: &MavMessage) -> Vec<u8> {
    let mut buf = Vec::with_capacity(64);
    let header = MavHeader {
        system_id: u32::from(system_id),
        component_id,
        sequence: next_seq(),
    };
    if let Err(e) = mavlink::write_v2_msg(&mut buf, header, msg) {
        tracing::warn!(error = %e, "mavlink encode failed");
        buf.clear();
    }
    buf
}

/// Connection address in `mavlink` crate syntax from `fcb_connection_uri`.
pub fn address_from_config(cfg: &Value) -> anyhow::Result<String> {
    let uri = cfg
        .get("fcb_connection_uri")
        .ok_or_else(|| anyhow::anyhow!("missing fcb_connection_uri"))?;
    let kind = uri
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let ip = uri.get("ip").and_then(Value::as_str).unwrap_or("127.0.0.1");
    let port = uri.get("port").and_then(Value::as_i64).unwrap_or(0);
    if kind.contains("serial") {
        let dev = uri.get("port").and_then(Value::as_str).unwrap_or("").trim();
        let baud = uri.get("baudrate").and_then(Value::as_i64).unwrap_or(57600);
        Ok(format!("serial:{dev}:{baud}"))
    } else if kind.contains("udp") {
        // DIVERGENCE: C++ UDPPort semantics not verified yet; we listen on ip:port.
        Ok(format!("udpin:{ip}:{port}"))
    } else if kind.contains("tcp") {
        Ok(format!("tcpout:{ip}:{port}"))
    } else {
        anyhow::bail!("Connection to FCB is not in (serial, udp, tcp)")
    }
}

/// The FC link. Reconnects on its own; `send` is a no-op while disconnected.
pub struct FcLink {
    address: String,
    conn: RwLock<Option<Arc<Conn>>>,
}

impl FcLink {
    pub fn new(address: String) -> Arc<Self> {
        Arc::new(Self {
            address,
            conn: RwLock::new(None),
        })
    }

    async fn current(&self) -> Option<Arc<Conn>> {
        self.conn.read().await.clone()
    }

    /// Keep a connection open; returns the next frame from the FC.
    pub async fn recv(&self) -> (MavHeader, MavMessage) {
        loop {
            let conn = match self.current().await {
                Some(c) => c,
                None => match mavlink::connect_async::<MavMessage>(&self.address).await {
                    Ok(c) => {
                        let mut c: Conn = c;
                        c.set_allow_recv_any_version(true);
                        c.set_protocol_version(mavlink::MavlinkVersion::V2);
                        let c = Arc::new(c);
                        tracing::info!(address = %self.address, "FCB connection open");
                        *self.conn.write().await = Some(c.clone());
                        c
                    }
                    Err(e) => {
                        tracing::warn!(address = %self.address, error = %e, "FCB connect failed, retrying");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                },
            };
            match conn.recv().await {
                Ok(frame) => return frame,
                Err(mavlink::error::MessageReadError::Io(e)) => {
                    tracing::warn!(error = %e, "FCB link lost, reconnecting");
                    *self.conn.write().await = None;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => tracing::debug!(error = ?e, "FCB bad frame dropped"),
            }
        }
    }

    pub async fn send(&self, system_id: u8, component_id: u8, msg: &MavMessage) {
        if let Some(c) = self.current().await {
            let header = MavHeader {
                system_id: u32::from(system_id),
                component_id,
                sequence: next_seq(),
            };
            if let Err(e) = c.send(&header, msg).await {
                tracing::warn!(error = %e, "FCB send failed");
            }
        }
    }

    // ---- CMavlinkCommand -------------------------------------------------------

    /// GCS heartbeat so ArduPilot keeps streaming to us.
    pub async fn send_heartbeat_of_gcs(&self, gcs_id: u8) {
        let hb = MavMessage::HEARTBEAT(apm::HEARTBEAT_DATA {
            custom_mode: 0,
            mavtype: apm::MavType::MAV_TYPE_GCS,
            autopilot: apm::MavAutopilot::MAV_AUTOPILOT_INVALID,
            base_mode: apm::MavModeFlag::empty(),
            system_status: apm::MavState::MAV_STATE_UNINIT,
            mavlink_version: 3,
        });
        self.send(gcs_id, 0, &hb).await;
    }

    /// Re-send the vehicle's own heartbeat as another component (C++ camera emulation).
    pub async fn send_heartbeat_of_component(
        &self,
        vehicle_sysid: u8,
        hb: &apm::HEARTBEAT_DATA,
        component_id: u8,
    ) {
        self.send(
            vehicle_sysid,
            component_id,
            &MavMessage::HEARTBEAT(hb.clone()),
        )
        .await;
    }

    pub async fn request_data_stream(&self, target: (u8, u8), stream: apm::MavDataStream) {
        let m = MavMessage::REQUEST_DATA_STREAM(apm::REQUEST_DATA_STREAM_DATA {
            req_message_rate: 4,
            target_system: target.0,
            target_component: target.1,
            req_stream_id: stream,
            start_stop: 1,
        });
        self.send(GCS_SYSID, GCS_COMPID, &m).await;
    }

    /// CVehicle::requestDataStream: ALL plus each group, 4 Hz.
    pub async fn request_all_data_streams(&self, target: (u8, u8)) {
        use apm::MavDataStream::*;
        for s in [
            MAV_DATA_STREAM_ALL,
            MAV_DATA_STREAM_RAW_CONTROLLER,
            MAV_DATA_STREAM_RAW_SENSORS,
            MAV_DATA_STREAM_RC_CHANNELS,
            MAV_DATA_STREAM_EXTENDED_STATUS,
            MAV_DATA_STREAM_EXTRA1,
            MAV_DATA_STREAM_EXTRA2,
            MAV_DATA_STREAM_EXTRA3,
        ] {
            self.request_data_stream(target, s).await;
        }
    }

    pub async fn send_long_command(
        &self,
        target: (u8, u8),
        command: apm::MavCmd,
        params: [f32; 7],
    ) {
        let m = MavMessage::COMMAND_LONG(apm::COMMAND_LONG_DATA {
            param1: params[0],
            param2: params[1],
            param3: params[2],
            param4: params[3],
            param5: params[4],
            param6: params[5],
            param7: params[6],
            command,
            target_system: target.0,
            target_component: target.1,
            confirmation: 0,
        });
        self.send(GCS_SYSID, GCS_COMPID, &m).await;
    }

    pub async fn request_home_location(&self, target: (u8, u8)) {
        self.send_long_command(target, apm::MavCmd::MAV_CMD_GET_HOME_POSITION, [0.0; 7])
            .await;
    }

    /// ArduPilot guided "goto": MISSION_ITEM_INT with current = 2.
    pub async fn goto_guided_point(
        &self,
        target: (u8, u8),
        latitude: f64,
        longitude: f64,
        relative_altitude: f64,
    ) {
        let m = MavMessage::MISSION_ITEM_INT(apm::MISSION_ITEM_INT_DATA {
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            x: (latitude * 1e7).round() as i32,
            y: (longitude * 1e7).round() as i32,
            z: relative_altitude as f32,
            seq: 0,
            command: apm::MavCmd::MAV_CMD_NAV_WAYPOINT,
            target_system: target.0,
            target_component: target.1,
            frame: apm::MavFrame::MAV_FRAME_GLOBAL_RELATIVE_ALT,
            current: 2,
            autocontinue: 1,
            mission_type: apm::MavMissionType::MAV_MISSION_TYPE_MISSION,
        });
        self.send(GCS_SYSID, GCS_COMPID, &m).await;
        tracing::debug!(
            lat = latitude,
            lon = longitude,
            alt = relative_altitude,
            "gotoGuidedPoint"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn addresses() {
        let t = json!({"fcb_connection_uri": {"type": "tcp", "ip": "127.0.0.1", "port": 5760}});
        assert_eq!(address_from_config(&t).unwrap(), "tcpout:127.0.0.1:5760");
        let s = json!({"fcb_connection_uri": {"type": "Serial", "port": "/dev/ttyAMA0 ", "baudrate": 115200}});
        assert_eq!(
            address_from_config(&s).unwrap(),
            "serial:/dev/ttyAMA0:115200"
        );
    }

    #[test]
    fn encode_is_v2_frame() {
        let hb = MavMessage::HEARTBEAT(apm::HEARTBEAT_DATA::default());
        let f = encode(1, 1, &hb);
        assert_eq!(f[0], 0xFD);
        assert_eq!(f[5], 1); // sysid
    }
}
