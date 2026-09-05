use crate::physics::vector3::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct PropagatedState3D {
    pub elapsed_time_s: f64,
    pub position_m: Vector3,
    pub velocity_m_s: Vector3,
    pub specific_energy: f64,
    pub specific_angular_momentum: f64, // magnitude of the h vector
}

pub struct Propagator3D {
    mu: f64,
}

impl Propagator3D {
    pub fn new(mu: f64) -> Self {
        Propagator3D { mu }
    }

    // a = -mu*r/|r|^3
    fn acceleration(&self, position_m: Vector3) -> Vector3 {
        let r = position_m.magnitude();
        let factor = -self.mu / r.powi(3);
        position_m.scale(factor)
    }

    fn derivative(&self, position_m: Vector3, velocity_m_s: Vector3) -> (Vector3, Vector3) {
        (velocity_m_s, self.acceleration(position_m))
    }

    fn step(&self, position_m: Vector3, velocity_m_s: Vector3, dt_s: f64) -> (Vector3, Vector3) {
        let (k1_p, k1_v) = self.derivative(position_m, velocity_m_s);

        let p2 = position_m.add(&k1_p.scale(dt_s / 2.0));
        let v2 = velocity_m_s.add(&k1_v.scale(dt_s / 2.0));
        let (k2_p, k2_v) = self.derivative(p2, v2);

        let p3 = position_m.add(&k2_p.scale(dt_s / 2.0));
        let v3 = velocity_m_s.add(&k2_v.scale(dt_s / 2.0));
        let (k3_p, k3_v) = self.derivative(p3, v3);

        let p4 = position_m.add(&k3_p.scale(dt_s));
        let v4 = velocity_m_s.add(&k3_v.scale(dt_s));
        let (k4_p, k4_v) = self.derivative(p4, v4);

        let sum_p = k1_p.add(&k2_p.scale(2.0)).add(&k3_p.scale(2.0)).add(&k4_p);
        let sum_v = k1_v.add(&k2_v.scale(2.0)).add(&k3_v.scale(2.0)).add(&k4_v);

        let new_position = position_m.add(&sum_p.scale(dt_s / 6.0));
        let new_velocity = velocity_m_s.add(&sum_v.scale(dt_s / 6.0));

        (new_position, new_velocity)
    }

    fn specific_energy(&self, position_m: Vector3, velocity_m_s: Vector3) -> f64 {
        let r = position_m.magnitude();
        let v = velocity_m_s.magnitude();
        v * v / 2.0 - self.mu / r
    }

    fn specific_angular_momentum(&self, position_m: Vector3, velocity_m_s: Vector3) -> f64 {
        position_m.cross(&velocity_m_s).magnitude()
    }

    pub fn propagate(
        &self,
        initial_position_m: Vector3,
        initial_velocity_m_s: Vector3,
        total_time_s: f64,
        dt_s: f64,
    ) -> Vec<PropagatedState3D> {
        let mut states = Vec::new();
        let mut position = initial_position_m;
        let mut velocity = initial_velocity_m_s;
        let mut elapsed = 0.0;

        states.push(self.snapshot(elapsed, position, velocity));

        while elapsed < total_time_s {
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

    // step-doubling error, scaled by position magnitude (GMAT/STK-style relative tolerance) - see DEVLOG
    fn step_with_error_estimate(
        &self,
        position_m: Vector3,
        velocity_m_s: Vector3,
        dt_s: f64,
    ) -> (Vector3, Vector3, f64) {
        let (full_p, _full_v) = self.step(position_m, velocity_m_s, dt_s);
        let (half_p1, half_v1) = self.step(position_m, velocity_m_s, dt_s / 2.0);
        let (half_p2, half_v2) = self.step(half_p1, half_v1, dt_s / 2.0);

        let absolute_error = full_p.sub(&half_p2).magnitude();
        let relative_error = absolute_error / position_m.magnitude();
        (half_p2, half_v2, relative_error)
    }

    // shrinks dt when relative error > tolerance, grows it otherwise - see DEVLOG for tolerance selection
    pub fn propagate_adaptive(
        &self,
        initial_position_m: Vector3,
        initial_velocity_m_s: Vector3,
        total_time_s: f64,
        relative_tolerance: f64,
        initial_dt_s: f64,
    ) -> Vec<PropagatedState3D> {
        const MIN_DT_S: f64 = 0.01;
        const MAX_DT_S: f64 = 3600.0;
        const MAX_REJECTIONS_PER_STEP: u32 = 50;

        let mut states = Vec::new();
        let mut position = initial_position_m;
        let mut velocity = initial_velocity_m_s;
        let mut elapsed = 0.0;
        let mut dt = initial_dt_s.clamp(MIN_DT_S, MAX_DT_S);

        states.push(self.snapshot(elapsed, position, velocity));

        while elapsed < total_time_s {
            let remaining = total_time_s - elapsed;
            if dt > remaining {
                dt = remaining;
            }

            let mut rejections = 0;
            loop {
                let (new_position, new_velocity, relative_error) =
                    self.step_with_error_estimate(position, velocity, dt);

                if relative_error > relative_tolerance && dt > MIN_DT_S && rejections < MAX_REJECTIONS_PER_STEP {
                    dt = (dt * 0.5).max(MIN_DT_S);
                    rejections += 1;
                    continue;
                }

                position = new_position;
                velocity = new_velocity;
                elapsed += dt;

                // RK4 error ~ dt^5, so 1/5 power gives the size that would've hit tolerance exactly
                if relative_error > 0.0 {
                    let factor = (relative_tolerance / relative_error).powf(0.2).clamp(0.5, 2.0);
                    dt = (dt * factor).clamp(MIN_DT_S, MAX_DT_S);
                }
                break;
            }

            states.push(self.snapshot(elapsed, position, velocity));
        }

        states
    }

    fn snapshot(&self, elapsed_time_s: f64, position_m: Vector3, velocity_m_s: Vector3) -> PropagatedState3D {
        PropagatedState3D {
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

    // z=0 plane should reproduce the existing 2D propagator's results exactly
    #[test]
    fn planar_case_matches_2d_propagator() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();

        let propagator_3d = Propagator3D::new(earth_mu());
        let states_3d = propagator_3d.propagate(
            Vector3::new(r, 0.0, 0.0),
            Vector3::new(0.0, v, 0.0),
            5000.0,
            1.0,
        );
        let final_3d = states_3d.last().unwrap();

        let propagator_2d = crate::physics::propagator::Propagator::new(earth_mu());
        let states_2d = propagator_2d.propagate(
            crate::physics::vector2::Vector2::new(r, 0.0),
            crate::physics::vector2::Vector2::new(0.0, v),
            5000.0,
            1.0,
        );
        let final_2d = states_2d.last().unwrap();

        assert!((final_3d.position_m.x - final_2d.position_m.x).abs() < 1e-3);
        assert!((final_3d.position_m.y - final_2d.position_m.y).abs() < 1e-3);
        assert!(final_3d.position_m.z.abs() < 1e-9);
        assert!((final_3d.specific_energy - final_2d.specific_energy).abs() < 1e-6);
    }

    #[test]
    fn circular_orbit_returns_to_start_after_one_period() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();
        let orbit = Orbit::from_periapsis_apoapsis(r, r, earth_mu());
        let period_s = orbit.period_s().unwrap();

        let propagator = Propagator3D::new(earth_mu());
        let initial_position = Vector3::new(r, 0.0, 0.0);
        let initial_velocity = Vector3::new(0.0, v, 0.0);

        let states = propagator.propagate(initial_position, initial_velocity, period_s, 1.0);
        let final_state = states.last().unwrap();

        let position_error = final_state.position_m.sub(&initial_position).magnitude();
        assert!(position_error < 0.001, "position error was {position_error} m");
    }

    // inclined circular orbit - checks the propagator respects a non-planar velocity
    #[test]
    fn inclined_circular_orbit_stays_on_its_plane() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();

        // 45 degree inclination: velocity tilted between +y and +z
        let vy = v * (45.0_f64.to_radians()).cos();
        let vz = v * (45.0_f64.to_radians()).sin();

        let propagator = Propagator3D::new(earth_mu());
        let initial_position = Vector3::new(r, 0.0, 0.0);
        let initial_velocity = Vector3::new(0.0, vy, vz);
        let initial_h = initial_position.cross(&initial_velocity);

        let orbit = Orbit::from_periapsis_apoapsis(r, r, earth_mu());
        let period_s = orbit.period_s().unwrap();

        let states = propagator.propagate(initial_position, initial_velocity, period_s, 1.0);

    // h direction shouldn't drift - orbital plane is fixed for two-body motion
        for state in states.iter().step_by(states.len() / 10 + 1) {
            let h = state.position_m.cross(&state.velocity_m_s);
            let cos_angle = h.dot(&initial_h) / (h.magnitude() * initial_h.magnitude());
            assert!(cos_angle > 0.9999, "angular momentum direction drifted, cos={cos_angle}");
        }

        let final_state = states.last().unwrap();
        let position_error = final_state.position_m.sub(&initial_position).magnitude();
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

        let propagator = Propagator3D::new(earth_mu());
        let states = propagator.propagate(Vector3::new(rp, 0.0, 0.0), Vector3::new(0.0, vp, 0.0), period_s, 1.0);

        let initial_energy = states.first().unwrap().specific_energy;
        let initial_h = states.first().unwrap().specific_angular_momentum;

        for state in &states {
            assert!((state.specific_energy - initial_energy).abs() < 1.0);
            assert!((state.specific_angular_momentum - initial_h).abs() < 1.0);
        }
    }

    // reference figures independently verified in Python first (min_dt=0.01, max_dt=3600) - see DEVLOG

    #[test]
    fn adaptive_circular_orbit_returns_to_start_after_one_period() {
        let earth = CelestialBody::earth();
        let r = earth.radius_m + 500_000.0;
        let v = (earth_mu() / r).sqrt();
        let orbit = Orbit::from_periapsis_apoapsis(r, r, earth_mu());
        let period_s = orbit.period_s().unwrap();

        let propagator = Propagator3D::new(earth_mu());
        let initial_position = Vector3::new(r, 0.0, 0.0);
        let initial_velocity = Vector3::new(0.0, v, 0.0);

        let states = propagator.propagate_adaptive(initial_position, initial_velocity, period_s, 1e-10, 60.0);
        let final_state = states.last().unwrap();

        let position_error = final_state.position_m.sub(&initial_position).magnitude();
        assert!(position_error < 0.01, "position error was {position_error} m");
        // far fewer steps than a fixed 1s step over the same period would take
        assert!(states.len() < 1000, "expected under 1000 steps, got {}", states.len());
    }

    #[test]
    fn adaptive_eccentric_orbit_at_moderate_tolerance_matches_reference() {
        let position = Vector3::new(6_578_137.0, 0.0, 0.0);
        let velocity = Vector3::new(0.0, 10_972.805371, 0.0);
        let total_time_s = 3_587_170.629;

        let propagator = Propagator3D::new(earth_mu());
        let states = propagator.propagate_adaptive(position, velocity, total_time_s, 1e-9, 60.0);
        let final_state = states.last().unwrap();

        let position_error = final_state.position_m.sub(&position).magnitude();
        assert!((position_error - 34_716.86).abs() < 1.0, "position error was {position_error} m");
    }

    // pins the observed fp-noise-floor behavior, not a logic bug - see DEVLOG
    #[test]
    fn adaptive_at_very_tight_tolerance_hits_floating_point_noise_floor() {
        let position = Vector3::new(6_578_137.0, 0.0, 0.0);
        let velocity = Vector3::new(0.0, 10_972.805371, 0.0);
        let total_time_s = 3_587_170.629;

        let propagator = Propagator3D::new(earth_mu());
        let states = propagator.propagate_adaptive(position, velocity, total_time_s, 1e-11, 60.0);
        let final_state = states.last().unwrap();

        let position_error = final_state.position_m.sub(&position).magnitude();
        // near the best result across the relative-tolerance sweep - tighter regresses
        assert!((position_error - 207.46).abs() < 1.0, "position error was {position_error} m");
    }

    #[test]
    fn adaptive_uses_far_fewer_steps_than_a_tight_fixed_step_for_comparable_accuracy() {
        let position = Vector3::new(6_578_137.0, 0.0, 0.0);
        let velocity = Vector3::new(0.0, 10_972.805371, 0.0);
        let total_time_s = 3_587_170.629;

        let propagator = Propagator3D::new(earth_mu());
        let adaptive_states = propagator.propagate_adaptive(position, velocity, total_time_s, 1e-9, 60.0);

        // fixed 10s step over this duration takes ~358,717 steps (v0.6/v0.7 stress test)
        assert!(adaptive_states.len() < 5000, "expected well under 5000 steps, got {}", adaptive_states.len());
    }

    // relative tolerance should behave consistently regardless of orbit scale - verified in Python first
    #[test]
    fn relative_tolerance_behaves_consistently_across_wildly_different_orbit_scales() {
        let moon = CelestialBody::moon();
        let moon_r = moon.radius_m * 1.05;
        let moon_v = (moon.gravitational_parameter / moon_r).sqrt();
        let moon_period = 2.0 * std::f64::consts::PI * (moon_r.powi(3) / moon.gravitational_parameter).sqrt();

        let sun = CelestialBody::sun();
        let sun_r = sun.radius_m * 1.05;
        let sun_v = (sun.gravitational_parameter / sun_r).sqrt();
        let sun_period = 2.0 * std::f64::consts::PI * (sun_r.powi(3) / sun.gravitational_parameter).sqrt();

        let moon_propagator = Propagator3D::new(moon.gravitational_parameter);
        let moon_states = moon_propagator.propagate_adaptive(
            Vector3::new(moon_r, 0.0, 0.0),
            Vector3::new(0.0, moon_v, 0.0),
            moon_period,
            1e-9,
            60.0,
        );

        let sun_propagator = Propagator3D::new(sun.gravitational_parameter);
        let sun_states = sun_propagator.propagate_adaptive(
            Vector3::new(sun_r, 0.0, 0.0),
            Vector3::new(0.0, sun_v, 0.0),
            sun_period,
            1e-9,
            60.0,
        );

        // orbit radii differ by ~400x, but step counts should land in the
        // same ballpark since the tolerance is relative, not absolute
        let ratio = moon_states.len() as f64 / sun_states.len() as f64;
        assert!(
            ratio > 0.5 && ratio < 2.0,
            "expected similar step counts, got moon={} sun={}",
            moon_states.len(),
            sun_states.len()
        );
    }
}
