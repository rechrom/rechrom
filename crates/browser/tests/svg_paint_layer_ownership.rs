//! Native SVG no-layer ownership must survive the Layout to Paint boundary.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::{
    fragment_tree::FragmentPaintData,
    internal::{
        layout_input::{
            FontFace, TrackedElementFrameToken, TrackedElementRelativeRect, TrackedElementSubRect,
            TrackedElementSubRectType,
        },
        layout_object::LayoutObject,
        layout_pass_scope::LayoutObjectFactoryScope,
    },
    layout_engine::PaintFragmentCursor,
};
use page_mutation::PageMutation;
use paint::paint_engine::PaintArtifact;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct Loader {
    html: Vec<u8>,
    image: Option<Vec<u8>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        let (mime_type, body) = if request.url.ends_with("fixture.svg") {
            (
                "image/svg+xml",
                self.image.as_ref().expect("fixture SVG").clone(),
            )
        } else {
            ("text/html", self.html.clone())
        };
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: mime_type.into(),
            body,
            ..Default::default()
        }))))
    }
}

fn page_fixture(markup: &str, image: Option<&str>) -> Page {
    let html =
        format!("<!doctype html><style>body{{margin:0}}svg,img{{display:block}}</style>{markup}");
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(320, 240);
    constraints.fonts = vec![FontFace {
        family: "SvgPaintOwnershipFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader {
            html: html.into_bytes(),
            image: image.map(|svg| svg.as_bytes().to_vec()),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(decoder)),
        constraints,
        None,
        None,
    );
    page.Open("https://svg-paint-ownership.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    assert!(page.CurrentFrame().is_some());
    page
}

fn fixture(svg: &str) -> Page {
    page_fixture(svg, None)
}

fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let doc = owner.GetDocument();
    (0..doc.NodeCount())
        .find_map(|index| {
            doc.Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == name)
                .map(|_| doc.Node(index).Id())
        })
        .unwrap_or_else(|| panic!("missing DOM node {name}"))
}

fn native<'a>(page: &'a Page, name: &str) -> &'a LayoutObject {
    let wanted = id(page, name);
    let assembly = browser::CreateLayoutAssembly();
    let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == wanted {
            return current;
        }
        object = current.NextInPreOrder(root);
    }
    panic!("missing native layout object {name}")
}

fn find_fragment(node: PaintFragmentCursor<'_>, wanted: u64) -> Option<PaintFragmentCursor<'_>> {
    if node.PaintData().node_id == wanted {
        Some(node)
    } else {
        node.Children()
            .into_iter()
            .find_map(|child| find_fragment(child, wanted))
    }
}

fn fragment<'a>(page: &'a Page, name: &str) -> &'a FragmentPaintData {
    find_fragment(
        PaintFragmentCursor::FromHandle(&page.CurrentFrame().unwrap().fragments),
        id(page, name),
    )
    .map(PaintFragmentCursor::PaintData)
    .unwrap_or_else(|| panic!("missing published fragment {name}"))
}

fn set_attribute(page: &mut Page, name: &str, attribute: &str, value: &str) {
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kSetAttribute,
        target_node_id: id(page, name),
        name: attribute.into(),
        value: value.into(),
        ..Default::default()
    }))
    .unwrap();
}

const GRAPHIC: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="80"><g id="group"><rect id="back" width="50" height="50" fill="red" transform="translate(5 5)"/><path id="front" d="M10 10H40V40H10Z" fill="blue"/></g></svg>"##;

fn pixels(page: &Page, artifact: &std::sync::Arc<PaintArtifact>) -> Vec<u32> {
    let mut result = vec![0; 320 * 240];
    page.PaintArtifactInto(
        artifact,
        320,
        240,
        1.0,
        &mut result,
        renderer_skia::PixelFormat::Rgba8888,
        320,
    )
    .unwrap();
    result
}

#[test]
fn svg_no_layer_keeps_sibling_paint_order_and_old_frames() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(GRAPHIC);
        for name in ["group", "back", "front"] {
            assert!(native(&page, name).IsSVGChild());
            let f = fragment(&page, name);
            assert!(f.paint.paint_layer_state_is_authoritative);
            assert_eq!(
                f.paint.paint_layer_client_id, 0,
                "SVG descendant has no native layer"
            );
            assert!(
                !paint::paint_order::ClassifyFragmentPaintLayer(f, false, false).is_paint_layer,
                "native no-layer ownership was replaced for {name}"
            );
        }
        let old = page.CurrentFrame().unwrap().display_items.clone();
        let before = pixels(&page, &old);
        assert_eq!(before[20 * 320 + 20].to_ne_bytes(), [0, 0, 255, 255]);
        set_attribute(&mut page, "back", "transform", "translate(0 0)");
        let after = pixels(&page, &page.CurrentFrame().unwrap().display_items);
        assert_eq!(after[20 * 320 + 20].to_ne_bytes(), [0, 0, 255, 255]);
        assert_ne!(
            before, after,
            "transform mutation changes exposed red edges"
        );
        assert_eq!(
            pixels(&page, &old),
            before,
            "old artifact remains immutable"
        );
        set_attribute(&mut page, "group", "transform", "translate(10 10)");
        let moved = pixels(&page, &page.CurrentFrame().unwrap().display_items);
        assert_eq!(moved[30 * 320 + 30].to_ne_bytes(), [0, 0, 255, 255]);
        assert_eq!(pixels(&page, &old), before);
    });
}

#[test]
fn fetched_svg_keeps_transformed_shapes_in_document_order() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let p = page_fixture(r#"<img id="image" src="fixture.svg">"#, Some(GRAPHIC));
        let actual = pixels(&p, &p.CurrentFrame().unwrap().display_items);
        assert_eq!(
            actual[20 * 320 + 20].to_ne_bytes(),
            [0, 0, 255, 255],
            "foreground is covered by earlier transformed rectangle"
        );
    });
}

#[test]
fn svg_mask_is_owned_by_the_svg_paint_scope_without_a_css_layer() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            r#"<svg width="80" height="80"><svg id="masked" width="60" height="60" style="mask-image:linear-gradient(black,black)"><rect width="40" height="40" fill="red"/></svg></svg>"#,
        );
        let masked = fragment(&page, "masked");
        assert!(native(&page, "masked").IsSVGChild());
        assert!(masked.paint.paint_layer_state_is_authoritative);
        assert_eq!(masked.paint.paint_layer_client_id, 0);
        assert!(!masked.paint.style.mask_images.is_empty());
        let classification = paint::paint_order::ClassifyFragmentPaintLayer(masked, false, false);
        assert!(!classification.is_paint_layer);
        let artifact = &page.CurrentFrame().unwrap().display_items;
        assert!(artifact.display_items.iter().any(|item| {
            item.id.client_id == masked.paint.display_item_client_id
                && item.id.r#type == paint::display_item_id::DisplayItemIdType::kSVGMask
        }));
        assert!(artifact.items.iter().any(|item| {
            item.node_id == id(&page, "masked")
                && item.r#type == paint::paint_engine::DisplayItemType::kSaveLayerDstIn
        }));
    });
}

#[test]
fn svg_shape_publishes_region_capture_bounds_before_cached_drawing() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(
            r#"<svg width="80" height="80"><rect id="capture" x="10" y="20" width="30" height="40" fill="red"/></svg>"#,
        );
        let node_id = id(&page, "capture");
        let old = page.CurrentFrame().unwrap().display_items.clone();
        assert!(old.chunks.iter().all(|chunk| chunk
            .compositor
            .as_deref()
            .and_then(|metadata| metadata.region_capture_data.as_deref())
            .is_none()));

        let crop_id = [0x1020_3040_5060_7080, 0x90a0_b0c0_d0e0_f001];
        assert!(page.SetRegionCaptureCropId(node_id, crop_id));
        for _ in 0..20 {
            page.RunTask().unwrap();
            if page
                .CurrentFrame()
                .unwrap()
                .display_items
                .chunks
                .iter()
                .any(|chunk| {
                    chunk
                        .compositor
                        .as_deref()
                        .and_then(|metadata| metadata.region_capture_data.as_deref())
                        .is_some_and(|data| data.iter().any(|(id, _)| *id == crop_id))
                })
            {
                break;
            }
        }

        let source = foundation::DynamicTo::<
            layoutng_assembly::internal::layout_node_metadata::Element,
        >(native(&page, "capture").GetNode());
        assert_eq!(
            unsafe { &*source }.InputRegionCaptureCropId(),
            Some(crop_id),
            "crop ID did not reach the resident Layout Element"
        );
        assert_eq!(
            fragment(&page, "capture").paint.region_capture_crop_id,
            Some(crop_id),
            "crop ID did not cross the DOM-to-fragment boundary"
        );

        let regions = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .chunks
            .iter()
            .filter_map(|chunk| chunk.compositor.as_deref())
            .filter_map(|metadata| metadata.region_capture_data.as_deref())
            .flatten()
            .filter(|(id, _)| *id == crop_id)
            .collect::<Vec<_>>();
        assert_eq!(regions.len(), 1);
        assert_eq!(
            regions[0].1,
            paint::paint_engine::PaintRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }
        );
        assert!(old.chunks.iter().all(|chunk| chunk
            .compositor
            .as_deref()
            .and_then(|metadata| metadata.region_capture_data.as_deref())
            .is_none()));
    });
}

#[test]
fn html_box_publishes_region_capture_and_tracked_subrects() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(
            r#"<div id="capture" style="position:absolute;left:10px;top:20px;width:30px;height:40px"></div>"#,
        );
        let node_id = id(&page, "capture");
        let old = page.CurrentFrame().unwrap().display_items.clone();
        let crop_id = [0x1111, 0x2222];
        let tracked_id = [0x3333, 0x4444];
        assert!(page.SetRegionCaptureCropId(node_id, crop_id));
        assert!(page.SetTrackedElementSubRect(
            node_id,
            TrackedElementSubRect {
                feature: 7,
                id: tracked_id,
                should_add_to_compositor_frame_metadata: true,
                should_exclude_fixed_and_sticky_occlusions: true,
                sub_rect: Some(TrackedElementRelativeRect {
                    x: 5,
                    y: 6,
                    width: 10,
                    height: 12,
                    r#type: TrackedElementSubRectType::IntersectWithElementRect,
                }),
                frame_token: Some(TrackedElementFrameToken::Remote([0x7777, 0x8888])),
                parent_frame_token: Some([0x9999, 0xaaaa]),
            }
        ));
        for _ in 0..20 {
            page.RunTask().unwrap();
            let artifact = &page.CurrentFrame().unwrap().display_items;
            let has_region = artifact.chunks.iter().any(|chunk| {
                chunk
                    .compositor
                    .as_deref()
                    .and_then(|metadata| metadata.region_capture_data.as_deref())
                    .is_some_and(|data| data.iter().any(|(id, _)| *id == crop_id))
            });
            let has_tracked = artifact.chunks.iter().any(|chunk| {
                chunk
                    .compositor
                    .as_deref()
                    .and_then(|metadata| metadata.tracked_element_rects.as_deref())
                    .is_some_and(|data| data.iter().any(|rect| rect.id == tracked_id))
            });
            if has_region && has_tracked {
                break;
            }
        }

        assert_eq!(
            fragment(&page, "capture").paint.region_capture_crop_id,
            Some(crop_id)
        );
        let fragment_tracking = &fragment(&page, "capture").paint.tracked_element_sub_rects;
        assert_eq!(fragment_tracking.len(), 1);
        assert_eq!(fragment_tracking[0].feature, 7);
        assert_eq!(fragment_tracking[0].id, tracked_id);
        let artifact = &page.CurrentFrame().unwrap().display_items;
        let regions = artifact
            .chunks
            .iter()
            .filter_map(|chunk| chunk.compositor.as_deref())
            .filter_map(|metadata| metadata.region_capture_data.as_deref())
            .flatten()
            .filter(|(id, _)| *id == crop_id)
            .collect::<Vec<_>>();
        assert_eq!(regions.len(), 1);
        assert_eq!(
            regions[0].1,
            paint::paint_engine::PaintRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }
        );
        let capture_chunk = artifact
            .chunks
            .iter()
            .find(|chunk| {
                chunk
                    .compositor
                    .as_deref()
                    .and_then(|metadata| metadata.region_capture_data.as_deref())
                    .is_some_and(|data| data.iter().any(|(id, _)| *id == crop_id))
            })
            .expect("capture metadata chunk");
        assert_eq!(
            capture_chunk.bounds,
            paint::paint_engine::PaintRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }
        );
        assert_eq!(
            capture_chunk
                .compositor
                .as_ref()
                .unwrap()
                .hit_test_opaqueness,
            paint::paint_engine::PaintHitTestOpaqueness::Opaque
        );
        let tracked = artifact
            .chunks
            .iter()
            .filter_map(|chunk| chunk.compositor.as_deref())
            .filter_map(|metadata| metadata.tracked_element_rects.as_deref())
            .flatten()
            .filter(|rect| rect.id == tracked_id)
            .collect::<Vec<_>>();
        assert_eq!(tracked.len(), 1);
        assert_eq!(tracked[0].feature, 7);
        assert_eq!(
            tracked[0].rect,
            paint::paint_engine::PaintRect {
                x: 15.0,
                y: 26.0,
                width: 10.0,
                height: 12.0,
            }
        );
        assert!(tracked[0].should_add_to_compositor_frame_metadata);
        assert!(tracked[0].should_exclude_fixed_and_sticky_occlusions);
        assert_eq!(
            tracked[0].frame_token,
            Some(paint::paint_engine::PaintFrameToken::Remote([
                0x7777, 0x8888
            ]))
        );
        assert_eq!(
            tracked[0].parent_frame_token,
            Some(paint::paint_engine::PaintFrameToken::Local([
                0x9999, 0xaaaa
            ]))
        );
        assert!(old.chunks.iter().all(|chunk| chunk
            .compositor
            .as_deref()
            .is_none_or(|metadata| metadata.region_capture_data.is_none()
                && metadata.tracked_element_rects.is_none())));

        assert!(page.ClearTrackedElementSubRect(node_id, 7));
        for _ in 0..20 {
            page.RunTask().unwrap();
            if page
                .CurrentFrame()
                .unwrap()
                .display_items
                .chunks
                .iter()
                .all(|chunk| {
                    chunk
                        .compositor
                        .as_deref()
                        .and_then(|metadata| metadata.tracked_element_rects.as_deref())
                        .is_none_or(|data| data.iter().all(|rect| rect.id != tracked_id))
                })
            {
                break;
            }
        }
        assert!(fragment(&page, "capture")
            .paint
            .tracked_element_sub_rects
            .is_empty());
        assert!(page
            .CurrentFrame()
            .unwrap()
            .display_items
            .chunks
            .iter()
            .all(|chunk| chunk
                .compositor
                .as_deref()
                .and_then(|metadata| metadata.tracked_element_rects.as_deref())
                .is_none_or(|data| data.iter().all(|rect| rect.id != tracked_id))));
    });
}

#[test]
fn tracked_plain_inline_forces_a_fragment_and_publishes_bounds() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(r#"<span id="capture">A</span>"#);
        let node_id = id(&page, "capture");
        let tracked_id = [0x5555, 0x6666];
        assert!(page.SetTrackedElementSubRect(
            node_id,
            TrackedElementSubRect {
                feature: 9,
                id: tracked_id,
                should_add_to_compositor_frame_metadata: false,
                should_exclude_fixed_and_sticky_occlusions: false,
                sub_rect: None,
                frame_token: None,
                parent_frame_token: None,
            }
        ));
        for _ in 0..20 {
            page.RunTask().unwrap();
            if page
                .CurrentFrame()
                .unwrap()
                .display_items
                .chunks
                .iter()
                .any(|chunk| {
                    chunk
                        .compositor
                        .as_deref()
                        .and_then(|metadata| metadata.tracked_element_rects.as_deref())
                        .is_some_and(|data| data.iter().any(|rect| rect.id == tracked_id))
                })
            {
                break;
            }
        }

        let fragment = fragment(&page, "capture");
        assert!(fragment.paint.box_fragment_is_inline_box);
        assert_eq!(fragment.paint.tracked_element_sub_rects.len(), 1);
        let tracked = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .chunks
            .iter()
            .filter_map(|chunk| chunk.compositor.as_deref())
            .filter_map(|metadata| metadata.tracked_element_rects.as_deref())
            .flatten()
            .find(|rect| rect.id == tracked_id)
            .expect("tracked inline bounds");
        assert!(tracked.rect.width > 0.0);
        assert!(tracked.rect.height > 0.0);
    });
}
