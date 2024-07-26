use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};

use glib::subclass::InitializingObject;
use url::URL;

use crate::chrome::gtk::WebView;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/rs/stormlicht/ui/window.ui")]
pub struct Window {
    #[template_child]
    pub reload_button: TemplateChild<gtk::Button>,

    #[template_child]
    pub search_bar: TemplateChild<gtk::Entry>,

    #[template_child]
    pub web_view: TemplateChild<WebView>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "StormlichtWindow";

    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();

        klass.install_action_async(
            "open-file",
            None,
            |win, _action_name, _action_target| async move {
                match win.open_file_dialog().await {
                    Ok(file_path) => match URL::try_from(file_path.as_path()) {
                        Ok(url) => {
                            win.imp().web_view.load(&url);
                        },
                        Err(e) => {
                            log::error!("Failed to parse path as url: {e:?}");
                        },
                    },
                    Err(error) => log::error!("Error loading file: {error}"),
                }
            },
        );
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl AdwApplicationWindowImpl for Window {}
impl WidgetImpl for Window {}
impl WindowImpl for Window {}
impl ApplicationWindowImpl for Window {}

#[gtk::template_callbacks]
impl Window {
    #[template_callback]
    fn handle_url_entered(&self) {
        let text = self.search_bar.buffer().text();
        let url = match URL::from_user_input(text.as_str()) {
            Ok(parsed_url) => parsed_url,
            Err(error) => {
                log::error!("Failed to parse {text:?} as a URL: {error:?}");
                return;
            },
        };

        self.web_view.load(&url);
    }

    #[template_callback]
    fn handle_reload_page(&self) {
        self.web_view.reload()
    }

    #[template_callback]
    fn on_mouse_move(&self, x: f64, y: f64) {
        self.web_view.handle_mouse_move(x, y);
    }
}
