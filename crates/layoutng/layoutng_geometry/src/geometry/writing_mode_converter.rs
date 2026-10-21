// cpp: layoutng_geometry/geometry/writing_mode_converter.h:8-11
// Pending foundation geometry, gfx geometry and text-direction APIs.
use super::logical_offset::LogicalOffset;
use super::logical_rect::LogicalRect;
use super::logical_size::{LogicalSize, ToLogicalSize, ToPhysicalSize};
use foundation::{
    gfx, PhysicalOffset, PhysicalRect, PhysicalSize, TextDirection, WritingDirectionMode,
    WritingMode,
};

// cpp: layoutng_geometry/geometry/writing_mode_converter.h:15-28
/// Converts between logical and physical coordinate systems.
#[derive(Clone, Copy, Debug)]
pub struct WritingModeConverter {
    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:92-93
    writing_direction_: WritingDirectionMode,
    outer_size_: PhysicalSize,
}

#[allow(non_snake_case)]
impl WritingModeConverter {
    pub fn new(writing_direction: WritingDirectionMode, outer_size: PhysicalSize) -> Self {
        Self {
            writing_direction_: writing_direction,
            outer_size_: outer_size,
        }
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:30-34
    pub fn from_logical_size(
        writing_direction: WritingDirectionMode,
        outer_size: LogicalSize,
    ) -> Self {
        Self::new(
            writing_direction,
            ToPhysicalSize(outer_size, writing_direction.GetWritingMode()),
        )
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:36-39
    pub fn without_outer_size(writing_direction: WritingDirectionMode) -> Self {
        Self::new(writing_direction, PhysicalSize::default())
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:41-54
    pub fn GetWritingDirection(&self) -> WritingDirectionMode {
        self.writing_direction_
    }
    pub fn GetWritingMode(&self) -> WritingMode {
        self.writing_direction_.GetWritingMode()
    }
    pub fn Direction(&self) -> TextDirection {
        self.writing_direction_.Direction()
    }
    pub fn IsLtr(&self) -> bool {
        self.writing_direction_.IsLtr()
    }
    pub fn OuterSize(&self) -> PhysicalSize {
        self.outer_size_
    }
    pub fn SetOuterSize(&mut self, outer_size: PhysicalSize) {
        self.outer_size_ = outer_size;
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:61-62
    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:96-102
    pub fn ToLogicalOffset(
        &self,
        offset: PhysicalOffset,
        inner_size: PhysicalSize,
    ) -> LogicalOffset {
        if self.writing_direction_.IsHorizontalLtr() {
            return LogicalOffset::new(offset.left, offset.top);
        }
        self.slow_to_logical_offset(offset, inner_size)
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:63-64
    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:104-110
    pub fn ToPhysicalOffset(
        &self,
        offset: LogicalOffset,
        inner_size: PhysicalSize,
    ) -> PhysicalOffset {
        if self.writing_direction_.IsHorizontalLtr() {
            return PhysicalOffset::new(offset.inline_offset, offset.block_offset);
        }
        self.slow_to_physical_offset(offset, inner_size)
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:66-68
    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:112-120
    pub fn ToLogicalSize(&self, size: PhysicalSize) -> LogicalSize {
        ToLogicalSize(size, self.GetWritingMode())
    }
    pub fn ToPhysicalSize(&self, size: LogicalSize) -> PhysicalSize {
        ToPhysicalSize(size, self.GetWritingMode())
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:70-72
    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:122-136
    pub fn ToLogicalRect(&self, rect: PhysicalRect) -> LogicalRect {
        if self.writing_direction_.IsHorizontalLtr() {
            return LogicalRect::from_units(rect.X(), rect.Y(), rect.Width(), rect.Height());
        }
        self.slow_to_logical_rect(rect)
    }
    pub fn ToPhysicalRect(&self, rect: LogicalRect) -> PhysicalRect {
        if self.writing_direction_.IsHorizontalLtr() {
            return PhysicalRect::from_units(
                rect.offset.inline_offset,
                rect.offset.block_offset,
                rect.size.inline_size,
                rect.size.block_size,
            );
        }
        self.slow_to_physical_rect(rect)
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:74-77
    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:138-157
    pub fn ToLogicalPointF(&self, offset: gfx::PointF) -> gfx::PointF {
        if self.writing_direction_.IsHorizontalLtr() {
            return offset;
        }
        self.slow_to_logical_point_f(offset, gfx::SizeF::default())
    }
    pub fn ToLogicalSizeF(&self, size: gfx::SizeF) -> gfx::SizeF {
        if self.writing_direction_.IsHorizontal() {
            size
        } else {
            gfx::TransposeSize(size)
        }
    }
    pub fn ToLogicalRectF(&self, rect: gfx::RectF) -> gfx::RectF {
        if self.writing_direction_.IsHorizontalLtr() {
            return rect;
        }
        self.slow_to_logical_rect_f(rect)
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:79-83
    // cpp: layoutng_geometry/geometry/writing_mode_converter.cc:9-38
    fn slow_to_logical_offset(
        &self,
        offset: PhysicalOffset,
        inner_size: PhysicalSize,
    ) -> LogicalOffset {
        match self.GetWritingMode() {
            WritingMode::kHorizontalTb => {
                debug_assert!(!self.IsLtr());
                LogicalOffset::new(
                    self.outer_size_.width - offset.left - inner_size.width,
                    offset.top,
                )
            }
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                if self.IsLtr() {
                    return LogicalOffset::new(
                        offset.top,
                        self.outer_size_.width - offset.left - inner_size.width,
                    );
                }
                LogicalOffset::new(
                    self.outer_size_.height - offset.top - inner_size.height,
                    self.outer_size_.width - offset.left - inner_size.width,
                )
            }
            WritingMode::kVerticalLr => {
                if self.IsLtr() {
                    return LogicalOffset::new(offset.top, offset.left);
                }
                LogicalOffset::new(
                    self.outer_size_.height - offset.top - inner_size.height,
                    offset.left,
                )
            }
            WritingMode::kSidewaysLr => {
                if self.IsLtr() {
                    return LogicalOffset::new(
                        self.outer_size_.height - offset.top - inner_size.height,
                        offset.left,
                    );
                }
                LogicalOffset::new(offset.top, offset.left)
            }
        }
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:88-89
    // cpp: layoutng_geometry/geometry/writing_mode_converter.cc:40-69
    fn slow_to_logical_point_f(&self, offset: gfx::PointF, inner_size: gfx::SizeF) -> gfx::PointF {
        match self.GetWritingMode() {
            WritingMode::kHorizontalTb => {
                debug_assert!(!self.IsLtr());
                gfx::PointF::new(
                    self.outer_size_.width.ToFloat() - offset.x() - inner_size.width(),
                    offset.y(),
                )
            }
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                if self.IsLtr() {
                    return gfx::PointF::new(
                        offset.y(),
                        self.outer_size_.width.ToFloat() - offset.x() - inner_size.width(),
                    );
                }
                gfx::PointF::new(
                    self.outer_size_.height.ToFloat() - offset.y() - inner_size.height(),
                    self.outer_size_.width.ToFloat() - offset.x() - inner_size.width(),
                )
            }
            WritingMode::kVerticalLr => {
                if self.IsLtr() {
                    return gfx::PointF::new(offset.y(), offset.x());
                }
                gfx::PointF::new(
                    self.outer_size_.height.ToFloat() - offset.y() - inner_size.height(),
                    offset.x(),
                )
            }
            WritingMode::kSidewaysLr => {
                if self.IsLtr() {
                    return gfx::PointF::new(
                        self.outer_size_.height.ToFloat() - offset.y() - inner_size.height(),
                        offset.x(),
                    );
                }
                gfx::PointF::new(offset.y(), offset.x())
            }
        }
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:82-83
    // cpp: layoutng_geometry/geometry/writing_mode_converter.cc:71-105
    fn slow_to_physical_offset(
        &self,
        offset: LogicalOffset,
        inner_size: PhysicalSize,
    ) -> PhysicalOffset {
        match self.GetWritingMode() {
            WritingMode::kHorizontalTb => {
                debug_assert!(!self.IsLtr());
                PhysicalOffset::new(
                    self.outer_size_.width - offset.inline_offset - inner_size.width,
                    offset.block_offset,
                )
            }
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                if self.IsLtr() {
                    return PhysicalOffset::new(
                        self.outer_size_.width - offset.block_offset - inner_size.width,
                        offset.inline_offset,
                    );
                }
                PhysicalOffset::new(
                    self.outer_size_.width - offset.block_offset - inner_size.width,
                    self.outer_size_.height - offset.inline_offset - inner_size.height,
                )
            }
            WritingMode::kVerticalLr => {
                if self.IsLtr() {
                    return PhysicalOffset::new(offset.block_offset, offset.inline_offset);
                }
                PhysicalOffset::new(
                    offset.block_offset,
                    self.outer_size_.height - offset.inline_offset - inner_size.height,
                )
            }
            WritingMode::kSidewaysLr => {
                if self.IsLtr() {
                    return PhysicalOffset::new(
                        offset.block_offset,
                        self.outer_size_.height - offset.inline_offset - inner_size.height,
                    );
                }
                PhysicalOffset::new(offset.block_offset, offset.inline_offset)
            }
        }
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:85
    // cpp: layoutng_geometry/geometry/writing_mode_converter.cc:107-111
    fn slow_to_logical_rect(&self, rect: PhysicalRect) -> LogicalRect {
        LogicalRect::new(
            self.slow_to_logical_offset(rect.offset, rect.size),
            self.ToLogicalSize(rect.size),
        )
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:90
    // cpp: layoutng_geometry/geometry/writing_mode_converter.cc:113-115
    fn slow_to_logical_rect_f(&self, rect: gfx::RectF) -> gfx::RectF {
        gfx::RectF::new(
            self.slow_to_logical_point_f(rect.origin(), rect.size()),
            self.ToLogicalSizeF(rect.size()),
        )
    }

    // cpp: layoutng_geometry/geometry/writing_mode_converter.h:86
    // cpp: layoutng_geometry/geometry/writing_mode_converter.cc:117-121
    fn slow_to_physical_rect(&self, rect: LogicalRect) -> PhysicalRect {
        let size = self.ToPhysicalSize(rect.size);
        PhysicalRect::new(self.slow_to_physical_offset(rect.offset, size), size)
    }
}
