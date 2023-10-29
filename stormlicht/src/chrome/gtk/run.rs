use crate::chrome::{INITIAL_HEIGHT, INITIAL_WIDTH};

use super::Window;

use std::process::ExitCode;

use gtk::{gio, glib, prelude::*};

const APP_ID: &str = "rs.stormlicht.browser";

pub fn run() -> ExitCode {
    gio::resources_register_include!("composite_template.gresource")
        .expect("Failed to register resources.");
    // let resource = gio::Resource::load(
    //     crate::env::pkg_data_dir()
    //         .expect("Could not retrieve pkg data dir")
    //         .join(globals::GRESOURCES_FILENAME),
    // )
    // .expect("Could not load gresource file");
    // gio::resources_register(&resource);

    let application = adw::Application::builder().application_id(APP_ID).build();

    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(
        glib::clone!(@weak application => move |_action, _parameter| {
            application.quit();
        }),
    );
    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    application.add_action(&quit);

    application.connect_activate(build_ui);

    let glib_exit_code = application.run();
    ExitCode::from(glib_exit_code.value() as u8)
}

fn build_ui(app: &adw::Application) {
    let window = Window::new(app);
    window.set_default_width(INITIAL_WIDTH as i32);
    window.set_default_height(INITIAL_HEIGHT as i32);
    window.present();
}
