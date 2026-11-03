//! Applied text decorations participate in native FragmentItem ink overflow.
//! This exercises the retained InlinePaintContext path, rather than Paint's
//! export fallback for undecorated text.

#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use layoutng_assembly::{
    internal::layout_input::{FontFace, NodeKind},
    layout_engine::PaintFragmentCursor,
};
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

fn find_text(fragment: PaintFragmentCursor<'_>) -> Option<PaintFragmentCursor<'_>> {
    if fragment.PaintData().paint.source_kind == NodeKind::kText {
        return Some(fragment);
    }
    fragment.Children().into_iter().find_map(find_text)
}

#[test]
fn thick_offset_underline_expands_source_text_ink_overflow() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><style>
            body { margin: 0 }
            span {
                font: 20px/20px DecorationFixture;
                text-decoration-line: underline;
                text-decoration-style: solid;
                text-decoration-thickness: 8px;
                text-underline-offset: 12px;
            }
        </style><span>Decorated text</span>"#;
        let assembly = browser::CreateLayoutAssembly();
        let mut constraints = browser::CreateBrowserConstraints(320, 120);
        constraints.fonts = vec![FontFace {
            family: "DecorationFixture".into(),
            bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
                .as_slice()
                .into(),
            ..Default::default()
        }]
        .into();
        let mut page = Page::Create(
            Rc::new(RefCell::new(FixtureLoader(html.to_vec()))),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
            constraints,
            None,
            None,
        );
        page.Open("https://decoration.test/", 16_384, 4_096)
            .unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if !page.IsLoading() {
                break;
            }
        }
        assert!(!page.IsLoading());

        let frame = page.CurrentFrame().unwrap();
        let text =
            find_text(PaintFragmentCursor::FromHandle(&frame.fragments)).expect("text fragment");
        let text = text.PaintData();
        assert!(text
            .paint
            .style
            .applied_text_decorations
            .iter()
            .any(|decoration| decoration.underline));
        let overflow = text
            .paint
            .self_ink_overflow
            .expect("decorated text must publish native self ink overflow");
        assert!(
            overflow.offset.y + overflow.size.height > text.size.height,
            "underline overflow {overflow:?} must extend below text fragment {:?}",
            text.size
        );
    });
}
