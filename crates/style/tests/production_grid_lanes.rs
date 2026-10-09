use foundation::{CSSPropertyID as P, EDisplay};
use layoutng_style::style::computed_style::ComputedStyle;
use style::{
    document_style_engine::DocumentStyleError, media_queries::MediaValuesCachedData, StyleEngine,
};

// runtime_enabled_features.json5:1641-1643 marks CSSGridLanesLayout experimental.
// Author declarations must stay hidden before CSS-wide or var() expansion.
#[test]
fn style_engine_grid_lanes_exposure_keeps_stable_grid_fields_and_native_defaults() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style>#g{display:block;grid-template-columns:10px 1fr;grid-template-rows:20px;grid-template-areas:'a a';grid-lanes:row 99px 'b';grid-lanes-direction:row fill-reverse;grid-lanes-pack:dense}</style><div id=g style='--lanes:row 30px;grid-lanes:var(--lanes);grid-lanes:initial;grid-lanes-direction:inherit;grid-lanes-pack:unset;flow-tolerance:1px;display:grid-lanes'><div id=c style='grid-lanes:inherit;display:inline grid-lanes'></div></div>",
    );
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    for id in [
        P::kGridLanes,
        P::kGridLanesDirection,
        P::kGridLanesPack,
        P::kFlowTolerance,
    ] {
        assert!(
            engine
                .Diagnostics()
                .iter()
                .any(|e| matches!(e, DocumentStyleError::Property{property,..} if *property==id)),
            "{id:?}: {:?}",
            engine.Diagnostics()
        );
    }
    let d = owner.GetDocument();
    let native = |id: &str| {
        let n = (0..d.NodeCount())
            .find(|&n| d.Node(n).FindAttribute("id").is_some_and(|a| a.value == id))
            .unwrap();
        unsafe { &*d.ResolvedStyleFor(n).unwrap().native_style.Get() }
    };
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let g = native("g");
    assert_eq!(g.Display(), EDisplay::kBlock);
    let areas = unsafe { &*g.GridTemplateAreas().Get() };
    assert_eq!((areas.row_count, areas.column_count), (1, 2));
    assert!(areas
        .named_areas
        .contains_key(&foundation::String::from("a")));
    let columns = unsafe { &*g.SpecifiedGridTemplateColumns().Get() }.GetTrackList();
    assert_eq!(columns.TrackCountWithoutAutoRepeat(), 2);
    assert_eq!(
        columns.RepeatTrackSize(0, 0).MinTrackBreadth().Pixels(),
        10.
    );
    let rows = unsafe { &*g.SpecifiedGridTemplateRows().Get() }.GetTrackList();
    assert_eq!(rows.RepeatTrackSize(0, 0).MinTrackBreadth().Pixels(), 20.);
    for s in [g, native("c")] {
        assert_eq!(s.GetGridLanesDirection(), initial.GetGridLanesDirection());
        assert_eq!(s.GridLanesPack(), initial.GridLanesPack());
        assert_eq!(s.GetFlowTolerance(), initial.GetFlowTolerance());
    }
    assert!(native("c").SpecifiedGridTemplateColumns().Get().is_null());
}
