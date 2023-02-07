use crate::GuiError;
use anyhow::Result;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    render::Canvas,
    video::Window,
};

use std::collections::VecDeque;

pub trait Application {
    /// A type that can be used to send user-defined messages
    /// throughout the application.
    type Message;

    /// Draw the widgets content on the given canvas
    ///
    /// Generally speaking, a paint event should not cause side effects.
    /// The reason that `self` is mutable here is that some applications may want
    /// to cache layout calculations which may or may not have to be refreshed on paint.
    fn on_paint(&mut self, canvas: &mut Canvas<Window>);

    /// Handle a resize event
    fn on_resize(&mut self, width: i32, height: i32) {
        _ = width;
        _ = height;
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
        message_queue: &mut AppendOnlyQueue<Self::Message>,
    ) {
        _ = window;
        _ = message;
        _ = message_queue;
    }

    // TODO this should be generic to support non-browser applications.
    // Currently, window dimensions, title and other attributes are hardcoded
    fn run(&mut self) -> Result<()> {
        // Create the application window
        let sdl_context = sdl2::init().map_err(GuiError::from_sdl)?;
        let video_subsystem = sdl_context.video().map_err(GuiError::from_sdl)?;

        let window = video_subsystem
            .window("Browser", 800, 600)
            .position_centered()
            .borderless()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        // Trigger one initial paint event
        self.on_paint(&mut canvas);

        // Game loop, handle events
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut message_queue = VecDeque::new();
        'running: loop {
            // Handle window events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        self.on_resize(width, height);
                        self.on_paint(&mut canvas);
                    },
                    _ => {},
                }
            }

            // Handle application-internal messages
            while let Some(message) = message_queue.pop_front() {
                let mut queue = message_queue.into();
                self.on_message(canvas.window_mut(), message, &mut queue);
                message_queue = queue.into();
            }
            std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
        }
        Ok(())
    }
}

/// An abstraction over a [VecDeque] that only allows append operations, without
/// looking at the queues content.
/// This is a type-safe to avoid confusion in [Application::on_message].
pub struct AppendOnlyQueue<T> {
    queue: VecDeque<T>,
}

impl<T> From<VecDeque<T>> for AppendOnlyQueue<T> {
    fn from(queue: VecDeque<T>) -> Self {
        Self { queue }
    }
}

impl<T> From<AppendOnlyQueue<T>> for VecDeque<T> {
    fn from(value: AppendOnlyQueue<T>) -> Self {
        value.queue
    }
}

impl<T> AppendOnlyQueue<T> {
    pub fn append(&mut self, element: T) {
        self.queue.push_back(element)
    }
}
