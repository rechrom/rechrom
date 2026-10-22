//! Root scroll publishes one coherent native sticky/property/paint snapshot.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use layoutng_assembly::internal::layout_input::FontFace;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Operation(Option<URLResponse>);
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct FixtureLoader(Vec<u8>);
impl URLLoader for FixtureLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Operation(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: self.0.clone(),
            ..Default::default()
        }))))
    }
}
struct XHR;
impl xhr_transport::XMLHttpRequestTransport for XHR {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(io::Error::other("unexpected fixture XHR"))
    }
}

fn fixture(css: &str, children: &str, overlay: bool) -> Page {
    fixture_with_theme(css, children, overlay, 15, 7, 1.0)
}
fn fixture_with_theme(
    css: &str,
    children: &str,
    overlay: bool,
    auto: i32,
    thin: i32,
    zoom: f32,
) -> Page {
    let html = format!(
        "<!doctype html><style>html{{font-size:16px}}body{{margin:0}}{css}</style>{children}"
    );
    let assembly = browser::CreateLayoutAssembly();
    let mut space = browser::CreateBrowserConstraints(300, 200);
    let theme = space.scrollbar_theme.as_mut().unwrap();
    theme.uses_overlay_scrollbars = overlay;
    theme.auto_thickness = auto;
    theme.thin_thickness = thin;
    space.layout_zoom_factor = zoom as f64;
    space.fonts = vec![FontFace {
        family: "ButtonFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }];
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader(html.into_bytes()))),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        space,
        Some(browser::page::ScriptEnvironment {
            runtime: Box::new(
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_native_stack_budget(
                    4 * 1024 * 1024,
                ),
            ),
            xhr: Box::new(XHR),
            user_agent: "button-fixture".into(),
        }),
        None,
    );
    page.Open("https://page.test/rows.html", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() {
            break;
        }
    }
    assert!(!page.IsLoading());
    page
}

fn check(page: &mut Page, expression: &str) {
    let result = page.Evaluate(expression, "window-scroll-fixture").unwrap();
    assert!(result.Succeeded(), "{:?}", result.exception);
}

fn pixels(page: &mut Page) -> Vec<u32> {
    let mut pixels = vec![0; 300 * 200];
    page.PaintInto(
        300,
        200,
        1.0,
        &mut pixels,
        renderer_skia::PixelFormat::Rgba8888,
        300,
    )
    .unwrap();
    pixels
}

#[test]
fn root_sticky_fixed_and_clipped_pixels_follow_scroll_and_return() {
    native_test_thread::run(|| {
        let mut page = fixture(
            "body{height:1800px}#sticky{position:sticky;top:0;width:100px;height:20px;background:yellow}#fixed{position:fixed;top:7px;right:20px;width:23px;height:23px;background:black}#fixedparent{position:fixed;top:50px;right:20px;width:40px;height:60px}#fixedsticky{position:sticky;top:0;height:15px;background:black}.section{height:300px}#clip{width:40px;height:30px;overflow:hidden;border-radius:8px}#layer{width:70px;height:60px;background:magenta;opacity:.8}",
            "<div id=sticky></div><div id=fixed></div><div id=fixedparent><div id=fixedsticky></div></div><div class=section style='background:red'></div><div class=section style='background:lime'><div id=clip><div id=layer></div></div></div><div class=section style='background:blue'></div><div class=section style='background:cyan'></div><div class=section style='background:magenta'></div><div class=section style='background:yellow'></div>", true);
        let original = pixels(&mut page);
        for y in [180, 360, 700, 1250, 180, 0] {
            check(
                &mut page,
                &format!("scrollTo(0,{y}); if(scrollY!=={y})throw Error('scroll');"),
            );
            for _ in 0..3 {
                page.RunTask().unwrap();
            }
            check(&mut page, "if(document.getElementById('sticky').getBoundingClientRect().y!==0)throw Error('root sticky');if(document.getElementById('fixed').getBoundingClientRect().y!==7)throw Error('fixed');if(document.getElementById('fixedsticky').getBoundingClientRect().y!==50)throw Error('fixed ancestor sticky');");
            let tree = &page.CurrentFrame().unwrap().fragments;
            fn sticky(node: &layoutng_assembly::fragment_tree::FragmentNode) {
                if let Some(constraint) = node.paint.sticky_constraint {
                    if let Some(axis) = constraint.vertical {
                        assert_eq!(axis.current_sticky_offset, node.paint.sticky_offset.y,
                            "native matrix and compositor recorded basis must be published together");
                    }
                }
                for child in &node.children {
                    sticky(child);
                }
            }
            sticky(tree);
            // Validate the sticky basis published in Paint property state.
            let artifact = &page.CurrentFrame().unwrap().display_items;
            let mut transforms = std::collections::HashMap::new();
            for chunk in &artifact.chunks {
                let mut node = Some(chunk.properties.transform.clone());
                while let Some(transform) = node {
                    transforms.insert(transform.id, transform.clone());
                    node = transform.parent.clone();
                }
            }
            let mut checked = 0;
            for transform in transforms.values() {
                if let Some(axis) = transform
                    .sticky_constraint
                    .and_then(|constraint| constraint.vertical)
                {
                    assert_eq!(transform.matrix.values[13], axis.current_sticky_offset);
                    checked += 1;
                }
            }
            assert!(
                checked >= 2,
                "both root and fixed-parent sticky transforms must be published"
            );
            let output = pixels(&mut page);
            assert_eq!(output[10 * 300 + 50].to_ne_bytes(), [255, 255, 0, 255]);
            let colors = [
                [255, 0, 0, 255],
                [0, 255, 0, 255],
                [0, 0, 255, 255],
                [0, 255, 255, 255],
                [255, 0, 255, 255],
                [255, 255, 0, 255],
            ];
            let section = ((y + 100 - 20) / 300) as usize;
            assert_eq!(output[100 * 300 + 150].to_ne_bytes(), colors[section]);
            if y == 0 {
                assert_eq!(output, original);
            }
        }
        check(&mut page, "document.body.style.height='2400px';document.getElementById('sticky').style.background='cyan';scrollTo(0,500)");
        for _ in 0..3 {
            page.RunTask().unwrap();
        }
        check(&mut page, "if(document.getElementById('sticky').getBoundingClientRect().y!==0)throw Error('changed-layout sticky');");
        assert_eq!(
            pixels(&mut page)[10 * 300 + 50].to_ne_bytes(),
            [0, 255, 255, 255]
        );
    });
}

#[test]
fn compositor_delta_acknowledgement_follows_commit_and_skips_second_default() {
    native_test_thread::run(|| {
        use interaction::input_event::{InputEvent, WheelEvent, WheelPhase};
        use layoutng_assembly::internal::layout_input::Offset;
        let mut page = fixture("body{height:230px}", "<div></div>", true);
        let wheel = WheelEvent {
            compositor_owned: true,
            phase: WheelPhase::kBegan,
            delta: Offset { x: 0., y: 80. },
            position: Offset { x: 20., y: 20. },
            ..Default::default()
        };
        let result = page.Dispatch(&InputEvent::Wheel(wheel.clone())).unwrap();
        assert!(!result.default_prevented);
        page.ScrollWheelDefault(&wheel).unwrap();
        check(
            &mut page,
            "if(scrollY!==0)throw Error('async default applied twice');",
        );
        let owner = page
            .CurrentFrame()
            .unwrap()
            .fragments
            .paint
            .display_item_client_id;
        page.ApplyCompositorScrollDelta(1, owner, Offset { x: 0., y: 80. })
            .unwrap();
        assert_eq!(page.CurrentFrame().unwrap().compositor_scroll_sequence, 0);
        for _ in 0..3 {
            page.RunTask().unwrap();
        }
        assert_eq!(page.CurrentFrame().unwrap().compositor_scroll_sequence, 1);
        check(
            &mut page,
            "if(scrollY!==30)throw Error('DOM clamps to range');",
        );
        page.ApplyCompositorScrollDelta(2, owner, Offset { x: 0., y: -10. })
            .unwrap();
        for _ in 0..3 {
            page.RunTask().unwrap();
        }
        assert_eq!(page.CurrentFrame().unwrap().compositor_scroll_sequence, 2);
        check(
            &mut page,
            "if(scrollY!==20)throw Error('second synchronized delta');",
        );
        page.ApplyCompositorScrollDelta(2, owner, Offset { x: 0., y: -10. })
            .unwrap();
        check(
            &mut page,
            "if(scrollY!==20)throw Error('duplicate delta');scrollTo(0,10);",
        );
        for _ in 0..3 {
            page.RunTask().unwrap();
        }
        page.ApplyCompositorScrollDelta(3, owner, Offset { x: 0., y: 5. })
            .unwrap();
        for _ in 0..3 {
            page.RunTask().unwrap();
        }
        check(
            &mut page,
            "if(scrollY!==15)throw Error('main write did not rebase cc offset');",
        );
    });
}

#[test]
fn native_post_layout_clamp_updates_dom_sticky_and_does_not_restore_on_expansion() {
    native_test_thread::run(|| {
        let mut page = fixture(
            "#sticky{position:sticky;top:0;height:20px;background:yellow}#content{height:1800px;background:blue}#nested{width:100px;height:100px;overflow:auto}#inner{height:500px;background:red}",
            "<div id=sticky></div><div id=content></div><div id=nested><div id=inner></div></div>", true);
        check(&mut page, "window.clamps=0;document.addEventListener('scroll',()=>clamps++);scrollTo(0,700);document.getElementById('nested').scrollTop=300;");
        for _ in 0..3 {
            page.RunTask().unwrap();
        }
        check(&mut page, "clamps=0;document.getElementById('content').style.height='100px';document.getElementById('inner').style.height='10px';if(document.getElementById('nested').scrollTop!==0)throw Error('nested clamp');if(scrollY!==20)throw Error('root clamp '+scrollY);if(document.getElementById('sticky').getBoundingClientRect().y!==0)throw Error('sticky after clamp');");
        for _ in 0..3 {
            page.RunTask().unwrap();
        }
        check(&mut page, "if(clamps!==1)throw Error('clamp event '+clamps);document.getElementById('content').style.height='1800px';document.getElementById('inner').style.height='500px';if(scrollY!==20)throw Error('old offset resurrected');if(document.getElementById('nested').scrollTop!==0)throw Error('old nested offset resurrected');");
        // A synchronous query must not leave an older pending request behind.
        check(&mut page, "scrollTo(0,700);document.getElementById('content').style.height='100px';if(scrollY!==20)throw Error('synchronous clamp');");
        for _ in 0..3 {
            page.RunTask().unwrap();
        }
        check(
            &mut page,
            "if(scrollY!==20)throw Error('pending request restored old offset');scrollTo(0,0);",
        );
    });
}

#[test]
fn owner_memory_diagnostics_are_read_only_and_do_not_advance_the_frame() {
    native_test_thread::run(|| {
        let mut page = fixture(
            "#box{height:500px;background:blue}",
            "<div id=box></div>",
            true,
        );
        let before = pixels(&mut page);
        let sequence = page.CurrentFrame().unwrap().sequence;
        let first = page.MemoryDiagnostics();
        let second = page.MemoryDiagnostics();
        assert_eq!(page.CurrentFrame().unwrap().sequence, sequence);
        assert!(first.javascript.unwrap().used_bytes > 0);
        assert_eq!(
            first.javascript.unwrap().used_bytes,
            second.javascript.unwrap().used_bytes
        );
        assert_eq!(first.dom.arena_nodes, second.dom.arena_nodes);
        assert_eq!(before, pixels(&mut page));
    });
}
