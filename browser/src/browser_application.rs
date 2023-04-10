use widgets::{
    application::{Application, RepaintRequired},
    colorscheme,
    layout::{
        widgets::{Button, ColoredBox, Input},
        Container, Orientation, Sizing, Widget,
    },
};

const INITIAL_WIDTH: u16 = 800;
const INITIAL_HEIGHT: u16 = 600;

pub struct BrowserApplication {
    should_run: bool,
    view_buffer: Vec<u32>,
    graphics_context: Option<softbuffer::GraphicsContext>,
    size: (u16, u16),
    repaint_required: RepaintRequired,
    window_handle: glazier::WindowHandle,
}

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Close,
}

impl Default for BrowserApplication {
    fn default() -> Self {
        Self {
            should_run: true,
            view_buffer: vec![],
            graphics_context: None,
            size: (INITIAL_WIDTH, INITIAL_HEIGHT),
            repaint_required: RepaintRequired::Yes,
            window_handle: glazier::WindowHandle::default(),
        }
    }
}

impl Application for BrowserApplication {
    type Message = Message;

    fn view(&self) -> Box<dyn Widget<Message = Self::Message>> {
        let search_bar = Input::default().into_widget();

        let close_btn = Button::new(ColoredBox::new(colorscheme::ALTERNATIVE).into_widget())
            .on_click(Message::Close)
            .set_width(Sizing::Exactly(50))
            .set_height(Sizing::Exactly(50))
            .into_widget();

        let navbar = Container::new(Orientation::Horizontal)
            .add_child(search_bar)
            .add_child(close_btn)
            .into_widget();

        let webcontent = ColoredBox::new(colorscheme::BACKGROUND_LIGHT).into_widget();

        let root = Container::new(Orientation::Vertical)
            .add_child(navbar)
            .add_child(webcontent);
        Box::new(root) as Box<dyn Widget<Message = Self::Message>>
    }

    fn should_run(&self) -> bool {
        self.should_run
    }

    fn on_message(
        &mut self,
        window: &mut widgets::sdl2::video::Window,
        message: Self::Message,
        message_queue: widgets::application::AppendOnlyQueue<Self::Message>,
    ) -> RepaintRequired {
        _ = window;
        _ = message_queue;
        dbg!(message);
        match message {
            Message::Close => self.should_run = false,
        }
        RepaintRequired::No
    }
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
        self.view_buffer = (0..(self.size.0 as usize * self.size.1 as usize))
            .map(|index| {
                let y = index / (self.size.0 as usize);
                let x = index % (self.size.0 as usize);
                let red = x % 255;
                let green = y % 255;
                let blue = (x * y) % 255;

                let color = blue | (green << 8) | (red << 16);

                color as u32
            })
            .collect::<Vec<_>>();

        if let Some(graphics_context) = &mut self.graphics_context {
            graphics_context.set_buffer(&self.view_buffer, self.size.0, self.size.1);
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn size(&mut self, size: glazier::kurbo::Size) {
        dbg!(size);
        self.size = (size.width.ceil() as u16 * 2, size.height.ceil() as u16 * 2);
        self.repaint_required = RepaintRequired::Yes;
    }

    fn request_close(&mut self) {
        self.window_handle.close();
        glazier::Application::global().quit();
    }
}

impl BrowserApplication {
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
