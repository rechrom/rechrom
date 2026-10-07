//! Shared layout positions for insertion carets, selection painting and hit testing.
use crate::fragment_tree::{FragmentKind, FragmentNode};
use crate::internal::layout_input::{Offset, Size, TextDirection};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalCaretRect {
    pub offset: Offset,
    pub size: Size,
}

pub fn TextInlinePosition(node: &FragmentNode, offset: u32) -> Option<f64> {
    node.paint
        .text_caret_positions
        .iter()
        .find(|(i, _)| *i == offset)
        .map(|(_, x)| *x)
}

pub fn TextFragmentCaretRect(node: &FragmentNode, offset: u32) -> Option<LocalCaretRect> {
    let x = TextInlinePosition(node, offset)?;
    let run = node.paint.glyph_runs.first()?;
    Some(LocalCaretRect {
        offset: Offset { x, y: 0.0 },
        size: Size {
            width: 1.0,
            height: node.size.height.max(run.font_size),
        },
    })
}

/// A text item is one visual bidi run. Clip range endpoints to its logical span.
pub fn TextFragmentRangeRect(node: &FragmentNode, start: u32, end: u32) -> Option<LocalCaretRect> {
    let start = start.max(node.text_start?);
    let end = end.min(node.text_end?);
    if start >= end {
        return None;
    }
    let a = TextFragmentCaretRect(node, start)?;
    let b = TextFragmentCaretRect(node, end)?;
    Some(LocalCaretRect {
        offset: Offset {
            x: a.offset.x.min(b.offset.x),
            y: a.offset.y,
        },
        size: Size {
            width: (b.offset.x - a.offset.x).abs(),
            height: a.size.height,
        },
    })
}

fn visit_text(node: &FragmentNode, parent: Offset, visit: &mut impl FnMut(&FragmentNode, Offset)) {
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
    for child in &node.children {
        visit_text(child, origin, visit);
    }
}

/// UTF-16 offset, local to the control's border box. Use the inner editor for
/// empty values (placeholder glyphs never determine the insertion position).
#[allow(non_snake_case)]
pub fn TextControlCaretRect(
    owner: &FragmentNode,
    offset: u32,
    empty: bool,
) -> Option<LocalCaretRect> {
    let metrics = owner.paint.text_control_caret_metrics?;
    let mut candidate: Option<(u32, LocalCaretRect)> = None;
    if !empty {
        for child in &owner.children {
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
                        if let Some(x) = TextInlinePosition(node, offset.min(end)) {
                            let top = position.y + node.paint.text_line_top_offset.unwrap_or(0.0);
                            let rect = LocalCaretRect {
                                offset: Offset {
                                    x: position.x + x,
                                    y: position.y
                                        + node
                                            .paint
                                            .glyph_runs
                                            .first()
                                            .map_or(0.0, |run| run.baseline - metrics.ascent)
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
    fn editor(node: &FragmentNode, parent: Offset) -> Option<LocalCaretRect> {
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
        for child in &node.children {
            if let Some(rect) = editor(child, position) {
                return Some(rect);
            }
        }
        None
    }
    for child in &owner.children {
        if let Some(rect) = editor(child, Offset::default()) {
            return Some(rect);
        }
    }
    let b = owner.paint.border;
    let p = owner.paint.padding;
    Some(LocalCaretRect {
        offset: Offset {
            x: if owner.paint.direction == TextDirection::kRtl {
                owner.size.width - b.right - p.right - 1.0
            } else {
                b.left + p.left
            },
            y: b.top + p.top + (metrics.line_height - metrics.font_height) * 0.5,
        },
        size: Size {
            width: 1.0,
            height: metrics.font_height,
        },
    })
}

/// Hit test the closest shaped insertion boundary on the closest text line.
#[allow(non_snake_case)]
pub fn TextControlOffsetForPoint(owner: &FragmentNode, point: Offset, empty: bool) -> u32 {
    if empty {
        return 0;
    }
    let mut best: Option<(f64, f64, u32)> = None;
    for child in &owner.children {
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
                for &(offset, x) in &node.paint.text_caret_positions {
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
