use std::marker::PhantomData;

use crate::{
    application::RepaintState,
    layout::{Sizing, Widget},
    GuiError,
};

use canvas::{Canvas, PixelFormat};
use font::Font;
use sdl2::{
    keyboard::{Keycode, Mod},
    pixels::PixelFormatEnum,
    rect::Rect,
    render::Canvas as SDLCanvas,
    video::Window,
};

pub struct Input<M> {
    bounding_box: Option<Rect>,
    width: Sizing,
    height: Sizing,
    text: String,
    phantom: PhantomData<M>,
}

impl<M> Default for Input<M> {
    fn default() -> Self {
        Self {
            bounding_box: None,
            width: Sizing::default(),
            height: Sizing::Exactly(20),
            text: "".to_string(),
            phantom: PhantomData,
        }
    }
}

impl<M> Widget for Input<M> {
    type Message = M;

    fn bounding_box(&self) -> Option<Rect> {
        self.bounding_box
    }

    fn width(&self) -> Sizing {
        self.width
    }

    fn height(&self) -> Sizing {
        self.height
    }

    fn render_to(&mut self, surface: &mut SDLCanvas<Window>, into: Rect) -> Result<(), GuiError> {
        // White Background
        // Black text, default font

        // TODO this is extremely inefficient (we create a new texture on every render call)
        // but since we don't do rerendering right now anyways it shouldn't matter and i want to
        // figure out the general architecture first.
        let font = Font::default();
        let texture_creator = surface.texture_creator();
        let mut texture = texture_creator.create_texture_target(
            Some(PixelFormatEnum::RGB24),
            into.width(),
            into.height(),
        ).unwrap();
        let mut canvas = Canvas::new(
            vec![
                255;
                into.width() as usize * into.height() as usize * PixelFormat::RGB8.pixel_size()
            ],
            into.width() as usize,
            into.height() as usize,
            PixelFormat::RGB8,
        );
        font.rasterize(&self.text, &mut canvas, (0, 0), 30.);

        texture.update(
            None,
            canvas.data(),
            into.width() as usize * canvas.format().pixel_size(),
        ).unwrap();

        surface
            .copy(&texture, None, into)
            .map_err(GuiError::SDL)?;
        Ok(())
    }

    fn invalidate_layout(&mut self) {
        self.bounding_box = None;
    }

    fn compute_layout(&mut self, into: Rect) {
        self.bounding_box = Some(into);
    }

    fn on_key_down(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
        message_queue: crate::application::AppendOnlyQueue<Self::Message>,
    ) -> RepaintState {
        _ = message_queue;
        let c = match keycode {
            Keycode::A => 'a',
            Keycode::B => 'b',
            Keycode::C => 'c',
            Keycode::D => 'd',
            Keycode::E => 'e',
            Keycode::F => 'f',
            Keycode::G => 'g',
            Keycode::H => 'h',
            Keycode::I => 'i',
            Keycode::J => 'j',
            Keycode::K => 'k',
            Keycode::L => 'l',
            Keycode::M => 'm',
            Keycode::N => 'n',
            Keycode::O => 'o',
            Keycode::P => 'p',
            Keycode::Q => 'q',
            Keycode::R => 'r',
            Keycode::S => 's',
            Keycode::T => 't',
            Keycode::U => 'u',
            Keycode::V => 'v',
            Keycode::W => 'w',
            Keycode::X => 'x',
            Keycode::Y => 'y',
            Keycode::Z => 'z',
            Keycode::Period => '.',
            Keycode::Slash => '/',
            Keycode::Backspace => {
                self.text.pop();
                return RepaintState::RepaintRequired;
            },
            Keycode::KpEnter => {
                return RepaintState::NoRepaintRequired;
            },
            _ => return RepaintState::NoRepaintRequired,
        };

        if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
            self.text.push(c.to_ascii_uppercase());
        } else {
            self.text.push(c);
        }
        RepaintState::RepaintRequired
    }
}
