use crate:: {
    color::Color,
    layout::Widget,
    primitives::Rect,
    events::{Event, MouseButton},
};

pub struct ColoredBox {
    color: Color,
    bounding_box: Option<Rect>,
}

impl ColoredBox {
    pub fn new(color: Color) -> Self {
        Self {
            color: color,
            bounding_box: None,
        }
    }
}

impl Widget for ColoredBox {
    fn bounding_box(&self) -> Option<Rect> {
        self.bounding_box
    }

    fn render_to(&mut self, surface: &mut Box<dyn crate::surface::Surface>, into: crate::primitives::Rect) {
        surface.draw_rect(into, self.color)
    }

    fn invalidate_layout(&mut self) {
        self.bounding_box = None;
    }

    fn compute_layout(&mut self, into: Rect) {
        self.bounding_box = Some(into);
    }

    fn swallow_event(&mut self, event: Event) {
        if let Event::MouseDown { button: MouseButton::Left, .. } = event {
            println!("Clicked {:?}", self.color);
        }
    }
}