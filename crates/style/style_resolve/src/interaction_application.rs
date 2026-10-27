// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native Chromium interaction/render-hint application. No CSS text is parsed.
#![allow(non_snake_case)]
use super::*;
use crate::{
    css_property_names::{FindProperty, ResolveCSSPropertyID},
    production_css_value::ListSeparator,
};
use foundation::{
    AtomicString, CSSBitset, HangingPunctuation, MakeGarbageCollected, Member,
    RespectImageOrientationEnum, TouchAction,
};
use layoutng_style::style::{
    appearance::AppearanceValue, computed_style_constants::EMarginTrim,
    style_will_change_data::StyleWillChangeData,
};

pub(super) fn IsInteractionProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTouchAction
            | kWillChange
            | kAppearance
            | kImageOrientation
            | kHangingPunctuation
            | kMarginTrim
    )
}
fn Flags(
    id: CSSPropertyID,
    v: &Value,
    map: impl Fn(CSSValueID) -> Option<u32>,
) -> std::result::Result<u32, LonghandApplicationError> {
    if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone) {
        return Ok(0);
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != ListSeparator::Space || list.values.is_empty() {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let bits = list
        .values
        .iter()
        .map(|v| map(Identifier(id, v)?).ok_or(LonghandApplicationError::InvalidValue(id)))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(crate::resolver::style_builder_converter::StyleBuilderConverter::ConvertFlagsFromBits(bits))
}
// css_identifier_value_mappings.h:285-323, exactly the source native enum.
fn Appearance(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<AppearanceValue, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, v)? {
        kNone => AppearanceValue::kNone,
        kAuto => AppearanceValue::kAuto,
        kCheckbox => AppearanceValue::kCheckbox,
        kRadio => AppearanceValue::kRadio,
        kButton => AppearanceValue::kButton,
        kListbox => AppearanceValue::kListbox,
        kInternalMediaControl => AppearanceValue::kMediaControl,
        kMenulist => AppearanceValue::kMenulist,
        kMenulistButton => AppearanceValue::kMenulistButton,
        kMeter => AppearanceValue::kMeter,
        kProgressBar => AppearanceValue::kProgressBar,
        kSliderVertical => AppearanceValue::kSliderVertical,
        kSearchfield => AppearanceValue::kSearchField,
        kTextfield => AppearanceValue::kTextField,
        kTextarea => AppearanceValue::kTextArea,
        kBaseSelect => AppearanceValue::kBaseSelect,
        kBase if foundation::RuntimeEnabledFeatures::AppearanceBaseEnabled() => {
            AppearanceValue::kBase
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
// longhands_custom.cc:12504-12591. Lookup only supplies name identity/exposure;
// shorthand expansion and all native transform flags use real property IDs.
fn WillChange(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, v: &Value) -> Result {
    use CSSPropertyID::*;
    let id = kWillChange;
    let ancestor_contents = parent.is_some_and(|p| p.SubtreeWillChangeContents());
    if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kAuto) {
        b.SetWillChange(Member::default());
        b.SetSubtreeWillChangeContents(ancestor_contents);
        return Ok(());
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != ListSeparator::Comma || list.values.is_empty() {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut names = Vec::with_capacity(list.values.len());
    let mut ids = CSSBitset::new();
    let mut contents = false;
    let mut scroll = false;
    let mut transform = false;
    let mut any_transform = false;
    let mut process = |longhand| {
        ids.Set(longhand);
        transform |= matches!(longhand, kTransform | kPerspective | kTransformStyle);
        any_transform |= matches!(
            longhand,
            kTransform
                | kPerspective
                | kTransformStyle
                | kTranslate
                | kScale
                | kRotate
                | kOffsetPath
                | kOffsetPosition
        );
    };
    for item in &list.values {
        match item.Payload() {
            CSSValuePayload::kIdentifierClass(k) => {
                match k.0 {
                    CSSValueID::kContents => contents = true,
                    CSSValueID::kScrollPosition => scroll = true,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
                names.push(AtomicString::from_str(
                    crate::css_value_keywords::GetCSSValueName(k.0),
                ));
            }
            CSSValuePayload::kCustomIdentClass(name) => {
                names.push(name.name.clone());
                if let Some(property) =
                    FindProperty(name.name.Utf8().to_ascii_lowercase().as_bytes())
                {
                    // ExecutionContext controls feature exposure for names with
                    // this generated bit. Never fabricate a resolved bitset.
                    if property.id_and_exposed_bit
                        & crate::css_property_names::kNotKnownExposedPropertyBit
                        != 0
                    {
                        return Err(LonghandApplicationError::Unsupported(id));
                    }
                    let resolved = ResolveCSSPropertyID(crate::parser::production_property_metadata::UnresolvedPropertyIDFromInteger(property.id_and_exposed_bit));
                    if !matches!(resolved, kInvalid | kVariable) {
                        let longhands =
                            crate::parser::production_property_metadata::ShorthandFor(resolved);
                        if longhands.is_empty() {
                            process(resolved);
                        } else {
                            for &longhand in longhands {
                                process(longhand);
                            }
                        }
                    }
                }
            }
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }
    }
    b.SetWillChange(Member::from_ptr(MakeGarbageCollected(
        StyleWillChangeData::new(names, ids, scroll, transform, any_transform),
    )));
    b.SetSubtreeWillChangeContents(contents || ancestor_contents);
    Ok(())
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    if !crate::production_interaction_features::IsExposed(id) {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    ApplyInternal(id, b, parent, v)
}
fn ApplyInternal(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    if initial {
        // This custom initial branch inherits the ancestor subtree flag.
        if id == kWillChange {
            b.SetWillChange(Member::default());
            b.SetSubtreeWillChangeContents(parent.is_some_and(|p| p.SubtreeWillChangeContents()));
            return Ok(());
        }
        return ApplyInitial(id, b);
    }
    if inherit {
        let p = parent.unwrap();
        ApplyInherit(id, b, p)?;
        if v.IsInheritedValue() {
            b.SetHasExplicitInheritance();
            p.SetChildHasExplicitInheritance();
        }
        return Ok(());
    }
    match id {
        // css_identifier_value_mappings.h:1203-1232; ConvertFlags:490-502.
        kTouchAction => {
            use CSSValueID::*;
            let bits = Flags(id, v, |k| {
                Some(
                    match k {
                        kNone => TouchAction::kNone,
                        kAuto => TouchAction::kAuto,
                        kPanLeft => TouchAction::kPanLeft,
                        kPanRight => TouchAction::kPanRight,
                        kPanX => TouchAction::kPanX,
                        kPanUp => TouchAction::kPanUp,
                        kPanDown => TouchAction::kPanDown,
                        kPanY => TouchAction::kPanY,
                        kPinchZoom => TouchAction::kPinchZoom,
                        kManipulation => TouchAction::kManipulation,
                        _ => return None,
                    }
                    .bits() as u32,
                )
            })?;
            b.SetTouchAction(TouchAction::from_bits(bits));
            Ok(())
        }
        kWillChange => WillChange(b, parent, v),
        kAppearance => {
            b.SetAppearance(Appearance(id, v)?);
            Ok(())
        }
        // style_builder_converter.cc:3441-3448.
        kImageOrientation => {
            let orientation = match Identifier(id, v)? {
                CSSValueID::kNone => RespectImageOrientationEnum::kDoNotRespectImageOrientation,
                CSSValueID::kFromImage => RespectImageOrientationEnum::kRespectImageOrientation,
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            };
            b.SetImageOrientation(orientation);
            Ok(())
        }
        // generated css_value_id_mappings_generated.h:1041-1053.
        kHangingPunctuation => {
            let bits = Flags(id, v, |k| match k {
                CSSValueID::kFirst => Some(HangingPunctuation::kFirst.bits()),
                CSSValueID::kLast => Some(HangingPunctuation::kLast.bits()),
                CSSValueID::kAllowEnd => Some(HangingPunctuation::kAllowEnd.bits()),
                _ => None,
            })?;
            b.SetHangingPunctuation(HangingPunctuation::from_bits(bits));
            Ok(())
        }
        // css_identifier_value_mappings.h:1689-1705.
        kMarginTrim => {
            let bits = Flags(id, v, |k| {
                Some(
                    match k {
                        CSSValueID::kBlock => EMarginTrim::kMarginTrimBlock,
                        CSSValueID::kBlockStart => EMarginTrim::kMarginTrimBlockStart,
                        CSSValueID::kBlockEnd => EMarginTrim::kMarginTrimBlockEnd,
                        _ => return None,
                    }
                    .value() as u32,
                )
            })?;
            b.SetMarginTrim(bits);
            Ok(())
        }
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
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
    fn invalid(id: CSSPropertyID, texts: &[&str]) {
        for text in texts {
            let result = ParseProperty(
                id,
                &foundation::String::from(*text),
                false,
                CSSParserMode::kHTMLStandardMode,
            );
            assert!(
                matches!(result, Err(e) if e.kind == PropertyParseErrorKind::Invalid),
                "{id:?}: {text}"
            );
        }
    }
    #[test]
    fn production_interaction_touch_action_combinations_and_native_css_wide() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut parent,
            None,
            "touch-action:pinch-zoom pan-down pan-left",
        );
        assert_eq!(
            snapshot(&parent).GetTouchAction(),
            TouchAction::kPinchZoom | TouchAction::kPanDown | TouchAction::kPanLeft
        );
        let parsed = ParseProperty(
            CSSPropertyID::kTouchAction,
            &foundation::String::from("pinch-zoom pan-down pan-left"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(
            parsed[0].Value().CssText().Utf8(),
            "pan-left pan-down pinch-zoom"
        );
        invalid(
            CSSPropertyID::kTouchAction,
            &[
                "pan-x pan-left",
                "pan-up pan-down",
                "pinch-zoom pinch-zoom",
                "none pan-x",
                "auto pinch-zoom",
                "pan-x pan-y pan-left",
                "",
            ],
        );
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(&mut child, Some(snapshot(&parent)), "touch-action:inherit");
        assert_eq!(
            snapshot(&child).GetTouchAction(),
            snapshot(&parent).GetTouchAction()
        );
        apply(&mut child, Some(snapshot(&parent)), "touch-action:unset");
        assert_eq!(snapshot(&child).GetTouchAction(), TouchAction::kAuto);
        apply(&mut child, None, "touch-action:manipulation");
        assert_eq!(
            snapshot(&child).GetTouchAction(),
            TouchAction::kManipulation
        );
        apply(&mut child, None, "touch-action:none");
        assert_eq!(snapshot(&child).GetTouchAction(), TouchAction::kNone);
        apply(&mut child, None, "touch-action:initial");
        assert_eq!(snapshot(&child).GetTouchAction(), TouchAction::kAuto);
    }
    #[test]
    fn production_interaction_will_change_preserves_names_expands_ids_and_propagates_subtree() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        apply(&mut parent, None, "will-change:contents,scroll-position,-webkit-transform,translate,margin,MiXeD,opacity,opacity");
        let data = unsafe { &*parent.WillChange() };
        assert_eq!(
            data.values()
                .iter()
                .map(AtomicString::Utf8)
                .collect::<Vec<_>>(),
            vec![
                "contents",
                "scroll-position",
                "-webkit-transform",
                "translate",
                "margin",
                "MiXeD",
                "opacity",
                "opacity"
            ]
        );
        assert!(data.has_scroll_position_value());
        assert!(data.has_transform_property());
        assert!(data.has_any_transform_property());
        for id in [
            CSSPropertyID::kTransform,
            CSSPropertyID::kTranslate,
            CSSPropertyID::kMarginTop,
            CSSPropertyID::kMarginRight,
            CSSPropertyID::kMarginBottom,
            CSSPropertyID::kMarginLeft,
            CSSPropertyID::kOpacity,
        ] {
            assert!(data.resolved_longhand_ids().Has(id));
        }
        assert!(parent.SubtreeWillChangeContents());
        let parent_style = snapshot(&parent);
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(&mut child, Some(parent_style), "will-change:inherit");
        assert_eq!(child.WillChange(), parent_style.WillChange());
        assert!(child.SubtreeWillChangeContents());
        for keyword in ["auto", "initial", "unset"] {
            apply(
                &mut child,
                Some(parent_style),
                &format!("will-change:{keyword}"),
            );
            assert!(child.WillChange().is_null());
            assert!(child.SubtreeWillChangeContents());
        }
        apply(
            &mut child,
            None,
            "will-change:translate, --custom, UnknownThing",
        );
        let data = unsafe { &*child.WillChange() };
        assert!(!data.has_transform_property());
        assert!(data.has_any_transform_property());
        assert!(!child.SubtreeWillChangeContents());
        invalid(
            CSSPropertyID::kWillChange,
            &[
                "none",
                "all",
                "will-change",
                "contents, auto",
                "opacity, default",
                "opacity, inherit",
                "opacity,",
                "\"opacity\"",
                "ident(opacity)",
                "contents scroll-position",
            ],
        );
        // Context-dependent property exposure remains a typed boundary and
        // rejects before mutating the native will-change data.
        let p = ParseProperty(
            CSSPropertyID::kWillChange,
            &foundation::String::from("margin-trim"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        let before = child.WillChange();
        assert!(matches!(
            super::super::Apply(
                CSSPropertyID::kWillChange,
                &mut child,
                None,
                p[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kWillChange
            ))
        ));
        assert_eq!(child.WillChange(), before);
    }
    #[test]
    fn production_interaction_appearance_orientation_keywords_modes_and_css_wide() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut parent,
            None,
            "appearance:base-select;image-orientation:none",
        );
        assert_eq!(parent.Appearance(), AppearanceValue::kBaseSelect);
        assert_eq!(
            parent.ImageOrientation(),
            RespectImageOrientationEnum::kDoNotRespectImageOrientation
        );
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut child,
            Some(snapshot(&parent)),
            "appearance:inherit;image-orientation:unset",
        );
        assert_eq!(child.Appearance(), AppearanceValue::kBaseSelect);
        assert_eq!(
            child.ImageOrientation(),
            RespectImageOrientationEnum::kDoNotRespectImageOrientation
        );
        apply(
            &mut child,
            Some(snapshot(&parent)),
            "appearance:unset;image-orientation:initial",
        );
        assert_eq!(child.Appearance(), AppearanceValue::kNone);
        assert_eq!(
            child.ImageOrientation(),
            RespectImageOrientationEnum::kRespectImageOrientation
        );
        apply(&mut child, None, "appearance:slider-vertical");
        assert_eq!(child.Appearance(), AppearanceValue::kSliderVertical);
        invalid(
            CSSPropertyID::kAppearance,
            &[
                "base",
                "push-button",
                "slider-horizontal",
                "auto button",
                "-internal-media-control",
            ],
        );
        invalid(
            CSSPropertyID::kImageOrientation,
            &["flip", "90deg", "from-image flip", "auto"],
        );
        let ua = ParseProperty(
            CSSPropertyID::kAppearance,
            &foundation::String::from("-internal-media-control"),
            false,
            CSSParserMode::kUASheetMode,
        )
        .unwrap();
        super::super::Apply(
            CSSPropertyID::kAppearance,
            &mut child,
            None,
            ua[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
        )
        .unwrap();
        assert_eq!(child.Appearance(), AppearanceValue::kMediaControl);
    }
    #[test]
    fn production_interaction_webkit_appearance_resolves_alias_to_native_field() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut parent,
            None,
            "appearance:button;-webkit-appearance:none",
        );
        assert_eq!(parent.Appearance(), AppearanceValue::kNone);
        let parsed = ParseProperty(
            CSSPropertyID::kAliasWebkitAppearance,
            &foundation::String::from("none"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(parsed[0].PropertyID(), CSSPropertyID::kAppearance);
        assert_eq!(parsed[0].Value().CssText().Utf8(), "none");
        apply(&mut parent, None, "-webkit-appearance:base-select");
        assert_eq!(parent.Appearance(), AppearanceValue::kBaseSelect);
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut child,
            Some(snapshot(&parent)),
            "-webkit-appearance:inherit",
        );
        assert_eq!(child.Appearance(), AppearanceValue::kBaseSelect);
        apply(&mut child, None, "-webkit-appearance:initial");
        assert_eq!(child.Appearance(), AppearanceValue::kNone);
        invalid(
            CSSPropertyID::kAliasWebkitAppearance,
            &[
                "base",
                "slider-horizontal",
                "none auto",
                "-internal-media-control",
            ],
        );
    }
    #[test]
    fn production_interaction_disabled_runtime_entries_and_native_bitsets() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        use crate::production_css_value as values;
        let hanging = values::list(
            vec![
                values::identifier(CSSValueID::kLast),
                values::identifier(CSSValueID::kFirst),
                values::identifier(CSSValueID::kAllowEnd),
            ],
            ListSeparator::Space,
        );
        let trim = values::list(
            vec![values::identifier(CSSValueID::kBlock)],
            ListSeparator::Space,
        );
        // Exercise the real source setters separately; the ordinary entry must
        // respect stable-disabled exposure even for CSS-wide values.
        ApplyInternal(
            CSSPropertyID::kHangingPunctuation,
            &mut parent,
            None,
            &hanging,
        )
        .unwrap();
        ApplyInternal(CSSPropertyID::kMarginTrim, &mut parent, None, &trim).unwrap();
        assert_eq!(
            snapshot(&parent).GetHangingPunctuation(),
            HangingPunctuation::kFirst | HangingPunctuation::kLast | HangingPunctuation::kAllowEnd
        );
        assert_eq!(
            parent.MarginTrim(),
            EMarginTrim::kMarginTrimBlock.value() as u32
        );
        let mut child = ComputedStyleBuilder::from_style(initial());
        for (id, v) in [
            (CSSPropertyID::kHangingPunctuation, &hanging),
            (CSSPropertyID::kMarginTrim, &trim),
        ] {
            for keyword in ["initial", "inherit", "unset", "var(--v)"] {
                let result = ParseProperty(
                    id,
                    &foundation::String::from(keyword),
                    false,
                    CSSParserMode::kHTMLStandardMode,
                );
                assert!(matches!(result, Err(e) if e.kind == PropertyParseErrorKind::Unsupported));
            }
            assert!(
                matches!(super::super::Apply(id,&mut child,None,v,16.0,&MediaValuesCachedData::default()),Err(LonghandApplicationError::Unsupported(p)) if p == id)
            );
            ApplyInternal(
                id,
                &mut child,
                Some(snapshot(&parent)),
                &values::wide(CSSValueID::kInherit).unwrap(),
            )
            .unwrap();
        }
        assert_eq!(
            snapshot(&child).GetHangingPunctuation(),
            snapshot(&parent).GetHangingPunctuation()
        );
        assert_eq!(child.MarginTrim(), parent.MarginTrim());
        ApplyInternal(
            CSSPropertyID::kHangingPunctuation,
            &mut child,
            None,
            &values::wide(CSSValueID::kInitial).unwrap(),
        )
        .unwrap();
        ApplyInternal(
            CSSPropertyID::kMarginTrim,
            &mut child,
            None,
            &values::wide(CSSValueID::kInitial).unwrap(),
        )
        .unwrap();
        assert_eq!(
            snapshot(&child).GetHangingPunctuation(),
            HangingPunctuation::kNone
        );
        assert_eq!(child.MarginTrim(), 0);
    }
}
