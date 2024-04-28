use image::{Rgbaf32, Texture};
use sl_std::safe_casts::cast_slice;
use url::URL;
use webcore::{BrowsingContext, BrowsingContextError};

use std::{cell::RefCell, mem};

use adw::subclass::prelude::*;
use gtk::{gdk, glib, prelude::*, CompositeTemplate};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/rs/stormlicht/ui/web_view.ui")]
pub struct WebView {
    state: RefCell<State>,
}

struct State {
    view_buffer: Texture,
    browsing_context: BrowsingContext,
    composition: render::Composition,
}

impl Default for State {
    fn default() -> Self {
        Self {
            view_buffer: Texture::new(0, 0),
            browsing_context: BrowsingContext::default(),
            composition: render::Composition::default(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for WebView {
    const NAME: &'static str = "WebView";
    type Type = super::WebView;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for WebView {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for WebView {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let widget = self.obj();

        let device_width = widget.width();
        let device_height = widget.height();

        if device_width <= 0 || device_height <= 0 {
            return;
        }

        let scale_factor = widget.scale_factor();
        self.state
            .borrow_mut()
            .composition
            .set_dpi((scale_factor as f32, scale_factor as f32));

        let window_width = (device_width * scale_factor) as usize;
        let window_height = (device_height * scale_factor) as usize;

        self.state
            .borrow_mut()
            .view_buffer
            .resize_buffer(window_width as usize, window_height as usize);

        self.state
            .borrow_mut()
            .paint(device_width as u16, device_height as u16);

        let state = self.state.borrow();
        let buffer_bytes: &[u8] = cast_slice(state.view_buffer.data());

        gdk::MemoryTexture::new(
            window_width as i32,
            window_height as i32,
            gdk::MemoryFormat::R32g32b32a32FloatPremultiplied,
            &glib::Bytes::from(buffer_bytes),
            mem::size_of::<Rgbaf32>() * window_width as usize,
        )
        .snapshot(snapshot, device_width as f64, device_height as f64);
    }
}

impl WebView {
    pub fn load_url(&self, url: &URL) -> Result<(), BrowsingContextError> {
        self.state.borrow_mut().browsing_context.load(url)?;
        self.obj().queue_draw();

        Ok(())
    }
}

impl State {
    fn paint(&mut self, width: u16, height: u16) {
        self.view_buffer.clear(Rgbaf32::rgb(1., 1., 1.));
        self.composition.clear();

        self.browsing_context
            .paint(&mut self.composition, (width, height));
        self.composition.render_to(&mut self.view_buffer);
    }
}
