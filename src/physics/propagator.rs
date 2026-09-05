use crate::physics::vector2::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct PropagatedState {
    pub elapsed_time_s: f64,
    pub position_m: Vector2,
    pub velocity_m_s: Vector2,
    pub specific_energy: f64,
    pub specific_angular_momentum: f64,
}

pub struct Propagator {
    mu: f64,
}

impl Propagator {
    pub fn new(mu: f64) -> Self {
        Propagator { mu }
    }

    // a = -mu*r/|r|^3
    fn acceleration(&self, position_m: Vector2) -> Vector2 {
        let r = position_m.magnitude();
        let factor = -self.mu / r.powi(3);
        Vector2::new(position_m.x * factor, position_m.y * factor)
    }

    fn derivative(&self, position_m: Vector2, velocity_m_s: Vector2) -> (Vector2, Vector2) {
        (velocity_m_s, self.acceleration(position_m))
    }

    fn step(&self, position_m: Vector2, velocity_m_s: Vector2, dt_s: f64) -> (Vector2, Vector2) {
        let (k1_p, k1_v) = self.derivative(position_m, velocity_m_s);

        let p2 = Vector2::new(position_m.x + dt_s / 2.0 * k1_p.x, position_m.y + dt_s / 2.0 * k1_p.y);
        let v2 = Vector2::new(velocity_m_s.x + dt_s / 2.0 * k1_v.x, velocity_m_s.y + dt_s / 2.0 * k1_v.y);
        let (k2_p, k2_v) = self.derivative(p2, v2);

        let p3 = Vector2::new(position_m.x + dt_s / 2.0 * k2_p.x, position_m.y + dt_s / 2.0 * k2_p.y);
        let v3 = Vector2::new(velocity_m_s.x + dt_s / 2.0 * k2_v.x, velocity_m_s.y + dt_s / 2.0 * k2_v.y);
        let (k3_p, k3_v) = self.derivative(p3, v3);

        let p4 = Vector2::new(position_m.x + dt_s * k3_p.x, position_m.y + dt_s * k3_p.y);
        let v4 = Vector2::new(velocity_m_s.x + dt_s * k3_v.x, velocity_m_s.y + dt_s * k3_v.y);
        let (k4_p, k4_v) = self.derivative(p4, v4);

        let new_position = Vector2::new(
            position_m.x + dt_s / 6.0 * (k1_p.x + 2.0 * k2_p.x + 2.0 * k3_p.x + k4_p.x),
            position_m.y + dt_s / 6.0 * (k1_p.y + 2.0 * k2_p.y + 2.0 * k3_p.y + k4_p.y),
        );
        let new_velocity = Vector2::new(
            velocity_m_s.x + dt_s / 6.0 * (k1_v.x + 2.0 * k2_v.x + 2.0 * k3_v.x + k4_v.x),
            velocity_m_s.y + dt_s / 6.0 * (k1_v.y + 2.0 * k2_v.y + 2.0 * k3_v.y + k4_v.y),
        );

        (new_position, new_velocity)
    }

    fn specific_energy(&self, position_m: Vector2, velocity_m_s: Vector2) -> f64 {
        let r = position_m.magnitude();
        let v = velocity_m_s.magnitude();
        v * v / 2.0 - self.mu / r
    }

    fn specific_angular_momentum(&self, position_m: Vector2, velocity_m_s: Vector2) -> f64 {
        position_m.cross(&velocity_m_s)
    }

    pub fn propagate(
        &self,
        initial_position_m: Vector2,
        initial_velocity_m_s: Vector2,
        total_time_s: f64,
        dt_s: f64,
    ) -> Vec<PropagatedState> {
        let mut states = Vec::new();
        let mut position = initial_position_m;
        let mut velocity = initial_velocity_m_s;
        let mut elapsed = 0.0;

        states.push(self.snapshot(elapsed, position, velocity));

        while elapsed < total_time_s {
            // last step is usually partial, so this lands exactly on total_time_s
            let remaining = total_time_s - elapsed;
            let this_step = if remaining < dt_s { remaining } else { dt_s };

            let (new_position, new_velocity) = self.step(position, velocity, this_step);
            position = new_position;
            velocity = new_velocity;
            elapsed += this_step;

            states.push(self.snapshot(elapsed, position, velocity));
        }

        states
    }

    fn snapshot(&self, elapsed_time_s: f64, position_m: Vector2, velocity_m_s: Vector2) -> PropagatedState {
        PropagatedState {
            elapsed_time_s,
            position_m,
            velocity_m_s,
            specific_energy: self.specific_energy(position_m, velocity_m_s),
            specific_angular_momentum: self.specific_angular_momentum(position_m, velocity_m_s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::CelestialBody;
    use crate::physics::orbit::Orbit;

    fn earth_mu() -> f64 {
        CelestialBody::earth().gravitational_parameter
    }

    #[test]
    fn circular_orbit_returns_to_start_after_one_period() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();

        let orbit = Orbit::from_periapsis_apoapsis(r, r, earth_mu());
        let period_s = orbit.period_s().unwrap();

        let propagator = Propagator::new(earth_mu());
        let initial_position = Vector2::new(r, 0.0);
        let initial_velocity = Vector2::new(0.0, v);

        let states = propagator.propagate(initial_position, initial_velocity, period_s, 1.0);
        let final_state = states.last().unwrap();

        let position_error = ((final_state.position_m.x - initial_position.x).powi(2)
            + (final_state.position_m.y - initial_position.y).powi(2))
        .sqrt();
        let velocity_error = ((final_state.velocity_m_s.x - initial_velocity.x).powi(2)
            + (final_state.velocity_m_s.y - initial_velocity.y).powi(2))
        .sqrt();

        assert!(position_error < 0.001, "position error was {position_error} m");
        assert!(velocity_error < 0.00001, "velocity error was {velocity_error} m/s");
    }

    #[test]
    fn eccentric_orbit_returns_to_start_after_one_period() {
        let earth = CelestialBody::earth();
        let rp = earth.radius_m + 200_000.0;
        let ra = earth.radius_m + 2_000_000.0;
        let orbit = Orbit::from_periapsis_apoapsis(rp, ra, earth_mu());
        let period_s = orbit.period_s().unwrap();
        let vp = orbit.velocity_at_periapsis_m_s();

        let propagator = Propagator::new(earth_mu());
        let initial_position = Vector2::new(rp, 0.0);
        let initial_velocity = Vector2::new(0.0, vp);

        let states = propagator.propagate(initial_position, initial_velocity, period_s, 1.0);
        let final_state = states.last().unwrap();

        let position_error = ((final_state.position_m.x - initial_position.x).powi(2)
            + (final_state.position_m.y - initial_position.y).powi(2))
        .sqrt();

        assert!(position_error < 0.01, "position error was {position_error} m");
    }

    #[test]
    fn energy_and_angular_momentum_are_conserved_during_propagation() {
        let earth = CelestialBody::earth();
        let rp = earth.radius_m + 300_000.0;
        let ra = earth.radius_m + 800_000.0;
        let orbit = Orbit::from_periapsis_apoapsis(rp, ra, earth_mu());
        let period_s = orbit.period_s().unwrap();
        let vp = orbit.velocity_at_periapsis_m_s();

        let propagator = Propagator::new(earth_mu());
        let states = propagator.propagate(Vector2::new(rp, 0.0), Vector2::new(0.0, vp), period_s, 1.0);

        let initial_energy = states.first().unwrap().specific_energy;
        let initial_h = states.first().unwrap().specific_angular_momentum;

        for state in &states {
            assert!(
                (state.specific_energy - initial_energy).abs() < 1.0,
                "energy drifted to {} from {}",
                state.specific_energy,
                initial_energy
            );
            assert!(
                (state.specific_angular_momentum - initial_h).abs() < 1.0,
                "angular momentum drifted to {} from {}",
                state.specific_angular_momentum,
                initial_h
            );
        }
    }

    #[test]
    fn quarter_orbit_of_circular_orbit_ends_up_perpendicular() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();
        let orbit = Orbit::from_periapsis_apoapsis(r, r, earth_mu());
        let quarter_period_s = orbit.period_s().unwrap() / 4.0;

        let propagator = Propagator::new(earth_mu());
        let states = propagator.propagate(Vector2::new(r, 0.0), Vector2::new(0.0, v), quarter_period_s, 1.0);
        let final_state = states.last().unwrap();

        let final_radius = final_state.position_m.magnitude();
        assert!((final_radius - r).abs() < 1.0);
        assert!(final_state.position_m.x.abs() < final_state.position_m.y.abs() * 0.01);
        assert!(final_state.position_m.y > 0.0);
    }

    #[test]
    fn final_snapshot_lands_exactly_on_requested_total_time() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();

        let propagator = Propagator::new(earth_mu());
        let states = propagator.propagate(Vector2::new(r, 0.0), Vector2::new(0.0, v), 137.0, 10.0);
        let final_state = states.last().unwrap();

        assert_eq!(final_state.elapsed_time_s, 137.0);
    }

    // e=0.987, ~41.5 day period - numbers verified independently, see DEVLOG
    #[test]
    fn highly_eccentric_orbit_60s_step_shows_expected_large_error() {
        let position = Vector2::new(6_578_137.0, 0.0);
        let velocity = Vector2::new(0.0, 10_972.805371);
        let total_time_s = 3_587_170.629;

        let propagator = Propagator::new(earth_mu());
        let states = propagator.propagate(position, velocity, total_time_s, 60.0);
        let final_state = states.last().unwrap();

        let pos_error_km = ((final_state.position_m.x - position.x).powi(2)
            + (final_state.position_m.y - position.y).powi(2))
        .sqrt()
            / 1000.0;

        assert!(pos_error_km > 2000.0 && pos_error_km < 3000.0, "position error was {pos_error_km} km");

        let initial_energy = velocity.magnitude().powi(2) / 2.0 - earth_mu() / position.magnitude();
        let drift = final_state.specific_energy - initial_energy;
        assert!(drift.abs() > 5.0, "expected a large energy drift at dt=60s, got {drift}");
    }

    #[test]
    fn highly_eccentric_orbit_10s_step_stays_tight() {
        let position = Vector2::new(6_578_137.0, 0.0);
        let velocity = Vector2::new(0.0, 10_972.805371);
        let total_time_s = 3_587_170.629;

        let propagator = Propagator::new(earth_mu());
        let states = propagator.propagate(position, velocity, total_time_s, 10.0);
        let final_state = states.last().unwrap();

        let pos_error_km = ((final_state.position_m.x - position.x).powi(2)
            + (final_state.position_m.y - position.y).powi(2))
        .sqrt()
            / 1000.0;

        assert!(pos_error_km < 1.0, "position error was {pos_error_km} km");

        let initial_energy = velocity.magnitude().powi(2) / 2.0 - earth_mu() / position.magnitude();
        let drift = final_state.specific_energy - initial_energy;
        assert!(drift.abs() < 0.01, "expected energy drift under 0.01 J/kg at dt=10s, got {drift}");
    }
}
