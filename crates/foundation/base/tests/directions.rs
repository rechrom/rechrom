use foundation::blink_geometry::geometry::{LogicalDirection, PhysicalDirection};
use foundation::text::native::text_direction::{DirectionFromLevel, IsLtr, IsRtl};
use foundation::text::native::writing_mode::{
    IsFlippedBlocksWritingMode, IsFlippedLinesWritingMode, IsHorizontalTypographicMode,
    IsHorizontalWritingMode, IsParallelWritingMode, IsVerticalWritingMode, ToLineWritingMode,
};
use foundation::text::native::{TextDirection, WritingDirectionMode, WritingMode};

#[test]
fn enum_values_and_packing_match_cpp() {
    assert_eq!(TextDirection::kLtr as u8, 0);
    assert_eq!(TextDirection::kRtl as u8, 1);
    assert_eq!(WritingMode::kHorizontalTb as u8, 0);
    assert_eq!(WritingMode::kVerticalRl as u8, 1);
    assert_eq!(WritingMode::kVerticalLr as u8, 2);
    assert_eq!(WritingMode::kSidewaysRl as u8, 3);
    assert_eq!(WritingMode::kSidewaysLr as u8, 4);
    assert_eq!(WritingMode::kMaxWritingMode as u8, 4);
    assert_eq!(std::mem::size_of::<WritingDirectionMode>(), 2);
}

#[test]
fn direction_and_writing_mode_helpers_match_cpp() {
    assert!(IsLtr(DirectionFromLevel(0)));
    assert!(IsRtl(DirectionFromLevel(1)));
    assert!(IsLtr(DirectionFromLevel(2)));
    assert!(IsHorizontalWritingMode(WritingMode::kHorizontalTb));
    assert!(IsVerticalWritingMode(WritingMode::kVerticalRl));
    assert!(IsVerticalWritingMode(WritingMode::kVerticalLr));
    assert!(IsFlippedLinesWritingMode(WritingMode::kVerticalLr));
    assert!(IsFlippedBlocksWritingMode(WritingMode::kVerticalRl));
    assert!(IsFlippedBlocksWritingMode(WritingMode::kSidewaysRl));
    assert!(IsParallelWritingMode(
        WritingMode::kVerticalRl,
        WritingMode::kVerticalLr
    ));
    assert!(IsHorizontalTypographicMode(WritingMode::kSidewaysLr));
    assert_eq!(
        ToLineWritingMode(WritingMode::kVerticalLr),
        WritingMode::kVerticalRl
    );
}

#[test]
fn writing_direction_matches_cpp_tables() {
    use LogicalDirection::{kBlockEnd, kBlockStart, kInlineEnd, kInlineStart};
    use PhysicalDirection::{kDown, kLeft, kRight, kUp};

    let horizontal_ltr = WritingDirectionMode::new(WritingMode::kHorizontalTb, TextDirection::kLtr);
    assert_eq!(horizontal_ltr.InlineStart(), kLeft);
    assert_eq!(horizontal_ltr.InlineEnd(), kRight);
    assert_eq!(horizontal_ltr.BlockStart(), kUp);
    assert_eq!(horizontal_ltr.BlockEnd(), kDown);
    assert_eq!(horizontal_ltr.Top(), kBlockStart);
    assert_eq!(horizontal_ltr.Right(), kInlineEnd);
    assert_eq!(horizontal_ltr.Bottom(), kBlockEnd);
    assert_eq!(horizontal_ltr.Left(), kInlineStart);

    let horizontal_rtl = WritingDirectionMode::new(WritingMode::kHorizontalTb, TextDirection::kRtl);
    assert_eq!(horizontal_rtl.InlineStart(), kRight);
    assert_eq!(horizontal_rtl.InlineEnd(), kLeft);
    assert_eq!(horizontal_rtl.Top(), kBlockStart);
    assert_eq!(horizontal_rtl.Right(), kInlineStart);
    assert_eq!(horizontal_rtl.Bottom(), kBlockEnd);
    assert_eq!(horizontal_rtl.Left(), kInlineEnd);
    assert!(horizontal_rtl.IsFlippedX());
    assert!(!horizontal_rtl.IsFlippedY());

    let vertical_rtl = WritingDirectionMode::new(WritingMode::kVerticalRl, TextDirection::kRtl);
    assert_eq!(vertical_rtl.InlineStart(), kDown);
    assert_eq!(vertical_rtl.InlineEnd(), kUp);
    assert_eq!(vertical_rtl.BlockStart(), kRight);
    assert_eq!(vertical_rtl.BlockEnd(), kLeft);
    assert_eq!(vertical_rtl.LineOver(), kRight);
    assert_eq!(vertical_rtl.LineUnder(), kLeft);
    assert_eq!(vertical_rtl.Top(), kInlineEnd);
    assert_eq!(vertical_rtl.Right(), kBlockStart);
    assert_eq!(vertical_rtl.Bottom(), kInlineStart);
    assert_eq!(vertical_rtl.Left(), kBlockEnd);
    assert!(vertical_rtl.IsFlippedX());
    assert!(vertical_rtl.IsFlippedY());
}
