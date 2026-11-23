#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

//! Layerize: the main-thread half of the compositor.
//!
//! Chromium runs PaintArtifactCompositor::Update and the cc LayerTreeHost side
//! of the commit on the main thread (local_frame_view.cc:2965-2975,3348;
//! cc/trees/proxy_main.cc BeginMainFrame). This crate is that half. The impl
//! half (`compositor`) may name only [`commit`], the values that cross the
//! ProxyMain -> ProxyImpl commit. See crates/compositor/MAIN_IMPL_SPLIT.md.

mod adapter;
mod cc;
pub mod commit;
mod skia_path;
