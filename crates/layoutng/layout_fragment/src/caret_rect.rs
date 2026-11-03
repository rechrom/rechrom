//! Fragment-owned caret geometry contract.

#![allow(non_snake_case, non_camel_case_types)]

use layout_fragment::inline_cursor::InlineCursor;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_style::style::computed_style::ComputedStyle;

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
pub trait CaretRectServices {
    type InlineCaretPosition;
    type LocalCaretRect;

    fn GetCaretShapeFromComputedStyle(style: &ComputedStyle) -> CaretShape;

    fn ComputeLocalCaretRect(
        position: &Self::InlineCaretPosition,
        shape: CaretShape,
    ) -> Self::LocalCaretRect;

    fn ComputeLocalSelectionRect(position: &Self::InlineCaretPosition) -> Self::LocalCaretRect;

    fn GetCaretRectAtTextOffset(
        cursor: &InlineCursor,
        text_offset: u32,
        shape: CaretShape,
    ) -> LogicalRect;
}
