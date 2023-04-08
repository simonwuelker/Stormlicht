use crate::{
    application::{AppendOnlyQueue, RepaintState},
    layout::{Sizing, Widget},
    GuiError,
};

use sdl2::{mouse::MouseButton, rect::Rect, render::Canvas, video::Window};

pub struct Button<M> {
    inner: Box<dyn Widget<Message = M>>,
    bounding_box: Option<Rect>,
    width: Sizing,
    height: Sizing,
    on_click: Option<M>,
}

impl<M> Button<M> {
    pub fn new(inner: Box<dyn Widget<Message = M>>) -> Self {
        Self {
            inner: inner,
            bounding_box: None,
            width: Sizing::default(),
            height: Sizing::default(),
            on_click: None,
        }
    }

    /// Define a message that should be emitted once the button is clicked
    pub fn on_click(mut self, message: M) -> Self {
        self.on_click = Some(message);
        self
    }

    pub fn set_width(mut self, width: Sizing) -> Self {
        self.width = width;
        self
    }

    pub fn set_height(mut self, height: Sizing) -> Self {
        self.height = height;
        self
    }
}

impl<M: Copy> Widget for Button<M> {
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

    fn render_to(&mut self, surface: &mut Canvas<Window>, into: Rect) -> Result<(), GuiError> {
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
    ) -> RepaintState {
        if let Some(message) = self.on_click {
            if mouse_btn == MouseButton::Left {
                message_queue.append(message)
            }
        }
        RepaintState::NoRepaintRequired
    }
}
