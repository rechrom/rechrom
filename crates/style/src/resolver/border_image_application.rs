// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium native NinePieceImage Apply* branches with real image bindings.
#![allow(non_snake_case)]
use super::*;
use crate::{
    css_math_expression_node::CalculationResultCategory as C,
    production_css_value::{CSSQuadValue, ListSeparator},
    resolver::css_to_style_map::{
        ApplyLegacyBorderImageWidths, BorderImageSliceSideFromResolved,
        MapNinePieceImageRepeatFromIdentifiers, NinePieceImageQuadFromSides,
    },
};
use foundation::LengthBox;
use layoutng_style::style::{
    border_image_length::BorderImageLength,
    border_image_length_box::BorderImageLengthBox,
    nine_piece_image::{ENinePieceImageRule, NinePieceImage},
};

pub(super) fn IsBorderImageProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kBorderImageSource
            | kBorderImageSlice
            | kBorderImageWidth
            | kBorderImageOutset
            | kBorderImageRepeat
            | kWebkitBorderImage
            | kWebkitMaskBoxImageSource
            | kWebkitMaskBoxImageSlice
            | kWebkitMaskBoxImageWidth
            | kWebkitMaskBoxImageOutset
            | kWebkitMaskBoxImageRepeat
    )
}
fn Number(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    percent: bool,
) -> std::result::Result<f64, LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(n)
            if if percent {
                n.GetType() == UnitType::kPercentage
            } else {
                matches!(n.GetType(), UnitType::kNumber | UnitType::kInteger)
            } =>
        {
            // css_numeric_literal_value.cc:142-153, ComputeNumber/Percentage.
            Ok(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampDouble(
                    n.DoubleValue(),
                ),
            )
        }
        CSSValuePayload::kMathFunctionClass(m)
            if m.Category() == if percent { C::Percent } else { C::Number } =>
        {
            m.ComputeValue(
                &mut MathLengthResolver(
                    id,
                    b.GetFontDescription().ComputedSize(),
                    root,
                    b.EffectiveZoom(),
                    media,
                ),
                None,
            )
            .map(crate::css_value_clamping_utils::CSSValueClampingUtils::ClampDouble)
            .map_err(|_| LonghandApplicationError::Unsupported(id))
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
fn QuadValue(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<&CSSQuadValue, LonghandApplicationError> {
    if let CSSValuePayload::kQuadClass(q) = v.Payload() {
        Ok(q)
    } else {
        Err(LonghandApplicationError::InvalidValue(id))
    }
}
// css_to_style_map.cc:617-651. Native resolved-side conversion is shared with
// the generic map, including the source round() on number slices.
fn MapSlice(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    image: &mut NinePieceImage,
) -> Result {
    let CSSValuePayload::kBorderImageSliceClass(slice) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    let quad = QuadValue(id, &slice.slices)?;
    let sides=quad.sides.iter().map(|v| {
        let percent=matches!(v.Payload(),CSSValuePayload::kNumericLiteralClass(n) if n.GetType()==UnitType::kPercentage)||matches!(v.Payload(),CSSValuePayload::kMathFunctionClass(m) if m.Category()==C::Percent);
        Number(id,b,v,root,media,percent).map(|n|BorderImageSliceSideFromResolved(n,percent))
    }).collect::<std::result::Result<Vec<_>,_>>()?;
    image.SetImageSlices(&LengthBox::new(
        sides[0].clone(),
        sides[1].clone(),
        sides[2].clone(),
        sides[3].clone(),
    ));
    image.SetFill(slice.fill);
    Ok(())
}
// css_to_style_map.cc:654-681. Number retains the native multiplier tag;
// lengths and percentages retain their native Length rather than CSS strings.
fn MapQuad(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<BorderImageLengthBox, LonghandApplicationError> {
    let q = QuadValue(id, v)?;
    let sides=q.sides.iter().map(|v| {
        if matches!(v.Payload(),CSSValuePayload::kNumericLiteralClass(n) if matches!(n.GetType(),UnitType::kNumber|UnitType::kInteger))||matches!(v.Payload(),CSSValuePayload::kMathFunctionClass(m) if m.Category()==C::Number) {
            Number(id,b,v,root,media,false).map(BorderImageLength::from_number)
        } else if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kAuto) {
            Ok(BorderImageLength::from_length(Length::Auto()))
        } else {
            crate::resolver::position_repeat_application::ConvertLength(id,v,b,root,media).map(|l|BorderImageLength::from_length(&l))
        }
    }).collect::<std::result::Result<Vec<_>,_>>()?;
    Ok(NinePieceImageQuadFromSides(&[
        sides[0].clone(),
        sides[1].clone(),
        sides[2].clone(),
        sides[3].clone(),
    ]))
}
fn MapRepeat(id: CSSPropertyID, v: &Value, image: &mut NinePieceImage) -> Result {
    let (first, second) = if let CSSValuePayload::kValuePairClass(pair) = v.Payload() {
        (Identifier(id, &pair.first)?, Identifier(id, &pair.second)?)
    } else {
        let k = Identifier(id, v)?;
        (k, k)
    };
    for k in [first, second] {
        if !matches!(
            k,
            CSSValueID::kStretch | CSSValueID::kRound | CSSValueID::kSpace | CSSValueID::kRepeat
        ) {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
    }
    MapNinePieceImageRepeatFromIdentifiers(first, second, image);
    Ok(())
}
// css_to_style_map.cc:537-615. The nested slash list is the actual legacy
// value structure; source native-width side effects are committed last.
pub(super) fn MapNinePieceImage(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    images: Option<&dyn URLImageResolver>,
    mut image: NinePieceImage,
) -> std::result::Result<NinePieceImage, LonghandApplicationError> {
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNone) {
        return Ok(image);
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != ListSeparator::Space || list.values.is_empty() {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    for value in &list.values {
        match value.Payload() {
            CSSValuePayload::kImageClass(_) | CSSValuePayload::kLinearGradientClass(_) => image
                .SetImage(ResolveStyleImage(
                    if id == CSSPropertyID::kWebkitBorderImage {
                        CSSPropertyID::kBorderImageSource
                    } else if id == CSSPropertyID::kWebkitMaskBoxImage {
                        CSSPropertyID::kWebkitMaskBoxImageSource
                    } else {
                        id
                    },
                    value,
                    images,
                )?),
            CSSValuePayload::kBorderImageSliceClass(_) => {
                MapSlice(id, b, value, root, media, &mut image)?
            }
            CSSValuePayload::kValueListClass(slash) if slash.separator == ListSeparator::Slash => {
                if let Some(first) = slash.values.first() {
                    if matches!(first.Payload(), CSSValuePayload::kBorderImageSliceClass(_)) {
                        MapSlice(id, b, first, root, media, &mut image)?;
                    }
                }
                if let Some(width) = slash.values.get(1) {
                    image.SetBorderSlices(&MapQuad(id, b, width, root, media)?);
                }
                if let Some(outset) = slash.values.get(2) {
                    image.SetOutset(&MapQuad(id, b, outset, root, media)?);
                }
            }
            CSSValuePayload::kValuePairClass(_) => MapRepeat(id, value, &mut image)?,
            CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone => {}
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }
    }
    Ok(image)
}
fn MapLegacy(
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    images: Option<&dyn URLImageResolver>,
) -> std::result::Result<NinePieceImage, LonghandApplicationError> {
    MapNinePieceImage(
        CSSPropertyID::kWebkitBorderImage,
        b,
        v,
        root,
        media,
        images,
        NinePieceImage::new(),
    )
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    images: Option<&dyn URLImageResolver>,
) -> Result {
    use CSSPropertyID::*;
    let mask = matches!(
        id,
        kWebkitMaskBoxImageSource
            | kWebkitMaskBoxImageSlice
            | kWebkitMaskBoxImageWidth
            | kWebkitMaskBoxImageOutset
            | kWebkitMaskBoxImageRepeat
    );
    let default_image = if mask {
        NinePieceImage::MaskDefaults()
    } else {
        ComputedStyleInitialValues::InitialBorderImage()
    };
    let field = match id {
        kWebkitMaskBoxImageSource => kBorderImageSource,
        kWebkitMaskBoxImageSlice => kBorderImageSlice,
        kWebkitMaskBoxImageWidth => kBorderImageWidth,
        kWebkitMaskBoxImageOutset => kBorderImageOutset,
        kWebkitMaskBoxImageRepeat => kBorderImageRepeat,
        _ => id,
    };
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    if field == kBorderImageSource {
        let image = if initial {
            std::ptr::null_mut()
        } else if inherit {
            if mask {
                parent.unwrap().MaskBoxImage().GetImage()
            } else {
                parent.unwrap().BorderImageSource()
            }
        } else {
            ResolveStyleImage(id, v, images)?
        };
        if mask {
            b.SetMaskBoxImageSource(image);
        } else {
            b.SetBorderImageSource(image);
        }
    } else if id == kWebkitBorderImage {
        if initial {
            b.SetBorderImageOwned(ComputedStyleInitialValues::InitialBorderImage());
        } else if inherit {
            b.SetBorderImage(parent.unwrap().BorderImage());
        } else {
            let image = MapLegacy(b, v, root, media, images)?;
            ApplyLegacyBorderImageWidths(&image, b);
            b.SetBorderImageOwned(image);
        }
    } else {
        if inherit
            && !initial
            && matches!(field, kBorderImageWidth | kBorderImageOutset)
            && b.EffectiveZoom() != parent.unwrap().EffectiveZoom()
        {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        // generated mask-box ApplyInitial uses mask defaults (zero slices,
        // fill, auto width) while preserving all other NinePieceImage fields.
        if initial {
            let current = if mask {
                b.MaskBoxImage()
            } else {
                b.BorderImage()
            };
            let unchanged = match field {
                kBorderImageSlice => {
                    current.Fill() == default_image.Fill()
                        && current.ImageSlices() == default_image.ImageSlices()
                }
                kBorderImageWidth => current.BorderSlices() == default_image.BorderSlices(),
                kBorderImageOutset => current.Outset() == default_image.Outset(),
                kBorderImageRepeat => {
                    current.HorizontalRule() == default_image.HorizontalRule()
                        && current.VerticalRule() == default_image.VerticalRule()
                }
                _ => false,
            };
            if unchanged {
                return Ok(());
            }
        }
        let mut image = if mask {
            b.MaskBoxImage()
        } else {
            b.BorderImage()
        }
        .clone();
        let parent_image = parent.map(|p| {
            if mask {
                p.MaskBoxImage()
            } else {
                p.BorderImage()
            }
        });
        match field {
            kBorderImageSlice => {
                if initial {
                    image.CopyImageSlicesFrom(&default_image);
                } else if inherit {
                    image.CopyImageSlicesFrom(parent_image.unwrap());
                } else {
                    MapSlice(id, b, v, root, media, &mut image)?;
                }
            }
            kBorderImageWidth => {
                if initial {
                    image.CopyBorderSlicesFrom(&default_image);
                } else if inherit {
                    image.CopyBorderSlicesFrom(parent_image.unwrap());
                } else {
                    image.SetBorderSlices(&MapQuad(id, b, v, root, media)?);
                }
            }
            kBorderImageOutset => {
                if initial {
                    image.CopyOutsetFrom(&default_image);
                } else if inherit {
                    image.CopyOutsetFrom(parent_image.unwrap());
                } else {
                    image.SetOutset(&MapQuad(id, b, v, root, media)?);
                }
            }
            kBorderImageRepeat => {
                if initial {
                    image.SetHorizontalRule(ENinePieceImageRule::kStretchImageRule);
                    image.SetVerticalRule(ENinePieceImageRule::kStretchImageRule);
                } else if inherit {
                    image.CopyRepeatFrom(parent_image.unwrap());
                } else {
                    MapRepeat(id, v, &mut image)?;
                }
            }
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
        if mask {
            b.SetMaskBoxImage(&image);
        } else {
            b.SetBorderImageOwned(image);
        }
    }
    if inherit && !initial && v.IsInheritedValue() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty, PropertyParseErrorKind},
    };
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) {
        let parsed = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert!(!parsed.properties.is_empty());
        for p in parsed.properties {
            super::super::Apply(
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
    fn snapshot(b: &ComputedStyleBuilder) -> &ComputedStyle {
        unsafe { &*b.CloneStyle() }
    }
    #[test]
    fn production_border_image_slice_fill_quad_rounding_and_typed_math() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b, None, "border-image-slice:fill 2.5 10% 4 20%");
        assert!(b.BorderImage().Fill());
        let slices = b.BorderImage().ImageSlices();
        assert_eq!(slices.Top().Pixels(), 3.0);
        assert_eq!(slices.Right().PercentValue(), 10.0);
        assert_eq!(slices.Bottom().Pixels(), 4.0);
        assert_eq!(slices.Left().PercentValue(), 20.0);
        let parsed = ParseProperty(
            CSSPropertyID::kBorderImageSlice,
            &foundation::String::from("fill 2.5 10% 4 20%"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(parsed[0].Value().CssText().Utf8(), "2.5 10% 4 20% fill");
        apply(
            &mut b,
            None,
            "border-image-slice:calc(3 + 2) calc(20% + 10%)",
        );
        assert!(!b.BorderImage().Fill());
        assert_eq!(b.BorderImage().ImageSlices().Top().Pixels(), 5.0);
        assert_eq!(b.BorderImage().ImageSlices().Right().PercentValue(), 30.0);
        assert_eq!(b.BorderImage().ImageSlices().Bottom().Pixels(), 5.0);
        apply(&mut b, None, "border-image-slice:calc(-3)");
        assert_eq!(b.BorderImage().ImageSlices().Top().Pixels(), 0.0);
        apply(
            &mut b,
            None,
            "border-image-slice:1e999;border-image-width:1e999",
        );
        assert!(b.BorderImage().ImageSlices().Top().Pixels().is_finite());
        assert!(b.BorderImage().BorderSlices().Top().Number().is_finite());
    }
    #[test]
    fn production_border_image_width_outset_repeat_native_tags_and_preservation() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"border-image-slice:15 fill;border-image-width:2 3px 25% auto;border-image-outset:1 4px;border-image-repeat:round space");
        let image = b.BorderImage();
        let width = image.BorderSlices();
        assert!(width.Top().IsNumber());
        assert_eq!(width.Top().Number(), 2.0);
        assert!(width.Right().IsLength());
        assert_eq!(width.Right().length().Pixels(), 3.0);
        assert_eq!(width.Bottom().length().PercentValue(), 25.0);
        assert!(width.Left().length().IsAuto());
        assert_eq!(image.Outset().Top().Number(), 1.0);
        assert_eq!(image.Outset().Right().length().Pixels(), 4.0);
        assert!(image.HorizontalRule() == ENinePieceImageRule::kRoundImageRule);
        assert!(image.VerticalRule() == ENinePieceImageRule::kSpaceImageRule);
        assert!(image.Fill());
        assert_eq!(image.ImageSlices().Top().Pixels(), 15.0);
        apply(
            &mut b,
            None,
            "border-image-width:calc(1 + 2) calc(2px + 3px);border-image-repeat:repeat",
        );
        assert_eq!(b.BorderImage().BorderSlices().Top().Number(), 3.0);
        assert_eq!(
            b.BorderImage().BorderSlices().Right().length().Pixels(),
            5.0
        );
        assert!(b.BorderImage().HorizontalRule() == ENinePieceImageRule::kRepeatImageRule);
        assert!(b.BorderImage().VerticalRule() == ENinePieceImageRule::kRepeatImageRule);
        assert_eq!(b.BorderImage().Outset().Right().length().Pixels(), 4.0);
    }
    #[test]
    fn production_border_image_shorthand_gradient_and_all_component_resets() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            "border-image:round space linear-gradient(to bottom, red, blue) 20 fill / 2 / 3px",
        );
        let image = b.BorderImage();
        assert!(image.HasImage());
        let style_image = unsafe { &*image.GetImage() };
        assert!(style_image.IsGeneratedImage());
        assert!(!style_image.IsImageResource());
        assert!(style_image.CanRender());
        assert!(image.Fill());
        assert_eq!(image.ImageSlices().Top().Pixels(), 20.0);
        assert_eq!(image.BorderSlices().Top().Number(), 2.0);
        assert_eq!(image.Outset().Top().length().Pixels(), 3.0);
        assert!(image.HorizontalRule() == ENinePieceImageRule::kRoundImageRule);
        assert!(image.VerticalRule() == ENinePieceImageRule::kSpaceImageRule);
        let ptr = image.GetImage();
        apply(&mut b, None, "border-image-repeat:stretch");
        assert_eq!(b.BorderImage().GetImage(), ptr);
        apply(&mut b, None, "border-image:stretch");
        let image = b.BorderImage();
        assert!(!image.HasImage());
        assert!(!image.Fill());
        assert_eq!(image.ImageSlices().Top().PercentValue(), 100.0);
        assert_eq!(image.BorderSlices().Top().Number(), 1.0);
        assert_eq!(image.Outset().Top().Number(), 0.0);
        assert!(image.HorizontalRule() == ENinePieceImageRule::kStretchImageRule);
        assert!(image.VerticalRule() == ENinePieceImageRule::kStretchImageRule);
        apply(&mut b, None, "border-image:20 // 4");
        assert_eq!(b.BorderImage().BorderSlices().Top().Number(), 1.0);
        assert_eq!(b.BorderImage().Outset().Top().Number(), 4.0);
    }
    #[test]
    fn production_border_image_webkit_legacy_fill_fixed_border_width_side_effects_and_css_wide() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut parent,
            None,
            "-webkit-border-image:linear-gradient(red,blue) 12 / 3px 4px 5px 6px / 2 round repeat",
        );
        assert!(parent.BorderImage().Fill());
        assert!(parent.BorderImage().HasImage());
        assert_eq!(*parent.SpecifiedBorderTopWidth(), 3);
        assert_eq!(*parent.SpecifiedBorderRightWidth(), 4);
        assert_eq!(*parent.SpecifiedBorderBottomWidth(), 5);
        assert_eq!(*parent.SpecifiedBorderLeftWidth(), 6);
        let p = snapshot(&parent);
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(&mut child, Some(p), "border-image:inherit");
        assert!(child.BorderImage() == p.BorderImage());
        assert!(child.HasExplicitInheritance());
        apply(&mut child, None, "border-image:unset");
        assert!(!child.BorderImage().HasImage());
        assert!(!child.BorderImage().Fill());
        apply(&mut child, Some(p), "-webkit-border-image:inherit");
        assert!(child.BorderImage() == p.BorderImage());
        let prior = *child.SpecifiedBorderTopWidth();
        apply(&mut child, Some(p), "-webkit-border-image:initial");
        assert!(!child.BorderImage().HasImage());
        assert!(!child.BorderImage().Fill());
        assert_eq!(*child.SpecifiedBorderTopWidth(), prior);
        apply(&mut child, None, "-webkit-border-image:10 / 2 25% auto 8px");
        assert!(child.BorderImage().Fill());
        assert_eq!(*child.SpecifiedBorderLeftWidth(), 8);
        assert_eq!(*child.SpecifiedBorderTopWidth(), prior);
        apply(&mut child, None, "border-image:inherit");
        assert!(child.BorderImage() == &NinePieceImage::new());
        apply(&mut child, None, "-webkit-border-image:inherit");
        assert!(child.BorderImage() == &NinePieceImage::new());
    }
    #[test]
    fn production_border_image_invalid_values_and_real_url_binding_boundary() {
        let _heap = foundation::LayoutHeapScope::new();
        for (id, texts) in [
            (
                CSSPropertyID::kBorderImageSlice,
                vec![
                    "fill",
                    "-1",
                    "10 fill fill",
                    "fill 10 fill",
                    "1 2 3 4 5",
                    "auto",
                    "5px",
                ],
            ),
            (
                CSSPropertyID::kBorderImageWidth,
                vec!["-1", "1 2 3 4 5", "fill"],
            ),
            (
                CSSPropertyID::kBorderImageOutset,
                vec!["2%", "auto", "-2px"],
            ),
            (
                CSSPropertyID::kBorderImageRepeat,
                vec!["no-repeat", "round space repeat", "none"],
            ),
            (
                CSSPropertyID::kBorderImage,
                vec![
                    "/ 2",
                    "20 /",
                    "20 //",
                    "20 / auto / auto",
                    "none none",
                    "20 30 fill 40",
                    "stretch round repeat",
                ],
            ),
        ] {
            for text in texts {
                let error = ParseProperty(
                    id,
                    &foundation::String::from(text),
                    false,
                    CSSParserMode::kHTMLStandardMode,
                )
                .err()
                .expect(text);
                assert_eq!(
                    error.kind,
                    PropertyParseErrorKind::Invalid,
                    "{id:?}: {text}"
                );
            }
        }
        let url = ParseProperty(
            CSSPropertyID::kBorderImageSource,
            &foundation::String::from("url(border.png)"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            "border-image-source:linear-gradient(red,blue)",
        );
        let before = b.BorderImage().GetImage();
        assert!(matches!(
            super::super::Apply(
                CSSPropertyID::kBorderImageSource,
                &mut b,
                None,
                url[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kBorderImageSource
            ))
        ));
        assert_eq!(b.BorderImage().GetImage(), before);
        struct Unbound(std::cell::Cell<bool>);
        impl URLImageResolver for Unbound {
            fn ResolveImage(
                &self,
                _: &crate::production_css_value::CSSImageValue,
            ) -> std::result::Result<FetchedImageBinding, LonghandApplicationError> {
                self.0.set(true);
                Err(LonghandApplicationError::Unsupported(
                    CSSPropertyID::kBackgroundImage,
                ))
            }
        }
        let resolver = Unbound(std::cell::Cell::new(false));
        assert!(super::super::ApplyWithImageResolver(
            CSSPropertyID::kBorderImageSource,
            &mut b,
            None,
            url[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
            &resolver
        )
        .is_err());
        assert!(resolver.0.get());
        assert_eq!(b.BorderImage().GetImage(), before);
    }
}
