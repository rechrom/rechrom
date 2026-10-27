//! Compatibility facade for the reuse-owned native measurement cache.

#[path = "../../../layout_reuse/src/native_measure_cache.rs"]
mod layout_reuse_native_measure_cache;

pub use layout_reuse_native_measure_cache::MeasureCache;
