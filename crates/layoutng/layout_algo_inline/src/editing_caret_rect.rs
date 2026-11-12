//! Source: core/layout/inline/caret_rect.cc:52-399.
//! Native inline caret geometry for editing and paint. The embedding
//! supplies LocalFrameView::BarCaretWidth as a platform-neutral CSS length.
use crate::inline_cursor::InlineCursor;
use crate::internal::caret_rect::CaretShape;
use crate::layout_algo_inline_editing_caret::{InlineCaretPosition, InlineCaretPositionType};
use font_engine::fonts::shaping::text_width::ComputeTextWidth;
use font_engine::text::implementation::character_break_iterator::LengthOfGraphemeCluster;
use foundation::{
    DynamicTo, ETextAlign, IsFlippedLinesWritingMode, LayoutUnit, PhysicalRect, TextDirection,
    WritingDirectionMode, WritingMode,
};
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

fn ResolvedDirection(cursor: &InlineCursor) -> TextDirection {
    if cursor.Current().IsText() || cursor.Current().IsAtomicInline() {
        return cursor.Current().ResolvedDirection();
    }
    let mut line = cursor.clone();
    line.MoveToContainingLine();
    line.Current().BaseDirection()
}
fn ShouldAlignRight(align: ETextAlign, direction: TextDirection) -> bool {
    match align {
        ETextAlign::kRight | ETextAlign::kWebkitRight => true,
        ETextAlign::kLeft
        | ETextAlign::kWebkitLeft
        | ETextAlign::kCenter
        | ETextAlign::kWebkitCenter => false,
        ETextAlign::kEnd => direction == TextDirection::kLtr,
        ETextAlign::kJustify | ETextAlign::kStart | ETextAlign::kMatchParent => {
            direction == TextDirection::kRtl
        }
    }
}
fn ClampAndRound(value: LayoutUnit, min: LayoutUnit, max: LayoutUnit) -> LayoutUnit {
    let min = LayoutUnit::from_signed(min.Ceil());
    let max = LayoutUnit::from_signed(max.Floor());
    if min >= max {
        return max;
    }
    LayoutUnit::from_signed(value.clamp(min, max).Round())
}
fn CharacterWidth(cursor: &InlineCursor, offset: u32) -> LayoutUnit {
    let text = cursor.CurrentText();
    let width = ComputeTextWidth(
        &text.Substring(offset, LengthOfGraphemeCluster(&text, offset)),
        cursor.Current().Style().GetFont(),
    );
    if width != 0.0 {
        return LayoutUnit::from_f64(f64::from(width));
    }
    ZeroWidth(cursor)
}
fn ZeroWidth(cursor: &InlineCursor) -> LayoutUnit {
    let font = unsafe { &*cursor.Current().Style().GetFont() };
    unsafe { font.PrimaryFont().as_ref() }.map_or(LayoutUnit::default(), |primary| {
        LayoutUnit::from_f64(f64::from(primary.GetFontMetrics().ZeroWidth()))
    })
}
fn NextInline(cursor: &InlineCursor) -> InlineCursor {
    let mut next = cursor.clone();
    if ResolvedDirection(cursor) == TextDirection::kRtl {
        next.MoveToPrevious();
    } else {
        next.MoveToNext();
    }
    next
}
fn LogicalTextCaretRect(
    cursor: &InlineCursor,
    offset: u32,
    shape: CaretShape,
    width: LayoutUnit,
) -> LogicalRect {
    let style = cursor.Current().Style();
    let converter = WritingModeConverter::new(
        WritingDirectionMode::new(style.GetWritingMode(), TextDirection::kLtr),
        cursor.Current().Size(),
    );
    let mut block_size = converter.ToLogicalSize(cursor.Current().Size()).block_size;
    let mut block_offset = LayoutUnit::default();
    let inline_size = if shape == CaretShape::kBar {
        width
    } else if offset < cursor.Current().TextEndOffset() {
        CharacterWidth(cursor, offset - cursor.Current().TextStartOffset())
    } else {
        let next = NextInline(cursor);
        if !next.IsNull() && next.Current().IsText() && !cursor.Current().IsLineBreak() {
            let converter = WritingModeConverter::new(
                WritingDirectionMode::new(
                    next.Current().Style().GetWritingMode(),
                    ResolvedDirection(&next),
                ),
                next.Current().Size(),
            );
            block_size = converter.ToLogicalSize(next.Current().Size()).block_size;
            let delta = next.Current().OffsetInContainerFragment()
                - cursor.Current().OffsetInContainerFragment();
            block_offset = if style.IsHorizontalWritingMode() {
                delta.top
            } else {
                delta.left
            };
            CharacterWidth(&next, 0)
        } else {
            ZeroWidth(cursor)
        }
    };
    if shape == CaretShape::kUnderscore {
        if !IsFlippedLinesWritingMode(style.GetWritingMode()) {
            block_offset += block_size - width;
        }
        block_size = width;
    }
    let mut left = cursor.CaretInlinePositionForOffset(offset);
    let item = unsafe { &*cursor.CurrentItem() };
    if item.IsSvgText() {
        left /= item.SvgScalingFactor();
    }
    if shape == CaretShape::kBar {
        if !cursor.Current().IsLineBreak() {
            left -= width / 2;
        }
    } else if ResolvedDirection(cursor) == TextDirection::kRtl {
        left -= inline_size;
    }
    LogicalRect::from_units(left, block_offset, inline_size, block_size)
}
/// The result is local to the container fragment's content coordinate space.
pub(crate) fn BarCaretRect(
    position: &InlineCaretPosition,
    width: LayoutUnit,
) -> Option<PhysicalRect> {
    CaretRect(position, CaretShape::kBar, width)
}
pub(crate) fn CaretRect(
    position: &InlineCaretPosition,
    shape: CaretShape,
    width: LayoutUnit,
) -> Option<PhysicalRect> {
    let cursor = &position.cursor;
    if cursor.IsNull() {
        return None;
    }
    let mut line = cursor.clone();
    line.MoveToContainingLine();
    if line.IsNull() {
        return None;
    }
    let style = cursor.Current().Style();
    if position.position_type != InlineCaretPositionType::AtTextOffset {
        let atomic = cursor.Current().IsAtomicInline();
        let converter = WritingModeConverter::new(
            WritingDirectionMode::new(style.GetWritingMode(), TextDirection::kLtr),
            if atomic {
                cursor.Current().Size()
            } else {
                cursor.ContainerFragment().Size()
            },
        );
        let line_rect = converter.ToLogicalRect(line.Current().RectInContainerFragment());
        let item_rect = converter.ToLogicalRect(cursor.Current().RectInContainerFragment());
        let mut rect = LogicalRect::from_units(
            item_rect.offset.inline_offset,
            line_rect.offset.block_offset,
            width,
            line_rect.size.block_size,
        );
        if atomic {
            rect.offset.inline_offset = LayoutUnit::default();
            rect.offset.block_offset -= item_rect.offset.block_offset;
        }
        let ltr = ResolvedDirection(cursor) == TextDirection::kLtr;
        if ltr != (position.position_type == InlineCaretPositionType::BeforeBox) {
            rect.offset.inline_offset += item_rect.size.inline_size - width;
        }
        if shape != CaretShape::kBar && position.position_type == InlineCaretPositionType::AfterBox
        {
            let next = NextInline(cursor);
            if !next.IsNull() && next.Current().IsText() {
                let size =
                    LogicalTextCaretRect(&next, next.Current().TextStartOffset(), shape, width)
                        .size;
                match next.Current().Style().GetWritingMode() {
                    WritingMode::kHorizontalTb => {
                        rect.offset.block_offset += rect.size.block_size - size.block_size;
                    }
                    WritingMode::kVerticalLr | WritingMode::kVerticalRl => {
                        rect.offset.block_offset += if shape == CaretShape::kBlock {
                            (rect.size.block_size - size.block_size) / 2
                        } else if next.Current().Style().GetWritingMode()
                            == WritingMode::kVerticalLr
                        {
                            (rect.size.block_size - next.Current().Size().width) / 2
                        } else {
                            (rect.size.block_size + next.Current().Size().width) / 2
                                - size.block_size
                        };
                    }
                    WritingMode::kSidewaysRl | WritingMode::kSidewaysLr => {
                        let adjustment = (rect.size.block_size - item_rect.size.block_size) / 2;
                        rect.offset.block_offset += rect.size.block_size
                            - adjustment
                            - if shape == CaretShape::kBlock {
                                size.block_size
                            } else {
                                LayoutUnit::default()
                            };
                    }
                }
                rect.size = size;
            }
        }
        let mut rect = converter.ToPhysicalRect(rect);
        if atomic {
            rect.offset += cursor.Current().OffsetInContainerFragment();
        }
        return Some(rect);
    }
    let offset = position.text_offset?;
    let converter = WritingModeConverter::new(
        WritingDirectionMode::new(style.GetWritingMode(), TextDirection::kLtr),
        cursor.Current().Size(),
    );
    let mut rect = converter.ToPhysicalRect(LogicalTextCaretRect(cursor, offset, shape, width));
    let mut location = rect.offset + cursor.Current().OffsetInContainerFragment();
    let object = unsafe { &*cursor.Current().GetLayoutObject() };
    let combine =
        DynamicTo::<crate::internal::layout_text_combine::LayoutTextCombine>(object.Parent());
    if let Some(combine) = unsafe { combine.as_ref() } {
        location = combine.AdjustOffsetForLocalCaretRect(&location);
        if shape != CaretShape::kBar {
            rect = combine.AdjustRectForBoundingBox(&rect);
        }
    }
    let line_rect = line.Current().RectInContainerFragment();
    let token = unsafe { line.Current().GetInlineBreakToken().as_ref() };
    let last_line = token.is_none_or(|token| token.IsForcedBreak());
    let align_right = ShouldAlignRight(
        cursor
            .ContainerFragment()
            .Style()
            .GetTextAlignForLine(last_line),
        line.Current().BaseDirection(),
    );
    if style.IsHorizontalWritingMode() {
        if align_right {
            location.left = ClampAndRound(
                location.left,
                LayoutUnit::default().min(line_rect.offset.left),
                line_rect.offset.left + line_rect.size.width - width,
            );
        } else {
            location.left = ClampAndRound(
                location.left,
                line_rect.offset.left,
                cursor
                    .ContainerFragment()
                    .Size()
                    .width
                    .max(line_rect.offset.left + line_rect.size.width)
                    - width,
            );
        }
    } else {
        location.top = ClampAndRound(
            location.top,
            LayoutUnit::default().min(line_rect.offset.top),
            cursor
                .ContainerFragment()
                .Size()
                .height
                .max(line_rect.offset.top + line_rect.size.height)
                - width,
        );
    }
    Some(PhysicalRect::new(location, rect.size))
}
