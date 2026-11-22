//! Compatibility facade for the reuse-owned source measurement cache.

#[path = "../reuse/measure_cache.rs"]
mod layout_reuse_measure_cache;

pub use layout_reuse_measure_cache::MeasureCache;
