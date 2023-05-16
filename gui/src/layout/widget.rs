use sdl2::{
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
    rect::Rect,
    render::Canvas,
    video::Window,
};

use crate::{
    application::{AppendOnlyQueue, RepaintRequired},
    layout::Sizing,
    GuiError,
};

pub trait Widget {
    /// The message type used in the [Application](crate::application::Application).
    /// Some widgets may emit messages, like a button being pressed or a textinput
    /// changing state.
    type Message;

    fn bounding_box(&self) -> Option<Rect>;

    fn render(&mut self, surface: &mut Canvas<Window>) -> Result<(), GuiError> {
        let viewport = surface.viewport();
        self.render_to(surface, viewport)?;
        Ok(())
    }

    fn width(&self) -> Sizing;
    fn height(&self) -> Sizing;

    fn render_to(&mut self, surface: &mut Canvas<Window>, into: Rect) -> Result<(), GuiError>;

    fn compute_layout(&mut self, into: Rect);

    fn invalidate_layout(&mut self);

    /// Handle a MouseDown event
    fn on_mouse_down(
        &mut self,
        mouse_btn: MouseButton,
        x: i32,
        y: i32,
        message_queue: AppendOnlyQueue<Self::Message>,
    ) -> RepaintRequired {
        _ = mouse_btn;
        _ = x;
        _ = y;
        _ = message_queue;
        RepaintRequired::No
    }

    fn on_key_down(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
        message_queue: AppendOnlyQueue<Self::Message>,
    ) -> RepaintRequired {
        _ = keycode;
        _ = keymod;
        _ = message_queue;
        RepaintRequired::No
    }

    fn into_widget(self) -> Box<dyn Widget<Message = Self::Message>>
    where
        Self: Sized + 'static,
    {
        Box::new(self) as Box<dyn Widget<Message = Self::Message>>
    }
}
