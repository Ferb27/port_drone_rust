// Ported from droneengage_mavlink@be5d8a5: src/helpers/gps.cpp (the functions swarm uses)

const PI: f64 = std::f64::consts::PI;
const RADIO_TERRESTRE: f64 = 6_372_797.560_85;
// C++ uses macros `x * PI / 180.0f` and `x * 180.0f / PI`, evaluated left to
// right. Mirror that order so results match to the last bit.
fn d2r(x: f64) -> f64 {
    x * PI / 180.0
}
fn r2d(x: f64) -> f64 {
    x * 180.0 / PI
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub latitude: f64,
    pub longitude: f64,
}

/// Haversine distance in metres.
pub fn calc_gps_distance(lat_new: f64, lon_new: f64, lat_old: f64, lon_old: f64) -> f64 {
    let l_new = d2r(lat_old); // names swapped exactly like C++ (harmless, symmetric)
    let l_old = d2r(lat_new);
    let dlat = d2r(lat_new - lat_old);
    let dlon = d2r(lon_new - lon_old);
    let a = (dlat / 2.0).sin().powi(2) + l_new.cos() * l_old.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    RADIO_TERRESTRE * c
}

/// Destination point from (lat1, lon1), bearing in radians, distance in metres.
pub fn point_at_bearing(lat1: f64, lon1: f64, bearing_rad: f64, distance_m: f64) -> Point2D {
    let lat1r = d2r(lat1);
    let lon1r = d2r(lon1);
    let d = distance_m / RADIO_TERRESTRE;
    let lat2 = (lat1r.sin() * d.cos() + lat1r.cos() * d.sin() * bearing_rad.cos()).asin();
    let lon2 = lon1r
        + (bearing_rad.sin() * d.sin() * lat1r.cos()).atan2(d.cos() - lat1r.sin() * lat2.sin());
    Point2D {
        latitude: r2d(lat2),
        longitude: r2d(lon2),
    }
}

/// atan2(vy, vx) of a NED velocity (vx north, vy east): bearing in radians.
pub fn bearing_of_vector(vx: f64, vy: f64) -> f64 {
    let m = (vx * vx + vy * vy).sqrt();
    if m == 0.0 {
        return 0.0;
    }
    (vy / m).atan2(vx / m)
}

/// Initial bearing (radians) from point 1 to point 2.
pub fn calculate_bearing(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (d2r(lat1), d2r(lat2));
    let dl = d2r(lon2 - lon1);
    let y = dl.sin() * p2.cos();
    let x = p1.cos() * p2.sin() - p1.sin() * p2.cos() * dl.cos();
    y.atan2(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_point_and_distance() {
        let p = point_at_bearing(-35.363261, 149.165230, PI / 2.0, 100.0);
        let d = calc_gps_distance(p.latitude, p.longitude, -35.363261, 149.165230);
        assert!((d - 100.0).abs() < 0.01, "{d}");
        let b = calculate_bearing(-35.363261, 149.165230, p.latitude, p.longitude);
        assert!((b - PI / 2.0).abs() < 1e-3, "{b}");
        assert!((bearing_of_vector(0.0, 5.0) - PI / 2.0).abs() < 1e-12);
    }
}
