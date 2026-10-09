#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod active_style_sheets;
pub mod cascade_layer;
pub mod cascade_layer_map;
pub mod cascade_layered;
pub mod color_scheme_flags;
pub mod css_anchor_query_enums;
pub mod css_color_channel_keywords;
pub mod css_color_channel_map;
pub mod css_default_style_sheets;
pub mod css_global_rule_set;
pub mod css_keyframe_rule;
pub mod css_keyframes_rule;
pub mod css_markup;
pub mod css_numeric_literal_value;
pub mod css_origin_clean;
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
pub mod css_style_sheet;
pub mod css_syntax_component;
pub mod css_value;
pub mod css_value_clamping_utils;
pub mod css_value_keywords;
pub mod element_rule_collector;
pub mod fixed_size_cache;
pub mod hash_tools;
pub mod invalidation;
pub mod kleene_value;
pub mod media_queries;
pub mod media_value_change;
pub mod parser;
pub mod pending_sheet_type;
pub mod persistent_selector;
pub mod post_style_update_scope;
pub mod production_css_value;
#[path = "production_border_image_values.rs"]
pub(crate) mod production_border_image_values;
#[path = "production_reflect_value.rs"]
pub(crate) mod production_reflect_value;
mod production_position_area;
mod production_line_features;
mod production_typography_features;
mod production_text_box_features;
mod production_interaction_features;
pub mod production_style_sheet;
pub mod production_style_sheet_projection;
pub mod properties;
pub mod property_bitsets;
pub mod resolver;
pub mod rule_set;
pub mod seeker;
pub mod selector_checker;
pub mod selector_filter;
pub mod selector_query;
pub mod style_engine;
pub mod style_environment_variables;
pub mod style_recalc_change;
pub mod style_recalc_context;
pub mod style_rule;
pub mod style_rule_counter_style;
pub mod style_rule_css_style_declaration;
pub mod style_rule_font_feature_values;
pub mod style_rule_font_palette_values;
pub mod style_rule_function_declarations;
pub mod style_rule_import;
pub mod style_rule_keyframe;
pub mod style_rule_location;
pub mod style_rule_namespace;
pub mod style_rule_nested_declarations;
pub mod style_rule_view_transition;
pub mod style_sheet_collection;
pub mod style_sheet_contents;
pub mod valid_property_filter;
pub mod vision_deficiency;
pub mod white_space;

pub use color_scheme_flags::{ColorSchemeFlag, ColorSchemeFlags};
pub use css_origin_clean::OriginClean;
pub use css_resource_fetch_restriction::ResourceFetchRestriction;
pub use media_queries::preferred_color_scheme::PreferredColorScheme;
pub use media_value_change::MediaValueChange;
pub use pending_sheet_type::PendingSheetType;
pub use production_style_sheet_projection::{ParseCSS, ParseCSSDeclarationList};
pub use style_engine::{
    AllowMarkForReattachFromRebuildLayoutTreeScope, AllowMarkStyleDirtyFromRecalcScope,
    AncestorAnalysis, AttachScrollMarkersScope, DOMRemovalScope, DetachLayoutTreeScope,
    InApplyAnimationUpdateScope, InEnsureComputedStyleScope, InvalidationScope,
    SkipStyleRecalcScope, StyleSheetKey,
};
pub use vision_deficiency::{CreateVisionDeficiencyFilterUrl, VisionDeficiency};

pub mod document_style_engine;
pub use document_style_engine::{DocumentStyleEngine as StyleEngine, DocumentStyleError};
#[cfg(test)]
mod document_style_engine_test;

pub mod container_query;

pub mod production_container_parser;
pub mod production_container_projection;

pub mod css_syntax_definition;
pub mod css_syntax_string_parser;
pub mod style_scope;
pub mod property_registration;
pub mod property_registry;

pub mod production_rule_effect_projection;

pub mod css_math_expression_node;
pub mod css_math_function_value;

pub mod production_effects_value;
pub mod production_corner_value;
mod production_corner_features;
pub mod production_dynamic_range_value;
mod production_render_delay_features;
pub mod production_motion_value;
