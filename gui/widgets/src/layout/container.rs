use super::{Orientation, Widget};
use crate::layout::Sizing;
use crate::Alignment;
use anyhow::Result;
use sdl2::{
    event::Event,
    mouse::MouseButton,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

pub struct Container<M> {
    orientation: Orientation,
    children: Vec<Box<dyn Widget<Message = M>>>,
    focused_child: Option<usize>,
    cached_layout: Option<Rect>,
    sizing: Sizing,
    alignment: Alignment,
}

fn compute_item_sizes(
    available_space: u32,
    items: Vec<Sizing>,
    alignment: Alignment,
) -> Vec<(u32, u32)> {
    let sum_exactly_sized_elements: u32 = items
        .iter()
        .filter_map(|sizing| match sizing {
            Sizing::Exactly(n) => Some(n),
            _ => None,
        })
        .sum();

    let sum_grow_elements: f32 = items
        .iter()
        .filter_map(|sizing| match sizing {
            Sizing::Grow(factor) => Some(factor),
            _ => None,
        })
        .sum();

    let mut element_sizes = Vec::with_capacity(items.len());

    let available_space_to_grow = available_space - sum_exactly_sized_elements;

    if sum_grow_elements == 0.0 {
        // All items have a fixed-size, Alignment decides positioning
        // Note that theoretically, an item can have a grow factor of 0.0
        // In this case, it is not rendered.
        match alignment {
            Alignment::Start => {
                let mut x = 0;
                for sizing in items {
                    if let Sizing::Exactly(n) = sizing {
                        element_sizes.push((x, n));
                        x += n;
                    }
                }
            },
            Alignment::End => {
                let mut x = available_space_to_grow;
                for sizing in items {
                    if let Sizing::Exactly(n) = sizing {
                        element_sizes.push((x, n));
                        x += n;
                    }
                }
            },
            Alignment::Center => {
                let mut x = available_space_to_grow / 2;
                for sizing in items {
                    if let Sizing::Exactly(n) = sizing {
                        element_sizes.push((x, n));
                        x += n;
                    }
                }
            },
            Alignment::Fill => {
                let space_between_items = available_space_to_grow / (items.len() as u32 + 1);
                let mut x = space_between_items;
                for sizing in items {
                    if let Sizing::Exactly(n) = sizing {
                        element_sizes.push((x, n));
                        x += space_between_items + n;
                    }
                }
            },
        }
    } else {
        // Alignment becomes irrelevant, items grow to take up available space
        let mut x = 0;
        for sizing in items {
            let size = match sizing {
                Sizing::Exactly(n) => n,
                Sizing::Grow(f) => {
                    ((f / sum_grow_elements) * available_space_to_grow as f32).round() as u32
                },
            };
            element_sizes.push((x, size));
            x += size;
        }
    }
    element_sizes
}

impl<M> Container<M> {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            children: vec![],
            focused_child: None,
            cached_layout: None,
            sizing: Sizing::Grow(1.),
            alignment: Alignment::default(),
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn add_child(mut self, child: Box<dyn Widget<Message = M>>) -> Self {
        self.children.push(child);
        self
    }

    pub fn widget_containing(&self, point: Point) -> Option<usize> {
        for (index, child) in self.children.iter().enumerate() {
            let bb = child
                .bounding_box()
                .expect("Widgets that do not have a layout cannot swallow input");

            if bb.contains_point(point) {
                return Some(index);
            }
        }

        None
    }
}

impl<M> Widget for Container<M> {
    type Message = M;

    fn bounding_box(&self) -> Option<Rect> {
        self.cached_layout
    }

    fn set_size(&mut self, sizing: Sizing) {
        self.sizing = sizing;
    }

    fn preferred_sizing(&self) -> Sizing {
        self.sizing
    }

    fn render_to(&mut self, surface: &mut Canvas<Window>, into: Rect) -> Result<()> {
        if self.cached_layout.is_none() {
            self.compute_layout(into);
        }

        assert!(self
            .children
            .iter()
            .all(|child| child.bounding_box().is_some()));

        for child in &mut self.children {
            child.render_to(surface, child.bounding_box().unwrap())?;
        }
        Ok(())
    }

    fn compute_layout(&mut self, into: Rect) {
        let available_space = match self.orientation {
            Orientation::Horizontal => into.width(),
            Orientation::Vertical => into.height(),
        };
        let positions = compute_item_sizes(
            available_space,
            self.children.iter().map(|c| c.preferred_sizing()).collect(),
            self.alignment,
        );

        match self.orientation {
            Orientation::Horizontal => {
                for (child, (offset, size)) in self.children.iter_mut().zip(positions) {
                    let mut child_box = into;
                    child_box.set_width(size);
                    child_box.set_x(offset as i32);
                    child.compute_layout(child_box);
                }
            },
            Orientation::Vertical => {
                for (child, (offset, size)) in self.children.iter_mut().zip(positions) {
                    let mut child_box = into;
                    child_box.set_height(size);
                    child_box.set_y(offset as i32);
                    child.compute_layout(child_box);
                }
            },
        }

        self.cached_layout = Some(into);
    }

    fn invalidate_layout(&mut self) {
        self.cached_layout = None;

        for child in &mut self.children {
            child.invalidate_layout();
        }
    }

    fn on_mouse_down(
        &mut self,
        mouse_btn: MouseButton,
        x: i32,
        y: i32,
        message_queue: crate::application::AppendOnlyQueue<Self::Message>,
    ) {
        // Forward the event to the child that contains the given location
        if let Some(child_index) = self.widget_containing(Point::new(x, y)) {
            self.children[child_index].on_mouse_down(mouse_btn, x, y, message_queue);
        }
    }

    fn swallow_event(&mut self, event: Event) {
        if let Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } = event
        {
            self.focused_child = self.widget_containing(Point::new(x, y))
        }

        // If the event has a location
        if let Event::MouseButtonUp { x, y, .. }
        | Event::MouseButtonDown { x, y, .. }
        | Event::MouseMotion { x, y, .. }
        | Event::MouseWheel { x, y, .. } = event
        {
            // Forward the event to the child that contains the given location
            let containing_child = self.widget_containing(Point::new(x, y));

            if let Some(child_index) = containing_child {
                self.children[child_index].swallow_event(event);
            }
        } else {
            // else, send it to the focused widget, if any
            if let Some(focused_child) = self.focused_child {
                self.children[focused_child].swallow_event(event);
            }
        }
    }
}
