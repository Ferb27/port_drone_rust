// Ported from droneengage_mavlink@be5d8a5: src/fcb_main.cpp, fcb_main.hpp, fcb_facade.cpp,
// src/swarm/fcb_swarm_manager.cpp (message-sending parts)
//
// `Fcb` is C++ CFCBMain + CFCBFacade + the swarm singletons. One instance,
// behind a tokio Mutex, driven by three tasks in main.rs: FC frames in,
// databus messages in, and the 10 ms scheduler.
//
// R2a scope (enough for the SITL swarm gate): identity/telemetry to GCS,
// FCB events, swarm leader/follower. Not yet (R2b): remote control, missions,
// geofence, DE-Pilot, tracking, precland, UDP proxy forwarding, parameters,
// config hot-reload, FlightControl/Arm/GuidedPoint/... GCS commands.

use std::sync::Arc;
use std::time::Instant;

use de_common::events::*;
use de_common::messages::*;
use de_common::Module;
use mavlink::dialects::ardupilotmega::{self as apm, MavMessage};
use serde_json::{json, Value};

use crate::defines::*;
use crate::mav::{self, FcLink, MAV_COMP_ID_CAMERA};
use crate::modes;
use crate::swarm::{self, Swarm};
use crate::vehicle::{self, Action, Vehicle, VehicleEvent};

/// C++ ANDRUAV_UDP_PROXY (only the status part; forwarding is R2b).
#[derive(Debug, Clone)]
pub struct UdpProxy {
    pub udp_ip1: String,
    pub udp_port1: i32,
    pub udp_ip2: String,
    pub udp_port2: i32,
    pub enabled: bool,
    pub paused: bool,
}

impl Default for UdpProxy {
    fn default() -> Self {
        Self {
            udp_ip1: String::new(),
            udp_port1: 0,
            udp_ip2: String::new(),
            udp_port2: 0,
            enabled: false,
            paused: true,
        }
    }
}

pub struct Fcb {
    pub module: Module,
    pub link: Arc<FcLink>,
    pub vehicle: Vehicle,
    pub info: VehicleInfo,
    pub swarm: Swarm,
    pub fcb_connected: bool,
    pub udp_proxy: UdpProxy,
    pub udp_proxy_requested_enabled: bool,
    pub udp_telemetry_fixed_port: u16,
    pub optimization_level: i32,
    pub server_status: i32,
    counter: u64,
    last_start_flying_us: u64,
    epoch: Instant,
}

impl Fcb {
    pub fn new(module: Module, link: Arc<FcLink>, config: Value) -> Self {
        let mut vehicle = Vehicle::new();
        if let Some(ids) = config.get("mavlink_ids").filter(|v| v.is_object()) {
            if let Some(s) = ids
                .get("only_allow_ardupilot_sysid")
                .and_then(Value::as_i64)
            {
                vehicle.restrict_sysid(s as u32);
            }
            if let Some(c) = ids
                .get("only_allow_ardupilot_compid")
                .and_then(Value::as_i64)
            {
                vehicle.restrict_compid(c as u32);
            }
            if let Some(g) = ids.get("de_mavlink_gcs_id").and_then(Value::as_i64) {
                vehicle.set_gcs_id(g as i32);
            }
        }
        let optimization_level = config
            .get("default_optimization_level")
            .and_then(Value::as_i64)
            .unwrap_or(0) as i32;
        let udp_enabled = config
            .get("udp_proxy_enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if udp_enabled {
            tracing::warn!("udp_proxy_enabled: UDP proxy forwarding is not ported yet (R2b)");
        }
        Self {
            module,
            link,
            vehicle,
            info: VehicleInfo::default(),
            swarm: Swarm::default(),
            fcb_connected: false,
            udp_proxy: UdpProxy::default(),
            udp_proxy_requested_enabled: udp_enabled,
            udp_telemetry_fixed_port: 0,
            optimization_level,
            server_status: SOCKET_STATUS_FREASH,
            counter: 0,
            last_start_flying_us: 0,
            epoch: Instant::now(),
        }
    }

    /// Monotonic microseconds (C++ get_time_usec_monotonic).
    pub fn now_us(&self) -> u64 {
        self.epoch.elapsed().as_micros() as u64 + 1
    }

    // ======================= FC side =========================================

    /// CMavlinkSDK::OnMessageReceived -> CVehicle::parseMessage -> CFCBMain events.
    pub async fn on_fc_frame(&mut self, header: mavlink::MavHeader, msg: MavMessage) {
        if !self.info.use_fcb {
            // CMavlinkCommunicator: first frame from the port = connected
            self.on_connected(true).await;
        }
        let now = self.now_us();
        let (processed, events, actions) = self.vehicle.parse(&header, &msg, now);
        for a in actions {
            match a {
                Action::RequestDataStreams => {
                    self.link
                        .request_all_data_streams(self.vehicle.target())
                        .await
                }
                Action::GcsHeartbeat(id) => self.link.send_heartbeat_of_gcs(id).await,
            }
        }
        for e in events {
            self.on_vehicle_event(e).await;
        }
        if processed {
            if let MavMessage::HEARTBEAT(_) = msg {
                self.on_heartbeat();
            }
        }
    }

    async fn on_connected(&mut self, connected: bool) {
        if self.info.use_fcb != connected {
            self.info.use_fcb = connected;
            let ev = if connected {
                DRONE_CONNECTED
            } else {
                DRONE_DISCONNECTED
            };
            self.send_sync_fire_event_with("", ev, json!({}), false)
                .await;
        }
    }

    fn on_heartbeat(&mut self) {
        if self.info.is_flying {
            self.info.flying_last_start_time = self.now_us() - self.last_start_flying_us;
        }
    }

    async fn on_vehicle_event(&mut self, e: VehicleEvent) {
        match e {
            VehicleEvent::HeartBeatFirst(hb) => {
                self.info.vehicle_type = modes::andruav_vehicle_type(hb.mavtype as i32);
                self.info.autopilot = hb.autopilot as u8;
                self.info.gps_mode = GPS_MODE_FCB;
                self.api_ic_send_id("").await;
                self.link.request_home_location(self.vehicle.target()).await;
            }
            VehicleEvent::HeartBeatResumed(hb) => {
                self.info.vehicle_type = modes::andruav_vehicle_type(hb.mavtype as i32);
                self.info.autopilot = hb.autopilot as u8;
                self.api_ic_send_id("").await;
            }
            VehicleEvent::BoardRestarted => {
                self.send_sync_fire_event_with("", DRONE_FCB_RESTARTED, json!({}), false)
                    .await;
                self.module
                    .send_error_message(
                        ANDRUAV_PROTOCOL_SENDER_ALL_GCS,
                        0,
                        ERROR_TYPE_LO7ETTA7AKOM,
                        NOTIFICATION_TYPE_ERROR,
                        "FCB board has been restarted",
                    )
                    .await;
            }
            VehicleEvent::Armed {
                armed,
                ready_to_arm,
            } => {
                self.info.is_armed = armed;
                if armed {
                    self.send_sync_fire_event_with(
                        "",
                        DRONE_ARMED,
                        json!({"r": ready_to_arm}),
                        false,
                    )
                    .await;
                } else {
                    self.send_sync_fire_event_with("", DRONE_DISARMED, json!({}), false)
                        .await;
                }
                self.info.is_ready_to_arm = ready_to_arm;
                self.api_ic_send_id("").await;
            }
            VehicleEvent::Flying(f) => {
                if self.info.is_flying != f {
                    let now = self.now_us();
                    if f {
                        self.last_start_flying_us = now;
                        self.info.flying_last_start_time = 0;
                    } else {
                        self.info.flying_total_duration += self.info.flying_last_start_time;
                        self.last_start_flying_us = 0;
                    }
                    self.info.is_flying = f;
                    let ev = if f { DRONE_FLYING } else { DRONE_LANDED };
                    self.send_sync_fire_event_with("", ev, json!({}), false)
                        .await;
                    self.api_ic_send_id("").await;
                }
            }
            VehicleEvent::ModeChanged {
                custom_mode,
                mav_type,
                autopilot,
            } => {
                let vt = modes::andruav_vehicle_type(mav_type);
                self.info.vehicle_type = vt;
                self.info.autopilot = autopilot;
                self.info.flying_mode = modes::andruav_mode(custom_mode, vt, autopilot);
                let ev = match self.info.flying_mode {
                    mode::ALT_HOLD => Some(DRONE_MODE_ALT_HOLD),
                    mode::GUIDED => Some(DRONE_MODE_GUIDED),
                    mode::GUIDED_NO_GPS => Some(DRONE_MODE_GUIDED_NO_GPS),
                    mode::AUTO => Some(DRONE_MODE_AUTO),
                    mode::RTL => Some(DRONE_MODE_RTL),
                    mode::STABILIZE => Some(DRONE_MODE_STABILIZE),
                    mode::LOITER => Some(DRONE_MODE_LOITER),
                    mode::POS_HOLD => Some(DRONE_MODE_POSHOLD),
                    _ => None,
                };
                if let Some(ev) = ev {
                    let m = json!({"mode": self.info.flying_mode, "custom_mode": custom_mode});
                    self.send_sync_fire_event_with("", ev, m, false).await;
                }
                // DIVERGENCE: C++ also notifies DE-Pilot here (sends a 1008 when the
                // mode disables pilot operations). DE-Pilot is R2b.
                self.api_ic_send_id("").await;
            }
            VehicleEvent::HomePositionUpdated(h) => {
                self.send_home_location("").await;
                let m = json!({"lat": h.latitude, "lng": h.longitude, "alt": h.altitude});
                self.send_sync_fire_event_with("", DRONE_HOME_SET, m, false)
                    .await;
            }
            VehicleEvent::StatusText { severity, text } => {
                if text.is_ascii() {
                    self.module
                        .send_error_message(
                            ANDRUAV_PROTOCOL_SENDER_ALL_GCS,
                            0,
                            ERROR_3DR,
                            i32::from(severity),
                            &text,
                        )
                        .await;
                }
            }
            VehicleEvent::Ack { command, result } => {
                // MAV_RESULT: 0 accepted, 5 in progress, 1 temp rejected, 3 unsupported, 2 denied, 4 failed
                if result != 0 {
                    let sev = match result {
                        5 => NOTIFICATION_TYPE_INFO,
                        1 | 3 => NOTIFICATION_TYPE_WARNING,
                        2 | 4 => NOTIFICATION_TYPE_ERROR,
                        _ => NOTIFICATION_TYPE_WARNING,
                    };
                    // DIVERGENCE: C++ uses CMavlinkHelper::getACKError text; ported in R2b.
                    let text = format!("cmd {command} result {result}");
                    self.module
                        .send_error_message(
                            ANDRUAV_PROTOCOL_SENDER_ALL_GCS,
                            0,
                            ERROR_3DR,
                            sev,
                            &text,
                        )
                        .await;
                }
            }
        }
    }

    // ======================= DE side ==========================================

    /// Called for TYPE_AndruavModule_ID from de_comm (main.cpp onReceive).
    pub async fn on_module_id(&mut self, cmd: &Value) {
        self.info.party_id = self.module.party_id();
        self.info.group_id = self.module.group_id();
        let Some(status) = cmd.get("g").and_then(de_common::json::as_cpp_int) else {
            return;
        };
        if status != self.server_status {
            self.on_server_status(status).await;
        }
        self.server_status = status;
    }

    async fn on_server_status(&mut self, status: i32) {
        if status == SOCKET_STATUS_REGISTERED {
            self.module.send_mremsg(TYPE_AndruavSystem_LoadTasks).await;
            let port = i32::from(self.udp_telemetry_fixed_port);
            self.request_udp_proxy_telemetry(
                self.udp_proxy_requested_enabled,
                "0.0.0.0",
                0,
                "0.0.0.0",
                port,
            )
            .await;
        }
        // ERROR / DISCONNECTED: C++ alertDroneEngageOffline() only prints.
    }

    /// TYPE_AndruavSystem_UDPProxy from the comm server.
    pub async fn update_udp_proxy(
        &mut self,
        enabled: bool,
        ip1: &str,
        port1: i32,
        ip2: &str,
        port2: i32,
    ) {
        if enabled {
            self.udp_proxy_requested_enabled = true;
            // DIVERGENCE: forwarding not ported (R2b); report it as not enabled.
            tracing::warn!("UDP proxy requested but not ported yet (R2b)");
        }
        self.udp_proxy.enabled = false;
        self.udp_proxy.udp_ip1 = ip1.to_string();
        self.udp_proxy.udp_port1 = port1;
        self.udp_proxy.udp_ip2 = ip2.to_string();
        self.udp_proxy.udp_port2 = port2;
        self.send_udp_proxy_status_of_me().await;
    }

    pub async fn send_udp_proxy_status_of_me(&self) {
        let p = &self.udp_proxy;
        self.send_udp_proxy_status(
            "",
            p.enabled,
            p.paused,
            &p.udp_ip2,
            p.udp_port2,
            self.optimization_level,
        )
        .await;
    }

    // ======================= scheduler =========================================

    /// One 10 ms tick of CFCBMain::loopScheduler.
    pub async fn tick(&mut self) {
        self.counter += 1;
        let c = self.counter;
        if c.is_multiple_of(10) {
            self.handle_swarms_as_leader().await;
        }
        if c.is_multiple_of(30) {
            self.send_location_info().await;
        }
        if c.is_multiple_of(50) {
            self.send_nav_info(ANDRUAV_PROTOCOL_SENDER_ALL_GCS).await;
            self.send_gps_info(ANDRUAV_PROTOCOL_SENDER_ALL_GCS).await;
            self.heartbeat_camera().await;
        }
        if c.is_multiple_of(100) {
            self.send_if_unprocessed(ANDRUAV_PROTOCOL_SENDER_ALL_GCS, &[vehicle::MSG_WIND])
                .await;
            self.send_if_unprocessed(
                ANDRUAV_PROTOCOL_SENDER_ALL_GCS,
                &[vehicle::MSG_TERRAIN_REPORT],
            )
            .await;
        }
        if c.is_multiple_of(500) {
            self.send_if_unprocessed(
                ANDRUAV_PROTOCOL_SENDER_ALL_GCS,
                &[vehicle::MSG_EKF_STATUS_REPORT],
            )
            .await;
            self.send_if_unprocessed(ANDRUAV_PROTOCOL_SENDER_ALL_GCS, &[vehicle::MSG_VIBRATION])
                .await;
            self.send_if_unprocessed(
                ANDRUAV_PROTOCOL_SENDER_ALL_GCS,
                &[vehicle::MSG_BATTERY_STATUS, vehicle::MSG_BATTERY2],
            )
            .await;
            let connected = self.vehicle.is_fcb_connected(self.now_us());
            if connected != self.fcb_connected {
                self.fcb_connected = connected;
                self.api_ic_send_id("").await;
            }
        }
        if c.is_multiple_of(1000) {
            self.api_ic_send_id("").await;
            self.send_mission_current("").await;
            // waypoint manager not ported: current mission item is always 0
            self.send_mission_item_sequence("0").await;
        }
    }

    async fn heartbeat_camera(&self) {
        let hb = self.vehicle.heartbeat().clone();
        self.link
            .send_heartbeat_of_component(self.vehicle.sysid(), &hb, MAV_COMP_ID_CAMERA)
            .await;
    }

    // ======================= swarm =============================================

    async fn handle_swarms_as_leader(&mut self) {
        if !self.swarm.is_leader {
            return;
        }
        let now = self.now_us();
        if now - self.swarm.leader_last_update_us <= swarm::LEADER_UPDATE_US {
            return;
        }
        self.swarm.leader_last_update_us = now;
        match self.swarm.formation_as_leader {
            swarm::FORMATION_THREAD | swarm::FORMATION_ARROW | swarm::FORMATION_ARROW_DYNAMIC => {
                let gpos = MavMessage::GLOBAL_POSITION_INT(self.vehicle.global_position_int());
                let att = MavMessage::ATTITUDE(self.vehicle.attitude());
                let followers: Vec<String> = self
                    .swarm
                    .followers
                    .iter()
                    .map(|f| f.party_id.clone())
                    .collect();
                for party in followers {
                    let mut buf = self.encode_as_vehicle(&gpos);
                    buf.extend(self.encode_as_vehicle(&att));
                    self.module
                        .send_bmsg(
                            &party,
                            &buf,
                            TYPE_AndruavMessage_SWARM_MAVLINK,
                            true,
                            Value::Null,
                        )
                        .await;
                }
            }
            _ => {}
        }
    }

    /// TYPE_AndruavMessage_SWARM_MAVLINK from my leader (CSwarmFollower::handle_leader_traffic).
    pub async fn handle_leader_traffic(&mut self, leader: &str, raw: &[u8]) {
        if !self.swarm.is_follower() || !self.swarm.is_my_leader(leader) {
            return;
        }
        let Some(nul) = raw.iter().position(|&b| b == 0) else {
            return;
        };
        let mut reader = mavlink::MavlinkReader::new(&raw[nul + 1..]);
        let mut valid = false;
        loop {
            match reader.read_any_message::<MavMessage>() {
                Ok((_, MavMessage::GLOBAL_POSITION_INT(g))) => {
                    self.swarm.follower.leader_gpos = g;
                    valid = true;
                }
                Ok((_, MavMessage::ATTITUDE(a))) => {
                    self.swarm.follower.leader_yaw = a.yaw;
                    self.swarm.follower.leader_yawspeed = a.yawspeed;
                    valid = true;
                }
                Ok(_) => valid = true,
                Err(_) => break,
            }
        }
        if valid {
            self.update_follower().await;
        }
    }

    async fn update_follower(&mut self) {
        let me = self.vehicle.global_position_int();
        let (idx, h, v) = (
            self.swarm.follower_index,
            self.swarm.follower_min_h,
            self.swarm.follower_min_v,
        );
        let f = &mut self.swarm.follower;
        let target = match self.swarm.formation_as_follower {
            swarm::FORMATION_THREAD => swarm::update_thread(f, &me, idx, h, v),
            swarm::FORMATION_ARROW => swarm::update_arrow(f, &me, idx, h, v, false),
            swarm::FORMATION_ARROW_DYNAMIC => swarm::update_arrow(f, &me, idx, h, v, true),
            _ => None,
        };
        if let Some(t) = target {
            self.link
                .goto_guided_point(self.vehicle.target(), t.latitude, t.longitude, t.altitude)
                .await;
            self.send_fcb_target_location(
                "",
                t.latitude,
                t.longitude,
                t.leader_relative_alt_mm,
                DESTINATION_SWARM_MY_LOCATION,
            )
            .await;
        }
    }

    /// CSwarmManager::followLeader
    pub async fn follow_leader(
        &mut self,
        leader: &str,
        follower_index: i32,
        formation: i32,
        requested_by: &str,
    ) {
        if leader.is_empty() {
            return;
        }
        if !self.swarm.leader_party_id.is_empty() && self.swarm.leader_party_id != leader {
            let old = self.swarm.leader_party_id.clone();
            let me = self.info.party_id.clone();
            Box::pin(self.unfollow_leader(&old, &me)).await;
        }
        self.swarm.leader_party_id = leader.to_string();
        self.swarm.follower_index = follower_index;
        self.swarm.formation_as_follower = formation;
        let text = format!(
            "Follow {leader} index:{follower_index} fromation#:{formation} requested by: {requested_by}"
        );
        self.warn_gcs(&text).await;
        if !requested_by.is_empty() && requested_by != leader {
            self.request_to_follow_leader(leader).await;
        }
        self.api_ic_send_id("").await;
    }

    /// CSwarmManager::unFollowLeader
    pub async fn unfollow_leader(&mut self, leader_to_unfollow: &str, requested_by: &str) {
        if !leader_to_unfollow.is_empty() && leader_to_unfollow != self.swarm.leader_party_id {
            return;
        }
        if requested_by != self.swarm.leader_party_id {
            let text = format!(
                "Unfollow {leader_to_unfollow} . Tell it that I am not following it anymore."
            );
            self.warn_gcs(&text).await;
            let l = self.swarm.leader_party_id.clone();
            self.request_unfollow_leader(&l).await;
        }
        self.swarm.leader_party_id.clear();
        self.swarm.follower_index = -1;
        self.swarm.formation_as_follower = swarm::FORMATION_NO_SWARM;
        self.api_ic_send_id("").await;
        let text = format!("Unfollow {leader_to_unfollow} requested by: {requested_by} DONE.");
        self.warn_gcs(&text).await;
    }

    pub async fn release_followers(&mut self) {
        let all: Vec<String> = self.swarm.followers.drain(..).map(|f| f.party_id).collect();
        for p in all {
            self.request_from_unit_to_unfollow_me(&p).await;
        }
    }

    pub async fn release_single_follower(&mut self, party: &str) {
        if party.is_empty() {
            return;
        }
        if let Some(i) = self.swarm.follower_exists(party) {
            self.request_from_unit_to_unfollow_me(party).await;
            self.swarm.followers.remove(i);
        }
    }

    pub async fn add_follower(&mut self, party: &str) {
        let idx = self.swarm.add_follower(party);
        self.request_from_unit_to_follow_me(party, idx).await;
        self.api_ic_p2p_access_mac(party).await;
    }

    async fn warn_gcs(&self, text: &str) {
        self.module
            .send_error_message(
                ANDRUAV_PROTOCOL_SENDER_ALL_GCS,
                0,
                ERROR_TYPE_LO7ETTA7AKOM,
                NOTIFICATION_TYPE_WARNING,
                text,
            )
            .await;
    }

    // ======================= facade (CFCBFacade) ==================================

    fn encode_as_vehicle(&self, msg: &MavMessage) -> Vec<u8> {
        mav::encode(self.vehicle.sysid(), self.vehicle.compid(), msg)
    }

    /// sendMavlinkData_Packed: concatenated frames in one 6502 binary message.
    async fn send_mavlink_packed(&self, target: &str, msgs: &[MavMessage], internal: bool) {
        if msgs.is_empty() {
            return;
        }
        let mut buf = Vec::new();
        for m in msgs {
            buf.extend(self.encode_as_vehicle(m));
        }
        self.module
            .send_bmsg(
                target,
                &buf,
                TYPE_AndruavMessage_MAVLINK,
                internal,
                Value::Null,
            )
            .await;
    }

    /// Generic "send the stored message(s) once if new" (EKF, vibration, wind, ...).
    async fn send_if_unprocessed(&mut self, target: &str, ids: &[u32]) {
        let msgs: Vec<MavMessage> = ids
            .iter()
            .filter_map(|&id| self.vehicle.take_unprocessed(id))
            .collect();
        self.send_mavlink_packed(target, &msgs, false).await;
    }

    /// API_IC_sendID: TYPE_AndruavMessage_ID (intermodule).
    pub async fn api_ic_send_id(&self, target: &str) {
        let i = &self.info;
        let mut arming = 0;
        if i.is_ready_to_arm {
            arming |= 1;
        }
        if i.is_armed {
            arming |= 2;
        }
        let tp = if self.fcb_connected {
            TELEMETRY_PROTOCOL_DRONEKIT_TELEMETRY
        } else {
            TELEMETRY_PROTOCOL_NO_TELEMETRY
        };
        let msg = json!({
            "VT": i.vehicle_type,
            "FM": i.flying_mode,
            "AP": i.autopilot,
            "GM": i.gps_mode,
            "FI": i.use_fcb,
            "AR": arming,
            "FL": i.is_flying,
            "TP": tp,
            "z": i.flying_last_start_time / 1000,
            "a": i.flying_total_duration / 1000,
            "b": i.is_tracking_mode,
            "C": i.rc_sub_action,
            "B": i.is_gcs_blocked,
            "DE": false,             // DE-Pilot not ported (R2b)
            "DO": DEPILOT_OP_DISABLED,
            "n": self.swarm.formation_as_follower,
            "o": self.swarm.formation_as_leader,
            "q": self.swarm.leader_party_id,
        });
        self.module
            .send_jmsg(target, msg, TYPE_AndruavMessage_ID, true)
            .await;
    }

    /// TYPE_AndruavModule_Location_Info (9102)
    async fn send_location_info(&self) {
        let g = self.vehicle.global_position_int();
        let gps = self.vehicle.gps_raw_int();
        let msg = json!({
            "la": g.lat, "ln": g.lon, "a": g.alt, "r": g.relative_alt,
            "ha": gps.h_acc, "y": gps.yaw,
        });
        self.module
            .send_jmsg("", msg, TYPE_AndruavModule_Location_Info, false)
            .await;
    }

    /// sendNavInfo: ATTITUDE + NAV_CONTROLLER_OUTPUT + VFR_HUD, always.
    async fn send_nav_info(&self, target: &str) {
        let msgs: Vec<MavMessage> = [
            vehicle::MSG_ATTITUDE,
            vehicle::MSG_NAV_CONTROLLER_OUTPUT,
            vehicle::MSG_VFR_HUD,
        ]
        .iter()
        .filter_map(|&id| self.vehicle.stored_or_default(id))
        .collect();
        self.send_mavlink_packed(target, &msgs, false).await;
    }

    /// sendGPSInfo: GLOBAL_POSITION_INT, GPS_RAW_INT, GPS2_RAW when new.
    async fn send_gps_info(&mut self, target: &str) {
        self.send_if_unprocessed(
            target,
            &[
                vehicle::MSG_GLOBAL_POSITION_INT,
                vehicle::MSG_GPS_RAW_INT,
                vehicle::MSG_GPS2_RAW,
            ],
        )
        .await;
    }

    /// sendMissionCurrent: MISSION_CURRENT + MISSION_COUNT (waypoint manager values).
    async fn send_mission_current(&mut self, target: &str) {
        self.vehicle.set_processed(44); // C++ marks MISSION_COUNT processed
        let current = self
            .vehicle
            .stored_or_default(vehicle::MSG_MISSION_CURRENT)
            .unwrap_or(MavMessage::MISSION_CURRENT(Default::default()));
        // DIVERGENCE: waypoint manager not ported; MISSION_COUNT is all zero.
        let count = MavMessage::MISSION_COUNT(apm::MISSION_COUNT_DATA::default());
        self.send_mavlink_packed(target, &[current, count], false)
            .await;
    }

    /// TYPE_AndruavMessage_Mission_Item_Sequence (intermodule).
    async fn send_mission_item_sequence(&self, seq: &str) {
        self.module
            .send_jmsg(
                "",
                json!({"s": seq}),
                TYPE_AndruavMessage_Mission_Item_Sequence,
                true,
            )
            .await;
    }

    /// TYPE_AndruavMessage_HomeLocation
    pub async fn send_home_location(&self, target: &str) {
        let h = self.vehicle.home();
        let msg = json!({
            "T": f64::from(h.latitude) / 1e7,
            "O": f64::from(h.longitude) / 1e7,
            // C++: altitude / 1000.0f -> float, then stored as double
            "A": f64::from(h.altitude as f32 / 1000.0f32),
        });
        self.module
            .send_jmsg(target, msg, TYPE_AndruavMessage_HomeLocation, false)
            .await;
    }

    /// HEARTBEAT re-encoded as the vehicle (remote execute MAVLINK/Act=0).
    pub async fn send_heartbeat(&self, target: &str) {
        let hb = MavMessage::HEARTBEAT(self.vehicle.heartbeat().clone());
        self.send_mavlink_packed(target, &[hb], false).await;
    }

    /// TYPE_AndruavMessage_DistinationLocation
    async fn send_fcb_target_location(
        &self,
        target: &str,
        lat: f64,
        lon: f64,
        alt: f64,
        target_type: i32,
    ) {
        let msg = json!({"P": target_type, "T": lat, "O": lon, "A": alt});
        self.module
            .send_jmsg(target, msg, TYPE_AndruavMessage_DistinationLocation, false)
            .await;
    }

    async fn send_sync_fire_event_with(&self, target: &str, event: &str, m: Value, internal: bool) {
        self.module
            .send_jmsg(
                target,
                json!({"d": event, "m": m}),
                TYPE_AndruavMessage_Sync_EventFire,
                internal,
            )
            .await;
    }

    async fn request_to_follow_leader(&self, leader: &str) {
        let msg = json!({"a": SWARM_ADD, "c": leader, "d": self.info.party_id});
        self.module
            .send_jmsg(leader, msg, TYPE_AndruavMessage_UpdateSwarm, false)
            .await;
    }

    async fn request_unfollow_leader(&self, leader: &str) {
        let msg = json!({"a": SWARM_DELETE, "c": leader, "d": self.info.party_id});
        self.module
            .send_jmsg(leader, msg, TYPE_AndruavMessage_UpdateSwarm, false)
            .await;
    }

    pub async fn request_from_unit_to_change_formation(&self, party: &str, idx: i32) {
        self.follow_him(party, idx, SWARM_CHANGE_FORMATION).await;
    }

    async fn request_from_unit_to_follow_me(&self, party: &str, idx: i32) {
        self.follow_him(party, idx, SWARM_FOLLOW).await;
    }

    async fn follow_him(&self, party: &str, idx: i32, action: i32) {
        let msg = json!({
            "a": idx,
            "b": self.info.party_id,
            "c": party,
            "d": self.swarm.formation_as_leader,
            "h": self.swarm.leader_min_h,
            "v": self.swarm.leader_min_v,
            "f": action,
        });
        self.module
            .send_jmsg(party, msg, TYPE_AndruavMessage_FollowHim_Request, false)
            .await;
    }

    async fn request_from_unit_to_unfollow_me(&self, party: &str) {
        let msg = json!({"b": self.info.party_id, "c": party, "f": SWARM_UNFOLLOW});
        self.module
            .send_jmsg(party, msg, TYPE_AndruavMessage_FollowHim_Request, false)
            .await;
    }

    async fn api_ic_p2p_access_mac(&self, party: &str) {
        let msg = json!({"a": P2P_ACTION_ACCESS_TO_MAC, "int_prty": party});
        self.module
            .send_jmsg(party, msg, TYPE_AndruavMessage_P2P_ACTION, true)
            .await;
    }

    async fn request_udp_proxy_telemetry(
        &self,
        enable: bool,
        ip1: &str,
        port1: i32,
        ip2: &str,
        port2: i32,
    ) {
        let msg = json!({
            "en": enable,
            "socket1": {"address": ip1, "port": port1},
            "socket2": {"address": ip2, "port": port2},
        });
        self.module
            .send_sysmsg(msg, TYPE_AndruavSystem_UDPProxy)
            .await;
    }

    async fn send_udp_proxy_status(
        &self,
        target: &str,
        enabled: bool,
        paused: bool,
        ip: &str,
        port: i32,
        level: i32,
    ) {
        let msg = json!({"a": ip, "p": port, "o": level, "en": enabled, "z": paused});
        self.module
            .send_jmsg(target, msg, TYPE_AndruavMessage_UDPProxy_Info, false)
            .await;
    }
}
