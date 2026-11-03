//! Text-fit scales shadows and glyphs, while text decorations stay in the
//! unscaled paint state in Chromium's TextFragmentPainter order.

#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use layoutng_assembly::{
    internal::layout_input::{FontFace, NodeKind},
    layout_engine::PaintFragmentCursor,
};
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

fn find_text(fragment: PaintFragmentCursor<'_>) -> Option<PaintFragmentCursor<'_>> {
    if fragment.PaintData().paint.source_kind == NodeKind::kText {
        return Some(fragment);
    }
    fragment.Children().into_iter().find_map(find_text)
}

#[test]
fn text_fit_scales_shadows_and_glyphs_but_not_decorations() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><div style="width:160px;white-space:pre;font:20px/40px TextFitFixture;text-fit:grow consistent;text-shadow:1px 0 red;text-decoration-line:underline line-through;text-decoration-thickness:from-font;text-decoration-skip-ink:none">ppp</div></body>"#;
        let assembly = browser::CreateLayoutAssembly();
        let mut constraints = browser::CreateBrowserConstraints(320, 100);
        constraints.fonts = vec![FontFace {
            family: "TextFitFixture".into(),
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
        page.Open("https://text-fit.test/", 16_384, 4_096).unwrap();
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
        let text_data = text.PaintData();
        assert!(
            text_data.paint.selection_text_decoration.is_none(),
            "documents without ::selection must not synthesize a selection decoration override"
        );
        assert!(
            text_data
                .paint
                .style
                .applied_text_decorations
                .iter()
                .any(|decoration| {
                    decoration.underline
                        && decoration.line_through
                        && decoration.thickness.IsFromFont()
                }),
            "combined decoration lines and from-font thickness must cross the Layout/Paint boundary"
        );
        let scale = text_data.paint.text_fit_scale;
        assert!(scale > 1.0, "expected grow text-fit scale, got {scale}");

        let items = &frame.display_items.items;
        let node_id = text_data.node_id;
        let concats: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.node_id == node_id && item.r#type == DisplayItemType::kConcat)
            .collect();
        assert_eq!(concats.len(), 2, "text-fit must enter once per glyph pass");
        for (_, concat) in &concats {
            assert!((concat.transform.values[0] - scale).abs() < 1e-9);
            assert!((concat.transform.values[5] - scale).abs() < 1e-9);
        }

        let shadow = items
            .iter()
            .position(|item| {
                item.node_id == node_id
                    && item.r#type == DisplayItemType::kDrawGlyphRun
                    && item.is_shadow
            })
            .expect("text shadow item");
        let foreground = items
            .iter()
            .position(|item| {
                item.node_id == node_id
                    && item.r#type == DisplayItemType::kDrawGlyphRun
                    && !item.is_shadow
            })
            .expect("foreground glyph item");
        let text_origin = items[foreground].text_blob_origin;
        for (_, concat) in &concats {
            assert!(
                (concat.transform.values[0] * text_origin.x + concat.transform.values[12]
                    - text_origin.x)
                    .abs()
                    < 1e-9
            );
            assert!(
                (concat.transform.values[5] * text_origin.y + concat.transform.values[13]
                    - text_origin.y)
                    .abs()
                    < 1e-9,
                "text-fit must scale around the baseline text origin"
            );
        }
        let decorations: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.node_id == node_id && item.is_text_decoration)
            .map(|(index, _)| index)
            .collect();
        assert!(!decorations.is_empty(), "text-decoration records");

        let first_concat = concats[0].0;
        let second_concat = concats[1].0;
        let first_restore = items
            .iter()
            .enumerate()
            .skip(shadow + 1)
            .find(|(_, item)| item.node_id == node_id && item.r#type == DisplayItemType::kRestore)
            .map(|(index, _)| index)
            .expect("shadow text-fit restore");
        let second_restore = items
            .iter()
            .enumerate()
            .skip(foreground + 1)
            .find(|(_, item)| item.node_id == node_id && item.r#type == DisplayItemType::kRestore)
            .map(|(index, _)| index)
            .expect("foreground text-fit restore");

        assert!(first_concat < shadow && shadow < first_restore);
        assert!(
            decorations
                .iter()
                .any(|index| first_restore < *index && *index < second_concat),
            "underline must paint before the glyph text-fit transform"
        );
        assert!(second_concat < foreground && foreground < second_restore);
        assert!(
            decorations.iter().any(|index| second_restore < *index),
            "line-through must paint after the glyph text-fit transform"
        );
    });
}
