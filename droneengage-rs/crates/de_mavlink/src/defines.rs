// Ported from droneengage_mavlink@be5d8a5: src/defines.hpp

pub const TELEMETRY_PROTOCOL_NO_TELEMETRY: i32 = 0;
pub const TELEMETRY_PROTOCOL_DRONEKIT_TELEMETRY: i32 = 4;

/// ANDRUAV_UNIT_TYPE (full C++ table)
#[allow(dead_code)]
pub mod vehicle_type {
    pub const UNKNOWN: i32 = 0;
    pub const TRI: i32 = 1;
    pub const QUAD: i32 = 2;
    pub const PLANE: i32 = 3;
    pub const ROVER: i32 = 4;
    pub const HELI: i32 = 5;
    pub const BOAT: i32 = 6;
    pub const SUBMARINE: i32 = 12;
    pub const GIMBAL: i32 = 15;
    pub const VTOL: i32 = 16;
}

/// ANDRUAV_UNIT_MODE (full C++ table)
#[allow(dead_code)]
pub mod mode {
    pub const RTL: i32 = 2;
    pub const FOLLOW_ME: i32 = 3;
    pub const AUTO: i32 = 5;
    pub const STABILIZE: i32 = 6;
    pub const ALT_HOLD: i32 = 7;
    pub const MANUAL: i32 = 8;
    pub const GUIDED: i32 = 9;
    pub const GUIDED_NO_GPS: i32 = 1000;
    pub const LOITER: i32 = 10;
    pub const POS_HOLD: i32 = 11;
    pub const LAND: i32 = 12;
    pub const CIRCLE: i32 = 13;
    pub const BRAKE: i32 = 17;
    pub const SMART_RTL: i32 = 21;
    pub const ACRO: i32 = 28;
    pub const INITALIZING: i32 = 99;
    pub const UNKNOWN: i32 = 999;
}

/// RC_SUB_ACTION
pub const RC_SUB_ACTION_RELEASED: i32 = 0;

/// DRONEENGAGE_PILOT_OPERATION
pub const DEPILOT_OP_DISABLED: i32 = 0;

/// ANDRUAV_VEHICLE_INFO (the fields the ported code uses).
#[allow(dead_code)] // current_waypoint is used by missions (R2b)
#[derive(Debug, Clone)]
pub struct VehicleInfo {
    pub party_id: String,
    pub group_id: String,
    pub use_fcb: bool,
    pub is_armed: bool,
    pub is_ready_to_arm: bool,
    pub is_flying: bool,
    pub is_tracking_mode: bool,
    pub is_gcs_blocked: bool,
    pub flying_mode: i32,
    pub gps_mode: i32,
    /// usec
    pub flying_total_duration: u64,
    /// usec
    pub flying_last_start_time: u64,
    pub vehicle_type: i32,
    pub autopilot: u8,
    pub current_waypoint: i32,
    pub rc_sub_action: i32,
}

impl Default for VehicleInfo {
    fn default() -> Self {
        Self {
            party_id: String::new(),
            group_id: String::new(),
            use_fcb: false,
            is_armed: false,
            is_ready_to_arm: false,
            is_flying: false,
            is_tracking_mode: false,
            is_gcs_blocked: false,
            // C++ leaves it uninitialised in a zeroed singleton.
            flying_mode: 0,
            gps_mode: 0,
            flying_total_duration: 0,
            flying_last_start_time: 0,
            vehicle_type: 0,
            autopilot: 0, // MAV_AUTOPILOT_GENERIC
            current_waypoint: 0,
            rc_sub_action: RC_SUB_ACTION_RELEASED,
        }
    }
}
