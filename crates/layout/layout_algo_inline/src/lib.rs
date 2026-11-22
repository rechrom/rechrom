// The inline layout algorithm package (Chromium core/layout/inline/; snapshot
// //src/layoutng_inline). It compiles as its own crate above base
// (`layout_base`) and reaches base only through its public API. Base
// reaches it back only through the registry slots `registration::Register`
// fills; there is no link-time (`extern "Rust"` / `#[no_mangle]`) coupling.
// Members of base-declared types whose bodies live here are extension traits
// (`algorithms`). See crates/layout/PACKAGE_MAP.md, R5 status.
// Translated Chromium identifiers keep their C++ spelling.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// The translated sources name base items through Chromium's `layoutng`
// package path, like the block package does.

// Fragment-tree modules that are still compiled with base. The inline sources
// keep their `crate::<module>` paths for them.
use layout_base::{
    block_break_token, box_fragment_builder, break_token, fragment_builder, fragment_item,
    fragment_items_builder, inline_break_token, inline_cursor, inline_items_data,
    layout_result, logical_box_fragment, logical_fragment, logical_line_container,
    logical_line_item, physical_box_fragment, physical_fragment, physical_line_box_fragment,
};
// Base LayoutObject / DOM-position classes that the inline sources name
// beside their own modules.
pub use layout_base::{layout_text_fragment, offset_mapping};
// LayoutBR, LayoutCounter, LayoutRubyAsBlock and LayoutWordBreak (Chromium
// core/layout/layout_*.cc) are object-model classes, base since R5.
pub use layout_base::{layout_br, layout_counter, layout_ruby_as_block, layout_word_break};

pub mod registration;
pub mod algorithms;
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
pub mod inline_layout_algorithm;
pub mod inline_node;
pub mod justification_utils;
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
