#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Style lifecycle facade and DOM/CSSOM adapters.
//!
//! Performance traces preserve the package boundaries used by the pipeline:
//! `style_parser`, `style_rules`, `style_invalidation`, `style_walk`,
//! `style_match`, `style_resolve`, `style_diff`, and `style_commit`. The
//! top-level `style` category measures lifecycle orchestration and may contain
//! those package spans. `style_model` is deliberately charged to its caller
//! because it owns shared values rather than an executable lifecycle stage.

pub use style_model::color_scheme_flags;
pub use style_model::css_anchor_query_enums;
pub use style_model::css_color_channel_keywords;
pub use style_model::css_color_channel_map;
pub use style_rules::active_style_sheets;
pub use style_rules::cascade_layer;
pub use style_rules::cascade_layer_map;
pub use style_rules::cascade_layered;
pub mod css_default_style_sheets;
pub mod css_global_rule_set;
pub mod css_keyframe_rule;
pub mod css_keyframes_rule;
pub use style_model::css_markup;
pub mod css_numeric_literal_value;
pub use style_model::css_origin_clean;
pub use style_model::css_primitive_value;
pub use style_model::css_property_name;
pub use style_model::css_property_names;
pub use style_model::css_property_value;
pub mod css_property_value_set;
pub use style_model::css_reflection_direction;
pub use style_model::css_resolution_units;
pub use style_model::css_resource_fetch_restriction;
pub use style_model::css_selector;
pub use style_model::css_selector_list;
pub mod css_style_sheet;
pub mod css_syntax_component;
pub use style_model::css_value;
pub use style_model::css_value_clamping_utils;
pub use style_model::css_value_keywords;
pub mod element_rule_collector;
pub mod fixed_size_cache;
pub mod font_feature_values_registry;
pub use style_model::hash_tools;
pub mod if_condition;
pub use style_invalidation as invalidation;
pub use style_model::kleene_value;
pub mod media_queries;
pub use style_model::media_value_change;
pub mod parser;
pub use style_model::pending_sheet_type;
mod persistent_invalidation;
pub mod persistent_selector;
pub use style_commit::post_style_update_scope;
pub(crate) use style_parser::production_border_image_values;
pub use style_parser::production_css_value;
pub(crate) mod production_element_rule_collector;
mod production_interaction_features;
mod production_line_features;
pub(crate) use style_parser::production_position_area;
pub(crate) use style_parser::production_reflect_value;
pub(crate) mod production_rule_context;
pub(crate) mod production_rule_matching;
pub mod production_style_sheet;
pub mod production_style_sheet_projection;
mod production_text_box_features;
mod production_typography_features;
pub mod properties;
pub use style_model::property_bitsets;
pub mod resolver;
pub mod responsive_image;
pub mod rule_set;
pub use style_match::selector_checker;
pub use style_match::selector_filter;
pub use style_rules::seeker;
pub mod selector_query;
pub mod style_engine;
pub mod style_environment_variables;
pub use style_walk::style_recalc_change;
pub use style_walk::style_recalc_context;
pub mod style_rule;
pub mod style_rule_counter_style;
pub use style_model::style_rule_css_style_declaration;
pub use style_model::style_rule_font_feature_values;
pub mod style_rule_font_palette_values;
pub mod style_rule_function_declarations;
pub mod style_rule_import;
pub mod style_rule_keyframe;
pub mod style_rule_location;
pub use style_model::style_rule_namespace;
pub mod style_rule_nested_declarations;
pub use style_model::style_rule_view_transition;
pub mod style_sheet_collection;
pub mod style_sheet_contents;
pub use style_model::valid_property_filter;
pub use style_model::vision_deficiency;
pub use style_model::white_space;

pub use color_scheme_flags::{ColorSchemeFlag, ColorSchemeFlags};
pub use css_origin_clean::OriginClean;
pub use css_resource_fetch_restriction::ResourceFetchRestriction;
pub use media_queries::preferred_color_scheme::PreferredColorScheme;
pub use media_value_change::MediaValueChange;
pub use pending_sheet_type::PendingSheetType;
pub use production_style_sheet_projection::{
    CSSDeclarationMutation, CSSInlineDeclarationMutation, GetInlineCSSDeclaration, ParseCSS,
    ParseCSSDeclarationList, ParseCSSWithBase, ParseUserAgentCSS, ParseUserAgentCSSWithRole,
    SetCSSDeclaration, SetInlineCSSDeclaration,
};
pub use style_engine::{
    AllowMarkForReattachFromRebuildLayoutTreeScope, AllowMarkStyleDirtyFromRecalcScope,
    AncestorAnalysis, AttachScrollMarkersScope, DOMRemovalScope, DetachLayoutTreeScope,
    InApplyAnimationUpdateScope, InEnsureComputedStyleScope, InvalidationScope,
    SkipStyleRecalcScope, StyleSheetKey,
};
pub use vision_deficiency::{CreateVisionDeficiencyFilterUrl, VisionDeficiency};

pub mod document_style_engine;
pub use document_style_engine::{
    DocumentStyleEngine as StyleEngine, DocumentStyleError, UserAgentStyleSheetInput,
};
#[cfg(test)]
mod document_style_engine_test;
#[cfg(test)]
mod document_style_retained_pipeline_test;
mod svg_paint_server;
mod svg_presentation_attributes;
#[cfg(test)]
mod svg_presentation_pipeline_test;

pub mod container_query;

pub mod production_container_parser;
pub mod production_container_projection;

pub mod css_syntax_definition;
pub mod css_syntax_string_parser;
pub mod property_registration;
pub mod property_registry;
pub mod style_scope;

pub mod production_rule_effect_projection;

pub mod css_math_expression_node;
pub mod css_math_function_value;

mod production_corner_features;
pub use style_parser::production_corner_value;
pub use style_parser::production_dynamic_range_value;
pub use style_parser::production_effects_value;
pub use style_parser::production_motion_value;
mod production_render_delay_features;
