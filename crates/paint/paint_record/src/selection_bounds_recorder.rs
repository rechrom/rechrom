//! Selection handle metadata recorded after the selected drawing closes.

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::{TextDirection, WritingMode};
use layoutng_assembly::internal::selection_state::SelectionState;
use paint_chunk::{PaintSelectionBound, PaintSelectionBoundType};

use crate::paint_context::PaintContext;
use crate::PaintRect;

#[derive(Clone, Copy)]
enum RectEdge {
    TopLeftToBottomLeft,
    TopRightToBottomRight,
    TopLeftToTopRight,
    BottomLeftToBottomRight,
    TopRightToTopLeft,
    BottomRightToBottomLeft,
}

#[derive(Clone, Copy)]
struct BoundEdges {
    start: RectEdge,
    end: RectEdge,
}

fn GetBoundEdges(writing_mode: WritingMode, is_ltr: bool) -> BoundEdges {
    use RectEdge::*;
    match writing_mode {
        WritingMode::kHorizontalTb => {
            if is_ltr {
                BoundEdges {
                    start: TopLeftToBottomLeft,
                    end: TopRightToBottomRight,
                }
            } else {
                BoundEdges {
                    start: TopRightToBottomRight,
                    end: TopLeftToBottomLeft,
                }
            }
        }
        // LayoutPaintWritingMode intentionally folds sideways-rl into
        // vertical-rl before crossing the Layout -> Paint boundary.
        WritingMode::kVerticalRl => {
            if is_ltr {
                BoundEdges {
                    start: TopLeftToTopRight,
                    end: BottomRightToBottomLeft,
                }
            } else {
                BoundEdges {
                    start: BottomLeftToBottomRight,
                    end: TopRightToTopLeft,
                }
            }
        }
        WritingMode::kVerticalLr => {
            if is_ltr {
                BoundEdges {
                    start: TopRightToTopLeft,
                    end: BottomLeftToBottomRight,
                }
            } else {
                BoundEdges {
                    start: BottomRightToBottomLeft,
                    end: TopLeftToTopRight,
                }
            }
        }
        WritingMode::kSidewaysLr => {
            if is_ltr {
                BoundEdges {
                    start: BottomLeftToBottomRight,
                    end: TopLeftToTopRight,
                }
            } else {
                BoundEdges {
                    start: TopLeftToTopRight,
                    end: BottomLeftToBottomRight,
                }
            }
        }
    }
}

fn RectPoints(rect: PaintRect, edge: RectEdge) -> ([i32; 2], [i32; 2]) {
    let left = rect.x.round() as i32;
    let top = rect.y.round() as i32;
    let right = (rect.x + rect.width).round() as i32;
    let bottom = (rect.y + rect.height).round() as i32;
    use RectEdge::*;
    match edge {
        TopLeftToBottomLeft => ([left, top], [left, bottom]),
        TopRightToBottomRight => ([right, top], [right, bottom]),
        TopLeftToTopRight => ([left, top], [right, top]),
        BottomLeftToBottomRight => ([left, bottom], [right, bottom]),
        TopRightToTopLeft => ([right, top], [left, top]),
        BottomRightToBottomLeft => ([right, bottom], [left, bottom]),
    }
}

fn MakeBound(
    rect: PaintRect,
    edge: RectEdge,
    bound_type: PaintSelectionBoundType,
) -> PaintSelectionBound {
    let (edge_start, edge_end) = RectPoints(rect, edge);
    PaintSelectionBound {
        bound_type,
        edge_start,
        edge_end,
    }
}

fn ComputeSelectionBounds(
    state: SelectionState,
    selection_rect: PaintRect,
    text_direction: TextDirection,
    writing_mode: WritingMode,
) -> (Option<PaintSelectionBound>, Option<PaintSelectionBound>) {
    let is_ltr = text_direction == TextDirection::kLtr;
    let edges = GetBoundEdges(writing_mode, is_ltr);
    let start = matches!(state, SelectionState::kStart | SelectionState::kStartAndEnd).then(|| {
        MakeBound(
            selection_rect,
            edges.start,
            if is_ltr {
                PaintSelectionBoundType::Left
            } else {
                PaintSelectionBoundType::Right
            },
        )
    });
    let end = matches!(state, SelectionState::kEnd | SelectionState::kStartAndEnd).then(|| {
        MakeBound(
            selection_rect,
            edges.end,
            if is_ltr {
                PaintSelectionBoundType::Right
            } else {
                PaintSelectionBoundType::Left
            },
        )
    });
    (start, end)
}

/// Declared before DrawingRecorder by TextFragmentPainter so Drop records the
/// metadata after either a fresh drawing or its cached display item is closed.
pub(crate) struct SelectionBoundsRecorder<'c, 'o> {
    state: SelectionState,
    selection_rect: PaintRect,
    context: &'c RefCell<PaintContext<'o>>,
    text_direction: TextDirection,
    writing_mode: WritingMode,
}

impl<'c, 'o> SelectionBoundsRecorder<'c, 'o> {
    pub(crate) fn new(
        state: SelectionState,
        selection_rect: PaintRect,
        context: &'c RefCell<PaintContext<'o>>,
        text_direction: TextDirection,
        writing_mode: WritingMode,
    ) -> Self {
        debug_assert_ne!(state, SelectionState::kNone);
        Self {
            state,
            selection_rect,
            context,
            text_direction,
            writing_mode,
        }
    }
}

impl Drop for SelectionBoundsRecorder<'_, '_> {
    fn drop(&mut self) {
        let mut context = self.context.borrow_mut();
        context.RecordAnySelectionWasPainted();
        if self.state == SelectionState::kInside {
            return;
        }
        let (start, end) = ComputeSelectionBounds(
            self.state,
            self.selection_rect,
            self.text_direction,
            self.writing_mode,
        );
        if start.is_some() || end.is_some() {
            context.RecordSelectionBounds(start, end);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: PaintRect = PaintRect {
        x: 10.0,
        y: 20.0,
        width: 30.0,
        height: 40.0,
    };

    #[test]
    fn horizontal_ltr_uses_left_and_right_vertical_edges() {
        let (start, end) = ComputeSelectionBounds(
            SelectionState::kStartAndEnd,
            RECT,
            TextDirection::kLtr,
            WritingMode::kHorizontalTb,
        );
        assert_eq!(start.unwrap().edge_start, [10, 20]);
        assert_eq!(start.unwrap().edge_end, [10, 60]);
        assert_eq!(end.unwrap().edge_start, [40, 20]);
        assert_eq!(end.unwrap().edge_end, [40, 60]);
    }

    #[test]
    fn vertical_edges_follow_block_direction() {
        let (start, end) = ComputeSelectionBounds(
            SelectionState::kStartAndEnd,
            RECT,
            TextDirection::kLtr,
            WritingMode::kVerticalRl,
        );
        assert_eq!(
            (start.unwrap().edge_start, start.unwrap().edge_end),
            ([10, 20], [40, 20])
        );
        assert_eq!(
            (end.unwrap().edge_start, end.unwrap().edge_end),
            ([40, 60], [10, 60])
        );

        let (start, end) = ComputeSelectionBounds(
            SelectionState::kStartAndEnd,
            RECT,
            TextDirection::kRtl,
            WritingMode::kVerticalLr,
        );
        assert_eq!(
            (start.unwrap().edge_start, start.unwrap().edge_end),
            ([40, 60], [10, 60])
        );
        assert_eq!(
            (end.unwrap().edge_start, end.unwrap().edge_end),
            ([10, 20], [40, 20])
        );
    }

    #[test]
    fn sideways_lr_keeps_its_distinct_edge_orientation() {
        let (start, end) = ComputeSelectionBounds(
            SelectionState::kStartAndEnd,
            RECT,
            TextDirection::kLtr,
            WritingMode::kSidewaysLr,
        );
        assert_eq!(
            (start.unwrap().edge_start, start.unwrap().edge_end),
            ([10, 60], [40, 60])
        );
        assert_eq!(
            (end.unwrap().edge_start, end.unwrap().edge_end),
            ([10, 20], [40, 20])
        );
    }
}
