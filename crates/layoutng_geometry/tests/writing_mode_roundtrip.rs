use foundation::{LayoutUnit, PhysicalRect, TextDirection, WritingDirectionMode, WritingMode};
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

#[test]
fn physical_rect_roundtrips_across_writing_modes_and_directions() {
    let outer =
        foundation::PhysicalSize::new(LayoutUnit::from_signed(120), LayoutUnit::from_signed(90));
    let rect = PhysicalRect::from_units(
        LayoutUnit::from_signed(13),
        LayoutUnit::from_signed(17),
        LayoutUnit::from_signed(29),
        LayoutUnit::from_signed(23),
    );
    for mode in [
        WritingMode::kHorizontalTb,
        WritingMode::kVerticalRl,
        WritingMode::kVerticalLr,
        WritingMode::kSidewaysRl,
        WritingMode::kSidewaysLr,
    ] {
        for direction in [TextDirection::kLtr, TextDirection::kRtl] {
            let converter =
                WritingModeConverter::new(WritingDirectionMode::new(mode, direction), outer);
            assert_eq!(
                converter.ToPhysicalRect(converter.ToLogicalRect(rect)),
                rect
            );
        }
    }
}
