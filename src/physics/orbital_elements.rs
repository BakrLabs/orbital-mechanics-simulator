use std::f64::consts::PI;

use crate::physics::vector3::Vector3;

pub struct OrbitalElements {
    pub semi_major_axis_m: f64,
    pub eccentricity: f64,
    pub inclination_rad: f64,
    pub raan_rad: f64,
    pub argument_of_periapsis_rad: f64,
    pub true_anomaly_rad: f64,
    mu: f64,
}

impl OrbitalElements {
    pub fn new(
        semi_major_axis_m: f64,
        eccentricity: f64,
        inclination_rad: f64,
        raan_rad: f64,
        argument_of_periapsis_rad: f64,
        true_anomaly_rad: f64,
        mu: f64,
    ) -> Self {
        OrbitalElements {
            semi_major_axis_m,
            eccentricity,
            inclination_rad,
            raan_rad,
            argument_of_periapsis_rad,
            true_anomaly_rad,
            mu,
        }
    }

    pub fn from_state_vector(position_m: Vector3, velocity_m_s: Vector3, mu: f64) -> Self {
        let r = position_m.magnitude();
        let v = velocity_m_s.magnitude();

        let h_vec = position_m.cross(&velocity_m_s);
        let h = h_vec.magnitude();

        let z_axis = Vector3::new(0.0, 0.0, 1.0);
        let n_vec = z_axis.cross(&h_vec);
        let n = n_vec.magnitude();

        let energy = v * v / 2.0 - mu / r;
        let a = -mu / (2.0 * energy);

        let v_cross_h = velocity_m_s.cross(&h_vec).scale(1.0 / mu);
        let e_vec = v_cross_h.sub(&position_m.scale(1.0 / r));
        let e = e_vec.magnitude();

        let i = (h_vec.z / h).clamp(-1.0, 1.0).acos();

        // no node line for an equatorial orbit - RAAN undefined, 0 by convention
        let raan = if n > 1e-12 {
            let mut angle = (n_vec.x / n).clamp(-1.0, 1.0).acos();
            if n_vec.y < 0.0 {
                angle = 2.0 * PI - angle;
            }
            angle
        } else {
            0.0
        };

        let argp = if n > 1e-12 && e > 1e-12 {
            let mut angle = (n_vec.dot(&e_vec) / (n * e)).clamp(-1.0, 1.0).acos();
            if e_vec.z < 0.0 {
                angle = 2.0 * PI - angle;
            }
            angle
        } else {
            0.0
        };

        let nu = if e > 1e-12 {
            let mut angle = (e_vec.dot(&position_m) / (e * r)).clamp(-1.0, 1.0).acos();
            if position_m.dot(&velocity_m_s) < 0.0 {
                angle = 2.0 * PI - angle;
            }
            angle
        } else {
            0.0
        };

        OrbitalElements {
            semi_major_axis_m: a,
            eccentricity: e,
            inclination_rad: i,
            raan_rad: raan,
            argument_of_periapsis_rad: argp,
            true_anomaly_rad: nu,
            mu,
        }
    }

    // perifocal -> ECI via R3(-raan) R1(-i) R3(-argp)
    pub fn to_state_vector(&self) -> (Vector3, Vector3) {
        let a = self.semi_major_axis_m;
        let e = self.eccentricity;
        let nu = self.true_anomaly_rad;

        let p = a * (1.0 - e * e);
        let r_mag = p / (1.0 + e * nu.cos());

        let r_pf = Vector3::new(r_mag * nu.cos(), r_mag * nu.sin(), 0.0);
        let h = (self.mu * p).sqrt();
        let v_pf = Vector3::new(-self.mu / h * nu.sin(), self.mu / h * (e + nu.cos()), 0.0);

        let (co, so) = (self.raan_rad.cos(), self.raan_rad.sin());
        let (ci, si) = (self.inclination_rad.cos(), self.inclination_rad.sin());
        let (cw, sw) = (self.argument_of_periapsis_rad.cos(), self.argument_of_periapsis_rad.sin());

        let r11 = co * cw - so * sw * ci;
        let r12 = -co * sw - so * cw * ci;
        let r13 = so * si;
        let r21 = so * cw + co * sw * ci;
        let r22 = -so * sw + co * cw * ci;
        let r23 = -co * si;
        let r31 = sw * si;
        let r32 = cw * si;
        let r33 = ci;

        let position = Vector3::new(
            r11 * r_pf.x + r12 * r_pf.y + r13 * r_pf.z,
            r21 * r_pf.x + r22 * r_pf.y + r23 * r_pf.z,
            r31 * r_pf.x + r32 * r_pf.y + r33 * r_pf.z,
        );
        let velocity = Vector3::new(
            r11 * v_pf.x + r12 * v_pf.y + r13 * v_pf.z,
            r21 * v_pf.x + r22 * v_pf.y + r23 * v_pf.z,
            r31 * v_pf.x + r32 * v_pf.y + r33 * v_pf.z,
        );

        (position, velocity)
    }

    pub fn inclination_deg(&self) -> f64 {
        self.inclination_rad.to_degrees()
    }

    pub fn raan_deg(&self) -> f64 {
        self.raan_rad.to_degrees()
    }

    pub fn argument_of_periapsis_deg(&self) -> f64 {
        self.argument_of_periapsis_rad.to_degrees()
    }

    pub fn true_anomaly_deg(&self) -> f64 {
        self.true_anomaly_rad.to_degrees()
    }

    pub fn period_s(&self) -> Option<f64> {
        if self.eccentricity < 1.0 && self.semi_major_axis_m > 0.0 {
            Some(2.0 * PI * (self.semi_major_axis_m.powi(3) / self.mu).sqrt())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::CelestialBody;

    fn earth_mu() -> f64 {
        CelestialBody::earth().gravitational_parameter
    }

    #[test]
    fn elements_to_state_and_back_round_trips() {
        let original = OrbitalElements::new(
            7_000_000.0,
            0.1,
            28.5_f64.to_radians(),
            45.0_f64.to_radians(),
            90.0_f64.to_radians(),
            120.0_f64.to_radians(),
            earth_mu(),
        );

        let (position, velocity) = original.to_state_vector();
        let recovered = OrbitalElements::from_state_vector(position, velocity, earth_mu());

        assert!((recovered.semi_major_axis_m - original.semi_major_axis_m).abs() < 1.0);
        assert!((recovered.eccentricity - original.eccentricity).abs() < 1e-9);
        assert!((recovered.inclination_rad - original.inclination_rad).abs() < 1e-9);
        assert!((recovered.raan_rad - original.raan_rad).abs() < 1e-9);
        assert!((recovered.argument_of_periapsis_rad - original.argument_of_periapsis_rad).abs() < 1e-9);
        assert!((recovered.true_anomaly_rad - original.true_anomaly_rad).abs() < 1e-9);
    }

    #[test]
    fn zero_inclination_orbit_matches_2d_case() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();

        let position = Vector3::new(r, 0.0, 0.0);
        let velocity = Vector3::new(0.0, v, 0.0);

        let elements = OrbitalElements::from_state_vector(position, velocity, earth_mu());

        assert!((elements.semi_major_axis_m - r).abs() < 1.0);
        assert!(elements.eccentricity < 1e-6);
        assert!(elements.inclination_rad.abs() < 1e-9);
    }

    #[test]
    fn polar_orbit_has_90_degree_inclination() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 700_000.0;
        let v = (earth_mu() / r).sqrt();

        let position = Vector3::new(r, 0.0, 0.0);
        let velocity = Vector3::new(0.0, 0.0, v);

        let elements = OrbitalElements::from_state_vector(position, velocity, earth_mu());
        assert!((elements.inclination_deg() - 90.0).abs() < 0.01);
    }

    #[test]
    fn period_matches_2d_orbit_period_for_equatorial_case() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();

        let position = Vector3::new(r, 0.0, 0.0);
        let velocity = Vector3::new(0.0, v, 0.0);
        let elements = OrbitalElements::from_state_vector(position, velocity, earth_mu());

        let orbit_2d = crate::physics::orbit::Orbit::from_periapsis_apoapsis(r, r, earth_mu());

        let period_3d = elements.period_s().unwrap();
        let period_2d = orbit_2d.period_s().unwrap();
        assert!((period_3d - period_2d).abs() < 0.001);
    }
}
