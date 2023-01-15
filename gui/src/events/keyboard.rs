#[derive(Clone, Copy, Debug)]
pub struct Modifier(u8);

impl Modifier {
    pub const LEFT_SHIFT: Modifier = Modifier(1 << 0);
    pub const RIGHT_SHIFT: Modifier = Modifier(1 << 1);
    pub const SHIFT: Modifier = Self::LEFT_SHIFT.or(Self::RIGHT_SHIFT);

    pub const LEFT_CONTROL: Modifier = Modifier(1 << 2);
    pub const RIGHT_CONTROL: Modifier = Modifier(1 << 3);
    pub const CONTROL: Modifier = Self::LEFT_CONTROL.or(Self::RIGHT_CONTROL);

    pub const LEFT_ALT: Modifier = Modifier(1 << 4);
    pub const RIGHT_ALT: Modifier = Modifier(1 << 5);
    pub const ALT: Modifier = Self::LEFT_ALT.or(Self::RIGHT_ALT);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn is_set(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn set(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub const fn or(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for Modifier {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.or(rhs)
    }
}

use sdl2::keyboard::Mod as SDL2Mod;

impl From<SDL2Mod> for Modifier {
    fn from(value: SDL2Mod) -> Self {
        let mut result = Self::empty();

        if value.contains(SDL2Mod::LSHIFTMOD) {
            result.set(Self::LEFT_SHIFT);
        }

        if value.contains(SDL2Mod::RSHIFTMOD) {
            result.set(Self::RIGHT_SHIFT);
        }

        if value.contains(SDL2Mod::LCTRLMOD) {
            result.set(Self::LEFT_CONTROL);
        }

        if value.contains(SDL2Mod::RCTRLMOD) {
            result.set(Self::RIGHT_CONTROL);
        }

        if value.contains(SDL2Mod::LALTMOD) {
            result.set(Self::LEFT_ALT);
        }

        if value.contains(SDL2Mod::RALTMOD) {
            result.set(Self::RIGHT_ALT);
        }
        result
    }
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Delete,
    Escape,
}

use sdl2::keyboard::Keycode as SDL2KeyCode;

impl TryFrom<SDL2KeyCode> for KeyCode {
    type Error = ();

    fn try_from(value: SDL2KeyCode) -> Result<Self, Self::Error> {
        match value {
            SDL2KeyCode::A => Ok(Self::A),
            SDL2KeyCode::B => Ok(Self::B),
            SDL2KeyCode::C => Ok(Self::C),
            SDL2KeyCode::D => Ok(Self::D),
            SDL2KeyCode::E => Ok(Self::E),
            SDL2KeyCode::F => Ok(Self::F),
            SDL2KeyCode::G => Ok(Self::G),
            SDL2KeyCode::H => Ok(Self::H),
            SDL2KeyCode::I => Ok(Self::I),
            SDL2KeyCode::J => Ok(Self::J),
            SDL2KeyCode::K => Ok(Self::K),
            SDL2KeyCode::L => Ok(Self::L),
            SDL2KeyCode::M => Ok(Self::M),
            SDL2KeyCode::N => Ok(Self::N),
            SDL2KeyCode::O => Ok(Self::O),
            SDL2KeyCode::P => Ok(Self::P),
            SDL2KeyCode::Q => Ok(Self::Q),
            SDL2KeyCode::R => Ok(Self::R),
            SDL2KeyCode::S => Ok(Self::S),
            SDL2KeyCode::T => Ok(Self::T),
            SDL2KeyCode::U => Ok(Self::U),
            SDL2KeyCode::V => Ok(Self::V),
            SDL2KeyCode::W => Ok(Self::W),
            SDL2KeyCode::X => Ok(Self::X),
            SDL2KeyCode::Y => Ok(Self::Y),
            SDL2KeyCode::Z => Ok(Self::Z),
            SDL2KeyCode::Delete => Ok(Self::Delete),
            SDL2KeyCode::Escape => Ok(Self::Escape),
            _ => Err(()),
        }
    }
}
