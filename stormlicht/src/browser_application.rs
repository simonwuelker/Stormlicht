use core::BrowsingContext;

const INITIAL_WIDTH: u16 = 800;
const INITIAL_HEIGHT: u16 = 600;

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
    _browsing_context: BrowsingContext,
}

impl glazier::WinHandler for BrowserApplication {
    fn connect(&mut self, handle: &glazier::WindowHandle) {
        let graphics_context = unsafe { softbuffer::GraphicsContext::new(handle, handle) }.unwrap();
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
    pub fn new(url: Option<&str>) -> Self {
        let font = font::Font::default();
        let d = font.compute_rendered_width("Font test", 200.);
        let mut composition = render::Composition::default();

        composition
            .get_or_insert_layer(1)
            .with_source(render::Source::Solid(math::Color::BLUE))
            .with_outline(render::Path::rect(
                math::Vec2D::new(50., 50.),
                math::Vec2D::new(50. + d, 250.),
            ));

        composition
            .get_or_insert_layer(2)
            .with_source(render::Source::Solid(math::Color::BLACK))
            .text(
                "Font test",
                font::Font::default(),
                200.,
                math::Vec2D::new(50., 50.),
            );

        let browsing_context = match url {
            Some(url) => BrowsingContext::load(url).unwrap(),
            None => {
                // FIXME: default url
                BrowsingContext
            },
        };

        Self {
            view_buffer: math::Bitmap::new(INITIAL_WIDTH as usize, INITIAL_HEIGHT as usize),
            graphics_context: None,
            size: (INITIAL_WIDTH, INITIAL_HEIGHT),
            repaint_required: RepaintRequired::Yes,
            composition,
            window_handle: glazier::WindowHandle::default(),
            _browsing_context: browsing_context,
        }
    }

    pub fn run(self) {
        let app = glazier::Application::new().unwrap();
        let window = glazier::WindowBuilder::new(app.clone())
            .resizable(true)
            .size(((INITIAL_WIDTH / 2) as f64, (INITIAL_HEIGHT / 2) as f64).into())
            .handler(Box::new(self))
            .title("Browser")
            .build()
            .unwrap();
        window.show();
        app.run(None);
    }
}
