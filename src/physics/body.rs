pub struct CelestialBody {
    pub name: &'static str,
    pub radius_m: f64,
    pub gravitational_parameter: f64, // mu = GM, m^3/s^2
}

impl CelestialBody {
    pub fn earth() -> Self {
        CelestialBody {
            name: "Earth",
            radius_m: 6_378_137.0,
            gravitational_parameter: 3.986_004_418e14,
        }
    }

    pub fn moon() -> Self {
        CelestialBody {
            name: "Moon",
            radius_m: 1_738_090.0,
            gravitational_parameter: 4.902_800_118e12,
        }
    }

    pub fn mars() -> Self {
        CelestialBody {
            name: "Mars",
            radius_m: 3_396_190.0,
            gravitational_parameter: 4.282_837e13,
        }
    }

    pub fn sun() -> Self {
        CelestialBody {
            name: "Sun",
            radius_m: 6.957e8,
            gravitational_parameter: 1.327_124_400_42e20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // surface gravity = mu/r^2, cross-checked against commonly cited values
    #[test]
    fn moon_surface_gravity_matches_known_value() {
        let moon = CelestialBody::moon();
        let g = moon.gravitational_parameter / moon.radius_m.powi(2);
        assert!((g - 1.62).abs() < 0.01);
    }

    #[test]
    fn mars_surface_gravity_matches_known_value() {
        let mars = CelestialBody::mars();
        let g = mars.gravitational_parameter / mars.radius_m.powi(2);
        assert!((g - 3.71).abs() < 0.01);
    }

    #[test]
    fn sun_surface_gravity_matches_known_value() {
        let sun = CelestialBody::sun();
        let g = sun.gravitational_parameter / sun.radius_m.powi(2);
        assert!((g - 274.0).abs() < 1.0);
    }
}
