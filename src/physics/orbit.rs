use std::f64::consts::PI;

use crate::physics::orbit_type::OrbitType;
use crate::physics::vector2::Vector2;

// a/e representation covers all four orbit types; apoapsis and
// period are None for unbound orbits. h is stored directly instead
// of derived from a/e since parabolic orbits have infinite a.
pub struct Orbit {
    pub semi_major_axis_m: f64,
    pub eccentricity: f64,
    specific_angular_momentum_m2_s: f64,
    mu: f64,
}

impl Orbit {
    fn from_a_e(semi_major_axis_m: f64, eccentricity: f64, mu: f64) -> Self {
        let h = (mu * semi_major_axis_m * (1.0 - eccentricity.powi(2))).sqrt();
        Orbit {
            semi_major_axis_m,
            eccentricity,
            specific_angular_momentum_m2_s: h,
            mu,
        }
    }

    pub fn from_periapsis_apoapsis(periapsis_radius_m: f64, apoapsis_radius_m: f64, mu: f64) -> Self {
        let a = (periapsis_radius_m + apoapsis_radius_m) / 2.0;
        let e = (apoapsis_radius_m - periapsis_radius_m) / (apoapsis_radius_m + periapsis_radius_m);
        Orbit::from_a_e(a, e, mu)
    }

    pub fn from_semi_major_axis_eccentricity(semi_major_axis_m: f64, eccentricity: f64, mu: f64) -> Self {
        Orbit::from_a_e(semi_major_axis_m, eccentricity, mu)
    }

    pub fn from_position_velocity(position_m: Vector2, velocity_m_s: Vector2, mu: f64) -> Self {
        let r = position_m.magnitude();
        let v = velocity_m_s.magnitude();

        let specific_energy = v * v / 2.0 - mu / r;
        let h = position_m.cross(&velocity_m_s);

        // clamp instead of NaN on fp rounding for near-circular orbits
        let under_sqrt = 1.0 + (2.0 * specific_energy * h * h) / (mu * mu);
        let e = under_sqrt.max(0.0).sqrt();

        // parabolic -> infinite a, but vis-viva only uses 1/a so it degrades fine (1/inf = 0)
        let a = if specific_energy.abs() > 1e-9 {
            -mu / (2.0 * specific_energy)
        } else {
            f64::INFINITY
        };

        Orbit {
            semi_major_axis_m: a,
            eccentricity: e,
            specific_angular_momentum_m2_s: h.abs(),
            mu,
        }
    }

    pub fn orbit_type(&self) -> OrbitType {
        OrbitType::from_eccentricity(self.eccentricity)
    }

    pub fn periapsis_radius_m(&self) -> f64 {
        self.semi_major_axis_m * (1.0 - self.eccentricity)
    }

    pub fn apoapsis_radius_m(&self) -> Option<f64> {
        if self.orbit_type().is_bound() {
            Some(self.semi_major_axis_m * (1.0 + self.eccentricity))
        } else {
            None
        }
    }

    pub fn period_s(&self) -> Option<f64> {
        if self.orbit_type().is_bound() {
            Some(2.0 * PI * (self.semi_major_axis_m.powi(3) / self.mu).sqrt())
        } else {
            None
        }
    }

    pub fn velocity_at_radius_m_s(&self, radius_m: f64) -> f64 {
        (self.mu * (2.0 / radius_m - 1.0 / self.semi_major_axis_m)).sqrt()
    }

    pub fn velocity_at_periapsis_m_s(&self) -> f64 {
        self.velocity_at_radius_m_s(self.periapsis_radius_m())
    }

    pub fn velocity_at_apoapsis_m_s(&self) -> Option<f64> {
        self.apoapsis_radius_m().map(|ra| self.velocity_at_radius_m_s(ra))
    }

    pub fn specific_energy(&self) -> f64 {
        -self.mu / (2.0 * self.semi_major_axis_m)
    }

    pub fn specific_angular_momentum(&self) -> f64 {
        self.specific_angular_momentum_m2_s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::CelestialBody;

    fn earth_mu() -> f64 {
        CelestialBody::earth().gravitational_parameter
    }

    fn example_orbit() -> Orbit {
        let earth = CelestialBody::earth();
        let rp = earth.radius_m + 200_000.0;
        let ra = earth.radius_m + 400_000.0;
        Orbit::from_periapsis_apoapsis(rp, ra, earth.gravitational_parameter)
    }

    #[test]
    fn semi_major_axis_matches_expected() {
        let o = example_orbit();
        assert!((o.semi_major_axis_m - 6_678_137.0).abs() < 1.0);
    }

    #[test]
    fn eccentricity_matches_expected() {
        let o = example_orbit();
        assert!((o.eccentricity - 0.01495).abs() < 0.0001);
    }

    #[test]
    fn period_is_about_90_5_minutes() {
        let o = example_orbit();
        let period_min = o.period_s().expect("elliptical orbit should have a period") / 60.0;
        assert!((period_min - 90.52).abs() < 0.05);
    }

    // spec's example gave 7.784/7.669 here - actual vis-viva gives 7.842/7.611, see DEVLOG
    #[test]
    fn periapsis_velocity_is_about_7_84_km_s() {
        let o = example_orbit();
        let v_km_s = o.velocity_at_periapsis_m_s() / 1000.0;
        assert!((v_km_s - 7.842).abs() < 0.01);
    }

    #[test]
    fn apoapsis_velocity_is_about_7_61_km_s() {
        let o = example_orbit();
        let v_km_s = o.velocity_at_apoapsis_m_s().expect("elliptical orbit should have an apoapsis") / 1000.0;
        assert!((v_km_s - 7.611).abs() < 0.01);
    }

    #[test]
    fn periapsis_and_apoapsis_conserve_angular_momentum() {
        let o = example_orbit();
        let h_at_p = o.velocity_at_periapsis_m_s() * o.periapsis_radius_m();
        let h_at_a = o.velocity_at_apoapsis_m_s().unwrap() * o.apoapsis_radius_m().unwrap();
        assert!((h_at_p - h_at_a).abs() < 1.0);
    }

    #[test]
    fn specific_energy_is_negative_for_bound_orbit() {
        let o = example_orbit();
        let energy_mj_kg = o.specific_energy() / 1_000_000.0;
        assert!((energy_mj_kg - (-29.84)).abs() < 0.05);
    }

    #[test]
    fn periapsis_is_always_less_than_apoapsis() {
        let o = example_orbit();
        assert!(o.periapsis_radius_m() < o.apoapsis_radius_m().unwrap());
    }

    #[test]
    fn periapsis_apoapsis_orbit_is_classified_elliptical() {
        let o = example_orbit();
        assert_eq!(o.orbit_type(), OrbitType::Elliptical);
    }

    #[test]
    fn semi_major_axis_eccentricity_derives_correct_periapsis_apoapsis() {
        let o = Orbit::from_semi_major_axis_eccentricity(7_000_000.0, 0.05, earth_mu());
        assert!((o.periapsis_radius_m() - 6_650_000.0).abs() < 1.0);
        assert!((o.apoapsis_radius_m().unwrap() - 7_350_000.0).abs() < 1.0);
    }

    #[test]
    fn semi_major_axis_eccentricity_period_matches_expected() {
        let o = Orbit::from_semi_major_axis_eccentricity(7_000_000.0, 0.05, earth_mu());
        let period_min = o.period_s().unwrap() / 60.0;
        assert!((period_min - 97.14).abs() < 0.05);
    }

    #[test]
    fn position_velocity_circular_orbit_gives_zero_eccentricity() {
        let r = 7_000_000.0;
        let v_circular = (earth_mu() / r).sqrt();
        let position = Vector2::new(r, 0.0);
        let velocity = Vector2::new(0.0, v_circular);

        let o = Orbit::from_position_velocity(position, velocity, earth_mu());
        assert!(o.eccentricity < 1e-4);
        assert_eq!(o.orbit_type(), OrbitType::Circular);
    }

    #[test]
    fn position_velocity_high_speed_gives_hyperbolic_orbit() {
        // escape velocity here is ~10.67 km/s, so 15 km/s is safely hyperbolic
        let r = 7_000_000.0;
        let position = Vector2::new(r, 0.0);
        let velocity = Vector2::new(0.0, 15_000.0);

        let o = Orbit::from_position_velocity(position, velocity, earth_mu());
        assert_eq!(o.orbit_type(), OrbitType::Hyperbolic);
        assert!(o.eccentricity > 1.0);
        assert!((o.eccentricity - 2.9513).abs() < 0.001);
        assert!(o.apoapsis_radius_m().is_none());
        assert!(o.period_s().is_none());
    }

    #[test]
    fn position_velocity_at_escape_velocity_gives_parabolic_orbit() {
        let r = 7_000_000.0;
        let v_escape = (2.0 * earth_mu() / r).sqrt();
        let position = Vector2::new(r, 0.0);
        let velocity = Vector2::new(0.0, v_escape);

        let o = Orbit::from_position_velocity(position, velocity, earth_mu());
        assert_eq!(o.orbit_type(), OrbitType::Parabolic);
        assert!(o.apoapsis_radius_m().is_none());
        assert!(o.period_s().is_none());
    }

    #[test]
    fn orbit_type_classification_boundaries() {
        assert_eq!(OrbitType::from_eccentricity(0.0), OrbitType::Circular);
        assert_eq!(OrbitType::from_eccentricity(0.3), OrbitType::Elliptical);
        assert_eq!(OrbitType::from_eccentricity(1.0), OrbitType::Parabolic);
        assert_eq!(OrbitType::from_eccentricity(1.8), OrbitType::Hyperbolic);
    }

    // vis-viva subtracts 2/r and 1/a - checking that stays precise near e=0
    #[test]
    fn near_circular_orbit_does_not_lose_precision_to_cancellation() {
        let earth = CelestialBody::earth();
        let rp = earth.radius_m + 500_000.0;
        let ra = rp + 1.0;
        let o = Orbit::from_periapsis_apoapsis(rp, ra, earth.gravitational_parameter);

        assert!(o.eccentricity > 0.0);
        assert!(o.eccentricity < 1e-6);

        let vp = o.velocity_at_periapsis_m_s();
        let va = o.velocity_at_apoapsis_m_s().unwrap();
        let diff_mm_s = (vp - va) * 1000.0;
        assert!(diff_mm_s > 0.0);
        assert!(diff_mm_s < 5.0);
    }

    #[test]
    fn very_low_orbit_precision_matches_hand_derived_values() {
        let earth = CelestialBody::earth();
        let rp = earth.radius_m + 100.0;
        let ra = earth.radius_m + 1000.0;
        let o = Orbit::from_periapsis_apoapsis(rp, ra, earth.gravitational_parameter);

        assert!((o.eccentricity - 7.054743e-5).abs() < 1e-9);
        let period_min = o.period_s().unwrap() / 60.0;
        assert!((period_min - 84.5).abs() < 0.001);
    }

    #[test]
    fn high_altitude_orbit_precision_matches_hand_derived_values() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 35_786_000.0;
        let o = Orbit::from_periapsis_apoapsis(r, r, earth.gravitational_parameter);

        // GEO period should land on a sidereal day, ~23.93 hours
        let period_hours = o.period_s().unwrap() / 3600.0;
        assert!((period_hours - 23.93).abs() < 0.02);
    }
}
