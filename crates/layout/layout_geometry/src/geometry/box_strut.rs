// cpp: layoutng_geometry/geometry/box_strut.h:10-19
// cpp: layoutng_geometry/geometry/box_strut.cc:7-10
// Pending foundation geometry, text, gfx, and string dependencies.
use super::box_sides::PhysicalBoxSides;
use super::logical_offset::LogicalOffset;
use super::logical_rect::LogicalRect;
use super::logical_size::LogicalSize;
use foundation::{
    gfx, Format, IsLtr, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, String,
    TextDirection, WritingDirectionMode, WritingMode,
};

// cpp: layoutng_geometry/geometry/box_strut.h:30-42
/// Margins, borders, or padding on all four logical edges.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoxStrut {
    // cpp: layoutng_geometry/geometry/box_strut.h:108-111
    pub inline_start: LayoutUnit,
    pub inline_end: LayoutUnit,
    pub block_start: LayoutUnit,
    pub block_end: LayoutUnit,
}

#[allow(non_snake_case)]
impl BoxStrut {
    pub fn new(
        inline_start: LayoutUnit,
        inline_end: LayoutUnit,
        block_start: LayoutUnit,
        block_end: LayoutUnit,
    ) -> Self {
        Self {
            inline_start,
            inline_end,
            block_start,
            block_end,
        }
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:42
    // cpp: layoutng_geometry/geometry/box_strut.cc:24-32
    pub fn from_line(line_relative: &LineBoxStrut, is_flipped_lines: bool) -> Self {
        if !is_flipped_lines {
            Self::new(
                line_relative.inline_start,
                line_relative.inline_end,
                line_relative.line_over,
                line_relative.line_under,
            )
        } else {
            Self::new(
                line_relative.inline_start,
                line_relative.inline_end,
                line_relative.line_under,
                line_relative.line_over,
            )
        }
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:44-45
    // cpp: layoutng_geometry/geometry/box_strut.cc:34-38
    pub fn from_size_and_inner(outer_size: &LogicalSize, inner_rect: &LogicalRect) -> Self {
        Self::new(
            inner_rect.offset.inline_offset,
            outer_size.inline_size - inner_rect.InlineEndOffset(),
            inner_rect.offset.block_offset,
            outer_size.block_size - inner_rect.BlockEndOffset(),
        )
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:47-48
    // cpp: layoutng_geometry/geometry/box_strut.cc:40-46
    pub fn from_rects(outer_rect: &LogicalRect, inner_rect: &LogicalRect) -> Self {
        Self::new(
            inner_rect.offset.inline_offset - outer_rect.offset.inline_offset,
            outer_rect.InlineEndOffset() - inner_rect.InlineEndOffset(),
            inner_rect.offset.block_offset - outer_rect.offset.block_offset,
            outer_rect.BlockEndOffset() - inner_rect.BlockEndOffset(),
        )
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:50-52
    // cpp: layoutng_geometry/geometry/box_strut.cc:48-54
    pub fn Intersect(&mut self, other: &Self) -> &mut Self {
        self.inline_start = min_unit(self.inline_start, other.inline_start);
        self.inline_end = min_unit(self.inline_end, other.inline_end);
        self.block_start = min_unit(self.block_start, other.block_start);
        self.block_end = min_unit(self.block_end, other.block_end);
        self
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:54-59
    pub fn LineLeft(&self, direction: TextDirection) -> LayoutUnit {
        if IsLtr(direction) {
            self.inline_start
        } else {
            self.inline_end
        }
    }
    pub fn LineRight(&self, direction: TextDirection) -> LayoutUnit {
        if IsLtr(direction) {
            self.inline_end
        } else {
            self.inline_start
        }
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:61-66
    pub fn InlineSum(&self) -> LayoutUnit {
        self.inline_start + self.inline_end
    }
    pub fn BlockSum(&self) -> LayoutUnit {
        self.block_start + self.block_end
    }
    pub fn StartOffset(&self) -> LogicalOffset {
        LogicalOffset::new(self.inline_start, self.block_start)
    }
    pub fn IsEmpty(&self) -> bool {
        *self == Self::default()
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:68
    // cpp: layoutng_geometry/geometry/box_strut.h:300-323
    pub fn ConvertToPhysical(&self, writing_direction: WritingDirectionMode) -> PhysicalBoxStrut {
        let mut direction_start = self.inline_start;
        let mut direction_end = self.inline_end;
        if writing_direction.IsRtl() {
            std::mem::swap(&mut direction_start, &mut direction_end);
        }
        #[allow(unreachable_patterns)]
        match writing_direction.GetWritingMode() {
            WritingMode::kHorizontalTb => PhysicalBoxStrut::new(
                self.block_start,
                direction_end,
                self.block_end,
                direction_start,
            ),
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => PhysicalBoxStrut::new(
                direction_start,
                self.block_start,
                direction_end,
                self.block_end,
            ),
            WritingMode::kVerticalLr => PhysicalBoxStrut::new(
                direction_start,
                self.block_end,
                direction_end,
                self.block_start,
            ),
            WritingMode::kSidewaysLr => PhysicalBoxStrut::new(
                direction_end,
                self.block_end,
                direction_start,
                self.block_start,
            ),
            _ => unreachable!("C++ NOTREACHED: unknown writing mode"),
        }
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:106
    // cpp: layoutng_geometry/geometry/box_strut.cc:14-18
    pub fn ToString(&self) -> String {
        Format(
            "Inline: ({} {}) Block: ({} {})",
            &[
                self.inline_start.ToString().into(),
                self.inline_end.ToString().into(),
                self.block_start.ToString().into(),
                self.block_end.ToString().into(),
            ],
        )
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:72-78
impl std::ops::AddAssign<Self> for BoxStrut {
    fn add_assign(&mut self, other: Self) {
        self.inline_start += other.inline_start;
        self.inline_end += other.inline_end;
        self.block_start += other.block_start;
        self.block_end += other.block_end;
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:80-84
impl std::ops::Add<Self> for BoxStrut {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut result = self;
        result += other;
        result
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:86-92
impl std::ops::SubAssign<Self> for BoxStrut {
    fn sub_assign(&mut self, other: Self) {
        self.inline_start -= other.inline_start;
        self.inline_end -= other.inline_end;
        self.block_start -= other.block_start;
        self.block_end -= other.block_end;
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:94-104
// Equality is derived above.
impl std::ops::Sub<Self> for BoxStrut {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let mut result = self;
        result -= other;
        result
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:114
// cpp: layoutng_geometry/geometry/box_strut.cc:20-22
impl std::fmt::Display for BoxStrut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ToString())
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:116-133
/// Strut with block sides expressed as line-over and line-under.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LineBoxStrut {
    pub inline_start: LayoutUnit,
    pub inline_end: LayoutUnit,
    pub line_over: LayoutUnit,
    pub line_under: LayoutUnit,
}

#[allow(non_snake_case)]
impl LineBoxStrut {
    pub fn new(
        inline_start: LayoutUnit,
        inline_end: LayoutUnit,
        line_over: LayoutUnit,
        line_under: LayoutUnit,
    ) -> Self {
        Self {
            inline_start,
            inline_end,
            line_over,
            line_under,
        }
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:133
    // cpp: layoutng_geometry/geometry/box_strut.cc:56-65
    pub fn from_box(flow_relative: &BoxStrut, is_flipped_lines: bool) -> Self {
        if !is_flipped_lines {
            Self::new(
                flow_relative.inline_start,
                flow_relative.inline_end,
                flow_relative.block_start,
                flow_relative.block_end,
            )
        } else {
            Self::new(
                flow_relative.inline_start,
                flow_relative.inline_end,
                flow_relative.block_end,
                flow_relative.block_start,
            )
        }
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:135-140
    pub fn InlineSum(&self) -> LayoutUnit {
        self.inline_start + self.inline_end
    }
    pub fn BlockSum(&self) -> LayoutUnit {
        self.line_over + self.line_under
    }
    pub fn IsEmpty(&self) -> bool {
        self.inline_start == LayoutUnit::default()
            && self.inline_end == LayoutUnit::default()
            && self.line_over == LayoutUnit::default()
            && self.line_under == LayoutUnit::default()
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:142-154
// Equality is derived above.
// cpp: layoutng_geometry/geometry/box_strut.cc:67-71
impl std::fmt::Display for LineBoxStrut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Inline: ({} {}) Line: ({} {}) ",
            self.inline_start, self.inline_end, self.line_over, self.line_under
        )
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:156-167
/// Physical dimensions independent of writing mode and direction.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PhysicalBoxStrut {
    // cpp: layoutng_geometry/geometry/box_strut.h:287-290
    pub top: LayoutUnit,
    pub right: LayoutUnit,
    pub bottom: LayoutUnit,
    pub left: LayoutUnit,
}

#[allow(non_snake_case)]
impl PhysicalBoxStrut {
    pub fn with_value(value: LayoutUnit) -> Self {
        Self::new(value, value, value, value)
    }
    pub fn new(top: LayoutUnit, right: LayoutUnit, bottom: LayoutUnit, left: LayoutUnit) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:169-172
    // cpp: layoutng_geometry/geometry/box_strut.h:292-297
    pub fn FromInts(t: i32, r: i32, b: i32, l: i32) -> Self {
        Self::new(
            LayoutUnit::from_signed(t),
            LayoutUnit::from_signed(r),
            LayoutUnit::from_signed(b),
            LayoutUnit::from_signed(l),
        )
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:174-176
    // cpp: layoutng_geometry/geometry/box_strut.cc:73-78
    pub fn from_size_and_inner(outer_size: &PhysicalSize, inner_rect: &PhysicalRect) -> Self {
        Self::new(
            inner_rect.offset.top,
            outer_size.width - inner_rect.Right(),
            outer_size.height - inner_rect.Bottom(),
            inner_rect.offset.left,
        )
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:178-186
    pub fn Enclosing(outsets: &gfx::OutsetsF) -> Self {
        Self::new(
            LayoutUnit::FromFloatCeil(outsets.top()),
            LayoutUnit::FromFloatCeil(outsets.right()),
            LayoutUnit::FromFloatCeil(outsets.bottom()),
            LayoutUnit::FromFloatCeil(outsets.left()),
        )
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:188
    pub fn Offset(&self) -> PhysicalOffset {
        PhysicalOffset::new(self.left, self.top)
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:190-195
    pub fn TruncateSides(&mut self, sides_to_include: &PhysicalBoxSides) {
        self.top = if sides_to_include.top {
            self.top
        } else {
            LayoutUnit::default()
        };
        self.bottom = if sides_to_include.bottom {
            self.bottom
        } else {
            LayoutUnit::default()
        };
        self.left = if sides_to_include.left {
            self.left
        } else {
            LayoutUnit::default()
        };
        self.right = if sides_to_include.right {
            self.right
        } else {
            LayoutUnit::default()
        };
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:197-219
    pub fn ConvertToLogical(&self, writing_direction: WritingDirectionMode) -> BoxStrut {
        let mut strut = match writing_direction.GetWritingMode() {
            WritingMode::kHorizontalTb => {
                BoxStrut::new(self.left, self.right, self.top, self.bottom)
            }
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                BoxStrut::new(self.top, self.bottom, self.right, self.left)
            }
            WritingMode::kVerticalLr => BoxStrut::new(self.top, self.bottom, self.left, self.right),
            WritingMode::kSidewaysLr => BoxStrut::new(self.bottom, self.top, self.left, self.right),
        };
        if writing_direction.IsRtl() {
            std::mem::swap(&mut strut.inline_start, &mut strut.inline_end);
        }
        strut
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:221-227
    pub fn ConvertToLineLogical(&self, writing_direction: WritingDirectionMode) -> LineBoxStrut {
        LineBoxStrut::from_box(
            &self.ConvertToLogical(writing_direction),
            writing_direction.IsFlippedLines(),
        )
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:229-230
    pub fn HorizontalSum(&self) -> LayoutUnit {
        self.left + self.right
    }
    pub fn VerticalSum(&self) -> LayoutUnit {
        self.top + self.bottom
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:232-238
    pub fn Inflate(&mut self, diff: LayoutUnit) -> &mut Self {
        self.top += diff;
        self.right += diff;
        self.bottom += diff;
        self.left += diff;
        self
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:240-242
    // cpp: layoutng_geometry/geometry/box_strut.cc:80-86
    pub fn Unite(&mut self, other: &Self) -> &mut Self {
        self.top = max_unit(self.top, other.top);
        self.right = max_unit(self.right, other.right);
        self.bottom = max_unit(self.bottom, other.bottom);
        self.left = max_unit(self.left, other.left);
        self
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:277-283
    pub fn to_outsets_f(&self) -> gfx::OutsetsF {
        let mut result = gfx::OutsetsF::default();
        result.set_left(self.left.ToFloat());
        result.set_right(self.right.ToFloat());
        result.set_top(self.top.ToFloat());
        result.set_bottom(self.bottom.ToFloat());
        result
    }

    // cpp: layoutng_geometry/geometry/box_strut.h:285
    pub fn IsZero(&self) -> bool {
        self.top == LayoutUnit::default()
            && self.right == LayoutUnit::default()
            && self.bottom == LayoutUnit::default()
            && self.left == LayoutUnit::default()
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:244-250
impl std::ops::AddAssign<Self> for PhysicalBoxStrut {
    fn add_assign(&mut self, other: Self) {
        self.top += other.top;
        self.right += other.right;
        self.bottom += other.bottom;
        self.left += other.left;
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:252-258
impl std::ops::SubAssign<Self> for PhysicalBoxStrut {
    fn sub_assign(&mut self, other: Self) {
        self.top -= other.top;
        self.right -= other.right;
        self.bottom -= other.bottom;
        self.left -= other.left;
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:260-264
impl std::ops::Add<Self> for PhysicalBoxStrut {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut result = self;
        result += other;
        result
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:266-275
// Equality is derived above.
impl std::ops::Sub<Self> for PhysicalBoxStrut {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let mut result = self;
        result -= other;
        result
    }
}

// cpp: layoutng_geometry/geometry/box_strut.h:325-327
impl std::ops::Neg for PhysicalBoxStrut {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.top, -self.right, -self.bottom, -self.left)
    }
}

fn min_unit(a: LayoutUnit, b: LayoutUnit) -> LayoutUnit {
    if a <= b {
        a
    } else {
        b
    }
}
fn max_unit(a: LayoutUnit, b: LayoutUnit) -> LayoutUnit {
    if a >= b {
        a
    } else {
        b
    }
}
