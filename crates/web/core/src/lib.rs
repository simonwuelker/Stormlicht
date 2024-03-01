#![feature(
    exclusive_range_pattern,
    associated_type_defaults,
    let_chains,
    lazy_cell,
    iter_advance_by,
    extend_one,
    const_fn_floating_point_arithmetic
)]

mod browsing_context;
mod interned_string;
mod selection;
mod tree_debug;
mod window;

pub mod css;
pub mod dom;
pub mod event;
pub mod event_loop;
pub mod html;
pub mod infra;

pub use browsing_context::BrowsingContext;
pub use interned_string::InternedString;
pub use selection::Selection;
pub use tree_debug::{TreeDebug, TreeFormatter};
pub use window::Window;
