//! Resolved linear-gradient mask values at the native style boundary.
#![allow(non_snake_case)]
use crate::internal::{
    layout_input::{ComputedStyle, TransformMatrix},
    paint_input::{
        BackgroundBox, BackgroundImageLayer, BackgroundRepeat, BackgroundRepeatRule,
        BackgroundSizeMode, PaintMaskComposite, PaintMaskMode, PaintShaderKind, PaintSpreadMethod,
    },
};
use foundation::{Color, Length, LengthSize, MakeGarbageCollected};
use layoutng_style::style::{
    computed_style::ComputedStyleBuilder,
    computed_style_constants::{
        CompositingOperator, EFillAttachment, EFillBox, EFillLayerType, EFillMaskMode, EFillRepeat,
        EFillSizeType,
    },
    fill_layer::{FillLayer, FillRepeat, FillSize},
    generated_gradient_image::{
        CSSImageGeneratorValue, CSSLinearGradientValue, GradientStop, LinearGradientData,
        StyleGeneratedImage,
    },
    style_image::StyleImage,
};

fn gradient(layer: &BackgroundImageLayer) -> Option<LinearGradientData> {
    let shader = layer.shader.as_ref()?;
    // These are resolved native CSS linear-gradient values. Other generators
    // require their real subtype/geometry resolver before they can be admitted.
    if layer.resource_id != 0 || shader.kind != PaintShaderKind::kLinearGradient
        || shader.spread != PaintSpreadMethod::kPad || !shader.unit_coordinates
        || shader.object_bounding_box_coordinates || shader.linear_angle.is_some()
        || shader.transform != TransformMatrix::default()
        || !shader.interpolate_premultiplied || shader.stops.len() < 2
        || shader.stops.iter().any(|stop| !stop.offset.is_finite() || stop.offset_length != 0.0
            || stop.offset < 0.0 || stop.offset > 1.0)
        || shader.stops.windows(2).any(|pair| pair[0].offset > pair[1].offset)
        || [shader.start.x, shader.start.y, shader.end.x, shader.end.y].iter().any(|v| !v.is_finite())
        // Diagonal CSS gradients require the source magic-corner resolver.
        || (shader.start.x != shader.end.x && shader.start.y != shader.end.y)
        || shader.start == shader.end
    {
        return None;
    }
    Some(LinearGradientData {
        start: [shader.start.x, shader.start.y],
        end: [shader.end.x, shader.end.y],
        stops: shader
            .stops
            .iter()
            .map(|stop| GradientStop {
                offset: stop.offset,
                color: Color::FromRGBAFloat(
                    stop.color.red,
                    stop.color.green,
                    stop.color.blue,
                    stop.color.alpha,
                ),
            })
            .collect(),
    })
}
fn fill_box(value: BackgroundBox) -> EFillBox {
    match value {
        BackgroundBox::kBorderBox => EFillBox::kBorder,
        BackgroundBox::kPaddingBox => EFillBox::kPadding,
        BackgroundBox::kContentBox => EFillBox::kContent,
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
fn size_length(px: Option<f64>, percent: Option<f64>, zoom: f64) -> Length {
    match (px, percent) {
        (None, None) => Length::Auto().clone(),
        (Some(px), None) => Length::Fixed(px * zoom),
        (None, Some(percent)) => Length::Percent(percent),
        (Some(px), Some(percent)) => super::LengthPercentage(px * zoom, percent, "mask size"),
    }
}

/// Admit a whole supported mask chain, preserving layer order and compositing.
/// Partial chains would lie about the source MaskLayers payload, so fetched,
/// repeating, angular, radial, and mixed unsupported chains remain unconverted.
// cpp: core/css/resolver/style_builder_converter.cc: ConvertStyleImage
// cpp: core/css/properties/longhands/mask_image.cc: ApplyValue
pub(super) fn ApplyNativeMaskImages(builder: &mut ComputedStyleBuilder, input: &ComputedStyle) {
    if input.paint.mask_images.is_empty() {
        return;
    }
    let Some(values) = input
        .paint
        .mask_images
        .iter()
        .map(|mask| gradient(&mask.image))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let zoom = input
        .extended
        .as_ref()
        .map_or(1.0, |style| style.effective_zoom as f64);
    *builder.AccessMaskLayers() = FillLayer::new(EFillLayerType::kMask, true);
    let mut native = builder.AccessMaskLayers() as *mut FillLayer;
    for (index, (mask, data)) in input.paint.mask_images.iter().zip(values).enumerate() {
        if index != 0 {
            native = unsafe { &mut *native }.EnsureNext();
        }
        let image = &mask.image;
        let generator = MakeGarbageCollected(CSSLinearGradientValue::FromResolved(data));
        let style_image = MakeGarbageCollected(StyleGeneratedImage::new(
            generator.cast::<CSSImageGeneratorValue>(),
        ));
        let layer = unsafe { &mut *native };
        layer.SetImage(style_image.cast::<StyleImage>());
        layer.SetAttachment(EFillAttachment::kScroll);
        layer.SetClip(fill_box(image.clip));
        layer.SetOrigin(fill_box(image.origin));
        layer.SetPositionX(&super::PositionLength(
            image.position.x,
            image.position_offset.x * zoom,
            "mask position x",
        ));
        layer.SetPositionY(&super::PositionLength(
            image.position.y,
            image.position_offset.y * zoom,
            "mask position y",
        ));
        let fallback_x = if matches!(
            image.repeat,
            BackgroundRepeat::kRepeat | BackgroundRepeat::kRepeatX
        ) {
            BackgroundRepeatRule::kRepeat
        } else {
            BackgroundRepeatRule::kNoRepeat
        };
        let fallback_y = if matches!(
            image.repeat,
            BackgroundRepeat::kRepeat | BackgroundRepeat::kRepeatY
        ) {
            BackgroundRepeatRule::kRepeat
        } else {
            BackgroundRepeatRule::kNoRepeat
        };
        layer.SetRepeat(&FillRepeat {
            x: repeat(image.repeat_rule_x.unwrap_or(fallback_x)),
            y: repeat(image.repeat_rule_y.unwrap_or(fallback_y)),
        });
        layer.SetMaskMode(match mask.mode {
            PaintMaskMode::kAlpha => EFillMaskMode::kAlpha,
            PaintMaskMode::kLuminance => EFillMaskMode::kLuminance,
        });
        layer.SetCompositingOperator(match mask.composite {
            PaintMaskComposite::kAdd => CompositingOperator::kAdd,
            PaintMaskComposite::kSubtract => CompositingOperator::kSubtract,
            PaintMaskComposite::kIntersect => CompositingOperator::kIntersect,
            PaintMaskComposite::kExclude => CompositingOperator::kExclude,
        });
        let type_ = match image.size_mode {
            BackgroundSizeMode::kContain => EFillSizeType::kContain,
            BackgroundSizeMode::kCover => EFillSizeType::kCover,
            BackgroundSizeMode::kAuto | BackgroundSizeMode::kExplicit => EFillSizeType::kSizeLength,
        };
        let size = LengthSize::new(
            &size_length(image.width, image.width_percentage, zoom),
            &size_length(image.height, image.height_percentage, zoom),
        );
        layer.SetSize(&FillSize::new(type_, &size));
    }
    builder.AdjustMaskLayers();
}
