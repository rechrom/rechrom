//! Resolved linear-gradient mask values at the native style boundary.
#![allow(non_snake_case)]
use crate::internal::{
    layout_input::{ComputedStyle, TransformMatrix},
    paint_input::{
        BackgroundBox, BackgroundImageLayer, BackgroundRepeat, BackgroundRepeatRule,
        BackgroundSizeMode, PaintColorInterpolationSpace, PaintHueInterpolationMethod,
        PaintMaskComposite, PaintMaskMode, PaintShaderKind, PaintSpreadMethod,
    },
};
use foundation::{Color, Length, LengthSize, MakeGarbageCollected};
use layoutng_style::style::{
    computed_style::ComputedStyleBuilder,
    computed_style_constants::{
        BackgroundEdgeOrigin, CompositingOperator, EFillAttachment, EFillBox, EFillLayerType,
        EFillMaskMode, EFillRepeat, EFillSizeType,
    },
    fill_layer::{FillLayer, FillRepeat, FillSize},
    generated_gradient_image::{
        CSSImageGeneratorValue, CSSLinearGradientValue, GradientStop, LinearGradientData,
        StyleGeneratedImage,
    },
    style_image::StyleImage,
};

#[path = "../../../../layout_fragment/src/native_mask_image_projection.rs"]
mod layout_fragment_native_mask_image_projection;
pub(super) use layout_fragment_native_mask_image_projection::ApplyNativeMaskImages;

fn fill_box(value: BackgroundBox) -> EFillBox {
    match value {
        BackgroundBox::kBorderBox => EFillBox::kBorder,
        BackgroundBox::kPaddingBox => EFillBox::kPadding,
        BackgroundBox::kContentBox => EFillBox::kContent,
        BackgroundBox::kText => EFillBox::kText,
        BackgroundBox::kBorderArea => EFillBox::kBorderArea,
        BackgroundBox::kBorderAreaText => EFillBox::kBorderAreaText,
    }
}
fn repeat(value: BackgroundRepeatRule) -> EFillRepeat {
    match value {
        BackgroundRepeatRule::kRepeat => EFillRepeat::kRepeatFill,
        BackgroundRepeatRule::kNoRepeat => EFillRepeat::kNoRepeatFill,
        BackgroundRepeatRule::kRound => EFillRepeat::kRoundFill,
        BackgroundRepeatRule::kSpace => EFillRepeat::kSpaceFill,
    }
}
fn size_length(
    px: Option<f64>,
    percent: Option<f64>,
    calculation: Option<&std::sync::Arc<foundation::CalculationValue>>,
    zoom: f64,
) -> Length {
    if let Some(calculation) = calculation {
        return Length::from_calculation_value(calculation.Zoom(zoom));
    }
    match (px, percent) {
        (None, None) => Length::Auto().clone(),
        (Some(px), None) => Length::Fixed(px * zoom),
        (None, Some(percent)) => Length::Percent(percent * 100.0),
        (Some(px), Some(percent)) => {
            super::LengthPercentage(px * zoom, percent * 100.0, "mask size")
        }
    }
}
