#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! CSS syntax primitives and parsing algorithms.
//!
//! Parsing consumes source text and produces `style_model` values. It has no
//! DOM, RuleSet index, selector walk, cascade, style commit, or lifecycle
//! ownership. Higher-level grammar modules move here as their model inputs are
//! separated from orchestration.

pub mod allowed_rules;
pub mod at_rule_descriptors;
pub mod at_rule_names;
pub mod css_at_rule_id;
pub use style_model::css_markup;
pub use style_model::css_nesting_type;
pub use style_model::css_property_name;
pub use style_model::css_property_names;
pub use style_model::css_property_value;
pub use style_model::css_property_value_set;
pub use style_model::css_resource_fetch_restriction;
pub use style_model::{css_selector, css_selector_list};
pub mod css_parser;
pub mod css_parser_context;
pub mod css_parser_idioms;
pub mod css_parser_impl;
pub mod css_parser_local_context;
pub use style_model::css_parser_mode;
pub use style_model::css_primitive_value;
pub use style_model::css_syntax_component;
pub use style_model::css_value;
pub use style_model::css_value_clamping_utils;
pub use style_model::css_value_keywords;
pub use style_model::kleene_value;
pub use style_model::properties;
pub use style_model::style_rule;
pub use style_model::style_rule_font_feature_values;
pub use style_model::style_rule_keyframe;
pub use style_model::{
    production_corner_features, production_interaction_features, production_line_features,
    production_render_delay_features, production_text_box_features, production_typography_features,
};
pub mod conditional_exp_node;
pub mod conditional_parser;
pub mod container_query;
pub mod container_query_parser;
pub mod css_parser_token;
pub mod css_parser_token_stream;
pub mod css_property_parser;
pub mod css_property_value_set_parser;
pub mod css_selector_parser;
pub mod css_supports_parser;
pub mod css_syntax_definition;
pub mod css_syntax_string_parser;
pub mod css_tokenizer;
pub mod css_tokenizer_input_stream;
pub mod media_query;
pub mod media_query_parser;
pub mod media_query_set;
pub mod number_parsing_options;
pub mod string_to_number;

pub mod css_math_expression_node;
pub mod css_math_function_value;
pub mod css_numeric_literal_value;
pub mod media_query_exp;
pub mod production_border_image_values;
pub mod production_corner_value;
pub mod production_css_value;
pub mod production_dynamic_range_value;
pub mod production_effects_value;
pub mod production_motion_value;
pub mod production_position_area;
pub mod production_property_metadata;
pub mod production_property_parser;
pub mod production_reflect_value;
