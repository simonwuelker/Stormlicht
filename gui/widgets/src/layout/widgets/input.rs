use crate::{
    layout::{Sizing, Widget},
    GuiError,
};
use anyhow::Result;
use sdl2::{event::Event, pixels::Color, rect::Rect, render::Canvas, video::Window};

pub struct Input<M> {
    bounding_box: Option<Rect>,
    on_input: Option<M>,
    width: Sizing,
    height: Sizing,
}

impl<M> Default for Input<M> {
    fn default() -> Self {
        Self {
            bounding_box: None,
            on_input: None,
            width: Sizing::default(),
            height: Sizing::Exactly(20),
        }
    }
}

impl<M> Input<M> {
    /// Define a message that should be emitted once the
    pub fn on_input(&mut self, message: M) {
        self.on_input = Some(message);
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

    fn render_to(&mut self, surface: &mut Canvas<Window>, into: Rect) -> Result<()> {
        surface.set_draw_color(Color::BLACK);
        surface.fill_rect(into).map_err(GuiError::from_sdl)?;
        Ok(())
    }

    fn invalidate_layout(&mut self) {
        self.bounding_box = None;
    }

    fn compute_layout(&mut self, into: Rect) {
        self.bounding_box = Some(into);
    }

    fn swallow_event(&mut self, event: Event) {
        if let Event::KeyDown { keycode, .. } = event {
            println!("{keycode:?}");
        }
    }
}
