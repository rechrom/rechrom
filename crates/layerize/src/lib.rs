#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

//! Layerize: the main-thread half of the compositor.
//!
//! Chromium runs PaintArtifactCompositor::Update and the cc LayerTreeHost side
//! of the commit on the main thread (local_frame_view.cc:2965-2975,3348;
//! cc/trees/proxy_main.cc BeginMainFrame). This crate is that half. The impl
//! half (`compositor`) may name only [`commit`], the values that cross the
//! ProxyMain -> ProxyImpl commit.

mod adapter;
pub mod third_party;
mod cc;
pub mod commit;
mod skia_path;
mod layerize_engine;
/// The main-half engine. During S1a only the compositor's EngineCore and
/// EngineAssembly name it (S1b moves it to Page).
pub use layerize_engine::LayerizeEngine;
/// Paint-value bindings, main paint services and the main LayerTreeHost.
/// Public because LayerizeEngine's API returns these owners; the impl half
/// names none of them (crates/layerize/tests/layerize_boundary.rs).
pub mod runtime;
