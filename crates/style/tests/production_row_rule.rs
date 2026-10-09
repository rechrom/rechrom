use foundation::{
    CSSPropertyID as P, CSSValueID as V, Color, EBorderStyle as S, EInsideLink, String,
};
use layoutng_style::{
    css::style_color::StyleColor,
    style::computed_style::{ComputedStyle, ComputedStyleBuilder},
};
use style::{
    css_value::CSSValuePayload,
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
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
#[test]
fn row_and_bidirectional_rule_lists_write_only_source_native_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='column-rule:7px double green;row-rule:red solid 0.5px,thin dashed rgb(0,0,255)'><div id=c style='row-rule:inherit'></div><div id=i style='row-rule:initial'></div><div id=u style='row-rule:unset'></div><div id=r style='row-rule:9px double red;row-rule:dotted'></div></div><div id=both style='rule:2px solid red,4px dashed blue'></div><div id=parts style='rule-color:red,blue;rule-width:2px,4px;rule-style:solid,double'></div><div id=reset style='rule:9px double red;rule:none'></div>",
    );
    update(&mut owner);
    for v in [native(&owner, "p"), native(&owner, "c")] {
        assert_eq!(v.RowRuleStyle().GetGapDataList().size(), 2);
        assert_eq!(
            v.RowRuleStyle().GetGapDataList().at(0).GetValue(),
            S::kSolid
        );
        assert_eq!(
            v.RowRuleStyle().GetGapDataList().at(1).GetValue(),
            S::kDashed
        );
        assert_eq!(v.RowRuleWidth().GetGapDataList().at(0).GetValue(), 1);
        assert_eq!(v.RowRuleWidth().GetGapDataList().at(1).GetValue(), 1);
        assert!(
            v.RowRuleColor().GetGapDataList().at(1).GetValue()
                == StyleColor::from_color(Color::FromRGB(0, 0, 255))
        );
        assert!(v.MaybeHasGapDecorations());
    }
    assert_eq!(native(&owner, "p").ColumnRuleWidth().GetSingleValue(), 7);
    assert_eq!(
        native(&owner, "p").ColumnRuleStyle().GetSingleValue(),
        S::kDouble
    );
    for v in [native(&owner, "i"), native(&owner, "u")] {
        assert_eq!(v.RowRuleWidth().GetSingleValue(), 3);
        assert_eq!(v.RowRuleStyle().GetSingleValue(), S::kNone);
        assert!(v.RowRuleColor().GetSingleValue().IsCurrentColor());
    }
    let r = native(&owner, "r");
    assert_eq!(r.RowRuleWidth().GetSingleValue(), 3);
    assert_eq!(r.RowRuleStyle().GetSingleValue(), S::kDotted);
    assert!(r.RowRuleColor().GetSingleValue().IsCurrentColor());
    for v in [native(&owner, "both"), native(&owner, "parts")] {
        assert_eq!(v.ColumnRuleWidth().GetGapDataList().at(1).GetValue(), 4);
        assert_eq!(v.RowRuleWidth().GetGapDataList().at(1).GetValue(), 4);
        assert!(
            v.RowRuleColor().GetGapDataList().at(0).GetValue()
                == v.ColumnRuleColor().GetGapDataList().at(0).GetValue()
        );
        assert!(
            v.RowRuleColor().GetGapDataList().at(1).GetValue()
                == v.ColumnRuleColor().GetGapDataList().at(1).GetValue()
        );
    }
    assert_eq!(
        native(&owner, "parts")
            .RowRuleStyle()
            .GetGapDataList()
            .at(1)
            .GetValue(),
        S::kDouble
    );
    let reset = native(&owner, "reset");
    assert_eq!(reset.ColumnRuleWidth().GetSingleValue(), 3);
    assert_eq!(reset.RowRuleWidth().GetSingleValue(), 3);
    assert_eq!(reset.ColumnRuleStyle().GetSingleValue(), S::kNone);
    assert_eq!(reset.RowRuleStyle().GetSingleValue(), S::kNone);
    assert!(reset.ColumnRuleColor().GetSingleValue().IsCurrentColor());
    assert!(reset.RowRuleColor().GetSingleValue().IsCurrentColor());
}
#[test]
fn bidirectional_rule_expansion_and_css_wide_preserve_all_six_longhands() {
    let _heap = foundation::LayoutHeapScope::new();
    let expanded = parse(P::kRule, "solid");
    assert_eq!(expanded.len(), 6);
    assert_eq!(
        expanded.iter().map(|v| v.PropertyID()).collect::<Vec<_>>(),
        vec![
            P::kColumnRuleWidth,
            P::kColumnRuleStyle,
            P::kColumnRuleColor,
            P::kRowRuleWidth,
            P::kRowRuleStyle,
            P::kRowRuleColor
        ]
    );
    for (index, value) in expanded.iter().enumerate() {
        assert_eq!(
            value.ShorthandID(),
            if index < 3 {
                P::kColumnRule
            } else {
                P::kRowRule
            }
        );
        assert!(!value.IsImplicit());
    }
    for (id, text) in [
        (P::kRuleColor, "red,blue"),
        (P::kRuleWidth, "thin,4px"),
        (P::kRuleStyle, "solid,double"),
    ] {
        let list = parse(id, text);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].Value().CssText(), list[1].Value().CssText());
        assert!(
            matches!(list[0].Value().Payload(),CSSValuePayload::kValueListClass(l) if l.separator==values::ListSeparator::Comma&&l.values.len()==2)
        );
    }
    for id in [
        P::kRowRule,
        P::kRule,
        P::kRuleColor,
        P::kRuleStyle,
        P::kRuleWidth,
    ] {
        for wide in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            assert_eq!(
                parse(id, wide).len(),
                if id == P::kRule {
                    6
                } else if id == P::kRowRule {
                    3
                } else {
                    2
                }
            );
        }
    }
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='rule:5px ridge blue'><div id=c style='rule:inherit'></div><div id=i style='rule:initial'></div><div id=u style='rule:unset'></div><div id=parts style='rule-color:inherit;rule-width:inherit;rule-style:inherit'></div></div>",
    );
    update(&mut owner);
    for v in [native(&owner, "c"), native(&owner, "parts")] {
        assert_eq!(v.RowRuleWidth().GetSingleValue(), 5);
        assert_eq!(v.ColumnRuleWidth().GetSingleValue(), 5);
        assert_eq!(v.RowRuleStyle().GetSingleValue(), S::kRidge);
        assert_eq!(v.ColumnRuleStyle().GetSingleValue(), S::kRidge);
        assert!(
            v.RowRuleColor().GetSingleValue() == StyleColor::from_color(Color::FromRGB(0, 0, 255))
        );
    }
    for v in [native(&owner, "i"), native(&owner, "u")] {
        assert_eq!(v.RowRuleWidth().GetSingleValue(), 3);
        assert_eq!(v.ColumnRuleWidth().GetSingleValue(), 3);
        assert_eq!(v.RowRuleStyle().GetSingleValue(), S::kNone);
        assert_eq!(v.ColumnRuleStyle().GetSingleValue(), S::kNone);
    }
}
#[test]
fn row_rule_invalid_repeat_and_native_context_boundaries_remain_typed() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, text) in [
        (P::kRowRuleWidth, "-1px"),
        (P::kRuleWidth, "20%"),
        (P::kRuleStyle, "solid dashed"),
        (P::kRowRule, "solid dashed"),
        (P::kRule, "red blue"),
        (P::kRuleColor, "no-such-color"),
    ] {
        assert!(ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_err());
    }
    for id in [
        P::kRowRuleColor,
        P::kRowRuleStyle,
        P::kRowRuleWidth,
        P::kRowRule,
        P::kRule,
        P::kRuleColor,
        P::kRuleStyle,
        P::kRuleWidth,
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from("repeat(2,solid)"),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Unsupported
        );
    }
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    b.SetEffectiveZoom(2.0);
    let media = MediaValuesCachedData::default();
    style::resolver::production_style_builder::Apply(
        P::kRowRuleWidth,
        &mut b,
        None,
        &values::wide(V::kInitial).unwrap(),
        16.0,
        &media,
    )
    .unwrap();
    assert_eq!(b.RowRuleWidth().GetSingleValue(), 6);
    b.SetInsideLink(EInsideLink::kInsideUnvisitedLink);
    let colors = parse(P::kRowRuleColor, "red,blue");
    style::resolver::production_style_builder::Apply(
        P::kRowRuleColor,
        &mut b,
        None,
        colors[0].Value(),
        16.0,
        &media,
    )
    .unwrap();
    assert!(b.RowRuleColor().GetSingleValue().IsCurrentColor());
    let parent = unsafe { &*b.TakeStyle() };
    let mut child = ComputedStyleBuilder::from_style(initial);
    assert!(style::resolver::production_style_builder::Apply(
        P::kRowRuleWidth,
        &mut child,
        Some(parent),
        &values::wide(V::kInherit).unwrap(),
        16.0,
        &media
    )
    .is_err());
}
