// Ported from droneengage_mavlink@be5d8a5: src/fcb_modes.cpp (vehicle type, ArduCopter modes)
//
// R2a: only the ArduCopter custom-mode table is ported. Plane/Rover/Sub/PX4
// return VEHICLE_MODE_UNKNOWN until R2b.

use crate::defines::{mode, vehicle_type};

// MAV_TYPE values
const MAV_TYPE_FIXED_WING: i32 = 1;
const MAV_TYPE_QUADROTOR: i32 = 2;
const MAV_TYPE_COAXIAL: i32 = 3;
const MAV_TYPE_HELICOPTER: i32 = 4;
const MAV_TYPE_GROUND_ROVER: i32 = 10;
const MAV_TYPE_SURFACE_BOAT: i32 = 11;
const MAV_TYPE_SUBMARINE: i32 = 12;
const MAV_TYPE_HEXAROTOR: i32 = 13;
const MAV_TYPE_OCTOROTOR: i32 = 14;
const MAV_TYPE_TRICOPTER: i32 = 15;
const MAV_TYPE_FLAPPING_WING: i32 = 16;
const MAV_TYPE_VTOL_DUOROTOR: i32 = 19;
const MAV_TYPE_VTOL_RESERVED5: i32 = 25;

pub fn andruav_vehicle_type(mav_type: i32) -> i32 {
    match mav_type {
        MAV_TYPE_QUADROTOR | MAV_TYPE_HEXAROTOR | MAV_TYPE_OCTOROTOR => vehicle_type::QUAD,
        MAV_TYPE_TRICOPTER => vehicle_type::TRI,
        MAV_TYPE_COAXIAL | MAV_TYPE_HELICOPTER => vehicle_type::HELI,
        MAV_TYPE_GROUND_ROVER => vehicle_type::ROVER,
        MAV_TYPE_SURFACE_BOAT => vehicle_type::BOAT,
        MAV_TYPE_SUBMARINE => vehicle_type::SUBMARINE,
        MAV_TYPE_VTOL_DUOROTOR..=MAV_TYPE_VTOL_RESERVED5 => vehicle_type::VTOL,
        MAV_TYPE_FLAPPING_WING | MAV_TYPE_FIXED_WING => vehicle_type::PLANE,
        _ => vehicle_type::UNKNOWN,
    }
}

/// CFCBModes::getAndruavMode, ArduCopter-family branch only.
pub fn andruav_mode(custom_mode: u32, andruav_type: i32, _autopilot: u8) -> i32 {
    match andruav_type {
        vehicle_type::TRI | vehicle_type::QUAD | vehicle_type::HELI => copter_mode(custom_mode),
        _ => mode::UNKNOWN,
    }
}

fn copter_mode(m: u32) -> i32 {
    match m {
        0 => mode::STABILIZE,
        1 => mode::ACRO,
        2 => mode::ALT_HOLD,
        3 => mode::AUTO,
        4 => mode::GUIDED,
        5 => mode::LOITER,
        6 => mode::RTL,
        7 => mode::CIRCLE,
        9 => mode::LAND,
        16 => mode::POS_HOLD,
        17 => mode::BRAKE,
        20 => mode::GUIDED_NO_GPS,
        21 => mode::SMART_RTL,
        23 => mode::FOLLOW_ME,
        25 => mode::INITALIZING, // SYSTEMID, as in C++
        _ => mode::UNKNOWN,      // DRIFT, SPORT, FLIP, AUTOTUNE, THROW, ...
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copter_guided() {
        assert_eq!(andruav_vehicle_type(2), vehicle_type::QUAD);
        assert_eq!(andruav_mode(4, vehicle_type::QUAD, 3), mode::GUIDED);
        assert_eq!(andruav_mode(0, vehicle_type::QUAD, 3), mode::STABILIZE);
        assert_eq!(andruav_vehicle_type(21), vehicle_type::VTOL);
    }
}
