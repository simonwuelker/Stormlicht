use widgets::{
    application::Application,
    colorscheme,
    layout::{
        widgets::{ColoredBox, Input},
        Divider, Orientation, Sizing, Widget,
    },
    sdl2::{render::Canvas, video::Window},
};

pub struct BrowserApplication<T: Widget> {
    root: T,
}

impl Default for BrowserApplication<Divider> {
    fn default() -> Self {
        let mut textbox = Input::new(colorscheme::BACKGROUND_DARK).into_widget();
        textbox.set_size(Sizing::Exactly(50));

        let root = Divider::new(Orientation::Vertical)
            .add_child(textbox)
            .add_child(ColoredBox::new(colorscheme::BACKGROUND_LIGHT).into_widget());

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
