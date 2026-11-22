//! Method families of the core `LayoutEngine`, one directory per family.
//!
//! The families match the facade: each `layout::views::<family>` query view
//! and each `LayoutMutation` family variant forwards to the `impl
//! LayoutEngine` blocks of the directory with the same name. This module is a
//! child of `layout_engine` so the blocks keep the engine's private state; the
//! crate root re-exports it as `crate::engine`.
//!
//! Files that add methods to `LayoutNode` or the hit-test cursors need
//! `LayoutOutput`'s private fields, so `layout_output.rs` declares them with
//! `#[path]` into these directories; each family's doc lists them.

pub mod event_registry;
pub mod form_control;
pub mod frame_view;
pub mod hit_test;
pub mod observers;
pub mod pointer;
pub mod scroll;
pub mod selection;
