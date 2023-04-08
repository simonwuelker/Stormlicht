use crate::{layout::Widget, GuiError};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::Window,
};

use std::{collections::VecDeque, ops::BitOrAssign};

pub trait Application {
    /// A type that can be used to send user-defined messages
    /// throughout the application.
    type Message;

    /// Draw the widgets content on the given canvas
    ///
    /// Generally speaking, a paint event should not cause side effects.
    /// The reason that `self` is mutable here is that some applications may want
    /// to cache layout calculations which may or may not have to be refreshed on paint.
    fn view(&self) -> Box<dyn Widget<Message = Self::Message>>;

    /// Handle a resize event
    fn on_resize(&mut self, width: i32, height: i32) -> RepaintState {
        _ = width;
        _ = height;
        RepaintState::NoRepaintRequired
    }

    /// Handle a message from another part of the application
    ///
    /// While handling the message, any number of additional messages
    /// may be produced. These messages should be appended to the
    /// `message_queue` parameter.
    fn on_message(
        &mut self,
        window: &mut Window,
        message: Self::Message,
        message_queue: AppendOnlyQueue<Self::Message>,
    ) -> RepaintState {
        _ = window;
        _ = message;
        _ = message_queue;
        RepaintState::default()
    }

    fn should_run(&self) -> bool;

    // TODO this should be generic to support non-browser applications.
    // Currently, window dimensions, title and other attributes are hardcoded
    fn run(&mut self) -> Result<(), GuiError> {
        // Create the application window
        let sdl_context = sdl2::init().map_err(GuiError::SDL)?;
        let video_subsystem = sdl_context.video().map_err(GuiError::SDL)?;

        let window = video_subsystem
            .window("Browser", 800, 600)
            .position_centered()
            .borderless()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        // Trigger one initial paint event
        let mut view_tree = self.view();
        view_tree.render(&mut canvas).unwrap();
        canvas.present();

        // Game loop, handle events
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut message_queue = VecDeque::new();
        'running: while self.should_run() {
            // While processing the state and new events, keep track of whether we need to redraw the application
            let mut repaint_required = RepaintState::default();

            // Handle window events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::MouseButtonDown {
                        mouse_btn, x, y, ..
                    } => {
                        repaint_required |= view_tree.on_mouse_down(
                            mouse_btn,
                            x,
                            y,
                            AppendOnlyQueue::new(&mut message_queue),
                        )
                    },
                    // I don't know when an event would not have a KeyCode.
                    // Let's just ignore any keyboard events without a code
                    Event::KeyDown {
                        keycode: Some(keycode),
                        keymod,
                        ..
                    } => {
                        repaint_required |= view_tree.on_key_down(
                            keycode,
                            keymod,
                            AppendOnlyQueue::new(&mut message_queue),
                        );
                    },
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        repaint_required |= self.on_resize(width, height);
                    },
                    _ => {},
                }
            }

            // Handle application-internal messages
            while let Some(message) = message_queue.pop_front() {
                repaint_required |= self.on_message(
                    canvas.window_mut(),
                    message,
                    AppendOnlyQueue::new(&mut message_queue),
                );
            }

            if repaint_required == RepaintState::RepaintRequired {
                view_tree.render(&mut canvas).unwrap();
                canvas.present()
            }
            std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
        }
        Ok(())
    }
}

/// An abstraction over a [VecDeque] that only allows append operations, without
/// looking at the queues content.
/// This is a type-safe to avoid confusion in [Application::on_message].
pub struct AppendOnlyQueue<'a, T> {
    queue: &'a mut VecDeque<T>,
}

impl<'a, T> AppendOnlyQueue<'a, T> {
    pub fn new(queue: &'a mut VecDeque<T>) -> Self {
        Self { queue }
    }

    pub fn append(&mut self, element: T) {
        self.queue.push_back(element)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RepaintState {
    #[default]
    NoRepaintRequired,
    RepaintRequired,
}

impl std::ops::BitOr for RepaintState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::NoRepaintRequired, Self::NoRepaintRequired) => Self::NoRepaintRequired,
            _ => Self::RepaintRequired,
        }
    }
}

impl BitOrAssign for RepaintState {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
