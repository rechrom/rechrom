//! Compatibility exports for generated gradient style images.
//!
//! CSS gradient values live in the source-aligned `css::css_gradient_value`
//! module. Existing package consumers keep this path while they migrate.
#![allow(non_snake_case)]

pub use crate::css::css_gradient_value::{
    CSSConicGradientValue, CSSGeneratedGradientValue, CSSGradientValue, CSSLinearGradientValue,
    CSSRadialGradientValue, ConicGradientData, GeneratedGradientData, GradientGeneratedImage,
    GradientLength, GradientStop, Image, LinearGradientData, RadialGradientData,
    RadialGradientExtent, RadialGradientShape,
};
pub use crate::css::css_image_generator_value::CSSImageGeneratorValue;
pub use crate::css::css_value::CSSValue;

// Preserve the established public import path while the source-shaped owner
// lives in style/style_generated_image.rs.
pub use super::style_generated_image::{
    ResolvedGradient, ResolvedLinearGradient, StyleGeneratedImage,
};
