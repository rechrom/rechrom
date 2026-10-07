//! Caret geometry derived from the current exported layout fragments and shaped runs.
//! This adapter covers bar carets in text controls; editing owns the selection.
use crate::fragment_tree::{FragmentKind, FragmentNode, PaintGlyphRun};
use crate::internal::layout_input::{Offset, Size, TextDirection};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalCaretRect {
    pub offset: Offset,
    pub size: Size,
}

fn inline_position(runs: &[PaintGlyphRun], offset: u32) -> Option<f64> {
    let mut before: Option<(u32, f64)> = None;
    let mut after: Option<(u32, f64)> = None;
    for run in runs {
        for glyph in &run.glyphs {
            if glyph.character_index < offset {
                if before.is_none_or(|(i, _)| glyph.character_index >= i) {
                    before = Some((
                        glyph.character_index,
                        if run.rtl {
                            glyph.offset.x
                        } else {
                            glyph.offset.x + glyph.advance
                        },
                    ));
                }
            } else if after.is_none_or(|(i, _)| glyph.character_index < i) {
                after = Some((
                    glyph.character_index,
                    if run.rtl {
                        glyph.offset.x + glyph.advance
                    } else {
                        glyph.offset.x
                    },
                ));
            }
        }
    }
    after.or(before).map(|(_, x)| x)
}

/// The offset is in the UTF-16 coordinate space used by the shaped layout runs.
/// The result is local to the owning control; paint applies its property state.
#[allow(non_snake_case)]
pub fn TextControlCaretRect(
    owner: &FragmentNode,
    offset: u32,
    empty: bool,
) -> Option<LocalCaretRect> {
    let metrics = owner.paint.text_control_caret_metrics?;
    let border = owner.paint.border;
    let padding = owner.paint.padding;
    let origin = Offset {
        x: border.left + padding.left,
        y: border.top + padding.top,
    };
    let width = (owner.size.width - origin.x - border.right - padding.right).max(0.0);
    let height = (owner.size.height - origin.y - border.bottom - padding.bottom).max(0.0);
    if width <= 0.0 || height <= 0.0 || metrics.font_height <= 0.0 {
        return None;
    }
    let caret_height = metrics.font_height.min(height);
    let mut candidate: Option<(u32, LocalCaretRect)> = None;
    fn visit(
        node: &FragmentNode,
        parent: Offset,
        offset: u32,
        font_height: f64,
        ascent: f64,
        candidate: &mut Option<(u32, LocalCaretRect)>,
    ) {
        if node.paint.hidden {
            return;
        }
        let position = Offset {
            x: parent.x + node.offset.x,
            y: parent.y + node.offset.y,
        };
        if matches!(
            node.kind,
            FragmentKind::kText | FragmentKind::kGeneratedText
        ) {
            let start = node.text_start.unwrap_or(0);
            let end = node.text_end.unwrap_or(start);
            if start <= offset {
                if let (Some(x), Some(run)) = (
                    inline_position(&node.paint.glyph_runs, offset.min(end)),
                    node.paint.glyph_runs.first(),
                ) {
                    let snap = node.paint.text_line_top_offset.map_or(0.0, |top| {
                        let top = position.y + top;
                        top.round() - top
                    });
                    let rect = LocalCaretRect {
                        offset: Offset {
                            x: position.x + x,
                            y: position.y + run.baseline - ascent + snap,
                        },
                        size: Size {
                            width: 1.0,
                            height: font_height,
                        },
                    };
                    if candidate.is_none_or(|(old, _)| start >= old) {
                        *candidate = Some((start, rect));
                    }
                }
            }
        }
        let mut child_origin = position;
        if node.paint.establishes_paint_state {
            child_origin.x -= node.paint.scroll_offset.x;
            child_origin.y -= node.paint.scroll_offset.y;
        }
        for child in &node.children {
            visit(child, child_origin, offset, font_height, ascent, candidate);
        }
    }
    if !empty {
        let parent = Offset {
            x: -owner.paint.scroll_offset.x,
            y: -owner.paint.scroll_offset.y,
        };
        for child in &owner.children {
            visit(
                child,
                parent,
                offset,
                caret_height,
                metrics.ascent,
                &mut candidate,
            );
        }
    }
    Some(candidate.map_or(
        LocalCaretRect {
            offset: Offset {
                x: origin.x
                    + if owner.paint.direction == TextDirection::kRtl {
                        width - 1.0
                    } else {
                        0.0
                    },
                y: origin.y + (metrics.line_height.min(height) - caret_height).max(0.0) * 0.5,
            },
            size: Size {
                width: 1.0,
                height: caret_height,
            },
        },
        |(_, rect)| rect,
    ))
}
