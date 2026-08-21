/// A central body for orbital calculations. Nothing fancy - just
/// enough to keep radius and mu out of the calculation code so we're
/// not hard-coding Earth's numbers all over the place. If a Moon or
/// Mars option gets added down the line, it's just another constructor
/// here, not a search-and-replace through the rest of the app.
pub struct CelestialBody {
    pub name: &'static str,
    /// Mean radius, in meters.
    pub radius_m: f64,
    /// Standard gravitational parameter (mu = GM), in m^3/s^2.
    pub gravitational_parameter: f64,
}

impl CelestialBody {
    /// Earth, using mean equatorial radius and standard mu.
    /// Values in km / km^3/s^2 per most orbital mechanics references,
    /// converted to SI (meters) here since that's what the physics
    /// module works in internally.
    pub fn earth() -> Self {
        CelestialBody {
            name: "Earth",
            radius_m: 6_378_137.0,
            gravitational_parameter: 3.986_004_418e14,
        }
    }
}
