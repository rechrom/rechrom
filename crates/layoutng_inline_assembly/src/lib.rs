// LayoutNG, fragment tree, and inline have reciprocal source references.
// Compile their owning module files once in this assembly boundary. No
// translated implementation is copied into a new package.
extern crate self as layoutng;
extern crate self as layoutng_fragment_tree;
extern crate self as layoutng_inline;

pub use foundation::UnsupportedLayout;

include!("../../layoutng/src/lib.rs");
include!("../../layoutng_fragment_tree/src/lib.rs");
include!("../../layoutng_inline/src/lib.rs");

#[path = "../../css_parser/src/color_parser.rs"]
pub mod css_color_parser;

#[path = "../../layoutng_assembly/src/block_virtual_dispatch.rs"]
mod block_virtual_dispatch;
#[path = "../../layoutng_assembly/src/layout_invalidation_reason_symbols.rs"]
mod layout_invalidation_reason_symbols;
