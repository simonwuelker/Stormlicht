use crate::primitives::Point;

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    MouseDown {
        button: MouseButton,
        at: Point,
    },
    MouseUp {
        button: MouseButton,
        at: Point,
    },
    Quit,
}

impl Event {
    pub fn location(&self) -> Option<Point> {
        match self {
            Self::MouseDown { at, .. } | Self::MouseUp { at, .. } => Some(*at),
            _ => None,
        }
    }
}

use sdl2::event::Event as SDL2Event;

impl TryFrom<SDL2Event> for Event {
    type Error = ();

    fn try_from(value: SDL2Event) -> Result<Event, Self::Error> {
        match value {
            SDL2Event::Quit { .. } => Ok(Self::Quit),
            SDL2Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                Ok(Self::MouseUp {
                    button: mouse_btn.try_into().unwrap(),
                    at: Point::new(x, y),
                })
            },
            SDL2Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                Ok(Self::MouseDown {
                    button: mouse_btn.try_into().unwrap(),
                    at: Point::new(x, y),
                })
            },
            _ => Err(())
        }
    }
}

use sdl2::mouse::MouseButton as SDL2MouseButton;

impl TryFrom<SDL2MouseButton> for MouseButton {
    type Error = ();

    fn try_from(value: SDL2MouseButton) -> Result<Self, Self::Error> {
        match value {
            SDL2MouseButton::Left => Ok(Self::Left),
            SDL2MouseButton::Middle => Ok(Self::Middle),
            SDL2MouseButton::Right => Ok(Self::Right),
            _ => Err(())
        }
    }
}