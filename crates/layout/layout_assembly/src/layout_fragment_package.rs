// Fragment-tree modules that still dereference base LayoutObject behaviour.
//
// `layout_fragment` is a real crate (`layout_fragment/src/lib.rs`) holding the
// fragment data that needs no base type. The files declared here are the
// fragment-tree types whose methods still call LayoutObject/LayoutBox
// behaviour, take base builders or embed base values (see
// `crates/layout/PACKAGE_MAP.md` §7, R4 status). Until those methods move into
// base extension traits, this assembly is their only compile unit; the
// sources stay in `layout_fragment/src/` because their data belongs to the
// fragment crate. `layout_algo_inline_assembly` declares this same file.
//
// `api`, `internal_headers`, and `runtime` are Bazel targets of one package.
// This module keeps their modules together; callers should use only
// interfaces allowed by the original target visibility.
#[path = "../../layout_fragment/src/block_break_token.rs"]
pub mod block_break_token;
#[path = "../../layout_core/src/fragment/box_fragment_builder.rs"]
pub mod box_fragment_builder;
#[path = "../../layout_fragment/src/break_token.rs"]
pub mod break_token;
#[path = "../../layout_fragment/src/break_token_algorithm_data.rs"]
pub mod break_token_algorithm_data;
#[path = "../../layout_core/src/fragment/fragment_builder.rs"]
pub mod fragment_builder;
#[path = "../../layout_fragment/src/fragment_data.rs"]
pub mod fragment_data;
#[path = "../../layout_fragment/src/fragment_data_iterator.rs"]
pub mod fragment_data_iterator;
#[path = "../../layout_fragment/src/fragment_item.rs"]
pub mod fragment_item;
#[path = "../../layout_fragment/src/fragment_items.rs"]
pub mod fragment_items;
#[path = "../../layout_core/src/fragment/fragment_items_builder.rs"]
pub mod fragment_items_builder;
#[path = "../../layout_fragment/src/fragment_tree.rs"]
pub mod fragment_tree;
#[path = "../../layout_fragment/src/inline_break_token.rs"]
pub mod inline_break_token;
#[path = "../../layout_fragment/src/inline_cursor.rs"]
pub mod inline_cursor;
#[path = "../../layout_core/src/fragment/inline_items_data.rs"]
pub mod inline_items_data;
#[path = "../../layout_fragment/src/layout_result.rs"]
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
#[path = "../../layout_fragment/src/physical_fragment.rs"]
pub mod physical_fragment;
#[path = "../../layout_fragment/src/physical_fragment_data.rs"]
pub mod physical_fragment_data;
#[path = "../../layout_fragment/src/physical_fragment_link.rs"]
pub mod physical_fragment_link;
#[path = "../../layout_fragment/src/physical_fragment_rare_data.rs"]
pub mod physical_fragment_rare_data;
#[path = "../../layout_fragment/src/physical_line_box_fragment.rs"]
pub mod physical_line_box_fragment;
#[path = "../../layout_core/src/fragment/trace_bindings.rs"]
mod trace_bindings;

#[path = "../../layout_fragment/src/physical_box_fragment.rs"]
pub mod physical_box_fragment;
#[path = "../../layout_core/src/fragment/hit_test_geometry.rs"]
mod hit_test_geometry;
