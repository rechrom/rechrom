//! Shared layout positions for insertion carets, selection painting and hit testing.
use crate::fragment_tree::FragmentKind;
use crate::internal::caret_rect::CaretShape;
use crate::internal::layout_input::{Offset, Size, TextDirection};
use crate::layout_engine::PaintFragmentCursor;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalCaretRect {
    pub offset: Offset,
    pub size: Size,
}

#[path = "../../../layout_fragment/src/caret_geometry.rs"]
mod layout_fragment_caret_geometry;

/// UTF-16 offset, local to the control's border box. Use the inner editor for
/// empty values (placeholder glyphs never determine the insertion position).
#[allow(non_snake_case)]
pub fn TextControlCaretRect(
    owner: PaintFragmentCursor<'_>,
    offset: u32,
    empty: bool,
    caret_shape: CaretShape,
) -> Option<LocalCaretRect> {
    layout_fragment_caret_geometry::text_control_caret_rect(owner, offset, empty, caret_shape)
}

/// Hit test the closest shaped insertion boundary on the closest text line.
#[allow(non_snake_case)]
pub fn TextControlOffsetForPoint(
    owner: PaintFragmentCursor<'_>,
    point: Offset,
    empty: bool,
) -> u32 {
    layout_fragment_caret_geometry::text_control_offset_for_point(owner, point, empty)
}
