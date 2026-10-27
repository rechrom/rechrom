use foundation::{
    CSSPropertyID as P, CSSValueID as V, EBreakBetween, EBreakInside, RuntimeEnabledFeatures,
    String,
};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    computed_style_constants::EVerticalAlign,
};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    StyleEngine,
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
fn parse(id: P, text: &str) -> Vec<style::production_css_value::PropertyValue> {
    ParseProperty(
        id,
        &String::from(text),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap()
}
#[test]
fn vertical_align_tab_size_and_legacy_line_clamp_reach_native_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=p style='vertical-align:calc(25% - 3px);tab-size:calc(2 + .5);-webkit-line-clamp:calc(2.5)'><div id=c style='vertical-align:inherit;tab-size:inherit;-webkit-line-clamp:inherit'></div><div id=i style='vertical-align:initial;tab-size:initial;-webkit-line-clamp:initial'></div><div id=u style='vertical-align:unset;tab-size:unset;-webkit-line-clamp:unset'></div><div id=l style='vertical-align:-8px;tab-size:12px;-webkit-line-clamp:999999999999'></div><div id=k style='vertical-align:-webkit-baseline-middle;tab-size:0;-webkit-line-clamp:none'></div><div id=n style='vertical-align:20%;tab-size:calc(3px + 4px);-webkit-line-clamp:calc(-5)'></div></div>");
    update(&mut owner);
    let (p, c, i, u, l, k, n) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
        native(&owner, "u"),
        native(&owner, "l"),
        native(&owner, "k"),
        native(&owner, "n"),
    );
    assert_eq!(p.VerticalAlign(), EVerticalAlign::kLength);
    assert!(p.GetVerticalAlignLength().IsCalculated());
    assert_eq!(p.GetVerticalAlignLength(), c.GetVerticalAlignLength());
    assert_eq!(p.GetTabSize().float_value_, 2.5);
    assert!(p.GetTabSize().IsSpaces());
    assert_eq!(c.GetTabSize(), p.GetTabSize());
    assert_eq!(u.GetTabSize(), p.GetTabSize());
    assert_eq!(p.WebkitLineClamp(), 3);
    assert_eq!(c.WebkitLineClamp(), 3);
    assert_eq!(i.GetTabSize().float_value_, 8.0);
    for s in [i, u] {
        assert_eq!(s.VerticalAlign(), EVerticalAlign::kBaseline);
        assert_eq!(s.WebkitLineClamp(), 0);
    }
    assert_eq!(l.GetVerticalAlignLength().Pixels(), -8.0);
    assert!(!l.GetTabSize().IsSpaces());
    assert_eq!(l.GetTabSize().float_value_, 12.0);
    assert_eq!(l.WebkitLineClamp(), i32::MAX);
    assert_eq!(k.VerticalAlign(), EVerticalAlign::kBaselineMiddle);
    assert_eq!(k.WebkitLineClamp(), 0);
    assert_eq!(n.GetVerticalAlignLength().PercentValue(), 20.0);
    assert!(!n.GetTabSize().IsSpaces());
    assert_eq!(n.GetTabSize().float_value_, 7.0);
    assert_eq!(n.WebkitLineClamp(), 1);
}
#[test]
fn page_break_aliases_expand_to_native_break_properties_with_css_wide_reset() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, longhand, text, expected) in [
        (P::kPageBreakBefore, P::kBreakBefore, "always", V::kPage),
        (P::kPageBreakAfter, P::kBreakAfter, "right", V::kRight),
        (P::kPageBreakInside, P::kBreakInside, "avoid", V::kAvoid),
    ] {
        let v = parse(id, text);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].Name().Id(), longhand);
        assert_eq!(v[0].ShorthandID(), id);
        assert!(!v[0].IsImplicit());
        assert!(
            matches!(v[0].Value().Payload(),style::css_value::CSSValuePayload::kIdentifierClass(k) if k.0==expected)
        );
    }
    let mut owner=html::html_parser::ParseHTML("<div id=p style='page-break-before:always;page-break-after:left;page-break-inside:avoid'><div id=c style='page-break-before:inherit;page-break-after:inherit;page-break-inside:inherit'></div><div id=i style='page-break-before:initial;page-break-after:initial;page-break-inside:initial'></div><div id=u style='page-break-before:unset;page-break-after:unset;page-break-inside:unset'></div><div id=b style='break-before:recto;break-after:avoid-column;break-inside:avoid-page'></div><div id=r style='break-before:column;page-break-before:auto;break-inside:avoid-column;page-break-inside:auto'></div></div>");
    update(&mut owner);
    for s in [native(&owner, "p"), native(&owner, "c")] {
        assert_eq!(s.BreakBefore(), EBreakBetween::kPage);
        assert_eq!(s.BreakAfter(), EBreakBetween::kLeft);
        assert_eq!(s.BreakInside(), EBreakInside::kAvoid);
    }
    for s in [native(&owner, "i"), native(&owner, "u")] {
        assert_eq!(s.BreakBefore(), EBreakBetween::kAuto);
        assert_eq!(s.BreakAfter(), EBreakBetween::kAuto);
        assert_eq!(s.BreakInside(), EBreakInside::kAuto);
    }
    let b = native(&owner, "b");
    assert_eq!(b.BreakBefore(), EBreakBetween::kRecto);
    assert_eq!(b.BreakAfter(), EBreakBetween::kAvoidColumn);
    assert_eq!(b.BreakInside(), EBreakInside::kAvoidPage);
    let r = native(&owner, "r");
    assert_eq!(r.BreakBefore(), EBreakBetween::kAuto);
    assert_eq!(r.BreakInside(), EBreakInside::kAuto);
}
#[test]
fn stable_runtime_exposure_and_invalid_values_are_preserved() {
    let _heap = foundation::LayoutHeapScope::new();
    assert!(!RuntimeEnabledFeatures::CSSLineClampEnabled());
    assert!(!RuntimeEnabledFeatures::CSSLineClampAsShorthandEnabled());
    for id in [
        P::kLineClamp,
        P::kMaxLines,
        P::kAlternativeLineClampShorthand,
        P::kAlternativeWebkitLineClampShorthand,
        P::kAlternativeWebkitLineClampLonghand,
        P::kContinue,
        P::kBlockEllipsis,
    ] {
        for text in [
            "3",
            "none",
            "3 no-ellipsis -webkit-legacy",
            "initial",
            "inherit",
            "var(--clamp)",
        ] {
            let e = ParseProperty(
                id,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .err()
            .unwrap();
            assert_eq!(e.kind, PropertyParseErrorKind::Unsupported);
        }
    }
    for (id, text) in [
        (P::kVerticalAlign, "left"),
        (P::kVerticalAlign, "2"),
        (P::kVerticalAlign, "auto"),
        (P::kTabSize, "-2"),
        (P::kTabSize, "-3px"),
        (P::kTabSize, "10%"),
        (P::kTabSize, "none"),
        (P::kWebkitLineClamp, "0"),
        (P::kWebkitLineClamp, "-1"),
        (P::kWebkitLineClamp, "1.5"),
        (P::kWebkitLineClamp, "auto"),
        (P::kPageBreakBefore, "page"),
        (P::kPageBreakAfter, "avoid-page"),
        (P::kPageBreakInside, "always"),
        (P::kBreakInside, "left"),
        (P::kPageBreakBefore, "auto avoid"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Invalid,
            "{id:?}: {text}"
        );
    }
    let q = ParseProperty(
        P::kVerticalAlign,
        &String::from("2"),
        false,
        CSSParserMode::kHTMLQuirksMode,
    )
    .unwrap();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    style::resolver::production_style_builder::Apply(
        P::kVerticalAlign,
        &mut builder,
        None,
        q[0].Value(),
        16.0,
        &MediaValuesCachedData::default(),
    )
    .unwrap();
    assert_eq!(builder.GetVerticalAlignLength().Pixels(), 2.0);
    for id in [
        P::kLineClamp,
        P::kMaxLines,
        P::kAlternativeWebkitLineClampLonghand,
    ] {
        assert_eq!(
            style::resolver::production_style_builder::Apply(
                id,
                &mut builder,
                None,
                &style::production_css_value::Value::new(
                    style::css_value::CSSValuePayload::kIdentifierClass(
                        style::production_css_value::CSSIdentifierValue(V::kNone)
                    )
                ),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(style::properties::longhand_dispatch::LonghandApplicationError::Unsupported(id))
        );
    }
    let mut changed = ComputedStyleBuilder::from_style(initial);
    changed.SetEffectiveZoom(2.0);
    for id in [P::kVerticalAlign, P::kTabSize] {
        let v = parse(id, "inherit");
        assert_eq!(
            style::resolver::production_style_builder::Apply(
                id,
                &mut changed,
                Some(initial),
                v[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(style::properties::longhand_dispatch::LonghandApplicationError::Unsupported(id))
        );
    }
}
