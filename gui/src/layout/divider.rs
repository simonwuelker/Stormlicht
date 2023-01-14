use super::{Orientation, Widget};
use crate::{
    events::{Event, MouseButton},
    primitives::{Point, Rect},
};

pub struct Divider {
    orientation: Orientation,
    ratio: f32,
    items: (Option<Box<dyn Widget>>, Option<Box<dyn Widget>>),
    focused_child: Option<usize>,
    cached_layout: Option<Rect>,
}

impl Divider {
    pub fn new(orientation: Orientation, ratio: f32) -> Self {
        assert!(0. < ratio && ratio < 1.);

        Self {
            orientation,
            ratio,
            items: (None, None),
            focused_child: None,
            cached_layout: None,
        }
    }

    pub fn set_first(mut self, item: Option<Box<dyn Widget>>) -> Self {
        self.items.0 = item;
        self
    }

    pub fn set_second(mut self, item: Option<Box<dyn Widget>>) -> Self {
        self.items.1 = item;
        self
    }

    pub fn widget_containing(&self, point: Point) -> Option<usize> {
        if let Some(child) = &self.items.0 {
            let bb = child
                .bounding_box()
                .expect("Widgets that do not have a layout cannot swallow input");
            if bb.contains(point) {
                return Some(0);
            }
        }

        if let Some(child) = &self.items.1 {
            let bb = child
                .bounding_box()
                .expect("Widgets that do not have a layout cannot swallow input");
            if bb.contains(point) {
                return Some(1);
            }
        }

        None
    }
}

impl Widget for Divider {
    fn bounding_box(&self) -> Option<Rect> {
        self.cached_layout
    }

    fn render_to(&mut self, surface: &mut Box<dyn crate::surface::Surface>, into: Rect) {
        if self.cached_layout.is_none() {
            self.compute_layout(into);
        }

        assert!(self.items.0.is_none() || self.items.0.as_ref().unwrap().bounding_box().is_some());
        assert!(self.items.1.is_none() || self.items.1.as_ref().unwrap().bounding_box().is_some());

        if let Some(child) = &mut self.items.0 {
            child.render_to(surface, child.bounding_box().unwrap());
        }

        if let Some(child) = &mut self.items.1 {
            child.render_to(surface, child.bounding_box().unwrap());
        }
    }

    fn compute_layout(&mut self, into: Rect) {
        let child_sizes = match self.orientation {
            Orientation::Horizontal => {
                let size = (into.width() as f32 * self.ratio) as u32;
                (
                    into.with_width(size),
                    into.with_x(into.x() + size as i32)
                        .with_width(into.width() - size),
                )
            },
            Orientation::Vertical => {
                let size = (into.height() as f32 * self.ratio) as u32;
                (
                    into.with_height(size),
                    into.with_y(into.y() + size as i32)
                        .with_height(into.height() - size),
                )
            },
        };

        if let Some(child) = &mut self.items.0 {
            child.compute_layout(child_sizes.0);
        }

        if let Some(child) = &mut self.items.1 {
            child.compute_layout(child_sizes.1);
        }

        self.cached_layout = Some(into);
    }

    fn invalidate_layout(&mut self) {
        self.cached_layout = None;

        // if stabilized, https://doc.rust-lang.org/std/option/enum.Option.html#method.inspect would be nice here
        if let Some(child) = &mut self.items.0 {
            child.invalidate_layout();
        }

        if let Some(child) = &mut self.items.1 {
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
                match self.widget_containing(location) {
                    Some(0) => self.items.0.as_mut().unwrap().swallow_event(event),
                    Some(1) => self.items.1.as_mut().unwrap().swallow_event(event),
                    _ => {},
                }
            },
            (_, Some(_focused_child)) => {
                // Forward to the focused child
                match self.focused_child {
                    Some(0) => self.items.0.as_mut().unwrap().swallow_event(event),
                    Some(1) => self.items.1.as_mut().unwrap().swallow_event(event),
                    _ => {},
                }
            },
            _ => {
                // Do nothing
            },
        }
    }
}
