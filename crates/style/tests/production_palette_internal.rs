use font_engine::fonts::font_palette::{
    BasePaletteValueType, FontPalette, KeywordPaletteName as K,
};
use foundation::{CSSPropertyID as P, CSSValueID as V, String};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode as M,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    production_css_value as values, StyleEngine,
};
fn native<'a>(d: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let d = d.GetDocument();
    let n = (0..d.NodeCount())
        .find(|&n| d.Node(n).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap();
    unsafe { &*d.ResolvedStyleFor(n).unwrap().native_style.Get() }
}
fn parse(id: P, text: &str, mode: M) -> Vec<values::PropertyValue> {
    ParseProperty(id, &String::from(text), false, mode).unwrap()
}
fn apply(id: P, b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, v: &values::Value) {
    style::resolver::production_style_builder::Apply(
        id,
        b,
        parent,
        v,
        16.0,
        &MediaValuesCachedData::default(),
    )
    .unwrap();
}
#[test]
fn palettes_retain_native_owner_name_and_css_wide_inheritance() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut d = html::html_parser::ParseHTML(
        "<div id=p style='font-palette:dark'><div id=c style='font-palette:inherit'></div><div id=u style='font-palette:unset'></div><div id=i style='font-palette:initial'></div><div id=n style='font-palette:normal'></div></div><div id=l style='font-palette:light'></div><div id=custom style='font-palette:--Case'></div>",
    );
    let mut e = StyleEngine::new(&d);
    e.Update(&mut d, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(e.Diagnostics().is_empty(), "{:?}", e.Diagnostics());
    let p = native(&d, "p")
        .GetFontDescription()
        .FontPaletteValue()
        .unwrap();
    assert_eq!(p.GetPaletteNameKind(), K::kDarkPalette);
    for id in ["c", "u"] {
        let child = native(&d, id)
            .GetFontDescription()
            .FontPaletteValue()
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(&p, &child));
    }
    for id in ["i", "n"] {
        assert!(native(&d, id)
            .GetFontDescription()
            .FontPaletteValue()
            .is_none());
    }
    assert_eq!(
        native(&d, "l")
            .GetFontDescription()
            .FontPaletteValue()
            .unwrap()
            .GetPaletteNameKind(),
        K::kLightPalette
    );
    let custom = native(&d, "custom")
        .GetFontDescription()
        .FontPaletteValue()
        .unwrap();
    assert_eq!(custom.GetPaletteValuesName().Utf8(), "--Case");
    assert_eq!(
        custom.GetBasePalette().r#type,
        BasePaletteValueType::kNoBasePalette
    );
    assert!(custom.GetColorOverrides().is_empty());
    assert!(custom.GetMatchFamilyName().empty());
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    let light = parse(P::kFontPalette, "light", M::kHTMLStandardMode);
    apply(P::kFontPalette, &mut b, None, light[0].Value());
    let previous = b.GetFontDescription().FontPaletteValue().unwrap();
    let mut equivalent = b.GetFontDescription().clone();
    let replacement = FontPalette::CreateKeyword(K::kLightPalette);
    equivalent.SetFontPalette(Some(replacement.clone()));
    assert!(b.GetFontDescription() == &equivalent);
    style::resolver::production_style_builder::StageFontDescription(&mut b, &equivalent);
    let after = b.GetFontDescription().FontPaletteValue().unwrap();
    assert!(!std::sync::Arc::ptr_eq(&previous, &after));
    assert!(std::sync::Arc::ptr_eq(&replacement, &after));
    for wide in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        assert_eq!(parse(P::kFontPalette, wide, M::kHTMLStandardMode).len(), 1);
    }
}
#[test]
fn internal_ua_consumers_apply_bool_fields_and_source_inheritance() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut parent = ComputedStyleBuilder::from_style(initial);
    for (id, text) in [
        (P::kInternalAlignContentBlock, "center"),
        (P::kInternalEmptyLineHeight, "fabricated"),
    ] {
        apply(
            id,
            &mut parent,
            None,
            parse(id, text, M::kUASheetMode)[0].Value(),
        );
    }
    let parent = parent.TakeStyle();
    let parent = unsafe { &*parent };
    assert!(parent.AlignContentBlockCenter() && parent.HasLineIfEmpty());
    let mut b = ComputedStyleBuilder::from_style(initial);
    for id in [P::kInternalAlignContentBlock, P::kInternalEmptyLineHeight] {
        apply(
            id,
            &mut b,
            Some(parent),
            &values::wide(V::kInherit).unwrap(),
        );
    }
    assert!(b.AlignContentBlockCenter() && b.HasLineIfEmpty());
    for id in [P::kInternalAlignContentBlock, P::kInternalEmptyLineHeight] {
        apply(id, &mut b, Some(parent), &values::wide(V::kUnset).unwrap());
    }
    assert!(!b.AlignContentBlockCenter());
    assert!(b.HasLineIfEmpty());
    for id in [P::kInternalAlignContentBlock, P::kInternalEmptyLineHeight] {
        apply(
            id,
            &mut b,
            Some(parent),
            &values::wide(V::kInitial).unwrap(),
        );
    }
    assert!(!b.AlignContentBlockCenter() && !b.HasLineIfEmpty());
    for (id, text) in [
        (P::kInternalAlignContentBlock, "normal"),
        (P::kInternalEmptyLineHeight, "none"),
    ] {
        apply(
            id,
            &mut b,
            Some(parent),
            parse(id, text, M::kUASheetMode)[0].Value(),
        );
    }
    // Source DynamicTo<CSSIdentifierValue> falls back to false for non-identifiers.
    for id in [P::kInternalAlignContentBlock, P::kInternalEmptyLineHeight] {
        apply(
            id,
            &mut b,
            Some(parent),
            &values::string(String::from("center")),
        );
    }
    assert!(!b.AlignContentBlockCenter() && !b.HasLineIfEmpty());
}
#[test]
fn palette_invalid_partial_and_internal_exposure_are_typed() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, text) in [
        (P::kFontPalette, "auto"),
        (P::kFontPalette, "ordinary"),
        (P::kFontPalette, "light dark"),
        (P::kInternalAlignContentBlock, "start"),
        (P::kInternalEmptyLineHeight, "normal"),
    ] {
        assert_eq!(
            ParseProperty(id, &String::from(text), false, M::kUASheetMode)
                .err()
                .unwrap()
                .kind,
            PropertyParseErrorKind::Invalid
        );
    }
    assert_eq!(
        ParseProperty(
            P::kFontPalette,
            &String::from("palette-mix(in srgb, light, dark)"),
            false,
            M::kHTMLStandardMode
        )
        .err()
        .unwrap()
        .kind,
        PropertyParseErrorKind::Unsupported
    );
    for id in [P::kInternalAlignContentBlock, P::kInternalEmptyLineHeight] {
        for text in [
            "initial",
            "inherit",
            "unset",
            "revert",
            "revert-layer",
            "var(--value)",
            "center",
            "fabricated",
        ] {
            assert_eq!(
                ParseProperty(id, &String::from(text), false, M::kHTMLStandardMode)
                    .err()
                    .unwrap()
                    .kind,
                PropertyParseErrorKind::Unsupported
            );
        }
        for wide in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            assert_eq!(parse(id, wide, M::kUASheetMode).len(), 1);
        }
    }
}
