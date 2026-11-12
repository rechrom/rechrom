//! Source: core/layout/layout_inline.cc:253-286.
use super::*;
use foundation::{TextDirection, WritingDirectionMode};
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

#[allow(non_snake_case)]
pub(super) fn LayoutInlineLocalCaretRect(
    owner: &LayoutInline,
    _offset: i32,
    shape: CaretShape,
) -> PhysicalRect {
    owner.CheckIsNotDestroyed();
    if !owner.FirstChild().is_null() {
        return PhysicalRect::default();
    }
    let rect = owner.LocalCaretRectForEmptyElement(
        owner.BorderPaddingInlineSize(),
        LayoutUnit::default(),
        shape,
    );
    if owner.IsInLayoutNGInlineFormattingContext() {
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(owner);
        if !cursor.IsNull() {
            let mut physical = WritingModeConverter::new(
                WritingDirectionMode::new(owner.StyleRef().GetWritingMode(), TextDirection::kLtr),
                cursor.Current().Size(),
            )
            .ToPhysicalRect(rect);
            physical.offset += cursor.Current().OffsetInContainerFragment();
            return physical;
        }
    }
    PhysicalRect::from_units(
        rect.offset.inline_offset,
        rect.offset.block_offset,
        rect.size.inline_size,
        rect.size.block_size,
    )
}
