//! SVG text decorations use the declaring ancestor's paint, not the leaf
//! LayoutSVGInlineText paint.  Paint-order remains stroke/fill ordered.

#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use layoutng_assembly::{
    internal::layout_input::{FontFace, NodeKind},
    layout_engine::PaintFragmentCursor,
};
use paint::paint_engine::DecorationPaintPass;
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

fn find_svg_text(fragment: PaintFragmentCursor<'_>) -> Option<PaintFragmentCursor<'_>> {
    let paint = fragment.PaintData();
    if paint.paint.source_kind == NodeKind::kText && paint.paint.svg_text.is_some() {
        return Some(fragment);
    }
    fragment.Children().into_iter().find_map(find_svg_text)
}

#[test]
fn svg_decoration_uses_declaring_ancestor_and_paint_order() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><svg width="320" height="100">
          <text y="40" style="font:20px DecorationFixture;fill:red;stroke:green;stroke-width:2px;paint-order:stroke fill;text-decoration-line:underline;text-decoration-style:dashed;outline:3px solid black">
            <tspan style="fill:blue;stroke:purple">Decorated</tspan>
          </text>
        </svg>"#;
        let assembly = browser::CreateLayoutAssembly();
        let mut constraints = browser::CreateBrowserConstraints(320, 100);
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
        page.Open("https://svg-decoration.test/", 16_384, 4_096)
            .unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if !page.IsLoading() {
                break;
            }
        }
        assert!(!page.IsLoading());

        let frame = page.CurrentFrame().unwrap();
        let text = find_svg_text(PaintFragmentCursor::FromHandle(&frame.fragments))
            .expect("SVG inline text fragment");
        let text_data = text.PaintData();
        let decoration_style = text_data
            .paint
            .svg_text_decoration_style
            .as_ref()
            .expect("declaring SVG decoration style");
        let fill = decoration_style.svg_fill.expect("ancestor fill");
        let stroke = decoration_style.svg_stroke.expect("ancestor stroke");
        assert!(fill.red > 0.99 && fill.green < 0.01 && fill.blue < 0.01);
        assert!(stroke.green > 0.49 && stroke.red < 0.01 && stroke.blue < 0.01);

        let passes: Vec<_> = frame
            .display_items
            .items
            .iter()
            .filter(|item| item.is_text_decoration && !item.is_shadow)
            .map(|item| item.decoration_paint_pass)
            .collect();
        assert!(
            passes.len() >= 2,
            "SVG decoration passes: {passes:?}; applied={:?}; ops={:?}",
            text_data.paint.style.applied_text_decorations,
            frame
                .display_items
                .items
                .iter()
                .map(|item| (item.r#type, item.is_text_decoration))
                .collect::<Vec<_>>()
        );
        assert_eq!(passes[0], DecorationPaintPass::SvgStroke);
        assert_eq!(passes[1], DecorationPaintPass::SvgFill);
    });
}
