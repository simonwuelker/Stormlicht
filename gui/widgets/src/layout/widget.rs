use anyhow::Result;
use sdl2::{event::Event, mouse::MouseButton, rect::Rect, render::Canvas, video::Window};

use crate::{application::AppendOnlyQueue, layout::Sizing};

pub trait Widget {
    /// The message type used in the [Application](crate::application::Application).
    /// Some widgets may emit messages, like a button being pressed or a textinput
    /// changing state.
    type Message;

    fn bounding_box(&self) -> Option<Rect>;

    fn render(&mut self, surface: &mut Canvas<Window>) -> Result<()> {
        let viewport = surface.viewport();
        self.render_to(surface, viewport)?;
        Ok(())
    }

    /// Set the preferred size of the element to the given size.
    fn set_size(&mut self, sizing: Sizing) {
        _ = sizing;
    }

    fn preferred_sizing(&self) -> Sizing;

    fn render_to(&mut self, surface: &mut Canvas<Window>, into: Rect) -> Result<()>;

    fn compute_layout(&mut self, into: Rect);

    fn invalidate_layout(&mut self);

    /// Handle a MouseDown event
    fn on_mouse_down(
        &mut self,
        mouse_btn: MouseButton,
        x: i32,
        y: i32,
        message_queue: AppendOnlyQueue<Self::Message>,
    ) {
        _ = mouse_btn;
        _ = x;
        _ = y;
        _ = message_queue;
    }

    fn swallow_event(&mut self, _event: Event) {}

    fn into_widget(self) -> Box<dyn Widget<Message = Self::Message>>
    where
        Self: Sized + 'static,
    {
        Box::new(self) as Box<dyn Widget<Message = Self::Message>>
    }
}
