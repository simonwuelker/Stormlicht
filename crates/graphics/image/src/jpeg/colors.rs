use crate::Rgbaf32;

/// Converts from [YCbCr](https://en.wikipedia.org/wiki/YCbCr) to RGB
#[must_use]
pub fn ycbcr_to_rgb(y: f32, cb: f32, cr: f32) -> Rgbaf32 {
    let red = cr * (2. - 2. * 0.299) + y;
    let blue = cb * (2. - 2. * 0.114) + y;
    let green = (y - 0.114 * blue - 0.299 * red) / 0.587;

    Rgbaf32::rgb(red, green, blue)
}
