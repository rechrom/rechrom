//! Root scrollbar ownership, native relayout and immutable CSSOM geometry.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use layoutng_assembly::internal::layout_input::FontFace;
use layoutng_assembly::layout_engine::PaintFragmentCursor;
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
    }]
    .into();
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader(html.into_bytes()))),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        space,
        Some(browser::page::ScriptEnvironment {
            runtime: Box::new(
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_max_stack_size(
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
    let result = page
        .Evaluate(expression, "scrollbar-viewport-fixture")
        .unwrap();
    assert!(result.Succeeded(), "{:?}", result.exception);
}

fn root_paint(
    fragments: &layoutng_assembly::fragment_tree::LayoutResultHandle,
) -> &layoutng_assembly::fragment_tree::FragmentPaintData {
    PaintFragmentCursor::FromHandle(fragments).PaintData()
}
#[test]
fn classic_vertical_overflow_reserves_root_gutter() {
    test_thread::run(|| {
        let mut page = fixture("#fill{height:400px}", "<div id=fill></div>", false);
        check(
            &mut page,
            "if(document.getElementById('fill').getBoundingClientRect().width!==285)throw Error(JSON.stringify({inner:innerWidth,client:document.documentElement.clientWidth,height:document.documentElement.clientHeight,rect:document.getElementById('fill').getBoundingClientRect()}))",
        );
    });
}
#[test]
fn overlay_vertical_overflow_keeps_root_width() {
    test_thread::run(|| {
        let mut page = fixture("#fill{height:400px}", "<div id=fill></div>", true);
        check(
            &mut page,
            "if(document.getElementById('fill').getBoundingClientRect().width!==300)throw Error('overlay root width')",
        );
    });
}

#[test]
fn programmatic_scroll_refreshes_macos_painted_thumb_opacity() {
    test_thread::run(|| {
        let mut page = fixture("#fill{height:400px}", "<div id=fill></div>", true);
        let before = page.CurrentFrame().unwrap().fragments.clone();
        assert_eq!(
            root_paint(&before)
                .paint
                .scrollbars
                .as_ref()
                .and_then(|scrollbars| scrollbars.vertical.as_ref())
                .map(|axis| axis.painted_opacity),
            Some(0.0)
        );
        assert!(page
            .CurrentFrame()
            .unwrap()
            .display_items
            .display_items
            .iter()
            .filter_map(|item| item.scrollbar.as_ref())
            .all(|scrollbar| scrollbar.painted_opacity == 0.0));
        check(&mut page, "window.scrollTo(0,100)");
        for _ in 0..4 {
            page.RunTask().unwrap();
        }
        let after = page.CurrentFrame().unwrap().fragments.clone();
        assert_eq!(
            root_paint(&after)
                .paint
                .scrollbars
                .as_ref()
                .and_then(|scrollbars| scrollbars.vertical.as_ref())
                .map(|axis| axis.painted_opacity),
            Some(1.0)
        );
        assert!(page
            .CurrentFrame()
            .unwrap()
            .display_items
            .display_items
            .iter()
            .filter_map(|item| item.scrollbar.as_ref())
            .any(|scrollbar| !scrollbar.horizontal && scrollbar.painted_opacity == 1.0));
    });
}

#[test]
fn rowspan_cell_ink_overflow_recalculates_through_table_owner() {
    test_thread::run(|| {
        let mut page = fixture(
            "table{border-collapse:collapse}td{width:40px;height:20px;padding:0}#span{box-shadow:0 0 0 7px red}",
            "<table><tr><td id=span rowspan=2></td><td></td></tr><tr><td></td></tr></table>",
            true,
        );
        check(
            &mut page,
            "const r=document.getElementById('span').getBoundingClientRect();if(r.width!==40||r.height!==40)throw Error(JSON.stringify(r))",
        );
    });
}

#[test]
fn text_combine_shadow_and_outline_ink_overflow_keep_combined_text_horizontal() {
    test_thread::run(|| {
        let mut page = fixture(
            "#vertical{writing-mode:vertical-rl;font-size:24px}#combined{text-combine-upright:all;text-shadow:5px 3px 2px red;outline:3px solid blue}",
            "<div id=vertical><span id=combined>12</span></div>",
            true,
        );
        check(
            &mut page,
            "const r=document.getElementById('combined').getBoundingClientRect();if(!(r.width>0&&r.height>0&&Number.isFinite(r.width)&&Number.isFinite(r.height)))throw Error(JSON.stringify(r))",
        );
    });
}

#[test]
fn classic_without_overflow_keeps_root_width() {
    test_thread::run(|| {
        let mut page = fixture("#fill{height:100px}", "<div id=fill></div>", false);
        check(
            &mut page,
            "if(document.getElementById('fill').getBoundingClientRect().width!==300)throw Error('classic no overflow root width')",
        );
    });
}

fn dimensions(page: &mut Page, width: u32, height: u32) {
    check(
        page,
        &format!(
            "if(document.documentElement.clientWidth!=={width}||document.documentElement.clientHeight!=={height})throw Error('client '+document.documentElement.clientWidth+'x'+document.documentElement.clientHeight+' expected {width}x{height}')"
        ),
    );
}
#[test]
fn classic_root_overflow_both_axes_and_css_policies() {
    test_thread::run(|| {
        for (css, w, h) in [
            ("#fill{width:400px;height:100px}", 300, 185),
            ("#fill{width:400px;height:400px}", 285, 185),
            ("#fill{width:298px;height:400px}", 285, 185),
            (
                "html{overflow:hidden}#fill{width:400px;height:400px}",
                300,
                200,
            ),
            ("html{overflow:scroll}#fill{height:100px}", 285, 185),
            (
                "html{overflow:auto;scrollbar-width:thin}#fill{height:400px}",
                293,
                200,
            ),
            ("html{scrollbar-width:none}#fill{height:400px}", 300, 200),
        ] {
            let mut page = fixture(css, "<div id=fill></div>", false);
            dimensions(&mut page, w, h);
        }
    });
}
#[test]
fn root_scrollbars_update_after_mutation_and_resize() {
    test_thread::run(|| {
        let mut page = fixture("#fill{height:400px}", "<div id=fill></div>", false);
        dimensions(&mut page, 285, 200);
        check(
            &mut page,
            "document.getElementById('fill').setAttribute('style','height:100px')",
        );
        dimensions(&mut page, 300, 200);
        check(
            &mut page,
            "document.getElementById('fill').setAttribute('style','height:400px')",
        );
        dimensions(&mut page, 285, 200);
        check(
            &mut page,
            "document.documentElement.setAttribute('style','scrollbar-width:thin')",
        );
        dimensions(&mut page, 293, 200);
        check(
            &mut page,
            "document.documentElement.setAttribute('style','scrollbar-width:auto')",
        );
        dimensions(&mut page, 285, 200);
        page.ResizeViewport(300.0, 500.0, 1.0).unwrap();
        dimensions(&mut page, 300, 500);
        page.ResizeViewport(300.0, 200.0, 1.0).unwrap();
        dimensions(&mut page, 285, 200);
        check(
            &mut page,
            "if(innerWidth!==300||innerHeight!==200)throw Error('outer viewport changed')",
        );
    });
}
#[test]
fn root_stable_gutter_reserves_space_without_overflow() {
    test_thread::run(|| {
        for (css, w, h) in [
            ("html{scrollbar-gutter:stable}", 285, 200),
            ("html{scrollbar-gutter:stable both-edges}", 270, 200),
            ("html{overflow:hidden;scrollbar-gutter:stable}", 285, 200),
            ("html{overflow:clip;scrollbar-gutter:stable}", 285, 200),
        ] {
            let mut page = fixture(css, "<div id=fill></div>", false);
            dimensions(&mut page, w, h);
        }
        let mut page = fixture(
            "html{scrollbar-gutter:stable both-edges}",
            "<div id=fill></div>",
            true,
        );
        dimensions(&mut page, 300, 200);
    });
}
#[test]
fn body_overflow_propagates_to_viewport() {
    test_thread::run(|| {
        for (css, w, h) in [
            ("body{overflow:hidden}#fill{height:400px}", 300, 200),
            ("body{overflow:scroll}#fill{height:100px}", 285, 185),
            (
                "html{overflow:auto}body{overflow:hidden}#fill{height:400px}",
                285,
                200,
            ),
        ] {
            let mut page = fixture(css, "<div id=fill></div>", false);
            dimensions(&mut page, w, h);
        }
    });
}
#[test]
fn auto_scrollbar_keeps_viewport_units_full_size() {
    test_thread::run(|| {
        let mut page = fixture(
            "#fill{height:400px}#vw{width:100vw;height:100vh}",
            "<div id=fill><div id=vw></div></div>",
            false,
        );
        dimensions(&mut page, 285, 185);
        check(
            &mut page,
            "let r=document.getElementById('vw').getBoundingClientRect();if(r.width!==300||r.height!==200)throw Error(JSON.stringify(r))",
        );
    });
}

#[test]
fn unconditional_scrollbars_reduce_viewport_units() {
    test_thread::run(|| {
        let mut page = fixture(
            "html{overflow:scroll}#vw{width:100vw;height:100vh}",
            "<div id=vw></div>",
            false,
        );
        dimensions(&mut page, 285, 185);
        check(
            &mut page,
            "let r=document.getElementById('vw').getBoundingClientRect();if(r.width!==285||r.height!==185)throw Error(JSON.stringify(r))",
        );
    });
}

#[test]
fn unconditional_scrollbar_mutation_updates_viewport_units_once() {
    test_thread::run(|| {
        let mut page = fixture(
            "html{overflow:auto}#vw{width:100vw;height:100vh}",
            "<div id=vw></div>",
            false,
        );
        dimensions(&mut page, 300, 200);
        check(
            &mut page,
            "(()=>{let rectangle=document.getElementById('vw').getBoundingClientRect();if(rectangle.width!==300||rectangle.height!==200)throw Error(JSON.stringify(rectangle))})() ",
        );
        check(
            &mut page,
            "document.documentElement.setAttribute('style','overflow:scroll')",
        );
        dimensions(&mut page, 285, 185);
        check(
            &mut page,
            "(()=>{let rectangle=document.getElementById('vw').getBoundingClientRect();if(rectangle.width!==285||rectangle.height!==185)throw Error(JSON.stringify(rectangle))})() ",
        );
        check(
            &mut page,
            "document.documentElement.setAttribute('style','overflow:auto')",
        );
        dimensions(&mut page, 300, 200);
        check(
            &mut page,
            "(()=>{let rectangle=document.getElementById('vw').getBoundingClientRect();if(rectangle.width!==300||rectangle.height!==200)throw Error(JSON.stringify(rectangle))})() ",
        );
    });
}

#[test]
fn injected_scrollbar_metrics_are_not_fixed_to_fifteen() {
    test_thread::run(|| {
        let mut page = fixture_with_theme(
            "#fill{height:400px}",
            "<div id=fill></div>",
            false,
            23,
            11,
            1.0,
        );
        dimensions(&mut page, 277, 200);
        let mut page = fixture_with_theme(
            "html{scrollbar-width:thin}#fill{height:400px}",
            "<div id=fill></div>",
            false,
            23,
            11,
            1.0,
        );
        dimensions(&mut page, 289, 200);
    });
}
#[test]
fn page_zoom_keeps_root_source_geometry_in_one_coordinate_space() {
    test_thread::run(|| {
        for zoom in [2.0, 0.8] {
            let mut page = fixture_with_theme(
                "#fill{height:100px}",
                "<div id=fill></div>",
                false,
                16,
                8,
                zoom,
            );
            dimensions(&mut page, 300, 200);
            let mut page = fixture_with_theme(
                "#fill{height:400px}",
                "<div id=fill></div>",
                false,
                16,
                8,
                zoom,
            );
            dimensions(&mut page, (300.0 - 16.0 / zoom).round() as u32, 200);
        }
    });
}

#[test]
fn body_propagation_mutation_updates_clip_and_retained_snapshots() {
    test_thread::run(|| {
        let mut page = fixture(
            "body{overflow:hidden}#fill{height:400px}",
            "<div id=fill></div>",
            false,
        );
        dimensions(&mut page, 300, 200);
        let old = page.CurrentFrame().unwrap().fragments.clone();
        assert_eq!(root_paint(&old).paint.client_size.unwrap().width, 300.0);
        check(
            &mut page,
            "document.documentElement.setAttribute('style','overflow:auto')",
        );
        dimensions(&mut page, 285, 200);
        check(
            &mut page,
            "document.documentElement.setAttribute('style','overflow:visible')",
        );
        dimensions(&mut page, 300, 200);
        check(
            &mut page,
            "document.body.setAttribute('style','overflow:visible')",
        );
        dimensions(&mut page, 285, 200);
        assert_eq!(root_paint(&old).paint.client_size.unwrap().width, 300.0);
    });
}
#[test]
fn containment_stops_body_overflow_propagation() {
    test_thread::run(|| {
        for css in [
            "body{overflow:hidden;contain:layout}#fill{height:400px}",
            "html{contain:layout}body{overflow:hidden}#fill{height:400px}",
        ] {
            let mut page = fixture(css, "<div id=fill></div>", false);
            dimensions(&mut page, 285, 200);
        }
    });
}
#[test]
fn classic_scrollbars_relayout_wrapping_and_nested_client_size() {
    test_thread::run(|| {
        let mut page = fixture(
            "#fill{height:400px}#wrap{font-size:0;line-height:20px}#wrap span{display:inline-block;vertical-align:top;width:146px;height:20px}#nested{height:50px;width:100px;padding:5px;border:2px solid;overflow:auto}#nested div{height:100px}",
            "<div id=fill><div id=wrap><span></span><span></span></div><div id=nested><div></div></div></div>",
            false,
        );
        dimensions(&mut page, 285, 200);
        check(
            &mut page,
            "let n=document.getElementById('nested');if(n.clientWidth!==95||n.clientHeight!==60)throw Error('nested '+n.clientWidth+'x'+n.clientHeight)",
        );
        check(
            &mut page,
            "if(document.getElementById('wrap').getBoundingClientRect().height!==40)throw Error('narrow lines '+JSON.stringify(document.getElementById('wrap').getBoundingClientRect()))",
        );
        let root = page.CurrentFrame().unwrap().fragments.clone();
        assert!(root_paint(&root).paint.scrollbars.as_ref().is_some());
        check(
            &mut page,
            "document.getElementById('fill').setAttribute('style','height:100px')",
        );
        dimensions(&mut page, 300, 200);
        check(
            &mut page,
            "if(document.getElementById('wrap').getBoundingClientRect().height!==20)throw Error('wide lines')",
        );
    });
}

#[test]
fn equal_viewport_size_retains_the_published_layout() {
    test_thread::run(|| {
        let mut page = fixture("#fill{height:400px}", "<div id=fill></div>", false);
        let old = page.CurrentFrame().unwrap().fragments.clone();
        page.ResizeViewport(300.0, 200.0, 1.0).unwrap();
        assert!(old.PtrEq(&page.CurrentFrame().unwrap().fragments));
        dimensions(&mut page, 285, 200);
    });
}

#[test]
fn classic_root_canvas_background_reaches_compositor_with_matching_clip() {
    test_thread::run(|| {
        for (background, expected) in [
            ("background:rgb(20,40,60)", [20, 40, 60, 255]),
            (
                "color-scheme:light dark;background:light-dark(#eee,#222)",
                [238, 238, 238, 255],
            ),
        ] {
            let page = fixture(
                &format!("html{{{background}}}#fill{{height:400px}}"),
                "<div id=fill></div>",
                false,
            );
            let frame = page.CurrentFrame().unwrap();
            let root = root_paint(&frame.fragments);
            let canvas = root.paint.canvas_background.as_ref().unwrap_or_else(|| {
                panic!(
                    "missing canvas background for {background}; root style={:?}",
                    root.paint.style.background_color()
                )
            });
            assert_eq!(
                [
                    (canvas.style.background_color().red * 255.0).round() as u8,
                    (canvas.style.background_color().green * 255.0).round() as u8,
                    (canvas.style.background_color().blue * 255.0).round() as u8,
                    (canvas.style.background_color().alpha * 255.0).round() as u8,
                ],
                expected,
                "canvas export for {background}"
            );
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
            assert_eq!(pixels[100 * 300 + 100].to_ne_bytes(), expected);
        }
    });
}

#[test]
fn canvas_scroll_and_fixed_layers_keep_clip_parity_after_scroll() {
    test_thread::run(|| {
        for overlay in [false, true] {
            for background in [
                "background:rgb(20,40,60)",
                "background:linear-gradient(red,blue) fixed",
                "background:linear-gradient(red,blue) scroll",
                "background-image:linear-gradient(red,blue),linear-gradient(green,yellow);background-attachment:fixed,scroll;background-color:rgb(20,40,60)",
            ] {
                let mut page = fixture(
                    &format!("html{{{background}}}#fill{{height:400px}}"),
                    "<div id=fill></div>",
                    overlay,
                );
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
                let retained = page.CurrentFrame().unwrap().fragments.clone();
                page.ScrollWheelDefault(&interaction::input_event::WheelEvent {
                    phase: interaction::input_event::WheelPhase::kEnded,
                    position: interaction::input_event::Offset {
                        x: 100.0,
                        y: 100.0,
                    },
                    delta: interaction::input_event::Offset { x: 0.0, y: 80.0 },
                    ..Default::default()
                })
                .unwrap();
                assert!(retained.PtrEq(&page.CurrentFrame().unwrap().fragments));
                assert_eq!(root_paint(&retained).paint.scroll_offset.y, 80.0);
                assert_eq!(
                    root_paint(&page.CurrentFrame().unwrap().fragments)
                        .paint
                        .scroll_offset
                        .y,
                    80.0
                );
                page.PaintInto(
                    300,
                    200,
                    1.0,
                    &mut pixels,
                    renderer_skia::PixelFormat::Rgba8888,
                    300,
                )
                .unwrap();
                assert_eq!(pixels[100 * 300 + 100].to_ne_bytes()[3], 255);
            }
        }
    });
}

fn paint_fixed_control(page: &Page, width: u32, height: u32, x: usize, y: usize) {
    let mut pixels = vec![0; (width * height) as usize];
    page.PaintInto(
        width,
        height,
        1.0,
        &mut pixels,
        renderer_skia::PixelFormat::Rgba8888,
        width as usize,
    )
    .unwrap();
    assert_eq!(
        pixels[y * width as usize + x].to_ne_bytes(),
        [20, 40, 60, 255]
    );
}

#[test]
fn fixed_control_keeps_root_overflow_clip_with_classic_and_overlay_scrollbars() {
    test_thread::run(|| {
        for overlay in [false, true] {
            for (fill, client_width, client_height) in [
                ("height:400px", if overlay { 300 } else { 285 }, 200),
                (
                    "width:400px;height:100px",
                    300,
                    if overlay { 200 } else { 185 },
                ),
                (
                    "width:400px;height:400px",
                    if overlay { 300 } else { 285 },
                    if overlay { 200 } else { 185 },
                ),
            ] {
                let mut page = fixture(
                    &format!(
                        "#fill{{{fill}}}#fixed{{position:fixed;right:24px;bottom:20px;width:44px;height:80px;background:rgb(20,40,60);border-radius:22px}}"
                    ),
                    "<div id=fill></div><div id=fixed></div>",
                    overlay,
                );
                dimensions(&mut page, client_width, client_height);
                // A rounded drawing must reach the full property-state
                // validator; the rect-only redundant-clip proof is insufficient.
                paint_fixed_control(
                    &page,
                    300,
                    200,
                    client_width as usize - 46,
                    client_height as usize - 60,
                );
            }
        }
    });
}

#[test]
fn fixed_control_root_clip_tracks_scrollbar_mutation_resize_and_scroll() {
    test_thread::run(|| {
        let mut page = fixture(
            "#fill{height:400px}#fixed{position:fixed;right:24px;bottom:20px;width:44px;height:80px;background:rgb(20,40,60);border-radius:22px}",
            "<div id=fill></div><div id=fixed></div>",
            false,
        );
        paint_fixed_control(&page, 300, 200, 239, 140);
        page.ScrollWheelDefault(&interaction::input_event::WheelEvent {
            phase: interaction::input_event::WheelPhase::kEnded,
            position: interaction::input_event::Offset { x: 100.0, y: 100.0 },
            delta: interaction::input_event::Offset { x: 0.0, y: 80.0 },
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            root_paint(&page.CurrentFrame().unwrap().fragments)
                .paint
                .scroll_offset
                .y,
            80.0
        );
        paint_fixed_control(&page, 300, 200, 239, 140);
        check(
            &mut page,
            "document.getElementById('fill').setAttribute('style','height:100px')",
        );
        dimensions(&mut page, 300, 200);
        paint_fixed_control(&page, 300, 200, 254, 140);
        check(
            &mut page,
            "document.getElementById('fill').setAttribute('style','height:400px')",
        );
        dimensions(&mut page, 285, 200);
        paint_fixed_control(&page, 300, 200, 239, 140);
        page.ResizeViewport(300.0, 500.0, 1.0).unwrap();
        dimensions(&mut page, 300, 500);
        paint_fixed_control(&page, 300, 500, 254, 440);
        page.ResizeViewport(300.0, 200.0, 1.0).unwrap();
        dimensions(&mut page, 285, 200);
        paint_fixed_control(&page, 300, 200, 239, 140);
    });
}

#[test]
fn fixed_control_uses_its_source_containing_block_clip() {
    test_thread::run(|| {
        let page = fixture(
            "#fill{height:400px}#clip{width:20px;height:20px;overflow:hidden}#fixed{position:fixed;right:24px;bottom:20px;width:44px;height:80px;background:rgb(20,40,60);border-radius:22px}",
            "<div id=clip><div id=fixed></div></div><div id=fill></div>",
            false,
        );
        // An unrelated logical ancestor's overflow must not replace the
        // LayoutView clip used by a viewport-fixed descendant.
        paint_fixed_control(&page, 300, 200, 239, 140);

        let page = fixture(
            "#fill{height:400px}#clip{margin:20px;width:100px;height:100px;overflow:hidden;transform:translateX(10px)}#fixed{position:fixed;left:10px;top:10px;width:44px;height:40px;background:rgb(20,40,60);border-radius:10px}",
            "<div id=clip><div id=fixed></div></div><div id=fill></div>",
            false,
        );
        // A transformed containing block remains on the ordinary native
        // geometry path, rather than acquiring the root fixed context.
        paint_fixed_control(&page, 300, 200, 62, 50);
    });
}
