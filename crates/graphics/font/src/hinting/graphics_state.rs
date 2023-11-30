use super::{interpreter::Zone, F26Dot6};

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html>
#[derive(Clone, Copy, Debug)]
pub struct GraphicsState {
    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#auto%20flip>
    pub auto_flip: bool,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#control_value_cut-in>
    pub control_value_cut_in: F26Dot6,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#delta%20base>
    pub delta_base: u32,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#delta%20shift>
    pub delta_shift: u32,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#dual%20projection%20vector>
    pub dual_projection_vector: Option<math::Vec2D>,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#freedom%20vector>
    pub freedom_vector: math::Vec2D,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#instruct%20control>
    pub instruct_control: bool,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#loop>
    pub loop_n: u32,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#minimum%20distance>
    pub minimum_distance: F26Dot6,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#projection%20vector>
    pub projection_vector: math::Vec2D,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#round%20state>
    pub round_state: u32,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#rp0>
    pub rp0: u32,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#rp1>
    pub rp1: u32,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#rp2>
    pub rp2: u32,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#scan%20control>
    pub scan_control: bool,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#single_width_cut_in>
    pub single_width_cut_in: F26Dot6,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#single_width_value>
    pub single_width_value: F26Dot6,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#zp0>
    pub zp0: Zone,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#zp1>
    pub zp1: Zone,

    /// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM04/Chap4.html#zp2>
    pub zp2: Zone,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            auto_flip: true,
            control_value_cut_in: F26Dot6::from(17. / 16.),
            delta_base: 9,
            delta_shift: 3,
            dual_projection_vector: None,
            freedom_vector: math::Vec2D::new(1., 0.),
            instruct_control: false,
            loop_n: 1,
            minimum_distance: F26Dot6::from(1.),
            projection_vector: math::Vec2D::new(1., 0.),
            round_state: 1,
            rp0: 0,
            rp1: 0,
            rp2: 0,
            scan_control: false,
            single_width_cut_in: F26Dot6::from(0.),
            single_width_value: F26Dot6::from(0.),
            zp0: Zone::Glyph,
            zp1: Zone::Glyph,
            zp2: Zone::Glyph,
        }
    }
}
