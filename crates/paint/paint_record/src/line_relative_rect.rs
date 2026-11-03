//! Type-safe geometry for Paint's line-relative coordinate space.

#![allow(non_snake_case)]

use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform, gfx, FloorToInt, LayoutUnit,
    PhysicalOffset, PhysicalRect, SnapSizeToPixel, WritingMode,
};
use layoutng_assembly::fragment_item::FragmentItem;
use layoutng_geometry::geometry::logical_size::LogicalSize;

// Source: third_party/blink/renderer/core/paint/line_relative_rect.h:37-62.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineRelativeOffset {
    pub line_left: LayoutUnit,
    pub line_over: LayoutUnit,
}

impl LineRelativeOffset {
    pub const fn new(line_left: LayoutUnit, line_over: LayoutUnit) -> Self {
        Self {
            line_left,
            line_over,
        }
    }

    // A line box has the same origin in physical and line-relative space.
    pub const fn CreateFromBoxOrigin(origin: &PhysicalOffset) -> Self {
        Self::new(origin.left, origin.top)
    }

    pub fn ToPointF(self) -> gfx::PointF {
        gfx::PointF::new(self.line_left.ToFloat(), self.line_over.ToFloat())
    }
}

impl std::ops::Add for LineRelativeOffset {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.line_left + other.line_left,
            self.line_over + other.line_over,
        )
    }
}

// Source: third_party/blink/renderer/core/paint/line_relative_rect.h:64-68.
pub fn ToRoundedPoint(offset: &LineRelativeOffset) -> (i32, i32) {
    (offset.line_left.Round(), offset.line_over.Round())
}

// Source: third_party/blink/renderer/core/paint/line_relative_rect.h:72-188 and
//         third_party/blink/renderer/core/paint/line_relative_rect.cc:11-154.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LineRelativeRect {
    pub offset: LineRelativeOffset,
    pub size: LogicalSize,
}

impl LineRelativeRect {
    pub const fn new(offset: LineRelativeOffset, size: LogicalSize) -> Self {
        Self { offset, size }
    }

    pub fn EnclosingRect(rect: &gfx::RectF) -> Self {
        let offset = LineRelativeOffset::new(
            LayoutUnit::FromFloatFloor(rect.x()),
            LayoutUnit::FromFloatFloor(rect.y()),
        );
        let size = LogicalSize::new(
            LayoutUnit::FromFloatCeil(rect.right()) - offset.line_left,
            LayoutUnit::FromFloatCeil(rect.bottom()) - offset.line_over,
        );
        Self::new(offset, size)
    }

    pub fn CreateFromLineBox(rect: &PhysicalRect, is_horizontal: bool) -> Self {
        Self::new(
            LineRelativeOffset::CreateFromBoxOrigin(&rect.offset),
            LogicalSize::new(
                if is_horizontal {
                    rect.size.width
                } else {
                    rect.size.height
                },
                if is_horizontal {
                    rect.size.height
                } else {
                    rect.size.width
                },
            ),
        )
    }

    pub fn Create(rect: &PhysicalRect, rotation: Option<&AffineTransform>) -> Self {
        if rotation.is_none_or(AffineTransform::IsIdentity) {
            return Self::new(
                LineRelativeOffset::new(rect.offset.left, rect.offset.top),
                LogicalSize::new(rect.size.width, rect.size.height),
            );
        }
        Self::EnclosingRect(&rotation.unwrap().Inverse().MapRect((*rect).into()))
    }

    pub fn ToRectF(self) -> gfx::RectF {
        gfx::RectF::new(
            gfx::PointF::new(self.LineLeft().ToFloat(), self.LineOver().ToFloat()),
            gfx::SizeF::new(self.InlineSize().ToFloat(), self.BlockSize().ToFloat()),
        )
    }

    pub fn LineLeft(&self) -> LayoutUnit {
        self.offset.line_left
    }

    pub fn LineOver(&self) -> LayoutUnit {
        self.offset.line_over
    }

    pub fn InlineSize(&self) -> LayoutUnit {
        self.size.inline_size
    }

    pub fn BlockSize(&self) -> LayoutUnit {
        self.size.block_size
    }

    pub fn Move(&mut self, other: &LineRelativeOffset) {
        self.offset.line_left += other.line_left;
        self.offset.line_over += other.line_over;
    }

    pub fn PixelSnappedOffset(&self) -> gfx::Point {
        let (x, y) = ToRoundedPoint(&self.offset);
        gfx::Point::new(x, y)
    }

    pub fn PixelSnappedInlineSize(&self) -> i32 {
        SnapSizeToPixel(self.size.inline_size, self.offset.line_left)
    }

    pub fn PixelSnappedBlockSize(&self) -> i32 {
        SnapSizeToPixel(self.size.block_size, self.offset.line_over)
    }

    pub fn PixelSnappedSize(&self) -> gfx::Size {
        gfx::Size::new(self.PixelSnappedInlineSize(), self.PixelSnappedBlockSize())
    }

    pub fn ComputeRelativeToPhysicalTransform(&self, writing_mode: WritingMode) -> AffineTransform {
        if writing_mode == WritingMode::kHorizontalTb {
            return AffineTransform::default();
        }
        if writing_mode != WritingMode::kSidewaysLr {
            AffineTransform::new(
                0.0,
                1.0,
                -1.0,
                0.0,
                (self.LineLeft() + self.LineOver() + self.BlockSize()).ToDouble(),
                (self.LineOver() - self.LineLeft()).ToDouble(),
            )
        } else {
            AffineTransform::new(
                0.0,
                -1.0,
                1.0,
                0.0,
                (self.LineLeft() - self.LineOver()).ToDouble(),
                (self.LineLeft() + self.LineOver() + self.InlineSize()).ToDouble(),
            )
        }
    }

    pub fn EnclosingLineRelativeRect(&self) -> Self {
        let left = FloorToInt(self.offset.line_left);
        let top = FloorToInt(self.offset.line_over);
        let max_right = (self.offset.line_left + self.size.inline_size).Ceil();
        let max_bottom = (self.offset.line_over + self.size.block_size).Ceil();
        Self::new(
            LineRelativeOffset::new(LayoutUnit::from_signed(left), LayoutUnit::from_signed(top)),
            LogicalSize::new(
                LayoutUnit::from_signed(max_right - left),
                LayoutUnit::from_signed(max_bottom - top),
            ),
        )
    }

    pub fn Inflate(&mut self, d: LayoutUnit) {
        self.offset.line_left -= d;
        self.size.inline_size += d * 2;
        self.offset.line_over -= d;
        self.size.block_size += d * 2;
    }

    pub fn Unite(&mut self, other: &Self) {
        let left = self.offset.line_left.min(other.offset.line_left);
        let top = self.offset.line_over.min(other.offset.line_over);
        let right = (self.offset.line_left + self.size.inline_size)
            .max(other.offset.line_left + other.size.inline_size);
        let bottom = (self.offset.line_over + self.size.block_size)
            .max(other.offset.line_over + other.size.block_size);
        self.size = LogicalSize::new(right - left, bottom - top);
        self.offset =
            LineRelativeOffset::new(right - self.size.inline_size, bottom - self.size.block_size);
    }

    pub fn AdjustLineStartToInkOverflow(&mut self, fragment: &FragmentItem) {
        let ink_overflow = fragment.InkOverflowRect();
        let ink_left = match fragment.GetWritingMode() {
            WritingMode::kHorizontalTb => ink_overflow.X(),
            WritingMode::kVerticalRl | WritingMode::kVerticalLr | WritingMode::kSidewaysRl => {
                ink_overflow.Y()
            }
            WritingMode::kSidewaysLr => fragment.Size().height - ink_overflow.Bottom(),
        };
        self.offset.line_left += ink_left;
        self.size.inline_size -= ink_left;
    }

    pub fn AdjustLineEndToInkOverflow(&mut self, fragment: &FragmentItem) {
        let ink_overflow = fragment.InkOverflowRect();
        let mut ink_right = match fragment.GetWritingMode() {
            WritingMode::kHorizontalTb => ink_overflow.Right(),
            WritingMode::kVerticalRl | WritingMode::kVerticalLr | WritingMode::kSidewaysRl => {
                ink_overflow.Bottom()
            }
            WritingMode::kSidewaysLr => fragment.Size().height - ink_overflow.Y(),
        };
        if fragment.IsSvgText() {
            ink_right *= fragment.SvgScalingFactor();
        }
        self.size.inline_size = ink_right;
    }
}

impl From<LineRelativeRect> for gfx::RectF {
    fn from(rect: LineRelativeRect) -> Self {
        rect.ToRectF()
    }
}

impl std::ops::Add<LineRelativeOffset> for LineRelativeRect {
    type Output = Self;

    fn add(self, other: LineRelativeOffset) -> Self {
        Self::new(self.offset + other, self.size)
    }
}

pub fn ToPixelSnappedRect(rect: &LineRelativeRect) -> gfx::Rect {
    gfx::Rect::new(rect.PixelSnappedOffset(), rect.PixelSnappedSize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_box_origin_reuses_coordinates() {
        let physical = PhysicalOffset::new(LayoutUnit::from_f32(3.25), LayoutUnit::from_f32(-2.5));
        let relative = LineRelativeOffset::CreateFromBoxOrigin(&physical);
        assert_eq!(relative.line_left, physical.left);
        assert_eq!(relative.line_over, physical.top);
        assert_eq!(relative.ToPointF(), gfx::PointF::new(3.25, -2.5));
    }

    #[test]
    fn addition_and_rounding_match_line_relative_offset() {
        let offset =
            LineRelativeOffset::new(LayoutUnit::from_f32(1.25), LayoutUnit::from_f32(2.75))
                + LineRelativeOffset::new(LayoutUnit::from_f32(2.25), LayoutUnit::from_f32(-1.25));
        assert_eq!(offset.ToPointF(), gfx::PointF::new(3.5, 1.5));
        assert_eq!(ToRoundedPoint(&offset), (4, 2));
    }

    fn line_rect(x: f32, y: f32, inline: f32, block: f32) -> LineRelativeRect {
        LineRelativeRect::new(
            LineRelativeOffset::new(LayoutUnit::from_f32(x), LayoutUnit::from_f32(y)),
            LogicalSize::new(LayoutUnit::from_f32(inline), LayoutUnit::from_f32(block)),
        )
    }

    #[test]
    fn enclosing_rect_rounds_each_outer_edge() {
        let enclosed = LineRelativeRect::EnclosingRect(&gfx::RectF::new(
            gfx::PointF::new(1000.005625, 10000.005625),
            gfx::SizeF::new(10.005625, 100.005625),
        ));
        assert_eq!(enclosed.offset.line_left, LayoutUnit::from_signed(1000));
        assert_eq!(enclosed.offset.line_over, LayoutUnit::from_signed(10000));
        assert_eq!(enclosed.size.inline_size, LayoutUnit::from_f32(10.015625));
        assert_eq!(enclosed.size.block_size, LayoutUnit::from_f32(100.015625));
    }

    #[test]
    fn line_box_reuses_origin_and_swaps_vertical_size() {
        let physical = PhysicalRect::from_units(
            LayoutUnit::from_signed(10),
            LayoutUnit::from_signed(20),
            LayoutUnit::from_signed(30),
            LayoutUnit::from_signed(40),
        );
        assert_eq!(
            LineRelativeRect::CreateFromLineBox(&physical, true),
            line_rect(10.0, 20.0, 30.0, 40.0)
        );
        assert_eq!(
            LineRelativeRect::CreateFromLineBox(&physical, false),
            line_rect(10.0, 20.0, 40.0, 30.0)
        );
    }

    #[test]
    fn relative_to_physical_transform_matches_writing_mode_rotation() {
        let rect = line_rect(1000.0, 10000.0, 100.0, 50.0);
        let horizontal = rect.ComputeRelativeToPhysicalTransform(WritingMode::kHorizontalTb);
        assert!(horizontal.IsIdentity());

        let clockwise = rect.ComputeRelativeToPhysicalTransform(WritingMode::kVerticalRl);
        assert_eq!(
            clockwise,
            AffineTransform::new(0.0, 1.0, -1.0, 0.0, 11050.0, 9000.0)
        );

        let counter_clockwise = rect.ComputeRelativeToPhysicalTransform(WritingMode::kSidewaysLr);
        assert_eq!(
            counter_clockwise,
            AffineTransform::new(0.0, -1.0, 1.0, 0.0, -9000.0, 11100.0)
        );
    }

    #[test]
    fn enclosing_inflate_and_unite_match_fixed_point_geometry() {
        let enclosed = line_rect(10.25, 0.25, 10.5, 10.5).EnclosingLineRelativeRect();
        assert_eq!(enclosed, line_rect(10.0, 0.0, 11.0, 11.0));

        let mut inflated = line_rect(10.0, 20.0, 30.0, 40.0);
        inflated.Inflate(LayoutUnit::from_signed(2));
        assert_eq!(inflated, line_rect(8.0, 18.0, 34.0, 44.0));

        let mut united = line_rect(10.0, 0.0, 20.0, 10.0);
        united.Unite(&line_rect(0.0, 10.0, 10.0, 20.0));
        assert_eq!(united, line_rect(0.0, 0.0, 30.0, 30.0));
    }
}
