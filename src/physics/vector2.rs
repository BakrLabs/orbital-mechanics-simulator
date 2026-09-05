// small enough not to justify pulling in a math crate
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

    // 2D cross product - really just the z-component of the 3D
    // version, which is what specific angular momentum needs
    pub fn cross(&self, other: &Vector2) -> f64 {
        self.x * other.y - self.y * other.x
    }
}
