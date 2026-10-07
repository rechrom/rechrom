#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use document_image::SVGImageDecoder;
use image_decoder::skia_image_decoder::SkiaImageDecoder;
use interaction::input_event::{
    InputEvent, MouseButton, MouseEvent, MouseEventType, WheelEvent, WheelPhase,
};
use layoutng_assembly::internal::layout_input::Offset;
use rechrom::page::Page;
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

fn fixture(body: &'static [u8]) -> Page {
    let assembly = rechrom::CreateLayoutAssembly();
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader(body))),
        Rc::new(RefCell::new(SkiaImageDecoder)),
        Rc::new(RefCell::new(SVGImageDecoder::new(&assembly))),
        rechrom::CreateBrowserConstraints(320, 200),
        None,
        None,
    );
    page.Open("https://page.test/", 16384, 4096).unwrap();
    for _ in 0..100 {
        page.RunTasks(0.0).unwrap();
        if !page.IsLoading() {
            break;
        }
    }
    assert!(!page.IsLoading());
    page
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
    native_test_thread::run(|| {
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
    native_test_thread::run(|| {
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
    native_test_thread::run(|| {
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
fn transformed_link_hit_test_applies_the_owner_transform_once() {
    native_test_thread::run(|| {
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
fn overlapping_positioned_links_follow_stable_z_index_order() {
    native_test_thread::run(|| {
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
    native_test_thread::run(|| {
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
