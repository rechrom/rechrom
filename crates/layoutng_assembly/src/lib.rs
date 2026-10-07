// LayoutNG and its fragment tree are mutually recursive C++ packages. Cargo
// cannot express that cycle between two crates, so compile their translated
// modules once in one assembly crate. The include paths point to the owning
// crates' source files and do not copy or relocate implementations.
extern crate self as layoutng;
extern crate self as layoutng_fragment_tree;
extern crate self as layoutng_inline;

pub use foundation::UnsupportedLayout;

include!("../../layoutng/src/lib.rs");
include!("../../layoutng_fragment_tree/src/lib.rs");
include!("../../layoutng_inline/src/lib.rs");

// CSS value parsers refer to LayoutNG-owned value types. Compile the owning
// css_parser source once at this reciprocal assembly boundary.
#[path = "../../css_parser/src/color_parser.rs"]
pub mod css_color_parser;

mod block_virtual_dispatch;
pub use block_virtual_dispatch::RegisterFlexStitchedRowGapIndex;
pub use block_virtual_dispatch::{
    GridObjectVirtuals, RegisterGridObjectVirtuals, RegisterGridStyleDidChange,
};
mod layout_invalidation_reason_symbols;
mod out_of_flow_element_data;

#[path = "../../layoutng_grid/src/gap_segment_state_aggregator.rs"]
pub mod grid_gap_segment_state_aggregator;

#[path = "../../layoutng_grid/src/grid_track_collection.rs"]
pub mod grid_track_collection_implementation;
