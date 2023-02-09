use crate::{
    application::AppendOnlyQueue,
    layout::{Sizing, Widget},
};

use anyhow::Result;
use sdl2::{mouse::MouseButton, rect::Rect, render::Canvas, video::Window};

pub struct Button<M> {
    inner: Box<dyn Widget<Message = M>>,
    bounding_box: Option<Rect>,
    sizing: Sizing,
    on_click: Option<M>,
}

impl<M> Button<M> {
    pub fn new(inner: Box<dyn Widget<Message = M>>) -> Self {
        Self {
            inner: inner,
            bounding_box: None,
            sizing: Sizing::Grow(1.),
            on_click: None,
        }
    }

    /// Define a message that should be emitted once the button is clicked
    pub fn on_click(mut self, message: M) -> Self {
        self.on_click = Some(message);
        self
    }
}

impl<M: Copy> Widget for Button<M> {
    type Message = M;

    fn bounding_box(&self) -> Option<Rect> {
        self.bounding_box
    }

    fn set_size(&mut self, sizing: Sizing) {
        self.sizing = sizing;
    }

    fn preferred_sizing(&self) -> Sizing {
        self.sizing
    }

    fn render_to(&mut self, surface: &mut Canvas<Window>, into: Rect) -> Result<()> {
        self.inner.render_to(surface, into)
    }

    fn invalidate_layout(&mut self) {
        self.bounding_box = None;
    }

    fn compute_layout(&mut self, into: Rect) {
        self.bounding_box = Some(into);
    }

    fn on_mouse_down(
        &mut self,
        mouse_btn: MouseButton,
        _x: i32,
        _y: i32,
        mut message_queue: AppendOnlyQueue<Self::Message>,
    ) {
        if let Some(message) = self.on_click {
            if mouse_btn == MouseButton::Left {
                message_queue.append(message)
            }
        }
    }
}
