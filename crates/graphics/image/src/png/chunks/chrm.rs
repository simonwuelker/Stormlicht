//! [cHRM](https://www.w3.org/TR/png/#11cHRM) chunk

#[derive(Clone, Copy, Debug)]
pub struct Chromacities {
    pub white_point: (u32, u32),
    pub red_point: (u32, u32),
    pub green_point: (u32, u32),
    pub blue_point: (u32, u32),
}

impl Chromacities {
    pub fn new(
        white_point: (u32, u32),
        red_point: (u32, u32),
        green_point: (u32, u32),
        blue_point: (u32, u32),
    ) -> Self {
        Self {
            white_point,
            red_point,
            green_point,
            blue_point,
        }
    }
}
