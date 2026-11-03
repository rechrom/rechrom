// Caret queries are consumers of canonical fragment text geometry. Keep their
// recursive walks and ShapeResult lookups in a fragment-owned cost domain.

use super::*;

#[inline(never)]
fn visit_text(
    node: PaintFragmentCursor<'_>,
    parent: Offset,
    visit: &mut impl FnMut(PaintFragmentCursor<'_>, Offset),
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
        visit(node, position);
    }
    let scroll = if node.paint.establishes_paint_state {
        node.paint.scroll_offset
    } else {
        Offset::default()
    };
    let origin = Offset {
        x: position.x - scroll.x,
        y: position.y - scroll.y,
    };
    for child in node.Children() {
        visit_text(child, origin, visit);
    }
}

#[inline(never)]
pub(super) fn text_control_caret_rect(
    owner: PaintFragmentCursor<'_>,
    offset: u32,
    empty: bool,
    caret_shape: crate::internal::caret_rect::CaretShape,
) -> Option<LocalCaretRect> {
    let metrics = owner.paint.text_control_caret_metrics?;
    let shape_rect = |mut rect: LocalCaretRect, inline_size: f64, rtl: bool| {
        use crate::internal::caret_rect::CaretShape;
        let inline_size = inline_size.max(1.0);
        match caret_shape {
            CaretShape::kBar => rect,
            CaretShape::kBlock => {
                if rtl {
                    rect.offset.x -= inline_size - rect.size.width;
                }
                rect.size.width = inline_size;
                rect
            }
            CaretShape::kUnderscore => {
                if rtl {
                    rect.offset.x -= inline_size - rect.size.width;
                }
                rect.offset.y += rect.size.height - 1.0;
                rect.size.width = inline_size;
                rect.size.height = 1.0;
                rect
            }
        }
    };
    let mut candidate: Option<(u32, LocalCaretRect)> = None;
    if !empty {
        for child in owner.Children() {
            visit_text(
                child,
                Offset {
                    x: -owner.paint.scroll_offset.x,
                    y: -owner.paint.scroll_offset.y,
                },
                &mut |node, position| {
                    let start = node.text_start.unwrap_or(0);
                    let end = node.text_end.unwrap_or(start);
                    if start <= offset {
                        if let Some(x) = node.TextInlinePosition(offset.min(end)) {
                            let top = position.y + node.paint.text_line_top_offset.unwrap_or(0.0);
                            let rect = shape_rect(
                                LocalCaretRect {
                                    offset: Offset {
                                        x: position.x + x
                                            - if caret_shape
                                                == crate::internal::caret_rect::CaretShape::kBar
                                            {
                                                0.5
                                            } else {
                                                0.0
                                            },
                                        y: position.y
                                            + node
                                                .TextPaintBaseline()
                                                .map_or(0.0, |baseline| baseline - metrics.ascent)
                                            + top.round()
                                            - top,
                                    },
                                    size: Size {
                                        width: 1.0,
                                        height: metrics.font_height,
                                    },
                                },
                                node.TextNextCharacterWidth(offset.min(end))
                                    .unwrap_or(metrics.zero_width),
                                node.paint.direction == TextDirection::kRtl,
                            );
                            if candidate.is_none_or(|(old, _)| start >= old) {
                                candidate = Some((start, rect));
                            }
                        }
                    }
                },
            );
        }
    }
    if let Some((_, rect)) = candidate {
        return Some(rect);
    }

    #[inline(never)]
    fn editor(node: PaintFragmentCursor<'_>, parent: Offset) -> Option<LocalCaretRect> {
        let position = Offset {
            x: parent.x + node.offset.x,
            y: parent.y + node.offset.y,
        };
        if let Some(rect) = node.paint.text_control_empty_caret {
            return Some(LocalCaretRect {
                offset: Offset {
                    x: position.x + rect.offset.x,
                    y: position.y + rect.offset.y,
                },
                size: rect.size,
            });
        }
        for child in node.Children() {
            if let Some(rect) = editor(child, position) {
                return Some(rect);
            }
        }
        None
    }

    for child in owner.Children() {
        if let Some(rect) = editor(child, Offset::default()) {
            return Some(shape_rect(
                rect,
                metrics.zero_width,
                owner.paint.direction == TextDirection::kRtl,
            ));
        }
    }
    let border = owner.paint.border;
    let padding = owner.paint.padding;
    Some(shape_rect(
        LocalCaretRect {
            offset: Offset {
                x: if owner.paint.direction == TextDirection::kRtl {
                    owner.size.width - border.right - padding.right - 1.0
                } else {
                    border.left + padding.left
                },
                y: border.top + padding.top + (metrics.line_height - metrics.font_height) * 0.5,
            },
            size: Size {
                width: 1.0,
                height: metrics.font_height,
            },
        },
        metrics.zero_width,
        owner.paint.direction == TextDirection::kRtl,
    ))
}

#[inline(never)]
pub(super) fn text_control_offset_for_point(
    owner: PaintFragmentCursor<'_>,
    point: Offset,
    empty: bool,
) -> u32 {
    if empty {
        return 0;
    }
    let mut best: Option<(f64, f64, u32)> = None;
    for child in owner.Children() {
        visit_text(
            child,
            Offset {
                x: -owner.paint.scroll_offset.x,
                y: -owner.paint.scroll_offset.y,
            },
            &mut |node, position| {
                let dy = (position.y - point.y)
                    .max(0.0)
                    .max(point.y - position.y - node.size.height);
                let local = Offset {
                    x: point.x - position.x,
                    y: point.y - position.y,
                };
                if let Some(offset) = node.TextOffsetForPoint(local) {
                    let x = node.TextInlinePosition(offset).unwrap_or(local.x);
                    let score = (dy, (position.x + x - point.x).abs(), offset);
                    if best.is_none_or(|old| score < old) {
                        best = Some(score);
                    }
                }
            },
        );
    }
    best.map_or(0, |(_, _, offset)| offset)
}
