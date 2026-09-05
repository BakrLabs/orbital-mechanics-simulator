use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OrbitType {
    Circular,
    Elliptical,
    Parabolic,
    Hyperbolic,
}

impl OrbitType {
    // fp eccentricity rarely lands on exactly 0 or 1, even when it should
    pub fn from_eccentricity(e: f64) -> Self {
        const CIRCULAR_TOLERANCE: f64 = 1e-6;
        const PARABOLIC_TOLERANCE: f64 = 1e-6;

        if e.abs() < CIRCULAR_TOLERANCE {
            OrbitType::Circular
        } else if (e - 1.0).abs() < PARABOLIC_TOLERANCE {
            OrbitType::Parabolic
        } else if e < 1.0 {
            OrbitType::Elliptical
        } else {
            OrbitType::Hyperbolic
        }
    }

    pub fn is_bound(&self) -> bool {
        matches!(self, OrbitType::Circular | OrbitType::Elliptical)
    }
}

impl fmt::Display for OrbitType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label = match self {
            OrbitType::Circular => "Circular",
            OrbitType::Elliptical => "Elliptical",
            OrbitType::Parabolic => "Parabolic",
            OrbitType::Hyperbolic => "Hyperbolic",
        };
        write!(f, "{}", label)
    }
}
