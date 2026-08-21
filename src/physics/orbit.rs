use std::f64::consts::PI;

use crate::physics::orbit_type::OrbitType;
use crate::physics::vector2::Vector2;

/// An orbit around some central body. Internally this still boils
/// down to semi-major axis + eccentricity, same as v0.2 - what's new
/// is that it no longer assumes those describe a closed ellipse.
/// A parabolic or hyperbolic trajectory has an eccentricity too (it's
/// just >= 1), so the same struct covers all three; the methods that
/// only make sense for a bound orbit (apoapsis, period) return
/// `Option` instead of quietly producing nonsense numbers.
///
/// For parabolic orbits specifically, semi-major axis is technically
/// undefined (it's infinite) - `specific_angular_momentum` is stored
/// directly rather than derived from `a`, since deriving it from a
/// formula involving infinity is asking for trouble.
pub struct Orbit {
    pub semi_major_axis_m: f64,
    pub eccentricity: f64,
    specific_angular_momentum_m2_s: f64,
    mu: f64,
}

impl Orbit {
    /// Shared constructor. `h` (specific angular momentum) is derived
    /// from a/e here for the periapsis/apoapsis and semi-major-axis/
    /// eccentricity entry points, where it isn't already known -
    /// `from_position_velocity` bypasses this and supplies its own `h`
    /// directly, since it has the real vector cross product on hand.
    fn from_a_e(semi_major_axis_m: f64, eccentricity: f64, mu: f64) -> Self {
        let h = (mu * semi_major_axis_m * (1.0 - eccentricity.powi(2))).sqrt();
        Orbit {
            semi_major_axis_m,
            eccentricity,
            specific_angular_momentum_m2_s: h,
            mu,
        }
    }

    /// Builds an orbit directly from periapsis/apoapsis radii (distance
    /// from the center of the body, not altitude - altitude gets
    /// converted to radius before this is called). Only makes sense
    /// for a closed orbit, which is exactly what having both a
    /// periapsis and an apoapsis means.
    pub fn from_periapsis_apoapsis(periapsis_radius_m: f64, apoapsis_radius_m: f64, mu: f64) -> Self {
        let a = (periapsis_radius_m + apoapsis_radius_m) / 2.0;
        let e = (apoapsis_radius_m - periapsis_radius_m) / (apoapsis_radius_m + periapsis_radius_m);
        Orbit::from_a_e(a, e, mu)
    }

    /// Builds an orbit directly from semi-major axis and eccentricity -
    /// the two numbers this struct stores internally anyway, so this
    /// is close to a pass-through, but it's still the right place to
    /// centralize construction rather than have callers build the
    /// struct's fields by hand.
    pub fn from_semi_major_axis_eccentricity(semi_major_axis_m: f64, eccentricity: f64, mu: f64) -> Self {
        Orbit::from_a_e(semi_major_axis_m, eccentricity, mu)
    }

    /// Builds an orbit from a 2D position and velocity vector at some
    /// instant - the classical two-body "state vector to orbital
    /// elements" conversion, restricted to the planar (2D) case.
    ///
    /// Position and velocity should be in meters and meters/second
    /// respectively; the unit conversion from km happens at the call
    /// site, same as the other constructors.
    pub fn from_position_velocity(position_m: Vector2, velocity_m_s: Vector2, mu: f64) -> Self {
        let r = position_m.magnitude();
        let v = velocity_m_s.magnitude();

        let specific_energy = v * v / 2.0 - mu / r;
        let h = position_m.cross(&velocity_m_s);

        // e = sqrt(1 + 2*energy*h^2/mu^2). For an orbit that's
        // circular in practice, floating-point error in `specific_energy`
        // can push the term under the sqrt fractionally below zero
        // (it should be exactly -1/2 there, giving e = 0) - clamping
        // at 0 avoids a NaN from sqrt of a tiny negative number.
        let under_sqrt = 1.0 + (2.0 * specific_energy * h * h) / (mu * mu);
        let e = under_sqrt.max(0.0).sqrt();

        // a = -mu / (2*energy) only holds when energy isn't ~0. A
        // parabolic orbit (energy == 0, e == 1) has no finite
        // semi-major axis at all - representing it as infinity keeps
        // downstream formulas (like velocity_at_radius_m_s, which only
        // uses 1/a) well-behaved: 1/inf is just 0, so vis-viva quietly
        // degenerates to the correct v = sqrt(2*mu/r) for a parabolic
        // trajectory without a special case there.
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

    /// Only bound orbits (circular/elliptical) have an apoapsis - a
    /// parabolic or hyperbolic trajectory just keeps going and never
    /// comes back, so there's nothing meaningful to return.
    pub fn apoapsis_radius_m(&self) -> Option<f64> {
        if self.orbit_type().is_bound() {
            Some(self.semi_major_axis_m * (1.0 + self.eccentricity))
        } else {
            None
        }
    }

    /// Orbital period, in seconds. Only defined for a closed orbit -
    /// something on a parabolic or hyperbolic trajectory never
    /// completes a revolution, so there's no period to report.
    pub fn period_s(&self) -> Option<f64> {
        if self.orbit_type().is_bound() {
            Some(2.0 * PI * (self.semi_major_axis_m.powi(3) / self.mu).sqrt())
        } else {
            None
        }
    }

    /// Vis-viva: speed at a given orbital radius (distance from the
    /// center of the body, in meters). Works the same regardless of
    /// orbit type since it only depends on 1/a, and 1/a is 0 for a
    /// parabolic orbit's (infinite) semi-major axis and negative for
    /// a hyperbolic orbit's - both fall out correctly with no special
    /// casing needed here.
    pub fn velocity_at_radius_m_s(&self, radius_m: f64) -> f64 {
        (self.mu * (2.0 / radius_m - 1.0 / self.semi_major_axis_m)).sqrt()
    }

    pub fn velocity_at_periapsis_m_s(&self) -> f64 {
        self.velocity_at_radius_m_s(self.periapsis_radius_m())
    }

    pub fn velocity_at_apoapsis_m_s(&self) -> Option<f64> {
        self.apoapsis_radius_m().map(|ra| self.velocity_at_radius_m_s(ra))
    }

    /// Specific orbital energy, in J/kg (equivalently m^2/s^2).
    /// Negative for bound orbits, ~zero for parabolic, positive for
    /// hyperbolic. For a parabolic orbit `a` is infinite, so this
    /// correctly rounds to 0 rather than needing a special case.
    pub fn specific_energy(&self) -> f64 {
        -self.mu / (2.0 * self.semi_major_axis_m)
    }

    /// Specific angular momentum, in m^2/s.
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

    // --- periapsis/apoapsis method (carried over from v0.2) ---

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

    // --- semi-major axis / eccentricity method ---

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

    // --- position/velocity method ---

    #[test]
    fn position_velocity_circular_orbit_gives_zero_eccentricity() {
        // A satellite at 7000km, moving purely tangentially at exactly
        // circular speed, should come back with e ~ 0.
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
        // Same position, but well above escape velocity (escape at
        // 7000km is ~10.67 km/s here, so 15 km/s is comfortably
        // hyperbolic).
        let r = 7_000_000.0;
        let position = Vector2::new(r, 0.0);
        let velocity = Vector2::new(0.0, 15_000.0);

        let o = Orbit::from_position_velocity(position, velocity, earth_mu());
        assert_eq!(o.orbit_type(), OrbitType::Hyperbolic);
        assert!(o.eccentricity > 1.0);
        assert!((o.eccentricity - 2.9513).abs() < 0.001);
        // Hyperbolic orbits have no apoapsis or period.
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

    // --- OrbitType classification, independent of any constructor ---

    #[test]
    fn orbit_type_classification_boundaries() {
        assert_eq!(OrbitType::from_eccentricity(0.0), OrbitType::Circular);
        assert_eq!(OrbitType::from_eccentricity(0.3), OrbitType::Elliptical);
        assert_eq!(OrbitType::from_eccentricity(1.0), OrbitType::Parabolic);
        assert_eq!(OrbitType::from_eccentricity(1.8), OrbitType::Hyperbolic);
    }

    // --- Precision / numerical stability checks ---
    //
    // These don't test new physics - they test that the existing
    // formulas hold up at the extremes where floating-point error is
    // most likely to show up: orbits that are circular to within a
    // meter, and very low/very high altitude ranges. The equations
    // for vis-viva involve subtracting two numbers that get close to
    // each other for near-circular orbits (2/r and 1/a), which is
    // exactly the kind of expression that can lose precision to
    // catastrophic cancellation - these confirm it doesn't, at least
    // not in any way that matters at real-world orbital altitudes.

    #[test]
    fn near_circular_orbit_does_not_lose_precision_to_cancellation() {
        // Periapsis and apoapsis exactly 1 meter apart - about as
        // close to circular as a "real" elliptical orbit gets.
        let earth = CelestialBody::earth();
        let rp = earth.radius_m + 500_000.0;
        let ra = rp + 1.0;
        let o = Orbit::from_periapsis_apoapsis(rp, ra, earth.gravitational_parameter);

        // Eccentricity should be tiny but not garbage/NaN.
        assert!(o.eccentricity > 0.0);
        assert!(o.eccentricity < 1e-6);

        // Velocity at periapsis and apoapsis should differ by only a
        // few millimeters/second, not blow up into noise.
        let vp = o.velocity_at_periapsis_m_s();
        let va = o.velocity_at_apoapsis_m_s().unwrap();
        let diff_mm_s = (vp - va) * 1000.0;
        assert!(diff_mm_s > 0.0);
        assert!(diff_mm_s < 5.0);
    }

    #[test]
    fn very_low_orbit_precision_matches_hand_derived_values() {
        // 100m to 1000m altitude - deliberately unrealistic (that's
        // inside the atmosphere) but a good stress test for the low
        // end of the altitude range the calculator accepts.
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
        // A GEO-altitude circular-ish orbit - checks the high end of
        // realistic altitudes doesn't introduce error either.
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 35_786_000.0;
        let o = Orbit::from_periapsis_apoapsis(r, r, earth.gravitational_parameter);

        let period_hours = o.period_s().unwrap() / 3600.0;
        // GEO period should be a sidereal day, ~23.93 hours.
        assert!((period_hours - 23.93).abs() < 0.02);
    }
}
