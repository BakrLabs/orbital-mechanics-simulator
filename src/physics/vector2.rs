/// A plain 2D vector. Used for position and velocity in the
/// position/velocity orbit-definition method - nothing about this
/// is orbit-specific, it's just x/y with a magnitude helper, but it
/// didn't seem worth pulling in an external math crate for two
/// floats and a sqrt.
#[derive(Clone, Copy, Debug)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Vector2 { x, y }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 2D "cross product" - not a real cross product since these are
    /// 2D vectors, but this is the standard shorthand for the z-component
    /// of what x-times-y would be in 3D, which is exactly what specific
    /// angular momentum needs here.
    pub fn cross(&self, other: &Vector2) -> f64 {
        self.x * other.y - self.y * other.x
    }
}
