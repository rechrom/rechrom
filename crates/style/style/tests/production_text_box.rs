use font_engine::fonts::shaping::text_spacing_trim::TextSpacingTrim as S;
use foundation::{
    CSSPropertyID as P, CSSValueID as V, ETextAutospace, ETextBoxTrim, String, TextDecorationLine,
};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    text_box_edge::TextBoxEdgeType as E,
    text_fit::{TextFitMethod as M, TextFitTarget as T, TextFitType as F},
};
use style::{
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
fn text_box_edges_shorthand_default_reset_and_css_wide_are_native() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='text-box:cap alphabetic trim-start'><div id=c style='text-box:inherit'></div><div id=u style='text-box:unset'></div><div id=i style='text-box:initial'></div><div id=n style='text-box:trim-end text;text-box:normal'></div><div id=e style='text-box:text alphabetic'></div><div id=t style='text-box:trim-end'></div><div id=x style='text-box:ex text trim-both'></div></div>",
    );
    update(&mut owner);
    let (p, c, u, i, n, e, t, x) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "u"),
        native(&owner, "i"),
        native(&owner, "n"),
        native(&owner, "e"),
        native(&owner, "t"),
        native(&owner, "x"),
    );
    for v in [p, c] {
        assert_eq!(*v.GetTextBoxEdge().Over(), E::kCap);
        assert_eq!(*v.GetTextBoxEdge().Under(), E::kAlphabetic);
        assert_eq!(v.TextBoxTrim(), ETextBoxTrim::kTrimStart);
    }
    assert_eq!(u.GetTextBoxEdge(), p.GetTextBoxEdge());
    assert_eq!(u.TextBoxTrim(), ETextBoxTrim::kNone);
    for v in [i, n] {
        assert!(v.GetTextBoxEdge().IsAuto());
        assert_eq!(v.TextBoxTrim(), ETextBoxTrim::kNone);
    }
    assert_eq!(*e.GetTextBoxEdge().Over(), E::kText);
    assert_eq!(*e.GetTextBoxEdge().Under(), E::kAlphabetic);
    assert_eq!(e.TextBoxTrim(), ETextBoxTrim::kTrimBoth);
    assert!(t.GetTextBoxEdge().IsAuto());
    assert_eq!(t.TextBoxTrim(), ETextBoxTrim::kTrimEnd);
    assert_eq!(*x.GetTextBoxEdge().Over(), E::kEx);
    assert_eq!(*x.GetTextBoxEdge().Under(), E::kText);
    assert_eq!(
        parse(P::kTextBoxEdge, "text text")[0]
            .Value()
            .CssText()
            .Utf8(),
        "text"
    );
    assert_eq!(
        parse(P::kTextBox, "text")
            .iter()
            .find(|p| p.PropertyID() == P::kTextBoxTrim)
            .unwrap()
            .Value()
            .CssText()
            .Utf8(),
        "trim-both"
    );
}
#[test]
fn text_fit_font_spacing_and_legacy_decoration_noop_preserve_native_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='text-fit:grow per-line-all 150%;text-autospace:normal;text-spacing-trim:space-first;font-size:21px;text-decoration-line:underline;-webkit-text-decorations-in-effect:overline'><div id=c style='text-fit:inherit;text-autospace:inherit;text-spacing-trim:inherit;-webkit-text-decorations-in-effect:inherit'></div><div id=u style='text-fit:unset;text-autospace:unset;text-spacing-trim:unset'></div><div id=i style='text-fit:initial;text-autospace:initial;text-spacing-trim:initial;-webkit-text-decorations-in-effect:initial'></div><div id=s style='text-fit:shrink calc(40% + 35%);text-spacing-trim:trim-start'></div><div id=n style='text-fit:none consistent 200%;text-autospace:no-autospace;text-spacing-trim:space-all'></div></div>",
    );
    update(&mut owner);
    let (p, c, u, i, s, n) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "u"),
        native(&owner, "i"),
        native(&owner, "s"),
        native(&owner, "n"),
    );
    for v in [p, c, u] {
        assert_eq!(v.GetTextFit().Type(), F::kGrow);
        assert_eq!(v.GetTextFit().Target(), T::kPerLineAll);
        assert_eq!(v.GetTextFit().ScaleFactorLimit(), Some(1.5));
        assert_eq!(v.TextAutospace(), ETextAutospace::kNormal);
        assert_eq!(v.GetFontDescription().GetTextSpacingTrim(), S::kSpaceFirst);
        assert_eq!(v.GetFontDescription().ComputedSize(), 21.0);
    }
    assert_eq!(p.GetTextFit().Method(), M::kScale);
    assert_eq!(p.GetTextDecorationLine(), TextDecorationLine::kUnderline);
    assert_eq!(i.GetTextFit().Type(), F::kNone);
    assert_eq!(i.GetTextFit().Target(), T::kConsistent);
    assert_eq!(i.GetTextFit().ScaleFactorLimit(), None);
    assert_eq!(i.TextAutospace(), ETextAutospace::kNoAutospace);
    assert_eq!(i.GetFontDescription().GetTextSpacingTrim(), S::kNormal);
    assert_eq!(s.GetTextFit().Type(), F::kShrink);
    assert_eq!(s.GetTextFit().Target(), T::kConsistent);
    assert_eq!(s.GetTextFit().ScaleFactorLimit(), Some(0.75));
    assert_eq!(s.GetFontDescription().GetTextSpacingTrim(), S::kTrimStart);
    assert_eq!(n.GetTextFit().Type(), F::kNone);
    assert_eq!(n.GetTextFit().ScaleFactorLimit(), Some(2.0));
    assert_eq!(n.GetFontDescription().GetTextSpacingTrim(), S::kSpaceAll);
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    b.SetTextDecorationLine(TextDecorationLine::kUnderline);
    for v in [
        values::wide(V::kInitial).unwrap(),
        values::wide(V::kInherit).unwrap(),
        values::wide(V::kUnset).unwrap(),
    ] {
        style::resolver::production_style_builder::Apply(
            P::kWebkitTextDecorationsInEffect,
            &mut b,
            Some(p),
            &v,
            16.0,
            &MediaValuesCachedData::default(),
        )
        .unwrap();
        assert_eq!(b.GetTextDecorationLine(), TextDecorationLine::kUnderline);
    }
    let parsed = parse(P::kWebkitTextDecorationsInEffect, "blink line-through");
    style::resolver::production_style_builder::Apply(
        P::kWebkitTextDecorationsInEffect,
        &mut b,
        Some(p),
        parsed[0].Value(),
        16.0,
        &MediaValuesCachedData::default(),
    )
    .unwrap();
    assert_eq!(b.GetTextDecorationLine(), TextDecorationLine::kUnderline);
}
#[test]
fn text_box_invalid_and_hidden_exposure_are_typed() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, text) in [
        (P::kTextBoxEdge, "cap"),
        (P::kTextBoxEdge, "ex"),
        (P::kTextBoxEdge, "alphabetic"),
        (P::kTextBoxEdge, "auto text"),
        (P::kTextBoxEdge, "text cap"),
        (P::kTextBox, "trim-start trim-end"),
        (P::kTextBox, "normal text"),
        (P::kTextFit, "grow shrink"),
        (P::kTextFit, "grow -1%"),
        (P::kTextFit, "per-line grow"),
        (P::kTextFit, "grow 10% per-line"),
        (P::kTextFit, "grow 1"),
        (P::kTextAutospace, "auto"),
        (P::kTextSpacingTrim, "none"),
        (P::kWebkitTextDecorationsInEffect, "underline underline"),
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
    for id in [P::kTextSpacing, P::kTextDecorationInset] {
        for text in [
            "initial",
            "inherit",
            "unset",
            "none",
            "auto",
            "var(--value)",
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
                PropertyParseErrorKind::Unsupported
            );
        }
    }
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    for id in [P::kTextDecorationInset] {
        assert!(style::resolver::production_style_builder::Apply(
            id,
            &mut b,
            None,
            &values::wide(V::kInitial).unwrap(),
            16.0,
            &MediaValuesCachedData::default()
        )
        .is_err());
    }
    assert!(b.GetTextDecorationInset().GetStart().IsFixed());
    assert_eq!(b.GetTextDecorationInset().GetStart().Pixels(), 0.0);
}
