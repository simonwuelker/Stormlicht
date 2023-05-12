mod alignment;
pub mod application;
pub mod colorscheme;
pub mod layout;
pub mod primitives;

pub use alignment::Alignment;
pub use sdl2;

#[derive(Clone, Debug)]
pub enum GuiError {
    SDL(String),
}