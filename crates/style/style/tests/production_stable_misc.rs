use foundation::{CSSPropertyID as P, String};
use layoutng_style::style::computed_style::ComputedStyle;
use style::{
    media_queries::MediaValuesCachedData,
    parser::{css_parser_mode::CSSParserMode, production_property_parser::ParseProperty},
    production_css_value as values, StyleEngine,
};
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let d = owner.GetDocument();
    let n = (0..d.NodeCount())
        .find(|&n| d.Node(n).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap();
    unsafe { &*d.ResolvedStyleFor(n).unwrap().native_style.Get() }
}
fn update(owner: &mut dom::DOM) {
    let mut e = StyleEngine::new(owner);
    e.Update(owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(e.Diagnostics().is_empty(), "{:?}", e.Diagnostics());
}
fn parse(id: P, text: &str) -> Vec<values::PropertyValue> {
    ParseProperty(
        id,
        &String::from(text),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap()
}
use font_engine::fonts::font_description::{
    FontSynthesisSmallCaps as C, FontSynthesisStyle as S, FontSynthesisWeight as W,
};
#[test]
fn font_synthesis_expands_all_source_longhands_and_stages_native_font() {
    let _heap = foundation::LayoutHeapScope::new();
    let expanded = parse(P::kFontSynthesis, "small-caps weight");
    assert_eq!(
        expanded.iter().map(|v| v.PropertyID()).collect::<Vec<_>>(),
        vec![
            P::kFontSynthesisWeight,
            P::kFontSynthesisStyle,
            P::kFontSynthesisSmallCaps
        ]
    );
    assert_eq!(
        expanded
            .iter()
            .map(|v| v.Value().CssText().Utf8())
            .collect::<Vec<_>>(),
        vec!["auto", "none", "auto"]
    );
    for v in expanded {
        assert_eq!(v.ShorthandID(), P::kFontSynthesis);
        assert!(!v.IsImplicit());
    }
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='font-synthesis:weight small-caps'><div id=c style='font-synthesis:inherit'></div><div id=u style='font-synthesis:unset'></div><div id=i style='font-synthesis:initial'></div></div><div id=reset style='font-synthesis:weight style small-caps;font-synthesis:weight'></div><div id=n style='font-synthesis:none'></div><div id=parts style='font-synthesis-weight:none;font-synthesis-style:auto;font-synthesis-small-caps:none'></div>",
    );
    update(&mut owner);
    for id in ["p", "c", "u"] {
        let d = native(&owner, id).GetFontDescription();
        assert_eq!(d.GetFontSynthesisWeight(), W::kAutoFontSynthesisWeight);
        assert_eq!(d.GetFontSynthesisStyle(), S::kNoneFontSynthesisStyle);
        assert_eq!(
            d.GetFontSynthesisSmallCaps(),
            C::kAutoFontSynthesisSmallCaps
        );
    }
    let reset = native(&owner, "reset").GetFontDescription();
    assert_eq!(reset.GetFontSynthesisWeight(), W::kAutoFontSynthesisWeight);
    assert_eq!(reset.GetFontSynthesisStyle(), S::kNoneFontSynthesisStyle);
    assert_eq!(
        reset.GetFontSynthesisSmallCaps(),
        C::kNoneFontSynthesisSmallCaps
    );
    let n = native(&owner, "n").GetFontDescription();
    assert_eq!(n.GetFontSynthesisWeight(), W::kNoneFontSynthesisWeight);
    assert_eq!(n.GetFontSynthesisStyle(), S::kNoneFontSynthesisStyle);
    assert_eq!(
        n.GetFontSynthesisSmallCaps(),
        C::kNoneFontSynthesisSmallCaps
    );
    let i = native(&owner, "i").GetFontDescription();
    assert_eq!(i.GetFontSynthesisWeight(), W::kAutoFontSynthesisWeight);
    assert_eq!(i.GetFontSynthesisStyle(), S::kAutoFontSynthesisStyle);
    assert_eq!(
        i.GetFontSynthesisSmallCaps(),
        C::kAutoFontSynthesisSmallCaps
    );
    let parts = native(&owner, "parts").GetFontDescription();
    assert_eq!(parts.GetFontSynthesisWeight(), W::kNoneFontSynthesisWeight);
    assert_eq!(parts.GetFontSynthesisStyle(), S::kAutoFontSynthesisStyle);
    assert_eq!(
        parts.GetFontSynthesisSmallCaps(),
        C::kNoneFontSynthesisSmallCaps
    );
}
#[test]
fn math_depth_parent_auto_add_integer_math_and_native_clamp() {
    let _heap = foundation::LayoutHeapScope::new();
    let v = parse(P::kMathDepth, "add(calc(-1.5))");
    assert!(
        matches!(v[0].Value().Payload(),style::css_value::CSSValuePayload::kFunctionClass(f) if f.function_id==foundation::CSSValueID::kAdd&&f.arguments.values.len()==1)
    );
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='math-style:compact;math-depth:5'><div id=a style='math-depth:auto-add'></div><div id=b style='math-depth:add(2)'></div><div id=m style='math-depth:add(calc(-1.5))'></div><div id=c style='math-depth:inherit'></div><div id=u style='math-depth:unset'></div><div id=i style='math-depth:initial'></div><div id=hi style='math-depth:add(999999999999999999999)'></div><div id=lo style='math-depth:add(-999999999999999999999)'></div></div><div id=normal style='math-style:normal;math-depth:3'><div id=owncompact style='math-style:compact;math-depth:auto-add'></div></div><div id=calc style='math-depth:calc(2.5)'></div><div id=literalhi style='math-depth:999999999999999999999'></div><div id=literaIlo style='math-depth:-999999999999999999999'></div>",
    );
    update(&mut owner);
    for (id, depth) in [
        ("p", 5),
        ("a", 6),
        ("b", 7),
        ("m", 4),
        ("c", 5),
        ("u", 5),
        ("i", 0),
        ("hi", i16::MAX),
        ("lo", i16::MIN),
        ("owncompact", 3),
        ("calc", 3),
        ("literalhi", i16::MAX),
        ("literaIlo", i16::MIN),
    ] {
        assert_eq!(native(&owner, id).MathDepth(), depth, "{id}");
    }
}
#[test]
fn ordinal_group_positive_integer_math_css_wide_and_invalid_declarations() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='-webkit-box-ordinal-group:9'><div id=c style='-webkit-box-ordinal-group:inherit'></div><div id=i style='-webkit-box-ordinal-group:initial'></div><div id=u style='-webkit-box-ordinal-group:unset'></div></div><div id=max style='-webkit-box-ordinal-group:999999999999999999999'></div><div id=calc style='-webkit-box-ordinal-group:calc(2.5)'></div><div id=clamped style='-webkit-box-ordinal-group:calc(-1.5)'></div>",
    );
    update(&mut owner);
    for (id, value) in [
        ("p", 9),
        ("c", 9),
        ("i", 1),
        ("u", 1),
        ("max", u32::MAX - 1),
        ("calc", 3),
        ("clamped", 1),
    ] {
        assert_eq!(native(&owner, id).BoxOrdinalGroup(), value, "{id}");
    }
    for (id, text) in [
        (P::kFontSynthesis, "weight weight"),
        (P::kFontSynthesis, "none style"),
        (P::kFontSynthesis, "position"),
        (P::kFontSynthesis, "auto"),
        (P::kFontSynthesisStyle, "oblique-only"),
        (P::kMathDepth, "1.5"),
        (P::kMathDepth, "add(1 2)"),
        (P::kMathDepth, "add()"),
        (P::kMathDepth, "add(auto-add)"),
        (P::kMathDepth, "1px"),
        (P::kWebkitBoxOrdinalGroup, "0"),
        (P::kWebkitBoxOrdinalGroup, "-1"),
        (P::kWebkitBoxOrdinalGroup, "1.5"),
    ] {
        assert!(
            ParseProperty(
                id,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .is_err(),
            "{id:?} {text}"
        );
    }
    for id in [P::kFontSynthesis, P::kMathDepth, P::kWebkitBoxOrdinalGroup] {
        for wide in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            assert_eq!(
                parse(id, wide).len(),
                if id == P::kFontSynthesis { 3 } else { 1 }
            );
        }
    }
}
