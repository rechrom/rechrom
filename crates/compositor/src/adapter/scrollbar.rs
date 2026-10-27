//! Facade compatibility exports for scrollbar input boundaries.

pub use super::layer_tree_impl::Color4f;
pub use compositor_input::scrollbar_controller_boundary::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Insets {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}
