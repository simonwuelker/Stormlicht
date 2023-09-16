#![feature(
    exclusive_range_pattern,
    associated_type_defaults,
    let_chains,
    lazy_cell,
    iter_advance_by
)]

mod browsing_context;
mod font_cache;
mod tree_debug;

pub mod css;
pub mod dom;
pub mod event;
pub mod html;
pub mod infra;

pub use browsing_context::BrowsingContext;
pub use font_cache::{FontCache, FONT_CACHE};
pub use tree_debug::{TreeDebug, TreeFormatter};
