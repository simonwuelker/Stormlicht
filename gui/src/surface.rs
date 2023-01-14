//! Abstraction over an SDL2 Window

use sdl2::render::{Canvas, RenderTarget};

use crate::{
    color::Color,
    primitives::{Rect},
};

pub trait Surface {
    fn viewport(&self) -> Rect;
    fn draw_rect(&mut self, area: Rect, color: Color);
    fn update(&mut self);
    fn fill(&mut self, color: Color) {
        self.draw_rect(self.viewport(), color);
    }
}

impl<T: RenderTarget> Surface for Canvas<T> {
    fn viewport(&self) -> Rect {
        self.viewport().into()
    }

    fn draw_rect(&mut self, area: Rect, color: Color) {
        self.set_draw_color(color);
        self.fill_rect(Some(area.into())).unwrap();
    }

    fn update(&mut self) {
        self.present();
    }
}