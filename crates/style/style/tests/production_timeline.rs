use foundation::{CSSPropertyID as P, Length, String as CSSString};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    computed_style_constants::TimelineAxis as Axis,
};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    properties::longhand_dispatch::LonghandApplicationError,
    resolver::production_style_builder::Apply,
    StyleEngine,
};

fn media() -> MediaValuesCachedData {
    MediaValuesCachedData {
        em_size: 16.,
        media_type: CSSString::from("screen"),
        viewport_width: 800.,
        viewport_height: 600.,
        ..Default::default()
    }
}
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let doc = owner.GetDocument();
    let node = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap();
    unsafe { &*doc.ResolvedStyleFor(node).unwrap().native_style.Get() }
}
fn names(values: &[foundation::AtomicString]) -> Vec<std::string::String> {
    values.iter().map(|n| n.Utf8()).collect()
}

#[test]
fn timeline_shorthands_align_defaults_and_reset_native_lists() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p></div><i id=r></i>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#p{scroll-timeline:--A x,none,--C inline;view-timeline:--V 10% 20px inline,none y,--W auto -2em}#r{scroll-timeline:--Old y,--Other x;scroll-timeline:--New;view-timeline:--Old 1px 2px x;view-timeline:--New}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    assert_eq!(names(p.ScrollTimelineName()), ["--A", "", "--C"]);
    assert!(p.ScrollTimelineName()[1].IsNull());
    assert_eq!(
        p.ScrollTimelineAxis().as_slice(),
        [Axis::kX, Axis::kBlock, Axis::kInline]
    );
    assert_eq!(names(p.ViewTimelineName()), ["--V", "", "--W"]);
    assert_eq!(
        p.ViewTimelineAxis().as_slice(),
        [Axis::kInline, Axis::kY, Axis::kBlock]
    );
    let inset = p.ViewTimelineInset();
    assert_eq!(inset.len(), 3);
    assert_eq!(inset[0].GetStart(), &Length::Percent(10.));
    assert_eq!(inset[0].GetEnd(), &Length::Fixed(20.));
    assert_eq!(inset[1].GetStart(), Length::Auto());
    assert_eq!(inset[1].GetEnd(), Length::Auto());
    assert_eq!(inset[2].GetStart(), Length::Auto());
    assert_eq!(inset[2].GetEnd(), &Length::Fixed(-32.));
    let r = native(&owner, "r");
    assert_eq!(names(r.ScrollTimelineName()), ["--New"]);
    assert_eq!(r.ScrollTimelineAxis().as_slice(), [Axis::kBlock]);
    assert_eq!(names(r.ViewTimelineName()), ["--New"]);
    assert_eq!(r.ViewTimelineAxis().as_slice(), [Axis::kBlock]);
    assert_eq!(r.ViewTimelineInset().len(), 1);
    assert_eq!(r.ViewTimelineInset()[0].GetStart(), Length::Auto());
}

#[test]
fn independent_lists_css_wide_and_scope_use_real_native_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p><i id=c></i><i id=r></i><i id=u></i><i id=bad></i></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#p{scroll-timeline-name:--First,none,--Last;scroll-timeline-axis:y;view-timeline-name:--One,--Two;view-timeline-axis:x,inline,y;view-timeline-inset:auto,calc(10% + 2px) -5%,3px;timeline-scope:--First,--One}#c{scroll-timeline:inherit;view-timeline:inherit;timeline-scope:inherit}#r{scroll-timeline:initial;view-timeline:initial;timeline-scope:initial}#u{scroll-timeline:unset;view-timeline:unset;timeline-scope:unset}#bad{scroll-timeline:--Saved y;scroll-timeline:x --Bad;view-timeline:--Saved 4px;view-timeline:--Bad inline x;timeline-scope:--Saved;timeline-scope:all}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    let c = native(&owner, "c");
    assert_eq!(p.ScrollTimelineName(), c.ScrollTimelineName());
    assert_eq!(p.ScrollTimelineAxis(), c.ScrollTimelineAxis());
    assert_eq!(p.ViewTimelineName(), c.ViewTimelineName());
    assert_eq!(p.ViewTimelineAxis(), c.ViewTimelineAxis());
    assert_eq!(p.ViewTimelineInset(), c.ViewTimelineInset());
    assert_eq!(p.TimelineScope(), c.TimelineScope());
    assert_eq!(p.ScrollTimelineName().len(), 3);
    assert_eq!(p.ScrollTimelineAxis().len(), 1);
    assert_eq!(p.ViewTimelineName().len(), 2);
    assert_eq!(p.ViewTimelineAxis().len(), 3);
    assert_eq!(p.ViewTimelineInset()[0].GetStart(), Length::Auto());
    assert_eq!(p.ViewTimelineInset()[2].GetEnd(), &Length::Fixed(3.));
    assert!(p.ViewTimelineInset()[1].GetStart().IsCalculated());
    assert_eq!(p.ViewTimelineInset()[1].GetEnd(), &Length::Percent(-5.));
    assert_eq!(names(p.TimelineScope().Names()), ["--First", "--One"]);
    assert!(!p.TimelineScope().IsNone());
    assert!(!p.TimelineScope().IsAll());
    for id in ["r", "u"] {
        let s = native(&owner, id);
        assert!(s.ScrollTimelineName().is_empty());
        assert!(s.ScrollTimelineAxis().is_empty());
        assert!(s.ViewTimelineName().is_empty());
        assert!(s.ViewTimelineAxis().is_empty());
        assert!(s.ViewTimelineInset().is_empty());
        assert!(s.TimelineScope().IsNone());
    }
    let bad = native(&owner, "bad");
    assert_eq!(names(bad.ScrollTimelineName()), ["--Saved"]);
    assert_eq!(bad.ScrollTimelineAxis().as_slice(), [Axis::kY]);
    assert_eq!(bad.ViewTimelineInset()[0].GetEnd(), &Length::Fixed(4.));
    assert_eq!(names(bad.TimelineScope().Names()), ["--Saved"]);
}

#[test]
fn timeline_grammar_rejects_wrong_order_duplicates_and_disabled_flags() {
    for (id, css) in [
        (P::kScrollTimelineName, "foo"),
        (P::kScrollTimelineName, "'--quoted'"),
        (P::kScrollTimelineName, "--a,"),
        (P::kScrollTimelineAxis, "horizontal"),
        (P::kViewTimelineAxis, "x y"),
        (P::kViewTimelineInset, "1px 2px 3px"),
        (P::kViewTimelineInset, "1"),
        (P::kViewTimelineInset, "none"),
        (P::kTimelineScope, "none,--a"),
        (P::kTimelineScope, "all"),
        (P::kTimelineScope, "foo"),
        (P::kScrollTimeline, "x --a"),
        (P::kScrollTimeline, "--a 10%"),
        (P::kScrollTimeline, "--a x y"),
        (P::kViewTimeline, "10% --a"),
        (P::kViewTimeline, "--a inline x"),
        (P::kViewTimeline, "--a 1px 2px auto"),
        (P::kViewTimelineName, "ident('--a')"),
    ] {
        let error = ParseProperty(
            id,
            &CSSString::from(css),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap_or_else(|| panic!("accepted {id:?} {css}"));
        assert_eq!(
            error.kind,
            PropertyParseErrorKind::Invalid,
            "{id:?} {css}: {error:?}"
        );
    }
    let svg = ParseProperty(
        P::kViewTimelineInset,
        &CSSString::from("calc(1 + 2)"),
        false,
        CSSParserMode::kSVGAttributeMode,
    )
    .unwrap();
    assert_eq!(svg[0].ValueRef().CssText().Utf8(), "calc(1 + 2)");
    let p = ParseProperty(
        P::kScrollTimelineName,
        &CSSString::from("--MiXeD, --\\41"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(p[0].ValueRef().CssText().Utf8(), "--MiXeD, --A");
}

#[test]
fn native_timeline_inset_preserves_zoom_math_and_inheritance_boundary() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    b.SetEffectiveZoom(2.);
    let values = ParseProperty(
        P::kViewTimelineInset,
        &CSSString::from("2px -10%,calc(3px + 4px),auto"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    Apply(
        P::kViewTimelineInset,
        &mut b,
        None,
        &values[0].ValueRef(),
        16.,
        &media(),
    )
    .unwrap();
    assert_eq!(b.ViewTimelineInset()[0].GetStart(), &Length::Fixed(4.));
    assert_eq!(b.ViewTimelineInset()[0].GetEnd(), &Length::Percent(-10.));
    assert_eq!(b.ViewTimelineInset()[1].GetStart(), &Length::Fixed(14.));
    assert_eq!(b.ViewTimelineInset()[1].GetEnd(), &Length::Fixed(14.));
    let parent = b.TakeStyle();
    let parent = unsafe { &*parent };
    let mut child = ComputedStyleBuilder::from_style(initial);
    let inherited = ParseProperty(
        P::kViewTimelineInset,
        &CSSString::from("inherit"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(
        Apply(
            P::kViewTimelineInset,
            &mut child,
            Some(parent),
            &inherited[0].ValueRef(),
            16.,
            &media()
        ),
        Err(LonghandApplicationError::Unsupported(P::kViewTimelineInset))
    );
    assert!(child.ViewTimelineInset().is_empty());
    child.SetEffectiveZoom(2.);
    Apply(
        P::kViewTimelineInset,
        &mut child,
        Some(parent),
        &inherited[0].ValueRef(),
        16.,
        &media(),
    )
    .unwrap();
    assert_eq!(child.ViewTimelineInset(), parent.ViewTimelineInset());
}
