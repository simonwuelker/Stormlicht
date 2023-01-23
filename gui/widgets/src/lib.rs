pub mod colorscheme;
pub mod layout;
pub mod primitives;

pub use sdl2;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuiError {
    #[error("SDL Error: {:?}", .0)]
    SDL(String),
}

impl GuiError {
    pub fn from_sdl(error_msg: String) -> Self {
        Self::SDL(error_msg)
    }
}
