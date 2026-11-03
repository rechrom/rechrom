#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use image_decoder::skia_image_decoder::SkiaImageDecoder;
use interaction::input_event::{
    InputEvent, MouseButton, MouseEvent, MouseEventType, WheelEvent, WheelPhase,
};
use layoutng_assembly::internal::layout_input::Offset;
use paint::display_item_id::DisplayItemIdType;
use paint::paint_engine::{DisplayItemType, PaintHitTestOpaqueness};
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Operation(Option<URLResponse>);
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct FixtureLoader(&'static [u8]);
impl URLLoader for FixtureLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Operation(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: self.0.to_vec(),
            ..Default::default()
        }))))
    }
}

fn fixture_with_options(body: &'static [u8], device_scale: f64, printing: bool) -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(320, 200);
    constraints.device_pixel_ratio = device_scale;
    constraints.printing = printing;
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader(body))),
        Rc::new(RefCell::new(SkiaImageDecoder)),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        constraints,
        None,
        None,
    );
    page.Open("https://page.test/", 16384, 4096).unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() {
            break;
        }
    }
    assert!(!page.IsLoading());
    page
}

fn fixture_with_scale(body: &'static [u8], device_scale: f64) -> Page {
    fixture_with_options(body, device_scale, false)
}

fn fixture(body: &'static [u8]) -> Page {
    fixture_with_scale(body, 1.0)
}

fn id(page: &Page, value: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == value)
                .map(|_| document.Node(index).Id())
        })
        .unwrap()
}

#[test]
fn root_scroll_hit_tests_visible_descendants() {
    test_thread::run(|| {
        let mut page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #spacer { height: 1000px; }
            #bottom { display: block; width: 200px; height: 40px; }
        </style><body><div id=spacer></div><a id=bottom href='https://target.test/'>Bottom</a>"#,
        );
        let bottom = id(&page, "bottom");
        page.ScrollWheelDefault(&WheelEvent {
            phase: WheelPhase::kEnded,
            position: Offset { x: 20.0, y: 100.0 },
            delta: Offset { x: 0.0, y: 1000.0 },
            ..Default::default()
        })
        .unwrap();
        let result = page
            .Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                button: MouseButton::kPrimary,
                position: Offset { x: 20.0, y: 180.0 },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(result.target_node_id, Some(bottom));
    });
}

#[test]
fn root_scroll_hit_tests_fixed_descendants_in_viewport_space() {
    test_thread::run(|| {
        let mut page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #fixed { position: fixed; top: 80px; display: block; width: 200px; height: 40px; }
            #spacer { height: 1000px; }
        </style><body>
            <a id=fixed href='https://fixed.test/'>Fixed</a>
            <div id=spacer></div>
        "#,
        );
        let fixed = id(&page, "fixed");
        page.ScrollWheelDefault(&WheelEvent {
            phase: WheelPhase::kEnded,
            position: Offset { x: 250.0, y: 100.0 },
            delta: Offset { x: 0.0, y: 600.0 },
            ..Default::default()
        })
        .unwrap();
        let result = page
            .Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                button: MouseButton::kPrimary,
                position: Offset { x: 20.0, y: 100.0 },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(result.target_node_id, Some(fixed));
    });
}

#[test]
fn overflow_scroll_hit_tests_sticky_descendants_at_painted_offset() {
    test_thread::run(|| {
        let mut page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #scroller { width: 300px; height: 180px; overflow: auto; }
            #sticky { position: sticky; top: 0; display: block; width: 200px; height: 40px; }
            #spacer { height: 1000px; }
        </style><body><div id=scroller>
            <a id=sticky href='https://sticky.test/'>Sticky</a>
            <div id=spacer></div>
        </div>"#,
        );
        let sticky = id(&page, "sticky");
        page.ScrollWheelDefault(&WheelEvent {
            phase: WheelPhase::kEnded,
            position: Offset { x: 250.0, y: 100.0 },
            delta: Offset { x: 0.0, y: 600.0 },
            ..Default::default()
        })
        .unwrap();
        let result = page
            .Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                button: MouseButton::kPrimary,
                position: Offset { x: 20.0, y: 20.0 },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(result.target_node_id, Some(sticky));
    });
}

#[test]
fn sticky_constraint_reaches_the_paint_property_scroll_owner() {
    test_thread::run(|| {
        let page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #scroller { width: 300px; height: 180px; overflow: auto; }
            #sticky { position: sticky; top: 0; left: 0; width: 200px; height: 40px; background: red; }
            #spacer { width: 900px; height: 1000px; }
        </style><body><div id=scroller><div id=sticky></div><div id=spacer></div></div>"#,
        );
        let artifact = &page.CurrentFrame().unwrap().display_items;
        let sticky = artifact
            .chunks
            .iter()
            .find_map(|chunk| {
                let mut transform = Some(&chunk.properties.transform);
                while let Some(node) = transform {
                    if node.sticky_constraint.is_some() {
                        return Some(node.clone());
                    }
                    transform = node.parent.as_ref();
                }
                None
            })
            .expect("sticky translation property");
        let constraint = sticky.sticky_constraint.unwrap();
        let vertical_owner = constraint.vertical.unwrap().scroll_container_client_id;
        let horizontal_owner = constraint.horizontal.unwrap().scroll_container_client_id;
        let mut found_vertical = false;
        let mut found_horizontal = false;
        let mut transform = sticky.parent.as_ref();
        while let Some(node) = transform {
            if let Some(scroll) = &node.scroll {
                found_vertical |= scroll.owner_client_id == vertical_owner;
                found_horizontal |= scroll.owner_client_id == horizontal_owner;
            }
            transform = node.parent.as_ref();
        }
        assert!(found_vertical && found_horizontal);
    });
}

#[test]
fn transformed_link_hit_test_applies_the_owner_transform_once() {
    test_thread::run(|| {
        let mut page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #link { display: block; width: 200px; height: 40px; transform: translateY(100px); }
        </style><body><a id=link href='https://transform.test/'>Transformed text</a>"#,
        );
        let link = id(&page, "link");
        let result = page
            .Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                button: MouseButton::kPrimary,
                position: Offset { x: 20.0, y: 120.0 },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(result.target_node_id, Some(link));
    });
}

#[test]
fn replaced_border_radius_excludes_the_outer_corner_from_self_hit_testing() {
    test_thread::run(|| {
        let mut page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; width: 320px; height: 200px; }
            #rounded { display: block; box-sizing: border-box; width: 100px; height: 100px;
                       border: 5px solid; border-radius: 40px; overflow: hidden; }
        </style><body id=body><img id=rounded>"#,
        );
        let body = id(&page, "body");
        let rounded = id(&page, "rounded");
        let click = |page: &mut Page, x, y| {
            page.Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                button: MouseButton::kPrimary,
                position: Offset { x, y },
                ..Default::default()
            }))
            .unwrap()
            .target_node_id
        };
        assert_eq!(click(&mut page, 1.0, 1.0), Some(body));
        assert_eq!(click(&mut page, 3.0, 40.0), Some(rounded));
        assert_eq!(click(&mut page, 50.0, 50.0), Some(rounded));
    });
}

#[test]
fn overlapping_positioned_links_follow_stable_z_index_order() {
    test_thread::run(|| {
        let mut page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            a { position: absolute; inset: 20px auto auto 20px; width: 160px; height: 80px; }
            #front { z-index: 10; }
            #back { z-index: 1; }
        </style><body>
            <a id=front href='https://front.test/'>Front</a>
            <a id=back href='https://back.test/'>Back</a>
        "#,
        );
        let front = id(&page, "front");
        let result = page
            .Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                button: MouseButton::kPrimary,
                position: Offset { x: 40.0, y: 40.0 },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(result.target_node_id, Some(front));
    });
}

#[test]
fn overlapping_flex_items_follow_z_index_order() {
    test_thread::run(|| {
        let mut page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #stack { display: flex; width: 160px; height: 80px; margin: 20px; }
            a { flex: 0 0 160px; height: 80px; }
            #front { z-index: 10; }
            #back { z-index: 1; margin-left: -160px; }
        </style><body><div id=stack>
            <a id=front href='https://front.test/'>Front</a>
            <a id=back href='https://back.test/'>Back</a>
        </div>"#,
        );
        let front = id(&page, "front");
        let result = page
            .Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                button: MouseButton::kPrimary,
                position: Offset { x: 40.0, y: 40.0 },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(result.target_node_id, Some(front));
    });
}

#[test]
fn resizer_records_the_expanded_touch_hit_test_chunk() {
    test_thread::run(|| {
        let page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #resizable { margin: 20px; width: 100px; height: 60px;
                         overflow: scroll; resize: both; }
        </style><body><div id=resizable></div>"#,
        );
        let chunk = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .chunks
            .iter()
            .find(|chunk| chunk.id.r#type == DisplayItemIdType::kResizerScrollHitTest)
            .expect("resizer touch hit-test chunk");
        let compositor = chunk.compositor.as_deref().expect("compositor metadata");
        let hit_test = compositor
            .hit_test_data
            .as_deref()
            .expect("resizer hit-test data");
        assert_eq!(
            compositor.hit_test_opaqueness,
            PaintHitTestOpaqueness::Mixed
        );
        assert_eq!(hit_test.scroll_hit_test_rect, chunk.bounds);
        assert!(hit_test.scroll_translation.is_none());
        assert_eq!(chunk.bounds.width, 30.0);
        assert_eq!(chunk.bounds.height, 30.0);

        let artifact = &page.CurrentFrame().unwrap().display_items;
        let drawing = artifact
            .display_items
            .iter()
            .find(|item| item.id.r#type == DisplayItemIdType::kResizer)
            .expect("platform resizer drawing");
        let record = &artifact.items[drawing.record_begin..drawing.record_end];
        assert_eq!(
            record
                .iter()
                .filter(|item| item.r#type == DisplayItemType::kStrokePath)
                .count(),
            2
        );
        assert_eq!(drawing.visual_rect.width, 15.0);
        assert_eq!(drawing.visual_rect.height, 15.0);
    });
}

#[test]
fn transparent_scroll_hit_test_omits_the_resizer_chunk() {
    test_thread::run(|| {
        let page = fixture(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #resizable { width: 100px; height: 60px; overflow: scroll;
                         resize: both; pointer-events: none; }
        </style><body><div id=resizable></div>"#,
        );
        assert!(!page
            .CurrentFrame()
            .unwrap()
            .display_items
            .chunks
            .iter()
            .any(|chunk| chunk.id.r#type == DisplayItemIdType::kResizerScrollHitTest));
    });
}

#[test]
fn printing_suppresses_overlay_scrollbars_but_not_the_resizer() {
    test_thread::run(|| {
        let page = fixture_with_options(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #resizable { width: 100px; height: 60px; overflow: scroll; resize: both; }
        </style><body><div id=resizable></div>"#,
            1.0,
            true,
        );
        let artifact = &page.CurrentFrame().unwrap().display_items;
        assert!(!artifact.display_items.iter().any(|item| matches!(
            item.id.r#type,
            DisplayItemIdType::kScrollbarHorizontal | DisplayItemIdType::kScrollbarVertical
        )));
        assert!(artifact
            .display_items
            .iter()
            .any(|item| item.id.r#type == DisplayItemIdType::kResizer));
    });
}

#[test]
fn resizer_geometry_scales_from_dip() {
    test_thread::run(|| {
        let page = fixture_with_scale(
            br#"<!doctype html><style>
            html, body { margin: 0; }
            #resizable { width: 100px; height: 60px; overflow: scroll; resize: both; }
        </style><body><div id=resizable></div>"#,
            2.0,
        );
        let artifact = &page.CurrentFrame().unwrap().display_items;
        let touch = artifact
            .chunks
            .iter()
            .find(|chunk| chunk.id.r#type == DisplayItemIdType::kResizerScrollHitTest)
            .expect("scaled resizer touch metadata");
        assert_eq!((touch.bounds.width, touch.bounds.height), (60.0, 60.0));
        let drawing = artifact
            .display_items
            .iter()
            .find(|item| item.id.r#type == DisplayItemIdType::kResizer)
            .expect("scaled platform resizer drawing");
        assert_eq!(
            (drawing.visual_rect.width, drawing.visual_rect.height),
            (30.0, 30.0)
        );
        assert!(artifact.items[drawing.record_begin..drawing.record_end]
            .iter()
            .filter(|item| item.r#type == DisplayItemType::kStrokePath)
            .all(|item| item.stroke_width == 2.0));
    });
}
