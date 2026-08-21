use std::f64::consts::PI;

/// A Hohmann transfer between two circular orbits - the classic
/// two-burn maneuver: one burn raises (or lowers) apoapsis onto the
/// transfer ellipse, the other circularizes at the target radius.
///
/// This only covers circular-to-circular transfers for now. A more
/// general version (elliptical start/end orbits, plane changes) is a
/// bigger piece of math and isn't part of this version.
pub struct HohmannTransfer {
    pub r1_m: f64,
    pub r2_m: f64,
    mu: f64,
}

impl HohmannTransfer {
    /// `r1` is the starting circular orbit's radius, `r2` the target's -
    /// both measured from the center of the body, in meters. r2 can be
    /// larger than r1 (raising orbit) or smaller (lowering orbit); the
    /// maneuver works the same either way, just with signs flipping on
    /// the two burns.
    pub fn new(r1_m: f64, r2_m: f64, mu: f64) -> Self {
        HohmannTransfer { r1_m, r2_m, mu }
    }

    /// Semi-major axis of the transfer ellipse - halfway between the
    /// two circular radii by definition, since one is periapsis and
    /// the other is apoapsis of the transfer orbit.
    pub fn transfer_semi_major_axis_m(&self) -> f64 {
        (self.r1_m + self.r2_m) / 2.0
    }

    pub fn initial_circular_velocity_m_s(&self) -> f64 {
        (self.mu / self.r1_m).sqrt()
    }

    pub fn final_circular_velocity_m_s(&self) -> f64 {
        (self.mu / self.r2_m).sqrt()
    }

    /// Speed on the transfer ellipse at the point coincident with the
    /// starting circular orbit (vis-viva evaluated at r1, using the
    /// transfer orbit's semi-major axis).
    pub fn transfer_velocity_at_r1_m_s(&self) -> f64 {
        let at = self.transfer_semi_major_axis_m();
        (self.mu * (2.0 / self.r1_m - 1.0 / at)).sqrt()
    }

    /// Speed on the transfer ellipse at the point coincident with the
    /// target circular orbit.
    pub fn transfer_velocity_at_r2_m_s(&self) -> f64 {
        let at = self.transfer_semi_major_axis_m();
        (self.mu * (2.0 / self.r2_m - 1.0 / at)).sqrt()
    }

    /// First burn: leaving the initial circular orbit and entering the
    /// transfer ellipse. Signed - positive means a prograde (speed-up)
    /// burn, negative means retrograde (slow-down), which happens
    /// naturally when r2 < r1 (lowering orbit).
    pub fn burn1_delta_v_m_s(&self) -> f64 {
        self.transfer_velocity_at_r1_m_s() - self.initial_circular_velocity_m_s()
    }

    /// Second burn: leaving the transfer ellipse and circularizing at
    /// the target radius. Also signed, same convention as burn 1.
    pub fn burn2_delta_v_m_s(&self) -> f64 {
        self.final_circular_velocity_m_s() - self.transfer_velocity_at_r2_m_s()
    }

    /// Total delta-v for the maneuver - propellant cost only cares
    /// about magnitude, not direction, so this sums the absolute value
    /// of each burn rather than the signed values (which could
    /// partially cancel and understate the actual cost).
    pub fn total_delta_v_m_s(&self) -> f64 {
        self.burn1_delta_v_m_s().abs() + self.burn2_delta_v_m_s().abs()
    }

    /// Transfer time: half the transfer ellipse's orbital period,
    /// since the maneuver only covers periapsis-to-apoapsis (or vice
    /// versa), not a full revolution.
    pub fn transfer_time_s(&self) -> f64 {
        let at = self.transfer_semi_major_axis_m();
        PI * (at.powi(3) / self.mu).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::CelestialBody;

    // Same 200km -> 400km altitude example used for the orbital
    // parameter tests, so the circular velocities and transfer time
    // here can be cross-checked against those.
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

    // These two check against hand-derived vis-viva values, not the
    // spec's original worked example - see the README for why (the
    // spec's burn delta-v figures don't match what vis-viva actually
    // produces for this transfer, off by roughly a factor of 2).
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

    // Raising vs lowering orbit should flip the sign of both burns -
    // raising means both burns are prograde (positive), lowering means
    // both are retrograde (negative). Total delta-v (the propellant
    // cost) should be identical either way, since a Hohmann transfer
    // is symmetric in that sense.
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

    // --- Precision / numerical stability checks ---

    #[test]
    fn leo_to_geo_matches_known_textbook_result() {
        // LEO (200km altitude) to GEO altitude (35786km) is one of the
        // most commonly cited Hohmann transfer examples there is - the
        // textbook total delta-v figure is right around 3.9 km/s. This
        // is as much a sanity check on the whole calculation chain as
        // it is a precision test.
        let earth = CelestialBody::earth();
        let r1 = earth.radius_m + 200_000.0;
        let r2 = earth.radius_m + 35_786_000.0;
        let t = HohmannTransfer::new(r1, r2, earth.gravitational_parameter);

        let total_km_s = t.total_delta_v_m_s() / 1000.0;
        assert!((total_km_s - 3.932).abs() < 0.01);

        let transfer_hours = t.transfer_time_s() / 3600.0;
        assert!((transfer_hours - 5.259).abs() < 0.01);
    }

    #[test]
    fn near_identical_altitude_transfer_gives_small_but_nonzero_delta_v() {
        // A 10-meter altitude difference should give a very small
        // total delta-v, not zero and not NaN/garbage from precision
        // loss in the vis-viva subtraction.
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
