#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Cascade and computed-style resolution.
//!
//! Resolve consumes retained match results and produces canonical native
//! ComputedStyle values. It does not own parser, RuleSet, DOM traversal,
//! invalidation, commit, or document lifecycle state.

pub use style_match::cascade_origin;
pub use style_match::match_result;
pub use style_model::{
    css_primitive_value, css_property_name, css_property_names, css_property_value, css_value,
    css_value_clamping_utils, css_value_keywords, kleene_value, production_corner_features,
    production_interaction_features, production_line_features, production_render_delay_features,
    production_text_box_features, production_typography_features, valid_property_filter,
};
pub use style_parser::{
    css_math_expression_node, css_math_function_value, css_numeric_literal_value,
    css_syntax_definition, production_css_value, production_position_area,
};

/// Compatibility namespaces for translated Chromium files while their
/// package-local imports are normalized. These only point down the dependency
/// graph; they do not reintroduce the facade or document lifecycle.
pub mod media_queries {
    pub use style_model::color_space_gamut;
    pub use style_model::container_state;
    pub use style_model::forced_colors;
    pub use style_model::media_values;
    pub use style_model::media_values_cached_data::MediaValuesCachedData;
    pub use style_model::navigation_controls;
    pub use style_model::preferred_color_scheme;
    pub use style_model::preferred_color_scheme::PreferredColorScheme;
    pub use style_model::preferred_contrast;
    pub use style_model::web_preferences;
    pub use style_parser::conditional_exp_node;
    pub use style_parser::media_query;

    pub mod media_values_cached {
        pub use style_model::media_values_cached_data::*;
    }

    pub mod media_query_exp {
        pub use style_parser::media_query_exp::*;
    }

    pub use style_parser::media_query_set;
    pub mod media_query_evaluator;
}

pub mod parser {
    pub use style_parser::*;
}

pub mod properties {
    pub use style_model::properties::*;

    pub mod longhand_dispatch {
        pub use crate::longhand_dispatch::*;
    }
}

pub mod resolver {
    pub use crate::css_to_style_map;
    pub use crate::position_repeat_application;
    pub use crate::style_builder_converter;
}

mod attribute_variable_resolver;
pub mod cascade_expansion;
pub mod cascade_filter;
pub mod cascade_map;
pub mod cascade_priority;
pub mod cascade_resolver;
pub mod css_to_style_map;
pub mod custom_properties;
pub mod element_resolve_context;
pub mod element_style_resources;
mod environment_variable_resolver;
pub mod filter_operation_resolver;
pub mod font_builder;
pub mod font_style_resolver;
pub mod longhand_dispatch;
pub mod matched_properties_cache;
pub mod position_repeat_application;
pub mod production_style_builder;
pub mod selector_filter_parent_scope;
pub mod style_adjuster;
pub mod style_builder;
pub mod style_builder_converter;
pub mod style_cascade;
pub mod style_resolver;
pub mod style_resolver_state;
pub mod style_resolver_stats;
pub mod viewport_style_resolver;
