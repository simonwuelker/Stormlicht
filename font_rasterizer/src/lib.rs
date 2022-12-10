pub mod tables;
pub mod ttf;

use std::fmt;

pub struct Vec2D {
    x: f32,
    y: f32,
}

impl fmt::Debug for Vec2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Vec 2D")
            .field(&self.x)
            .field(&self.y)
            .finish()
    }
}
