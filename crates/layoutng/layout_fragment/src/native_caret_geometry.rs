// Caret queries are consumers of canonical fragment text geometry. Keep their
// recursive walks and ShapeResult lookups in a fragment-owned cost domain.

use super::*;

#[inline(never)]
fn visit_text(
    node: NativePaintFragmentCursor<'_>,
    parent: Offset,
    visit: &mut impl FnMut(NativePaintFragmentCursor<'_>, Offset),
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
    owner: NativePaintFragmentCursor<'_>,
    offset: u32,
    empty: bool,
) -> Option<LocalCaretRect> {
    let metrics = owner.paint.text_control_caret_metrics?;
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
                            let rect = LocalCaretRect {
                                offset: Offset {
                                    x: position.x + x,
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
                            };
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
    fn editor(node: NativePaintFragmentCursor<'_>, parent: Offset) -> Option<LocalCaretRect> {
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
            return Some(rect);
        }
    }
    let border = owner.paint.border;
    let padding = owner.paint.padding;
    Some(LocalCaretRect {
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
    })
}

#[inline(never)]
pub(super) fn text_control_offset_for_point(
    owner: NativePaintFragmentCursor<'_>,
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
