//! Public host boundary smoke tests. This target is independent of historical
//! browser unit-test fixtures that are absent from source-only checkouts.
use browser::system_font_provider::BrowserSystemFontProvider;
use font_engine::{
    FontContextVersion, FontPresentation, SharedFontBytes, SystemFontAnswer, SystemFontProvider,
    SystemFontQuery, SystemFontRequest,
};

fn Request(query: SystemFontQuery) -> SystemFontRequest {
    SystemFontRequest {
        id: 1,
        context_version: FontContextVersion(9),
        query,
    }
}

#[test]
fn metadata_queries_and_explicit_opens_keep_native_face_identity() {
    let mut provider = BrowserSystemFontProvider::new(FontContextVersion(9));
    let query = Request(SystemFontQuery::MatchGeneric {
        generic: "sans-serif".into(),
        weight: 400.,
        italic: false,
        locale: "en".into(),
    });
    let SystemFontAnswer::Faces(faces) = provider.Resolve(&query) else {
        panic!("metadata query has wrong answer kind");
    };
    let Some(selected) = faces.first() else {
        return;
    }; // A fontless platform is supported.
    assert!(selected.face.bytes.is_empty());
    assert_eq!(selected.face.family, "sans-serif");
    assert!(!selected.face.native_family.is_empty());
    let open = Request(SystemFontQuery::OpenFace {
        id: selected.id.clone(),
    });
    let SystemFontAnswer::Face(first) = provider.Resolve(&open) else {
        panic!("installed generic face must open");
    };
    let SystemFontAnswer::Face(second) = provider.Resolve(&open) else {
        panic!("cached face must remain open");
    };
    assert!(SharedFontBytes::ptr_eq(&first.bytes, &second.bytes));
    let font =
        font_engine::OpenTypeFont::new_shared(first.bytes.clone(), first.face_index, 16., &[]);
    assert!(font.GlyphForCharacter('A' as u32).is_some());
    println!(
        "opened {} collection-index={} bytes={}",
        first.native_family,
        first.face_index,
        first.bytes.len()
    );
    provider.SetContextVersion(FontContextVersion(10));
    assert_eq!(provider.Resolve(&open), SystemFontAnswer::Unavailable);
    let mut fresh = open;
    fresh.context_version = FontContextVersion(10);
    assert_eq!(provider.Resolve(&fresh), SystemFontAnswer::Unavailable);
}

#[test]
fn local_name_miss_does_not_substitute_a_default_font() {
    let mut provider = BrowserSystemFontProvider::new(FontContextVersion(9));
    assert_eq!(
        provider.Resolve(&Request(SystemFontQuery::MatchUniqueName {
            name: "Rechrom-Missing-Unique-Name-1580f7a8".into(),
        })),
        SystemFontAnswer::Faces(Vec::new())
    );
}

#[test]
fn contextual_fallback_roundtrips_exact_ttc_faces() {
    let mut provider = BrowserSystemFontProvider::new(FontContextVersion(9));
    let SystemFontAnswer::Faces(base) = provider.Resolve(&Request(SystemFontQuery::MatchGeneric {
        generic: "serif".into(),
        weight: 400.,
        italic: false,
        locale: "zh-Hant".into(),
    })) else {
        panic!("metadata query has wrong answer kind");
    };
    let Some(base) = base.first() else {
        return;
    };
    for (text, presentation) in [
        ("水", FontPresentation::Text),
        ("😀", FontPresentation::Emoji),
    ] {
        let SystemFontAnswer::Faces(candidates) =
            provider.Resolve(&Request(SystemFontQuery::Fallback {
                text: text.encode_utf16().collect(),
                weight: 400.,
                italic: false,
                locale: "zh-Hant".into(),
                presentation,
                base_face: Some(base.id.clone()),
            }))
        else {
            panic!("valid base face must remain usable");
        };
        let mut found = false;
        for descriptor in &candidates {
            assert!(descriptor.face.bytes.is_empty());
            let SystemFontAnswer::Face(face) =
                provider.Resolve(&Request(SystemFontQuery::OpenFace {
                    id: descriptor.id.clone(),
                }))
            else {
                continue;
            };
            let font = font_engine::OpenTypeFont::new_shared(face.bytes, face.face_index, 16., &[]);
            if font
                .GlyphForCharacter(text.chars().next().unwrap() as u32)
                .is_some()
            {
                println!(
                    "contextual {} -> {} collection-index={}",
                    text, face.native_family, face.face_index
                );
                found = true;
                break;
            }
        }
        if !candidates.is_empty() {
            assert!(
                found,
                "platform metadata must identify a real character-bearing face"
            );
        }
    }
}
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;
#[test]
fn page_settles_system_font_requests_before_publishing_glyphs() {
    native_test_thread::run(|| {
        use std::{cell::RefCell, io, rc::Rc};
        use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
        struct Operation(Option<URLResponse>);
        impl URLLoadOperation for Operation {
            fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
                Ok(self.0.take())
            }
        }
        struct Loader;
        impl URLLoader for Loader {
            fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
                Ok(Box::new(Operation(Some(URLResponse { final_url: request.url.clone(), status_code: 200,
                    mime_type: "text/html".into(), body: "<!doctype html><style>body{font:20px sans-serif}b{font-weight:700}</style><p>office <b>bold</b></p>".as_bytes().to_vec(),
                    ..Default::default() }))))
            }
        }
        let assembly = browser::CreateLayoutAssembly();
        let constraints = browser::CreateBrowserConstraints(320, 120);
        let emergency = constraints.fonts[0].bytes.clone();
        assert_eq!(
            constraints.fonts.len(),
            1,
            "construction must not preload the OS font catalog"
        );
        struct Client {
            emergency: SharedFontBytes,
            text_frames: Rc<RefCell<usize>>,
        }
        impl browser::page::PageClient for Client {
            fn DidPresentFrame(&mut self, frame: &browser::page::PageFrame) {
                let mut fonts = vec![];
                collect(&frame.fragments, &mut fonts);
                if !fonts.is_empty() {
                    assert!(
                        fonts
                            .iter()
                            .all(|font| !SharedFontBytes::ptr_eq(font, &self.emergency)),
                        "no callback may observe speculative emergency text"
                    );
                    *self.text_frames.borrow_mut() += 1;
                }
            }
        }
        struct Injected {
            inner: BrowserSystemFontProvider,
            calls: Rc<RefCell<usize>>,
        }
        impl SystemFontProvider for Injected {
            fn SetContextVersion(&mut self, version: FontContextVersion) {
                self.inner.SetContextVersion(version);
            }
            fn Resolve(&mut self, request: &SystemFontRequest) -> SystemFontAnswer {
                *self.calls.borrow_mut() += 1;
                std::thread::sleep(std::time::Duration::from_millis(1));
                self.inner.Resolve(request)
            }
        }
        let text_frames = Rc::new(RefCell::new(0));
        let calls = Rc::new(RefCell::new(0));
        let mut page = browser::page::Page::Create(
            Rc::new(RefCell::new(Loader)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
            constraints,
            None,
            Some(Rc::new(RefCell::new(Client {
                emergency: emergency.clone(),
                text_frames: text_frames.clone(),
            }))),
        );
        page.SetSystemFontProvider(Box::new(Injected {
            inner: BrowserSystemFontProvider::default(),
            calls: calls.clone(),
        }));
        page.Open("https://font-host.test/", 16384, 4096).unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if !page.IsLoading() {
                break;
            }
        }
        assert!(!page.IsLoading());
        let frame = page.CurrentFrame().expect("settled font/layout frame");
        fn collect(
            node: &layoutng_assembly::fragment_tree::FragmentPaintData,
            fonts: &mut Vec<SharedFontBytes>,
        ) {
            for run in &node.paint.glyph_runs {
                if let Some(resources) = &node.paint.resources {
                    if let Some(face) = resources.fonts.get(run.font_face_index as usize) {
                        fonts.push(face.bytes.clone());
                    }
                }
            }
            for child in &node.children {
                collect(child, fonts);
            }
        }
        let mut selected = Vec::new();
        collect(&frame.fragments, &mut selected);
        assert!(
            !selected.is_empty(),
            "real text must produce retained glyph runs"
        );
        assert!(
            selected
                .iter()
                .any(|font| !SharedFontBytes::ptr_eq(font, &emergency)),
            "published glyphs must use lazily admitted OS bytes rather than the emergency face"
        );
        assert!(*text_frames.borrow() > 0);
        assert!(
            *calls.borrow() > 0,
            "externally injected provider must be used"
        );
        let old_snapshot = frame.fragments.clone();
        let before = *calls.borrow();
        page.SystemFontsChanged();
        page.RunTask().unwrap();
        assert!(
            *calls.borrow() > before,
            "environment change must requery host"
        );
        let mut retained = vec![];
        collect(&old_snapshot, &mut retained);
        assert!(
            retained.iter().all(|bytes| !bytes.is_empty()),
            "old frame keeps immutable font bytes"
        );
    });
}
