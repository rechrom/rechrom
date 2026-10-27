//! Shared layout positions for insertion carets, selection painting and hit testing.
use crate::fragment_tree::FragmentKind;
use crate::internal::layout_input::{Offset, Size, TextDirection};
use crate::layout_engine::NativePaintFragmentCursor;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalCaretRect {
    pub offset: Offset,
    pub size: Size,
}

#[path = "../../../layout_fragment/src/native_caret_geometry.rs"]
mod layout_fragment_native_caret_geometry;

/// UTF-16 offset, local to the control's border box. Use the inner editor for
/// empty values (placeholder glyphs never determine the insertion position).
#[allow(non_snake_case)]
pub fn TextControlCaretRect(
    owner: NativePaintFragmentCursor<'_>,
    offset: u32,
    empty: bool,
) -> Option<LocalCaretRect> {
    layout_fragment_native_caret_geometry::text_control_caret_rect(owner, offset, empty)
}

/// Hit test the closest shaped insertion boundary on the closest text line.
#[allow(non_snake_case)]
pub fn TextControlOffsetForPoint(
    owner: NativePaintFragmentCursor<'_>,
    point: Offset,
    empty: bool,
) -> u32 {
    layout_fragment_native_caret_geometry::text_control_offset_for_point(owner, point, empty)
}
