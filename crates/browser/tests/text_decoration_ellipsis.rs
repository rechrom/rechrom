//! Ellipsis glyphs are generated text, but Chromium deliberately excludes
//! them from TextDecorationPainter::UpdateDecorationInfo.

#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use layoutng_assembly::{internal::layout_input::FontFace, layout_engine::PaintFragmentCursor};
use paint::paint_engine::DisplayItemType;
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

fn find_ellipsis(fragment: PaintFragmentCursor<'_>) -> Option<PaintFragmentCursor<'_>> {
    if fragment.PaintData().paint.is_ellipsis {
        return Some(fragment);
    }
    fragment.Children().into_iter().find_map(find_ellipsis)
}

#[test]
fn generated_ellipsis_paints_glyphs_without_text_decoration() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><div style="width:70px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font:24px/32px EllipsisDecorationFixture;text-decoration:underline 3px red;text-decoration-skip-ink:none">decorated text must overflow</div></body>"#;
        let assembly = browser::CreateLayoutAssembly();
        let mut constraints = browser::CreateBrowserConstraints(240, 80);
        constraints.fonts = vec![FontFace {
            family: "EllipsisDecorationFixture".into(),
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
        page.Open("https://ellipsis-decoration.test/", 16_384, 4_096)
            .unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if !page.IsLoading() {
                break;
            }
        }
        assert!(!page.IsLoading());

        let frame = page.CurrentFrame().expect("painted frame");
        let ellipsis = find_ellipsis(PaintFragmentCursor::FromHandle(&frame.fragments))
            .expect("generated ellipsis fragment");
        let fragment_instance_id = ellipsis.fragment_instance_id;
        assert!(frame.display_items.items.iter().any(|item| {
            item.fragment_instance_id == fragment_instance_id
                && item.r#type == DisplayItemType::kDrawGlyphRun
        }));
        assert!(!frame.display_items.items.iter().any(|item| {
            item.fragment_instance_id == fragment_instance_id && item.is_text_decoration
        }));
    });
}
