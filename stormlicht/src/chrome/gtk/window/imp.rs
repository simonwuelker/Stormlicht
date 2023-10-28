use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use glib::subclass::InitializingObject;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/rs/stormlicht/ui/window.ui")]
pub struct Window {}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "StormlichtWindow";

    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Window {}
impl AdwApplicationWindowImpl for Window {}
impl WidgetImpl for Window {}
impl WindowImpl for Window {}
impl ApplicationWindowImpl for Window {}
