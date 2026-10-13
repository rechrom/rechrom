//! External SVG graphics through the production resource, image, mask and raster path.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;
use browser::page::{Page, ScriptEnvironment};
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct Loader {
    html: String,
    svg: String,
    requests: Rc<RefCell<Vec<String>>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, r: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        self.requests.borrow_mut().push(r.url.clone());
        if r.url.ends_with("broken.svg") {
            return Err(io::Error::other("controlled transport failure"));
        }
        let image = r.url.ends_with(".svg");
        Ok(Box::new(Response(Some(URLResponse {
            final_url: r.url.clone(),
            status_code: 200,
            mime_type: if image { "image/svg+xml" } else { "text/html" }.into(),
            body: if image { &self.svg } else { &self.html }
                .as_bytes()
                .to_vec(),
            ..Default::default()
        }))))
    }
}
const GRAPHIC: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24"><path d="m21 21-4.34-4.34"/><circle cx="11" cy="11" r="8"/></svg>"#;
struct XHR;
impl xhr_transport::XMLHttpRequestTransport for XHR {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(io::Error::other("unexpected XHR"))
    }
}
fn fixture_html(html: String, svg: &str) -> (Page, Rc<RefCell<Vec<String>>>) {
    let requests = Rc::new(RefCell::new(Vec::new()));
    let a = browser::CreateLayoutAssembly();
    let c = browser::CreateBrowserConstraints(160, 60);
    let decoder = browser::CreateSVGImageDecoderWithConstraints(&a, &c);
    let mut p = Page::Create(
        Rc::new(RefCell::new(Loader {
            html,
            svg: svg.into(),
            requests: requests.clone(),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(decoder)),
        c,
        Some(ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::with_native_stack_budget(
                4 * 1024 * 1024,
            )),
            xhr: Box::new(XHR),
            user_agent: "svg-mask-fixture".into(),
        }),
        None,
    );
    p.Open("https://svg-mask.test/", 16384, 4096).unwrap();
    for _ in 0..30 {
        p.RunTask().unwrap();
    }
    assert!(!p.IsLoading());
    (p, requests)
}
fn fixture(svg: &str) -> (Page, Rc<RefCell<Vec<String>>>) {
    fixture_html(
        format!(
            r#"<!doctype html><style>body{{margin:0;background:white}}.mask{{position:absolute;left:0;top:0;width:24px;height:24px;background:red;mask-image:url(graphic.svg);mask-size:contain;mask-repeat:no-repeat}}img{{position:absolute;left:40px;top:0}}.inline{{position:absolute;left:80px;top:0}}</style><div class=mask></div><img src=graphic.svg><div class=inline>{svg}</div>"#
        ),
        svg,
    )
}
fn evaluate(page: &mut Page, script: &str) {
    let r = page.Evaluate(script, "test:svg-mask-graphics").unwrap();
    assert!(r.Succeeded(), "{:?}", r.exception);
    for _ in 0..30 {
        page.RunTask().unwrap();
    }
}

#[test]
fn stroked_svg_is_visible_as_mask_and_image() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (p, requests) = fixture(GRAPHIC);
        let mut pixels = vec![0; 160 * 60];
        p.PaintArtifactInto(
            &p.CurrentFrame().unwrap().display_items,
            160,
            60,
            1.0,
            &mut pixels,
            renderer::PixelFormat::Rgba8888,
            160,
        )
        .unwrap();
        let count = |left: usize| {
            (0..24)
                .flat_map(|y| (left..left + 24).map(move |x| y * 160 + x))
                .filter(|i| pixels[*i].to_ne_bytes() != [255, 255, 255, 255])
                .count()
        };
        eprintln!(
            "requests {:?}; ink mask={} image={} inline={}",
            requests.borrow(),
            count(0),
            count(40),
            count(80)
        );
        assert!(
            count(0) > 75,
            "external stroked SVG mask has no graphic pixels"
        );
        assert!(
            count(40) > 75,
            "external stroked SVG image has no graphic pixels"
        );
        assert!(count(80) > 75, "inline SVG control has no graphic pixels");
    });
}

#[test]
fn connected_shadow_pseudo_mask_fetches_and_paints_external_graphic() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (mut p, requests) = fixture_html(
            "<!doctype html><style>body{margin:0;background:white}</style><div id=host></div>"
                .into(),
            GRAPHIC,
        );
        evaluate(
            &mut p,
            r#"const root=document.getElementById('host').attachShadow({mode:'open'});const style=document.createElement('style');style.textContent='button{display:flex;margin:0;padding:0;border:0;background:none;color:red}button::before{content:"";display:block;width:24px;height:24px;background:currentColor;mask-image:url(shadow.svg);mask-size:contain;mask-repeat:no-repeat}';const button=document.createElement('button');button.id='shadow-mask';root.appendChild(style);root.appendChild(button);"#,
        );
        let mut pixels = vec![0; 160 * 60];
        p.PaintArtifactInto(
            &p.CurrentFrame().unwrap().display_items,
            160,
            60,
            1.0,
            &mut pixels,
            renderer::PixelFormat::Rgba8888,
            160,
        )
        .unwrap();
        let ink = pixels
            .iter()
            .filter(|x| x.to_ne_bytes() != [255, 255, 255, 255])
            .count();
        eprintln!("shadow requests {:?}; ink {}", requests.borrow(), ink);
        assert!(
            requests.borrow().iter().any(|s| s.ends_with("/shadow.svg")),
            "connected shadow pseudo's native style image was never fetched"
        );
        assert!(ink > 75, "loaded external SVG mask has no graphic pixels");
    });
}

fn artifact_pixels(
    page: &Page,
    artifact: &std::sync::Arc<paint::paint_engine::PaintArtifact>,
) -> Vec<u32> {
    let mut pixels = vec![0; 160 * 60];
    page.PaintArtifactInto(
        artifact,
        160,
        60,
        1.0,
        &mut pixels,
        renderer::PixelFormat::Rgba8888,
        160,
    )
    .unwrap();
    pixels
}
fn pixels(page: &Page) -> Vec<u32> {
    artifact_pixels(page, &page.CurrentFrame().unwrap().display_items)
}
fn empty_fixture() -> (Page, Rc<RefCell<Vec<String>>>) {
    fixture_html(
        "<!doctype html><style>body{margin:0;background:white}</style><div id=host></div>".into(),
        GRAPHIC,
    )
}
fn assert_native_mask_ready(page: &Page, id: &str) {
    let owner = page.Document();
    let d = owner.GetDocument();
    let i = (0..d.NodeCount())
        .find(|&i| d.Node(i).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap();
    let r = d.ResolvedStyleFor(i).unwrap();
    let pseudo = r.before.as_ref().unwrap();
    let style = unsafe { &*pseudo.native_style.Get() };
    let image = unsafe { &*style.MaskLayers().GetImage() };
    assert!(image.IsLoaded() && image.CanRender() && !image.IsLoading() && !image.ErrorOccurred());
    let sources = r.ReferencedStyleImageSources();
    assert_eq!(sources.len(), 1);
    let image = d
        .ImageResourceFor(&sources[0])
        .expect("same canonical source owns decoded resource");
    assert_ne!(image.id, 0);
}
#[test]
fn closed_nested_shadow_masks_match_static_graphic_pixels_and_keep_old_artifact() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (control, _) = fixture(GRAPHIC);
        let expected = pixels(&control);
        let (mut p, requests) = empty_fixture();
        let old = p.CurrentFrame().unwrap().display_items.clone();
        let old_pixels = pixels(&p);
        evaluate(
            &mut p,
            r#"const outer=document.getElementById('host').attachShadow({mode:'closed'});const inner=document.createElement('div');outer.appendChild(inner);const root=inner.attachShadow({mode:'closed'});const style=document.createElement('style');style.textContent='div::before{content:"";display:block;width:24px;height:24px;background:red;mask:url(nested.svg) no-repeat;mask-size:contain}';const target=document.createElement('div');target.id='nested-mask';root.appendChild(style);root.appendChild(target);"#,
        );
        let actual = pixels(&p);
        for y in 0..24 {
            for x in 0..24 {
                assert_eq!(
                    actual[y * 160 + x],
                    expected[y * 160 + x],
                    "graphic pixel {x},{y}"
                );
            }
        }
        assert_native_mask_ready(&p, "nested-mask");
        assert_eq!(
            requests
                .borrow()
                .iter()
                .filter(|s| s.ends_with("/nested.svg"))
                .count(),
            1
        );
        assert_eq!(artifact_pixels(&p, &old), old_pixels);
        evaluate(
            &mut p,
            "style.textContent=style.textContent.replace('nested.svg','replacement.svg')",
        );
        assert_native_mask_ready(&p, "nested-mask");
        assert_eq!(pixels(&p), actual);
        assert_eq!(
            requests
                .borrow()
                .iter()
                .filter(|s| s.ends_with("/replacement.svg"))
                .count(),
            1
        );
        assert_eq!(artifact_pixels(&p, &old), old_pixels);
    });
}
#[test]
fn detached_shadow_css_waits_for_connection_and_shared_urls_are_deduplicated() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (mut p, requests) = empty_fixture();
        evaluate(
            &mut p,
            r#"const detached=document.createElement('div');const root=detached.attachShadow({mode:'open'});const style=document.createElement('style');style.textContent='.graphic{width:24px;height:24px;background:url(shared.svg) no-repeat}';root.appendChild(style);for(let i=0;i<2;i++){const e=document.createElement('div');e.className='graphic';root.appendChild(e)}"#,
        );
        assert_eq!(requests.borrow().len(), 1);
        assert!(pixels(&p)
            .iter()
            .all(|p| p.to_ne_bytes() == [255, 255, 255, 255]));
        evaluate(
            &mut p,
            "document.getElementById('host').appendChild(detached)",
        );
        assert_eq!(
            requests
                .borrow()
                .iter()
                .filter(|s| s.ends_with("/shared.svg"))
                .count(),
            1
        );
        assert!(
            pixels(&p)
                .iter()
                .filter(|p| p.to_ne_bytes() != [255, 255, 255, 255])
                .count()
                > 250
        );
    });
}
#[test]
fn shadow_and_light_image_load_events_follow_shadow_including_order_once() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (mut p, requests) = empty_fixture();
        evaluate(
            &mut p,
            r#"var events=[];const host=document.getElementById('host');const root=host.attachShadow({mode:'closed'});const slot=document.createElement('slot');slot.setAttribute('name','picture');root.appendChild(slot);const shadow=document.createElement('img');shadow.style.cssText='position:absolute;left:40px;top:0';shadow.src='events.svg';shadow.addEventListener('load',()=>events.push('shadow'));root.appendChild(shadow);const light=document.createElement('img');light.setAttribute('slot','picture');light.style.cssText='position:absolute;left:0;top:0';light.src='events.svg';light.addEventListener('load',()=>events.push('light'));host.appendChild(light);const unassigned=document.createElement('img');unassigned.setAttribute('slot','absent');unassigned.src='events.svg';unassigned.addEventListener('load',()=>events.push('unassigned'));host.appendChild(unassigned);"#,
        );
        evaluate(&mut p,"if(JSON.stringify(events)!=='[\"shadow\",\"light\",\"unassigned\"]')throw new Error(JSON.stringify(events))");
        assert_eq!(
            requests
                .borrow()
                .iter()
                .filter(|s| s.ends_with("/events.svg"))
                .count(),
            1
        );
        assert!(
            pixels(&p)
                .iter()
                .filter(|p| p.to_ne_bytes() != [255, 255, 255, 255])
                .count()
                > 250
        );
    });
}
#[test]
fn failed_shadow_image_delivers_error_once() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (mut p, requests) = empty_fixture();
        evaluate(
            &mut p,
            r#"var events=[];const root=document.getElementById('host').attachShadow({mode:'open'});const image=document.createElement('img');image.addEventListener('load',()=>events.push('load'));image.addEventListener('error',()=>events.push('error'));image.src='broken.svg';root.appendChild(image);"#,
        );
        evaluate(
            &mut p,
            "if(JSON.stringify(events)!=='[\"error\"]')throw new Error(JSON.stringify(events))",
        );
        assert_eq!(
            requests
                .borrow()
                .iter()
                .filter(|s| s.ends_with("/broken.svg"))
                .count(),
            1
        );
    });
}
