//! Private encoded clip ownership and real receiver adapter.
#[path = "color_glyph_clip/CanvasClipOwner.rs"]
mod owner;
pub(crate) use owner::dispatch::runs::receiver::Receiver;
pub(crate) use owner::*;
#[path = "color_glyph_clip/Producer.rs"]
mod producer;
pub(crate) use producer::{produce_path, rounded_commands, PathProduct};

pub(crate) use producer::cache::produce_path_cached;
pub use producer::cache::{
    RasterClipProductCache, RasterClipProductCacheAccounting, RasterClipProductCacheStats,
};
