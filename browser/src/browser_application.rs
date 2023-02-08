use widgets::{
    application::Application,
    colorscheme,
    layout::{
        widgets::{ColoredBox, Input},
        Container, Orientation, Sizing, Widget,
    },
    sdl2::{render::Canvas, video::Window},
};

pub struct BrowserApplication<T: Widget> {
    root: T,
}

impl Default for BrowserApplication<Container> {
    fn default() -> Self {
        let mut textbox = Input::new(colorscheme::BACKGROUND_DARK).into_widget();
        textbox.set_size(Sizing::Exactly(50));

        let webcontent = ColoredBox::new(colorscheme::BACKGROUND_LIGHT).into_widget();

        let root = Container::new(Orientation::Vertical)
            .add_child(textbox)
            .add_child(webcontent);

        Self { root }
    }
}

impl<W: Widget> Application for BrowserApplication<W> {
    type Message = ();

    fn on_paint(&mut self, canvas: &mut Canvas<Window>) {
        self.root.render(canvas).unwrap();
        canvas.present();
    }
}
