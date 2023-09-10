use core::BrowsingContext;
use std::process::ExitCode;

use url::URL;

const INITIAL_WIDTH: u16 = 800;
const INITIAL_HEIGHT: u16 = 600;

const WELCOME_PAGE: &str = concat!(
    "file://localhost",
    env!("CARGO_MANIFEST_DIR"),
    "/../pages/welcome.html"
);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum RepaintRequired {
    #[default]
    Yes,
    No,
}

pub struct BrowserApplication {
    view_buffer: math::Bitmap<u32>,
    graphics_context: Option<softbuffer::GraphicsContext>,
    size: (u16, u16),
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
        self.browsing_context
            .paint(&mut self.composition, self.size);
        self.composition.render_to(&mut self.view_buffer);

        if let Some(graphics_context) = &mut self.graphics_context {
            graphics_context.set_buffer(self.view_buffer.data(), self.size.0, self.size.1);
        }
        self.repaint_required = RepaintRequired::No;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn size(&mut self, size: glazier::kurbo::Size) {
        let width = size.width.ceil() as u16 * 2;
        let height = size.height.ceil() as u16 * 2;

        self.size = (width, height);
        self.view_buffer.resize(width as usize, height as usize);
        self.repaint_required = RepaintRequired::Yes;
    }

    fn request_close(&mut self) {
        self.window_handle.close();
        glazier::Application::global().quit();
    }
}

impl BrowserApplication {
    pub fn run(url: Option<&str>) -> ExitCode {
        let url = match URL::from_user_input(url.unwrap_or(WELCOME_PAGE)) {
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

        let application = Self {
            view_buffer: math::Bitmap::new(INITIAL_WIDTH as usize, INITIAL_HEIGHT as usize),
            graphics_context: None,
            size: (INITIAL_WIDTH, INITIAL_HEIGHT),
            repaint_required: RepaintRequired::Yes,
            composition: render::Composition::default(),
            window_handle: glazier::WindowHandle::default(),
            browsing_context: browsing_context,
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
            .size(((INITIAL_WIDTH / 2) as f64, (INITIAL_HEIGHT / 2) as f64).into())
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
}
