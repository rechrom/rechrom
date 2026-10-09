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
fn complete_dom_projection_matches_fresh_after_input_changes() {
    native_test_thread::run(|| {
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
                    rgba8: color.repeat((width * height) as usize).into(),
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
            let resident = engine.GetLayoutTree().unwrap();
            if step == 1 {
                assert!(resident.UpdateStats().reused > 5);
                assert_eq!(resident.UpdateStats().created, 0);
                assert_eq!(resident.UpdateStats().updated, 0);
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
