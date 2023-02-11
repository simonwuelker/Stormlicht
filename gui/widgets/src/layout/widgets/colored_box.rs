use crate::{
    layout::{Sizing, Widget},
    GuiError,
};

use anyhow::Result;
use sdl2::{
    event::Event, mouse::MouseButton, pixels::Color, rect::Rect, render::Canvas, video::Window,
};
use std::marker::PhantomData;

pub struct ColoredBox<M> {
    color: Color,
    bounding_box: Option<Rect>,
    width: Sizing,
    height: Sizing,
    phantom: PhantomData<M>,
}

impl<M> ColoredBox<M> {
    pub fn new(color: Color) -> Self {
        Self {
            color: color,
            bounding_box: None,
            width: Sizing::default(),
            height: Sizing::default(),
            phantom: PhantomData,
        }
    }
}

impl<M> Widget for ColoredBox<M> {
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
        surface.set_draw_color(self.color);
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
        if let Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            ..
        } = event
        {
            println!("Clicked {:?}", self.color);
        }
    }
}
