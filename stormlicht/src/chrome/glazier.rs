use core::{event, BrowsingContext};
use url::URL;

use std::process::ExitCode;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum RepaintRequired {
    #[default]
    Yes,
    No,
}

pub struct BrowserApplication {
    view_buffer: math::Bitmap<u32>,
    graphics_context: Option<softbuffer::GraphicsContext>,

    /// Viewport size, in Display Points (not pixels)
    viewport_size: (u16, u16),

    repaint_required: RepaintRequired,
    composition: render::Composition,
    window_handle: glazier::WindowHandle,
    browsing_context: BrowsingContext,
}

impl glazier::WinHandler for BrowserApplication {
    fn connect(&mut self, handle: &glazier::WindowHandle) {
        let graphics_context = unsafe { softbuffer::GraphicsContext::new(handle, handle) }
            .expect("Failed to connect to softbuffer graphics context");
        self.window_handle = handle.clone();
        self.graphics_context = Some(graphics_context);
    }

    fn prepare_paint(&mut self) {
        if self.repaint_required == RepaintRequired::Yes {
            self.window_handle.invalidate();
        }
    }

    fn paint(&mut self, _invalid: &glazier::Region) {
        self.view_buffer.clear(math::Color::WHITE.into());
        self.composition.clear();

        let dpi = self
            .window_handle
            .get_scale()
            .expect("Could not access dpi scale");

        self.composition.set_dpi((dpi.x() as f32, dpi.y() as f32));

        self.browsing_context
            .paint(&mut self.composition, self.viewport_size);
        self.composition.render_to(&mut self.view_buffer);

        if let Some(graphics_context) = &mut self.graphics_context {
            graphics_context.set_buffer(
                self.view_buffer.data(),
                self.view_buffer.width() as u16,
                self.view_buffer.height() as u16,
            );
        }
        self.repaint_required = RepaintRequired::No;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn size(&mut self, size: glazier::kurbo::Size) {
        let dpi = self
            .window_handle
            .get_scale()
            .expect("Could not access dpi scale");

        self.viewport_size = (size.width.ceil() as u16, size.height.ceil() as u16);

        // Size is given in dp, we need to convert it to pixels to know
        // how large our view buffer should be
        let width_px = (size.width * dpi.x()).ceil() as usize;
        let height_px = (size.height * dpi.y()).ceil() as usize;

        self.view_buffer.resize(width_px, height_px);
        self.repaint_required = RepaintRequired::Yes;
    }

    fn request_close(&mut self) {
        self.window_handle.close();
        glazier::Application::global().quit();
    }

    fn pointer_down(&mut self, glazier_event: &glazier::PointerEvent) {
        let button = match glazier_event.button {
            glazier::PointerButton::Primary => event::MouseButton::Left,
            glazier::PointerButton::Auxiliary => event::MouseButton::Middle,
            glazier::PointerButton::Secondary => event::MouseButton::Right,
            _ => {
                // Some kind of button we don't support
                return;
            },
        };
        let position = math::Vec2D {
            x: glazier_event.pos.x.round() as i32,
            y: glazier_event.pos.y.round() as i32,
        };
        let mouse_event = event::MouseEvent {
            position,
            kind: event::MouseEventKind::Down(button),
        };

        self.dispatch_event(event::Event::Mouse(mouse_event));
    }

    fn pointer_move(&mut self, glazier_event: &glazier::PointerEvent) {
        let position = math::Vec2D {
            x: glazier_event.pos.x.round() as i32,
            y: glazier_event.pos.y.round() as i32,
        };
        let mouse_event = event::MouseEvent {
            position,
            kind: event::MouseEventKind::Move,
        };

        self.dispatch_event(event::Event::Mouse(mouse_event));
    }

    fn pointer_up(&mut self, glazier_event: &glazier::PointerEvent) {
        let button = match glazier_event.button {
            glazier::PointerButton::Primary => event::MouseButton::Left,
            glazier::PointerButton::Auxiliary => event::MouseButton::Middle,
            glazier::PointerButton::Secondary => event::MouseButton::Right,
            _ => {
                // Some kind of button we don't support
                return;
            },
        };
        let position = math::Vec2D {
            x: glazier_event.pos.x.round() as i32,
            y: glazier_event.pos.y.round() as i32,
        };
        let mouse_event = event::MouseEvent {
            position,
            kind: event::MouseEventKind::Up(button),
        };

        self.dispatch_event(event::Event::Mouse(mouse_event));
    }
}

impl BrowserApplication {
    /// Forwards an event to the browsing context and repaints if necessary
    pub fn dispatch_event(&mut self, event: event::Event) {
        let needs_repaint = self.browsing_context.handle_event(event);

        if needs_repaint {
            self.window_handle.invalidate();
        }
    }
}

pub fn run(url: Option<&str>) -> ExitCode {
    let url = match URL::from_user_input(url.unwrap_or(super::WELCOME_PAGE)) {
        Ok(parsed_url) => parsed_url,
        Err(error) => {
            log::error!("Failed to parse {url:?} as a URL: {error:?}");
            return ExitCode::FAILURE;
        },
    };

    let browsing_context = match BrowsingContext::load(&url) {
        Ok(context) => context,
        Err(error) => {
            log::error!("Failed to load {}: {error:?}", url.to_string());
            return ExitCode::FAILURE;
        },
    };

    // The view buffer is initialized once the window size method is called on startup.
    // Before that, we can't know the windows dpi scaling and therefore cant know how large the
    // view buffer needs to be.
    let view_buffer = math::Bitmap::new(0, 0);

    let application = BrowserApplication {
        view_buffer,
        graphics_context: None,
        viewport_size: (super::INITIAL_WIDTH, super::INITIAL_HEIGHT),
        repaint_required: RepaintRequired::Yes,
        composition: render::Composition::default(),
        window_handle: glazier::WindowHandle::default(),
        browsing_context,
    };

    let app = match glazier::Application::new() {
        Ok(app) => app,
        Err(error) => {
            log::error!("Failed to initialize application: {error:?}");
            return ExitCode::FAILURE;
        },
    };

    let window_or_error = glazier::WindowBuilder::new(app.clone())
        .resizable(true)
        .size(
            (
                (super::INITIAL_WIDTH) as f64,
                (super::INITIAL_HEIGHT) as f64,
            )
                .into(),
        )
        .handler(Box::new(application))
        .title("Stormlicht")
        .build();

    match window_or_error {
        Ok(window) => {
            window.show();
            app.run(None);
            ExitCode::SUCCESS
        },
        Err(error) => {
            log::error!("Failed to create application window: {error:?}");
            ExitCode::FAILURE
        },
    }
}
