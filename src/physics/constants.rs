/// Standard gravity, in m/s^2 - used in the rocket equation to convert
/// specific impulse (given in seconds) into an actual exhaust velocity.
/// This is a defined constant, not a measured one (it's fixed by
/// international agreement as exactly this value), so there's no
/// "more precise" version to reach for.
pub const STANDARD_GRAVITY: f64 = 9.80665;
