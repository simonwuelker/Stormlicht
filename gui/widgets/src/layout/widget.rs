use anyhow::Result;
use sdl2::{event::Event, rect::Rect, render::Canvas, video::Window};

use crate::layout::Sizing;

pub trait Widget {
    fn bounding_box(&self) -> Option<Rect>;

    fn render(&mut self, surface: &mut Canvas<Window>) -> Result<()> {
        let viewport = surface.viewport();
        self.render_to(surface, viewport)?;
        Ok(())
    }

    /// Set the preferred size of the element to the given size.
    fn set_size(&mut self, _sizing: Sizing) {}

    fn preferred_sizing(&self) -> Sizing;

    fn render_to(&mut self, surface: &mut Canvas<Window>, into: Rect) -> Result<()>;

    fn compute_layout(&mut self, _into: Rect);

    fn invalidate_layout(&mut self);

    fn swallow_event(&mut self, _event: Event) {}

    fn into_widget(self) -> Box<dyn Widget>
    where
        Self: Sized + 'static,
    {
        Box::new(self) as Box<dyn Widget>
    }
}
