cfg_match! {
    cfg(chrome = "glazier") => {
        mod glazier;
        pub use glazier::run;
    }
    cfg(chrome = "gtk") => {
        mod gtk;
        pub use gtk::run;
    }
    _ => {
        compile_error!("You must select one of the available frontends");
    }
}

/// Initial viewport width, in display points
const INITIAL_WIDTH: u16 = 800;

/// Initial viewport height, in display points
const INITIAL_HEIGHT: u16 = 600;

#[allow(dead_code)] // Not all chromes have to use this
const WELCOME_PAGE: &str = concat!(
    "file://localhost/",
    env!("CARGO_MANIFEST_DIR"),
    "/../pages/welcome.html"
);
