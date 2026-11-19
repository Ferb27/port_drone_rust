// Ported from droneengage_common@5fb11c2: de_databus/sync_fire_events.hpp
//! Event names carried in TYPE_AndruavMessage_Sync_EventFire ("d" field).

pub const DRONE_FCB_RESTARTED: &str = "d_f_r";
pub const DRONE_ARMED: &str = "d_f_a";
pub const DRONE_DISARMED: &str = "d_f_d";
pub const DRONE_FLYING: &str = "d_f_f";
pub const DRONE_LANDED: &str = "d_f_l";
pub const DRONE_MODE_ALT_HOLD: &str = "d_f_ah";
pub const DRONE_MODE_GUIDED: &str = "d_f_gdd";
pub const DRONE_MODE_GUIDED_NO_GPS: &str = "d_f_gdd_ng";
pub const DRONE_MODE_AUTO: &str = "d_f_auto";
pub const DRONE_MODE_RTL: &str = "d_f_rtl";
pub const DRONE_MODE_STABILIZE: &str = "d_f_stb";
pub const DRONE_MODE_LOITER: &str = "d_f_loiter";
pub const DRONE_MODE_POSHOLD: &str = "d_f_ph";
pub const DRONE_CONNECTED: &str = "d_f_c";
pub const DRONE_DISCONNECTED: &str = "d_f_dc";
pub const DRONE_HOME_SET: &str = "d_f_hs";
pub const DRONE_MISSION_STARTED: &str = "d_f_ms";
pub const DRONE_MISSION_COMPLETED: &str = "d_f_mc";
pub const DRONE_DEPILOT_MODE_ACTIVATED: &str = "d_f_de_a";
pub const DRONE_DEPILOT_SUBOPERATION_UPDATED: &str = "d_f_de_s";
pub const DRONE_FENCE_ACTION: &str = "onFenceAction";
pub const DRONE_COMM_LINE_ONLINE: &str = "d_cl_on";
pub const DRONE_COMM_LINE_OFFLINE_REQUEST: &str = "d_cl_off_r";
pub const DRONE_COMM_LINE_OFFLINE_ERROR: &str = "d_cl_off_e";
