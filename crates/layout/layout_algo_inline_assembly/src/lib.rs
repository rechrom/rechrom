// LayoutNG, fragment tree, and inline have reciprocal source references.
// Compile their owning module files once in this assembly boundary. No
// translated implementation is copied into a new package.
extern crate self as layout_algo_inline;
extern crate self as layout_core;

pub use foundation::UnsupportedLayout;

include!("../../layout_core/src/lib.rs");

// The fragment-tree modules (fragment data is part of the included base root).
#[path = "../../layout_assembly/src/layout_fragment_package.rs"]
pub mod layout_fragment_package;
pub use layout_fragment_package::*;

pub mod layout_algo_inline_package {
    include!("../../layout_algo_inline/src/lib.rs");
}
pub use layout_algo_inline_package::*;

pub use css_parser::color_parser as css_color_parser;

#[path = "../../layout_assembly/src/block_virtual_dispatch.rs"]
mod block_virtual_dispatch;
#[path = "../../layout_assembly/src/layout_invalidation_reason_symbols.rs"]
mod layout_invalidation_reason_symbols;
