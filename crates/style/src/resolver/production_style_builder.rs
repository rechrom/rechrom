// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Concrete branches of StyleBuilder/StyleBuilderConverter. Untranslated
//! values remain typed errors; no CSS text participates in property application.
#![allow(non_snake_case)]
use crate::{
    css_primitive_value::UnitType,
    css_value::CSSValuePayload,
    media_queries::MediaValuesCachedData,
    production_css_value::Value,
    properties::{css_property::CSSProperty, longhand_dispatch::*},
};
use font_engine::FontSelectionValue;
use foundation::{CSSPropertyID, CSSValueID, Length, LengthType};
use layoutng_style::{
    css::style_color::StyleColor,
    style::{
        computed_style::{ComputedStyle, ComputedStyleBuilder},
        computed_style_constants::{
            ContentDistributionType, ContentPosition, EAnimPlayState, FlexWrapMode, ItemPosition,
            ItemPositionType, OverflowAlignment,
        },
        computed_style_initial_values::ComputedStyleInitialValues,
        css_timing_data::{
            EaseType, FillMode, LinearEasingPoint, PlaybackDirection, StepPosition, TimingDelay,
            TimingFunction, TransitionBehavior, TransitionProperty,
        },
        style_content_alignment_data::StyleContentAlignmentData,
        style_flex_wrap_data::StyleFlexWrapData,
        style_self_alignment_data::StyleSelfAlignmentData,
    },
};
#[path = "anchor_application.rs"]
mod anchor_application;
#[path = "animation_application.rs"]
mod animation_application;
#[path = "border_application.rs"]
mod border_application;
#[path = "box_geometry_application.rs"]
mod box_geometry_application;
#[path = "column_rule_application.rs"]
mod column_rule_application;
#[path = "corner_application.rs"]
mod corner_application;
#[path = "custom_properties.rs"]
pub mod custom_properties;
#[path = "grid_application.rs"]
mod grid_application;
#[path = "initial_scope_application.rs"]
mod initial_scope_application;
#[path = "layout_misc_application.rs"]
mod layout_misc_application;
#[path = "line_application.rs"]
mod line_application;
#[path = "list_counter_application.rs"]
mod list_counter_application;
#[path = "palette_internal_application.rs"]
mod palette_internal_application;
#[path = "reflection_application.rs"]
mod reflection_application;
#[path = "rule_inset_application.rs"]
mod rule_inset_application;
#[path = "scroll_application.rs"]
mod scroll_application;
#[path = "stable_misc_application.rs"]
mod stable_misc_application;
#[path = "svg_application.rs"]
mod svg_application;
#[path = "svg_presentation_application.rs"]
mod svg_presentation_application;
#[path = "text_application.rs"]
mod text_application;
#[path = "text_box_application.rs"]
mod text_box_application;
#[path = "timeline_application.rs"]
mod timeline_application;
#[path = "transform_application.rs"]
mod transform_application;
#[path = "typography_application.rs"]
mod typography_application;
#[path = "viewport_application.rs"]
mod viewport_application;
#[path = "writing_direction_application.rs"]
mod writing_direction_application;

#[path = "font_core_application.rs"]
mod font_core_application;

#[path = "font_variant_application.rs"]
mod font_variant_application;

#[path = "border_image_application.rs"]
mod border_image_application;
#[path = "color_application.rs"]
mod color_application;
#[path = "color_ui_application.rs"]
mod color_ui_application;
#[path = "content_application.rs"]
mod content_application;
#[path = "effects_application.rs"]
mod effects_application;
#[path = "grid_lanes_application.rs"]
mod grid_lanes_application;
#[path = "interaction_application.rs"]
mod interaction_application;
#[path = "motion_application.rs"]
mod motion_application;
#[path = "render_delay_application.rs"]
mod render_delay_application;
#[path = "timeline_trigger_application.rs"]
mod timeline_trigger_application;
#[path = "view_transition_application.rs"]
mod view_transition_application;
pub use color_ui_application::ColorSchemeSettings;

pub fn ApplyWithColorSchemeSettings(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    settings: ColorSchemeSettings,
) -> Result {
    if color_ui_application::IsColorUIProperty(id) {
        return color_ui_application::Apply(id, b, parent, v, media, settings);
    }
    Apply(id, b, parent, v, root, media)
}

type Result = std::result::Result<(), LonghandApplicationError>;

/// A fetched image keeps the resource owner's existing identity and its real
/// native StyleImage. Only the resource owner may supply this binding.
pub struct FetchedImageBinding {
    resource_id: std::num::NonZeroU64,
    image: std::ptr::NonNull<layoutng_style::style::style_image::StyleImage>,
}
impl FetchedImageBinding {
    /// The resource owner supplies a live, GC-traced fetched subtype. This
    /// checked binding cannot turn a generated or pending object into a fetch.
    ///
    /// # Safety
    /// `image` must point to a live GC allocation whose native subtype and
    /// resource owner retain the supplied resource identity.
    pub unsafe fn FromResource(
        resource_id: std::num::NonZeroU64,
        image: std::ptr::NonNull<layoutng_style::style::style_image::StyleImage>,
    ) -> std::result::Result<Self, LonghandApplicationError> {
        if !unsafe { image.as_ref() }.IsImageResource() {
            return Err(LonghandApplicationError::InvalidValue(
                CSSPropertyID::kBackgroundImage,
            ));
        }
        Ok(Self { resource_id, image })
    }
    pub fn ResourceId(&self) -> std::num::NonZeroU64 {
        self.resource_id
    }
    pub fn Image(&self) -> std::ptr::NonNull<layoutng_style::style::style_image::StyleImage> {
        self.image
    }
}
pub trait URLImageResolver {
    fn ResolveImage(
        &self,
        value: &crate::production_css_value::CSSImageValue,
    ) -> std::result::Result<FetchedImageBinding, LonghandApplicationError>;
}
// css_to_style_map.cc:119-131, StyleResolverState::GetStyleImage. The calling
// document/resource service installs genuine fetched objects through this
// interface; absent bindings remain typed Unsupported, never a null URL image.
pub fn ApplyWithImageResolver(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    images: &dyn URLImageResolver,
) -> Result {
    if id == CSSPropertyID::kWebkitBoxReflect {
        return reflection_application::Apply(b, parent, v, root, media, Some(images));
    }
    if border_image_application::IsBorderImageProperty(id) {
        return border_image_application::Apply(id, b, parent, v, root, media, Some(images));
    }
    if matches!(
        id,
        CSSPropertyID::kBackgroundImage | CSSPropertyID::kMaskImage
    ) {
        return ApplyLayerImages(id, b, parent, v, Some(images));
    }
    if id == CSSPropertyID::kListStyleImage {
        return list_counter_application::Apply(id, b, parent, v, root, media, Some(images));
    }
    if id == CSSPropertyID::kContent {
        return content_application::Apply(b, parent, v, Some(images));
    }
    Apply(id, b, parent, v, root, media)
}

// cpp: css_gradient_value.cc:698-789,1282-1324,1371-1465; AddStops,
// EndPointsFromAngle axis branches, CSSLinearGradientValue::CreateGradient.
// The current native generated-image subtype stores resolved axis endpoints
// and percentage stops; target-dependent geometry stays an explicit dependency.
fn ResolveLinearGradient(
    value: &crate::production_css_value::CSSLinearGradientValue,
) -> std::result::Result<
    layoutng_style::style::generated_gradient_image::LinearGradientData,
    LonghandApplicationError,
> {
    use layoutng_style::style::generated_gradient_image::{GradientStop, LinearGradientData};
    let unsupported = || LonghandApplicationError::Unsupported(CSSPropertyID::kBackgroundImage);
    if value.repeating || value.stops.len() < 2 || value.end_x.is_some() && value.end_y.is_some() {
        return Err(unsupported());
    }
    let degrees = if value.angle.is_some() {
        (value.Degrees().ok_or_else(unsupported)? as f32).rem_euclid(360.0)
    } else {
        match value.end_x.or(value.end_y) {
            Some(CSSValueID::kTop) => 0.0,
            Some(CSSValueID::kRight) => 90.0,
            Some(CSSValueID::kLeft) => 270.0,
            _ => 180.0,
        }
    };
    let (start, end) = match degrees {
        0.0 => ([0.0, 1.0], [0.0, 0.0]),
        90.0 => ([0.0, 0.0], [1.0, 0.0]),
        180.0 => ([0.0, 0.0], [0.0, 1.0]),
        270.0 => ([1.0, 0.0], [0.0, 0.0]),
        _ => return Err(unsupported()),
    };
    let mut offsets = Vec::new();
    let mut colors = Vec::new();
    let mut previous = 0.0f32;
    for (i, stop) in value.stops.iter().enumerate() {
        let color = match stop.color.as_deref().map(Value::Payload) {
            Some(CSSValuePayload::kColorClass(color)) => color.0,
            Some(CSSValuePayload::kIdentifierClass(color)) => {
                crate::production_css_value::NamedColor(color.0).ok_or_else(unsupported)?
            }
            _ => return Err(unsupported()),
        };
        colors.push(color);
        let offset = if let Some(offset) = &stop.offset {
            let CSSValuePayload::kNumericLiteralClass(offset) = offset.Payload() else {
                return Err(unsupported());
            };
            let offset = match offset.GetType() {
                UnitType::kPercentage => offset.DoubleValue() as f32 / 100.0,
                _ if offset.DoubleValue() == 0.0 => 0.0,
                _ => return Err(unsupported()),
            };
            if !(0.0..=1.0).contains(&offset) {
                return Err(unsupported());
            }
            Some(offset.max(previous))
        } else if i == 0 {
            Some(0.0)
        } else if i + 1 == value.stops.len() {
            Some(1.0f32.max(previous))
        } else {
            None
        };
        if let Some(offset) = offset {
            previous = offset;
        }
        offsets.push(offset);
    }
    let mut left = 0;
    while left + 1 < offsets.len() {
        let right = (left + 1..offsets.len())
            .find(|&i| offsets[i].is_some())
            .unwrap();
        let a = offsets[left].unwrap();
        let delta = (offsets[right].unwrap() - a) / (right - left) as f32;
        for i in left + 1..right {
            offsets[i] = Some(a + (i - left) as f32 * delta);
        }
        left = right;
    }
    Ok(LinearGradientData {
        start,
        end,
        stops: offsets
            .into_iter()
            .zip(colors)
            .map(|(offset, color)| GradientStop {
                offset: offset.unwrap() as f64,
                color,
            })
            .collect(),
    })
}

// cpp: css_to_style_map.cc:119-131. Shared typed image boundary for native fields.
fn ResolveStyleImage(
    id: CSSPropertyID,
    value: &Value,
    images: Option<&dyn URLImageResolver>,
) -> std::result::Result<
    *mut layoutng_style::style::style_image::StyleImage,
    LonghandApplicationError,
> {
    use layoutng_style::style::{
        generated_gradient_image::{
            CSSImageGeneratorValue, CSSLinearGradientValue, StyleGeneratedImage,
        },
        style_image::StyleImage,
    };
    let image = match value.Payload() {
        CSSValuePayload::kIdentifierClass(keyword) if keyword.0 == CSSValueID::kNone => {
            std::ptr::null_mut()
        }
        CSSValuePayload::kImageClass(url) => images
            .ok_or(LonghandApplicationError::Unsupported(id))?
            .ResolveImage(url)
            .map_err(|error| match error {
                LonghandApplicationError::Unsupported(_) => {
                    LonghandApplicationError::Unsupported(id)
                }
                LonghandApplicationError::InvalidValue(_) => {
                    LonghandApplicationError::InvalidValue(id)
                }
            })?
            .image
            .as_ptr(),
        CSSValuePayload::kLinearGradientClass(gradient) => {
            let data = ResolveLinearGradient(gradient)
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            let generator =
                foundation::MakeGarbageCollected(CSSLinearGradientValue::FromResolved(data));
            foundation::MakeGarbageCollected(StyleGeneratedImage::new(
                generator.cast::<CSSImageGeneratorValue>(),
            ))
            .cast::<StyleImage>()
        }
        _ => {
            return Err(LonghandApplicationError::Unsupported(id));
        }
    };
    Ok(image)
}

// cpp: generated longhands.cc:3286-3340,10761-10810;
// shared BackgroundImage/MaskImage FillLayer::SetImage/ClearImage loops.
fn ApplyLayerImages(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    images: Option<&dyn URLImageResolver>,
) -> Result {
    use layoutng_style::style::fill_layer::FillLayer;
    let mask = id == CSSPropertyID::kMaskImage;
    let inherit = v.IsInheritedValue();
    let initial = v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none();
    let mut values = Vec::new();
    if initial {
        values.push(std::ptr::null_mut());
    } else if inherit {
        let mut layer = if mask {
            parent.unwrap().MaskLayers()
        } else {
            parent.unwrap().BackgroundLayers()
        } as *const FillLayer;
        while let Some(current) = unsafe { layer.as_ref() } {
            if !current.IsImageSet() {
                break;
            }
            values.push(current.GetImage());
            layer = current.Next();
        }
    } else {
        if let CSSValuePayload::kValueListClass(list) = v.Payload() {
            if list.separator != crate::production_css_value::ListSeparator::Comma
                || list.values.is_empty()
            {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            for value in &list.values {
                values.push(ResolveStyleImage(id, value, images)?);
            }
        } else {
            values.push(ResolveStyleImage(id, v, images)?);
        }
    }
    let mut current = if mask {
        b.AccessMaskLayers()
    } else {
        b.AccessBackgroundLayers()
    } as *mut FillLayer;
    let mut previous: *mut FillLayer = std::ptr::null_mut();
    for image in values {
        if current.is_null() {
            current = unsafe { (*previous).EnsureNext() };
        }
        unsafe {
            (*current).SetImage(image);
        }
        previous = current;
        current = unsafe { (*current).NextMut() };
    }
    while let Some(layer) = unsafe { current.as_mut() } {
        layer.ClearImage();
        current = layer.NextMut();
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

// cpp: css_primitive_value.cc:ComputeLengthDouble; css_numeric_literal_value.cc:ComputeLengthDouble.
// Font metric-dependent units are intentionally not approximated with media-query metrics.
pub fn Pixels(
    id: CSSPropertyID,
    value: f64,
    unit: UnitType,
    font: f32,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    let factor = match unit {
        UnitType::kPixels => 1.0,
        UnitType::kEms | UnitType::kQuirkyEms => font as f64,
        UnitType::kRems => root as f64,
        UnitType::kCentimeters => 96.0 / 2.54,
        UnitType::kMillimeters => 96.0 / 25.4,
        UnitType::kQuarterMillimeters => 96.0 / 101.6,
        UnitType::kInches => 96.0,
        UnitType::kPoints => 96.0 / 72.0,
        UnitType::kPicas => 16.0,
        UnitType::kViewportWidth => media.large_viewport_width / 100.0,
        UnitType::kViewportHeight => media.large_viewport_height / 100.0,
        UnitType::kViewportMin => {
            media.large_viewport_width.min(media.large_viewport_height) / 100.0
        }
        UnitType::kViewportMax => {
            media.large_viewport_width.max(media.large_viewport_height) / 100.0
        }
        UnitType::kSmallViewportWidth => media.small_viewport_width / 100.0,
        UnitType::kSmallViewportHeight => media.small_viewport_height / 100.0,
        UnitType::kLargeViewportWidth => media.large_viewport_width / 100.0,
        UnitType::kLargeViewportHeight => media.large_viewport_height / 100.0,
        UnitType::kDynamicViewportWidth => media.dynamic_viewport_width / 100.0,
        UnitType::kDynamicViewportHeight => media.dynamic_viewport_height / 100.0,
        UnitType::kNumber if value == 0.0 => 1.0,
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    };
    Ok(value * factor)
}

// cpp: style_builder_converter.cc:2695-2714,2732-2748.
// This is the document StyleResolverState branch; offscreen-canvas defaults
// are a separate converter caller and are not used by property application.
fn ConvertShadow(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    value: &crate::production_css_value::CSSShadowValue,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<layoutng_style::style::shadow_data::ShadowData, LonghandApplicationError> {
    use layoutng_style::style::shadow_data::{ShadowData, ShadowStyle};
    let length = |value: &Value| {
        if let CSSValuePayload::kMathFunctionClass(math) = value.Payload() {
            if math.Category() != crate::css_math_expression_node::CalculationResultCategory::Length
            {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            let pixels = math
                .ComputeValue(
                    &mut MathLengthResolver(
                        id,
                        b.GetFontDescription().ComputedSize(),
                        root,
                        b.EffectiveZoom(),
                        media,
                    ),
                    None,
                )
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            return Ok(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels)
                    .clamp(-(f32::MAX as f64), f32::MAX as f64) as f32,
            );
        }
        let CSSValuePayload::kNumericLiteralClass(number) = value.Payload() else {
            return Err(LonghandApplicationError::Unsupported(id));
        };
        if b.EffectiveZoom() != 1.0 {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        let pixels = Pixels(
            id,
            number.DoubleValue(),
            number.GetType(),
            b.GetFontDescription().ComputedSize(),
            root,
            media,
        )?;
        // css_primitive_value.cc:355-360 ComputeLength<float>: ClampLength,
        // then ClampTo<float>; keep extreme literals finite in native storage.
        let pixels = crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels);
        Ok(pixels.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32)
    };
    if id == CSSPropertyID::kTextShadow && (value.spread.is_some() || value.style.is_some()) {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let offset = foundation::gfx::Vector2dF::new(length(&value.x)?, length(&value.y)?);
    let blur = value
        .blur
        .as_deref()
        .map(length)
        .transpose()?
        .unwrap_or(0.0);
    if blur < 0.0 {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let spread = value
        .spread
        .as_deref()
        .map(length)
        .transpose()?
        .unwrap_or(0.0);
    let style = match value.style.as_deref().map(Value::Payload) {
        None => ShadowStyle::kNormal,
        Some(CSSValuePayload::kIdentifierClass(keyword)) if keyword.0 == CSSValueID::kInset => {
            ShadowStyle::kInset
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    let color = match value.color.as_deref().map(Value::Payload) {
        None => StyleColor::CurrentColor(),
        Some(CSSValuePayload::kColorClass(color)) => StyleColor::from_color(color.0),
        Some(CSSValuePayload::kIdentifierClass(keyword))
            if keyword.0 == CSSValueID::kCurrentcolor =>
        {
            StyleColor::CurrentColor()
        }
        Some(CSSValuePayload::kIdentifierClass(keyword)) => {
            let color = crate::production_css_value::NamedColor(keyword.0)
                .ok_or(LonghandApplicationError::Unsupported(id))?;
            StyleColor::from_color(color)
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    };
    Ok(ShadowData::new(offset, blur, spread, style, color, 1.0))
}

// cpp: generated longhands.cc:5556-5569,16772-16785.
fn ApplyShadow(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    inherit: bool,
    initial: bool,
) -> Result {
    use foundation::{MakeGarbageCollected, Member};
    use layoutng_style::style::shadow_list::{ShadowDataVector, ShadowList};
    let shadow = if initial {
        Member::default()
    } else if inherit {
        let p = parent.unwrap();
        if p.EffectiveZoom() != b.EffectiveZoom() {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        Member::from_ptr(if id == CSSPropertyID::kBoxShadow {
            p.BoxShadow()
        } else {
            p.TextShadow()
        })
    } else if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(keyword) if keyword.0 == CSSValueID::kNone)
    {
        Member::default()
    } else {
        let CSSValuePayload::kValueListClass(list) = v.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        if list.separator != crate::production_css_value::ListSeparator::Comma
            || list.values.is_empty()
        {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        let shadows = list
            .values
            .iter()
            .map(|value| {
                let CSSValuePayload::kShadowClass(shadow) = value.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                ConvertShadow(id, b, shadow, root, media)
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Member::from_ptr(MakeGarbageCollected(ShadowList::new(
            ShadowDataVector::from(shadows),
        )))
    };
    if id == CSSPropertyID::kBoxShadow {
        b.SetBoxShadow(shadow);
    } else {
        b.SetTextShadow(shadow);
    }
    if inherit && !initial && v.IsInheritedValue() && !CSSProperty::Get(id).IsInherited() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

fn IsOverflowOutlineProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kOverflowX
            | CSSPropertyID::kOverflowY
            | CSSPropertyID::kZIndex
            | CSSPropertyID::kOutlineColor
            | CSSPropertyID::kOutlineStyle
            | CSSPropertyID::kOutlineWidth
            | CSSPropertyID::kOutlineOffset
    )
}

// cpp: generated longhands.cc:6821-6829,6850-6858;
// style_builder_converter.cc:2287-2294,3823-3837;
// css_identifier_value_mappings.h:1670-1685.
fn ApplyContainer(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    inherit: bool,
    initial: bool,
) -> Result {
    use foundation::{HeapVector, MakeGarbageCollected, Member, ScopedCSSName, ScopedCSSNameList};
    use layoutng_style::style::computed_style_constants::EContainerType;
    if id == CSSPropertyID::kContainerName {
        if initial {
            b.SetContainerNameOwned(Member::default());
        } else if inherit {
            b.SetContainerName(parent.unwrap().ContainerName());
        } else if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(value) if value.0 == CSSValueID::kNone)
        {
            b.SetContainerNameOwned(Member::default());
        } else {
            let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            if list.separator != crate::production_css_value::ListSeparator::Space
                || list.values.is_empty()
            {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            // Validate before allocating or changing the builder.
            let names = list
                .values
                .iter()
                .map(|value| {
                    let CSSValuePayload::kCustomIdentClass(name) = value.Payload() else {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    };
                    if name.property != CSSPropertyID::kInvalid {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                    Ok(&name.name)
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            // The current production cascade operates on document-root names.
            // Native null scope is that domain; shadow TreeScope population and
            // StyleResolverState::SetHasTreeScopedReference remain collaborators.
            let names: HeapVector<Member<ScopedCSSName>> = names
                .into_iter()
                .map(|name| {
                    Member::from_ptr(MakeGarbageCollected(ScopedCSSName::new(
                        name,
                        std::ptr::null(),
                    )))
                })
                .collect();
            b.SetContainerNameOwned(Member::from_ptr(MakeGarbageCollected(
                ScopedCSSNameList::new(names),
            )));
        }
    } else {
        let flags = if initial {
            ComputedStyleInitialValues::InitialContainerType()
        } else if inherit {
            parent.unwrap().ContainerType()
        } else {
            let convert = |value: &Value| {
                let CSSValuePayload::kIdentifierClass(keyword) = value.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                Ok(match keyword.0 {
                    CSSValueID::kNormal => EContainerType::kContainerTypeNormal,
                    CSSValueID::kSize => EContainerType::kContainerTypeSize,
                    CSSValueID::kInlineSize => EContainerType::kContainerTypeInlineSize,
                    CSSValueID::kScrollState => EContainerType::kContainerTypeScrollState,
                    CSSValueID::kAnchored => EContainerType::kContainerTypeAnchored,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
                .value() as u32)
            };
            match v.Payload() {
                CSSValuePayload::kValueListClass(list) => {
                    if list.separator != crate::production_css_value::ListSeparator::Space
                        || list.values.is_empty()
                    {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                    let mut flags = 0;
                    for item in &list.values {
                        flags |= convert(item)?;
                    }
                    flags
                }
                _ => convert(v)?,
            }
        };
        b.SetContainerType(flags);
    }
    if inherit && !initial && v.IsInheritedValue() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

// cpp: longhands_custom.cc:7565-7603,7738-7767,7777-7806;
// generated longhands.cc:12089-12099,12120-12133,12174-12176,19755-19771.
fn ApplyOverflowOutline(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    inherit: bool,
    initial: bool,
) -> Result {
    use foundation::{EBorderStyle, EOverflow, LayoutUnit};
    use CSSPropertyID::*;
    use CSSValueID::*;
    let identifier = || match v.Payload() {
        CSSValuePayload::kIdentifierClass(value) => Ok(value.0),
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    };
    match id {
        kOverflowX | kOverflowY => {
            let x = id == kOverflowX;
            let overflow = if initial {
                if x {
                    ComputedStyleInitialValues::InitialOverflowX()
                } else {
                    ComputedStyleInitialValues::InitialOverflowY()
                }
            } else if inherit {
                let p = parent.unwrap();
                if x {
                    p.OverflowX()
                } else {
                    p.OverflowY()
                }
            } else {
                match identifier()? {
                    kVisible => EOverflow::kVisible,
                    kHidden => EOverflow::kHidden,
                    kScroll => EOverflow::kScroll,
                    kAuto => EOverflow::kAuto,
                    CSSValueID::kOverlay => EOverflow::kOverlay,
                    CSSValueID::kClip => EOverflow::kClip,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            if x {
                b.SetOverflowX(overflow);
                if overflow == EOverflow::kVisible {
                    b.SetHasExplicitOverflowXVisible();
                }
            } else {
                b.SetOverflowY(overflow);
                if overflow == EOverflow::kVisible {
                    b.SetHasExplicitOverflowYVisible();
                }
            }
        }
        kZIndex => {
            if initial
                || inherit && parent.unwrap().HasAutoZIndex()
                || !inherit
                    && matches!(v.Payload(), CSSValuePayload::kIdentifierClass(value) if value.0 == kAuto)
            {
                b.SetHasAutoZIndex();
            } else if inherit {
                b.SetZIndex(parent.unwrap().ZIndex());
            } else {
                let CSSValuePayload::kNumericLiteralClass(number) = v.Payload() else {
                    return Err(LonghandApplicationError::Unsupported(id));
                };
                if number.GetType() != UnitType::kInteger {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                // css_numeric_literal_value.cc:137-140 ComputeInteger uses
                // ClampTo<int>(num_); Rust's float cast saturates the same range.
                b.SetZIndex(number.DoubleValue() as i32);
            }
        }
        kOutlineStyle => {
            let (style, auto) = if initial {
                (
                    EBorderStyle::kNone,
                    ComputedStyleInitialValues::InitialOutlineStyleIsAuto(),
                )
            } else if inherit {
                let p = parent.unwrap();
                (p.OutlineStyle(), p.OutlineStyleIsAuto())
            } else {
                let keyword = identifier()?;
                // css_identifier_value_mappings.h:96-110: auto stores dotted
                // plus its independent OutlineIsAuto bit.
                let style = match keyword {
                    kNone => EBorderStyle::kNone,
                    kAuto | kDotted => EBorderStyle::kDotted,
                    kDashed => EBorderStyle::kDashed,
                    kSolid => EBorderStyle::kSolid,
                    kDouble => EBorderStyle::kDouble,
                    kGroove => EBorderStyle::kGroove,
                    kRidge => EBorderStyle::kRidge,
                    CSSValueID::kInset => EBorderStyle::kInset,
                    kOutset => EBorderStyle::kOutset,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                };
                (style, keyword == kAuto)
            };
            b.SetOutlineStyleIsAuto(auto);
            b.SetOutlineStyle(style);
        }
        kOutlineColor => {
            let color = if initial {
                StyleColor::CurrentColor()
            } else if inherit {
                parent.unwrap().OutlineColor().clone()
            } else {
                color_ui_application::ConvertStyleColorValue(id, v)?
            };
            b.SetOutlineColor(&color);
        }
        kOutlineWidth | kOutlineOffset => {
            let width = id == kOutlineWidth;
            let length = if initial {
                if width {
                    // OutlineWidth::ApplyInitial uses ZoomedComputedPixels.
                    (ComputedStyleInitialValues::InitialOutlineWidth() as f32 * b.EffectiveZoom())
                        as i32
                } else {
                    ComputedStyleInitialValues::InitialOutlineOffset()
                }
            } else if inherit {
                let p = parent.unwrap();
                // ApplyParentValueIfZoomChanged requires the inherited length
                // conversion data adapter, not a ratio guessed from storage.
                if b.EffectiveZoom() != p.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                if width {
                    *p.OutlineWidth()
                } else {
                    *p.OutlineOffset()
                }
            } else {
                if b.EffectiveZoom() != 1.0 {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                let pixels = match v.Payload() {
                    CSSValuePayload::kIdentifierClass(_) if width => match identifier()? {
                        kThin => 1.0,
                        kMedium => 3.0,
                        kThick => 5.0,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    },
                    CSSValuePayload::kNumericLiteralClass(number) => Pixels(
                        id,
                        number.DoubleValue(),
                        number.GetType(),
                        b.GetFontDescription().ComputedSize(),
                        root,
                        media,
                    )?,
                    _ => return Err(LonghandApplicationError::Unsupported(id)),
                };
                if width {
                    if pixels < 0.0 {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                    // ConvertBorderWidth computes float before ClampLineWidth.
                    crate::resolver::style_builder_converter::StyleBuilderConverter::ClampLineWidth(
                        pixels as f32 as f64,
                    )
                } else {
                    // style_builder_converter.cc:1992-2003 ConvertOutlineOffset.
                    let absolute = pixels.abs();
                    if absolute > 0.0 && absolute < 1.0 {
                        if pixels > 0.0 {
                            1
                        } else {
                            -1
                        }
                    } else {
                        let integral = absolute
                            .floor()
                            .clamp(0.0, LayoutUnit::Max().ToInt() as f64)
                            as i32;
                        if pixels < 0.0 {
                            -integral
                        } else {
                            integral
                        }
                    }
                }
            };
            if width {
                b.SetOutlineWidthOwned(length);
            } else {
                b.SetOutlineOffsetOwned(length);
            }
        }
        _ => unreachable!(),
    }
    if inherit && !initial && v.IsInheritedValue() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

// cpp: CSSLengthResolver::ZoomedComputedPixels and CSSPrimitiveValue
// ComputeLengthDouble. Existing literal conversion supplies actual font/root
// and viewport metrics; absolute and viewport units use the target zoom.
pub(super) fn MathLengthResolver<'a>(
    id: CSSPropertyID,
    font: f32,
    root: f32,
    zoom: f32,
    media: &'a MediaValuesCachedData,
) -> impl crate::css_math_expression_node::CSSMathLengthResolver + 'a {
    move |value, unit| {
        let pixels = Pixels(id, value, unit, font, root, media)
            .map_err(|_| crate::css_math_expression_node::MathError::MissingLengthContext)?;
        Ok(
            if matches!(
                unit,
                UnitType::kEms | UnitType::kQuirkyEms | UnitType::kRems
            ) {
                pixels
            } else {
                pixels * zoom as f64
            },
        )
    }
}
fn ResolveNumericMathList(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<Option<std::rc::Rc<Value>>, LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kMathFunctionClass(m) => {
            if matches!(
                m.Category(),
                crate::css_math_expression_node::CalculationResultCategory::Length
                    | crate::css_math_expression_node::CalculationResultCategory::LengthFunction
            ) {
                return Err(LonghandApplicationError::Unsupported(id));
            }
            let value = m
                .ComputeValue(
                    &mut |_, _| {
                        Err(crate::css_math_expression_node::MathError::MissingLengthContext)
                    },
                    None,
                )
                .map_err(|_| LonghandApplicationError::InvalidValue(id))?;
            Ok(Some(crate::production_css_value::numeric(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampDouble(value),
                m.expression
                    .CanonicalUnit()
                    .ok_or(LonghandApplicationError::InvalidValue(id))?,
            )))
        }
        CSSValuePayload::kValueListClass(list) => {
            let mut changed = false;
            let mut values = Vec::new();
            for child in &list.values {
                if let Some(value) = ResolveNumericMathList(id, child)? {
                    changed = true;
                    values.push(value)
                } else {
                    values.push(child.clone())
                }
            }
            Ok(changed.then(|| crate::production_css_value::list(values, list.separator)))
        }
        _ => Ok(None),
    }
}
// cpp: style_builder.cc:ApplyPhysicalProperty, generated longhands.cc Apply*.
pub fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    if !crate::production_corner_features::IsExposed(id) {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    if !crate::production_render_delay_features::IsExposed(id) {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    if super::position_repeat_application::IsPositionRepeatProperty(id) {
        return super::position_repeat_application::Apply(id, b, parent, v, root, media);
    }
    if corner_application::IsProperty(id) {
        return corner_application::Apply(id, b, parent, v, root, media);
    }
    if initial_scope_application::IsProperty(id) {
        return initial_scope_application::Apply(id, b, parent, v);
    }
    if id == CSSPropertyID::kGridLanesDirection {
        return grid_lanes_application::ApplyInternal(b, parent, v);
    }
    if render_delay_application::IsProperty(id) {
        return render_delay_application::ApplyInternal(id, b, parent, v, root, media);
    }
    if timeline_trigger_application::IsProperty(id) {
        return timeline_trigger_application::Apply(id, b, parent, v, root, media);
    }
    if view_transition_application::IsProperty(id) {
        return view_transition_application::Apply(id, b, parent, v);
    }
    if motion_application::IsProperty(id) {
        return motion_application::Apply(id, b, parent, v, root, media);
    }
    if border_image_application::IsBorderImageProperty(id) {
        return border_image_application::Apply(id, b, parent, v, root, media, None);
    }
    if interaction_application::IsInteractionProperty(id) {
        return interaction_application::Apply(id, b, parent, v);
    }
    if color_application::IsProperty(id) {
        return color_application::Apply(id, b, parent, v);
    }
    if color_ui_application::IsColorUIProperty(id) {
        return color_ui_application::Apply(
            id,
            b,
            parent,
            v,
            media,
            ColorSchemeSettings::default(),
        );
    }
    if effects_application::IsEffectsProperty(id) {
        return effects_application::Apply(id, b, parent, v, root, media);
    }
    if matches!(
        id,
        CSSPropertyID::kFontPalette
            | CSSPropertyID::kInternalAlignContentBlock
            | CSSPropertyID::kInternalEmptyLineHeight
    ) {
        return palette_internal_application::Apply(id, b, parent, v);
    }
    if id == CSSPropertyID::kWebkitBoxReflect {
        return reflection_application::Apply(b, parent, v, root, media, None);
    }
    if stable_misc_application::IsProperty(id) {
        return stable_misc_application::Apply(id, b, parent, v, root, media);
    }
    if column_rule_application::IsProperty(id) {
        return column_rule_application::Apply(id, b, parent, v, root, media);
    }
    if rule_inset_application::IsProperty(id) {
        return rule_inset_application::Apply(id, b, parent, v, root, media);
    }
    if text_box_application::IsProperty(id) {
        return text_box_application::Apply(id, b, parent, v, root, media);
    }
    if typography_application::IsTypographyProperty(id)
        || id == CSSPropertyID::kTextEmphasisColor
            && (v.IsInitialValue() || v.IsInheritedValue() || v.IsUnsetValue())
    {
        return typography_application::Apply(id, b, parent, v, root, media);
    }
    if line_application::IsLineProperty(id) {
        return line_application::Apply(id, b, parent, v, root, media);
    }
    if anchor_application::IsAnchorProperty(id) {
        return anchor_application::Apply(id, b, parent, v);
    }
    if id == CSSPropertyID::kLineHeight {
        return font_variant_application::ApplyLineHeight(b, parent, v, root, media);
    }
    if scroll_application::IsScrollProperty(id) {
        return scroll_application::Apply(id, b, parent, v, root, media);
    }
    if list_counter_application::IsListCounterProperty(id) {
        return list_counter_application::Apply(id, b, parent, v, root, media, None);
    }
    if layout_misc_application::IsLayoutProperty(id) {
        return layout_misc_application::Apply(id, b, parent, v, root, media);
    }
    if box_geometry_application::IsProperty(id) {
        return box_geometry_application::Apply(id, b, parent, v, root, media);
    }
    if matches!(id, CSSPropertyID::kDirection | CSSPropertyID::kWritingMode) {
        return writing_direction_application::Apply(id, b, parent, v);
    }
    if text_application::IsTextProperty(id) {
        return text_application::Apply(id, b, parent, v, root, media);
    }
    if transform_application::IsTransformProperty(id) {
        return transform_application::Apply(id, b, parent, v, root, media);
    }
    if timeline_application::IsProperty(id) {
        return timeline_application::Apply(id, b, parent, v, root, media);
    }
    if viewport_application::IsProperty(id) {
        return viewport_application::Apply(id, b, parent, v, root, media);
    }
    if svg_presentation_application::IsProperty(id) {
        return svg_presentation_application::Apply(id, b, parent, v, root, media);
    }
    if svg_application::IsSVGProperty(id) {
        return svg_application::Apply(id, b, parent, v, root, media);
    }
    if border_application::IsBorderProperty(id) {
        return border_application::Apply(id, b, parent, v, root, media);
    }
    if matches!(
        id,
        CSSPropertyID::kBackgroundImage | CSSPropertyID::kMaskImage
    ) {
        return ApplyLayerImages(id, b, parent, v, None);
    }
    // Font longhands own their conversion because spacing and size changes
    // must update the staged FontDescription without constructing a platform
    // Font before the layout host installs its resolver. In particular, do
    // not let the generic math fast path call ComputedStyleBuilder's direct
    // SetWordSpacing/SetLetterSpacing helpers.
    if font_core_application::IsProperty(id) {
        let inherit =
            v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
        let initial = v.IsInitialValue()
            || v.IsUnsetValue() && !CSSProperty::Get(id).IsInherited()
            || inherit && parent.is_none();
        if inherit && !initial && v.IsInheritedValue() {
            b.SetHasExplicitInheritance();
            parent.unwrap().SetChildHasExplicitInheritance();
        }
        return font_core_application::Apply(id, b, parent, v, root, media, inherit, initial);
    }
    if let CSSValuePayload::kMathFunctionClass(math) = v.Payload() {
        use crate::css_math_expression_node::CalculationResultCategory as C;
        let font = if id == CSSPropertyID::kFontSize {
            parent.map_or(media.em_size, |p| p.GetFontDescription().SpecifiedSize())
        } else {
            b.GetFontDescription().ComputedSize()
        };
        let mut resolver = MathLengthResolver(
            id,
            font,
            root,
            if id == CSSPropertyID::kFontSize {
                1.0
            } else {
                b.EffectiveZoom()
            },
            media,
        );
        if id == CSSPropertyID::kFontSize && math.Category() == C::LengthFunction {
            let number = math
                .ComputeValue(&mut resolver, Some(font as f64))
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            return Apply(
                id,
                b,
                parent,
                &crate::production_css_value::numeric(number, UnitType::kPixels),
                root,
                media,
            );
        }
        if math.Category() == C::LengthFunction {
            let length = math
                .ConvertToLength(&mut resolver)
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            return ApplyConvertedLength(id, b, &length);
        }
        let mut number = math
            .ComputeValue(&mut resolver, None)
            .map_err(|_| LonghandApplicationError::Unsupported(id))?;
        let mut unit = math
            .expression
            .CanonicalUnit()
            .ok_or(LonghandApplicationError::InvalidValue(id))?;
        if math.Category() == C::Percent
            && matches!(
                id,
                CSSPropertyID::kOpacity
                    | CSSPropertyID::kFillOpacity
                    | CSSPropertyID::kFloodOpacity
                    | CSSPropertyID::kStopOpacity
                    | CSSPropertyID::kStrokeOpacity
            )
        {
            number /= 100.0;
            unit = UnitType::kNumber;
        }
        number = match math.Category() {
            C::Angle => crate::css_value_clamping_utils::CSSValueClampingUtils::ClampAngle(number),
            C::Length => {
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(number)
            }
            _ => crate::css_value_clamping_utils::CSSValueClampingUtils::ClampDouble(number),
        };
        if matches!(
            math.range,
            crate::css_math_function_value::ValueRange::Integer
                | crate::css_math_function_value::ValueRange::NonNegativeInteger
                | crate::css_math_function_value::ValueRange::PositiveInteger
        ) {
            unit = UnitType::kInteger;
        }
        return Apply(
            id,
            b,
            parent,
            &crate::production_css_value::numeric(number, unit),
            root,
            media,
        );
    }
    if IsTimingStorageProperty(id) {
        if let Some(value) = ResolveNumericMathList(id, v)? {
            return Apply(id, b, parent, &value, root, media);
        }
    }
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue()
        || v.IsUnsetValue() && !CSSProperty::Get(id).IsInherited()
        || inherit && parent.is_none();
    if grid_application::IsGridProperty(id) {
        return grid_application::Apply(id, b, parent, v, root, media, inherit, initial);
    }
    if animation_application::IsAnimationProperty(id) {
        return animation_application::Apply(id, b, parent, v, inherit, initial);
    }
    if font_variant_application::IsFontProperty(id) {
        if inherit && !initial && v.IsInheritedValue() {
            b.SetHasExplicitInheritance();
            parent.unwrap().SetChildHasExplicitInheritance();
        }
        return font_variant_application::Apply(id, b, parent, v, inherit, initial);
    }
    if matches!(id, CSSPropertyID::kBoxShadow | CSSPropertyID::kTextShadow) {
        return ApplyShadow(id, b, parent, v, root, media, inherit, initial);
    }
    if matches!(
        id,
        CSSPropertyID::kContainerName | CSSPropertyID::kContainerType
    ) {
        return ApplyContainer(id, b, parent, v, inherit, initial);
    }
    if IsOverflowOutlineProperty(id) {
        return ApplyOverflowOutline(id, b, parent, v, root, media, inherit, initial);
    }
    if id == CSSPropertyID::kContent {
        return content_application::Apply(b, parent, v, None);
    }
    if IsTimingStorageProperty(id) {
        if inherit && !initial && v.IsInheritedValue() {
            b.SetHasExplicitInheritance();
            parent.unwrap().SetChildHasExplicitInheritance();
        }
        return ApplyTimingStorage(id, b, parent, v, inherit, initial);
    }
    if matches!(
        id,
        CSSPropertyID::kFontFeatureSettings
            | CSSPropertyID::kFontVariationSettings
            | CSSPropertyID::kFontStretch
            | CSSPropertyID::kFontKerning
            | CSSPropertyID::kFontOpticalSizing
    ) {
        if inherit && !initial && v.IsInheritedValue() {
            b.SetHasExplicitInheritance();
            parent.unwrap().SetChildHasExplicitInheritance();
        }
        return ApplyFont(id, b, parent, v, root, media, inherit, initial);
    }
    if inherit {
        if v.IsInheritedValue() && !CSSProperty::Get(id).IsInherited() {
            b.SetHasExplicitInheritance();
            parent.unwrap().SetChildHasExplicitInheritance();
        }
        return ApplyInherit(id, b, parent.unwrap());
    }
    if initial {
        return ApplyInitial(id, b);
    }
    match id {
        CSSPropertyID::kAlignContent | CSSPropertyID::kJustifyContent => {
            return ApplyConvertedContentAlignment(id, b, &ConvertContentAlignment(id, v)?);
        }
        CSSPropertyID::kAlignItems
        | CSSPropertyID::kAlignSelf
        | CSSPropertyID::kJustifyItems
        | CSSPropertyID::kJustifySelf => {
            return ApplyConvertedSelfAlignment(id, b, &ConvertSelfAlignment(id, v)?);
        }
        CSSPropertyID::kFlexWrap => {
            return ApplyConvertedFlexWrap(id, b, &ConvertFlexWrap(id, v)?);
        }
        _ => {}
    }
    match v.Payload() {
        CSSValuePayload::kIdentifierClass(value) => {
            use CSSValueID::*;
            if let Some(width) = match value.0 {
                kThin => Some(1),
                kMedium => Some(3),
                kThick => Some(5),
                _ => None,
            } {
                if ApplyConvertedBorderWidth(id, b, &width).is_ok() {
                    return Ok(());
                }
            }
            let length = match value.0 {
                kAuto => Some(Length::new(0.0, LengthType::kAuto)),
                kNone => Some(Length::new(0.0, LengthType::kNone)),
                kMinContent => Some(Length::new(0.0, LengthType::kMinContent)),
                kMaxContent => Some(Length::new(0.0, LengthType::kMaxContent)),
                kFitContent => Some(Length::new(0.0, LengthType::kFitContent)),
                kStretch => Some(Length::new(0.0, LengthType::kStretch)),
                _ => None,
            };
            if let Some(length) = length {
                if ApplyConvertedLength(id, b, &length).is_ok() {
                    return Ok(());
                }
            }
            let color = if value.0 == kCurrentcolor {
                Some(StyleColor::CurrentColor())
            } else {
                crate::parser::production_property_metadata::NamedColor(
                    crate::css_value_keywords::GetCSSValueName(value.0),
                )
                .map(StyleColor::from_color)
            };
            if let Some(color) = color {
                return ApplyColor(id, b, parent, &color);
            }
            ApplyIdentifier(id, b, value.0)
        }
        CSSValuePayload::kNumericLiteralClass(value) => {
            if value.IsNumber() {
                if let Ok(()) = ApplyNumber(id, b, value.DoubleValue()) {
                    return Ok(());
                }
            }
            if value.GetType() != UnitType::kPercentage {
                if let Ok(px) = Pixels(
                    id,
                    value.DoubleValue(),
                    value.GetType(),
                    b.GetFontDescription().ComputedSize(),
                    root,
                    media,
                ) {
                    let width=crate::resolver::style_builder_converter::StyleBuilderConverter::ClampLineWidth(px as f32 as f64);
                    if ApplyConvertedBorderWidth(id, b, &width).is_ok() {
                        return Ok(());
                    }
                }
            }
            let length = if value.GetType() == UnitType::kPercentage {
                Length::Percent(value.DoubleValue())
            } else {
                Length::Fixed(Pixels(
                    id,
                    value.DoubleValue(),
                    value.GetType(),
                    b.GetFontDescription().ComputedSize(),
                    root,
                    media,
                )?)
            };
            ApplyConvertedLength(id, b, &length)
        }
        CSSValuePayload::kColorClass(value) => {
            ApplyColor(id, b, parent, &StyleColor::from_color(value.0))
        }
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
}

// cpp: generated longhands.cc:2351-2649,2826-2852,17527-17737.
fn IsTimingStorageProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kAnimationDelay
            | CSSPropertyID::kAnimationDuration
            | CSSPropertyID::kAnimationDirection
            | CSSPropertyID::kAnimationFillMode
            | CSSPropertyID::kAnimationIterationCount
            | CSSPropertyID::kAnimationPlayState
            | CSSPropertyID::kAnimationTimingFunction
            | CSSPropertyID::kTransitionDelay
            | CSSPropertyID::kTransitionDuration
            | CSSPropertyID::kTransitionProperty
            | CSSPropertyID::kTransitionBehavior
            | CSSPropertyID::kTransitionTimingFunction
    )
}

fn MapList<T>(
    id: CSSPropertyID,
    value: &Value,
    mut convert: impl FnMut(&Value) -> std::result::Result<T, LonghandApplicationError>,
) -> std::result::Result<Vec<T>, LonghandApplicationError> {
    let CSSValuePayload::kValueListClass(list) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.values.is_empty() {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    list.values.iter().map(|value| convert(value)).collect()
}

// cpp: CSSPrimitiveValue::ComputeSeconds; CSSToStyleMap::MapAnimationTimingDelay.
fn Seconds(id: CSSPropertyID, value: &Value) -> std::result::Result<f64, LonghandApplicationError> {
    let CSSValuePayload::kNumericLiteralClass(number) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    let seconds = match number.GetType() {
        UnitType::kSeconds => number.DoubleValue(),
        UnitType::kMilliseconds => number.DoubleValue() / 1000.0,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    Ok(seconds)
}

// cpp: CSSToStyleMap::MapAnimationTimingFunction (css_to_style_map.cc:471-529).
fn MapTimingFunction(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<foundation::ScopedRefPtr<TimingFunction>, LonghandApplicationError> {
    use CSSValueID::*;
    let timing = match value.Payload() {
        CSSValuePayload::kIdentifierClass(value) => match value.0 {
            kLinear => TimingFunction::Linear(vec![]),
            kEase => TimingFunction::CubicBezierPreset(EaseType::EASE),
            kEaseIn => TimingFunction::CubicBezierPreset(EaseType::EASE_IN),
            kEaseOut => TimingFunction::CubicBezierPreset(EaseType::EASE_OUT),
            kEaseInOut => TimingFunction::CubicBezierPreset(EaseType::EASE_IN_OUT),
            kStepStart => TimingFunction::Steps {
                number_of_steps: 1,
                step_position: StepPosition::START,
            },
            kStepEnd => TimingFunction::Steps {
                number_of_steps: 1,
                step_position: StepPosition::END,
            },
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        },
        CSSValuePayload::kCubicBezierTimingFunctionClass(value) => {
            TimingFunction::CubicBezier(value.0)
        }
        CSSValuePayload::kLinearTimingFunctionClass(value) => TimingFunction::Linear(
            value
                .0
                .iter()
                .map(|point| LinearEasingPoint {
                    input: point.input,
                    output: point.output,
                })
                .collect(),
        ),
        CSSValuePayload::kStepsTimingFunctionClass(value) => {
            let resolved = ResolveNumericMathList(id, &value.steps)?;
            let steps_value = resolved.as_deref().unwrap_or(&value.steps);
            let CSSValuePayload::kNumericLiteralClass(number) = steps_value.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            if !number.IsNumber() {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            let mut steps = number
                .DoubleValue()
                .round()
                .clamp(i32::MIN as f64, i32::MAX as f64) as i32;
            let position = match value.position {
                kStart => StepPosition::START,
                kEnd => StepPosition::END,
                kJumpBoth => StepPosition::JUMP_BOTH,
                kJumpEnd => StepPosition::JUMP_END,
                kJumpNone => StepPosition::JUMP_NONE,
                kJumpStart => StepPosition::JUMP_START,
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            };
            if position == StepPosition::JUMP_NONE && steps < 2 {
                steps = 2;
            }
            TimingFunction::Steps {
                number_of_steps: steps,
                step_position: position,
            }
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    Ok(foundation::ScopedRefPtr::new(timing))
}

// cpp: CSSToStyleMap::MapAnimation* (css_to_style_map.cc:254-346,389-397,453-469).
// Conversion completes before mutating any list, retaining the typed error boundary.
fn ApplyTimingStorage(
    id: CSSPropertyID,
    builder: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    value: &Value,
    inherit: bool,
    initial: bool,
) -> Result {
    if initial {
        return ApplyInitial(id, builder);
    }
    if inherit {
        return ApplyInherit(id, builder, parent.unwrap());
    }
    use CSSPropertyID::*;
    use CSSValueID::*;
    match id {
        kAnimationDelay | kTransitionDelay => ApplyConvertedDelayStartList(
            id,
            builder,
            &MapList(id, value, |value| {
                Ok(TimingDelay {
                    time_delay: Seconds(id, value)?,
                    relative_delay: None,
                })
            })?,
        ),
        kAnimationDuration | kTransitionDuration => ApplyConvertedDurationList(
            id,
            builder,
            &MapList(id, value, |value| {
                if matches!(value.Payload(), CSSValuePayload::kIdentifierClass(value) if value.0 == kAuto)
                {
                    return Ok(None);
                }
                let seconds = Seconds(id, value)?;
                if seconds < 0.0 {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                Ok(Some(seconds))
            })?,
        ),
        kAnimationDirection => ApplyConvertedDirectionList(
            id,
            builder,
            &MapList(id, value, |value| {
                Ok(match Identifier(id, value)? {
                    kNormal => PlaybackDirection::NORMAL,
                    kAlternate => PlaybackDirection::ALTERNATE_NORMAL,
                    kReverse => PlaybackDirection::REVERSE,
                    kAlternateReverse => PlaybackDirection::ALTERNATE_REVERSE,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                })
            })?,
        ),
        kAnimationFillMode => ApplyConvertedFillModeList(
            id,
            builder,
            &MapList(id, value, |value| {
                Ok(match Identifier(id, value)? {
                    kNone => FillMode::NONE,
                    kForwards => FillMode::FORWARDS,
                    kBackwards => FillMode::BACKWARDS,
                    kBoth => FillMode::BOTH,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                })
            })?,
        ),
        kAnimationIterationCount => ApplyConvertedIterationCountList(
            id,
            builder,
            &MapList(id, value, |value| {
                if matches!(value.Payload(), CSSValuePayload::kIdentifierClass(value) if value.0 == kInfinite)
                {
                    return Ok(f64::INFINITY);
                }
                let CSSValuePayload::kNumericLiteralClass(number) = value.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                if !number.IsNumber() || number.DoubleValue() < 0.0 {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                Ok(number.DoubleValue())
            })?,
        ),
        kAnimationPlayState => ApplyConvertedPlayStateList(
            id,
            builder,
            &MapList(id, value, |value| {
                Ok(match Identifier(id, value)? {
                    kPaused => EAnimPlayState::kPaused,
                    kRunning => EAnimPlayState::kPlaying,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                })
            })?,
        ),
        kTransitionProperty => ApplyConvertedPropertyList(
            id,
            builder,
            &MapList(id, value, |value| {
                Ok(match value.Payload() {
                    CSSValuePayload::kCustomIdentClass(value) => {
                        if value.property != CSSPropertyID::kInvalid {
                            TransitionProperty::Known(value.property)
                        } else {
                            TransitionProperty::Unknown(value.name.clone())
                        }
                    }
                    CSSValuePayload::kIdentifierClass(value) if value.0 == CSSValueID::kAll => {
                        TransitionProperty::Known(CSSPropertyID::kAll)
                    }
                    CSSValuePayload::kIdentifierClass(value) if value.0 == kNone => {
                        TransitionProperty::None
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                })
            })?,
        ),
        kTransitionBehavior => ApplyConvertedBehaviorList(
            id,
            builder,
            &MapList(id, value, |value| {
                Ok(match Identifier(id, value)? {
                    kNormal => TransitionBehavior::kNormal,
                    kAllowDiscrete => TransitionBehavior::kAllowDiscrete,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                })
            })?,
        ),
        kAnimationTimingFunction | kTransitionTimingFunction => ApplyConvertedTimingFunctionList(
            id,
            builder,
            &MapList(id, value, |value| MapTimingFunction(id, value))?,
        ),
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
}

fn Identifier(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<CSSValueID, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(value) = value.Payload() {
        Ok(value.0)
    } else {
        Err(LonghandApplicationError::InvalidValue(id))
    }
}

// cpp: css_identifier_value_mappings.h:1325-1365.
fn ItemPositionFor(
    id: CSSPropertyID,
    value: CSSValueID,
) -> std::result::Result<ItemPosition, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match value {
        kLegacy => ItemPosition::kLegacy,
        kAuto => ItemPosition::kAuto,
        kNormal => ItemPosition::kNormal,
        kStretch => ItemPosition::kStretch,
        kBaseline | kFirstBaseline => ItemPosition::kBaseline,
        kLastBaseline => ItemPosition::kLastBaseline,
        kAnchorCenter => ItemPosition::kAnchorCenter,
        kCenter => ItemPosition::kCenter,
        kStart => ItemPosition::kStart,
        kEnd => ItemPosition::kEnd,
        kSelfStart => ItemPosition::kSelfStart,
        kSelfEnd => ItemPosition::kSelfEnd,
        kFlexStart => ItemPosition::kFlexStart,
        kFlexEnd => ItemPosition::kFlexEnd,
        kLeft => ItemPosition::kLeft,
        kRight => ItemPosition::kRight,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}

// cpp: css_identifier_value_mappings.h:1493-1504.
fn OverflowFor(
    id: CSSPropertyID,
    value: CSSValueID,
) -> std::result::Result<OverflowAlignment, LonghandApplicationError> {
    Ok(match value {
        CSSValueID::kSafe => OverflowAlignment::kSafe,
        CSSValueID::kUnsafe => OverflowAlignment::kUnsafe,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}

// cpp: style_builder_converter.cc:1434-1462 ConvertSelfOrDefaultAlignmentData.
fn ConvertSelfAlignment(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<StyleSelfAlignmentData, LonghandApplicationError> {
    let mut result = ComputedStyleInitialValues::InitialAlignSelf();
    if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
        let first = Identifier(id, &pair.first)?;
        let second = Identifier(id, &pair.second)?;
        match first {
            CSSValueID::kLegacy => {
                result.SetPositionType(ItemPositionType::kLegacy);
                result.SetPosition(ItemPositionFor(id, second)?);
            }
            CSSValueID::kFirst | CSSValueID::kLast if second == CSSValueID::kBaseline => {
                result.SetPosition(if first == CSSValueID::kFirst {
                    ItemPosition::kBaseline
                } else {
                    ItemPosition::kLastBaseline
                });
            }
            _ => {
                result.SetOverflow(OverflowFor(id, first)?);
                result.SetPosition(ItemPositionFor(id, second)?);
            }
        }
    } else {
        result.SetPosition(ItemPositionFor(id, Identifier(id, value)?)?);
    }
    Ok(result)
}

// cpp: style_builder_converter.cc:1464-1489 ConvertContentAlignmentData;
// css_identifier_value_mappings.h:1405-1431,1459-1474,1493-1504.
fn ConvertContentAlignment(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<StyleContentAlignmentData, LonghandApplicationError> {
    let CSSValuePayload::kCSSContentDistributionClass(value) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    use CSSValueID::*;
    let mut result = ComputedStyleInitialValues::InitialContentAlignment();
    if value.distribution != kInvalid {
        result.SetDistribution(match value.distribution {
            kSpaceBetween => ContentDistributionType::kSpaceBetween,
            kSpaceAround => ContentDistributionType::kSpaceAround,
            kSpaceEvenly => ContentDistributionType::kSpaceEvenly,
            kStretch => ContentDistributionType::kStretch,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        });
    }
    if value.position != kInvalid {
        result.SetPosition(match value.position {
            kNormal => ContentPosition::kNormal,
            kBaseline | kFirstBaseline => ContentPosition::kBaseline,
            kLastBaseline => ContentPosition::kLastBaseline,
            kCenter => ContentPosition::kCenter,
            kStart => ContentPosition::kStart,
            kEnd => ContentPosition::kEnd,
            kFlexStart => ContentPosition::kFlexStart,
            kFlexEnd => ContentPosition::kFlexEnd,
            kLeft => ContentPosition::kLeft,
            kRight => ContentPosition::kRight,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        });
    }
    if value.overflow != kInvalid {
        result.SetOverflow(OverflowFor(id, value.overflow)?);
    }
    Ok(result)
}

// cpp: style_builder_converter.cc:404-431 ConvertFlexWrapData.
fn ConvertFlexWrap(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<StyleFlexWrapData, LonghandApplicationError> {
    let mut mode = FlexWrapMode::kNowrap;
    let mut balanced = false;
    let mut process = |value: &Value| -> Result {
        match Identifier(id, value)? {
            CSSValueID::kNowrap => mode = FlexWrapMode::kNowrap,
            CSSValueID::kWrap => mode = FlexWrapMode::kWrap,
            CSSValueID::kWrapReverse => mode = FlexWrapMode::kWrapReverse,
            CSSValueID::kBalance => balanced = true,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }
        Ok(())
    };
    if let CSSValuePayload::kValueListClass(list) = value.Payload() {
        for value in &list.values {
            process(value)?;
        }
    } else {
        process(value)?;
    }
    if balanced && mode == FlexWrapMode::kNowrap {
        mode = FlexWrapMode::kWrap;
    }
    Ok(StyleFlexWrapData::with_balance(mode, balanced))
}
fn ApplyColor(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    _parent: Option<&ComputedStyle>,
    c: &StyleColor,
) -> Result {
    ApplyConvertedColor(id, b, c)
}
// cpp: longhands.cc Font* Apply*; style_builder_converter.cc ConvertFontSize/Weight;
// font_builder.cc:116-131,284-292,426-633 (unzoomed scalar-font branches).
fn ApplyFont(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    _root: f32,
    _media: &MediaValuesCachedData,
    inherit: bool,
    initial: bool,
) -> Result {
    let mut d = b.GetFontDescription().clone();
    let p = parent.map(ComputedStyle::GetFontDescription);
    match id {
        CSSPropertyID::kFontFeatureSettings => {
            let settings = if inherit && !initial {
                p.unwrap().FeatureSettings().cloned()
            } else if initial {
                None
            } else {
                Some(ConvertFontFeatureSettings(id, v)?)
            };
            d.SetFeatureSettings(settings);
        }
        CSSPropertyID::kFontVariationSettings => {
            let settings = if inherit && !initial {
                p.unwrap().VariationSettings().cloned()
            } else if initial
                || matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kNormal)
            {
                None
            } else {
                Some(ConvertFontVariationSettings(id, v)?)
            };
            d.SetVariationSettings(settings);
        }
        // cpp: generated longhands.cc:381-389,445-453;
        // style_builder_converter.cc:580-608,645-666; font_builder.h:138-144.
        CSSPropertyID::kFontKerning => {
            use font_engine::fonts::font_description::Kerning;
            let kerning = if inherit && !initial {
                p.unwrap().GetKerning()
            } else if initial {
                Kerning::kAutoKerning
            } else {
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(i) => match i.0 {
                        CSSValueID::kAuto => Kerning::kAutoKerning,
                        CSSValueID::kNormal => Kerning::kNormalKerning,
                        CSSValueID::kNone => Kerning::kNoneKerning,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    },
                    _ => return Err(LonghandApplicationError::Unsupported(id)),
                }
            };
            d.SetKerning(kerning);
        }
        CSSPropertyID::kFontOpticalSizing => {
            use font_engine::fonts::font_optical_sizing::OpticalSizing;
            let sizing = if inherit && !initial {
                p.unwrap().FontOpticalSizing()
            } else if initial {
                OpticalSizing::kAutoOpticalSizing
            } else {
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(i) => match i.0 {
                        CSSValueID::kAuto => OpticalSizing::kAutoOpticalSizing,
                        CSSValueID::kNone => OpticalSizing::kNoneOpticalSizing,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    },
                    _ => return Err(LonghandApplicationError::Unsupported(id)),
                }
            };
            d.SetFontOpticalSizing(sizing);
        }
        // cpp: generated longhands.cc:579-587; converter.cc:1032-1090.
        CSSPropertyID::kFontStretch => {
            use font_engine::fonts::font_selection_types::*;
            let stretch = if inherit && !initial {
                p.unwrap().Stretch()
            } else if initial {
                kNormalWidthValue
            } else {
                match v.Payload() {
                    CSSValuePayload::kNumericLiteralClass(n)
                        if n.GetType() == UnitType::kPercentage =>
                    {
                        FontSelectionValue::from_double(n.DoubleValue())
                    }
                    CSSValuePayload::kIdentifierClass(i) => match i.0 {
                        CSSValueID::kUltraCondensed => kUltraCondensedWidthValue,
                        CSSValueID::kExtraCondensed => kExtraCondensedWidthValue,
                        CSSValueID::kCondensed => kCondensedWidthValue,
                        CSSValueID::kSemiCondensed => kSemiCondensedWidthValue,
                        CSSValueID::kNormal => kNormalWidthValue,
                        CSSValueID::kSemiExpanded => kSemiExpandedWidthValue,
                        CSSValueID::kExpanded => kExpandedWidthValue,
                        CSSValueID::kExtraExpanded => kExtraExpandedWidthValue,
                        CSSValueID::kUltraExpanded => kUltraExpandedWidthValue,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    },
                    _ => return Err(LonghandApplicationError::Unsupported(id)),
                }
            };
            d.SetStretch(stretch);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    StageFontDescription(b, &d);
    Ok(())
}

// cpp: style_builder_converter.cc:676-705. Last occurrence wins and output
// is sorted by the packed OpenType tag, matching Chromium's std::map.
fn ConvertFontFeatureSettings(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<
    font_engine::fonts::opentype::font_settings::FontFeatureSettings,
    LonghandApplicationError,
> {
    use font_engine::fonts::opentype::font_settings::*;
    let mut settings = FontSettings::default();
    if matches!(value.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kNormal)
    {
        return Ok(std::sync::Arc::new(settings));
    }
    let CSSValuePayload::kValueListClass(list) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Comma || list.values.is_empty()
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut features = std::collections::BTreeMap::new();
    for item in &list.values {
        let CSSValuePayload::kFontFeatureClass(feature) = item.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        let CSSValuePayload::kNumericLiteralClass(number) = feature.value.Payload() else {
            return Err(LonghandApplicationError::Unsupported(id));
        };
        features.insert(
            AtomicStringToFourByteTag(&feature.tag),
            number.DoubleValue() as i32,
        );
    }
    for (tag, value) in features {
        settings.Append(FontFeature::new(tag, value));
    }
    Ok(std::sync::Arc::new(settings))
}
// cpp: style_builder_converter.cc:707-743. BTreeMap preserves the same
// deduplication and final tag order as Chromium's HashMap followed by sort.
fn ConvertFontVariationSettings(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<
    font_engine::fonts::opentype::font_settings::FontVariationSettings,
    LonghandApplicationError,
> {
    use font_engine::fonts::opentype::font_settings::*;
    let CSSValuePayload::kValueListClass(list) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Comma || list.values.is_empty()
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut axes = std::collections::BTreeMap::new();
    for item in &list.values {
        let CSSValuePayload::kFontVariationClass(axis) = item.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        let CSSValuePayload::kNumericLiteralClass(number) = axis.value.Payload() else {
            return Err(LonghandApplicationError::Unsupported(id));
        };
        axes.insert(
            AtomicStringToFourByteTag(&axis.tag),
            number.DoubleValue().clamp(f32::MIN as f64, f32::MAX as f64) as f32,
        );
    }
    let mut settings = FontSettings::default();
    for (tag, value) in axes {
        settings.Append(FontVariationAxis::new(tag, value));
    }
    Ok(std::sync::Arc::new(settings))
}

// cpp: platform/fonts/font_data.cc:33; computed_style_base.h:SetFont.
// This document assembly runs before Layout's font-resolver scope. Its complete
// FontDescription is installed in an actual Font; layout binds font data from
// ConstraintSpace before using font metrics. No metric-dependent CSS unit is
// admitted while the description-only font is present.
pub fn StageFontDescription(
    builder: &mut ComputedStyleBuilder,
    description: &font_engine::FontDescription,
) {
    // FontBuilder marks language override dirty independently of the source's
    // FontDescription equality, which does not compare language_override_.
    // font_builder.cc:577-582 also compares palette owner pointers even when
    // font_description.cc:154 ValuesEquivalent sees equal palette values.
    if builder.GetFontDescription() != description
        || builder.GetFontDescription().FontLanguageOverride() != description.FontLanguageOverride()
        || builder.GetFontDescription().GetFontPalette() != description.GetFontPalette()
    {
        builder.SetFont(foundation::Member::from_ptr(
            foundation::MakeGarbageCollected(font_engine::Font::new(description.clone())),
        ));
    }
}

#[cfg(test)]
mod content_remaining_tests {
    use super::*;
    use layoutng_style::style::content_data::{
        CloneContentData, ContentData, CounterContentData, QuoteContentData, TextContentData,
    };
    #[test]
    fn content_typed_values_build_native_chain_and_survive_gc() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        let properties = crate::parser::production_property_parser::ParseProperty(
            CSSPropertyID::kContent,
            &foundation::String::from("'a' 'b' open-quote counter(item) counters(item, '') close-quote / 'alt' 'other' counter(item)"),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        ).unwrap();
        Apply(
            CSSPropertyID::kContent,
            &mut builder,
            None,
            properties[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
        )
        .unwrap();
        let style = foundation::Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
        let first = unsafe { (&*style.Get()).GetContentData().unwrap() };
        assert_eq!(
            unsafe { (&*(first as *const TextContentData)).GetText().Utf8() },
            "ab"
        );
        let quote = unsafe { (&*first).Next().unwrap() };
        assert_eq!(
            unsafe { (&*(quote as *const QuoteContentData)).Quote() },
            layoutng_style::style::computed_style_constants::QuoteType::kOpen
        );
        let counter = unsafe { (&*quote).Next().unwrap() };
        let single = unsafe { &*(counter as *const CounterContentData) };
        assert_eq!(single.Identifier(), "item");
        assert_eq!(single.ListStyle(), "decimal");
        assert!(single.Separator().IsNull());
        let counters = unsafe { (&*counter).Next().unwrap() };
        assert!(!unsafe {
            (&*(counters as *const CounterContentData))
                .Separator()
                .IsNull()
        });
        let mut current = Some(first);
        let mut types = Vec::new();
        while let Some(item) = current {
            let item = unsafe { &*item };
            types.push((
                item.IsText(),
                item.IsQuote(),
                item.IsCounter(),
                item.IsAltText(),
                item.IsAltCounter(),
            ));
            current = item.Next();
        }
        assert_eq!(
            types,
            vec![
                (true, false, false, false, false),
                (false, true, false, false, false),
                (false, false, true, false, false),
                (false, false, true, false, false),
                (false, true, false, false, false),
                (false, false, false, true, false),
                (false, false, false, true, false),
                (false, false, true, false, true),
            ]
        );
        let cloned = foundation::Persistent::from_ptr(
            CloneContentData(unsafe { &*first }) as *mut TextContentData
        );
        drop(_heap);
        foundation::CollectLayoutHeapForTesting();
        assert!(unsafe { (&*cloned.Get() as &dyn ContentData) == (&*first) });
        let _heap = foundation::LayoutHeapScope::new();
        let mut builder = ComputedStyleBuilder::from_style(unsafe { &*style.Get() });
        let before = builder.GetContentData();
        let image = crate::parser::production_property_parser::ParseProperty(
            CSSPropertyID::kContent,
            &foundation::String::from("url(icon.svg)"),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(
            content_application::Apply(&mut builder, None, image[0].Value(), None),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kContent
            ))
        );
        assert_eq!(
            builder.GetContentData().map(|value| value as *mut ()),
            before.map(|value| value as *mut ())
        );
    }
}

#[cfg(test)]
mod background_linear_image_production_tests {
    use super::*;
    fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) -> Result {
        let parsed = crate::parser::production_property_parser::ParseProperty(
            CSSPropertyID::kBackgroundImage,
            &foundation::String::from(css),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        Apply(
            CSSPropertyID::kBackgroundImage,
            b,
            parent,
            parsed[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
        )
    }
    #[test]
    fn background_linear_image_properties_reach_native_layers_and_preserve_dependency_errors() {
        use layoutng_style::style::generated_gradient_image::CSSLinearGradientValue;
        let heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        apply(
            &mut b,
            None,
            "linear-gradient(to right, red 20%, green, blue 80%), none",
        )
        .unwrap();
        let first = b.AccessBackgroundLayers();
        let image = unsafe { &*(*first).GetImage() };
        assert!(image.IsGeneratedImage());
        assert!(image.CanRender() && image.IsLoaded());
        let value = unsafe { &*(image.CssValue() as *const CSSLinearGradientValue) };
        assert_eq!(value.Data().start, [0.0, 0.0]);
        assert_eq!(value.Data().end, [1.0, 0.0]);
        // A generated subtype cannot be supplied as a fetched URL resource.
        assert!(matches!(
            unsafe {
                FetchedImageBinding::FromResource(
                    std::num::NonZeroU64::new(7).unwrap(),
                    std::ptr::NonNull::new((*first).GetImage()).unwrap(),
                )
            },
            Err(LonghandApplicationError::InvalidValue(
                CSSPropertyID::kBackgroundImage
            ))
        ));
        assert_eq!(
            value
                .Data()
                .stops
                .iter()
                .map(|s| s.offset)
                .collect::<Vec<_>>(),
            vec![0.2f32 as f64, 0.5, 0.8f32 as f64]
        );
        let second = unsafe { &*(*first).Next() };
        assert!(second.IsImageSet() && second.GetImage().is_null());
        let previous = unsafe { (*first).GetImage() };
        let size = crate::parser::production_property_parser::ParseProperty(
            CSSPropertyID::kBackgroundSize,
            &foundation::String::from("calc(2px)"),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        Apply(
            CSSPropertyID::kBackgroundSize,
            &mut b,
            None,
            size[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
        )
        .unwrap();
        assert_eq!(
            b.AccessBackgroundLayers().SizeLength().Width().Pixels(),
            2.0
        );
        assert_eq!(b.AccessBackgroundLayers().GetImage(), previous);
        for css in [
            "url(icon.svg)",
            "linear-gradient(45deg, red, blue)",
            "linear-gradient(red 2px, blue)",
            "linear-gradient(red calc(10% + 1px), blue)",
            "linear-gradient(red, 50%, blue)",
            "repeating-linear-gradient(red,blue)",
            "linear-gradient(currentcolor,blue)",
        ] {
            assert_eq!(
                apply(&mut b, None, css),
                Err(LonghandApplicationError::Unsupported(
                    CSSPropertyID::kBackgroundImage
                )),
                "{css}"
            );
            assert_eq!(
                unsafe { (*b.AccessBackgroundLayers()).GetImage() },
                previous
            );
        }
        let parent = foundation::Persistent::from_ptr(b.TakeStyle() as *mut ComputedStyle);
        let p = unsafe { &*parent.Get() };
        let mut child = ComputedStyleBuilder::from_style(initial);
        apply(&mut child, Some(p), "inherit").unwrap();
        assert_eq!(
            unsafe { (*child.AccessBackgroundLayers()).GetImage() },
            previous
        );
        assert!(child.HasExplicitInheritance());
        apply(
            &mut child,
            Some(p),
            "linear-gradient(red 80%, green 20%, blue)",
        )
        .unwrap();
        let image = unsafe { &*(*child.AccessBackgroundLayers()).GetImage() };
        let value = unsafe { &*(image.CssValue() as *const CSSLinearGradientValue) };
        assert_eq!(
            value
                .Data()
                .stops
                .iter()
                .map(|s| s.offset)
                .collect::<Vec<_>>(),
            vec![0.8f32 as f64, 0.8f32 as f64, 1.0]
        );
        assert!(!unsafe { &*(*child.AccessBackgroundLayers()).Next() }.IsImageSet());
        apply(&mut child, Some(p), "initial").unwrap();
        assert!(unsafe { (*child.AccessBackgroundLayers()).GetImage() }.is_null());
        assert!(unsafe { &*child.AccessBackgroundLayers() }.IsImageSet());
        let retained = foundation::Persistent::from_ptr(child.TakeStyle() as *mut ComputedStyle);
        drop(heap);
        foundation::CollectLayoutHeapForTesting();
        assert!(unsafe { &*(*parent.Get()).BackgroundLayers() }.GetImage() == previous);
        assert!(unsafe { &*(*retained.Get()).BackgroundLayers() }
            .GetImage()
            .is_null());
    }
}

#[cfg(test)]
mod shadow_properties_production_tests {
    use super::*;
    fn apply(
        b: &mut ComputedStyleBuilder,
        p: Option<&ComputedStyle>,
        id: CSSPropertyID,
        css: &str,
    ) -> Result {
        let parsed = crate::parser::production_property_parser::ParseProperty(
            id,
            &foundation::String::from(css),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        Apply(
            id,
            b,
            p,
            parsed[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
        )
    }
    #[test]
    fn shadow_properties_store_native_defaults_components_inheritance_and_gc() {
        use layoutng_style::style::shadow_data::ShadowStyle;
        let heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        // The document engine stages computed font size before resolving em.
        let mut font = b.GetFontDescription().clone();
        font.SetComputedSize(16.0);
        StageFontDescription(&mut b, &font);
        apply(
            &mut b,
            None,
            CSSPropertyID::kBoxShadow,
            "inset red 1px -2px 3px -4px, 0 0",
        )
        .unwrap();
        apply(
            &mut b,
            None,
            CSSPropertyID::kTextShadow,
            "1em 2px 3px rgba(10,20,30,.5), currentcolor -1px -2px",
        )
        .unwrap();
        let shadows = unsafe { &*b.BoxShadow() }.Shadows();
        assert_eq!(shadows.len(), 2);
        assert_eq!(
            (
                shadows[0].X(),
                shadows[0].Y(),
                shadows[0].BlurValue(),
                shadows[0].Spread()
            ),
            (1.0, -2.0, 3.0, -4.0)
        );
        assert_eq!(shadows[0].Style(), ShadowStyle::kInset);
        assert_eq!(
            shadows[0].GetColor().GetColor(),
            foundation::Color::FromRGBA(255, 0, 0, 255)
        );
        assert_eq!(shadows[0].Opacity(), 1.0);
        assert_eq!(shadows[0].BlurAsSigma(), 1.5);
        assert_eq!(
            (
                shadows[1].X(),
                shadows[1].Y(),
                shadows[1].BlurValue(),
                shadows[1].Spread()
            ),
            (0.0, 0.0, 0.0, 0.0)
        );
        assert_eq!(shadows[1].Style(), ShadowStyle::kNormal);
        assert!(shadows[1].GetColor().IsCurrentColor());
        let text = unsafe { &*b.TextShadow() }.Shadows();
        assert_eq!(
            (
                text[0].X(),
                text[0].Y(),
                text[0].BlurValue(),
                text[0].Spread()
            ),
            (16.0, 2.0, 3.0, 0.0)
        );
        assert_eq!(text[0].Style(), ShadowStyle::kNormal);
        assert!(!text[0].GetColor().IsCurrentColor());
        assert!(text[1].GetColor().IsCurrentColor());
        let parent = foundation::Persistent::from_ptr(b.TakeStyle() as *mut ComputedStyle);
        let p = unsafe { &*parent.Get() };
        let mut child = ComputedStyleBuilder::from_style(initial);
        apply(&mut child, Some(p), CSSPropertyID::kBoxShadow, "inherit").unwrap();
        apply(&mut child, Some(p), CSSPropertyID::kTextShadow, "unset").unwrap();
        assert_eq!(child.BoxShadow(), p.BoxShadow());
        assert_eq!(child.TextShadow(), p.TextShadow());
        assert!(child.HasExplicitInheritance());
        let before = child.BoxShadow();
        assert_eq!(
            apply(
                &mut child,
                None,
                CSSPropertyID::kBoxShadow,
                "1px 2px, canvas 3px 4px"
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kBoxShadow
            ))
        );
        assert_eq!(child.BoxShadow(), before);
        assert_eq!(
            apply(&mut child, None, CSSPropertyID::kTextShadow, "1px 2ex"),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kTextShadow
            ))
        );
        assert_eq!(child.TextShadow(), p.TextShadow());
        apply(
            &mut child,
            None,
            CSSPropertyID::kBoxShadow,
            "1e100px -1e100px 1e100px -1e100px",
        )
        .unwrap();
        let extreme = unsafe { &*child.BoxShadow() }.Shadows();
        assert_eq!(
            (
                extreme[0].X(),
                extreme[0].Y(),
                extreme[0].BlurValue(),
                extreme[0].Spread()
            ),
            (f32::MAX, -f32::MAX, f32::MAX, -f32::MAX)
        );
        apply(&mut child, Some(p), CSSPropertyID::kBoxShadow, "unset").unwrap();
        apply(&mut child, Some(p), CSSPropertyID::kTextShadow, "initial").unwrap();
        assert!(child.BoxShadow().is_null() && child.TextShadow().is_null());
        apply(&mut child, Some(p), CSSPropertyID::kBoxShadow, "none").unwrap();
        apply(&mut child, Some(p), CSSPropertyID::kTextShadow, "none").unwrap();
        assert!(child.BoxShadow().is_null() && child.TextShadow().is_null());
        let retained = foundation::Persistent::from_ptr(child.TakeStyle() as *mut ComputedStyle);
        drop(heap);
        foundation::CollectLayoutHeapForTesting();
        assert_eq!(
            unsafe { &*(*parent.Get()).BoxShadow() }.Shadows()[0].Spread(),
            -4.0
        );
        assert_eq!(
            unsafe { &*(*parent.Get()).TextShadow() }.Shadows()[0].X(),
            16.0
        );
        assert!(unsafe { &*retained.Get() }.BoxShadow().is_null());
    }
}

#[cfg(test)]
mod overflow_outline_production_tests {
    use super::*;
    fn declarations(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) {
        let parsed = crate::parser::production_property_parser::ParseDeclarationList(
            &foundation::String::from(css),
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        for p in parsed.properties {
            Apply(
                p.PropertyID(),
                b,
                parent,
                p.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    #[test]
    fn production_container_name_type_reach_native_style_and_inherit() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        declarations(
            &mut b,
            None,
            "container:Card card / anchored scroll-state size",
        );
        assert_eq!(b.ContainerType(), 15);
        let list = unsafe { &*b.ContainerName().Get() };
        let names = list
            .GetNames()
            .iter()
            .map(|name| unsafe { &*name.Get() })
            .collect::<Vec<_>>();
        assert_eq!(
            names.iter().map(|n| n.GetName().Utf8()).collect::<Vec<_>>(),
            vec!["Card", "card"]
        );
        assert!(names.iter().all(|n| n.GetTreeScope().is_null()));
        let p = unsafe { &*b.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(initial);
        declarations(
            &mut child,
            Some(p),
            "container-name:inherit;container-type:inherit",
        );
        assert_eq!(child.ContainerType(), 15);
        assert_eq!(child.ContainerName().Get(), p.ContainerName().Get());
        assert!(child.HasExplicitInheritance());
        declarations(&mut child, None, "container:none / inline-size");
        assert!(child.ContainerName().Get().is_null());
        assert_eq!(child.ContainerType(), 1);
        declarations(&mut child, Some(p), "container:initial");
        assert!(child.ContainerName().Get().is_null());
        assert_eq!(child.ContainerType(), 0);
        declarations(
            &mut child,
            None,
            "container-name:inherit;container-type:inherit",
        );
        assert!(child.ContainerName().Get().is_null());
        assert_eq!(child.ContainerType(), 0);
    }
    #[test]
    fn overflow_outline_integer_native_storage_initial_inherit_and_value() {
        use foundation::{EBorderStyle, EOverflow, LayoutUnit};
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        declarations(
            &mut b,
            None,
            "overflow:overlay clip;z-index:-999999999999;outline:.25px auto rgba(10,20,30,.5);outline-offset:-.25px;object-position:right 10% bottom 20%",
        );
        assert_eq!(b.OverflowX(), EOverflow::kOverlay);
        assert_eq!(b.OverflowY(), EOverflow::kClip);
        assert!(!b.HasAutoZIndex());
        assert_eq!(b.ZIndex(), i32::MIN);
        assert_eq!(b.OutlineStyle(), EBorderStyle::kDotted);
        assert!(b.OutlineStyleIsAuto());
        assert_eq!(*b.OutlineWidth(), 1);
        assert_eq!(*b.OutlineOffset(), -1);
        assert!(!b.OutlineColor().IsCurrentColor());
        assert_eq!(b.ObjectPosition().X().PercentValue(), 90.0);
        assert_eq!(b.ObjectPosition().Y().PercentValue(), 80.0);
        let p = unsafe { &*b.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(initial);
        declarations(
            &mut child,
            Some(p),
            "overflow-x:inherit;overflow-y:inherit;z-index:inherit;outline-color:inherit;outline-style:inherit;outline-width:inherit;outline-offset:inherit;object-position:inherit",
        );
        assert_eq!(child.OverflowX(), EOverflow::kOverlay);
        assert_eq!(child.OverflowY(), EOverflow::kClip);
        assert_eq!(child.ZIndex(), i32::MIN);
        assert!(!child.HasAutoZIndex());
        assert_eq!(child.OutlineStyle(), EBorderStyle::kDotted);
        assert!(child.OutlineStyleIsAuto());
        assert!(child.OutlineColor() == p.OutlineColor());
        assert_eq!(child.OutlineWidth(), p.OutlineWidth());
        assert_eq!(child.OutlineOffset(), p.OutlineOffset());
        assert!(child.ObjectPosition() == p.ObjectPosition());
        assert!(child.HasExplicitInheritance());
        declarations(
            &mut child,
            Some(p),
            "overflow:initial;z-index:initial;outline:initial;outline-offset:initial;object-position:initial",
        );
        assert_eq!(child.OverflowX(), EOverflow::kVisible);
        assert_eq!(child.OverflowY(), EOverflow::kVisible);
        assert!(child.HasExplicitOverflowXVisible());
        assert!(child.HasExplicitOverflowYVisible());
        assert!(child.HasAutoZIndex());
        assert_eq!(child.ZIndex(), 0);
        assert_eq!(child.OutlineStyle(), EBorderStyle::kNone);
        assert!(!child.OutlineStyleIsAuto());
        assert!(child.OutlineColor().IsCurrentColor());
        assert_eq!(*child.OutlineWidth(), 3);
        assert_eq!(*child.OutlineOffset(), 0);
        assert!(child.ObjectPosition() == initial.ObjectPosition());
        declarations(
            &mut child,
            None,
            "overflow-x:hidden;z-index:999999999999;outline-style:solid;outline-offset:-1.9px;outline-width:999999999999px",
        );
        assert!(child.HasExplicitOverflowXVisible());
        assert_eq!(child.ZIndex(), i32::MAX);
        assert!(!child.OutlineStyleIsAuto());
        assert_eq!(*child.OutlineOffset(), -1);
        assert_eq!(*child.OutlineWidth(), LayoutUnit::Max().ToInt());
        declarations(
            &mut child,
            None,
            "z-index:auto;outline-color:currentcolor;outline-width:thick",
        );
        assert!(child.HasAutoZIndex());
        assert_eq!(child.ZIndex(), 0);
        assert_eq!(*child.OutlineWidth(), 5);
        for value in ["-webkit-focus-ring-color", "canvastext"] {
            let parsed = crate::parser::production_property_parser::ParseProperty(
                CSSPropertyID::kOutlineColor,
                &foundation::String::from(value),
                false,
                crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            assert_eq!(
                Apply(
                    CSSPropertyID::kOutlineColor,
                    &mut child,
                    None,
                    parsed[0].Value(),
                    16.0,
                    &MediaValuesCachedData::default()
                ),
                Err(LonghandApplicationError::Unsupported(
                    CSSPropertyID::kOutlineColor
                ))
            );
            assert!(child.OutlineColor().IsCurrentColor());
        }
    }
}

#[cfg(test)]
mod font_settings_production_tests {
    use super::*;
    fn declarations(builder: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) {
        let parsed = crate::parser::production_property_parser::ParseDeclarationList(
            &foundation::String::from(css),
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        for property in parsed.properties {
            Apply(
                property.PropertyID(),
                builder,
                parent,
                property.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    #[test]
    fn opentype_font_settings_reach_native_style_preserve_last_tags_and_inherit() {
        use font_engine::fonts::{font_description::Kerning, font_optical_sizing::OpticalSizing};
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        declarations(
            &mut builder,
            None,
            "font-feature-settings:'liga' 0,'kern' 1,'liga' 4;font-variation-settings:'wght' 200,'wdth' 87.5,'wght' 620.5;font-stretch:semi-condensed;font-kerning:none;font-optical-sizing:none",
        );
        let font = builder.GetFontDescription();
        let features = font.FeatureSettings().unwrap();
        assert_eq!(
            features
                .iter()
                .map(|value| (value.Tag(), value.Value()))
                .collect::<Vec<_>>(),
            vec![
                (u32::from_be_bytes(*b"kern"), 1),
                (u32::from_be_bytes(*b"liga"), 4)
            ]
        );
        let axes = font.VariationSettings().unwrap();
        assert_eq!(
            axes.iter()
                .map(|value| (value.Tag(), value.Value()))
                .collect::<Vec<_>>(),
            vec![
                (u32::from_be_bytes(*b"wdth"), 87.5),
                (u32::from_be_bytes(*b"wght"), 620.5)
            ]
        );
        assert_eq!(font.Stretch().ToFloat(), 87.5);
        assert_eq!(font.GetKerning(), Kerning::kNoneKerning);
        assert_eq!(font.FontOpticalSizing(), OpticalSizing::kNoneOpticalSizing);
        let parent = unsafe { &*builder.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(initial);
        declarations(
            &mut child,
            Some(parent),
            "font-feature-settings:inherit;font-variation-settings:inherit;font-stretch:inherit;font-kerning:inherit;font-optical-sizing:inherit",
        );
        assert!(child.HasExplicitInheritance());
        let font = child.GetFontDescription();
        assert!(std::sync::Arc::ptr_eq(
            font.FeatureSettings().unwrap(),
            parent.GetFontDescription().FeatureSettings().unwrap()
        ));
        assert!(std::sync::Arc::ptr_eq(
            font.VariationSettings().unwrap(),
            parent.GetFontDescription().VariationSettings().unwrap()
        ));
        assert_eq!(font.Stretch(), parent.GetFontDescription().Stretch());
        assert_eq!(font.GetKerning(), Kerning::kNoneKerning);
        assert_eq!(font.FontOpticalSizing(), OpticalSizing::kNoneOpticalSizing);
        declarations(
            &mut child,
            None,
            "font-feature-settings:normal;font-variation-settings:normal;font-stretch:125.5%;font-kerning:normal;font-optical-sizing:auto",
        );
        assert_eq!(
            child.GetFontDescription().FeatureSettings().unwrap().size(),
            0
        );
        assert!(child.GetFontDescription().VariationSettings().is_none());
        assert_eq!(child.GetFontDescription().Stretch().ToFloat(), 125.5);
        assert_eq!(
            child.GetFontDescription().GetKerning(),
            Kerning::kNormalKerning
        );
        assert_eq!(
            child.GetFontDescription().FontOpticalSizing(),
            OpticalSizing::kAutoOpticalSizing
        );
        declarations(
            &mut child,
            None,
            "font-feature-settings:initial;font-variation-settings:initial;font-stretch:initial;font-kerning:initial;font-optical-sizing:initial",
        );
        assert!(child.GetFontDescription().FeatureSettings().is_none());
        assert!(child.GetFontDescription().VariationSettings().is_none());
        assert_eq!(child.GetFontDescription().Stretch().ToFloat(), 100.0);
        assert_eq!(
            child.GetFontDescription().GetKerning(),
            Kerning::kAutoKerning
        );
        assert_eq!(
            child.GetFontDescription().FontOpticalSizing(),
            OpticalSizing::kAutoOpticalSizing
        );
    }
}
