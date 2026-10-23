//! Window CSSOM View calls use the existing viewport scroll owner and lifecycle.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! third_party/blink/renderer/core/frame/local_dom_window.cc:1832-1876,1942-2083.
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
#[test]
fn numeric_window_scroll_updates_native_viewport_geometry_and_events() {
    native_test_thread::run(|| {
        let mut page = fixture("body{width:1000px;height:2400px}#anchor{position:absolute;left:600px;top:1500px;width:20px;height:20px}#fixed{position:fixed;top:0;left:0;width:20px;height:20px}", "<div id=anchor></div><div id=fixed></div>", true);
        let old = page.CurrentFrame().unwrap().fragments.clone();
        check(
            &mut page,
            r#"
          if (typeof window.scroll !== 'function' || typeof window.scrollTo !== 'function' || typeof window.scrollBy !== 'function') throw Error('Window scroll family missing');
          var events=[]; document.addEventListener('scroll',()=>events.push('document'));
          window.addEventListener('scroll',()=>events.push('window'));
          var results=[],order=['call'],promise=window.scroll(50,100);
          if(!(promise instanceof Promise))throw Error('Promise return');
          promise.then(result=>{results.push(result);order.push('completion');});
          Promise.resolve().then(()=>order.push('microtask'));
          if(scrollX!==50||scrollY!==100||pageXOffset!==50||pageYOffset!==100)throw Error('offset getters');
          window.scrollTo(75,200); window.scrollBy(25,50);
          if(document.documentElement.scrollLeft!==100||document.documentElement.scrollTop!==250)throw Error('native owner');
          var r=document.getElementById('anchor').getBoundingClientRect(),f=document.getElementById('fixed').getBoundingClientRect();
          if(Math.abs(r.x-500)>.01||Math.abs(r.y-1250)>.01||Math.abs(f.y)>.01)throw Error('native geometry');
        "#,
        );
        check(&mut page, "if(order.join(',')!=='call,microtask'||results.length)throw Error('premature scroll completion');");
        for _ in 0..4 {
            page.RunTask().unwrap();
        }
        assert_eq!(
            old.paint.scroll_offset,
            layoutng_assembly::internal::layout_input::Offset::default()
        );
        check(&mut page, "if(order.join(',')!=='call,microtask,completion')throw Error('task completion order');if(JSON.stringify(events)!=='[\"document\",\"window\"]')throw Error('coalesced scroll events '+JSON.stringify(events)); window.scrollTo(100,250);");
        for _ in 0..4 {
            page.RunTask().unwrap();
        }
        check(&mut page, "if(events.length!==2)throw Error('unchanged offset event');if(JSON.stringify(results)!=='[{\"interrupted\":false}]')throw Error('completion result');");
    });
}

#[test]
fn scroll_dictionary_conversion_normalization_and_layout_flush() {
    native_test_thread::run(|| {
        let mut page = fixture(
            "body{width:1000px;height:2400px}",
            "<div id=anchor style='height:20px'></div>",
            true,
        );
        check(
            &mut page,
            r#"
          function position(x,y,label){if(scrollX!==x||scrollY!==y)throw Error(label+': '+scrollX+','+scrollY);}
          scrollTo(40,80); scrollTo({top:120}); position(40,120,'partial absolute');
          scrollBy({left:20}); position(60,120,'partial relative');
          scroll();scrollTo(null);scrollBy();position(60,120,'empty dictionaries');
          scrollBy(NaN,Infinity);position(60,120,'nonfinite relative');
          scrollTo(NaN,Infinity);position(0,0,'nonfinite absolute');
          scrollTo(1e300,-1e300);position(0,0,'float overflow');
          scrollTo(3.4028234663852886e38+1e25,0);position(0,0,'just above float maximum');
          scrollTo(20.5,30.5);position(21,31,'physical rounding');
          var conversion=[];
          scrollTo({get behavior(){conversion.push('behavior');return 'instant'},get left(){conversion.push('left');return {valueOf(){conversion.push('left number');return 25}}},get top(){conversion.push('top');return 35}});
          position(25,35,'converted coordinates');
          var hint=[];scrollTo({behavior:{toString(){hint.push('string');return 'instant'},valueOf(){throw Error('wrong conversion hint')}}});
          if(hint.join(',')!=='string')throw Error('enum ToString hint');
          if(conversion.join('|')!=='behavior|left|left number|top')throw Error('dictionary order');
          var errors=[];scrollTo(1).catch(e=>errors.push(e.name));
          scrollTo({behavior:'invalid',get top(){throw Error('must not read top')}}).catch(e=>errors.push(e.name));
          scrollBy(1n,0).catch(e=>errors.push(e.name));
          scroll.call({},0,0).catch(e=>errors.push(e.name));
          position(25,35,'invalid requests atomic');
          document.body.style.height='4000px';scrollTo(0,3000);position(0,3000,'fresh layout range');
          scrollTo(-100,-100);position(0,0,'lower range');
          scrollTo(1e6,1e6);position(700,3800,'upper native range');
        "#,
        );
        check(&mut page,"if(errors.join(',')!=='TypeError,TypeError,TypeError,TypeError')throw Error('rejection conversion '+errors)");
    });
}

#[test]
fn viewport_scroll_uses_page_zoom_and_negative_native_ranges() {
    native_test_thread::run(|| {
        for zoom in [2.0, 0.8] {
            let mut page = fixture_with_theme(
                "body{width:1000px;height:2400px}",
                "<div></div>",
                true,
                15,
                7,
                zoom,
            );
            check(
                &mut page,
                &format!(
                    r#"
              scrollTo(10.3,100.3);
              var z={zoom},x=Math.floor(Math.fround(10.3)*z+.5),y=Math.floor(Math.fround(100.3)*z+.5);
              var expectedX=Math.fround(x/Math.fround(z)),expectedY=Math.fround(y/Math.fround(z));
              if(scrollX!==expectedX||scrollY!==expectedY)throw Error('page zoom '+JSON.stringify({{scrollX,scrollY,expectedX,expectedY}}));
              if(pageXOffset!==scrollX||pageYOffset!==scrollY)throw Error('zoom aliases');
            "#
                ),
            );
        }
        let mut page = fixture(
            "html{direction:rtl}body{width:1000px;height:2400px}",
            "<div></div>",
            true,
        );
        check(
            &mut page,
            r#"
          scrollTo(-50.5,10.5);
          if(scrollX!==-50||scrollY!==11)throw Error('negative half '+scrollX+','+scrollY);
          scrollBy(-.5,-.5);if(scrollX!==-50||scrollY!==11)throw Error('relative half');
          scrollTo(-1e6,0);if(scrollX!==-700)throw Error('negative native minimum '+scrollX);
          scrollTo(100,0);if(scrollX!==0)throw Error('negative range maximum');
          var descriptor=Object.getOwnPropertyDescriptor(window,'scrollX'),badReceiver=false;
          try{descriptor.get.call({})}catch(e){badReceiver=e instanceof TypeError}
          if(!badReceiver||descriptor.get.call(null)!==0||descriptor.get.call(undefined)!==0)throw Error('getter receiver');
          window.pageXOffset=42;if(pageXOffset!==42||scrollX!==0)throw Error('replaceable alias');
        "#,
        );
    });
}

#[test]
fn unsupported_scroll_animation_and_snap_are_explicit_and_atomic() {
    native_test_thread::run(|| {
        let mut page = fixture(
            "html{scroll-behavior:smooth}body{height:2400px}",
            "<div></div>",
            true,
        );
        check(
            &mut page,
            r#"
          var unsupported=[];
          scrollTo(0,100).catch(e=>unsupported.push(e.name));
          scrollBy({top:100,behavior:'smooth'}).catch(e=>unsupported.push(e.name));
          if(scrollY!==0)throw Error('unsupported changed offset');
          scrollTo({top:100,behavior:'instant'});if(scrollY!==100)throw Error('instant overrides CSS smooth');
          document.documentElement.style.scrollBehavior='auto';
          document.documentElement.style.scrollSnapType='y mandatory';
          scrollTo({top:200,behavior:'instant'}).catch(e=>unsupported.push(e.name));
          if(scrollY!==100)throw Error('snap unsupported changed offset');
        "#,
        );
        check(&mut page,"if(unsupported.join(',')!=='NotSupportedError,NotSupportedError,NotSupportedError')throw Error('unsupported results '+unsupported)");
    });
}

#[test]
fn relative_scroll_preserves_native_scroll_origin_float_roundtrip() {
    native_test_thread::run(|| {
        let mut page = fixture(
            "html{direction:rtl}body{width:1324px;height:2400px}",
            "<div></div>",
            true,
        );
        check(
            &mut page,
            r#"
            scrollTo(-1,0);scrollBy(.49998,0);
            if(scrollX!==0)throw Error('relative scroll-position precision '+scrollX);
        "#,
        );
    });
}
