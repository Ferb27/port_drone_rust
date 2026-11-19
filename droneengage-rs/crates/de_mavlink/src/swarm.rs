// Ported from droneengage_mavlink@be5d8a5: src/swarm/fcb_swarm_manager.cpp, .hpp,
// fcb_swarm_leader.cpp, fcb_swarm_follower.cpp
//
// Pure state + geometry. Message sending lives in fcb.rs; the functions here
// return what should be sent so they can be tested without a databus.

use std::f64::consts::PI;

use crate::gps::{bearing_of_vector, calc_gps_distance, calculate_bearing, point_at_bearing};
use mavlink::dialects::ardupilotmega as apm;

pub const KNODE_LENGTH: i32 = 100;

pub const FORMATION_NO_SWARM: i32 = 0;
pub const FORMATION_THREAD: i32 = 1;
pub const FORMATION_ARROW: i32 = 2;
#[allow(dead_code)] // no follower logic for VECTOR in C++ either
pub const FORMATION_VECTOR: i32 = 3;
pub const FORMATION_ARROW_DYNAMIC: i32 = 4;

/// Leader -> followers update period (DEF_SWARM_LEADER_LOCATION_UPDATE_RATE).
pub const LEADER_UPDATE_US: u64 = 1_000_000;

const SPEED_STATIONARY_THRESHOLD_SQ: f64 = 2500.0;
const MAX_BEARING_SLEW_RATE_DEG: f64 = 30.0;
const TARGET_SMOOTHING_ALPHA: f64 = 0.3;
const HDG_DEADZONE_DEG: f64 = 5.0;
const TARGET_JUMP_THRESHOLD_M: f64 = 5.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Follower {
    pub party_id: String,
    pub follower_index: i32,
}

/// CSwarmManager + CSwarmLeader distances + CSwarmFollower filter state.
#[derive(Debug, Clone)]
pub struct Swarm {
    pub formation_as_leader: i32,
    pub is_leader: bool,
    pub followers: Vec<Follower>,
    pub leader_party_id: String,
    pub follower_index: i32,
    pub formation_as_follower: i32,
    // CSwarmLeader
    pub leader_min_h: i32,
    pub leader_min_v: i32,
    pub leader_last_update_us: u64,
    // CSwarmFollower
    pub follower_min_h: i32,
    pub follower_min_v: i32,
    pub follower: FollowerFilter,
}

impl Default for Swarm {
    fn default() -> Self {
        Self {
            formation_as_leader: FORMATION_NO_SWARM,
            is_leader: false,
            followers: Vec::new(),
            leader_party_id: String::new(),
            follower_index: -1,
            formation_as_follower: FORMATION_NO_SWARM,
            leader_min_h: KNODE_LENGTH,
            leader_min_v: KNODE_LENGTH,
            leader_last_update_us: 0,
            follower_min_h: KNODE_LENGTH,
            follower_min_v: KNODE_LENGTH,
            follower: FollowerFilter::default(),
        }
    }
}

impl Swarm {
    pub fn is_follower(&self) -> bool {
        self.follower_index != -1
    }

    pub fn is_my_leader(&self, party: &str) -> bool {
        self.leader_party_id == party
    }

    pub fn follower_exists(&self, party: &str) -> Option<usize> {
        self.followers.iter().position(|f| f.party_id == party)
    }

    /// Smallest free index (C++ insertFollowerInSwarmFormation).
    pub fn insert_follower(&mut self, party: &str) -> i32 {
        let mut used: Vec<i32> = self.followers.iter().map(|f| f.follower_index).collect();
        used.sort_unstable();
        let mut idx = 0;
        for u in used {
            if u == idx {
                idx += 1;
            }
        }
        self.followers.push(Follower {
            party_id: party.to_string(),
            follower_index: idx,
        });
        idx
    }

    /// Index to announce to `party` (existing or newly inserted).
    pub fn add_follower(&mut self, party: &str) -> i32 {
        match self.follower_exists(party) {
            Some(i) => self.followers[i].follower_index,
            None => self.insert_follower(party),
        }
    }
}

/// A guided target the follower should fly to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FollowerTarget {
    pub latitude: f64,
    pub longitude: f64,
    /// metres, relative
    pub altitude: f64,
    /// leader relative_alt in mm, reported back in TYPE_AndruavMessage_DistinationLocation
    pub leader_relative_alt_mm: f64,
}

#[derive(Debug, Clone, Default)]
pub struct FollowerFilter {
    pub leader_gpos: apm::GLOBAL_POSITION_INT_DATA,
    pub leader_yaw: f32,
    pub leader_yawspeed: f32,
    last_leader_bearing: f64,
    has_last_bearing: bool,
    smoothed_lat: f64,
    smoothed_lon: f64,
    has_smoothed: bool,
    prev_raw_lat: f64,
    prev_raw_lon: f64,
    has_prev_raw: bool,
    consecutive_skips: i32,
}

fn smooth(f: &mut FollowerFilter, lat: f64, lon: f64) {
    if f.has_smoothed {
        f.smoothed_lat =
            TARGET_SMOOTHING_ALPHA * lat + (1.0 - TARGET_SMOOTHING_ALPHA) * f.smoothed_lat;
        f.smoothed_lon =
            TARGET_SMOOTHING_ALPHA * lon + (1.0 - TARGET_SMOOTHING_ALPHA) * f.smoothed_lon;
    } else {
        f.smoothed_lat = lat;
        f.smoothed_lon = lon;
        f.has_smoothed = true;
    }
}

fn wrap_pi(mut a: f64) -> f64 {
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

fn target_from(f: &FollowerFilter, follower_index: i32, min_v: i32) -> FollowerTarget {
    let rel = f.leader_gpos.relative_alt;
    FollowerTarget {
        latitude: f.smoothed_lat,
        longitude: f.smoothed_lon,
        // C++: (int relative_alt + (idx+1) * min_v * 1000) / 1000.0f  (int arithmetic, then float)
        altitude: ((rel + (follower_index + 1) * min_v * 1000) as f32 / 1000.0f32) as f64,
        leader_relative_alt_mm: f64::from(rel),
    }
}

/// CSwarmFollower::updateFollowerInThreadFormation
pub fn update_thread(
    f: &mut FollowerFilter,
    me: &apm::GLOBAL_POSITION_INT_DATA,
    follower_index: i32,
    min_h: i32,
    min_v: i32,
) -> Option<FollowerTarget> {
    let g = &f.leader_gpos;
    let (llat, llon) = (f64::from(g.lat) / 1e7, f64::from(g.lon) / 1e7);
    let speed_sq = f64::from(i32::from(g.vx) * i32::from(g.vx) + i32::from(g.vy) * i32::from(g.vy));
    if speed_sq < SPEED_STATIONARY_THRESHOLD_SQ {
        return None;
    }
    let (my_lat, my_lon) = (f64::from(me.lat) / 1e7, f64::from(me.lon) / 1e7);
    let base = f64::from((follower_index + 1) * min_h);
    if calc_gps_distance(llat, llon, my_lat, my_lon) < base {
        return None;
    }
    let bearing = calculate_bearing(llat, llon, my_lat, my_lon);
    let p = point_at_bearing(llat, llon, bearing, base);
    smooth(f, p.latitude, p.longitude);
    Some(target_from(f, follower_index, min_v))
}

/// CSwarmFollower::updateFollowerInArrowFormation
#[allow(clippy::needless_late_init)] // keeps the C++ branch structure readable
pub fn update_arrow(
    f: &mut FollowerFilter,
    me: &apm::GLOBAL_POSITION_INT_DATA,
    follower_index: i32,
    min_h: i32,
    min_v: i32,
    is_dynamic: bool,
) -> Option<FollowerTarget> {
    let g = f.leader_gpos.clone();
    let (llat, llon) = (f64::from(g.lat) / 1e7, f64::from(g.lon) / 1e7);
    let speed_sq = f64::from(i32::from(g.vx) * i32::from(g.vx) + i32::from(g.vy) * i32::from(g.vy));
    let formation_bearing;
    if speed_sq >= SPEED_STATIONARY_THRESHOLD_SQ {
        let mut b = bearing_of_vector(f64::from(g.vx), f64::from(g.vy));
        if f.has_last_bearing {
            let diff = wrap_pi(b - f.last_leader_bearing);
            let max_slew = MAX_BEARING_SLEW_RATE_DEG * PI / 180.0;
            if diff.abs() > max_slew {
                b = f.last_leader_bearing + if diff > 0.0 { max_slew } else { -max_slew };
            }
        }
        f.last_leader_bearing = b;
        f.has_last_bearing = true;
        formation_bearing = b;
    } else if is_dynamic {
        if !f.has_last_bearing {
            return None;
        }
        formation_bearing = f.last_leader_bearing;
    } else if g.hdg != 65535 && g.hdg < 36000 {
        let hdg_rad = f64::from(g.hdg) / 100.0 * PI / 180.0;
        if f.has_last_bearing {
            if wrap_pi(hdg_rad - f.last_leader_bearing).abs() > HDG_DEADZONE_DEG * PI / 180.0 {
                f.last_leader_bearing = hdg_rad;
            }
        } else {
            f.last_leader_bearing = hdg_rad;
            f.has_last_bearing = true;
        }
        formation_bearing = f.last_leader_bearing;
    } else {
        if !f.has_last_bearing {
            return None;
        }
        formation_bearing = f.last_leader_bearing;
    }

    let (my_lat, my_lon) = (f64::from(me.lat) / 1e7, f64::from(me.lon) / 1e7);
    let is_left = follower_index % 2 == 0;
    let base = f64::from((follower_index / 2 + 1) * min_h);
    let angle_offset = PI + if is_left { -PI / 4.0 } else { PI / 4.0 };
    let p = point_at_bearing(llat, llon, formation_bearing + angle_offset, base);

    let cos_lat = (llat * PI / 180.0).cos();
    if f.has_prev_raw {
        let jdx = (p.latitude - f.prev_raw_lat) * 111000.0;
        let jdy = (p.longitude - f.prev_raw_lon) * 111000.0 * cos_lat;
        let jump = (jdx * jdx + jdy * jdy).sqrt();
        if jump > TARGET_JUMP_THRESHOLD_M && f.consecutive_skips < 2 {
            // C++ stores these two in float
            let tdx = ((my_lat - f.prev_raw_lat) * 111000.0) as f32;
            let tdy = ((my_lon - f.prev_raw_lon) * 111000.0 * cos_lat) as f32;
            let dot = (jdx * f64::from(tdx) + jdy * f64::from(tdy)) as f32;
            if dot < 0.0 {
                f.prev_raw_lat = p.latitude;
                f.prev_raw_lon = p.longitude;
                f.consecutive_skips += 1;
                return None;
            }
        }
    }
    f.prev_raw_lat = p.latitude;
    f.prev_raw_lon = p.longitude;
    f.has_prev_raw = true;
    f.consecutive_skips = 0;
    smooth(f, p.latitude, p.longitude);
    Some(target_from(f, follower_index, min_v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_fill_gaps() {
        let mut s = Swarm::default();
        assert_eq!(s.add_follower("a"), 0);
        assert_eq!(s.add_follower("b"), 1);
        assert_eq!(s.add_follower("a"), 0);
        s.followers.remove(0);
        assert_eq!(s.add_follower("c"), 0);
    }

    #[test]
    fn arrow_target_behind_left_of_northbound_leader() {
        let mut f = FollowerFilter {
            leader_gpos: apm::GLOBAL_POSITION_INT_DATA {
                lat: -353632610,
                lon: 1491652300,
                relative_alt: 15000,
                vx: 480, // 4.8 m/s north
                ..Default::default()
            },
            ..Default::default()
        };
        let me = apm::GLOBAL_POSITION_INT_DATA {
            lat: -353633610,
            lon: 1491651300,
            ..Default::default()
        };
        let t = update_arrow(&mut f, &me, 0, 15, 0, false).unwrap();
        // index 0 ("left" in C++) = bearing + 135 deg: south-east of a northbound
        // leader, 15 m away (same convention as tools/sitl_gate/analyze.py)
        assert!(t.latitude < -35.363261 && t.longitude > 149.165230);
        let d = calc_gps_distance(t.latitude, t.longitude, -35.363261, 149.165230);
        assert!((d - 15.0).abs() < 0.01, "{d}");
        assert_eq!(t.altitude, 15.0);
        assert_eq!(t.leader_relative_alt_mm, 15000.0);
    }

    #[test]
    fn stationary_leader_without_heading_gives_no_target() {
        let mut f = FollowerFilter {
            leader_gpos: apm::GLOBAL_POSITION_INT_DATA {
                hdg: 65535,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(update_arrow(&mut f, &Default::default(), 0, 15, 0, false).is_none());
        assert!(update_thread(&mut f, &Default::default(), 0, 15, 0).is_none());
    }
}
