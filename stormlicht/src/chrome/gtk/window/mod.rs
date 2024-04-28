mod imp;

use std::path::PathBuf;

use glib::Object;
use gtk::{gio, glib, prelude::*};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    async fn open_file_dialog(&self) -> Result<PathBuf, glib::Error> {
        let filter = gtk::FileFilter::new();
        filter.add_mime_type("text/html");
        filter.set_name(Some(".html"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .title("Open File")
            .accept_label("Open")
            .modal(true)
            .filters(&filters)
            .build();

        let file = dialog.open_future(Some(self)).await?;
        let path = file.path().expect("Path should always exist");

        Ok(path)
    }
}
