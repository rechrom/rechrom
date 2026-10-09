use foundation::{CSSPropertyID as P, CSSValueID as K, String as CSSString};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    production_css_value as values,
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
fn parse(id: P, css: &str) -> Vec<values::PropertyValue> {
    ParseProperty(
        id,
        &CSSString::from(css),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap()
}
fn apply(b: &mut ComputedStyleBuilder, id: P, css: &str) {
    Apply(id, b, None, &parse(id, css)[0].ValueRef(), 16., &media()).unwrap();
}

#[test]
fn initial_letter_native_drop_raise_omitted_integer_and_math() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<i id=a></i><i id=b></i><i id=c></i><i id=d></i><i id=e></i><i id=f></i>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#a{initial-letter:3.9}#b{initial-letter:drop 3.9}#c{initial-letter:3.9 raise}#d{initial-letter:2.5 7}#e{initial-letter:calc(2 + .5) calc(1.6)}#f{initial-letter:1e30 drop}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let a = native(&owner, "a").InitialLetter();
    assert_eq!((a.Size(), a.Sink()), (3.9, 3));
    assert!(!a.IsDrop() && !a.IsRaise() && !a.IsIntegerSink());
    let b = native(&owner, "b").InitialLetter();
    assert_eq!((b.Size(), b.Sink()), (3.9, 3));
    assert!(b.IsDrop());
    let c = native(&owner, "c").InitialLetter();
    assert_eq!((c.Size(), c.Sink()), (3.9, 1));
    assert!(c.IsRaise());
    let d = native(&owner, "d").InitialLetter();
    assert_eq!((d.Size(), d.Sink()), (2.5, 7));
    assert!(d.IsIntegerSink());
    let e = native(&owner, "e").InitialLetter();
    assert_eq!((e.Size(), e.Sink()), (2.5, 2));
    assert!(e.IsIntegerSink());
    assert_eq!(native(&owner, "f").InitialLetter().Sink(), i32::MAX);
}

#[test]
fn marker_modes_are_stable_and_scope_owns_native_scoped_names() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p><i id=c></i></div><i id=t></i><i id=l></i><i id=a></i>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#p{scroll-marker-group:before;trigger-scope:--MiXeD,--\\41,--MiXeD}#c{scroll-marker-group:inherit;trigger-scope:inherit}#t{scroll-marker-group:after tabs}#l{scroll-marker-group:before links}#a{trigger-scope:all}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    let c = native(&owner, "c");
    assert_eq!(p.GetScrollMarkerGroup(), c.GetScrollMarkerGroup());
    let group = unsafe { &*p.GetScrollMarkerGroup() };
    assert!(group.PositionBefore() && group.IsInLinksMode());
    let tabs = unsafe { &*native(&owner, "t").GetScrollMarkerGroup() };
    assert!(tabs.PositionAfter() && tabs.IsInTabsMode());
    let links = unsafe { &*native(&owner, "l").GetScrollMarkerGroup() };
    assert!(links.PositionBefore() && links.IsInLinksMode());
    assert_eq!(p.TriggerScope().Names(), c.TriggerScope().Names());
    let names = unsafe { &*p.TriggerScope().Names() }.GetNames();
    assert_eq!(
        names
            .iter()
            .map(|n| unsafe { &*n.Get() }.GetName().Utf8())
            .collect::<Vec<_>>(),
        ["--MiXeD", "--A", "--MiXeD"]
    );
    assert!(names
        .iter()
        .all(|n| unsafe { &*n.Get() }.GetTreeScope().is_null()));
    let all = native(&owner, "a").TriggerScope();
    assert!(all.IsAll() && all.AllTreeScope().is_null() && all.Names().is_null());
}

#[test]
fn css_wide_resets_and_explicit_inheritance_reach_native_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p><i id=c></i><i id=i></i><i id=u></i><i id=n></i></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#p{initial-letter:4 drop;scroll-marker-group:after tabs;trigger-scope:--a}#c{initial-letter:inherit;scroll-marker-group:inherit;trigger-scope:inherit}#i{initial-letter:2;initial-letter:initial;scroll-marker-group:before;scroll-marker-group:initial;trigger-scope:all;trigger-scope:initial}#u{initial-letter:unset;scroll-marker-group:unset;trigger-scope:unset}#n{initial-letter:normal;scroll-marker-group:none;trigger-scope:none}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    let c = native(&owner, "c");
    assert_eq!(p.InitialLetter(), c.InitialLetter());
    assert_eq!(p.GetScrollMarkerGroup(), c.GetScrollMarkerGroup());
    assert!(p.TriggerScope() == c.TriggerScope());
    assert!(c.HasExplicitInheritance() && p.ChildHasExplicitInheritance());
    for id in ["i", "u", "n"] {
        let s = native(&owner, id);
        assert!(s.InitialLetter().IsNormal());
        assert!(s.GetScrollMarkerGroup().is_null());
        assert!(s.TriggerScope().IsNone());
    }
    for id in [P::kInitialLetter, P::kScrollMarkerGroup, P::kTriggerScope] {
        for keyword in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            assert_eq!(parse(id, keyword).len(), 1);
        }
    }
}

#[test]
fn invalid_grammar_and_typed_native_boundaries_are_explicit() {
    for (id, css) in [
        (P::kInitialLetter, "0"),
        (P::kInitialLetter, ".9"),
        (P::kInitialLetter, "drop .5"),
        (P::kInitialLetter, "raise"),
        (P::kInitialLetter, "2 0"),
        (P::kInitialLetter, "2 1.5"),
        (P::kInitialLetter, "2 drop 3"),
        (P::kInitialLetter, "2 3 raise"),
        (P::kInitialLetter, "2px"),
        (P::kScrollMarkerGroup, "tabs"),
        (P::kScrollMarkerGroup, "none tabs"),
        (P::kScrollMarkerGroup, "before tabs links"),
        (P::kScrollMarkerGroup, "after before"),
        (P::kTriggerScope, "foo"),
        (P::kTriggerScope, "'--a'"),
        (P::kTriggerScope, "none,--a"),
        (P::kTriggerScope, "all,--a"),
        (P::kTriggerScope, "--a,"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &CSSString::from(css),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Invalid,
            "{id:?} {css}"
        );
    }
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    for css in ["calc(.5)", "2 2147483648"] {
        assert_eq!(
            Apply(
                P::kInitialLetter,
                &mut b,
                None,
                &parse(P::kInitialLetter, css)[0].ValueRef(),
                16.,
                &media()
            ),
            Err(LonghandApplicationError::Unsupported(P::kInitialLetter))
        );
    }
    apply(&mut b, P::kInitialLetter, "2 calc(-100)");
    assert_eq!(b.InitialLetter().Sink(), 1);
    // Hidden/internal direct values must not bypass real payload grammar.
    assert_eq!(
        Apply(
            P::kTriggerScope,
            &mut b,
            None,
            &values::identifier(K::kAll),
            16.,
            &media()
        ),
        Err(LonghandApplicationError::InvalidValue(P::kTriggerScope))
    );
    assert_eq!(
        Apply(
            P::kScrollMarkerGroup,
            &mut b,
            None,
            &values::identifier(K::kTabs),
            16.,
            &media()
        ),
        Err(LonghandApplicationError::InvalidValue(
            P::kScrollMarkerGroup
        ))
    );
    let prefixed = parse(P::kInitialLetter, "raise 2.5");
    let suffix = parse(P::kInitialLetter, "2.5 raise");
    assert!(prefixed[0].ValueRef() == suffix[0].ValueRef());
}
