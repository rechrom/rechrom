// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! StyleBuilderConverter::ConvertBoxReflect into actual StyleReflection.
#![allow(non_snake_case)]
use super::*;
use foundation::{MakeGarbageCollected, Member};
use layoutng_style::{
    css::css_reflection_direction::CSSReflectionDirection as D,
    style::{nine_piece_image::NinePieceImage, style_reflection::StyleReflection},
};
pub(super) fn Apply(
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    images: Option<&dyn URLImageResolver>,
) -> Result {
    let id = CSSPropertyID::kWebkitBoxReflect;
    let inherit = v.IsInheritedValue();
    let initial = v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none();
    let reflection = if initial {
        Member::default()
    } else if inherit {
        let p = parent.unwrap();
        if p.EffectiveZoom() != b.EffectiveZoom() {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        Member::from_ptr(p.BoxReflect())
    } else if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(i) if i.0==CSSValueID::kNone) {
        Member::default()
    } else {
        let CSSValuePayload::kReflectClass(v) = v.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        let mut result = StyleReflection::default();
        result.SetDirection(match Identifier(id, &v.direction)? {
            CSSValueID::kBelow => D::kReflectionBelow,
            CSSValueID::kAbove => D::kReflectionAbove,
            CSSValueID::kLeft => D::kReflectionLeft,
            CSSValueID::kRight => D::kReflectionRight,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        });
        result.SetOffset(&text_application::ConvertLength(
            id, b, &v.offset, root, media,
        )?);
        if let Some(mask) = &v.mask {
            result.SetMask(&border_image_application::MapNinePieceImage(
                id,
                b,
                mask,
                root,
                media,
                images,
                NinePieceImage::MaskDefaults(),
            )?);
        }
        Member::from_ptr(MakeGarbageCollected(result))
    };
    b.SetBoxReflect(reflection);
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
