use foundation::{CSSPropertyID as P, CSSValueID as V, EPositionTryOrder, String};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    computed_style_constants::{PositionVisibility, TryTactic},
    position_area::PositionAreaRegion as R,
    style_position_anchor::Type,
};
use style::{
    css_value::CSSValuePayload,
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
fn anchor_names_scope_and_position_anchor_preserve_document_root_native_identity() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=p style='anchor-name:--Foo,--foo;anchor-scope:--Foo,--other;position-anchor:--Foo'><div id=c style='anchor-name:inherit;anchor-scope:inherit;position-anchor:inherit'></div><div id=i style='anchor-name:initial;anchor-scope:initial;position-anchor:initial'></div><div id=u style='anchor-name:unset;anchor-scope:unset;position-anchor:unset'></div><div id=a style='anchor-name:none;anchor-scope:all;position-anchor:auto'></div><div id=n style='anchor-scope:none;position-anchor:none'></div></div>");
    update(&mut owner);
    let (p, c, i, u, a, n) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
        native(&owner, "u"),
        native(&owner, "a"),
        native(&owner, "n"),
    );
    let names = unsafe { &*p.AnchorName().Get() }.GetNames();
    assert_eq!(names.len(), 2);
    for (name, expected) in names.iter().zip(["--Foo", "--foo"]) {
        let name = unsafe { &*name.Get() };
        assert_eq!(name.GetName().Utf8(), expected);
        assert!(name.GetTreeScope().is_null());
    }
    assert_eq!(p.AnchorName().Get(), c.AnchorName().Get());
    assert!(p.AnchorScope() == c.AnchorScope());
    assert!(!p.AnchorScope().IsAll());
    let scoped = unsafe { &*p.AnchorScope().Names() }.GetNames();
    assert_eq!(scoped.len(), 2);
    assert_eq!(p.PositionAnchor().GetType(), Type::kName);
    assert_eq!(p.PositionAnchor().GetName().GetName().Utf8(), "--Foo");
    assert!(p.PositionAnchor().GetName().GetTreeScope().is_null());
    assert!(p.PositionAnchor() == c.PositionAnchor());
    for s in [i, u] {
        assert!(s.AnchorName().Get().is_null());
        assert!(s.AnchorScope().IsNone());
        assert_eq!(s.PositionAnchor().GetType(), Type::kNormal);
    }
    assert!(a.AnchorName().Get().is_null());
    assert!(a.AnchorScope().IsAll());
    assert!(a.AnchorScope().AllTreeScope().is_null());
    assert_eq!(a.PositionAnchor().GetType(), Type::kAuto);
    assert!(n.AnchorScope().IsNone());
    assert_eq!(n.PositionAnchor().GetType(), Type::kNone);
    let all = parse(P::kAnchorScope, "all");
    let value = all[0].Value();
    assert!(!value.IsScopedValue());
    assert!(
        matches!(value.Payload(),CSSValuePayload::kScopedKeywordClass(k) if k.GetValueID()==V::kAll)
    );
    let populated = value.PopulateWithTreeScope(None);
    assert!(populated.IsScopedValue());
    assert!(value != populated);
    assert!(
        matches!(populated.Payload(),CSSValuePayload::kScopedKeywordClass(k) if k.GetPopulatedTreeScope().is_null())
    );
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    style::resolver::production_style_builder::Apply(
        P::kAnchorScope,
        &mut b,
        None,
        populated,
        16.0,
        &MediaValuesCachedData::default(),
    )
    .unwrap();
    assert!(b.AnchorScope().IsAll());
}
#[test]
fn position_area_visibility_and_try_options_reach_native_fields_and_reset() {
    let _heap = foundation::LayoutHeapScope::new();
    assert_eq!(
        parse(P::kPositionArea, "top left")[0]
            .Value()
            .CssText()
            .Utf8(),
        "left top"
    );
    assert_eq!(
        parse(P::kPositionArea, "span-all top")[0]
            .Value()
            .CssText()
            .Utf8(),
        "top"
    );
    assert_eq!(
        parse(P::kPositionVisibility, "no-overflow anchors-visible")[0]
            .Value()
            .CssText()
            .Utf8(),
        "anchors-visible no-overflow"
    );
    let parsed = parse(
        P::kPositionTry,
        "most-width flip-y flip-x --preferred, top span-right, --other",
    );
    assert_eq!(parsed.len(), 2);
    for p in &parsed {
        assert_eq!(p.ShorthandID(), P::kPositionTry);
        assert!(!p.IsImplicit());
    }
    assert_eq!(
        parsed[1].Value().CssText().Utf8(),
        "--preferred flip-y flip-x, span-right top, --other"
    );
    let mut owner=html::html_parser::ParseHTML("<div id=p style='position-area:top left;position-visibility:no-overflow anchors-visible;position-try:most-width flip-y flip-x --preferred,top span-right,--other'><div id=c style='position-area:inherit;position-visibility:inherit;position-try:inherit'></div><div id=i style='position-area:initial;position-visibility:initial;position-try:initial'></div><div id=u style='position-area:unset;position-visibility:unset;position-try:unset'></div><div id=r style='position-try-order:most-height;position-try-fallbacks:--old;position-try:none;position-area:none;position-visibility:always'></div><div id=l style='position-area:self-inline-end self-block-start;position-try-fallbacks:flip-block flip-inline flip-start'></div><div id=s style='position-area:span-start;position-try:bottom'></div></div>");
    update(&mut owner);
    let (p, c, i, u, r, l, s) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
        native(&owner, "u"),
        native(&owner, "r"),
        native(&owner, "l"),
        native(&owner, "s"),
    );
    assert_eq!(p.GetPositionArea().FirstStart(), R::kLeft);
    assert_eq!(p.GetPositionArea().FirstEnd(), R::kLeft);
    assert_eq!(p.GetPositionArea().SecondStart(), R::kTop);
    assert!(p.HasAnchorFunctions());
    assert_eq!(p.GetPositionArea(), c.GetPositionArea());
    assert!(c.HasAnchorFunctions());
    assert_eq!(
        p.GetPositionVisibility(),
        PositionVisibility::kAnchorsVisible | PositionVisibility::kNoOverflow
    );
    assert_eq!(p.GetPositionVisibility(), c.GetPositionVisibility());
    assert_eq!(p.PositionTryOrder(), EPositionTryOrder::kMostWidth);
    assert_eq!(c.PositionTryOrder(), p.PositionTryOrder());
    assert_eq!(
        p.GetPositionTryFallbacks().Get(),
        c.GetPositionTryFallbacks().Get()
    );
    let fs = unsafe { &*p.GetPositionTryFallbacks().Get() };
    let fallbacks = fs.GetFallbacks();
    assert_eq!(fallbacks.len(), 3);
    assert_eq!(
        unsafe { &*fallbacks[0].GetPositionTryName() }
            .GetName()
            .Utf8(),
        "--preferred"
    );
    assert!(unsafe { &*fallbacks[0].GetPositionTryName() }
        .GetTreeScope()
        .is_null());
    assert_eq!(
        *fallbacks[0].GetTryTactic(),
        [TryTactic::kFlipY, TryTactic::kFlipX, TryTactic::kNone]
    );
    assert_eq!(fallbacks[1].GetPositionArea().FirstStart(), R::kCenter);
    assert_eq!(fallbacks[1].GetPositionArea().FirstEnd(), R::kRight);
    assert_eq!(fallbacks[1].GetPositionArea().SecondStart(), R::kTop);
    assert!(fallbacks[0].Matches(&fallbacks[0]));
    assert!(!fallbacks[0].Matches(&fallbacks[2]));
    let mut names = foundation::HashSet::new();
    names.insert(foundation::AtomicString::from_str("--other"));
    assert!(fs.HasPositionTryName(&names));
    for native in [i, u] {
        assert!(native.GetPositionArea().IsNone());
        assert_eq!(
            native.GetPositionVisibility(),
            PositionVisibility::kAnchorsVisible
        );
        assert_eq!(native.PositionTryOrder(), EPositionTryOrder::kNormal);
        assert!(native.GetPositionTryFallbacks().Get().is_null());
    }
    assert!(r.GetPositionArea().IsNone());
    assert_eq!(r.GetPositionVisibility(), PositionVisibility::kAlways);
    assert_eq!(r.PositionTryOrder(), EPositionTryOrder::kNormal);
    assert!(r.GetPositionTryFallbacks().Get().is_null());
    assert_eq!(l.GetPositionArea().FirstStart(), R::kSelfBlockStart);
    assert_eq!(l.GetPositionArea().SecondStart(), R::kSelfInlineEnd);
    assert_eq!(
        *unsafe { &*l.GetPositionTryFallbacks().Get() }.GetFallbacks()[0].GetTryTactic(),
        [
            TryTactic::kFlipBlock,
            TryTactic::kFlipInline,
            TryTactic::kFlipStart
        ]
    );
    assert_eq!(s.GetPositionArea().FirstStart(), R::kStart);
    assert_eq!(s.GetPositionArea().FirstEnd(), R::kCenter);
    assert_eq!(s.GetPositionArea().SecondStart(), R::kStart);
    assert_eq!(s.PositionTryOrder(), EPositionTryOrder::kNormal);
}
#[test]
fn anchor_invalid_grammars_and_native_capacity_remain_explicit() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, text) in [
        (P::kAnchorName, "foo"),
        (P::kAnchorName, "--a --b"),
        (P::kAnchorName, "--a,"),
        (P::kAnchorScope, "all --a"),
        (P::kPositionAnchor, "--a --b"),
        (P::kPositionAnchor, "default"),
        (P::kPositionArea, "top bottom"),
        (P::kPositionArea, "left inline-start"),
        (P::kPositionArea, "any"),
        (P::kPositionArea, "none top"),
        (P::kPositionVisibility, "anchors-valid"),
        (P::kPositionVisibility, "always no-overflow"),
        (P::kPositionVisibility, "anchors-visible anchors-visible"),
        (P::kPositionTryFallbacks, "flip-x flip-x"),
        (P::kPositionTryFallbacks, "none, top"),
        (P::kPositionTryFallbacks, "flip-x --x flip-y"),
        (P::kPositionTry, "most-width"),
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
    for id in [
        P::kAnchorName,
        P::kAnchorScope,
        P::kPositionAnchor,
        P::kPositionTryFallbacks,
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from("ident(--a)"),
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
    let media = MediaValuesCachedData::default();
    let old = parse(P::kPositionTryFallbacks, "--keep");
    style::resolver::production_style_builder::Apply(
        P::kPositionTryFallbacks,
        &mut b,
        None,
        old[0].Value(),
        16.0,
        &media,
    )
    .unwrap();
    let pointer = b.GetPositionTryFallbacks().Get();
    let too_many = parse(
        P::kPositionTryFallbacks,
        "flip-x flip-y flip-start flip-inline flip-block",
    );
    assert_eq!(
        style::resolver::production_style_builder::Apply(
            P::kPositionTryFallbacks,
            &mut b,
            None,
            too_many[0].Value(),
            16.0,
            &media
        ),
        Err(
            style::properties::longhand_dispatch::LonghandApplicationError::Unsupported(
                P::kPositionTryFallbacks
            )
        )
    );
    assert_eq!(b.GetPositionTryFallbacks().Get(), pointer);
    let ordinary_all = style::production_css_value::identifier(V::kAll);
    assert_eq!(
        style::resolver::production_style_builder::Apply(
            P::kAnchorScope,
            &mut b,
            None,
            &ordinary_all,
            16.0,
            &media
        ),
        Err(
            style::properties::longhand_dispatch::LonghandApplicationError::InvalidValue(
                P::kAnchorScope
            )
        )
    );
}
