#![allow(non_snake_case)]

use layoutng_style::style::computed_style::ComputedStyle;

use super::layout_node_metadata::Node;

// The declaration has no definition in the supplied source tree.
unsafe extern "Rust" {
    // cpp: layoutng/internal/outline_utils.h:13
    pub fn HasPaintedOutline(style: &ComputedStyle, node: *const Node) -> bool;
}
