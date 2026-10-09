use foundation::{
    CSSPropertyID as P, ERubyOverhang, RubyPosition, String, TextEmphasisFill as F,
    TextEmphasisMark as M,
};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    computed_style_constants::TextEmphasisPosition as E,
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
fn emphasis_custom_keyword_position_inheritance_and_shorthand_reset() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='color:blue;text-emphasis-style:&quot;\\203b x&quot;;text-emphasis-position:left under;text-emphasis-color:red'><div id=c style='text-emphasis-style:inherit;text-emphasis-position:inherit;text-emphasis-color:inherit'></div><div id=i style='text-emphasis:initial;text-emphasis-position:initial'></div><div id=u style='text-emphasis:unset;text-emphasis-position:unset'></div><div id=s style='text-emphasis-color:red;text-emphasis:circle open'></div><div id=a style='text-emphasis-style:open'></div><div id=n style='text-emphasis-style:none'></div><div id=q style='text-emphasis:&quot;&quot;'></div></div>",
    );
    update(&mut owner);
    let (p, c, i, u, s, a, n, q) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
        native(&owner, "u"),
        native(&owner, "s"),
        native(&owner, "a"),
        native(&owner, "n"),
        native(&owner, "q"),
    );
    for v in [p, c, u] {
        assert_eq!(v.GetTextEmphasisFill(), F::kFilled);
        assert_eq!(v.GetTextEmphasisMark(), M::kCustom);
        assert_eq!(v.TextEmphasisCustomMark().Utf8(), "※x");
        assert_eq!(v.GetTextEmphasisPosition(), E::kUnderLeft);
        assert!(v.TextEmphasisColor() == p.TextEmphasisColor());
    }
    assert_eq!(i.GetTextEmphasisMark(), M::kNone);
    assert_eq!(i.GetTextEmphasisPosition(), E::kOverRight);
    assert!(i.TextEmphasisColor().IsCurrentColor());
    assert_eq!(s.GetTextEmphasisFill(), F::kOpen);
    assert_eq!(s.GetTextEmphasisMark(), M::kCircle);
    assert!(s.TextEmphasisCustomMark().IsNull());
    assert!(s.TextEmphasisColor().IsCurrentColor());
    assert_eq!(a.GetTextEmphasisFill(), F::kOpen);
    assert_eq!(a.GetTextEmphasisMark(), M::kDot);
    assert_eq!(n.GetTextEmphasisMark(), M::kNone);
    assert_eq!(q.GetTextEmphasisMark(), M::kCustom);
    assert!(q.TextEmphasisCustomMark().empty());
    assert!(!q.TextEmphasisCustomMark().IsNull());
    assert!(q.TextEmphasisColor().IsCurrentColor());
    let parsed = parse(P::kTextEmphasis, "open");
    assert_eq!(parsed.len(), 2);
    assert!(
        parsed
            .iter()
            .find(|v| v.PropertyID() == P::kTextEmphasisColor)
            .unwrap()
            .Value()
            .IsInitialValue()
    );
    let pos = parse(P::kTextEmphasisPosition, "left over");
    assert!(
        matches!(pos[0].Value().Payload(),CSSValuePayload::kValueListClass(l) if matches!(l.values[0].Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==foundation::CSSValueID::kOver))
    );
}
#[test]
fn hyphenation_ruby_and_text_size_adjust_native_fields_css_wide() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='hyphenate-character:&quot;\\2010 &quot;;hyphenate-limit-chars:999 auto 3;ruby-position:under;ruby-overhang:none;text-size-adjust:125%'><div id=c style='hyphenate-character:inherit;hyphenate-limit-chars:inherit;ruby-position:inherit;ruby-overhang:inherit;text-size-adjust:inherit'></div><div id=u style='hyphenate-character:unset;hyphenate-limit-chars:unset;ruby-position:unset;ruby-overhang:unset;text-size-adjust:unset'></div><div id=i style='hyphenate-character:initial;hyphenate-limit-chars:initial;ruby-position:initial;ruby-overhang:initial;text-size-adjust:initial'></div><div id=a style='hyphenate-character:auto;hyphenate-limit-chars:auto;text-size-adjust:auto'></div><div id=e style='hyphenate-character:&quot;&quot;;hyphenate-limit-chars:7;text-size-adjust:none'></div><div id=t style='hyphenate-limit-chars:calc(2.5) 4;text-size-adjust:calc(50% + 25%)'></div></div>",
    );
    update(&mut owner);
    let (p, c, u, i, a, e, t) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "u"),
        native(&owner, "i"),
        native(&owner, "a"),
        native(&owner, "e"),
        native(&owner, "t"),
    );
    for v in [p, c, u] {
        assert_eq!(v.HyphenationString().Utf8(), "‐");
        assert_eq!(v.HyphenateLimitChars().MinWordChars(), 255);
        assert_eq!(v.HyphenateLimitChars().MinBeforeChars(), 0);
        assert_eq!(v.HyphenateLimitChars().MinAfterChars(), 3);
        assert_eq!(v.GetRubyPosition(), RubyPosition::kUnder);
        assert_eq!(v.RubyOverhang(), ERubyOverhang::kSpaces);
        assert_eq!(v.GetTextSizeAdjust().Multiplier(), 1.25);
    }
    for v in [i, a] {
        assert!(v.HyphenationString().IsNull());
        assert!(v.HyphenateLimitChars().IsAuto());
        assert!(v.GetTextSizeAdjust().IsAuto());
    }
    assert_eq!(i.GetRubyPosition(), RubyPosition::kOver);
    assert_eq!(i.RubyOverhang(), ERubyOverhang::kAuto);
    assert!(!e.HyphenationString().IsNull());
    assert!(e.HyphenationString().empty());
    assert_eq!(e.HyphenateLimitChars().MinWordChars(), 7);
    assert_eq!(e.HyphenateLimitChars().MinBeforeChars(), 0);
    assert_eq!(e.HyphenateLimitChars().MinAfterChars(), 0);
    assert_eq!(e.GetTextSizeAdjust().Multiplier(), 1.0);
    assert_eq!(t.HyphenateLimitChars().MinWordChars(), 3);
    assert_eq!(t.HyphenateLimitChars().MinBeforeChars(), 4);
    assert_eq!(t.HyphenateLimitChars().MinAfterChars(), 0);
    assert_eq!(t.GetTextSizeAdjust().Multiplier(), 0.75);
}
#[test]
fn typography_invalid_values_and_stable_exposure() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, text) in [
        (P::kTextEmphasisStyle, "open filled"),
        (P::kTextEmphasisStyle, "dot circle"),
        (P::kTextEmphasisPosition, "left"),
        (P::kTextEmphasisPosition, "over under"),
        (P::kTextEmphasisPosition, "auto"),
        (P::kHyphenateCharacter, "none"),
        (P::kHyphenateLimitChars, "0"),
        (P::kHyphenateLimitChars, "2.5"),
        (P::kHyphenateLimitChars, "1 2 3 4"),
        (P::kRubyPosition, "before"),
        (P::kRubyPosition, "inter-character"),
        (P::kRubyPosition, "alternate"),
        (P::kRubyOverhang, "over"),
        (P::kTextSizeAdjust, "-1%"),
        (P::kTextSizeAdjust, "1"),
        (P::kTextSizeAdjust, "1px"),
        (P::kTextEmphasis, "none red blue"),
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
    for text in [
        "initial",
        "inherit",
        "none",
        "all",
        "start end",
        "var(--spaces)",
    ] {
        assert_eq!(
            ParseProperty(
                P::kTextDecorationSkipSpaces,
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
    assert!(!foundation::RuntimeEnabledFeatures::TextEmphasisPositionAutoEnabled());
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    let media = MediaValuesCachedData::default();
    for (id, v) in [
        (
            P::kTextEmphasisStyle,
            values::list(
                vec![
                    values::identifier(foundation::CSSValueID::kOpen),
                    values::identifier(foundation::CSSValueID::kFilled),
                ],
                values::ListSeparator::Space,
            ),
        ),
        (
            P::kTextSizeAdjust,
            values::numeric(-1.0, style::css_primitive_value::UnitType::kPercentage),
        ),
    ] {
        assert!(
            style::resolver::production_style_builder::Apply(id, &mut b, None, &v, 16.0, &media)
                .is_err()
        );
    }
    assert_eq!(b.GetTextEmphasisFill(), F::kFilled);
    assert!(b.GetTextSizeAdjust().IsAuto());
    assert!(
        style::resolver::production_style_builder::Apply(
            P::kTextDecorationSkipSpaces,
            &mut b,
            None,
            &values::wide(foundation::CSSValueID::kInitial).unwrap(),
            16.0,
            &media
        )
        .is_err()
    );
}
