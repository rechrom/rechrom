// The fragment tree (Chromium `core/layout` fragment classes): break tokens,
// builders, PhysicalFragment/PhysicalBoxFragment, FragmentItem(s),
// InlineCursor, LayoutResult and the fragment Paint sidecars. Fragments are
// base: the sources live in `layout_core/src/fragment/` and these modules
// mount them at the assembly root paths (`crate::physical_box_fragment`, ...)
// that base and the algorithm crates name. The module keeps its historical
// `layout_fragment_package` name so profiler symbols stay comparable.
// `layout_algo_inline_assembly` declares this same file.
//
// `api`, `internal_headers`, and `runtime` are Bazel targets of one package.
// This module keeps their modules together; callers should use only
// interfaces allowed by the original target visibility.
#[path = "fragment/block_break_token.rs"]
pub mod block_break_token;
#[path = "fragment/box_fragment_builder.rs"]
pub mod box_fragment_builder;
#[path = "fragment/break_token.rs"]
pub mod break_token;
#[path = "fragment/break_token_algorithm_data.rs"]
pub mod break_token_algorithm_data;
#[path = "fragment/fragment_builder.rs"]
pub mod fragment_builder;
#[path = "fragment/fragment_data.rs"]
pub mod fragment_data;
#[path = "fragment/fragment_data_iterator.rs"]
pub mod fragment_data_iterator;
#[path = "fragment/fragment_item.rs"]
pub mod fragment_item;
#[path = "fragment/fragment_items.rs"]
pub mod fragment_items;
#[path = "fragment/fragment_items_builder.rs"]
pub mod fragment_items_builder;
#[path = "fragment/fragment_tree.rs"]
pub mod fragment_tree;
#[path = "fragment/inline_break_token.rs"]
pub mod inline_break_token;
#[path = "fragment/inline_cursor.rs"]
pub mod inline_cursor;
#[path = "fragment/inline_items_data.rs"]
pub mod inline_items_data;
#[path = "fragment/layout_result.rs"]
pub mod layout_result;
#[path = "fragment/logical_box_fragment.rs"]
pub mod logical_box_fragment;
#[path = "fragment/logical_fragment.rs"]
pub mod logical_fragment;
#[path = "fragment/logical_fragment_link.rs"]
pub mod logical_fragment_link;
#[path = "fragment/logical_line_container.rs"]
pub mod logical_line_container;
#[path = "fragment/logical_line_item.rs"]
pub mod logical_line_item;
#[path = "fragment/physical_fragment.rs"]
pub mod physical_fragment;
#[path = "fragment/physical_fragment_data.rs"]
pub mod physical_fragment_data;
#[path = "fragment/physical_fragment_link.rs"]
pub mod physical_fragment_link;
#[path = "fragment/physical_fragment_rare_data.rs"]
pub mod physical_fragment_rare_data;
#[path = "fragment/physical_line_box_fragment.rs"]
pub mod physical_line_box_fragment;
#[path = "fragment/trace_bindings.rs"]
mod trace_bindings;

#[path = "fragment/physical_box_fragment.rs"]
pub mod physical_box_fragment;
#[path = "fragment/hit_test_geometry.rs"]
mod hit_test_geometry;
