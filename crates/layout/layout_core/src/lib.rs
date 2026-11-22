// `shared_api`, `internal_headers`, `nodes`, `shared_runtime`, boundary, and
// top-level core targets belong to this Bazel package. Modules are introduced
// as their production source files are translated.
pub mod internal {
    pub mod algorithm_entry;
    pub mod algorithm_forward;
    pub mod anchor_map;
    pub mod anchor_map_services;
    pub mod anchor_position_scroll_data;
    #[path = "../fragment/core/anchor_scope.rs"]
    pub mod anchor_scope;
    pub mod anchor_scroll_services;
    pub mod boundary {
        pub mod registration;
        pub mod control_theme_validation;
        pub mod font_from_selector;
        pub mod input;
        pub mod layout_boundary;
        pub mod node_metadata_input;
        pub mod paint_style_export;
    }
    pub mod background_bleed_avoidance;
    pub mod baseline_utils;
    pub mod block_node;
    pub mod block_node_data;
    pub mod break_appeal {
        #[path = "../../fragment/fragmentation/break_appeal.rs"]
        mod layout_algo_fragmentation_break_appeal;
        pub use layout_algo_fragmentation_break_appeal::*;
    }
    pub mod caret_rect;
    pub mod column_spanner_path;
    pub mod layout_block;
    pub mod layout_block_flow;
    pub mod css {
        pub mod counters_attachment_context;
        pub mod out_of_flow_data;
        pub mod successful_position_fallback;
    }
    #[path = "../fragment/core/constraint_space.rs"]
    pub mod constraint_space;
    pub mod constraint_space_builder;
    pub mod constraint_space_builder_style;
    pub mod constraint_space_custom_data {
        #[path = "../../fragment/custom/constraint_space_custom_data.rs"]
        mod layout_algo_custom_constraint_space_custom_data;
        pub use layout_algo_custom_constraint_space_custom_data::*;
    }
    pub mod content_change_type;
    pub mod custom_layout_payload {
        #[path = "../../fragment/custom/custom_layout_payload.rs"]
        mod layout_algo_custom_custom_layout_payload;
        pub use layout_algo_custom_custom_layout_payload::*;
    }
    pub mod custom_scrollbar;
    pub mod devtools_flex_info {
        #[path = "../../fragment/flex/devtools_flex_info.rs"]
        mod layout_algo_flex_devtools_flex_info;
        pub use layout_algo_flex_devtools_flex_info::*;
    }
    #[path = "../fragment/core/disable_layout_side_effects_scope.rs"]
    pub mod disable_layout_side_effects_scope;
    pub mod document_marker;
    pub mod early_break;
    pub mod editing {
        pub mod forward;
    }
    pub mod exclusions {
        pub mod exclusion_area {
            #[path = "../../../fragment/float/exclusion_area.rs"]
            mod layout_algo_float_exclusion_area;
            pub use layout_algo_float_exclusion_area::*;
        }
        pub mod exclusion_space {
            #[path = "../../../fragment/float/exclusion_space.rs"]
            mod layout_algo_float_exclusion_space;
            pub use layout_algo_float_exclusion_space::*;
        }
        pub mod layout_opportunity;
        #[path = "../../fragment/core/exclusions/line_layout_opportunity.rs"]
        pub mod line_layout_opportunity;
        pub mod shape_exclusions {
            #[path = "../../../fragment/float/shape_exclusions.rs"]
            mod layout_algo_float_shape_exclusions;
            pub use layout_algo_float_shape_exclusions::*;
        }
    }
    pub mod form_control_sizing_service;
    pub mod form_control_types {
        #[path = "../../fragment/forms/form_control_types.rs"]
        mod layout_algo_forms_form_control_types;
        pub use layout_algo_forms_form_control_types::*;
    }
    pub mod form_node_metadata;
    pub mod fragment_item_with_offset_fwd;
    pub mod fragment_repeater;
    pub mod fragmentation_utils;
    pub mod frame_set_layout_data {
        #[path = "../../fragment/frame_set_layout_data.rs"]
        mod layout_fragment_frame_set_layout_data;
        pub use layout_fragment_frame_set_layout_data::*;
    }
    pub mod gap {
        pub mod cross_gap {
            #[path = "../../../fragment/cross_gap.rs"]
            mod layout_fragment_cross_gap;
            pub use layout_fragment_cross_gap::*;
        }
        pub mod gap_decoration_utils;
        pub mod gap_geometry;
        pub mod gap_intersection {
            #[path = "../../../fragment/gap_intersection.rs"]
            mod layout_fragment_gap_intersection;
            pub use layout_fragment_gap_intersection::*;
        }
        #[allow(non_upper_case_globals)]
        pub mod gap_utils {
            #[path = "../../../fragment/gap_utils.rs"]
            mod layout_fragment_gap_utils;
            pub use layout_fragment_gap_utils::*;
        }
        pub mod main_gap {
            #[path = "../../../fragment/main_gap.rs"]
            mod layout_fragment_main_gap;
            pub use layout_fragment_main_gap::*;
        }
    }
    pub mod css_zoom;
    pub mod grid_item;
    pub mod grid_lanes_item_group;
    pub mod grid_layout_data {
        #[path = "../../fragment/grid/grid_layout_data.rs"]
        mod layout_algo_grid_grid_layout_data;
        pub use layout_algo_grid_grid_layout_data::*;
    }
    pub mod grid_subtree {
        #[path = "../../fragment/grid/grid_subtree.rs"]
        mod layout_algo_grid_grid_subtree;
        pub use layout_algo_grid_grid_subtree::*;
    }
    pub mod grid_track_collection {
        #[path = "../../fragment/grid/grid_track_collection_types.rs"]
        mod layout_algo_grid_grid_track_collection_types;
        pub use layout_algo_grid_grid_track_collection_types::*;
    }
    #[path = "../fragment/core/hit_test_phase.rs"]
    pub mod hit_test_phase;
    pub mod hyphen_result;
    pub mod ink_overflow;
    pub mod inline_containing_block_utils;
    pub mod inline_item;
    pub mod inline_item_result;
    pub mod inline_item_segment;
    pub mod inline_item_span;
    pub mod inline_item_text_index {
        #[path = "../../fragment/inline/inline_item_text_index.rs"]
        mod layout_algo_inline_inline_item_text_index;
        pub use layout_algo_inline_inline_item_text_index::*;
    }
    pub mod inline_node;
    pub mod inline_node_core;
    pub mod inline_node_data;
    pub mod layout_algorithm;
    pub mod layout_algorithm_set;
    pub mod layout_alignment_utils;
    pub mod layout_box;
    pub mod layout_box_core_services;
    pub mod layout_box_layout;
    pub mod layout_box_lifecycle;
    pub mod layout_box_model_lifecycle;
    pub mod layout_box_model_object;
    pub mod layout_box_model_tree;
    pub mod layout_box_scrollbar_gutter;
    pub mod layout_box_utils;
    pub mod layout_box_visual_overflow;
    pub mod layout_custom_scrollbar_part;
    pub mod layout_font_resolver;
    pub mod layout_geometry_invalidation;
    pub mod layout_inline;
    pub mod layout_input;
    pub mod layout_input_node;
    pub mod layout_input_node_data;
    pub mod layout_input_types;
    pub mod layout_invalidation_reason;
    pub mod layout_node_data;
    pub mod layout_node_metadata;
    pub mod layout_node_style;
    pub mod layout_object;
    pub mod layout_object_builder;
    pub mod layout_object_child_list;
    pub mod layout_object_containers;
    pub mod layout_object_core_services;
    pub mod layout_object_factory_set;
    pub mod layout_object_hot;
    pub mod layout_object_inlines;
    pub mod layout_object_tree;
    pub mod layout_pass_scope;
    pub mod layout_replaced;
    pub mod layout_scrollable_area;
    pub mod layout_sticky_constraints;
    pub mod layout_text;
    pub mod layout_text_combine;
    pub mod layout_text_core;
    #[path = "../fragment/core/layout_theme.rs"]
    pub mod layout_theme;
    pub mod layout_transform;
    pub mod layout_tree_invalidation;
    pub mod layout_utils;
    pub mod layout_view;
    pub mod length_utils;
    pub mod line_clamp_data {
        #[path = "../../fragment/block/line_clamp_data.rs"]
        mod layout_algo_block_line_clamp_data;
        pub use layout_algo_block_line_clamp_data::*;
    }
    pub mod motion_path_geometry;
    pub mod paint_client_commit;
    pub mod paint_layer;
    pub mod scrollbar;
    pub mod loader {
        pub mod fetch {
            pub mod resource_priority;
        }
        pub mod resource {
            pub mod image_resource_observer;
        }
    }
    #[path = "../fragment/core/map_coordinates_flags.rs"]
    pub mod map_coordinates_flags;
    pub mod mathml_paint_info {
        #[path = "../../fragment/mathml_paint_info.rs"]
        mod layout_fragment_mathml_paint_info;
        pub use layout_fragment_mathml_paint_info::*;
    }
    pub mod measure_cache;
    pub mod min_max_sizes;
    pub mod min_max_sizes_cache;
    pub mod multicol_break_token_data;
    pub mod naming_scope;
    pub mod node_rare_data_field;
    pub mod non_overflowing_scroll_range {
        #[path = "../../fragment/out_of_flow/non_overflowing_scroll_range.rs"]
        mod layout_algo_out_of_flow_non_overflowing_scroll_range;
        pub use layout_algo_out_of_flow_non_overflowing_scroll_range::*;
    }
    pub mod oof_positioned_node;
    pub mod outline_geometry;
    pub mod outline_info {
        #[path = "../../fragment/outline_info.rs"]
        mod layout_fragment_outline_info;
        pub use layout_fragment_outline_info::*;
    }
    pub mod outline_rect_collector;
    pub mod outline_utils;
    pub mod overflow_model;
    pub mod pagination_utils;
    pub mod paint_input;
    pub mod positioned_float;
    pub mod pre_paint_subtree_walk_reasons;
    pub mod relative_utils;
    pub mod resolved_text_layout_attributes_iterator;
    pub mod scroll_anchor;
    pub mod scroll_layout_scope;
    pub mod scroll_types;
    pub mod scrollable_overflow_calculator;
    pub mod scrollbar_mode;
    pub mod scrollbar_orientation;
    pub mod scrollbar_part;
    pub mod scrollbar_theme_metrics;
    #[path = "../fragment/core/selection_state.rs"]
    pub mod selection_state;
    pub mod shapes {
        pub mod shape {
            #[path = "../../../fragment/float/shape.rs"]
            mod layout_algo_float_shape;
            pub use layout_algo_float_shape::*;
        }
        pub mod shape_image_services;
        pub mod shape_interval;
        pub mod shape_outside_info;
    }
    pub mod snap_area;
    pub mod space_utils;
    #[path = "../fragment/core/split_axis_item.rs"]
    pub mod split_axis_item;
    pub mod sticky_position_scrolling_constraints;
    #[path = "../fragment/core/style_variant.rs"]
    pub mod style_variant;
    pub mod svg_character_data;
    pub mod svg_inline_node_data;
    pub mod svg_layout_info;
    pub mod svg_length_adjust_type;
    pub mod svg_text_attributes_request;
    pub mod table_borders;
    pub mod table_column_location {
        #[path = "../../fragment/table/table_column_location.rs"]
        mod layout_algo_table_table_column_location;
        pub use layout_algo_table_table_column_location::*;
    }
    pub mod table_constraint_space_data {
        #[path = "../../fragment/table/table_constraint_space_data.rs"]
        mod layout_algo_table_table_constraint_space_data;
        pub use layout_algo_table_table_constraint_space_data::*;
    }
    pub mod table_fragment_data;
    pub mod table_layout_algorithm_types;
    pub mod table_node;
    pub mod text_combine_style_service;
    #[path = "../fragment/core/text_decoration_offset.rs"]
    pub mod text_decoration_offset;
    pub mod text_fit_scale {
        #[path = "../../fragment/inline/text_fit_scale.rs"]
        mod layout_algo_inline_text_fit_scale;
        pub use layout_algo_inline_text_fit_scale::*;
    }
    pub mod text_item_type {
        #[path = "../../fragment/inline/text_item_type.rs"]
        mod layout_algo_inline_text_item_type;
        pub use layout_algo_inline_text_item_type::*;
    }
    #[path = "../fragment/core/text_offset_range.rs"]
    pub mod text_offset_range;
    pub mod text_overflow_post_layout_snapshot;
    pub mod transform_utils;
    pub mod tree_traversal_utils;
    pub mod trigger_scoped_name;
    pub mod unpositioned_float;
    pub mod unpositioned_list_marker;
    pub mod used_font {
        #[path = "../../fragment/inline/used_font_interface.rs"]
        mod layout_algo_inline_used_font_interface;
        pub use layout_algo_inline_used_font_interface::*;
    }
    pub mod variable_length_transform_result;
}

// Fragment data and the fragment tree are base (`fragment/`). The data
// modules are mounted under `internal::<module>` above; these are the
// fragment roots that sit beside them.
//
// Fragment-local Paint sidecar values and the paint-input schema re-exports
// (`fragment_tree` specializes the generic ones with `PaintStyle` and
// `PaintProperties`).
#[path = "fragment/boundary_types.rs"]
pub mod boundary_types;
// Lifetime handle of the canonical source LayoutResult.
#[path = "fragment/layout_result_handle.rs"]
pub mod layout_result_handle;
// Traceable bindings of the fragment data types in `internal`.
#[path = "fragment/fragment_data_trace_bindings.rs"]
mod fragment_data_trace_bindings;
// Method bodies of fragment data types whose declarations live in the type's
// file; that file imports them directly (they were link-time symbols before R7a).
// cpp: layoutng_grid/grid_track_collection.cc
#[path = "fragment/grid/grid_track_collection.rs"]
pub mod grid_track_collection_implementation;
// cpp: layoutng_inline/used_font.cc
#[path = "fragment/inline/used_font.rs"]
mod used_font_implementation;
// Gap segment-state aggregation over the gap types in `internal::gap`;
// `layout_algo_grid` re-exports it.
#[path = "fragment/grid/gap_segment_state_aggregator.rs"]
pub mod grid_gap_segment_state_aggregator;

// The per-box cached-result lookup is base code (`reuse/`); the module name
// keeps its reuse profiler attribution.
#[path = "reuse/cache_lookup.rs"]
pub mod layout_reuse_package;

// Completed-client tree traversal is base code (`commit/`); the module name
// keeps its commit profiler attribution.
#[path = "commit/client_commit.rs"]
pub mod layout_commit_package;

// Layout boundary object indexing lives in `fragment/`; the module name keeps
// the `layout_fragment` profiler attribution.
#[path = "fragment/layout_boundary_object_index.rs"]
pub mod layout_fragment_object_index;

// Sticky-position refresh lives in `fragment/`; the module name keeps the
// `layout_fragment` profiler attribution.
#[path = "fragment/paint_layer_scrollable_area_sticky_offsets.rs"]
pub mod layout_fragment_sticky_offsets;

// LayoutTextFragment (Chromium core/layout/layout_text_fragment.h) and
// OffsetMapping (core/layout/inline/offset_mapping.h) are LayoutObject and
// DOM-position classes that base, editing and inline all name. They sit under
// `inline/` with the other LayoutText/LayoutInline classes; base declares them
// so `layout_algo_inline` only depends on base, never the reverse.
#[cfg(feature = "translation_in_progress")]
#[path = "inline/layout_text_fragment.rs"]
pub mod layout_text_fragment;
// LayoutBR, LayoutCounter, LayoutRubyAsBlock and LayoutWordBreak (Chromium
// core/layout/layout_br.cc, layout_counter.cc, layout_ruby_as_block.cc,
// layout_word_break.cc) are object-model classes: base since R5 part 2.
// `layout_algo_inline::registration` still constructs them.
#[cfg(feature = "translation_in_progress")]
#[path = "inline/layout_br.rs"]
pub mod layout_br;
#[cfg(feature = "translation_in_progress")]
#[path = "inline/layout_counter.rs"]
pub mod layout_counter;
#[cfg(feature = "translation_in_progress")]
#[path = "inline/layout_ruby_as_block.rs"]
pub mod layout_ruby_as_block;
#[cfg(feature = "translation_in_progress")]
#[path = "inline/layout_word_break.rs"]
pub mod layout_word_break;
#[path = "inline/offset_mapping.rs"]
pub mod offset_mapping;

pub mod layout_registry;
pub mod layout_boundary_support;
pub mod layout_engine;
// The LayoutEngine method families (scroll, pointer, selection, form_control,
// observers, frame_view, event_registry, hit_test) live in layout_engine's
// `engine` child module; the facade's `layout::views` mirror them.
pub use layout_engine::engine;
mod layoutng_trace_bindings;
pub mod password_echo;
pub mod style_projection;

pub use internal::form_control_types::{AutofillState, FormControlType};
pub use internal::layout_algorithm_set::LayoutAlgorithmSet;
pub use internal::layout_input_types::{
    Color, ControlThemeMetrics, IntSize, ScrollbarThemeMetrics,
};
pub use internal::layout_object_factory_set::LayoutObjectFactorySet;
pub use internal::scrollbar_mode::mojom;

// Rust caret controller extension; no layout invalidation is implied by blinking.
pub mod caret;

pub mod editing_state;
pub mod layout_selection;
pub mod svg_filter_geometry;

// Canonical values crossing the Layout/editing boundary; no LayoutObject owner
// or dependency on an Interaction implementation crosses with them.
pub use internal::editing::forward::{
    EditingCaretFacts, EditingInlineAdjustmentFacts, EditingNodeFacts, EditingPositionOptions,
    EditingTextFacts, Position as EditingPosition, PositionAnchorType as EditingPositionAnchorType,
    PositionWithAffinity as EditingPositionWithAffinity, TextAffinity as EditingTextAffinity,
};

#[path = "editing/editing_caret.rs"]
pub(crate) mod layout_algo_inline_editing_caret;

#[path = "editing/editing_caret_rect.rs"]
pub(crate) mod layout_algo_inline_editing_caret_rect;


