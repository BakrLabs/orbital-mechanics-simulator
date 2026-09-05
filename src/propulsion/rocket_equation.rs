use std::fmt;

use crate::physics::constants::STANDARD_GRAVITY;

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

// once you know two of {m0, mf, dv, Isp} you can get the rest, so
// both solve directions fill in all four fields regardless of which
// was the input
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

// mf = m0 / exp(dv / (Isp * g0))
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

// dv = Isp * g0 * ln(m0 / mf)
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

    #[test]
    fn propellant_required_matches_spec_example() {
        let result = propellant_required(9400.0, 450.0, 10000.0).unwrap();
        assert!((result.final_mass_kg - 1188.285).abs() < 0.01);
        assert!((result.propellant_mass_kg() - 8811.715).abs() < 0.01);
        assert!((result.mass_ratio() - 8.4155).abs() < 0.001);
    }

    // solving forward then feeding the result back through the
    // inverse should return the original dv - these two functions
    // are inverses of each other by construction
    #[test]
    fn achievable_delta_v_is_the_exact_inverse_of_propellant_required() {
        let forward = propellant_required(9400.0, 450.0, 10000.0).unwrap();
        let backward = achievable_delta_v(450.0, forward.initial_mass_kg, forward.final_mass_kg).unwrap();
        assert!((backward.delta_v_m_s - 9400.0).abs() < 0.001);
    }

    #[test]
    fn zero_propellant_gives_zero_delta_v() {
        let result = achievable_delta_v(450.0, 10000.0, 10000.0).unwrap();
        assert_eq!(result.delta_v_m_s, 0.0);
    }

    #[test]
    fn higher_isp_requires_less_propellant_for_same_delta_v() {
        let low_isp = propellant_required(9400.0, 300.0, 10000.0).unwrap();
        let high_isp = propellant_required(9400.0, 450.0, 10000.0).unwrap();
        assert!(high_isp.propellant_mass_kg() < low_isp.propellant_mass_kg());
    }

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
