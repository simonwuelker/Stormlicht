use crate::{color::Color, events::Event, layout::{Widget, Sizing}, primitives::Rect};

pub struct Input {
    color: Color,
    bounding_box: Option<Rect>,
    sizing: Sizing,
}

impl Input {
    pub fn new(color: Color) -> Self {
        Self {
            color: color,
            bounding_box: None,
            sizing: Sizing::Grow(1.),
        }
    }
}

impl Widget for Input {
    fn bounding_box(&self) -> Option<Rect> {
        self.bounding_box
    }

    fn set_size(&mut self, sizing: Sizing) {
        self.sizing = sizing;
    }

    fn preferred_sizing(&self) -> Sizing {
        self.sizing
    }

    fn render_to(
        &mut self,
        surface: &mut Box<dyn crate::surface::Surface>,
        into: crate::primitives::Rect,
    ) {
        surface.draw_rect(into, self.color)
    }

    fn invalidate_layout(&mut self) {
        self.bounding_box = None;
    }

    fn compute_layout(&mut self, into: Rect) {
        self.bounding_box = Some(into);
    }

    fn swallow_event(&mut self, event: Event) {
        if let Event::KeyDown { keycode, .. } = event {
            println!("{:?}", keycode);
        }
    }
}
