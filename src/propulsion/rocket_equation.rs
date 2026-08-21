use std::fmt;

use crate::physics::constants::STANDARD_GRAVITY;

/// Everything that can go wrong feeding numbers into the rocket
/// equation. Masses and Isp have to be positive to mean anything
/// physically, and final mass can't exceed initial mass (that would
/// mean negative propellant) - all of this is user input, so none of
/// it gets to panic; it comes back as one of these instead.
#[derive(Debug, PartialEq)]
pub enum RocketEquationError {
    NonPositiveMass,
    NonPositiveIsp,
    FinalMassExceedsInitialMass,
    NonPositiveDeltaV,
}

impl fmt::Display for RocketEquationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            RocketEquationError::NonPositiveMass => "Mass must be greater than zero.",
            RocketEquationError::NonPositiveIsp => "Specific impulse must be greater than zero.",
            RocketEquationError::FinalMassExceedsInitialMass => {
                "Final mass can't be greater than initial mass - that would mean negative propellant."
            }
            RocketEquationError::NonPositiveDeltaV => "Delta-v must be greater than zero.",
        };
        write!(f, "{}", message)
    }
}

/// The result of a rocket equation calculation - always ends up with
/// all four figures filled in regardless of which one was "given" and
/// which was "solved for", since once you know two of {m0, mf, Δv,
/// Isp} you can get the rest.
#[derive(Debug)]
pub struct PropulsionResult {
    pub delta_v_m_s: f64,
    pub specific_impulse_s: f64,
    pub initial_mass_kg: f64,
    pub final_mass_kg: f64,
}

impl PropulsionResult {
    pub fn propellant_mass_kg(&self) -> f64 {
        self.initial_mass_kg - self.final_mass_kg
    }

    pub fn mass_ratio(&self) -> f64 {
        self.initial_mass_kg / self.final_mass_kg
    }

    pub fn exhaust_velocity_m_s(&self) -> f64 {
        self.specific_impulse_s * STANDARD_GRAVITY
    }
}

/// Solves for final mass and propellant mass, given Δv, Isp, and
/// initial mass - the direction the spec calls "Propellant Required":
/// you know how much velocity change you need and what engine you're
/// using, and you want to know how much fuel that costs.
///
/// mf = m0 / exp(Δv / (Isp * g0))
pub fn propellant_required(
    delta_v_m_s: f64,
    specific_impulse_s: f64,
    initial_mass_kg: f64,
) -> Result<PropulsionResult, RocketEquationError> {
    if initial_mass_kg <= 0.0 {
        return Err(RocketEquationError::NonPositiveMass);
    }
    if specific_impulse_s <= 0.0 {
        return Err(RocketEquationError::NonPositiveIsp);
    }
    if delta_v_m_s <= 0.0 {
        return Err(RocketEquationError::NonPositiveDeltaV);
    }

    let exhaust_velocity = specific_impulse_s * STANDARD_GRAVITY;
    let final_mass_kg = initial_mass_kg / (delta_v_m_s / exhaust_velocity).exp();

    Ok(PropulsionResult {
        delta_v_m_s,
        specific_impulse_s,
        initial_mass_kg,
        final_mass_kg,
    })
}

/// Solves for achievable Δv, given Isp, initial mass, and final mass -
/// the other direction: you know how much fuel you're carrying and
/// what engine you have, and you want to know how much velocity
/// change that actually buys you.
///
/// Δv = Isp * g0 * ln(m0 / mf)
pub fn achievable_delta_v(
    specific_impulse_s: f64,
    initial_mass_kg: f64,
    final_mass_kg: f64,
) -> Result<PropulsionResult, RocketEquationError> {
    if initial_mass_kg <= 0.0 || final_mass_kg <= 0.0 {
        return Err(RocketEquationError::NonPositiveMass);
    }
    if specific_impulse_s <= 0.0 {
        return Err(RocketEquationError::NonPositiveIsp);
    }
    if final_mass_kg > initial_mass_kg {
        return Err(RocketEquationError::FinalMassExceedsInitialMass);
    }

    let exhaust_velocity = specific_impulse_s * STANDARD_GRAVITY;
    let delta_v_m_s = exhaust_velocity * (initial_mass_kg / final_mass_kg).ln();

    Ok(PropulsionResult {
        delta_v_m_s,
        specific_impulse_s,
        initial_mass_kg,
        final_mass_kg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example from the spec: Δv = 9.40 km/s, Isp = 450s,
    // m0 = 10000kg. Verified this one by hand and it checks out
    // cleanly, unlike some of the other worked examples in this
    // project's spec - see the README for the ones that didn't.
    #[test]
    fn propellant_required_matches_spec_example() {
        let result = propellant_required(9400.0, 450.0, 10000.0).unwrap();
        assert!((result.final_mass_kg - 1188.285).abs() < 0.01);
        assert!((result.propellant_mass_kg() - 8811.715).abs() < 0.01);
        assert!((result.mass_ratio() - 8.4155).abs() < 0.001);
    }

    #[test]
    fn achievable_delta_v_is_the_exact_inverse_of_propellant_required() {
        // If you solve "how much propellant for this Δv" and then feed
        // the resulting masses back into "what Δv does this propellant
        // buy", you should get the original Δv back - the two
        // functions are inverses of each other by construction, so
        // this is really a round-trip consistency check rather than a
        // test of any specific number.
        let forward = propellant_required(9400.0, 450.0, 10000.0).unwrap();
        let backward = achievable_delta_v(450.0, forward.initial_mass_kg, forward.final_mass_kg).unwrap();
        assert!((backward.delta_v_m_s - 9400.0).abs() < 0.001);
    }

    #[test]
    fn zero_propellant_gives_zero_delta_v() {
        // Final mass equal to initial mass means no propellant was
        // burned at all - ln(1) = 0, so Δv should come out to exactly
        // zero, not some tiny nonzero residue from floating point.
        let result = achievable_delta_v(450.0, 10000.0, 10000.0).unwrap();
        assert_eq!(result.delta_v_m_s, 0.0);
    }

    #[test]
    fn higher_isp_requires_less_propellant_for_same_delta_v() {
        let low_isp = propellant_required(9400.0, 300.0, 10000.0).unwrap();
        let high_isp = propellant_required(9400.0, 450.0, 10000.0).unwrap();
        assert!(high_isp.propellant_mass_kg() < low_isp.propellant_mass_kg());
    }

    // --- Error handling: never unwrap/panic on bad user input ---

    #[test]
    fn negative_initial_mass_is_rejected() {
        let result = propellant_required(9400.0, 450.0, -10000.0);
        assert_eq!(result.unwrap_err(), RocketEquationError::NonPositiveMass);
    }

    #[test]
    fn zero_initial_mass_is_rejected() {
        let result = propellant_required(9400.0, 450.0, 0.0);
        assert_eq!(result.unwrap_err(), RocketEquationError::NonPositiveMass);
    }

    #[test]
    fn negative_isp_is_rejected() {
        let result = propellant_required(9400.0, -450.0, 10000.0);
        assert_eq!(result.unwrap_err(), RocketEquationError::NonPositiveIsp);
    }

    #[test]
    fn negative_delta_v_is_rejected() {
        let result = propellant_required(-9400.0, 450.0, 10000.0);
        assert_eq!(result.unwrap_err(), RocketEquationError::NonPositiveDeltaV);
    }

    #[test]
    fn final_mass_greater_than_initial_mass_is_rejected() {
        let result = achievable_delta_v(450.0, 5000.0, 10000.0);
        assert_eq!(result.unwrap_err(), RocketEquationError::FinalMassExceedsInitialMass);
    }

    #[test]
    fn negative_final_mass_is_rejected() {
        let result = achievable_delta_v(450.0, 10000.0, -1.0);
        assert_eq!(result.unwrap_err(), RocketEquationError::NonPositiveMass);
    }
}
