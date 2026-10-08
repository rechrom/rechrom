#![allow(non_snake_case)]
use dom::{UserInteractionState, DOM};
use layoutng_assembly::{
    fragment_tree::FragmentNode, internal::layout_input::ConstraintSpace,
    layout_engine::LayoutEngine,
};
use std::rc::Rc;

// cpp: browser/browser.cc:1733-1743
pub fn LayoutPersistentDocument(
    engine: &mut LayoutEngine,
    owner: &mut DOM,
    interaction: &UserInteractionState,
    constraints: &ConstraintSpace,
) -> FragmentNode {
    LayoutDocumentWithEngine(engine, owner, interaction, constraints);
    let fragments = engine
        .TakeLayoutResult()
        .expect("successful layout has fragments");
    Rc::try_unwrap(fragments).unwrap_or_else(|shared| (*shared).clone())
}

/// Synchronize the document's resident tree, then update the retained engine's
/// current fragment snapshot through its single layout/export entry point.
pub(crate) fn LayoutDocumentWithEngine(
    engine: &mut LayoutEngine,
    owner: &mut DOM,
    interaction: &UserInteractionState,
    constraints: &ConstraintSpace,
) {
    let _trace = browser_tracing::span("layout", "Page.LayoutDocumentWithEngine");
    // Tree synchronization, layout and export form one synchronous lifecycle.
    // Their nested scopes must not each collect every resident Page on this
    // thread. Allocation/root changes still collect at this outer scope exit.
    let profile = std::env::var_os("BROWSER_PROFILE_INPUT")
        .is_some()
        .then(std::time::Instant::now);
    let mut heap_scope = foundation::LayoutHeapScope::new();
    let synchronize = browser_tracing::span("layout", "Page.SynchronizeLayoutTree");
    owner.EmitConstraints(constraints, |mutation| {
        engine.ApplyMutation(mutation);
    });
    owner.EmitLayoutMutations(interaction, |mutation| {
        engine.ApplyMutation(mutation);
    });
    drop(synchronize);
    let build_done = profile.map(|start| start.elapsed());
    engine.Layout();
    let layout_done = profile.map(|start| start.elapsed());
    heap_scope.AllowUnchangedReuse();
    // The engine roots the native tree throughout collection. Include
    // this collection in diagnostics rather than ending the timer before Drop.
    {
        let _trace = browser_tracing::span("layout", "LayoutHeapScopeExit");
        drop(heap_scope);
    }
    if let Some(start) = profile {
        let elapsed = start.elapsed();
        eprintln!("page-layout-phase-profile build_ms={:.3} layout_export_ms={:.3} gc_ms={:.3} total_ms={:.3}",
            build_done.unwrap().as_secs_f64() * 1000.0,
            (layout_done.unwrap() - build_done.unwrap()).as_secs_f64() * 1000.0,
            (elapsed - layout_done.unwrap()).as_secs_f64() * 1000.0,
            elapsed.as_secs_f64() * 1000.0);
    }
}

/// Synchronize paint-only style inputs and export a current fragment snapshot
/// from the resident layout result. This is Blink's ordinary paint-only style
/// lifecycle: style and PrePaint/Paint advance, while layout geometry remains
/// the previously committed result.
pub(crate) fn ExportPaintOnlyDocumentWithEngine(
    engine: &mut LayoutEngine,
    owner: &mut DOM,
    interaction: &UserInteractionState,
) -> Option<Rc<FragmentNode>> {
    let _trace = browser_tracing::span("layout", "Page.ExportPaintOnlyDocumentWithEngine");
    let mut heap_scope = foundation::LayoutHeapScope::new();
    owner.EmitLayoutMutations(interaction, |mutation| {
        engine.ApplyMutation(mutation);
    });
    if !engine.ExportPaintOnly() {
        return None;
    }
    heap_scope.AllowUnchangedReuse();
    engine.TakeLayoutResult()
}

/// Direct animation update for compositor-friendly paint properties. The Page
/// supplies only nodes sampled in the current animation batch; any missing or
/// geometry-dirty native object rejects the whole fast path.
pub(crate) fn ExportTargetedPaintOnlyDocumentWithEngine(
    engine: &mut LayoutEngine,
    owner: &DOM,
    nodes: &[u64],
) -> Option<Rc<FragmentNode>> {
    let _trace = browser_tracing::span("layout", "Page.ExportTargetedPaintOnlyDocumentWithEngine");
    if nodes.is_empty() {
        return None;
    }
    for &node_id in nodes {
        let mut applied = false;
        if !owner.EmitPaintStyle(node_id, |mutation| {
            applied = engine.ApplyMutation(mutation);
        }) || !applied
        {
            return None;
        }
    }
    if !engine.ExportTargetedPaintOnly() {
        return None;
    }
    engine.TakeLayoutResult()
}

/// A proven offset-only change retains the completed frame's geometry. Mixed
/// mutations still enter LayoutEngine::Layout and verify result identity before
/// refreshing paint-only scroll, sticky and scrollbar properties.
pub(crate) use layoutng_assembly::layout_engine::ScrollLayoutUpdate;

#[cfg(test)]
pub(crate) fn LayoutPersistentScrollDocument(
    engine: &mut LayoutEngine,
    owner: &mut DOM,
    constraints: &ConstraintSpace,
    changes: &[(u64, layoutng_assembly::internal::layout_input::Offset)],
    fragments: &mut FragmentNode,
    scroll_only: bool,
) -> bool {
    LayoutPersistentScrollDocumentWithMode(
        engine,
        owner,
        constraints,
        changes,
        fragments,
        scroll_only,
    )
    .is_some()
}

pub(crate) fn LayoutPersistentScrollDocumentWithMode(
    engine: &mut LayoutEngine,
    owner: &mut DOM,
    constraints: &ConstraintSpace,
    changes: &[(u64, layoutng_assembly::internal::layout_input::Offset)],
    fragments: &mut FragmentNode,
    scroll_only: bool,
) -> Option<ScrollLayoutUpdate> {
    fn contains(node: &FragmentNode, id: u64) -> bool {
        node.node_id == id || node.children.iter().any(|child| contains(child, id))
    }
    if !owner.GetDocument().StyleState().impact.IsEmpty()
        || !changes.iter().all(|&(id, _)| contains(fragments, id))
    {
        return None;
    }
    for &(id, offset) in changes {
        let mut applied = false;
        if !owner.EmitScrollOffset(id, offset, |mutation| {
            applied = engine.ApplyMutation(mutation);
        }) || !applied
        {
            return None;
        }
    }
    if !scroll_only {
        owner.EmitConstraints(constraints, |mutation| {
            engine.ApplyMutation(mutation);
        });
    }
    engine.UpdateScrollLayout(fragments, scroll_only)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::persistent_document::DOMNodeType;
    use layoutng_assembly::internal::{
        layout_node_metadata::{Element, Text},
        layout_object::LayoutObject,
    };
    use layoutng_assembly::layout_engine::LayoutMutation;
    use std::collections::BTreeMap;
    fn resolve(owner: &mut DOM) {
        dom::style_resolver::ResolveComputedStyles(
            owner,
            &dom::style_resolver::StyleEnvironment::default(),
            &[],
        );
    }
    fn dom_id(owner: &DOM, name: &str) -> usize {
        (0..owner.GetDocument().NodeCount())
            .find(|&i| {
                owner
                    .GetDocument()
                    .Node(i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == name)
            })
            .unwrap()
    }
    fn native(root: *mut LayoutObject, id: u64) -> *mut LayoutObject {
        let mut object = root;
        while !object.is_null() {
            let node = unsafe { &*object }.GetNode();
            if !node.is_null() && unsafe { &*node }.InputId() == id {
                return object;
            }
            object = unsafe { &*object }.NextInPreOrder(root);
        }
        std::ptr::null_mut()
    }
    fn cached(root: *mut LayoutObject) -> *const layoutng_assembly::layout_result::LayoutResult {
        let b = foundation::DynamicTo::<layoutng_assembly::internal::layout_box::LayoutBox>(root);
        unsafe { &*b }.GetLayoutResult(0)
    }
    fn fragment(root: &FragmentNode, id: u64) -> &FragmentNode {
        fn find(root: &FragmentNode, id: u64) -> Option<&FragmentNode> {
            if root.node_id == id {
                return Some(root);
            }
            root.children.iter().find_map(|c| find(c, id))
        }
        find(root, id).expect("fragment for DOM node")
    }
    #[test]
    fn engine_owns_tree_after_document_release() {
        crate::native_test_thread::run(|| {
            let mut owner = html::html_parser::ParseHTML(
                "<html><body><div style='display:block;width:80px;height:30px'></div></body></html>",
            );
            resolve(&mut owner);
            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let cs = crate::CreateBrowserConstraints(320, 200);
            engine.ApplyMutation(LayoutMutation::Constraints(&cs));
            let interaction = UserInteractionState::default();
            let root = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &interaction,
                &mut engine,
            );
            let same_root = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &interaction,
                &mut engine,
            );
            assert_eq!(root, same_root);
            let stats = engine.GetLayoutTree().unwrap().UpdateStats();
            assert!(stats.reused >= 3, "{stats:?}");
            assert_eq!((stats.created, stats.updated, stats.removed), (0, 0, 0));
            // Document no longer roots native objects. Engine independently
            // keeps the input tree and its constraints alive through GC/Layout.
            drop(owner);
            foundation::CollectLayoutHeapForTesting();
            assert_eq!(
                engine.GetLayoutTree().unwrap().Root() as *const LayoutObject,
                root.cast_const()
            );
            engine.Layout();
            assert_eq!(engine.GetLayoutResult().unwrap().size.width, 320.0);
            assert!(!engine
                .GetLayoutTree()
                .unwrap()
                .Root()
                .SlowFirstChild()
                .is_null());
        });
    }

    #[test]
    fn scroll_layout_entry_retains_result_and_matches_fresh_layout() {
        crate::native_test_thread::run(|| {
            use layoutng_assembly::internal::layout_input::Offset;

            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner=html::html_parser::ParseHTML("<html><body style='margin:0'><div id=s style='width:100px;height:40px;overflow:hidden;zoom:80%'><div style='width:200px;height:120px;background:red'>scrolling text</div></div></body></html>");
            resolve(&mut owner);
            let cs = crate::CreateBrowserConstraints(320, 200);
            let assembly = crate::CreateLayoutAssembly();
            let mut fragments = LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            owner.GetDocumentMut().StyleStateMut().impact = Default::default();
            let index = dom_id(&owner, "s");
            let id = owner.GetDocument().Node(index).Id();

            let root =
                engine.GetLayoutTree().unwrap().Root() as *const LayoutObject as *mut LayoutObject;
            let before = cached(root);
            let object = native(root, id);
            let object_result = cached(object);
            let pixels = |f: &FragmentNode| {
                raster::pure_replay::RasterizeDisplayItemList(
                    &paint::paint_engine::Paint(f),
                    320,
                    200,
                )
            };
            for offset in [
                Offset { x: 25.0, y: 30.0 },
                Offset { x: 100.0, y: 80.0 },
                Offset::default(),
            ] {
                owner.GetDocumentMut().SetScrollOffset(index, offset);
                assert!(LayoutPersistentScrollDocument(
                    &mut engine,
                    &mut owner,
                    &cs,
                    &[(id, offset)],
                    &mut fragments,
                    false,
                ));
                assert_eq!(
                    before,
                    cached(root),
                    "layout entry must reuse root geometry"
                );
                assert_eq!(
                    object_result,
                    cached(object),
                    "scroller geometry remains valid"
                );
                assert_eq!(fragment(&fragments, id).paint.scroll_offset, offset);
                let node = unsafe { &*object }.GetNode();
                let element = foundation::DynamicTo::<Element>(node);
                assert_eq!(
                    unsafe { &*element }
                        .InputElementData()
                        .as_ref()
                        .unwrap()
                        .scroll_offset,
                    offset
                );
                let mut fresh_engine = LayoutEngine::new(&assembly);
                let fresh = LayoutPersistentDocument(
                    &mut fresh_engine,
                    &mut owner,
                    &UserInteractionState::default(),
                    &cs,
                );
                assert!(
                    pixels(&fragments) == pixels(&fresh),
                    "scroll paint must match complete fresh layout"
                );
            }
        });
    }

    #[test]
    fn positioned_scroll_descendants_refresh_paint_properties() {
        crate::native_test_thread::run(|| {
            use layoutng_assembly::internal::layout_input::Offset;
            for position in ["sticky", "fixed", "absolute"] {
                let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
                let mut owner=html::html_parser::ParseHTML(&format!("<html><body><div id=s style='height:40px;overflow:scroll'><div style='position:{position};top:0;height:20px'></div><div style='height:200px'></div></div></body></html>"));
                resolve(&mut owner);
                let cs = crate::CreateBrowserConstraints(320, 200);
                let mut fragments = LayoutPersistentDocument(
                    &mut engine,
                    &mut owner,
                    &UserInteractionState::default(),
                    &cs,
                );
                owner.GetDocumentMut().StyleStateMut().impact = Default::default();
                let index = dom_id(&owner, "s");
                let id = owner.GetDocument().Node(index).Id();
                let offset = Offset { x: 0.0, y: 10.0 };
                owner.GetDocumentMut().SetScrollOffset(index, offset);
                assert!(
                    LayoutPersistentScrollDocument(
                        &mut engine,
                        &mut owner,
                        &cs,
                        &[(id, offset)],
                        &mut fragments,
                        false,
                    ),
                    "{position} refreshes paint properties through the layout entry"
                );
                let after = LayoutPersistentDocument(
                    &mut engine,
                    &mut owner,
                    &UserInteractionState::default(),
                    &cs,
                );
                assert_eq!(fragment(&after, id).paint.scroll_offset, offset);
            }
        });
    }

    #[test]
    fn scroll_metadata_reuses_layout_with_positioned_and_floating_descendants() {
        crate::native_test_thread::run(|| {
            use layoutng_assembly::internal::layout_input::Offset;

            for placement in [
                "position:fixed;top:12px",
                "position:absolute;top:12px",
                "position:sticky;top:12px",
                "float:left",
            ] {
                for root_scroll in [false, true] {
                    let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
                    let mut owner = html::html_parser::ParseHTML(&format!("<html><body style='margin:0'><div id=s style='position:relative;width:180px;height:80px;overflow:scroll;background:white'><div id=p style='{placement};width:40px;height:25px;background:blue'></div><svg width='10' height='10'><rect width='10' height='10' fill='blue'/></svg><div style='width:220px;height:350px;background:red'>text</div></div><div style='height:600px;background:green'></div></body></html>"));
                    resolve(&mut owner);
                    let cs = crate::CreateBrowserConstraints(320, 200);
                    let assembly = crate::CreateLayoutAssembly();
                    let mut persistent = LayoutPersistentDocument(
                        &mut engine,
                        &mut owner,
                        &UserInteractionState::default(),
                        &cs,
                    );

                    let root = engine.GetLayoutTree().unwrap().Root() as *const LayoutObject
                        as *mut LayoutObject;
                    let index = if root_scroll {
                        (0..owner.GetDocument().NodeCount())
                            .find(|&i| {
                                owner.GetDocument().Node(i).Id()
                                    == unsafe { &*(*root).GetNode() }.InputId()
                            })
                            .unwrap()
                    } else {
                        dom_id(&owner, "s")
                    };
                    let scroller_id = owner.GetDocument().Node(index).Id();
                    let before = cached(root);
                    for (step, offset) in [
                        Offset { x: 0.0, y: 20.0 },
                        Offset { x: 15.0, y: 60.0 },
                        Offset::default(),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        owner.GetDocumentMut().SetScrollOffset(index, offset);
                        // The full normal lifecycle still synchronizes the tree and calls Layout.
                        if step == 0 {
                            persistent = LayoutPersistentDocument(
                                &mut engine,
                                &mut owner,
                                &UserInteractionState::default(),
                                &cs,
                            );
                        } else {
                            owner.GetDocumentMut().StyleStateMut().impact = Default::default();
                            assert!(LayoutPersistentScrollDocument(
                                &mut engine,
                                &mut owner,
                                &cs,
                                &[(scroller_id, offset)],
                                &mut persistent,
                                true,
                            ));
                        }
                        assert_eq!(
                            before,
                            cached(root),
                            "scroll must retain box geometry: {placement}, root={root_scroll}"
                        );
                        assert_eq!(
                            fragment(&persistent, scroller_id).paint.scroll_offset,
                            offset
                        );
                        let mut fresh_engine = LayoutEngine::new(&assembly);
                        let fresh = LayoutPersistentDocument(
                            &mut fresh_engine,
                            &mut owner,
                            &UserInteractionState::default(),
                            &cs,
                        );
                        let pixels = |tree: &FragmentNode| {
                            raster::pure_replay::RasterizeDisplayItemList(
                                &paint::paint_engine::Paint(tree),
                                320,
                                200,
                            )
                        };
                        let left = pixels(&persistent);
                        let right = pixels(&fresh);
                        assert!(left == right, "fresh pixels differ: {placement}, root={root_scroll}, offset={offset:?}, differing bytes={}", left.iter().zip(&right).filter(|(a,b)| a!=b).count());
                    }
                }
            }
        });
    }

    #[test]
    fn paint_color_update_reuses_geometry_and_matches_fresh_pixels() {
        crate::native_test_thread::run(|| {
            use dom::persistent_document::DOMAttribute;

            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner = html::html_parser::ParseHTML("<html><body><div id=a style='width:80px;height:30px;background-color:blue;color:black'>hello</div></body></html>");
            resolve(&mut owner);
            let assembly = crate::CreateLayoutAssembly();
            let _factories =
                layoutng_assembly::internal::layout_pass_scope::LayoutObjectFactoryScope::new(
                    &assembly.objects,
                );
            let cs = crate::CreateBrowserConstraints(320, 200);
            let before = LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            let root =
                engine.GetLayoutTree().unwrap().Root() as *const LayoutObject as *mut LayoutObject;
            let a = dom_id(&owner, "a");
            let id = owner.GetDocument().Node(a).Id();
            let object = native(root, id);
            let result = cached(object);
            let root_result = cached(root);
            owner.GetDocumentMut().SetAttribute(
                a,
                DOMAttribute {
                    local_name: "style".into(),
                    value: "width:80px;height:30px;background-color:red;color:green".into(),
                    ..Default::default()
                },
            );
            resolve(&mut owner);
            let persistent = LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            assert_eq!(object, native(root, id));
            assert_eq!(
                result,
                cached(object),
                "a color change must retain geometry"
            );
            assert_eq!(
                root_result,
                cached(root),
                "root geometry must also be retained"
            );

            let mut fresh_engine = LayoutEngine::new(&assembly);
            let fresh = LayoutPersistentDocument(
                &mut fresh_engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            let pixels = |tree: &FragmentNode| {
                raster::pure_replay::RasterizeDisplayItemList(
                    &paint::paint_engine::Paint(tree),
                    320,
                    200,
                )
            };
            assert_ne!(pixels(&before), pixels(&persistent));
            let left = pixels(&persistent);
            let right = pixels(&fresh);
            if left != right {
                for (name, tree) in [
                    ("before", &before),
                    ("persistent", &persistent),
                    ("fresh", &fresh),
                ] {
                    let f = fragment(tree, id);
                    println!(
                        "{name}: size={:?} offset={:?} background={:?} source={} establishes={}",
                        f.size,
                        f.offset,
                        f.paint.style.background_color,
                        f.paint.has_source,
                        f.paint.establishes_paint_state
                    );
                }
                println!(
                    "pixel differences={}",
                    left.iter().zip(&right).filter(|(a, b)| a != b).count()
                );
                fn show(name: &str, tree: &FragmentNode) {
                    println!(
                        "{name}: node={} kind={:?} color={:?} offset={:?} size={:?}",
                        tree.node_id, tree.kind, tree.paint.style.color, tree.offset, tree.size
                    );
                    for child in &tree.children {
                        show(name, child);
                    }
                }
                show("persistent", &persistent);
                show("fresh", &fresh);
            }
            assert!(left == right, "reused layout must export current colors");
        });
    }
    #[test]
    fn inline_background_presence_changes_match_fresh_pixels() {
        crate::native_test_thread::run(|| {
            use dom::persistent_document::DOMAttribute;

            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner =
                html::html_parser::ParseHTML("<html><body><span id=a>hello</span></body></html>");
            resolve(&mut owner);
            let assembly = crate::CreateLayoutAssembly();
            let cs = crate::CreateBrowserConstraints(320, 200);
            LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            let a = dom_id(&owner, "a");
            for color in ["red", "transparent"] {
                owner.GetDocumentMut().StyleStateMut().impact = Default::default();
                owner.GetDocumentMut().SetAttribute(
                    a,
                    DOMAttribute {
                        local_name: "style".into(),
                        value: format!("background-color:{color}"),
                        ..Default::default()
                    },
                );
                resolve(&mut owner);
                assert!(owner.GetDocument().StyleState().impact.layout);
                let persistent = LayoutPersistentDocument(
                    &mut engine,
                    &mut owner,
                    &UserInteractionState::default(),
                    &cs,
                );

                let mut fresh_engine = LayoutEngine::new(&assembly);
                let fresh = LayoutPersistentDocument(
                    &mut fresh_engine,
                    &mut owner,
                    &UserInteractionState::default(),
                    &cs,
                );
                let pixels = |tree: &FragmentNode| {
                    raster::pure_replay::RasterizeDisplayItemList(
                        &paint::paint_engine::Paint(tree),
                        320,
                        200,
                    )
                };
                assert!(
                    pixels(&persistent) == pixels(&fresh),
                    "background presence transition must update inline fragments"
                );
            }
        });
    }
    #[test]
    fn persistent_tree_noop_reuses_objects_styles_and_cached_results() {
        crate::native_test_thread::run(|| {
            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner = html::html_parser::ParseHTML(
                "<html><body><div id='a' style='width:80px;height:30px'>hello</div><div id='b' style='height:40px'></div></body></html>"
            );
            resolve(&mut owner);
            let assembly = crate::CreateLayoutAssembly();
            let _factories =
                layoutng_assembly::internal::layout_pass_scope::LayoutObjectFactoryScope::new(
                    &assembly.objects,
                );
            let cs = crate::CreateBrowserConstraints(320, 200);
            engine.ApplyMutation(LayoutMutation::Constraints(&cs));
            let root = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &UserInteractionState::default(),
                &mut engine,
            );

            let (): () = engine.Layout();
            let a_id = owner.GetDocument().Node(dom_id(&owner, "a")).Id();
            let a = native(root, a_id);
            let result = cached(root);
            let a_result = cached(a);
            let style = unsafe { &*a }.StyleRef() as *const _;
            let heap = foundation::LayoutHeapAllocationCountForTesting();
            foundation::CollectLayoutHeapForTesting();
            let root_again = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &UserInteractionState::default(),
                &mut engine,
            );
            assert_eq!(root, root_again);
            assert_eq!(a, native(root_again, a_id));
            assert_eq!(style, unsafe { &*a }.StyleRef() as *const _);
            let stats = engine.GetLayoutTree().unwrap().UpdateStats().clone();
            assert!(stats.reused >= 5, "{stats:?}");
            assert_eq!((stats.created, stats.updated, stats.removed), (0, 0, 0));
            engine.Layout();
            assert_eq!(result, cached(root), "root must hit layout cache");
            assert_eq!(a_result, cached(a), "child must hit layout cache");
            foundation::CollectLayoutHeapForTesting();
            assert_eq!(
                heap,
                foundation::LayoutHeapAllocationCountForTesting(),
                "no-op pass must not grow the native heap"
            );
            let snapshot = engine
                .GetLayoutResult()
                .expect("successful layout has fragments");
            assert_eq!(fragment(&snapshot, a_id).size.width, 80.0);
        });
    }
    #[test]
    fn persistent_tree_style_text_structure_and_reattach_update_in_place() {
        crate::native_test_thread::run(|| {
            use dom::persistent_document::{DOMAttribute, DOMNamespace};
            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner = html::html_parser::ParseHTML(
                "<html><body><div id='a' style='width:80px;height:30px'>hello</div><div id='b' style='height:40px'></div></body></html>"
            );
            resolve(&mut owner);
            let assembly = crate::CreateLayoutAssembly();
            let _factories =
                layoutng_assembly::internal::layout_pass_scope::LayoutObjectFactoryScope::new(
                    &assembly.objects,
                );
            let cs = crate::CreateBrowserConstraints(320, 200);
            engine.ApplyMutation(LayoutMutation::Constraints(&cs));
            let root = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &UserInteractionState::default(),
                &mut engine,
            );
            engine.Layout();
            let a_index = dom_id(&owner, "a");
            let b_index = dom_id(&owner, "b");
            let a_id = owner.GetDocument().Node(a_index).Id();
            let b_id = owner.GetDocument().Node(b_index).Id();
            let a = native(root, a_id);
            let b = native(root, b_id);
            let text_index = owner.GetDocument().Node(a_index).Children()[0];
            let text_id = owner.GetDocument().Node(text_index).Id();
            let text_object = native(root, text_id);
            owner.GetDocumentMut().SetAttribute(
                a_index,
                DOMAttribute {
                    local_name: "style".into(),
                    value: "width:120px;height:30px".into(),
                    ..Default::default()
                },
            );
            owner
                .GetDocumentMut()
                .SetTextContent(text_index, "updated text".into());
            resolve(&mut owner);
            let again = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &UserInteractionState::default(),
                &mut engine,
            );
            assert_eq!(root, again);
            assert_eq!(a, native(again, a_id));
            assert_eq!(b, native(again, b_id));
            assert_eq!(text_object, native(again, text_id));
            engine.Layout();
            let snapshot = engine
                .GetLayoutResult()
                .expect("successful layout has fragments");
            assert_eq!(fragment(&snapshot, a_id).size.width, 120.0);
            assert_eq!(
                unsafe { &*foundation::DynamicTo::<Text>(unsafe { &*text_object }.GetNode()) }
                    .data()
                    .as_str(),
                "updated text"
            );
            let body = owner.GetDocument().Node(a_index).Parent().unwrap();
            let c = owner
                .GetDocumentMut()
                .CreateElementDefault(DOMNamespace::kHTML, "div".into());
            let c_id = owner.GetDocument().Node(c).Id();
            owner.GetDocumentMut().InsertBefore(body, c, a_index);
            owner.GetDocumentMut().InsertBefore(body, b_index, c);
            resolve(&mut owner);
            let again = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &UserInteractionState::default(),
                &mut engine,
            );
            assert_eq!(root, again);
            assert_eq!(a, native(again, a_id));
            assert_eq!(b, native(again, b_id));
            engine.Layout();
            let snapshot = engine
                .GetLayoutResult()
                .expect("successful layout has fragments");
            assert!(fragment(&snapshot, b_id).offset.y < fragment(&snapshot, a_id).offset.y);
            let removed = foundation::WeakPersistent::from_ptr(native(again, c_id));
            owner.GetDocumentMut().Remove(c);
            owner.GetDocumentMut().SetAttribute(
                a_index,
                DOMAttribute {
                    local_name: "style".into(),
                    value: "display:flex;width:120px;height:30px".into(),
                    ..Default::default()
                },
            );
            resolve(&mut owner);
            let again = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &UserInteractionState::default(),
                &mut engine,
            );
            assert_eq!(root, again);
            assert_eq!(b, native(again, b_id));
            assert!(unsafe { &*native(again, a_id) }.IsFlexibleBox());
            assert!(native(again, c_id).is_null());
            engine.Layout();
            foundation::CollectLayoutHeapForTesting();
            assert!(
                removed.Get().is_null(),
                "detached object must be collectible while the engine remains alive"
            );
            let snapshot = engine
                .GetLayoutResult()
                .expect("successful layout has fragments");
            assert_eq!(fragment(&snapshot, a_id).size.width, 120.0);
        });
    }
    #[test]
    fn persistent_mixed_inline_reorder_matches_fresh_tree_pixels() {
        crate::native_test_thread::run(|| {
            use dom::persistent_document::DOMAttribute;

            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner = html::html_parser::ParseHTML(
                "<html><body><div id='parent' style='width:220px'><span id='a' style='background:red'>first</span><div id='block' style='height:20px;background:green'></div><span id='b' style='background:blue'>second</span><input id='input' value='hello'></div></body></html>"
            );
            resolve(&mut owner);
            let assembly = crate::CreateLayoutAssembly();
            let cs = crate::CreateBrowserConstraints(320, 200);
            LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            let parent = dom_id(&owner, "parent");
            let a = dom_id(&owner, "a");
            let b = dom_id(&owner, "b");
            let input = dom_id(&owner, "input");
            owner.GetDocumentMut().InsertBefore(parent, b, a);
            owner.GetDocumentMut().SetAttribute(
                a,
                DOMAttribute {
                    local_name: "style".into(),
                    value: "font-size:24px;background:red".into(),
                    ..Default::default()
                },
            );
            owner
                .GetDocumentMut()
                .SetControlValue(input, "new value".into());
            resolve(&mut owner);
            let persistent = LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );

            let mut fresh_engine = LayoutEngine::new(&assembly);
            let fresh = LayoutPersistentDocument(
                &mut fresh_engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            let pixels = |tree: &FragmentNode| {
                raster::pure_replay::RasterizeDisplayItemList(
                    &paint::paint_engine::Paint(tree),
                    320,
                    200,
                )
            };
            assert_eq!(pixels(&persistent), pixels(&fresh), "anonymous inline wrappers, inherited text and form state must match a fresh rebuild");
        });
    }
    #[test]
    fn persistent_tree_recovers_after_failed_projection() {
        crate::native_test_thread::run(|| {
            use dom::persistent_document::DOMAttribute;
            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner = html::html_parser::ParseHTML("<html><body>hello</body></html>");
            resolve(&mut owner);
            let cs = crate::CreateBrowserConstraints(320, 200);
            LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            let html = owner
                .GetDocument()
                .Node(owner.GetDocument().Root())
                .Children()
                .iter()
                .copied()
                .find(|&i| owner.GetDocument().Node(i).Type() == DOMNodeType::kElement)
                .unwrap();
            owner.GetDocumentMut().SetAttribute(
                html,
                DOMAttribute {
                    local_name: "style".into(),
                    value: "display:none".into(),
                    ..Default::default()
                },
            );
            resolve(&mut owner);
            let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                LayoutPersistentDocument(
                    &mut engine,
                    &mut owner,
                    &UserInteractionState::default(),
                    &cs,
                )
            }));
            assert!(failed.is_err());
            owner.GetDocumentMut().RemoveAttribute(html, "style", "");
            resolve(&mut owner);
            let recovered = LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &cs,
            );
            assert_eq!(recovered.size.width, 320.0);
        });
    }
    #[test]
    fn line_clamp_relayout_finishes_for_nested_video_titles() {
        crate::native_test_thread::run(|| {
            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner = html::html_parser::ParseHTML(
                "<html><body><section style='display:grid;grid-template-columns:100px 100px'>\
                 <article><h3 id='title' style='display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:2;overflow:hidden;font-size:14px;line-height:20px;margin:0'>\
                 这是一个很长的视频标题需要正确换行并且在两行之后显示省略号这是更多文字\
                 </h3></article><article>next card</article></section></body></html>"
            );
            let id = (0..owner.GetDocument().NodeCount())
                .find(|&i| {
                    owner
                        .GetDocument()
                        .Node(i)
                        .FindAttribute("id")
                        .is_some_and(|a| a.value == "title")
                })
                .map(|i| owner.GetDocument().Node(i).Id())
                .unwrap();
            dom::style_resolver::ResolveComputedStyles(
                &mut owner,
                &dom::style_resolver::StyleEnvironment::default(),
                &[],
            );
            let root = LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &UserInteractionState::default(),
                &crate::CreateBrowserConstraints(320, 200),
            );
            fn find(f: &FragmentNode, id: u64) -> Option<&FragmentNode> {
                if f.node_id == id && f.kind == layoutng_assembly::fragment_tree::FragmentKind::kBox
                {
                    return Some(f);
                }
                f.children.iter().find_map(|c| find(c, id))
            }
            let title = find(&root, id).expect("title fragment");
            assert!(
                title.size.height > 0.0 && title.size.height <= 40.0,
                "title height {}",
                title.size.height
            );
        });
    }
    #[derive(Debug, PartialEq)]
    enum Token {
        Number(f64),
        Id(u64),
        Text(String),
        None,
    }
    fn n(out: &mut Vec<Token>, v: impl Into<f64>) {
        out.push(Token::Number(v.into()));
    }
    fn optional(out: &mut Vec<Token>, v: Option<f64>) {
        out.push(v.map_or(Token::None, Token::Number));
    }
    fn b(out: &mut Vec<Token>, v: bool) {
        n(out, v as u8);
    }
    fn text(out: &mut Vec<Token>, v: &str) {
        out.push(Token::Text(v.into()));
    }
    fn dump(object: &LayoutObject, path: &str, out: &mut BTreeMap<String, Vec<Token>>) {
        let mut values = Vec::new();
        let node = object.GetNode();
        b(&mut values, !node.is_null());
        if !node.is_null() {
            let node = unsafe { &*node };
            values.push(Token::Id(node.InputId()));
            n(&mut values, node.InputKind() as i32);
            b(&mut values, node.InputIsStyleGenerated());
            text(&mut values, node.InputDebugName());
            let t = foundation::DynamicTo::<Text>(node);
            text(
                &mut values,
                if t.is_null() {
                    ""
                } else {
                    unsafe { &*t }.data().as_str()
                },
            );
            let s = node.InputStyle();
            for v in [s.width, s.height, s.min_width] {
                optional(&mut values, v);
            }
            n(&mut values, s.display as i32);
            n(&mut values, s.flex_grow);
            n(&mut values, s.flex_shrink);
            if let Some(e) = &s.extended {
                n(&mut values, e.font_size);
                n(&mut values, e.font_weight);
                n(&mut values, e.white_space as i32);
                n(&mut values, e.effective_appearance as i32);
                optional(&mut values, e.width_percent);
            } else {
                values.extend((0..5).map(|_| Token::None));
            }
            let e = foundation::DynamicTo::<Element>(node);
            let data = if e.is_null() {
                None
            } else {
                unsafe { &*e }.InputElementData().as_ref()
            };
            b(&mut values, data.is_some());
            if let Some(d) = data {
                optional(&mut values, d.form_control_type.map(|v| v as i32 as f64));
                for v in [
                    d.control_checked,
                    d.control_disabled,
                    d.control_hovered,
                    d.control_active,
                    d.control_focused,
                    d.control_read_only,
                ] {
                    b(&mut values, v);
                }
                optional(&mut values, d.range_value_ratio);
                optional(&mut values, d.control_value_ratio);
                optional(&mut values, d.control_host_ancestor.map(|v| v as f64));
                for v in [
                    d.first_letter_pseudo,
                    d.text_control_inner_editor,
                    d.file_upload_button,
                ] {
                    b(&mut values, v);
                }
                n(&mut values, d.scroll_offset.x);
                n(&mut values, d.scroll_offset.y);
                n(&mut values, d.row_span);
                n(&mut values, d.column_span);
                for v in [d.natural_width, d.natural_height, d.natural_aspect_ratio] {
                    optional(&mut values, v);
                }
                n(&mut values, d.image_device_pixel_ratio);
                optional(&mut values, d.select_uses_menu_list.map(|v| v as u8 as f64));
                n(&mut values, d.math_token_kind as i32);
            }
        }
        out.insert(path.into(), values);
        let mut child = object.SlowFirstChild();
        let mut i = 0;
        while !child.is_null() {
            let c = unsafe { &*child };
            dump(c, &format!("{path}.{i}"), out);
            i += 1;
            child = c.NextSibling();
        }
    }
    fn decode(s: &str) -> String {
        String::from_utf8(
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect(),
        )
        .unwrap()
    }
    #[test]
    fn persistent_native_tree_controls_first_letter_and_ids_match_cpp() {
        crate::native_test_thread::run(native_tree_body);
    }
    fn native_tree_body() {
        let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
        let mut owner = html::html_parser::ParseHTML(include_str!(
            "../../../artifacts/cpp-reference/persistent-tree.html"
        ));
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(include_str!(
                "../../../artifacts/cpp-reference/persistent-tree.css"
            )));
        let nodes: BTreeMap<_, _> = (0..owner.GetDocument().NodeCount())
            .filter_map(|i| {
                owner
                    .GetDocument()
                    .Node(i)
                    .FindAttribute("id")
                    .map(|a| (a.value.clone(), i))
            })
            .collect();
        owner.GetDocumentMut().SetImageResource(
            "image.png".into(),
            dom::ImageResourceMetadata {
                id: 7,
                natural_width: 32.0,
                natural_height: 24.0,
                resolution_scale: 2.0,
            },
        );
        let interaction = UserInteractionState {
            hovered_node_id: Some(owner.GetDocument().Node(nodes["range"]).Id()),
            pressed_node_id: Some(owner.GetDocument().Node(nodes["check"]).Id()),
            focus_visible_node_id: Some(owner.GetDocument().Node(nodes["input"]).Id()),
            ..Default::default()
        };
        for stage in 0..2 {
            if stage == 1 {
                let d = owner.GetDocumentMut();
                d.SetControlValue(nodes["input"], "live中文".into());
                d.SetControlValue(nodes["range"], "11".into());
                d.SetControlValue(nodes["select"], "one".into());
                d.SetControlChecked(nodes["check"], false);
                d.AppendChild(nodes["contents"], nodes["letter-next"]);
                d.SetScrollOffset(
                    nodes["contents"],
                    layoutng_assembly::internal::layout_input::Offset { x: 3.0, y: 4.0 },
                );
            }
            dom::style_resolver::ResolveComputedStyles(
                &mut owner,
                &dom::style_resolver::StyleEnvironment {
                    viewport_width: Some(1024.0),
                    viewport_height: Some(768.0),
                    resolution_dppx: Some(1.0),
                    ..Default::default()
                },
                &[],
            );
            let assembly = crate::CreateLayoutAssembly();
            let root = crate::native_test_thread::BuildDOMProjection(
                &mut owner,
                &interaction,
                &mut engine,
            );
            let _factory_scope =
                layoutng_assembly::internal::layout_pass_scope::LayoutObjectFactoryScope::new(
                    &assembly.objects,
                );
            let mut actual = BTreeMap::new();
            dump(unsafe { &*root }, "0", &mut actual);
            let mut expected: BTreeMap<String, Vec<Token>> = BTreeMap::new();
            for line in
                include_str!("../../../artifacts/cpp-reference/persistent-tree-results.tsv").lines()
            {
                let mut fields = line.split('\t');
                let key = fields.next().unwrap();
                let (s, path) = key.split_once(':').unwrap();
                if s.parse::<usize>().unwrap() != stage {
                    continue;
                }
                expected.insert(
                    path.into(),
                    fields
                        .map(|s| {
                            if s == "none" {
                                Token::None
                            } else if let Some(s) = s.strip_prefix("i:") {
                                Token::Id(s.parse().unwrap())
                            } else if let Some(s) = s.strip_prefix("s:") {
                                Token::Text(decode(s))
                            } else {
                                Token::Number(s.parse().unwrap())
                            }
                        })
                        .collect::<Vec<_>>(),
                );
            }
            assert_eq!(
                actual.keys().collect::<Vec<_>>(),
                expected.keys().collect::<Vec<_>>(),
                "stage {stage} tree order"
            );
            for (path, values) in actual {
                assert_eq!(values, expected[&path], "stage {stage} path {path}");
            }
            // LayoutEngine keeps the native tree alive after construction returns.
            assert_eq!(
                unsafe { &*(*root).GetNode() }.InputId(),
                owner
                    .GetDocument()
                    .Node(
                        *owner
                            .GetDocument()
                            .Node(owner.GetDocument().Root())
                            .Children()
                            .iter()
                            .find(|&&i| owner.GetDocument().Node(i).Type() == DOMNodeType::kElement)
                            .unwrap()
                    )
                    .Id()
            );
        }
    }
}

#[cfg(test)]
mod heap_lifecycle_tests {
    use super::*;
    #[test]
    fn unchanged_layout_does_not_collect_other_pages_but_new_objects_still_collect() {
        crate::native_test_thread::run(|| {
            use std::{cell::Cell, rc::Rc};
            struct Sentinel(Rc<Cell<usize>>);
            impl foundation::Traceable for Sentinel {
                fn Trace(&self, _: &mut foundation::Visitor<'_>) {
                    self.0.set(self.0.get() + 1);
                }
            }
            let traces = Rc::new(Cell::new(0));
            let anchor;
            {
                let _scope = foundation::LayoutHeapScope::new();
                anchor = foundation::Persistent::from_ptr(foundation::MakeGarbageCollected(
                    Sentinel(traces.clone()),
                ));
            }
            let mut engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut owner =
                html::html_parser::ParseHTML("<html><body><input value=address></body></html>");
            dom::style_resolver::ResolveComputedStyles(&mut owner, &Default::default(), &[]);
            let cs = crate::CreateBrowserConstraints(320, 200);
            let interaction = UserInteractionState::default();
            LayoutPersistentDocument(&mut engine, &mut owner, &interaction, &cs);
            let before = traces.get();
            let count = foundation::LayoutHeapAllocationCountForTesting();
            for _ in 0..3 {
                LayoutPersistentDocument(&mut engine, &mut owner, &interaction, &cs);
            }
            assert_eq!(
                traces.get(),
                before,
                "unchanged lifecycle must not trace unrelated resident Page objects"
            );
            assert_eq!(foundation::LayoutHeapAllocationCountForTesting(), count);
            let mut second_engine = LayoutEngine::new(&crate::CreateLayoutAssembly());
            let mut second = html::html_parser::ParseHTML("<html><body>another Page</body></html>");
            dom::style_resolver::ResolveComputedStyles(&mut second, &Default::default(), &[]);
            LayoutPersistentDocument(&mut second_engine, &mut second, &interaction, &cs);
            assert!(
                traces.get() > before,
                "new allocations still require collection at lifecycle end"
            );
            std::hint::black_box(anchor);
        });
    }
}
