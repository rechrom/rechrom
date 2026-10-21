// cpp: layoutng_geometry/geometry/box_sides.h:10-13
// Pending connection to //src/foundation:text_values_api.
use foundation::{
    IsRtl, LogicalToPhysical, PhysicalToLogical, TextDirection, WritingDirectionMode, WritingMode,
};

// cpp: layoutng_geometry/geometry/box_sides.h:19-27
/// Side presence in the logical coordinate space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalBoxSides {
    pub inline_start: bool,
    pub inline_end: bool,
    pub block_start: bool,
    pub block_end: bool,
}

#[allow(non_snake_case)]
impl LogicalBoxSides {
    // cpp: layoutng_geometry/geometry/box_sides.h:29-39
    pub fn with_value(value: bool) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn new(inline_start: bool, inline_end: bool, block_start: bool, block_end: bool) -> Self {
        Self {
            inline_start,
            inline_end,
            block_start,
            block_end,
        }
    }

    // cpp: layoutng_geometry/geometry/box_sides.h:43-50
    // Equality is derived above.
    pub fn IsEmpty(&self) -> bool {
        !self.block_start && !self.inline_start && !self.block_end && !self.inline_end
    }

    // cpp: layoutng_geometry/geometry/box_sides.h:41
    // cpp: layoutng_geometry/geometry/box_sides.h:145-151
    pub fn ToPhysical(&self, writing_direction: WritingDirectionMode) -> PhysicalBoxSides {
        let converter = LogicalToPhysical::new(
            writing_direction,
            self.inline_start,
            self.inline_end,
            self.block_start,
            self.block_end,
        );
        PhysicalBoxSides::new(
            converter.Top(),
            converter.Right(),
            converter.Bottom(),
            converter.Left(),
        )
    }
}

// cpp: layoutng_geometry/geometry/box_sides.h:29-31
impl Default for LogicalBoxSides {
    fn default() -> Self {
        Self::with_value(true)
    }
}

// cpp: layoutng_geometry/geometry/box_sides.h:52-72
/// Side presence in the line-relative coordinate space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineLogicalBoxSides {
    pub block_start: bool,
    pub line_right: bool,
    pub block_end: bool,
    pub line_left: bool,
}

#[allow(non_snake_case)]
impl LineLogicalBoxSides {
    pub fn new(block_start: bool, line_right: bool, block_end: bool, line_left: bool) -> Self {
        Self {
            block_start,
            line_right,
            block_end,
            line_left,
        }
    }

    // cpp: layoutng_geometry/geometry/box_sides.h:74-82
    pub fn from_logical(sides: LogicalBoxSides, dir: TextDirection) -> Self {
        let mut result = Self::new(
            sides.block_start,
            sides.inline_end,
            sides.block_end,
            sides.inline_start,
        );
        if IsRtl(dir) {
            std::mem::swap(&mut result.line_left, &mut result.line_right);
        }
        result
    }

    // cpp: layoutng_geometry/geometry/box_sides.h:84-86
    pub fn IsEmpty(&self) -> bool {
        !self.block_start && !self.line_right && !self.block_end && !self.line_left
    }
}

// cpp: layoutng_geometry/geometry/box_sides.h:59-64
impl Default for LineLogicalBoxSides {
    fn default() -> Self {
        Self::new(true, true, true, true)
    }
}

// cpp: layoutng_geometry/geometry/box_sides.h:89-103
/// Side presence in physical coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalBoxSides {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

#[allow(non_snake_case)]
impl PhysicalBoxSides {
    pub fn with_value(value: bool) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn new(top: bool, right: bool, bottom: bool, left: bool) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    // cpp: layoutng_geometry/geometry/box_sides.h:104-128
    pub fn from_line_logical(logical: LineLogicalBoxSides, writing_mode: WritingMode) -> Self {
        if writing_mode == WritingMode::kHorizontalTb {
            return Self::new(
                logical.block_start,
                logical.line_right,
                logical.block_end,
                logical.line_left,
            );
        } else if writing_mode == WritingMode::kSidewaysLr {
            return Self::new(
                logical.line_right,
                logical.block_end,
                logical.line_left,
                logical.block_start,
            );
        } else {
            let top = logical.line_left;
            let bottom = logical.line_right;
            if writing_mode == WritingMode::kVerticalRl || writing_mode == WritingMode::kSidewaysRl
            {
                return Self::new(top, logical.block_start, bottom, logical.block_end);
            } else {
                debug_assert_eq!(writing_mode, WritingMode::kVerticalLr);
                return Self::new(top, logical.block_end, bottom, logical.block_start);
            }
        }
    }

    // cpp: layoutng_geometry/geometry/box_sides.h:130-134
    pub fn ToLogical(&self, writing_direction: WritingDirectionMode) -> LogicalBoxSides {
        let converter = PhysicalToLogical::new(
            writing_direction,
            self.top,
            self.right,
            self.bottom,
            self.left,
        );
        LogicalBoxSides::new(
            converter.InlineStart(),
            converter.InlineEnd(),
            converter.BlockStart(),
            converter.BlockEnd(),
        )
    }

    // cpp: layoutng_geometry/geometry/box_sides.h:136-143
    // Equality is derived above.
    pub fn IsEmpty(&self) -> bool {
        !self.top && !self.right && !self.bottom && !self.left
    }
    pub fn HasAllSides(&self) -> bool {
        self.top && self.right && self.bottom && self.left
    }
}

// cpp: layoutng_geometry/geometry/box_sides.h:99-101
impl Default for PhysicalBoxSides {
    fn default() -> Self {
        Self::with_value(true)
    }
}
