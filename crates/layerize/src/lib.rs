#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

//! Layerize: the main-thread half of the compositor.
//!
//! Chromium runs PaintArtifactCompositor::Update and the cc LayerTreeHost side
//! of the commit on the main thread (local_frame_view.cc:2965-2975,3348;
//! cc/trees/proxy_main.cc BeginMainFrame). This crate is that half. The impl
//! half (`compositor`) may name only [`commit`], the values that cross the
//! ProxyMain -> ProxyImpl commit. See crates/compositor/MAIN_IMPL_SPLIT.md.

mod adapter;
pub mod third_party;
mod cc;
pub mod commit;
mod skia_path;
mod layerize_engine;
/// The main-half engine. During S1a only the compositor's EngineCore and
/// EngineAssembly name it (S1b moves it to Page).
pub use layerize_engine::LayerizeEngine;
/// Paint-value bindings of the main half. Until S1a-3 B3 the compositor's
/// EngineCore still drives them directly.
pub mod runtime;

/// Main-half type families of the Blink compositing translations. Until
/// S1a-3 B3 the compositor's EngineCore still drives them directly.
pub mod main_types {
    pub use crate::adapter::{
        content_layer_client_impl, paint_artifact_compositor, paint_chunks_to_cc_layer,
        paint_geometry, pending_layer, property_tree_manager, raster_invalidator,
    };
}
