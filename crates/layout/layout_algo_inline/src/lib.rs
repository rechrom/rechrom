// The inline layout algorithm package (Chromium core/layout/inline/; snapshot
// //src/layoutng_inline). It compiles as its own crate above base
// (`layout_assembly`) and reaches base only through its public API. Base
// reaches it back through the slots `registration::Register` fills and the
// `extern "Rust"` declarations of the LayoutText / LayoutInline / InlineNode /
// InlineItem members whose Chromium bodies live in this package's .cc files
// (see crates/layout/PACKAGE_MAP.md, R5 status).
#![allow(non_snake_case)]

// The translated sources name base items through Chromium's `layoutng`
// package path, like the block package does.
extern crate layout_assembly as layout_core;

// Fragment-tree modules that are still compiled with base. The inline sources
// keep their `crate::<module>` paths for them.
use layout_assembly::{
    block_break_token, box_fragment_builder, break_token, fragment_builder, fragment_item,
    fragment_items, fragment_items_builder, inline_break_token, inline_cursor, inline_items_data,
    layout_result, logical_box_fragment, logical_fragment, logical_line_container,
    logical_line_item, physical_box_fragment, physical_fragment, physical_line_box_fragment,
};
// Base LayoutObject / DOM-position classes that the inline sources name
// beside their own modules.
pub use layout_assembly::{layout_text_fragment, offset_mapping};

pub mod registration;
pub mod empty_offset_mapping_builder;
pub mod hyphen_result;
pub mod initial_letter_utils;
pub mod inline_box_state;
pub mod inline_child_layout_context;
pub mod inline_item;
pub mod inline_item_result;
pub mod inline_item_result_ruby_column;
pub mod inline_item_segment;
pub mod inline_items_builder;
pub mod inline_items_data_algorithm;
pub mod inline_layout_algorithm;
pub mod inline_node;
pub mod justification_utils;
pub mod layout_br;
pub mod layout_counter;
pub mod layout_ruby_as_block;
pub mod layout_word_break;
pub mod leading_floats;
pub mod line_box_fragment_builder;
pub mod line_break_candidate;
pub mod line_break_point;
pub mod line_breaker;
pub mod line_info;
pub mod line_info_list;
pub mod line_truncator;
pub mod line_utils;
pub mod line_widths;
pub mod logical_line_builder;
pub mod offset_mapping_builder;
pub mod paragraph_line_breaker;
pub mod physical_line_box_fragment_builder;
pub mod ruby_utils;
pub mod score_line_break_context;
pub mod score_line_breaker;
pub mod text_auto_space;
pub mod text_combine_tree;
pub mod text_diff_range;
pub mod text_fit_utils;
pub mod transformed_string;
