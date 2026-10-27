// LayoutNG, fragment tree, and inline have reciprocal source references.
// Compile their owning module files once in this assembly boundary. No
// translated implementation is copied into a new package.
extern crate self as layout_algo_inline;
extern crate self as layout_fragment;
extern crate self as layoutng;

pub use foundation::UnsupportedLayout;

include!("../../layoutng/src/lib.rs");

pub mod layout_fragment_package {
    include!("../../layout_fragment/src/lib.rs");
}
pub use layout_fragment_package::*;

pub mod layout_algo_inline_package {
    include!("../../layout_algo_inline/src/lib.rs");
}
pub use layout_algo_inline_package::*;

#[path = "../../../css_parser/src/color_parser.rs"]
pub mod css_color_parser;

#[path = "../../layoutng_assembly/src/block_virtual_dispatch.rs"]
mod block_virtual_dispatch;
#[path = "../../layoutng_assembly/src/layout_invalidation_reason_symbols.rs"]
mod layout_invalidation_reason_symbols;
