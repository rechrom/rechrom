use super::*;
use crate::page::{Page, PageClient, PageState};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::{
    fragment_tree::FragmentKind,
    internal::layout_input::{ConstraintSpace, Offset, PaintImage, Size},
};
use page_mutation::{
    AnimationStyleSample, AnimationTick, InteractionStateMutation, PageMutation,
    ResourceLoadFailed, ResourceMutation, ScrollMutation, ViewportMutation,
};
use std::cell::RefCell;

struct Client;
impl PageClient for Client {}
fn constraints() -> ConstraintSpace {
    // Match the live browser's viewport, themes, font catalog and native backend.
    crate::CreateBrowserConstraints(320, 200)
}
fn state(html: &str) -> Rc<PageState> {
    let state = PageState::new(constraints(), Rc::new(RefCell::new(Client)));
    *state.document.borrow_mut() = html::html_parser::ParseHTML(html);
    state
}
fn id(state: &PageState, name: &str) -> u64 {
    let owner = state.document.borrow();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find_map(|i| {
            document
                .Node(i)
                .FindAttribute("id")
                .filter(|a| a.value == name)
                .map(|_| document.Node(i).Id())
        })
        .unwrap()
}
fn geometry(state: &PageState, id: u64) -> Vec<PaintRect> {
    state.EnsureMeasurement();
    state.measurement.borrow().as_ref().unwrap().ClientRects(id)
}
fn held(state: &PageState) -> Rc<FragmentNode> {
    state.EnsureMeasurement();
    state
        .measurement
        .borrow()
        .as_ref()
        .unwrap()
        .fragments
        .clone()
}
#[test]
fn rect_index_builds_once_for_all_nodes_preserves_order_and_returns_independent_values() {
    let root = Rc::new(FragmentNode {
        node_id: 1,
        size: Size {
            width: 320.,
            height: 200.,
        },
        children: vec![
            FragmentNode {
                node_id: 2,
                offset: Offset { x: 10., y: 20. },
                size: Size {
                    width: 25.,
                    height: 10.,
                },
                ..Default::default()
            },
            FragmentNode {
                node_id: 2,
                offset: Offset { x: 50., y: 50. },
                size: Size {
                    width: 20.,
                    height: 5.,
                },
                ..Default::default()
            },
            FragmentNode {
                node_id: 3,
                kind: FragmentKind::kText,
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    let snapshot = MeasurementSnapshot::new(root);
    assert!(snapshot.rects.get().is_none());
    // Explicit fixture coordinates are independent of either paint walker.
    let expected_root = vec![PaintRect {
        x: 0.,
        y: 0.,
        width: 320.,
        height: 200.,
    }];
    let expected_repeated = vec![
        PaintRect {
            x: 10.,
            y: 20.,
            width: 25.,
            height: 10.,
        },
        PaintRect {
            x: 50.,
            y: 50.,
            width: 20.,
            height: 5.,
        },
    ];
    for _ in 0..20 {
        assert_eq!(snapshot.ClientRects(1), expected_root);
        assert_eq!(snapshot.ClientRects(2), expected_repeated);
        assert!(snapshot.ClientRects(3).is_empty());
        assert!(snapshot.ClientRects(999).is_empty());
    }
    assert_eq!(snapshot.index_builds.get(), 1);
    let mut returned = snapshot.ClientRects(2);
    assert_eq!(returned.len(), 2);
    returned[0].x = 999.;
    returned.clear();
    assert_eq!(snapshot.ClientRects(2), expected_repeated);
    assert_eq!(snapshot.index_builds.get(), 1);
}
#[test]
fn empty_animation_tick_same_scroll_same_interaction_and_failed_resource_keep_snapshot() {
    crate::native_test_thread::run(|| {
        let state =
            state("<html><body><div id=b style='width:40px;height:20px'></div></body></html>");
        let node = id(&state, "b");
        let expected = geometry(&state, node);
        let before = held(&state);
        state.ApplyMutation(PageMutation::AnimationTick(AnimationTick {
            monotonic_time: 1.,
            ..Default::default()
        }));
        state.ApplyMutation(PageMutation::ScrollMutation(ScrollMutation {
            target_node_id: node,
            offset: Offset::default(),
        }));
        state.ApplyMutation(PageMutation::InteractionStateMutation(
            InteractionStateMutation::default(),
        ));
        state.ApplyMutation(PageMutation::ResourceMutation(
            ResourceMutation::ResourceLoadFailed(ResourceLoadFailed::default()),
        ));
        assert!(Rc::ptr_eq(&before, &held(&state)));
        assert_eq!(geometry(&state, node), expected);
        assert_eq!(
            state
                .measurement
                .borrow()
                .as_ref()
                .unwrap()
                .index_builds
                .get(),
            1
        );
    });
}
#[test]
fn actual_dom_change_flushes_style_before_synchronous_notification_reads_geometry() {
    crate::native_test_thread::run(|| {
        let state =
            state("<html><body><div id=b style='width:40px;height:20px'></div></body></html>");
        let node = id(&state, "b");
        assert_eq!(geometry(&state, node)[0].width, 40.);
        let before = held(&state);
        state.ApplyDOMMutation(
            &DOMMutation {
                mutation_type: DOMMutationType::kSetAttribute,
                target_node_id: node,
                name: "style".into(),
                value: "width:80px;height:20px".into(),
                ..Default::default()
            },
            &mut || {
                assert_eq!(geometry(&state, node)[0].width, 80.);
            },
            &mut |_, _| panic!("unexpected image callback"),
        );
        assert!(!Rc::ptr_eq(&before, &held(&state)));
        assert_eq!(geometry(&state, node)[0].width, 80.);
        assert_eq!(
            state
                .measurement
                .borrow()
                .as_ref()
                .unwrap()
                .index_builds
                .get(),
            1
        );
    });
}
#[test]
fn changed_scroll_animation_viewport_font_and_image_invalidate_snapshot() {
    crate::native_test_thread::run(|| {
        let state=state("<html><body><div id=s style='width:80px;height:20px;overflow:hidden'><div id=b style='width:40px;height:100px'></div></div></body></html>");
        let scroller = id(&state, "s");
        let node = id(&state, "b");
        let y = geometry(&state, node)[0].y;
        state.ApplyMutation(PageMutation::ScrollMutation(ScrollMutation {
            target_node_id: scroller,
            offset: Offset { x: 0., y: 10. },
        }));
        assert!(state.measurement.borrow().is_none());
        assert_eq!(geometry(&state, node)[0].y, y - 10.);
        state.ApplyMutation(PageMutation::AnimationTick(AnimationTick {
            monotonic_time: 1.,
            samples: vec![AnimationStyleSample {
                node_id: node,
                effect_id: 1,
                declarations: cssom::ParseCSSDeclarationList("width:70px"),
                ..Default::default()
            }],
        }));
        assert!(state.measurement.borrow().is_none());
        let _ = geometry(&state, node);
        state.ApplyMutation(PageMutation::ViewportMutation(ViewportMutation {
            size: Size {
                width: 400.,
                height: 200.,
            },
        }));
        assert!(state.measurement.borrow().is_none());
        let _ = geometry(&state, node);
        let font = state.constraints.borrow().fonts[0].clone();
        state.ApplyResourceMutation(ResourceMutation::FontResourceReady(
            page_mutation::FontResourceReady { font },
        ));
        assert!(state.measurement.borrow().is_none());
        let _ = geometry(&state, node);
        state.ApplyResourceMutation(ResourceMutation::ImageResourceReady(
            page_mutation::ImageResourceReady {
                source: "fixture.png".into(),
                image: PaintImage {
                    id: 800,
                    width: 2,
                    height: 3,
                    resolution_scale: 1.,
                    rgba8: vec![255; 24].into(),
                    ..Default::default()
                },
            },
        ));
        assert!(state.measurement.borrow().is_none());
    });
}
struct NoNetwork;
impl url_loader::URLLoader for NoNetwork {
    fn Load(
        &mut self,
        _: &url_loader::URLRequest,
    ) -> std::io::Result<Box<dyn url_loader::URLLoadOperation>> {
        Err(std::io::Error::other("unexpected fixture network request"))
    }
}
#[test]
fn completed_frame_publishes_same_snapshot_without_a_second_measurement_layout() {
    crate::native_test_thread::run(|| {
        let assembly = crate::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(NoNetwork)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(
                image_decoder::svg_image_decoder::SVGImageDecoder::new(&assembly),
            )),
            constraints(),
            None,
            None,
        );
        *page.state.document.borrow_mut() = html::html_parser::ParseHTML(
            "<html><body><div id=b style='width:40px;height:20px'></div></body></html>",
        );
        page.document_started = true;
        page.UpdateFrameIfNeeded().unwrap();
        let frame = page
            .frame
            .as_ref()
            .expect("completed frame")
            .fragments
            .clone();
        assert!(Rc::ptr_eq(&frame, &held(&page.state)));
        let node = id(&page.state, "b");
        assert_eq!(geometry(&page.state, node)[0].width, 40.);
        // An effective CSSOM change must still invalidate the shared frame.
        page.state
            .ApplyMutation(PageMutation::CSSOMMutation(page_mutation::CSSOMMutation {
                style_sheet: cssom::ParseCSS("#b {width:42px!important}"),
                base_url: "https://fixture.test/".into(),
            }));
        assert!(page.state.measurement.borrow().is_none());
        assert_eq!(geometry(&page.state, node)[0].width, 42.);
    });
}
#[test]
fn style_write_geometry_and_frame_share_one_layout_snapshot_and_match_fresh_pixels() {
    crate::native_test_thread::run(|| {
        let assembly = crate::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(NoNetwork)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(
                image_decoder::svg_image_decoder::SVGImageDecoder::new(&assembly),
            )),
            constraints(),
            None,
            None,
        );
        *page.state.document.borrow_mut()=html::html_parser::ParseHTML("<html><body style='margin:0'><div id=b style='width:40px;height:20px;background:red'></div></body></html>");
        page.document_started = true;
        page.UpdateFrameIfNeeded().unwrap();
        assert_eq!(page.state.full_layout_lifecycles.get(), 1);
        let node = id(&page.state, "b");
        page.state.ApplyDOMMutation(
            &DOMMutation {
                mutation_type: DOMMutationType::kSetAttribute,
                target_node_id: node,
                name: "style".into(),
                value: "width:80px;height:20px;background:blue".into(),
                ..Default::default()
            },
            &mut || {},
            &mut |_, _| {},
        );
        assert_eq!(geometry(&page.state, node)[0].width, 80.);
        let measured = held(&page.state);
        assert_eq!(page.state.full_layout_lifecycles.get(), 2);
        page.UpdateFrameIfNeeded().unwrap();
        assert_eq!(
            page.state.full_layout_lifecycles.get(),
            2,
            "the CSSOM query already performed build/layout/export"
        );
        assert!(Rc::ptr_eq(
            &measured,
            &page.frame.as_ref().unwrap().fragments
        ));
        assert!(Rc::ptr_eq(&measured, &held(&page.state)));
        assert_eq!(
            page.state
                .measurement
                .borrow()
                .as_ref()
                .unwrap()
                .index_builds
                .get(),
            1,
            "frame must retain the geometry index"
        );
        let actual = renderer::pure_replay::RasterizeDisplayItemList(
            &page.frame.as_ref().unwrap().display_items,
            320,
            200,
        );
        // Obtain the comparison from a fresh resident native tree, not a second
        // paint of the reused fragment snapshot.
        let mut owner=html::html_parser::ParseHTML("<html><body style='margin:0'><div id=b style='width:80px;height:20px;background:blue'></div></body></html>");
        crate::style_services::ResolveLayoutStyles(&mut owner, &page.state.constraints.borrow());
        let mut layout_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
        let fresh = crate::LayoutPersistentDocument(
            &mut layout_engine,
            &mut owner,
            &dom::UserInteractionState::default(),
            &page.state.constraints.borrow(),
        );
        let expected = renderer::pure_replay::RasterizeDisplayItemList(
            &paint::paint_engine::Paint(&fresh),
            320,
            200,
        );
        assert_eq!(
            actual, expected,
            "geometry-to-frame reuse must render the fresh updated document"
        );
    });
}

#[test]
fn opacity_queries_retain_geometry_but_frame_exports_current_paint() {
    crate::native_test_thread::run(|| {
        let assembly = crate::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(NoNetwork)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(
                image_decoder::svg_image_decoder::SVGImageDecoder::new(&assembly),
            )),
            constraints(),
            None,
            None,
        );
        *page.state.document.borrow_mut()=html::html_parser::ParseHTML("<html><body style='margin:0'><div id=b style='width:40px;height:20px;background:red;opacity:.3'></div></body></html>");
        page.document_started = true;
        page.UpdateFrameIfNeeded().unwrap();
        let node = id(&page.state, "b");
        let before = geometry(&page.state, node);
        let original = held(&page.state);
        for opacity in [".4", ".5", ".6"] {
            page.state.ApplyDOMMutation(
                &DOMMutation {
                    mutation_type: DOMMutationType::kSetAttribute,
                    target_node_id: node,
                    name: "style".into(),
                    value: format!("width:40px;height:20px;background:blue;opacity:{opacity}"),
                    ..Default::default()
                },
                &mut || {},
                &mut |_, _| {},
            );
            assert_eq!(geometry(&page.state, node), before);
            assert!(Rc::ptr_eq(&original, &held(&page.state)));
            assert!(!page
                .state
                .measurement
                .borrow()
                .as_ref()
                .unwrap()
                .PaintCurrent());
            assert_eq!(
                page.state.full_layout_lifecycles.get(),
                1,
                "paint-only writes must not rebuild query geometry"
            );
        }
        page.UpdateFrameIfNeeded().unwrap();
        assert_eq!(
            page.state.full_layout_lifecycles.get(),
            2,
            "Paint must export the new color and opacity"
        );
        let actual = renderer::pure_replay::RasterizeDisplayItemList(
            &page.frame.as_ref().unwrap().display_items,
            320,
            200,
        );
        let mut owner=html::html_parser::ParseHTML("<html><body style='margin:0'><div id=b style='width:40px;height:20px;background:blue;opacity:.6'></div></body></html>");
        crate::style_services::ResolveLayoutStyles(&mut owner, &page.state.constraints.borrow());
        let mut layout_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
        let fresh = crate::LayoutPersistentDocument(
            &mut layout_engine,
            &mut owner,
            &dom::UserInteractionState::default(),
            &page.state.constraints.borrow(),
        );
        assert_eq!(
            actual,
            renderer::pure_replay::RasterizeDisplayItemList(
                &paint::paint_engine::Paint(&fresh),
                320,
                200
            )
        );
        assert!(page
            .state
            .measurement
            .borrow()
            .as_ref()
            .unwrap()
            .PaintCurrent());
    });
}
