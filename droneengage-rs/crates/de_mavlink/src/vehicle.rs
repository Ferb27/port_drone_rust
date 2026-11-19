// Ported from droneengage_mavlink@be5d8a5: mavlink_sdk/src/vehicle.cpp, vehicle.h
//
// Tracks the flight controller's state from incoming MAVLink and reports the
// same events the C++ CVehicle fires through CCallBack_Vehicle. Side effects
// (heartbeat to FC, data-stream requests) are returned as `Action`s so this
// stays a pure, testable state machine.
//
// R2a: heartbeat/arm/fly/mode/home/status-text/system-time/ack and message
// storage. Not yet: parameters, waypoints, high-latency, distance sensors,
// ADS-B callbacks, EKF/vibration change events (stored, not reported).

use std::collections::HashMap;

use mavlink::dialects::ardupilotmega::{self as apm, MavMessage};
use mavlink::{MavHeader, Message};

pub const HEART_BEAT_TIMEOUT_US: u64 = 3_000_000;
const DATA_STREAM_REQUEST_HEARTBEAT_COUNT: u32 = 3;
const DATA_STREAM_REQUEST_TIME_WINDOW_US: u64 = 3_000_000;
const NO_SYSID_RESTRICTION: u32 = 0;

pub const MSG_HEARTBEAT: u32 = 0;
pub const MSG_GLOBAL_POSITION_INT: u32 = 33;
pub const MSG_GPS_RAW_INT: u32 = 24;
pub const MSG_GPS2_RAW: u32 = 124;
pub const MSG_ATTITUDE: u32 = 30;
pub const MSG_NAV_CONTROLLER_OUTPUT: u32 = 62;
pub const MSG_VFR_HUD: u32 = 74;
pub const MSG_BATTERY_STATUS: u32 = 147;
pub const MSG_BATTERY2: u32 = 181;
pub const MSG_EKF_STATUS_REPORT: u32 = 193;
pub const MSG_VIBRATION: u32 = 241;
pub const MSG_WIND: u32 = 168;
pub const MSG_TERRAIN_REPORT: u32 = 136;
pub const MSG_MISSION_CURRENT: u32 = 42;

/// CCallBack_Vehicle events.
#[derive(Debug, Clone, PartialEq)]
pub enum VehicleEvent {
    HeartBeatFirst(apm::HEARTBEAT_DATA),
    HeartBeatResumed(apm::HEARTBEAT_DATA),
    Armed {
        armed: bool,
        ready_to_arm: bool,
    },
    Flying(bool),
    ModeChanged {
        custom_mode: u32,
        mav_type: i32,
        autopilot: u8,
    },
    HomePositionUpdated(apm::HOME_POSITION_DATA),
    StatusText {
        severity: u8,
        text: String,
    },
    BoardRestarted,
    Ack {
        command: u32,
        result: u32,
    },
}

/// Things the vehicle wants sent to the FC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    RequestDataStreams,
    GcsHeartbeat(u8),
}

#[derive(Debug, Default)]
pub struct Vehicle {
    sysid: u8,
    compid: u8,
    restrict_sysid: u32,
    restrict_compid: u32,
    /// -1 disabled, 0 means 255.
    gcs_id: i32,
    heartbeat: apm::HEARTBEAT_DATA,
    heart_beat_first_received: bool,
    armed: bool,
    is_flying: bool,
    sys_status_prearm: bool,
    home: apm::HOME_POSITION_DATA,
    system_time_boot_ms: u32,
    consecutive_heartbeat_count: u32,
    heartbeat_sequence_start: u64,
    /// last decoded message per id
    last: HashMap<u32, MavMessage>,
    /// receive time (monotonic usec) per id
    stamp: HashMap<u32, u64>,
    /// MESSAGE_UNPROCESSED flag per id (true = not yet forwarded)
    unprocessed: HashMap<u32, bool>,
}

impl Vehicle {
    pub fn new() -> Self {
        Self {
            gcs_id: -1,
            ..Default::default()
        }
    }

    pub fn restrict_sysid(&mut self, id: u32) {
        self.restrict_sysid = id;
    }
    pub fn restrict_compid(&mut self, id: u32) {
        self.restrict_compid = id;
    }
    pub fn set_gcs_id(&mut self, id: i32) {
        self.gcs_id = id;
    }

    pub fn sysid(&self) -> u8 {
        self.sysid
    }
    pub fn compid(&self) -> u8 {
        self.compid
    }
    pub fn target(&self) -> (u8, u8) {
        (self.sysid, self.compid)
    }
    pub fn heartbeat(&self) -> &apm::HEARTBEAT_DATA {
        &self.heartbeat
    }
    pub fn home(&self) -> &apm::HOME_POSITION_DATA {
        &self.home
    }

    pub fn is_fcb_connected(&self, now_us: u64) -> bool {
        let t = self.stamp.get(&MSG_HEARTBEAT).copied().unwrap_or(0);
        now_us.saturating_sub(t) <= HEART_BEAT_TIMEOUT_US
    }

    pub fn global_position_int(&self) -> apm::GLOBAL_POSITION_INT_DATA {
        match self.last.get(&MSG_GLOBAL_POSITION_INT) {
            Some(MavMessage::GLOBAL_POSITION_INT(d)) => d.clone(),
            _ => Default::default(),
        }
    }
    pub fn attitude(&self) -> apm::ATTITUDE_DATA {
        match self.last.get(&MSG_ATTITUDE) {
            Some(MavMessage::ATTITUDE(d)) => d.clone(),
            _ => Default::default(),
        }
    }
    pub fn gps_raw_int(&self) -> apm::GPS_RAW_INT_DATA {
        match self.last.get(&MSG_GPS_RAW_INT) {
            Some(MavMessage::GPS_RAW_INT(d)) => d.clone(),
            _ => Default::default(),
        }
    }

    /// Last stored message of an id, or the all-zero default C++ would send.
    pub fn stored_or_default(&self, id: u32) -> Option<MavMessage> {
        if let Some(m) = self.last.get(&id) {
            return Some(m.clone());
        }
        MavMessage::default_message_from_id(id)
    }

    /// getProcessedFlag == MESSAGE_UNPROCESSED; clears it (setProcessedFlag PROCESSED).
    pub fn take_unprocessed(&mut self, id: u32) -> Option<MavMessage> {
        if self.unprocessed.get(&id).copied().unwrap_or(false) {
            self.unprocessed.insert(id, false);
            self.last.get(&id).cloned()
        } else {
            None
        }
    }

    pub fn set_processed(&mut self, id: u32) {
        self.unprocessed.insert(id, false);
    }

    /// CVehicle::parseMessage. Returns (processed_by_vehicle, events, actions).
    pub fn parse(
        &mut self,
        header: &MavHeader,
        msg: &MavMessage,
        now_us: u64,
    ) -> (bool, Vec<VehicleEvent>, Vec<Action>) {
        let id = msg.message_id();
        let mut ev = Vec::new();
        let mut act = Vec::new();
        let ids_known = self.sysid != 0 && self.compid != 0;
        let from_vehicle = ids_known
            && header.system_id == u32::from(self.sysid)
            && header.component_id == self.compid;
        if from_vehicle && id != MSG_HEARTBEAT {
            self.consecutive_heartbeat_count = 0;
            self.heartbeat_sequence_start = 0;
        }
        let mut processed = true;
        match msg {
            MavMessage::HEARTBEAT(hb) => {
                if self.handle_heartbeat(header, hb, now_us, &mut ev, &mut act) {
                    self.stamp.insert(MSG_HEARTBEAT, now_us);
                } else {
                    processed = false;
                }
                // heartbeat time stamp is only set when it is accepted
                return (processed, ev, act);
            }
            MavMessage::SYS_STATUS(s) => {
                let prearm = s
                    .onboard_control_sensors_health
                    .contains(apm::MavSysStatusSensor::MAV_SYS_STATUS_PREARM_CHECK);
                if prearm != self.sys_status_prearm {
                    self.sys_status_prearm = prearm;
                    ev.push(VehicleEvent::Armed {
                        armed: self.armed,
                        ready_to_arm: prearm,
                    });
                }
            }
            MavMessage::SYSTEM_TIME(t) => {
                if t.time_boot_ms < self.system_time_boot_ms && self.system_time_boot_ms > 0 {
                    ev.push(VehicleEvent::BoardRestarted);
                }
                self.system_time_boot_ms = t.time_boot_ms;
            }
            MavMessage::GLOBAL_POSITION_INT(g) => {
                let mut g = g.clone();
                if self.heartbeat.mavtype == apm::MavType::MAV_TYPE_GROUND_ROVER {
                    g.relative_alt = 0;
                }
                self.last.insert(id, MavMessage::GLOBAL_POSITION_INT(g));
            }
            MavMessage::HOME_POSITION(h) => {
                let changed = self.home.latitude != h.latitude
                    || self.home.longitude != h.longitude
                    || self.home.altitude != h.altitude;
                self.home = h.clone();
                if changed {
                    ev.push(VehicleEvent::HomePositionUpdated(h.clone()));
                }
            }
            MavMessage::STATUSTEXT(s) => {
                let bytes: Vec<u8> = s.text.iter().copied().take_while(|&b| b != 0).collect();
                ev.push(VehicleEvent::StatusText {
                    severity: s.severity as u8,
                    text: String::from_utf8_lossy(&bytes).into_owned(),
                });
            }
            MavMessage::COMMAND_ACK(a) => ev.push(VehicleEvent::Ack {
                command: a.command as u32,
                result: a.result as u32,
            }),
            _ => {}
        }
        if !matches!(msg, MavMessage::GLOBAL_POSITION_INT(_)) {
            self.last.insert(id, msg.clone());
        }
        self.stamp.insert(id, now_us);
        self.unprocessed.insert(id, true);
        (processed, ev, act)
    }

    fn handle_heartbeat(
        &mut self,
        header: &MavHeader,
        hb: &apm::HEARTBEAT_DATA,
        now_us: u64,
        ev: &mut Vec<VehicleEvent>,
        act: &mut Vec<Action>,
    ) -> bool {
        if self.restrict_sysid == NO_SYSID_RESTRICTION {
            let t = hb.mavtype as u32;
            if t >= apm::MavType::MAV_TYPE_GIMBAL as u32
                || hb.mavtype == apm::MavType::MAV_TYPE_GCS
                || hb.mavtype == apm::MavType::MAV_TYPE_ONBOARD_CONTROLLER
                || hb.autopilot == apm::MavAutopilot::MAV_AUTOPILOT_INVALID
            {
                return false;
            }
        } else if self.restrict_sysid != header.system_id {
            return false;
        }
        let expected_comp = if self.restrict_compid == NO_SYSID_RESTRICTION {
            1
        } else {
            self.restrict_compid as u8
        };
        if header.component_id != expected_comp {
            return false;
        }
        self.sysid = header.system_id as u8;
        self.compid = header.component_id;

        let is_armed = hb
            .base_mode
            .contains(apm::MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED);
        let mode_changed = self.heartbeat.custom_mode != hb.custom_mode
            || self.heartbeat.mavtype != hb.mavtype
            || self.heartbeat.autopilot != hb.autopilot
            || !self.heart_beat_first_received;
        let prev_system_status = self.heartbeat.system_status;
        self.heartbeat = hb.clone();

        self.consecutive_heartbeat_count += 1;
        if self.consecutive_heartbeat_count == 1 {
            self.heartbeat_sequence_start = now_us;
        }
        if self.consecutive_heartbeat_count >= DATA_STREAM_REQUEST_HEARTBEAT_COUNT
            && now_us.saturating_sub(self.heartbeat_sequence_start)
                >= DATA_STREAM_REQUEST_TIME_WINDOW_US
        {
            // only heartbeats arrive: the FC stopped streaming, ask again
            act.push(Action::RequestDataStreams);
            self.consecutive_heartbeat_count = 0;
            self.heartbeat_sequence_start = now_us;
        }

        let last_hb = self.stamp.get(&MSG_HEARTBEAT).copied().unwrap_or(0);
        if !self.heart_beat_first_received {
            self.heart_beat_first_received = true;
            ev.push(VehicleEvent::HeartBeatFirst(hb.clone()));
            act.push(Action::RequestDataStreams);
        } else if now_us.saturating_sub(last_hb) > HEART_BEAT_TIMEOUT_US {
            ev.push(VehicleEvent::HeartBeatResumed(hb.clone()));
            act.push(Action::RequestDataStreams);
        }

        if self.armed != is_armed {
            self.armed = is_armed;
            ev.push(VehicleEvent::Armed {
                armed: is_armed,
                ready_to_arm: self.sys_status_prearm,
            });
        }
        if hb.autopilot != apm::MavAutopilot::MAV_AUTOPILOT_PX4 {
            // C++ checks the *previous* heartbeat for EMERGENCY (after the
            // assignment it is the same one); kept as written.
            let _ = prev_system_status;
            let is_flying = is_armed
                && (hb.system_status == apm::MavState::MAV_STATE_ACTIVE
                    || hb.system_status == apm::MavState::MAV_STATE_CRITICAL
                    || self.heartbeat.system_status == apm::MavState::MAV_STATE_EMERGENCY);
            if self.is_flying != is_flying {
                self.is_flying = is_flying;
                ev.push(VehicleEvent::Flying(is_flying));
            }
        }
        if mode_changed {
            ev.push(VehicleEvent::ModeChanged {
                custom_mode: hb.custom_mode,
                mav_type: hb.mavtype as i32,
                autopilot: hb.autopilot as u8,
            });
        }
        if self.gcs_id != -1 {
            let id = if self.gcs_id == 0 {
                255
            } else {
                self.gcs_id as u8
            };
            act.push(Action::GcsHeartbeat(id));
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hdr(sys: u8, comp: u8) -> MavHeader {
        MavHeader {
            system_id: u32::from(sys),
            component_id: comp,
            sequence: 0,
        }
    }

    fn copter_hb(armed: bool, mode: u32) -> MavMessage {
        let mut base = apm::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED;
        if armed {
            base |= apm::MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED;
        }
        MavMessage::HEARTBEAT(apm::HEARTBEAT_DATA {
            custom_mode: mode,
            mavtype: apm::MavType::MAV_TYPE_QUADROTOR,
            autopilot: apm::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
            base_mode: base,
            system_status: if armed {
                apm::MavState::MAV_STATE_ACTIVE
            } else {
                apm::MavState::MAV_STATE_STANDBY
            },
            mavlink_version: 3,
        })
    }

    #[test]
    fn first_heartbeat_then_arm_fly_mode() {
        let mut v = Vehicle::new();
        v.set_gcs_id(255);
        let (ok, ev, act) = v.parse(&hdr(2, 1), &copter_hb(false, 0), 1_000_000);
        assert!(ok);
        assert!(matches!(ev[0], VehicleEvent::HeartBeatFirst(_)));
        assert!(ev
            .iter()
            .any(|e| matches!(e, VehicleEvent::ModeChanged { custom_mode: 0, .. })));
        assert!(
            act.contains(&Action::RequestDataStreams) && act.contains(&Action::GcsHeartbeat(255))
        );
        assert_eq!(v.target(), (2, 1));

        let (_, ev, _) = v.parse(&hdr(2, 1), &copter_hb(true, 4), 2_000_000);
        assert!(ev.contains(&VehicleEvent::Armed {
            armed: true,
            ready_to_arm: false
        }));
        assert!(ev.contains(&VehicleEvent::Flying(true)));
        assert!(ev
            .iter()
            .any(|e| matches!(e, VehicleEvent::ModeChanged { custom_mode: 4, .. })));
        assert!(v.is_fcb_connected(2_500_000));
        assert!(!v.is_fcb_connected(6_000_000));
    }

    #[test]
    fn gcs_and_other_components_ignored() {
        let mut v = Vehicle::new();
        let gcs = MavMessage::HEARTBEAT(apm::HEARTBEAT_DATA {
            mavtype: apm::MavType::MAV_TYPE_GCS,
            ..Default::default()
        });
        assert!(!v.parse(&hdr(255, 0), &gcs, 1).0);
        assert!(!v.parse(&hdr(1, 100), &copter_hb(false, 0), 1).0);
        assert_eq!(v.sysid(), 0);
    }

    #[test]
    fn unprocessed_flags() {
        let mut v = Vehicle::new();
        let g = MavMessage::GLOBAL_POSITION_INT(apm::GLOBAL_POSITION_INT_DATA {
            lat: 5,
            ..Default::default()
        });
        v.parse(&hdr(1, 1), &g, 1);
        assert!(v.take_unprocessed(MSG_GLOBAL_POSITION_INT).is_some());
        assert!(v.take_unprocessed(MSG_GLOBAL_POSITION_INT).is_none());
        assert_eq!(v.global_position_int().lat, 5);
    }
}
