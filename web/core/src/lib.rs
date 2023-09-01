#![feature(
    exclusive_range_pattern,
    associated_type_defaults,
    let_chains,
    lazy_cell
)]

mod browsing_context;
mod font_cache;
mod tree_debug;

pub mod css;
pub mod dom;
pub mod html;
pub mod infra;

pub use browsing_context::BrowsingContext;
pub use font_cache::{FontCache, FONT_CACHE};
pub use tree_debug::{TreeDebug, TreeFormatter};
