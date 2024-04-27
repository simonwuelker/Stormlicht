use glib::Object;
use gtk::{glib, subclass::prelude::*};
use url::URL;

mod imp;

glib::wrapper! {
    pub struct WebView(ObjectSubclass<imp::WebView>)
        @extends gtk::Widget;
}

impl WebView {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn load(&self, url: &URL) {
        self.imp().load_url(url).unwrap();
    }
}

impl Default for WebView {
    fn default() -> Self {
        Self::new()
    }
}
