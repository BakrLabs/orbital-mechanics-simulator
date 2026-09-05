use std::f64::consts::PI;

// circular-to-circular only for now - elliptical start/end orbits and plane changes need more math
pub struct HohmannTransfer {
    pub r1_m: f64,
    pub r2_m: f64,
    mu: f64,
}

impl HohmannTransfer {
    pub fn new(r1_m: f64, r2_m: f64, mu: f64) -> Self {
        HohmannTransfer { r1_m, r2_m, mu }
    }

    pub fn transfer_semi_major_axis_m(&self) -> f64 {
        (self.r1_m + self.r2_m) / 2.0
    }

    pub fn initial_circular_velocity_m_s(&self) -> f64 {
        (self.mu / self.r1_m).sqrt()
    }

    pub fn final_circular_velocity_m_s(&self) -> f64 {
        (self.mu / self.r2_m).sqrt()
    }

    pub fn transfer_velocity_at_r1_m_s(&self) -> f64 {
        let at = self.transfer_semi_major_axis_m();
        (self.mu * (2.0 / self.r1_m - 1.0 / at)).sqrt()
    }

    pub fn transfer_velocity_at_r2_m_s(&self) -> f64 {
        let at = self.transfer_semi_major_axis_m();
        (self.mu * (2.0 / self.r2_m - 1.0 / at)).sqrt()
    }

    // signed: positive = prograde, negative = retrograde
    pub fn burn1_delta_v_m_s(&self) -> f64 {
        self.transfer_velocity_at_r1_m_s() - self.initial_circular_velocity_m_s()
    }

    pub fn burn2_delta_v_m_s(&self) -> f64 {
        self.final_circular_velocity_m_s() - self.transfer_velocity_at_r2_m_s()
    }

    // propellant cost only cares about magnitude, not direction
    pub fn total_delta_v_m_s(&self) -> f64 {
        self.burn1_delta_v_m_s().abs() + self.burn2_delta_v_m_s().abs()
    }

    pub fn transfer_time_s(&self) -> f64 {
        let at = self.transfer_semi_major_axis_m();
        PI * (at.powi(3) / self.mu).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::CelestialBody;

    fn example_transfer() -> HohmannTransfer {
        let earth = CelestialBody::earth();
        let r1 = earth.radius_m + 200_000.0;
        let r2 = earth.radius_m + 400_000.0;
        HohmannTransfer::new(r1, r2, earth.gravitational_parameter)
    }

    #[test]
    fn transfer_semi_major_axis_matches_expected() {
        let t = example_transfer();
        assert!((t.transfer_semi_major_axis_m() - 6_678_137.0).abs() < 1.0);
    }

    #[test]
    fn initial_circular_velocity_matches_expected() {
        let t = example_transfer();
        let v_km_s = t.initial_circular_velocity_m_s() / 1000.0;
        assert!((v_km_s - 7.784).abs() < 0.01);
    }

    #[test]
    fn final_circular_velocity_matches_expected() {
        let t = example_transfer();
        let v_km_s = t.final_circular_velocity_m_s() / 1000.0;
        assert!((v_km_s - 7.669).abs() < 0.01);
    }

    // spec's worked example put these at 0.117/0.114 - actual vis-viva
    // for this transfer gives 0.058/0.058, off by ~2x. see DEVLOG.
    #[test]
    fn burn1_delta_v_matches_hand_derived_value() {
        let t = example_transfer();
        let dv1_km_s = t.burn1_delta_v_m_s() / 1000.0;
        assert!((dv1_km_s - 0.0581).abs() < 0.001);
    }

    #[test]
    fn burn2_delta_v_matches_hand_derived_value() {
        let t = example_transfer();
        let dv2_km_s = t.burn2_delta_v_m_s() / 1000.0;
        assert!((dv2_km_s - 0.0576).abs() < 0.001);
    }

    #[test]
    fn total_delta_v_matches_hand_derived_value() {
        let t = example_transfer();
        let total_km_s = t.total_delta_v_m_s() / 1000.0;
        assert!((total_km_s - 0.1157).abs() < 0.001);
    }

    #[test]
    fn transfer_time_matches_expected() {
        let t = example_transfer();
        let minutes = t.transfer_time_s() / 60.0;
        assert!((minutes - 45.26).abs() < 0.05);
    }

    #[test]
    fn raising_orbit_gives_positive_burns() {
        let t = example_transfer(); // r1 < r2, raising
        assert!(t.burn1_delta_v_m_s() > 0.0);
        assert!(t.burn2_delta_v_m_s() > 0.0);
    }

    #[test]
    fn lowering_orbit_gives_negative_burns_with_same_total_delta_v() {
        let earth = CelestialBody::earth();
        let raising = HohmannTransfer::new(
            earth.radius_m + 200_000.0,
            earth.radius_m + 400_000.0,
            earth.gravitational_parameter,
        );
        let lowering = HohmannTransfer::new(
            earth.radius_m + 400_000.0,
            earth.radius_m + 200_000.0,
            earth.gravitational_parameter,
        );

        assert!(lowering.burn1_delta_v_m_s() < 0.0);
        assert!(lowering.burn2_delta_v_m_s() < 0.0);

        let raising_total = raising.total_delta_v_m_s();
        let lowering_total = lowering.total_delta_v_m_s();
        assert!((raising_total - lowering_total).abs() < 1.0);
    }

    #[test]
    fn leo_to_geo_matches_known_textbook_result() {
        let earth = CelestialBody::earth();
        let r1 = earth.radius_m + 200_000.0;
        let r2 = earth.radius_m + 35_786_000.0;
        let t = HohmannTransfer::new(r1, r2, earth.gravitational_parameter);

        // textbook figure for this transfer is ~3.9 km/s total
        let total_km_s = t.total_delta_v_m_s() / 1000.0;
        assert!((total_km_s - 3.932).abs() < 0.01);

        let transfer_hours = t.transfer_time_s() / 3600.0;
        assert!((transfer_hours - 5.259).abs() < 0.01);
    }

    #[test]
    fn near_identical_altitude_transfer_gives_small_but_nonzero_delta_v() {
        let earth = CelestialBody::earth();
        let r1 = earth.radius_m + 500_000.0;
        let r2 = r1 + 10.0;
        let t = HohmannTransfer::new(r1, r2, earth.gravitational_parameter);

        let total_m_s = t.total_delta_v_m_s();
        assert!(total_m_s > 0.0);
        assert!(total_m_s < 0.01, "expected a sub-cm/s delta-v, got {} m/s", total_m_s);
        assert!(total_m_s.is_finite());
    }
}
