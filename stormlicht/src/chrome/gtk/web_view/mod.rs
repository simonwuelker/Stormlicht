use std::error::Report;

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
        if let Err(error) = self.imp().load_url(url) {
            log::error!(
                "Failed to load {url}:\n{}",
                Report::new(error).pretty(true).show_backtrace(true)
            );
        }
    }

    pub fn reload(&self) {
        if let Err(error) = self.imp().reload() {
            log::error!(
                "Failed to refresh page:\n{}",
                Report::new(error).pretty(true).show_backtrace(true)
            );
        }
    }

    pub fn handle_mouse_move(&self, x: f64, y: f64) {
        self.imp().handle_mouse_move(x, y);
    }
}

impl Default for WebView {
    fn default() -> Self {
        Self::new()
    }
}
