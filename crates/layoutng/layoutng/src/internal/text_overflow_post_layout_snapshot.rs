//! Compatibility facade for fragment-owned text-overflow scroll snapshots.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_text_overflow_post_layout_snapshot.rs"]
mod layout_fragment_native_text_overflow_post_layout_snapshot;

pub use layout_fragment_native_text_overflow_post_layout_snapshot::*;
