// Not a crate root: `layout_assembly` `include!`s this file as
// `layout_fragment_package`. The standalone `layout_fragment` crate is
// `package.rs` (+ `boundary_types.rs`, `trace.rs`); see its top comment for
// which compile unit owns each file in this directory.
//
// `api`, `internal_headers`, and `runtime` are Bazel targets of one package.
// This crate keeps their modules together; callers should use only interfaces
// allowed by the original target visibility.
pub mod block_break_token;
#[path = "../../layout_core/src/fragment/box_fragment_builder.rs"]
pub mod box_fragment_builder;
pub mod break_token;
pub mod break_token_algorithm_data;
#[path = "../../layout_core/src/fragment/fragment_builder.rs"]
pub mod fragment_builder;
pub mod fragment_data;
pub mod fragment_data_iterator;
pub mod fragment_item;
pub mod fragment_items;
#[path = "../../layout_core/src/fragment/fragment_items_builder.rs"]
pub mod fragment_items_builder;
pub mod fragment_tree;
pub mod inline_break_token;
pub mod inline_cursor;
#[path = "../../layout_core/src/fragment/inline_items_data.rs"]
pub mod inline_items_data;
pub mod layout_result;
#[path = "../../layout_core/src/fragment/logical_box_fragment.rs"]
pub mod logical_box_fragment;
#[path = "../../layout_core/src/fragment/logical_fragment.rs"]
pub mod logical_fragment;
#[path = "../../layout_core/src/fragment/logical_fragment_link.rs"]
pub mod logical_fragment_link;
#[path = "../../layout_core/src/fragment/logical_line_container.rs"]
pub mod logical_line_container;
#[path = "../../layout_core/src/fragment/logical_line_item.rs"]
pub mod logical_line_item;
pub mod physical_fragment;
pub mod physical_fragment_data;
pub mod physical_fragment_link;
pub mod physical_fragment_rare_data;
pub mod physical_line_box_fragment;
#[path = "../../layout_core/src/fragment/trace_bindings.rs"]
mod trace_bindings;

pub mod physical_box_fragment;
#[path = "../../layout_core/src/fragment/hit_test_geometry.rs"]
mod hit_test_geometry;
