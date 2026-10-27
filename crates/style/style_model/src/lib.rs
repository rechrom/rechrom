#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Shared data types carried across the Style pipeline.
//!
//! This crate is the dependency root of the split Style subsystem. It owns
//! value identity, intrinsic value behavior, property identifiers, compact
//! enums, and rule data objects, but no parsing, selector matching,
//! invalidation traversal, cascade, DOM mutation, or lifecycle orchestration.

pub mod at_rule_descriptors;
pub mod color_scheme_flags;
pub mod color_space_gamut;
pub mod container_state;
pub mod css_anchor_query_enums;
pub mod css_at_rule_id;
pub mod css_color_channel_keywords;
pub mod css_color_channel_map;
pub mod css_markup;
pub mod css_nesting_type;
pub mod css_origin_clean;
pub mod css_parser_mode;
pub mod css_primitive_value;
pub mod css_property_name;
pub mod css_property_names;
pub mod css_property_value;
pub mod css_property_value_set;
pub mod css_reflection_direction;
pub mod css_resolution_units;
pub mod css_resource_fetch_restriction;
pub mod css_selector;
pub mod css_selector_list;
pub mod css_syntax_component;
pub mod css_syntax_predicates;
pub mod css_value;
pub mod css_value_clamping_utils;
pub mod css_value_keywords;
pub mod device_posture_provider;
pub mod display_mode;
pub mod forced_colors;
pub mod hash_tools;
pub mod kleene_value;
pub mod media_value_change;
pub mod media_values;
pub mod media_values_cached_data;
pub mod navigation_controls;
pub mod pending_sheet_type;
pub mod preferred_color_scheme;
pub mod preferred_contrast;
pub mod production_corner_features;
pub mod production_interaction_features;
pub mod production_line_features;
pub mod production_render_delay_features;
pub mod production_text_box_features;
pub mod production_typography_features;
pub mod properties;
pub mod property_bitsets;
pub mod property_metadata;
pub mod scripting;
pub mod style_rule;
pub mod style_rule_counter_style;
pub mod style_rule_css_style_declaration;
pub mod style_rule_font_feature_values;
pub mod style_rule_font_palette_values;
pub mod style_rule_function_declarations;
pub mod style_rule_keyframe;
pub mod style_rule_location;
pub mod style_rule_namespace;
pub mod style_rule_nested_declarations;
pub mod style_rule_view_transition;
pub mod valid_property_filter;
pub mod vision_deficiency;
pub mod web_preferences;
pub mod white_space;
pub mod window_show_state;
