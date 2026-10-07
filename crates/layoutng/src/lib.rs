// `shared_api`, `internal_headers`, `nodes`, `shared_runtime`, boundary, and
// top-level core targets belong to this Bazel package. Modules are introduced
// as their production source files are translated.
pub mod internal {
    pub mod algorithm_entry;
    pub mod algorithm_forward;
    pub mod anchor_map;
    pub mod anchor_map_services;
    pub mod anchor_position_scroll_data;
    pub mod anchor_scope;
    pub mod anchor_scroll_services;
    pub mod boundary {
        pub mod assembly;
        pub mod control_theme_validation;
        pub mod font_from_selector;
        pub mod layout_boundary;
        pub mod native_input;
        pub mod node_metadata_input;
    }
    pub mod background_bleed_avoidance;
    pub mod baseline_utils;
    pub mod block_node;
    pub mod block_node_data;
    pub mod break_appeal;
    pub mod caret_rect;
    pub mod column_spanner_path;
    pub mod layout_block;
    pub mod layout_block_flow;
    pub mod css {
        pub mod counters_attachment_context;
        pub mod out_of_flow_data;
        pub mod successful_position_fallback;
    }
    pub mod constraint_space;
    pub mod constraint_space_builder;
    pub mod constraint_space_builder_style;
    pub mod constraint_space_custom_data;
    pub mod content_change_type;
    pub mod custom_layout_payload;
    pub mod custom_scrollbar;
    pub mod devtools_flex_info;
    pub mod disable_layout_side_effects_scope;
    pub mod document_marker;
    pub mod early_break;
    pub mod editing {
        pub mod forward;
    }
    pub mod exclusions {
        pub mod exclusion_area;
        pub mod exclusion_space;
        pub mod layout_opportunity;
        pub mod line_layout_opportunity;
        pub mod shape_exclusions;
    }
    pub mod form_control_sizing_service;
    pub mod form_control_types;
    pub mod form_node_metadata;
    pub mod fragment_item_with_offset_fwd;
    pub mod fragment_repeater;
    pub mod fragmentation_utils;
    pub mod frame_set_layout_data;
    pub mod gap {
        pub mod cross_gap;
        pub mod gap_decoration_utils;
        pub mod gap_geometry;
        pub mod gap_intersection;
        pub mod gap_utils;
        pub mod main_gap;
    }
    pub mod css_zoom;
    pub mod grid_item;
    pub mod grid_lanes_item_group;
    pub mod grid_layout_data;
    pub mod grid_subtree;
    pub mod grid_track_collection;
    pub mod hit_test_phase;
    pub mod hyphen_result;
    pub mod ink_overflow;
    pub mod inline_containing_block_utils;
    pub mod inline_item;
    pub mod inline_item_result;
    pub mod inline_item_segment;
    pub mod inline_item_span;
    pub mod inline_item_text_index;
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
    pub mod layout_transform;
    pub mod layout_tree_invalidation;
    pub mod layout_utils;
    pub mod layout_view;
    pub mod length_utils;
    pub mod line_clamp_data;
    pub mod native_scrollbar;
    pub mod paint_client_commit;
    pub mod paint_layer;
    pub mod loader {
        pub mod fetch {
            pub mod resource_priority;
        }
        pub mod resource {
            pub mod image_resource_observer;
        }
    }
    pub mod map_coordinates_flags;
    pub mod mathml_paint_info;
    pub mod measure_cache;
    pub mod min_max_sizes;
    pub mod min_max_sizes_cache;
    pub mod multicol_break_token_data;
    pub mod naming_scope;
    pub mod node_rare_data_field;
    pub mod non_overflowing_scroll_range;
    pub mod oof_positioned_node;
    pub mod outline_geometry;
    pub mod outline_info;
    pub mod outline_rect_collector;
    pub mod outline_utils;
    pub mod overflow_model;
    pub mod pagination_utils;
    pub mod paint_input;
    pub mod positioned_float;
    pub mod pre_paint_disable_side_effects_scope;
    pub mod pre_paint_subtree_walk_reasons;
    pub mod relative_utils;
    pub mod resolved_text_layout_attributes_iterator;
    pub mod scroll_layout_scope;
    pub mod scroll_types;
    pub mod scrollable_overflow_calculator;
    pub mod scrollbar_mode;
    pub mod scrollbar_orientation;
    pub mod scrollbar_part;
    pub mod scrollbar_theme_metrics;
    pub mod selection_state;
    pub mod shapes {
        pub mod shape;
        pub mod shape_image_services;
        pub mod shape_interval;
        pub mod shape_outside_info;
    }
    pub mod snap_area;
    pub mod space_utils;
    pub mod split_axis_item;
    pub mod sticky_position_scrolling_constraints;
    pub mod style_variant;
    pub mod svg_character_data;
    pub mod svg_inline_node_data;
    pub mod svg_layout_info;
    pub mod svg_length_adjust_type;
    pub mod svg_text_attributes_request;
    pub mod table_borders;
    pub mod table_column_location;
    pub mod table_constraint_space_data;
    pub mod table_fragment_data;
    pub mod table_layout_algorithm_types;
    pub mod table_node;
    pub mod text_combine_style_service;
    pub mod text_fit_scale;
    pub mod text_item_type;
    pub mod text_offset_range;
    pub mod text_overflow_post_layout_snapshot;
    pub mod transform_utils;
    pub mod tree_traversal_utils;
    pub mod trigger_scoped_name;
    pub mod unpositioned_float;
    pub mod unpositioned_list_marker;
    pub mod used_font;
    pub mod variable_length_transform_result;
}

pub mod layout_assembly;
pub mod layout_boundary_support;
pub mod layout_engine;
mod layoutng_trace_bindings;

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
