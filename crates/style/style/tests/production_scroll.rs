use foundation::{CSSPropertyID as P, Color, EOverscrollBehavior, String};
use layoutng_style::style::{computed_style::ComputedStyle, scroll_snap_data::cc};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    StyleEngine,
};
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let document = owner.GetDocument();
    let index = (0..document.NodeCount())
        .find(|&i| {
            document
                .Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap();
    unsafe { &*document.ResolvedStyleFor(index).unwrap().native_style.Get() }
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
fn scrollbar_overscroll_and_snap_reach_native_fields_with_css_wide_values() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='scrollbar-color:currentColor rgb(255,0,0);scrollbar-gutter:both-edges stable;overscroll-behavior:contain none;scroll-snap-align:start end;scroll-snap-type:inline mandatory'><div id=c style='scrollbar-color:inherit;scrollbar-gutter:inherit;overscroll-behavior:inherit;scroll-snap-align:inherit;scroll-snap-type:inherit'></div><div id=i style='scrollbar-color:initial;scrollbar-gutter:initial;overscroll-behavior:initial;scroll-snap-align:initial;scroll-snap-type:initial'></div><div id=u style='scrollbar-color:unset;scrollbar-gutter:unset;overscroll-behavior:unset;scroll-snap-align:unset;scroll-snap-type:unset'></div><div id=a style='scrollbar-color:auto;scrollbar-gutter:auto;scroll-snap-type:y proximity;scroll-snap-align:center center;overscroll-behavior:chain'></div></div>");
    update(&mut owner);
    let (p, c, i, u, a) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
        native(&owner, "u"),
        native(&owner, "a"),
    );
    let colors = unsafe { &*p.ScrollbarColor() };
    assert!(colors.GetThumbColor().IsCurrentColor());
    assert_eq!(
        colors.GetTrackColor().GetColor(),
        Color::FromRGBA(255, 0, 0, 255)
    );
    assert_eq!(p.ScrollbarColor(), c.ScrollbarColor());
    assert_eq!(p.ScrollbarColor(), u.ScrollbarColor());
    assert!(i.ScrollbarColor().is_null() && a.ScrollbarColor().is_null());
    for s in [p, c] {
        assert_eq!(s.ScrollbarGutter(), 3);
        assert_eq!(
            (s.OverscrollBehaviorX(), s.OverscrollBehaviorY()),
            (EOverscrollBehavior::kContain, EOverscrollBehavior::kNone)
        );
        assert_eq!(
            *s.GetScrollSnapAlign(),
            cc::ScrollSnapAlign::new(cc::SnapAlignment::kStart, cc::SnapAlignment::kEnd)
        );
        assert_eq!(
            *s.GetScrollSnapType(),
            cc::ScrollSnapType::new(false, cc::SnapAxis::kInline, cc::SnapStrictness::kMandatory)
        );
    }
    for s in [i, u] {
        assert_eq!(s.ScrollbarGutter(), 0);
        assert_eq!(
            (s.OverscrollBehaviorX(), s.OverscrollBehaviorY()),
            (EOverscrollBehavior::kAuto, EOverscrollBehavior::kAuto)
        );
        assert_eq!(*s.GetScrollSnapAlign(), cc::ScrollSnapAlign::default());
        assert_eq!(*s.GetScrollSnapType(), cc::ScrollSnapType::default());
    }
    assert_eq!(
        (a.OverscrollBehaviorX(), a.OverscrollBehaviorY()),
        (EOverscrollBehavior::kChain, EOverscrollBehavior::kChain)
    );
    assert_eq!(
        *a.GetScrollSnapType(),
        cc::ScrollSnapType::new(false, cc::SnapAxis::kY, cc::SnapStrictness::kProximity)
    );
    assert_eq!(
        *a.GetScrollSnapAlign(),
        cc::ScrollSnapAlign::from_alignment(cc::SnapAlignment::kCenter)
    );
}
#[test]
fn logical_scroll_side_shorthands_map_by_final_writing_direction() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='direction:rtl;writing-mode:vertical-rl;scroll-margin-block:calc(3px + 2px) -7px;scroll-margin-inline:11px 13px;scroll-padding-block:10% 20px;scroll-padding-inline:auto calc(6px + 4px)'><div id=c style='scroll-margin-block:inherit;scroll-margin-inline:inherit;scroll-padding-block:inherit;scroll-padding-inline:inherit'></div><div id=i style='scroll-margin-block:initial;scroll-margin-inline:initial;scroll-padding-block:initial;scroll-padding-inline:initial'></div></div><div id=h style='direction:rtl;scroll-margin-inline:2px;scroll-padding-inline:4% 8%;scroll-margin-block:3px 9px'></div>");
    update(&mut owner);
    for s in [native(&owner, "p"), native(&owner, "c")] {
        assert_eq!(
            (
                s.ScrollMarginRight(),
                s.ScrollMarginLeft(),
                s.ScrollMarginBottom(),
                s.ScrollMarginTop()
            ),
            (5.0, -7.0, 11.0, 13.0)
        );
        assert_eq!(s.ScrollPaddingRight().PercentValue(), 10.0);
        assert_eq!(s.ScrollPaddingLeft().Pixels(), 20.0);
        assert!(s.ScrollPaddingBottom().IsAuto());
        assert_eq!(s.ScrollPaddingTop().Pixels(), 10.0);
    }
    let i = native(&owner, "i");
    assert_eq!(
        (
            i.ScrollMarginRight(),
            i.ScrollMarginLeft(),
            i.ScrollMarginBottom(),
            i.ScrollMarginTop()
        ),
        (0.0, 0.0, 0.0, 0.0)
    );
    assert!(
        i.ScrollPaddingRight().IsAuto()
            && i.ScrollPaddingLeft().IsAuto()
            && i.ScrollPaddingBottom().IsAuto()
            && i.ScrollPaddingTop().IsAuto()
    );
    let h = native(&owner, "h");
    assert_eq!(
        (
            h.ScrollMarginRight(),
            h.ScrollMarginLeft(),
            h.ScrollMarginTop(),
            h.ScrollMarginBottom()
        ),
        (2.0, 2.0, 3.0, 9.0)
    );
    assert_eq!(
        (
            h.ScrollPaddingRight().PercentValue(),
            h.ScrollPaddingLeft().PercentValue()
        ),
        (4.0, 8.0)
    );
    for (id, text, ids) in [
        (
            P::kOverscrollBehavior,
            "none",
            [P::kOverscrollBehaviorX, P::kOverscrollBehaviorY],
        ),
        (
            P::kScrollMarginBlock,
            "5px",
            [P::kScrollMarginBlockStart, P::kScrollMarginBlockEnd],
        ),
        (
            P::kScrollPaddingInline,
            "auto",
            [P::kScrollPaddingInlineStart, P::kScrollPaddingInlineEnd],
        ),
    ] {
        let properties = parse(id, text);
        assert_eq!(
            properties
                .iter()
                .map(|p| p.PropertyID())
                .collect::<Vec<_>>(),
            ids
        );
        assert!(properties
            .iter()
            .all(|p| p.ShorthandID() == id && !p.IsImplicit()));
        assert_eq!(
            properties[0].Value().CssText(),
            properties[1].Value().CssText()
        );
    }
}
#[test]
fn scroll_consumers_reject_invalid_values_and_preserve_runtime_boundaries() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, text) in [
        (P::kScrollbarColor, "red"),
        (P::kScrollbarColor, "auto red"),
        (P::kScrollbarGutter, "both-edges"),
        (P::kScrollbarGutter, "stable stable"),
        (P::kScrollbarGutter, "auto stable"),
        (P::kOverscrollBehavior, "none contain auto"),
        (P::kOverscrollBehaviorX, "scroll"),
        (P::kScrollSnapAlign, "start end center"),
        (P::kScrollSnapType, "none mandatory"),
        (P::kScrollSnapType, "mandatory"),
        (P::kScrollSnapType, "x proximity mandatory"),
        (P::kScrollMarginBlock, "10%"),
        (P::kScrollPaddingInline, "-2px"),
        (P::kScrollPaddingInline, "1px 2px 3px"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap_or_else(|| panic!("accepted invalid {text}"))
            .kind,
            PropertyParseErrorKind::Invalid
        );
    }
    assert_eq!(
        ParseProperty(
            P::kScrollSnapType,
            &String::from("pair mandatory"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .err()
        .unwrap()
        .kind,
        PropertyParseErrorKind::Unsupported
    );
    assert_eq!(
        ParseProperty(
            P::kScrollPaddingInlineEnd,
            &String::from("2"),
            false,
            CSSParserMode::kSVGAttributeMode
        )
        .err()
        .unwrap()
        .kind,
        PropertyParseErrorKind::Invalid
    );
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder =
        layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(initial);
    for (text, expected) in [("2", 2.0), ("calc(2 + 3)", 5.0)] {
        let value = ParseProperty(
            P::kScrollMarginBlockStart,
            &String::from(text),
            false,
            CSSParserMode::kSVGAttributeMode,
        )
        .unwrap();
        style::resolver::production_style_builder::Apply(
            P::kScrollMarginBlockStart,
            &mut builder,
            None,
            value[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
        )
        .unwrap();
        assert_eq!(builder.ScrollMarginTop(), expected);
    }
    let colors = parse(P::kScrollbarColor, "Canvas CanvasText");
    assert_eq!(
        style::resolver::production_style_builder::Apply(
            P::kScrollbarColor,
            &mut builder,
            None,
            colors[0].Value(),
            16.0,
            &MediaValuesCachedData::default()
        ),
        Err(
            style::properties::longhand_dispatch::LonghandApplicationError::Unsupported(
                P::kScrollbarColor
            )
        )
    );
    assert!(builder.ScrollbarColor().is_null());
    builder.SetEffectiveZoom(2.0);
    let inherit = parse(P::kScrollMarginTop, "inherit");
    assert_eq!(
        style::resolver::production_style_builder::Apply(
            P::kScrollMarginTop,
            &mut builder,
            Some(initial),
            inherit[0].Value(),
            16.0,
            &MediaValuesCachedData::default()
        ),
        Err(
            style::properties::longhand_dispatch::LonghandApplicationError::Unsupported(
                P::kScrollMarginTop
            )
        )
    );
    assert_eq!(
        parse(P::kScrollbarGutter, "both-edges stable")[0]
            .Value()
            .CssText()
            .Utf8(),
        "stable both-edges"
    );
    assert_eq!(
        parse(P::kScrollSnapType, "x proximity")[0]
            .Value()
            .CssText()
            .Utf8(),
        "x"
    );
    assert_eq!(
        parse(P::kScrollSnapAlign, "center center")[0]
            .Value()
            .CssText()
            .Utf8(),
        "center"
    );
}
