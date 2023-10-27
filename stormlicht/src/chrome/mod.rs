pub mod gtk;

/// Initial viewport width, in display points
const INITIAL_WIDTH: u16 = 800;

/// Initial viewport height, in display points
const INITIAL_HEIGHT: u16 = 600;

const _WELCOME_PAGE: &str = concat!(
    "file://localhost/",
    env!("CARGO_MANIFEST_DIR"),
    "/../pages/welcome.html"
);
