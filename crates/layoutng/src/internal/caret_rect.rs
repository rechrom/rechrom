#![allow(non_snake_case, non_camel_case_types)]

use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng/internal/caret_rect.h:20-20
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretShape {
    kBar,
    kBlock,
    kUnderscore,
}

// The supplied header only forward-declares InlineCaretPosition and
// LocalCaretRect, and the supplied source tree defines none of these four
// functions. Associated types retain their external ownership and prevent
// fabricated caret geometry from entering the layout implementation.
// cpp: layoutng/internal/caret_rect.h:14-17,22-36
pub trait CaretRectServices {
    type InlineCaretPosition;
    type LocalCaretRect;

    // cpp: layoutng/internal/caret_rect.h:23-23
    fn GetCaretShapeFromComputedStyle(style: &ComputedStyle) -> CaretShape;

    // cpp: layoutng/internal/caret_rect.h:26-26
    fn ComputeLocalCaretRect(
        position: &Self::InlineCaretPosition,
        shape: CaretShape,
    ) -> Self::LocalCaretRect;

    // cpp: layoutng/internal/caret_rect.h:30-30
    fn ComputeLocalSelectionRect(position: &Self::InlineCaretPosition) -> Self::LocalCaretRect;

    // cpp: layoutng/internal/caret_rect.h:34-36
    fn GetCaretRectAtTextOffset(
        cursor: &InlineCursor,
        text_offset: u32,
        shape: CaretShape,
    ) -> LogicalRect;
}
