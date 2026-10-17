#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use layoutng_assembly::{
    fragment_tree::FragmentNode,
    internal::{
        layout_input::{ComputedStyle, ConstraintSpace},
        layout_object::LayoutObject,
        layout_object_builder::LayoutObjectTree,
    },
    layout_engine::{LayoutEngine, LayoutMutation, LayoutTreeUpdate},
};

fn project(engine: &mut LayoutEngine, ids: &[u64]) {
    engine.ApplyMutation(LayoutMutation::TreeUpdate(
        &mut |tree: &mut LayoutTreeUpdate<'_>| {
            let style = ComputedStyle {
                width: Some(200.),
                height: Some(200.),
                ..Default::default()
            };
            let root = tree.CreateRootDefault(1, &style) as *mut LayoutObject;
            for &id in ids {
                let child = ComputedStyle {
                    width: Some(20. + id as f64),
                    height: Some(10.),
                    ..Default::default()
                };
                tree.AddBoxDefault(unsafe { &mut *root }, id, &child);
            }
        },
    ));
}

fn new_engine(space: &ConstraintSpace) -> LayoutEngine {
    let assembly = browser::CreateLayoutAssembly();
    let mut engine = LayoutEngine::new(&assembly);
    engine.ApplyMutation(LayoutMutation::Constraints(space));
    engine.ApplyMutation(LayoutMutation::ReplaceTree(
        LayoutObjectTree::new_with_factories(space, &assembly.objects),
    ));
    engine
}

fn geometry(fragment: &FragmentNode, out: &mut Vec<(u64, f64, f64, f64, f64)>) {
    out.push((
        fragment.node_id,
        fragment.offset.x,
        fragment.offset.y,
        fragment.size.width,
        fragment.size.height,
    ));
    for child in &fragment.children {
        geometry(child, out);
    }
}

fn layout_geometry(engine: &mut LayoutEngine) -> Vec<(u64, f64, f64, f64, f64)> {
    engine.Layout();
    let fragment = engine
        .GetLayoutResult()
        .expect("successful layout has fragments");
    let mut result = Vec::new();
    geometry(&fragment, &mut result);
    result
}

#[test]
fn complete_tree_updates_survive_reorder_removal_and_failed_build() {
    native_test_thread::run(|| {
        let space = browser::CreateBrowserConstraints(320, 240);
        let mut engine = new_engine(&space);
        for ids in [
            &[2, 3, 4][..],
            &[2, 3, 4],
            &[4, 2],
            &[4, 2, 5, 6],
            &[],
            &[7],
        ] {
            project(&mut engine, ids);
            let actual = layout_geometry(&mut engine);
            let mut fresh = new_engine(&space);
            project(&mut fresh, ids);
            assert_eq!(actual, layout_geometry(&mut fresh));
            let root = engine.GetLayoutTree().unwrap().Root() as *const LayoutObject;
            project(&mut engine, ids);
            assert_eq!(
                root,
                engine.GetLayoutTree().unwrap().Root() as *const LayoutObject
            );
            assert_eq!(
                engine.GetLayoutTree().unwrap().UpdateStats().reused,
                ids.len() + 1
            );
            assert_eq!(engine.GetLayoutTree().unwrap().UpdateStats().created, 0);
            assert_eq!(actual, layout_geometry(&mut engine));
        }
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.ApplyMutation(LayoutMutation::TreeUpdate(
                &mut |tree: &mut LayoutTreeUpdate<'_>| {
                    tree.CreateRootDefault(1, &ComputedStyle::default());
                    panic!("intentional failed projection");
                },
            ));
        }));
        assert!(failure.is_err());
        project(&mut engine, &[8, 9]);
        assert_eq!(
            engine.GetLayoutTree().unwrap().UpdateStats().created,
            3,
            "failed-build cleanup removes both maps' records"
        );
        let actual = layout_geometry(&mut engine);
        let mut fresh = new_engine(&space);
        project(&mut fresh, &[8, 9]);
        assert_eq!(actual, layout_geometry(&mut fresh));
        project(&mut engine, &[8, 9]);
        assert_eq!(engine.GetLayoutTree().unwrap().UpdateStats().reused, 3);
        assert_eq!(actual, layout_geometry(&mut engine));
    });
}

#[test]
fn retained_style_inputs_survive_dom_replacement_and_collection() {
    native_test_thread::run(|| {
        let mut owner = html::html_parser::ParseHTML(
            "<html><head><style>html:active {color:red}</style></head>\
             <body><p>Retained layout must own its previous style.</p></body></html>",
        );
        let html = (0..owner.GetDocument().NodeCount())
            .find(|&i| owner.GetDocument().Node(i).IsHTMLElement("html"))
            .unwrap();
        let interaction = dom::UserInteractionState {
            pressed_node_id: Some(owner.GetDocument().Node(html).Id()),
            ..Default::default()
        };
        let space = browser::CreateBrowserConstraints(320, 240);
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        let mut styles = style::StyleEngine::new(&owner);
        let old;
        {
            let _heap = foundation::LayoutHeapScope::new();
            styles
                .SetUserInteractionState(&mut owner, interaction.clone())
                .unwrap();
            styles.Update(&mut owner, &Default::default(), &[]).unwrap();
            browser::LayoutPersistentDocument(&mut engine, &mut owner, &interaction, &space);
            let canonical = owner
                .GetDocument()
                .ResolvedStyleFor(html)
                .unwrap()
                .native_style
                .Get();
            old = foundation::WeakPersistent::from_ptr(canonical);
            assert_ne!(
                engine.GetLayoutTree().unwrap().Root().StyleRef() as *const _,
                canonical.cast_const(),
                "font binding must exercise a distinct native style"
            );
            owner.GetDocumentMut().SetAttribute(
                html,
                dom::persistent_document::DOMAttribute {
                    local_name: "style".into(),
                    value: "font-size:19px; gap:7px; color:blue".into(),
                    ..Default::default()
                },
            );
            styles.Update(&mut owner, &Default::default(), &[]).unwrap();
            assert_ne!(
                owner
                    .GetDocument()
                    .ResolvedStyleFor(html)
                    .unwrap()
                    .native_style
                    .Get(),
                old.Get(),
                "DOM must have replaced the canonical style before GC"
            );
        }
        foundation::CollectLayoutHeapForTesting();
        assert!(
            !old.Get().is_null(),
            "resident comparison inputs must survive collection"
        );
        let actual =
            browser::LayoutPersistentDocument(&mut engine, &mut owner, &interaction, &space);
        let mut fresh = LayoutEngine::new(&assembly);
        let expected =
            browser::LayoutPersistentDocument(&mut fresh, &mut owner, &interaction, &space);
        let mut actual_geometry = Vec::new();
        let mut expected_geometry = Vec::new();
        geometry(&actual, &mut actual_geometry);
        geometry(&expected, &mut expected_geometry);
        assert_eq!(actual_geometry, expected_geometry);
        drop((actual, expected, fresh, engine, styles, owner));
        foundation::CollectLayoutHeapForTesting();
        assert!(old.Get().is_null(), "retired style inputs must not leak");
    });
}

#[test]
fn complete_dom_projection_matches_fresh_after_input_changes() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        use dom::persistent_document::{DOMAttribute, DOMNamespace};
        let mut owner = html::html_parser::ParseHTML(
            "<html><head><style>p::before{content:'prefix';color:red} \
                 p::first-letter{font-size:20px}</style></head><body style='margin:0'>\
                 <section id=container><p id=text>Hello</p><input id=control value=abc>\
                 <img id=image src=first></section><div id=other>Other</div></body></html>",
        );
        let index = |owner: &dom::DOM, id: &str| {
            (0..owner.GetDocument().NodeCount())
                .find(|&i| {
                    owner
                        .GetDocument()
                        .Node(i)
                        .FindAttribute("id")
                        .is_some_and(|attribute| attribute.value == id)
                })
                .unwrap()
        };
        let text = index(&owner, "text");
        let control = index(&owner, "control");
        let image = index(&owner, "image");
        let container = index(&owner, "container");
        let other = index(&owner, "other");
        let set = |owner: &mut dom::DOM, element, name: &str, value: &str| {
            owner.GetDocumentMut().SetAttribute(
                element,
                DOMAttribute {
                    local_name: name.into(),
                    value: value.into(),
                    ..Default::default()
                },
            );
        };
        owner.GetDocumentMut().SetImageResource(
            "first".into(),
            dom::ImageResourceMetadata {
                id: 100,
                natural_width: 12.,
                natural_height: 7.,
                ..Default::default()
            },
        );
        let mut space = browser::CreateBrowserConstraints(320, 240);
        for (id, width, height, color) in [
            (100, 12, 7, [255, 0, 0, 255]),
            (200, 52, 29, [0, 0, 255, 255]),
        ] {
            space
                .images
                .push(layoutng_assembly::internal::layout_input::PaintImage {
                    id,
                    width,
                    height,
                    content: image_resource::PaintImageContent::Bitmap(
                        color.repeat((width * height) as usize).into(),
                    ),
                    ..Default::default()
                });
        }
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        let mut extra = None;
        for step in 0..10 {
            match step {
                2 => owner
                    .GetDocumentMut()
                    .SetTextContent(text, "Changed text length".into()),
                3 => set(
                    &mut owner,
                    text,
                    "style",
                    "font-size:24px;font-family:serif;width:140px",
                ),
                4 => set(
                    &mut owner,
                    text,
                    "style",
                    "font-size:24px;font-family:serif;width:140px;color:blue;background:yellow",
                ),
                5 => {
                    set(&mut owner, control, "disabled", "");
                    owner
                        .GetDocumentMut()
                        .SetControlValue(control, "a longer input".into());
                }
                6 => {
                    owner.GetDocumentMut().SetImageResource(
                        "second".into(),
                        dom::ImageResourceMetadata {
                            id: 200,
                            natural_width: 52.,
                            natural_height: 29.,
                            ..Default::default()
                        },
                    );
                    set(&mut owner, image, "src", "second");
                }
                7 => owner.GetDocumentMut().InsertBefore(container, image, text),
                8 => {
                    let child = owner
                        .GetDocumentMut()
                        .CreateElementDefault(DOMNamespace::kHTML, "div".into());
                    owner.GetDocumentMut().AppendText(child, "new child");
                    owner.GetDocumentMut().AppendChild(other, child);
                    owner.GetDocumentMut().AppendChild(other, text);
                    extra = Some(child);
                }
                9 => owner.GetDocumentMut().Remove(extra.unwrap()),
                _ => {}
            }
            // An unchanged lifecycle can retain the tree without visiting its
            // objects. Check the output owners, not a full-walk reuse counter.
            let retained_clients = (step == 1).then(|| {
                let output = engine.Output().unwrap();
                let document = owner.GetDocument();
                (0..document.NodeCount())
                    .filter_map(|index| {
                        let id = document.Node(index).Id();
                        output.Node(id).map(|node| (id, node.PaintClientId()))
                    })
                    .collect::<Vec<_>>()
            });
            let mut style_engine = style::StyleEngine::new(&owner);
            style_engine
                .Update(&mut owner, &Default::default(), &[])
                .expect("style update");
            let actual = browser::LayoutPersistentDocument(
                &mut engine,
                &mut owner,
                &Default::default(),
                &space,
            );
            if let Some(retained_clients) = retained_clients {
                assert!(retained_clients.len() > 5);
                let output = engine.Output().unwrap();
                for (id, client) in retained_clients {
                    assert_eq!(
                        output.Node(id).unwrap().PaintClientId(),
                        client,
                        "unchanged output owner {id}"
                    );
                }
            }
            let mut fresh_engine = LayoutEngine::new(&assembly);
            let fresh = browser::LayoutPersistentDocument(
                &mut fresh_engine,
                &mut owner,
                &Default::default(),
                &space,
            );
            let mut actual_geometry = Vec::new();
            let mut fresh_geometry = Vec::new();
            geometry(&actual, &mut actual_geometry);
            geometry(&fresh, &mut fresh_geometry);
            assert_eq!(actual_geometry, fresh_geometry, "step {step}");
            #[cfg(feature = "pure_source_png")]
            {
                let pixels = |fragment: &FragmentNode| {
                    raster::pure_replay::RasterizeDisplayItemList(
                        &paint::paint_engine::Paint(fragment),
                        320,
                        240,
                    )
                };
                assert_eq!(pixels(&actual), pixels(&fresh), "paint step {step}");
            }
        }
    });
}

#[test]
fn dirty_subtree_updates_match_fresh_and_retain_clean_sibling_identity() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        fn branch(
            tree: &mut LayoutTreeUpdate<'_>,
            parent: &mut LayoutObject,
            ids: &[u64],
            width: f64,
        ) {
            let style = ComputedStyle {
                width: Some(width),
                ..Default::default()
            };
            let root = tree.AddBoxDefault(parent, 2, &style) as *mut LayoutObject;
            for &id in ids {
                tree.AddBoxDefault(
                    unsafe { &mut *root },
                    id,
                    &ComputedStyle {
                        width: Some(20.),
                        height: Some(id as f64),
                        ..Default::default()
                    },
                );
            }
        }
        fn complete(engine: &mut LayoutEngine, ids: &[u64], width: f64) {
            engine.ApplyMutation(LayoutMutation::TreeUpdate(&mut |tree| {
                let parent = tree.CreateRootDefault(
                    1,
                    &ComputedStyle {
                        width: Some(200.),
                        height: Some(200.),
                        ..Default::default()
                    },
                ) as *mut LayoutObject;
                branch(tree, unsafe { &mut *parent }, ids, width);
                tree.AddBoxDefault(
                    unsafe { &mut *parent },
                    3,
                    &ComputedStyle {
                        width: Some(150.),
                        height: Some(20.),
                        ..Default::default()
                    },
                );
            }));
        }
        let space = browser::CreateBrowserConstraints(320, 240);
        let mut engine = new_engine(&space);
        assert!(!engine.ApplyMutation(LayoutMutation::RetainTree));
        complete(&mut engine, &[7, 8], 80.);
        let before = layout_geometry(&mut engine);
        assert!(engine.ApplyMutation(LayoutMutation::RetainTree));
        let clean = engine.Output().unwrap().Node(3).unwrap().PaintClientId();
        assert!(!engine.ApplyMutation(LayoutMutation::SubtreeUpdate {
            node_id: 2,
            parent_id: 999,
            update: &mut |_, _| panic!("invalid parent was admitted"),
        }));
        let mut unchanged = Vec::new();
        geometry(engine.GetLayoutResult().unwrap(), &mut unchanged);
        assert_eq!(before, unchanged);
        for (ids, width) in [
            (&[7, 8][..], 80.),
            (&[8], 95.),
            (&[], 100.),
            (&[9, 7], 70.),
            (&[7, 9], 85.),
        ] {
            assert!(engine.ApplyMutation(LayoutMutation::SubtreeUpdate {
                node_id: 2,
                parent_id: 1,
                update: &mut |tree, parent| branch(tree, parent, ids, width),
            }));
            let actual = layout_geometry(&mut engine);
            let mut fresh = new_engine(&space);
            complete(&mut fresh, ids, width);
            assert_eq!(actual, layout_geometry(&mut fresh));
            assert_eq!(
                engine.Output().unwrap().Node(3).unwrap().PaintClientId(),
                clean
            );
        }
    });
}
