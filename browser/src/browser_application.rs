use widgets::{
    application::Application,
    colorscheme,
    layout::{
        widgets::{Button, ColoredBox, Input},
        Container, Orientation, Sizing, Widget,
    },
};

#[derive(Clone, Copy, Default, Debug)]
pub struct BrowserApplication;

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Click,
}

impl Application for BrowserApplication {
    type Message = Message;

    fn view(&self) -> Box<dyn Widget<Message = Self::Message>> {
        let mut textbox = Input::new(colorscheme::BACKGROUND_DARK).into_widget();
        textbox.set_size(Sizing::Exactly(50));

        let webcontent = Button::new(ColoredBox::new(colorscheme::BACKGROUND_LIGHT).into_widget())
            .on_click(Message::Click)
            .into_widget();

        let root = Container::new(Orientation::Vertical)
            .add_child(textbox)
            .add_child(webcontent);
        Box::new(root) as Box<dyn Widget<Message = Self::Message>>
    }

    fn on_message(
        &mut self,
        window: &mut widgets::sdl2::video::Window,
        message: Self::Message,
        message_queue: widgets::application::AppendOnlyQueue<Self::Message>,
    ) {
        dbg!(message);
        _ = window;
        _ = message_queue;
    }
}
