use super::{Orientation, Widget};
use crate::{
    layout::Sizing,
    events::{Event, MouseButton},
    primitives::{Point, Rect},
};

pub struct Divider {
    orientation: Orientation,
    children: Vec<Box<dyn Widget>>,
    focused_child: Option<usize>,
    cached_layout: Option<Rect>,
    sizing: Sizing,
}

impl Divider {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            children: vec![],
            focused_child: None,
            cached_layout: None,
            sizing: Sizing::Grow(1.)
        }
    }

    pub fn add_child(mut self, child: Box<dyn Widget>) -> Self {
        self.children.push(child);
        self
    }

    pub fn widget_containing(&self, point: Point) -> Option<usize> {
        for (index, child) in self.children.iter().enumerate() {
            let bb = child
                .bounding_box()
                .expect("Widgets that do not have a layout cannot swallow input");

            if bb.contains(point) {
                return Some(index);
            }
        }

        None
    }
}

impl Widget for Divider {
    fn bounding_box(&self) -> Option<Rect> {
        self.cached_layout
    }

    fn set_size(&mut self, sizing: Sizing) {
        self.sizing = sizing;
    }

    fn preferred_sizing(&self) -> Sizing {
        self.sizing
    }

    fn render_to(&mut self, surface: &mut Box<dyn crate::surface::Surface>, into: Rect) {
        if self.cached_layout.is_none() {
            self.compute_layout(into);
        }

        assert!(self.children.iter().all(|child| child.bounding_box().is_some()));

        for child in &mut self.children {
            child.render_to(surface, child.bounding_box().unwrap());
        }
    }

    fn compute_layout(&mut self, into: Rect) {
        let sum_exactly_sized_elements: u32 = self.children.iter().filter_map(|child| match child.preferred_sizing() {
            Sizing::Exactly(n) => Some(n),
            _ => None,
        }).sum();

        let sum_grow_elements: f32 = self.children.iter().filter_map(|child| match child.preferred_sizing() {
            Sizing::Grow(factor) => Some(factor),
            _ => None,
        }).sum();


        let mut element_sizes = Vec::with_capacity(self.children.len());

        let available_space_to_grow = match self.orientation {
            Orientation::Horizontal => into.width() - sum_exactly_sized_elements,
            Orientation::Vertical => into.height() - sum_exactly_sized_elements,
        } as f32;

        for child in &self.children {
            let assigned_size = match child.preferred_sizing() {
                Sizing::Exactly(n) => n,
                Sizing::Grow(factor) => ((factor / sum_grow_elements) * available_space_to_grow) as u32
            };

            element_sizes.push(assigned_size);
        }

        match self.orientation {
            Orientation::Horizontal => {
                let mut total_width = into.x() as u32;
                for (child, size) in self.children.iter_mut().zip(element_sizes) {
                    let rect = into.with_width(size).with_x(total_width as i32);
                    total_width += size;
                    child.compute_layout(rect);
                }
            },
            Orientation::Vertical => {
                let mut total_height= into.y() as u32;
                for (child, size) in self.children.iter_mut().zip(element_sizes) {
                    let rect = into.with_height(size).with_y(total_height as i32);
                    total_height += size;
                    child.compute_layout(rect);
                }
            }
        }

        self.cached_layout = Some(into);
    }

    fn invalidate_layout(&mut self) {
        self.cached_layout = None;

        for child in &mut self.children {
            child.invalidate_layout();
        }
    }

    fn swallow_event(&mut self, event: Event) {
        if let Event::MouseDown {
            button: MouseButton::Left,
            at,
        } = event
        {
            self.focused_child = self.widget_containing(at)
        }

        match (event.location(), self.focused_child) {
            (Some(location), _) => {
                // Forward the event to the child that contains the given location
                let containing_child = self.widget_containing(location);

                if let Some(child_index) = containing_child {
                    self.children[child_index].swallow_event(event);
                }

            },
            (_, Some(focused_child)) => {
                // Forward to the focused child
                self.children[focused_child].swallow_event(event);
            },
            _ => {
                // Do nothing
            },
        }
    }
}
