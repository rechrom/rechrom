use dom::persistent_document::DOMAttribute;
use style::{
    media_queries::{
        container_state::*,
        production_container_query::{ContainerFallback, ContainerState},
        MediaValuesCachedData,
    },
    StyleEngine,
};
fn media() -> MediaValuesCachedData {
    MediaValuesCachedData {
        media_type: foundation::String::from("screen"),
        viewport_width: 800.0,
        viewport_height: 600.0,
        large_viewport_width: 800.0,
        large_viewport_height: 600.0,
        ..Default::default()
    }
}
fn node(owner: &dom::DOM, id: &str) -> usize {
    let d = owner.GetDocument();
    (0..d.NodeCount())
        .find(|&i| d.Node(i).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap()
}
fn width(owner: &dom::DOM, node: usize) -> f64 {
    unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(node)
            .unwrap()
            .native_style
            .Get()
    }
    .Width()
    .Pixels() as f64
}
fn state(width: f64) -> ContainerState {
    ContainerState {
        scroll_state_available: true,
        anchored_state_available: true,
        width: Some(width),
        height: Some(200.0),
        stuck_horizontal: ContainerStuckPhysical::kNo,
        stuck_vertical: ContainerStuckPhysical::kNo,
        stuck_inline: ContainerStuckLogical::kNo,
        stuck_block: ContainerStuckLogical::kNo,
        snapped: 0,
        scrollable_horizontal: 0,
        scrollable_vertical: 0,
        scrollable_inline: 0,
        scrollable_block: 0,
        scrolled_horizontal: ContainerScrolled::kNone,
        scrolled_vertical: ContainerScrolled::kNone,
        scrolled_inline: ContainerScrolled::kNone,
        scrolled_block: ContainerScrolled::kNone,
        anchored_fallback: ContainerFallback::None(),
        abs_container_direction: foundation::WritingDirectionMode::new(
            foundation::WritingMode::kHorizontalTb,
            foundation::TextDirection::kLtr,
        ),
    }
}
#[test]
fn computed_ancestor_style_queries_match_and_invalidate_without_layout() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=outer style='--tone:blue;--gap:20px'><div id=inner><span id=target></span></div></div>");
    let sheet=style::ParseCSS("#target{width:1px}@container style(--tone:blue){#target{width:31px}}@container style(10px < --gap < 25px){#target{height:47px}}@container style(--absent){#target{width:99px}}@container style(--tone:revert){#target{width:101px}}");
    assert_eq!(
        sheet
            .rules
            .iter()
            .filter(|r| !r.container_conditions.is_empty())
            .count(),
        4
    );
    owner.GetDocumentMut().AppendStyleSheet(sheet);
    let outer = node(&owner, "outer");
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    assert_eq!(width(&owner, target), 31.0);
    assert_eq!(
        unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        }
        .Height()
        .Pixels(),
        47.0
    );
    owner.GetDocumentMut().SetAttribute(
        outer,
        DOMAttribute {
            local_name: "style".into(),
            value: "--tone:red;--gap:30px".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(width(&owner, target), 1.0);
    assert!(unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    }
    .Height()
    .IsAuto());
}
#[test]
fn container_size_snapshot_changes_recompute_matches() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=outer><span id=target></span></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#outer{container-type:inline-size;container-name:card}#target{width:1px}@container card (width > 300px){#target{width:55px}}"));
    let outer = node(&owner, "outer");
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine
        .SetContainerState(&mut owner, outer, Some(state(400.0)))
        .unwrap();
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    assert_eq!(width(&owner, target), 55.0);
    assert!(!engine
        .SetContainerState(&mut owner, outer, Some(state(400.0)))
        .unwrap());
    assert!(engine
        .SetContainerState(&mut owner, outer, Some(state(250.0)))
        .unwrap());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(width(&owner, target), 1.0);
    engine.SetContainerState(&mut owner, outer, None).unwrap();
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(width(&owner, target), 1.0);
}
#[test]
fn scroll_state_snapshots_and_computed_font_units_use_the_selected_owner() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=outer><span id=target></span></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("html{font-size:10px}#outer{container-type:scroll-state;font-size:20px;--gap:2rem}#target{width:1px}@container scroll-state(snapped:x){#target{width:71px}}@container style(19px < --gap < 21px){#target{height:63px}}"));
    let outer = node(&owner, "outer");
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    let mut snapshot = state(400.0);
    snapshot.snapped = ContainerSnapped::kX as u32;
    engine
        .SetContainerState(&mut owner, outer, Some(snapshot))
        .unwrap();
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    assert_eq!(width(&owner, target), 71.0);
    assert_eq!(
        unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        }
        .Height()
        .Pixels(),
        63.0
    );
    engine
        .SetContainerState(&mut owner, outer, Some(state(400.0)))
        .unwrap();
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(width(&owner, target), 1.0);
    let document = owner.GetDocument();
    let native = unsafe { &*document.ResolvedStyleFor(outer).unwrap().native_style.Get() };
    let values =
        style::media_queries::production_container_query::DocumentMediaValues::ForContainer(
            document,
            outer,
            &media(),
            native,
            None,
        );
    assert_eq!(values.ContainerElement(), Some(outer));
    use style::media_queries::MediaValues;
    assert!(std::ptr::eq(values.GetDocument().unwrap(), document));
    assert_eq!(values.Width(), None);
    assert_eq!(
        values.ComputeLength(2.0, style::css_primitive_value::UnitType::kEms),
        40.0
    );
    use style::media_queries::media_query_evaluator::MediaQueryFrameValues;
    assert!(values
        .CreateDynamicIfFrameExists()
        .unwrap()
        .GetDocument()
        .is_some());
}
