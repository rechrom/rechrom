use foundation::{
    CSSPropertyID as P, CSSValueID as V, Color, EBorderStyle as S, EBreakBetween as B,
    EBreakInside as I, EInsideLink, String,
};
use layoutng_style::{
    css::style_color::StyleColor,
    style::computed_style::{ComputedStyle, ComputedStyleBuilder},
};
use style::{
    StyleEngine,
    css_value::CSSValuePayload,
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    production_css_value as values,
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
#[test]
fn column_rule_shorthand_lists_defaults_and_css_wide_use_native_gap_data() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='column-rule:red solid 0.5px,thin dashed rgb(0,0,255)'><div id=c style='column-rule:inherit'></div><div id=i style='column-rule:initial'></div><div id=u style='column-rule:unset'></div><div id=r style='column-rule:9px double red;column-rule:dotted'></div><div id=n style='column-rule:none'></div><div id=h style='column-rule:999999999px hidden currentcolor'></div><div id=l style='column-rule-color:red,blue;column-rule-style:solid,double;column-rule-width:2px,4px'></div></div>",
    );
    update(&mut owner);
    let (p, c, i, u, r, n, h, l) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
        native(&owner, "u"),
        native(&owner, "r"),
        native(&owner, "n"),
        native(&owner, "h"),
        native(&owner, "l"),
    );
    for v in [p, c] {
        assert_eq!(v.ColumnRuleStyle().GetGapDataList().size(), 2);
        assert_eq!(
            v.ColumnRuleStyle().GetGapDataList().at(0).GetValue(),
            S::kSolid
        );
        assert_eq!(
            v.ColumnRuleStyle().GetGapDataList().at(1).GetValue(),
            S::kDashed
        );
        assert_eq!(v.ColumnRuleWidth().GetGapDataList().at(0).GetValue(), 1);
        assert_eq!(v.ColumnRuleWidth().GetGapDataList().at(1).GetValue(), 1);
        assert!(
            v.ColumnRuleColor().GetGapDataList().at(0).GetValue()
                == StyleColor::from_color(Color::FromRGB(255, 0, 0))
        );
        assert!(
            v.ColumnRuleColor().GetGapDataList().at(1).GetValue()
                == StyleColor::from_color(Color::FromRGB(0, 0, 255))
        );
        assert!(v.MaybeHasGapDecorations());
    }
    for v in [i, u, n] {
        assert!(v.ColumnRuleColor().GetSingleValue().IsCurrentColor());
        assert_eq!(v.ColumnRuleStyle().GetSingleValue(), S::kNone);
        assert_eq!(v.ColumnRuleWidth().GetSingleValue(), 3);
    }
    assert_eq!(r.ColumnRuleStyle().GetSingleValue(), S::kDotted);
    assert_eq!(r.ColumnRuleWidth().GetSingleValue(), 3);
    assert!(r.ColumnRuleColor().GetSingleValue().IsCurrentColor());
    assert_eq!(h.ColumnRuleWidth().GetSingleValue(), u16::MAX as i32);
    assert_eq!(h.ColumnRuleStyle().GetSingleValue(), S::kHidden);
    assert_eq!(l.ColumnRuleWidth().GetGapDataList().at(0).GetValue(), 2);
    assert_eq!(l.ColumnRuleWidth().GetGapDataList().at(1).GetValue(), 4);
    assert_eq!(
        l.ColumnRuleStyle().GetGapDataList().at(1).GetValue(),
        S::kDouble
    );
    let parsed = parse(P::kColumnRule, "blue");
    assert_eq!(parsed.len(), 3);
    for v in parsed {
        assert!(
            matches!(v.Value().Payload(),CSSValuePayload::kValueListClass(l) if l.separator==values::ListSeparator::Comma&&l.values.len()==1)
        );
    }
    // Preserve the supplied source helper's consumed trailing-comma behavior.
    assert_eq!(
        parse(P::kColumnRuleStyle, "solid,")[0]
            .Value()
            .CssText()
            .Utf8(),
        "solid"
    );
    assert_eq!(parse(P::kColumnRule, "solid,").len(), 3);
}
#[test]
fn webkit_column_break_aliases_expand_to_modern_native_break_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='-webkit-column-break-before:always;-webkit-column-break-after:avoid;-webkit-column-break-inside:avoid'><div id=c style='-webkit-column-break-before:inherit;-webkit-column-break-after:inherit;-webkit-column-break-inside:inherit'></div><div id=i style='-webkit-column-break-before:initial;-webkit-column-break-after:initial;-webkit-column-break-inside:initial'></div><div id=u style='-webkit-column-break-before:unset;-webkit-column-break-after:unset;-webkit-column-break-inside:unset'></div></div>",
    );
    update(&mut owner);
    for v in [native(&owner, "p"), native(&owner, "c")] {
        assert_eq!(v.BreakBefore(), B::kColumn);
        assert_eq!(v.BreakAfter(), B::kAvoid);
        assert_eq!(v.BreakInside(), I::kAvoid);
    }
    for v in [native(&owner, "i"), native(&owner, "u")] {
        assert_eq!(v.BreakBefore(), B::kAuto);
        assert_eq!(v.BreakAfter(), B::kAuto);
        assert_eq!(v.BreakInside(), I::kAuto);
    }
    for (id, physical) in [
        (P::kWebkitColumnBreakBefore, P::kBreakBefore),
        (P::kWebkitColumnBreakAfter, P::kBreakAfter),
    ] {
        let p = parse(id, "always");
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].PropertyID(), physical);
        assert_eq!(p[0].Value().CssText().Utf8(), "column");
    }
}
#[test]
fn column_rule_invalid_partial_and_native_zoom_color_boundaries() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, text) in [
        (P::kColumnRuleWidth, "-1px"),
        (P::kColumnRuleWidth, "20%"),
        (P::kColumnRuleWidth, "2"),
        (P::kColumnRuleStyle, "solid dotted"),
        (P::kColumnRuleStyle, "auto"),
        (P::kColumnRule, "1px 2px"),
        (P::kColumnRule, "red blue"),
        (P::kColumnRule, "solid double"),
        (P::kColumnRule, ","),
        (P::kWebkitColumnBreakBefore, "left"),
        (P::kWebkitColumnBreakAfter, "column"),
        (P::kWebkitColumnBreakInside, "always"),
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
            "{id:?} {text}"
        );
    }
    for id in [
        P::kColumnRule,
        P::kColumnRuleColor,
        P::kColumnRuleStyle,
        P::kColumnRuleWidth,
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from("repeat(2, solid)"),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Unsupported
        );
    }
    assert!(
        ParseProperty(
            P::kColumnRuleWidth,
            &String::from("2"),
            false,
            CSSParserMode::kHTMLQuirksMode
        )
        .is_err()
    );
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    let media = MediaValuesCachedData::default();
    b.SetEffectiveZoom(2.0);
    style::resolver::production_style_builder::Apply(
        P::kColumnRuleWidth,
        &mut b,
        None,
        &values::wide(V::kInitial).unwrap(),
        16.0,
        &media,
    )
    .unwrap();
    assert_eq!(b.ColumnRuleWidth().GetSingleValue(), 6);
    let parsed = parse(P::kColumnRuleWidth, "calc(1px + 0.5px), thick");
    style::resolver::production_style_builder::Apply(
        P::kColumnRuleWidth,
        &mut b,
        None,
        parsed[0].Value(),
        16.0,
        &media,
    )
    .unwrap();
    assert_eq!(b.ColumnRuleWidth().GetGapDataList().at(0).GetValue(), 3);
    assert_eq!(b.ColumnRuleWidth().GetGapDataList().at(1).GetValue(), 10);
    let parent = unsafe { &*b.TakeStyle() };
    let mut child = ComputedStyleBuilder::from_style(initial);
    assert!(
        style::resolver::production_style_builder::Apply(
            P::kColumnRuleWidth,
            &mut child,
            Some(parent),
            &values::wide(V::kInherit).unwrap(),
            16.0,
            &media
        )
        .is_err()
    );
    child.SetInsideLink(EInsideLink::kInsideUnvisitedLink);
    let colors = parse(P::kColumnRuleColor, "red, blue");
    style::resolver::production_style_builder::Apply(
        P::kColumnRuleColor,
        &mut child,
        None,
        colors[0].Value(),
        16.0,
        &media,
    )
    .unwrap();
    assert!(child.ColumnRuleColor().HasSingleValue());
    assert!(child.ColumnRuleColor().GetSingleValue().IsCurrentColor());
    let single = parse(P::kColumnRuleColor, "red");
    style::resolver::production_style_builder::Apply(
        P::kColumnRuleColor,
        &mut child,
        None,
        single[0].Value(),
        16.0,
        &media,
    )
    .unwrap();
    assert!(
        child.ColumnRuleColor().GetSingleValue()
            == StyleColor::from_color(Color::FromRGB(255, 0, 0))
    );
}
