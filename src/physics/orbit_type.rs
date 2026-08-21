use std::fmt;

/// What kind of conic section an orbit actually is, based on
/// eccentricity. This matters because periapsis/apoapsis/period only
/// make full sense for a closed (elliptical) orbit - a hyperbolic
/// trajectory only has a periapsis, and asking for its "period"
/// doesn't mean anything, since it never comes back around.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OrbitType {
    /// e very close to 0. Rare to hit exactly, but common enough
    /// (near-circular orbits) that it's worth calling out rather
    /// than just saying "elliptical" every time.
    Circular,
    Elliptical,
    Parabolic,
    Hyperbolic,
}

impl OrbitType {
    /// Classifies eccentricity into an orbit type. Circular gets a
    /// small tolerance around zero rather than requiring an exact
    /// match, since floating-point eccentricity from a position/
    /// velocity calculation is basically never going to land on
    /// precisely 0.0 even for an orbit that's circular in practice.
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
