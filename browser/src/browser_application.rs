use widgets::{
    application::{Application, RepaintState},
    colorscheme,
    layout::{
        widgets::{Button, ColoredBox, Input},
        Container, Orientation, Sizing, Widget,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct BrowserApplication {
    should_run: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Close,
}

impl Default for BrowserApplication {
    fn default() -> Self {
        Self { should_run: true }
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
    ) -> RepaintState {
        _ = window;
        _ = message_queue;
        dbg!(message);
        match message {
            Message::Close => self.should_run = false,
        }
        RepaintState::NoRepaintRequired
    }
}
