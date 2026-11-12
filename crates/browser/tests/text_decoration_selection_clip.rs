//! Originating text decorations clip out selection while selection-phase
//! decorations retain the Chromium under-text / glyph / line-through order.

#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use interaction::input_event::{InputEvent, MouseButton, MouseEvent, MouseEventType, Offset};
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

fn collect_text<'a>(fragment: PaintFragmentCursor<'a>, output: &mut Vec<PaintFragmentCursor<'a>>) {
    if fragment
        .Cursor()
        .InlineItem()
        .is_some_and(|item| item.IsText())
    {
        output.push(fragment);
    }
    for child in fragment.Children() {
        collect_text(child, output);
    }
}

fn load_page(html: &[u8]) -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(360, 100);
    constraints.fonts = vec![FontFace {
        family: "SelectionDecorationFixture".into(),
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
    page.Open("https://selection-clip.test/", 16_384, 4_096)
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

fn dispatch_selection(
    page: &mut Page,
    node_id: u64,
    initial_sequence: u64,
    start: Offset,
    end: Offset,
) {
    for event in [
        MouseEvent {
            r#type: MouseEventType::kDown,
            position: start,
            button: MouseButton::kPrimary,
            ..Default::default()
        },
        MouseEvent {
            r#type: MouseEventType::kMove,
            position: end,
            button: MouseButton::kPrimary,
            ..Default::default()
        },
        MouseEvent {
            r#type: MouseEventType::kUp,
            position: end,
            button: MouseButton::kPrimary,
            ..Default::default()
        },
    ] {
        page.Dispatch(&InputEvent::Mouse(event)).unwrap();
    }
    for _ in 0..100 {
        page.RunTask().unwrap();
        if page.CurrentFrame().is_some_and(|frame| {
            frame.sequence > initial_sequence
                && frame.display_items.items.iter().any(|item| {
                    item.node_id == node_id && item.r#type == DisplayItemType::kClipRect
                })
        }) {
            break;
        }
    }
    assert!(page
        .CurrentFrame()
        .is_some_and(|frame| frame.sequence > initial_sequence));
}

fn load_and_select_range(
    html: &[u8],
    start_fraction: f64,
    end_fraction: f64,
    vertical: bool,
) -> (Page, u64) {
    let mut page = load_page(html);
    let frame = page.CurrentFrame().expect("initial frame");
    let initial_sequence = frame.sequence;
    let text = find_text(PaintFragmentCursor::FromHandle(&frame.fragments)).expect("text fragment");
    let node_id = text.PaintData().node_id;
    let glyph_rect = frame
        .display_items
        .items
        .iter()
        .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawGlyphRun)
        .expect("glyph item")
        .rect;
    let center_x = glyph_rect.x + glyph_rect.width / 2.0;
    let center_y = glyph_rect.y + glyph_rect.height / 2.0;
    let selection_position = |fraction: f64| Offset {
        x: if vertical {
            center_x
        } else {
            glyph_rect.x + glyph_rect.width * fraction
        },
        y: if vertical {
            glyph_rect.y + glyph_rect.height * fraction
        } else {
            center_y
        },
    };
    dispatch_selection(
        &mut page,
        node_id,
        initial_sequence,
        selection_position(start_fraction),
        selection_position(end_fraction),
    );
    (page, node_id)
}

fn load_and_select_across_lines(html: &[u8]) -> (Page, u64) {
    let mut page = load_page(html);
    let frame = page.CurrentFrame().expect("initial frame");
    let initial_sequence = frame.sequence;
    let mut glyph_rects: Vec<_> = frame
        .display_items
        .items
        .iter()
        .filter(|item| {
            item.r#type == DisplayItemType::kDrawGlyphRun && !item.is_shadow && !item.stroke_glyphs
        })
        .map(|item| (item.node_id, item.rect))
        .collect();
    glyph_rects.sort_by(|a, b| {
        a.1.y
            .total_cmp(&b.1.y)
            .then_with(|| a.1.x.total_cmp(&b.1.x))
    });
    glyph_rects.dedup();
    assert!(glyph_rects.len() >= 2, "fixture must wrap onto two lines");
    let (node_id, first) = glyph_rects[0];
    let (_, second) = glyph_rects[1];
    dispatch_selection(
        &mut page,
        node_id,
        initial_sequence,
        Offset {
            x: first.x,
            y: first.y + first.height / 2.0,
        },
        Offset {
            x: second.x + second.width / 2.0,
            y: second.y + second.height / 2.0,
        },
    );
    (page, node_id)
}

#[test]
fn selection_splits_originating_and_selection_decoration_passes() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><style>::selection{color:#123456;background-color:#abcdef;text-decoration-color:lime;text-shadow:4px 5px 0 magenta}</style><body style="margin:0"><span style="font:24px/32px SelectionDecorationFixture;text-decoration-line:underline line-through;text-decoration-color:red;text-decoration-thickness:3px;text-decoration-skip-ink:none;text-shadow:1px 2px 0 orange">selection</span></body>"#;
        let assembly = browser::CreateLayoutAssembly();
        let mut constraints = browser::CreateBrowserConstraints(360, 100);
        constraints.fonts = vec![FontFace {
            family: "SelectionDecorationFixture".into(),
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
        page.Open("https://selection-decoration.test/", 16_384, 4_096)
            .unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if !page.IsLoading() {
                break;
            }
        }
        assert!(!page.IsLoading());

        let frame = page.CurrentFrame().expect("initial frame");
        let initial_sequence = frame.sequence;
        let text =
            find_text(PaintFragmentCursor::FromHandle(&frame.fragments)).expect("text fragment");
        let data = text.PaintData();
        let node_id = data.node_id;
        let ink_overflow_height = data
            .paint
            .ink_overflow
            .expect("text fragment ink overflow")
            .size
            .height;
        let exported_selection_decoration = data.paint.selection_text_decoration.clone();
        let glyph_rect = frame
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawGlyphRun)
            .expect("glyph item")
            .rect;
        let y = glyph_rect.y + glyph_rect.height / 2.0;
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kDown,
            position: Offset {
                x: glyph_rect.x + 2.0,
                y,
            },
            button: MouseButton::kPrimary,
            ..Default::default()
        }))
        .unwrap();
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kMove,
            position: Offset {
                x: glyph_rect.x + glyph_rect.width - 2.0,
                y,
            },
            button: MouseButton::kPrimary,
            ..Default::default()
        }))
        .unwrap();
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kUp,
            position: Offset {
                x: glyph_rect.x + glyph_rect.width - 2.0,
                y,
            },
            button: MouseButton::kPrimary,
            ..Default::default()
        }))
        .unwrap();
        let document_selection = page
            .GetLayoutEngine()
            .GetLayoutTree()
            .expect("layout tree")
            .EditingState()
            .selections
            .Document();
        assert!(
            document_selection.is_some_and(|selection| !selection.Collapsed()),
            "mouse drag must create a non-collapsed document selection; glyph rect: {glyph_rect:?}, state: {document_selection:?}"
        );
        for _ in 0..100 {
            page.RunTask().unwrap();
            if page.CurrentFrame().is_some_and(|frame| {
                frame.sequence > initial_sequence
                    && frame.display_items.items.iter().any(|item| {
                        item.node_id == node_id && item.r#type == DisplayItemType::kClipOutRect
                    })
            }) {
                break;
            }
        }

        let frame = page.CurrentFrame().expect("selection frame");
        assert!(frame.sequence > initial_sequence);
        let items = &frame.display_items.items;
        let clip_outs: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.node_id == node_id && item.r#type == DisplayItemType::kClipOutRect
            })
            .map(|(index, _)| index)
            .collect();
        let node_item_types: Vec<_> = items
            .iter()
            .filter(|item| item.node_id == node_id)
            .map(|item| item.r#type)
            .collect();
        assert_eq!(
            clip_outs.len(),
            2,
            "originating underline and line-through each clip out selection; node items: {node_item_types:?}"
        );

        let matching_restore = |start: usize| {
            let mut nested_saves = 0usize;
            for (index, item) in items.iter().enumerate().skip(start + 1) {
                if matches!(
                    item.r#type,
                    DisplayItemType::kSave
                        | DisplayItemType::kSaveLayer
                        | DisplayItemType::kSaveLayerAlpha
                        | DisplayItemType::kSaveLayerBlend
                        | DisplayItemType::kSaveLayerFilter
                        | DisplayItemType::kSaveLayerDstIn
                ) {
                    nested_saves += 1;
                } else if item.r#type == DisplayItemType::kRestore {
                    if nested_saves == 0 {
                        return index;
                    }
                    nested_saves -= 1;
                }
            }
            panic!("selection clip restore");
        };
        let selection_scopes: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.node_id == node_id && item.r#type == DisplayItemType::kClipRect
            })
            .map(|(clip, item)| {
                let restore = matching_restore(clip);
                let scope = &items[clip + 1..restore];
                (
                    clip,
                    item.rect,
                    scope.iter().any(|item| {
                        item.r#type == DisplayItemType::kDrawGlyphRun && !item.is_shadow
                    }),
                    scope.iter().any(|item| item.is_text_decoration),
                )
            })
            .collect();
        assert_eq!(selection_scopes.len(), 4);
        let selected_glyph = selection_scopes
            .iter()
            .find(|(_, _, has_glyph, _)| *has_glyph)
            .map(|(clip, _, _, _)| *clip)
            .expect("selected glyph clip");
        let selected_decorations: Vec<_> = selection_scopes
            .iter()
            .filter(|(_, _, _, has_decoration)| *has_decoration)
            .collect();
        assert_eq!(selected_decorations.len(), 2);
        assert!(selected_decorations
            .iter()
            .all(|(_, rect, _, _)| { (rect.height - ink_overflow_height).abs() <= f64::EPSILON }));
        let selected_decoration_colors: Vec<_> = selected_decorations
            .iter()
            .map(|(clip, _, _, _)| {
                let restore = matching_restore(*clip);
                items[*clip + 1..restore]
                    .iter()
                    .rev()
                    .find(|item| item.is_text_decoration && !item.is_shadow)
                    .expect("selected decoration in clip scope")
                    .color
            })
            .collect();
        assert!(
            selected_decoration_colors.iter().all(|color| {
                color.red == 0.0
                    && color.green == 1.0
                    && color.blue == 0.0
                    && color.alpha == 1.0
            }),
            "selected colors: {selected_decoration_colors:?}; exported selection decoration: {exported_selection_decoration:?}"
        );
        assert!(items.iter().any(|item| {
            item.node_id == node_id
                && item.is_text_decoration
                && item.color.red == 1.0
                && item.color.green == 0.0
                && item.color.blue == 0.0
                && item.color.alpha == 1.0
        }));
        let close = |actual: f32, expected: f32| (actual - expected).abs() < 1e-5;
        let selection_background = items
            .iter()
            .enumerate()
            .find(|(_, item)| {
                item.node_id == node_id
                    && item.r#type == DisplayItemType::kDrawRect
                    && close(item.color.red, 171.0 / 255.0)
                    && close(item.color.green, 205.0 / 255.0)
                    && close(item.color.blue, 239.0 / 255.0)
                    && close(item.color.alpha, 1.0)
            })
            .map(|(index, _)| index)
            .expect("selection background must come from ::selection, not a Paint constant");
        let selected_glyph_restore = items
            .iter()
            .enumerate()
            .skip(selected_glyph + 1)
            .find(|(_, item)| item.node_id == node_id && item.r#type == DisplayItemType::kRestore)
            .map(|(index, _)| index)
            .expect("selected glyph restore");
        let selected_glyph_items = &items[selected_glyph + 1..selected_glyph_restore];
        assert!(
            selected_glyph_items.iter().any(|item| {
                item.r#type == DisplayItemType::kDrawGlyphRun
                    && !item.is_shadow
                    && close(item.color.red, 18.0 / 255.0)
                    && close(item.color.green, 52.0 / 255.0)
                    && close(item.color.blue, 86.0 / 255.0)
            }),
            "selected glyph foreground must come from ::selection"
        );
        let selection_shadow = items
            .iter()
            .enumerate()
            .find(|(_, item)| {
                item.r#type == DisplayItemType::kDrawGlyphRun
                    && item.is_shadow
                    && close(item.color.red, 1.0)
                    && close(item.color.green, 0.0)
                    && close(item.color.blue, 1.0)
                    && item.shadow_offset.x == 4.0
                    && item.shadow_offset.y == 5.0
            })
            .map(|(index, _)| index)
            .expect("selected glyph shadow must replace the originating text-shadow");
        let originating_shadow = items
            .iter()
            .enumerate()
            .find(|(_, item)| {
                item.r#type == DisplayItemType::kDrawGlyphRun
                    && item.is_shadow
                    && close(item.color.red, 1.0)
                    && close(item.color.green, 165.0 / 255.0)
                    && close(item.color.blue, 0.0)
                    && item.shadow_offset.x == 1.0
                    && item.shadow_offset.y == 2.0
            })
            .map(|(index, _)| index)
            .expect("originating text shadow");
        let selection_shadow_clip = selection_scopes
            .iter()
            .find(|(clip, _, _, _)| {
                let restore = matching_restore(*clip);
                *clip < selection_shadow && selection_shadow < restore
            })
            .map(|(_, rect, _, _)| *rect)
            .expect("selection shadow clip");
        let selection_background_rect = items[selection_background].rect;
        assert_eq!(selection_shadow_clip.x, selection_background_rect.x);
        assert_eq!(selection_shadow_clip.y, selection_background_rect.y);
        assert!((selection_shadow_clip.width - selection_background_rect.width - 4.0).abs() < 1.0);
        assert!(
            (selection_shadow_clip.height - selection_background_rect.height - 5.0).abs() < 1.0
        );
        assert!(originating_shadow < selection_background);
        assert!(selection_background < selection_shadow);
        assert!(selection_shadow < selected_glyph);
        assert!(clip_outs.iter().all(|clip| originating_shadow < *clip));
        assert!(selected_decorations
            .iter()
            .all(|(clip, _, _, _)| selection_shadow < *clip));
        assert!(
            selected_decorations.iter().all(|(clip, _, _, _)| {
                items[*clip + 1..matching_restore(*clip)]
                    .iter()
                    .any(|item| {
                        item.r#type == DisplayItemType::kSaveLayerFilter
                            && item.text_shadow_filters.iter().any(|shadow| {
                                close(shadow.color.red, 1.0)
                                    && close(shadow.color.green, 0.0)
                                    && close(shadow.color.blue, 1.0)
                                    && shadow.offset.x == 4.0
                                    && shadow.offset.y == 5.0
                            })
                    })
            }),
            "selection decorations must use the selection layer shadow"
        );
        assert!(
            selected_decorations
                .iter()
                .any(|(index, _, _, _)| *index < selected_glyph),
            "selected underline must precede selected glyphs"
        );
        assert!(
            selected_decorations
                .iter()
                .any(|(index, _, _, _)| *index > selected_glyph),
            "selected line-through must follow selected glyphs"
        );
    });
}

#[test]
fn selection_text_clip_inflates_for_originating_stroke() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><span style="font:24px/32px SelectionDecorationFixture;-webkit-text-stroke:6px blue">stroke</span></body>"#;
        let (page, node_id) = load_and_select_range(html, 0.2, 0.8, false);
        let frame = page.CurrentFrame().expect("selection frame");
        let items = &frame.display_items.items;
        let background = items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawRect)
            .expect("default selection background")
            .rect;
        let (clip_index, clip_rect) = items
            .iter()
            .enumerate()
            .find_map(|(clip_index, item)| {
                if item.node_id != node_id || item.r#type != DisplayItemType::kClipRect {
                    return None;
                }
                let restore = items
                    .iter()
                    .enumerate()
                    .skip(clip_index + 1)
                    .find(|(_, item)| {
                        item.node_id == node_id && item.r#type == DisplayItemType::kRestore
                    })
                    .map(|(index, _)| index)?;
                items[clip_index + 1..restore]
                    .iter()
                    .any(|item| item.stroke_glyphs)
                    .then_some((clip_index, item.rect))
            })
            .expect("selected stroked glyph clip");
        let expected_left = (background.x - 3.0).floor();
        let expected_right = (background.x + background.width + 3.0).ceil();
        assert_eq!(clip_rect.x, expected_left);
        assert_eq!(clip_rect.width, expected_right - expected_left);
        let restore = items
            .iter()
            .enumerate()
            .skip(clip_index + 1)
            .find(|(_, item)| item.node_id == node_id && item.r#type == DisplayItemType::kRestore)
            .map(|(index, _)| index)
            .expect("selected glyph restore");
        assert!(items[clip_index + 1..restore].iter().any(|item| {
            item.stroke_glyphs
                && item.stroke_width == 6.0
                && item.color.red == 0.0
                && item.color.green == 0.0
                && item.color.blue == 1.0
        }));
    });
}

#[test]
fn full_selection_text_clip_uses_fragment_ink_edges() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><span style="font:italic 32px/40px SelectionDecorationFixture">italic</span></body>"#;
        let (page, node_id) = load_and_select_range(html, 0.0, 1.0, false);
        let frame = page.CurrentFrame().expect("selection frame");
        let text =
            find_text(PaintFragmentCursor::FromHandle(&frame.fragments)).expect("text fragment");
        let ink = text
            .PaintData()
            .paint
            .ink_overflow
            .expect("native text ink overflow");
        let items = &frame.display_items.items;
        let background = items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawRect)
            .expect("default selection background")
            .rect;
        let clip = items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kClipRect)
            .expect("selected glyph clip")
            .rect;
        let adjusted_x = background.x.floor() + ink.offset.x;
        let expected_left = adjusted_x.floor();
        let expected_right = (adjusted_x + ink.offset.x + ink.size.width).ceil();
        assert_eq!(clip.x, expected_left);
        assert_eq!(clip.width, expected_right - expected_left);
        assert!(
            ink.offset.x != 0.0 || ink.size.width != background.width,
            "fixture must exercise an ink edge distinct from the selection range"
        );
    });
}

#[test]
fn svg_selection_text_proper_does_not_use_canvas_clip() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><style>::selection{color:red;background:transparent}</style><body style="margin:0"><svg width="240" height="80"><text x="10" y="45" style="font:32px SelectionDecorationFixture">svgtext</text></svg></body>"#;
        let (page, node_id) = load_and_select_range(html, 0.15, 0.75, false);
        let frame = page.CurrentFrame().expect("selection frame");
        let items = &frame.display_items.items;
        assert!(items.iter().any(|item| {
            item.node_id == node_id
                && item.r#type == DisplayItemType::kDrawGlyphRun
                && !item.is_shadow
                && item.color.red == 1.0
                && item.color.green == 0.0
                && item.color.blue == 0.0
        }));
        assert!(!items
            .iter()
            .any(|item| { item.node_id == node_id && item.r#type == DisplayItemType::kClipRect }));
    });
}

#[test]
fn text_fit_selection_clip_uses_unscaled_font_block_geometry() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><div style="width:160px;white-space:pre;font:20px/40px SelectionDecorationFixture;text-fit:grow consistent">ppp</div></body>"#;
        let (page, node_id) = load_and_select_range(html, 0.2, 0.8, false);
        let frame = page.CurrentFrame().expect("selection frame");
        let text =
            find_text(PaintFragmentCursor::FromHandle(&frame.fragments)).expect("text fragment");
        let item = text.Cursor().InlineItem().expect("native inline text item");
        let used_font = item.GetUsedFont();
        let scale = f64::from(used_font.ScalingFactor());
        assert!(scale > 1.0, "text-fit grow scale: {scale}");
        let primary = used_font.PrimaryFont();
        assert!(!primary.is_null());
        let unscaled_ascent = f64::from(unsafe { &*primary }.GetFontMetrics().FloatAscent());
        let ink = item.InkOverflowRect();
        let items = &frame.display_items.items;
        let background = items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawRect)
            .expect("default selection background")
            .rect;
        let clip = items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kClipRect)
            .expect("selected glyph clip")
            .rect;
        let line_top = background.y.floor();
        let line_height = (background.y + ink.Height().ToDouble()).ceil() - line_top;
        let expected_y = line_top + ink.Y().ToDouble() / scale - unscaled_ascent * (1.0 - scale);
        assert!(
            (clip.y - expected_y).abs() <= 1.0 / 64.0,
            "clip={clip:?}, background={background:?}, ink={ink:?}, scale={scale}, ascent={unscaled_ascent}, expected_y={expected_y}"
        );
        assert!((clip.height - line_height / scale).abs() <= 1.0 / 64.0);
    });
}

#[test]
fn selection_background_expands_to_containing_line_height() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><div style="font:32px/60px SelectionDecorationFixture"><span style="font-size:12px;line-height:12px;vertical-align:super">small</span></div></body>"#;
        let (page, node_id) = load_and_select_range(html, 0.1, 0.9, false);
        let frame = page.CurrentFrame().expect("selection frame");
        let text =
            find_text(PaintFragmentCursor::FromHandle(&frame.fragments)).expect("text fragment");
        let data = text.PaintData();
        let line_height = data.paint.text_line_height.expect("containing line height");
        assert!(line_height > data.size.height);
        let background = frame
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawRect)
            .expect("selection background")
            .rect;
        assert!(background.height >= line_height);
    });
}

#[test]
fn vertical_selection_background_expands_to_containing_line_block_size() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><div style="writing-mode:vertical-rl;height:90px;font:32px/60px SelectionDecorationFixture"><span style="font-size:12px;line-height:12px">small</span></div></body>"#;
        let (page, node_id) = load_and_select_range(html, 0.1, 0.9, true);
        let frame = page.CurrentFrame().expect("selection frame");
        let text =
            find_text(PaintFragmentCursor::FromHandle(&frame.fragments)).expect("text fragment");
        let data = text.PaintData();
        assert!(60.0 > data.size.width);
        let item = text.Cursor().InlineItem().expect("native text item");
        let ink_block_size = item.InkOverflowRect().Height().ToDouble();
        let background = frame
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawRect)
            .expect("selection background")
            .rect;
        assert!(background.width >= 60.0);
        let glyph_clip = frame
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kClipRect)
            .expect("selected glyph clip")
            .rect;
        assert_eq!(glyph_clip.width, ink_block_size.ceil());
        assert!(
            glyph_clip.width < background.width,
            "glyph clip must use line-relative ink block geometry, not the expanded physical selection background: clip={glyph_clip:?}, background={background:?}"
        );
    });
}

#[test]
fn selection_extends_across_a_selected_soft_line_break() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><div style="width:70px;font:20px/28px SelectionDecorationFixture">alpha beta</div></body>"#;
        let (page, node_id) = load_and_select_across_lines(html);
        let frame = page.CurrentFrame().expect("selection frame");
        let mut texts = Vec::new();
        collect_text(
            PaintFragmentCursor::FromHandle(&frame.fragments),
            &mut texts,
        );
        let first = texts
            .into_iter()
            .filter(|text| text.PaintData().node_id == node_id)
            .min_by(|a, b| a.Cursor().Offset().top.cmp(&b.Cursor().Offset().top))
            .expect("first wrapped text fragment");
        let start = first.text_start.expect("text start");
        let end = first.text_end.expect("text end");
        let base = first.TextRangeRect(start, end).expect("base range rect");
        let expanded = first
            .TextSelectionRangeRect(start, end, true)
            .expect("soft-break selection rect");
        assert!(expanded.width > base.width);
        let background = frame
            .display_items
            .items
            .iter()
            .filter(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawRect)
            .min_by(|a, b| a.rect.y.total_cmp(&b.rect.y))
            .expect("first-line selection background")
            .rect;
        assert!(background.width >= expanded.width - 1e-5);
    });
}

#[test]
fn selection_uses_space_width_for_a_selected_hard_line_break() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0;font:20px/28px SelectionDecorationFixture"><span>alpha<br>beta</span></body>"#;
        let (page, _) = load_and_select_across_lines(html);
        let frame = page.CurrentFrame().expect("selection frame");
        let mut texts = Vec::new();
        collect_text(
            PaintFragmentCursor::FromHandle(&frame.fragments),
            &mut texts,
        );
        let line_break = texts
            .into_iter()
            .find(|text| {
                text.Cursor()
                    .InlineItem()
                    .is_some_and(|item| item.IsLineBreak())
            })
            .expect("forced line-break fragment");
        let start = line_break.text_start.expect("line-break start");
        let end = line_break.text_end.expect("line-break end");
        let rect = line_break
            .TextSelectionRangeRect(start, end, false)
            .expect("hard-break selection rect");
        assert!(rect.width > 0.0);
        let background = frame
            .display_items
            .items
            .iter()
            .find(|item| {
                item.node_id == line_break.PaintData().node_id
                    && item.r#type == DisplayItemType::kDrawRect
            })
            .expect("hard-break selection background")
            .rect;
        let snapped_width = (rect.x + rect.width).round() - rect.x.round();
        assert!(
            (background.width - snapped_width).abs() < 1e-5,
            "hard-break background {background:?} differs from pixel-snapped range {rect:?}"
        );
    });
}

#[test]
fn transparent_selection_background_does_not_record_draw_rect() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><style>::selection{color:red;background-color:transparent}</style><body style="margin:0"><span style="font:24px/32px SelectionDecorationFixture">transparent</span></body>"#;
        let (page, node_id) = load_and_select_range(html, 0.05, 0.45, false);
        let frame = page.CurrentFrame().expect("selection frame");
        let items = &frame.display_items.items;
        let selected_glyph_count = items
            .iter()
            .find(|item| {
                item.node_id == node_id
                    && item.r#type == DisplayItemType::kDrawGlyphRun
                    && !item.is_shadow
                    && !item.stroke_glyphs
                    && item.color.red == 1.0
                    && item.color.green == 0.0
                    && item.color.blue == 0.0
            })
            .expect("partially selected red glyph run")
            .glyphs
            .len();
        let originating_glyph_count = items
            .iter()
            .find(|item| {
                item.node_id == node_id
                    && item.r#type == DisplayItemType::kDrawGlyphRun
                    && !item.is_shadow
                    && !item.stroke_glyphs
                    && item.color.red == 0.0
                    && item.color.green == 0.0
                    && item.color.blue == 0.0
            })
            .expect("originating glyph run")
            .glyphs
            .len();
        assert!(selected_glyph_count > 0);
        assert!(selected_glyph_count < originating_glyph_count);
        assert!(!items.iter().any(|item| {
            item.node_id == node_id
                && item.r#type == DisplayItemType::kDrawRect
                && item.color.alpha <= 0.0
        }));
    });
}

#[test]
fn visible_selection_handles_publish_compositor_bounds() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let html = br#"<!doctype html><body style="margin:0"><span style="font:24px/32px SelectionDecorationFixture">selection bounds</span></body>"#;
        let mut page = load_page(html);
        let frame = page.CurrentFrame().expect("initial frame");
        let initial_sequence = frame.sequence;
        let text =
            find_text(PaintFragmentCursor::FromHandle(&frame.fragments)).expect("text fragment");
        let node_id = text.PaintData().node_id;
        let endpoint = text.text_start.expect("text start");
        let endpoint_rect = text
            .TextSelectionRangeRect(endpoint, endpoint, false)
            .expect("selection endpoint rect");
        assert_eq!(endpoint_rect.width, 0.0);
        assert!(endpoint_rect.height > 0.0);
        let glyph_rect = frame
            .display_items
            .items
            .iter()
            .find(|item| item.node_id == node_id && item.r#type == DisplayItemType::kDrawGlyphRun)
            .expect("glyph item")
            .rect;
        let y = glyph_rect.y + glyph_rect.height / 2.0;
        let start = Offset {
            x: glyph_rect.x + 2.0,
            y,
        };
        let end = Offset {
            x: glyph_rect.x + glyph_rect.width - 2.0,
            y,
        };
        for event in [
            MouseEvent {
                r#type: MouseEventType::kDown,
                position: start,
                button: MouseButton::kPrimary,
                ..Default::default()
            },
            MouseEvent {
                r#type: MouseEventType::kMove,
                position: end,
                button: MouseButton::kPrimary,
                ..Default::default()
            },
        ] {
            page.Dispatch(&InputEvent::Mouse(event)).unwrap();
        }
        // SelectionBoundsRecorder is active only while touch-style selection
        // handles are visible. This retained-state transition invalidates paint.
        page.GetLayoutEngine()
            .GetLayoutTree()
            .expect("layout tree")
            .EditingState()
            .selections
            .SetSelectionHandlesVisible(true);
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kUp,
            position: end,
            button: MouseButton::kPrimary,
            ..Default::default()
        }))
        .unwrap();

        for _ in 0..100 {
            page.RunTask().unwrap();
            if page.CurrentFrame().is_some_and(|frame| {
                frame.sequence > initial_sequence
                    && frame.display_items.chunks.iter().any(|chunk| {
                        chunk
                            .compositor
                            .as_deref()
                            .and_then(|metadata| metadata.layer_selection_data.as_deref())
                            .is_some_and(|selection| {
                                selection.any_selection_was_painted
                                    && selection.start.is_some()
                                    && selection.end.is_some()
                            })
                    })
            }) {
                break;
            }
        }

        let document_selection = page
            .GetLayoutEngine()
            .GetLayoutTree()
            .expect("layout tree")
            .EditingState()
            .selections
            .Document();
        assert!(document_selection.is_some_and(|selection| !selection.Collapsed()));
        let frame = page.CurrentFrame().expect("selection frame");
        assert!(frame.sequence > initial_sequence);
        let layer_selections: Vec<_> = frame
            .display_items
            .chunks
            .iter()
            .filter_map(|chunk| chunk.compositor.as_deref())
            .filter_map(|metadata| metadata.layer_selection_data.as_deref())
            .collect();
        assert!(
            layer_selections
                .iter()
                .any(|selection| selection.any_selection_was_painted),
            "selection paint must mark its chunk for compositor selection metadata"
        );
        assert!(
            layer_selections
                .iter()
                .any(|selection| selection.start.is_some()),
            "visible selection handles must publish a start bound"
        );
        assert!(
            layer_selections
                .iter()
                .any(|selection| selection.end.is_some()),
            "visible selection handles must publish an end bound"
        );
    });
}
