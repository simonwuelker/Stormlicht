mod graphics_state;
mod interpreter;
mod op;

pub use graphics_state::GraphicsState;
pub use interpreter::Interpreter;
use sl_std::fixed::Fixed;

pub type F26Dot6 = Fixed<6>;
