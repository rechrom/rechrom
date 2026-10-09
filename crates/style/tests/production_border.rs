use foundation::{
    CSSPropertyID, Color, EBorderStyle, String, TextDirection, WritingDirectionMode, WritingMode,
};
use layoutng_style::style::computed_style::ComputedStyle;
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    properties::longhand_dispatch::ResolvePhysical,
    StyleEngine,
};

fn node(owner: &dom::DOM, id: &str) -> usize {
    let d = owner.GetDocument();
    (0..d.NodeCount())
        .find(|&i| d.Node(i).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap()
}
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(node(owner, id))
            .unwrap()
            .native_style
            .Get()
    }
}
fn update(owner: &mut dom::DOM) {
    let mut engine = StyleEngine::new(owner);
    engine
        .Update(owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
}

#[test]
fn border_native_values_initial_inherit_and_math_pairs() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='border-color:red green blue currentcolor;border-radius:2px 4px / 6px 8px;border-spacing:3.999px 5.2px'><div id=c style='border-color:inherit;border-radius:inherit;border-spacing:inherit'></div><div id=i style='border-color:initial;border-radius:initial;border-spacing:initial'></div><div id=m style='border-top-left-radius:calc(10% + 2px) max(3px, 4px);border-spacing:calc(1px + 3px)'></div></div>");
    update(&mut owner);
    let (p, c, i, m) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
        native(&owner, "m"),
    );
    assert_eq!(p.BorderTopColor().GetColor(), Color::FromRGB(255, 0, 0));
    assert_eq!(p.BorderBottomColor().GetColor(), Color::FromRGB(0, 0, 255));
    assert!(p.BorderLeftColor().IsCurrentColor());
    assert!(p.BorderTopColor() == c.BorderTopColor());
    assert_eq!(p.BorderTopLeftRadius(), c.BorderTopLeftRadius());
    assert_eq!(p.BorderTopRightRadius().Width().Pixels(), 4.0);
    assert_eq!(p.BorderTopLeftRadius().Height().Pixels(), 6.0);
    assert_eq!(
        (p.HorizontalBorderSpacing(), p.VerticalBorderSpacing()),
        (4, 5)
    );
    assert_eq!(
        (c.HorizontalBorderSpacing(), c.VerticalBorderSpacing()),
        (4, 5)
    );
    assert!(i.BorderTopColor().IsCurrentColor());
    assert!(i.BorderTopLeftRadius().Width().IsZero());
    assert_eq!(
        (i.HorizontalBorderSpacing(), i.VerticalBorderSpacing()),
        (0, 0)
    );
    assert_eq!(
        m.BorderTopLeftRadius()
            .Width()
            .GetCalculationValue()
            .Evaluate(100.0, &foundation::EvaluationInput::default()),
        12.0
    );
    assert_eq!(m.BorderTopLeftRadius().Height().Pixels(), 4.0);
    assert_eq!(
        (m.HorizontalBorderSpacing(), m.VerticalBorderSpacing()),
        (4, 4)
    );
}

#[test]
fn logical_border_shorthands_and_corners_reach_native_physical_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=x style='writing-mode:vertical-rl;direction:rtl;border-block-color:red blue;border-inline-color:green currentcolor;border-block-style:solid dashed;border-inline-style:dotted double;border-block-width:2px 4px;border-inline-width:6px 8px;border-start-start-radius:10px 20%;border-end-end-radius:30px 40%'></div>");
    update(&mut owner);
    let x = native(&owner, "x");
    assert_eq!(x.BorderRightColor().GetColor(), Color::FromRGB(255, 0, 0));
    assert_eq!(x.BorderLeftColor().GetColor(), Color::FromRGB(0, 0, 255));
    assert_eq!(
        (x.BorderRightStyle(), x.BorderLeftStyle()),
        (EBorderStyle::kSolid, EBorderStyle::kDashed)
    );
    assert_eq!(
        (x.BorderBottomStyle(), x.BorderTopStyle()),
        (EBorderStyle::kDotted, EBorderStyle::kDouble)
    );
    assert_eq!(
        (
            x.BorderRightWidth(),
            x.BorderLeftWidth(),
            x.BorderBottomWidth(),
            x.BorderTopWidth()
        ),
        (2, 4, 6, 8)
    );
    assert_eq!(x.BorderBottomRightRadius().Width().Pixels(), 10.0);
    assert_eq!(
        x.BorderBottomRightRadius().Height(),
        &foundation::Length::Percent(20.0)
    );
    assert_eq!(x.BorderTopLeftRadius().Width().Pixels(), 30.0);
    // Source tables cover all five writing modes, especially sideways-lr.
    for (mode, start, end) in [
        (
            WritingMode::kHorizontalTb,
            CSSPropertyID::kBorderTopLeftRadius,
            CSSPropertyID::kBorderBottomRightRadius,
        ),
        (
            WritingMode::kVerticalRl,
            CSSPropertyID::kBorderTopRightRadius,
            CSSPropertyID::kBorderBottomLeftRadius,
        ),
        (
            WritingMode::kVerticalLr,
            CSSPropertyID::kBorderTopLeftRadius,
            CSSPropertyID::kBorderBottomRightRadius,
        ),
        (
            WritingMode::kSidewaysRl,
            CSSPropertyID::kBorderTopRightRadius,
            CSSPropertyID::kBorderBottomLeftRadius,
        ),
        (
            WritingMode::kSidewaysLr,
            CSSPropertyID::kBorderBottomLeftRadius,
            CSSPropertyID::kBorderTopRightRadius,
        ),
    ] {
        let direction = WritingDirectionMode::new(mode, TextDirection::kLtr);
        assert_eq!(
            ResolvePhysical(CSSPropertyID::kBorderStartStartRadius, direction),
            start
        );
        assert_eq!(
            ResolvePhysical(CSSPropertyID::kBorderEndEndRadius, direction),
            end
        );
    }
}

#[test]
fn invalid_border_declarations_preserve_previous_values_and_quirks_are_contextual() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=x style='border-top-color:red;border-top-color:#nope;border-radius:5px;border-radius:-2px;border-spacing:7px;border-spacing:10%;border-block-style:solid;border-block-style:solid bogus'></div>");
    update(&mut owner);
    let x = native(&owner, "x");
    assert_eq!(x.BorderTopColor().GetColor(), Color::FromRGB(255, 0, 0));
    assert_eq!(x.BorderTopLeftRadius().Width().Pixels(), 5.0);
    assert_eq!(x.HorizontalBorderSpacing(), 7);
    assert_eq!(x.BorderTopStyle(), EBorderStyle::kSolid);
    for text in ["112233", "0001FF", "FF0000"] {
        assert!(ParseProperty(
            CSSPropertyID::kBorderTopColor,
            &String::from(text),
            false,
            CSSParserMode::kHTMLQuirksMode
        )
        .is_ok());
        assert_eq!(
            ParseProperty(
                CSSPropertyID::kBorderTopColor,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Invalid
        );
    }
    assert!(ParseProperty(
        CSSPropertyID::kBorderColor,
        &String::from("112233"),
        false,
        CSSParserMode::kHTMLQuirksMode
    )
    .is_ok());
    assert!(ParseProperty(
        CSSPropertyID::kBorderTop,
        &String::from("solid 112233"),
        false,
        CSSParserMode::kHTMLQuirksMode
    )
    .is_err());
}

#[test]
fn logical_border_wide_keywords_and_writing_direction_inherit_native_state() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='direction:rtl;writing-mode:vertical-rl;border-block-width:2px 4px;border-inline-width:6px 8px;border-block-style:solid dashed;border-inline-style:dotted double;border-block-color:red blue'><div id=c style='direction:inherit;writing-mode:inherit;border-block-width:inherit;border-inline-width:inherit;border-block-style:inherit;border-inline-style:inherit;border-block-color:inherit'></div><div id=i style='direction:initial;writing-mode:initial;border-block-width:initial;border-inline-width:initial;border-block-style:initial;border-inline-style:initial;border-block-color:initial'></div><div id=o style='border-spacing:32768px 32766.999px'></div></div>");
    update(&mut owner);
    let (p, c, i, o) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
        native(&owner, "o"),
    );
    assert_eq!(c.Direction(), TextDirection::kRtl);
    assert_eq!(c.GetWritingMode(), WritingMode::kVerticalRl);
    assert_eq!(
        c.GetFontDescription().Orientation(),
        font_engine::fonts::font_orientation::FontOrientation::kVerticalMixed
    );
    assert_eq!(
        (
            c.BorderRightWidth(),
            c.BorderLeftWidth(),
            c.BorderBottomWidth(),
            c.BorderTopWidth()
        ),
        (2, 4, 6, 8)
    );
    assert!(p.BorderRightColor() == c.BorderRightColor());
    assert_eq!(i.Direction(), TextDirection::kLtr);
    assert_eq!(i.GetWritingMode(), WritingMode::kHorizontalTb);
    assert_eq!(
        i.GetFontDescription().Orientation(),
        font_engine::fonts::font_orientation::FontOrientation::kHorizontal
    );
    assert_eq!(*i.SpecifiedBorderTopWidth(), 3);
    assert_eq!(i.BorderTopStyle(), EBorderStyle::kNone);
    assert!(i.BorderTopColor().IsCurrentColor());
    assert_eq!(
        (o.HorizontalBorderSpacing(), o.VerticalBorderSpacing()),
        (0, 0)
    );
}
