use crate::{events::Event, primitives::Rect, surface::Surface, layout::Sizing};

pub trait Widget {
    fn bounding_box(&self) -> Option<Rect>;

    fn render(&mut self, surface: &mut Box<dyn Surface>) {
        let viewport = surface.viewport();
        self.render_to(surface, viewport);
    }

    /// Set the preferred size of the element to the given size.
    fn set_size(&mut self, _sizing: Sizing) {}

    fn preferred_sizing(&self) -> Sizing;

    fn render_to(&mut self, surface: &mut Box<dyn Surface>, into: Rect);

    fn compute_layout(&mut self, _into: Rect);

    fn invalidate_layout(&mut self);

    fn swallow_event(&mut self, _event: Event) {}

    fn as_widget(self) -> Box<dyn Widget>
    where
        Self: Sized + 'static,
    {
        Box::new(self) as Box<dyn Widget>
    }
}
