// cpp: layoutng_geometry/geometry/logical_rect.h:8-11
// cpp: layoutng_geometry/geometry/logical_rect.cc:7-11
// Pending connection to foundation geometry, gfx geometry and text APIs.
use super::box_strut::BoxStrut;
use super::logical_offset::LogicalOffset;
use super::logical_size::LogicalSize;
use foundation::{gfx, Format, LayoutUnit, String};

// cpp: layoutng_geometry/geometry/logical_rect.h:17-44
/// Position and size of a rectangle relative to its parent, in logical axes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalRect {
    pub offset: LogicalOffset,
    pub size: LogicalSize,
}

#[allow(non_snake_case)]
impl LogicalRect {
    pub fn new(offset: LogicalOffset, size: LogicalSize) -> Self {
        Self { offset, size }
    }

    pub fn from_units(
        inline_offset: LayoutUnit,
        block_offset: LayoutUnit,
        inline_size: LayoutUnit,
        block_size: LayoutUnit,
    ) -> Self {
        Self::new(
            LogicalOffset::new(inline_offset, block_offset),
            LogicalSize::new(inline_size, block_size),
        )
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:46-53
    pub fn IsEmpty(&self) -> bool {
        self.size.IsEmpty()
    }
    pub fn InlineStartOffset(&self) -> LayoutUnit {
        self.offset.inline_offset
    }
    pub fn BlockStartOffset(&self) -> LayoutUnit {
        self.offset.block_offset
    }
    pub fn InlineSize(&self) -> LayoutUnit {
        self.size.inline_size
    }
    pub fn BlockSize(&self) -> LayoutUnit {
        self.size.block_size
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:55-61
    pub fn InlineEndOffset(&self) -> LayoutUnit {
        self.offset.inline_offset + self.size.inline_size
    }
    pub fn BlockEndOffset(&self) -> LayoutUnit {
        self.offset.block_offset + self.size.block_size
    }
    pub fn EndOffset(&self) -> LogicalOffset {
        self.offset + self.size
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:69-70
    // cpp: layoutng_geometry/geometry/logical_rect.cc:34-43
    pub fn Unite(&mut self, other: &Self) {
        if other.IsEmpty() {
            return;
        }
        if self.IsEmpty() {
            *self = *other;
            return;
        }
        self.UniteEvenIfEmpty(other);
    }

    // cpp: layoutng_geometry/geometry/logical_rect.cc:45-51
    pub fn UniteEvenIfEmpty(&mut self, other: &Self) {
        let new_end_offset = max_offset(self.EndOffset(), other.EndOffset());
        let new_start_offset = min_offset(self.offset, other.offset);
        self.size = (new_end_offset - new_start_offset).into();
        self.offset = LogicalOffset::new(
            new_end_offset.inline_offset - self.size.inline_size,
            new_end_offset.block_offset - self.size.block_size,
        );
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:72-79
    pub fn Inflate(&mut self, d: LayoutUnit) {
        self.offset.inline_offset -= d;
        self.size.inline_size += d * 2;
        self.offset.block_offset -= d;
        self.size.block_size += d * 2;
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:81-92
    pub fn ExpandEdges(
        &mut self,
        block_start: LayoutUnit,
        inline_end: LayoutUnit,
        block_end: LayoutUnit,
        inline_start: LayoutUnit,
    ) {
        self.offset.inline_offset -= inline_start;
        self.offset.block_offset -= block_start;
        self.size.inline_size += inline_start + inline_end;
        self.size.block_size += block_start + block_end;
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:94
    // cpp: layoutng_geometry/geometry/logical_rect.cc:29-32
    pub fn Contract(&mut self, strut: &BoxStrut) {
        self.ExpandEdges(
            -strut.block_start,
            -strut.inline_end,
            -strut.block_end,
            -strut.inline_start,
        );
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:95-100
    pub fn ContractEdges(
        &mut self,
        block_start: LayoutUnit,
        inline_end: LayoutUnit,
        block_end: LayoutUnit,
        inline_start: LayoutUnit,
    ) {
        self.ExpandEdges(-block_start, -inline_end, -block_end, -inline_start);
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:102-107
    pub fn ShiftInlineStartEdgeTo(&mut self, edge: LayoutUnit) {
        let new_size = (self.InlineEndOffset() - edge).ClampNegativeToZero();
        self.offset.inline_offset = edge;
        self.size.inline_size = new_size;
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:109-114
    pub fn ShiftBlockStartEdgeTo(&mut self, edge: LayoutUnit) {
        let new_block_size = (self.BlockEndOffset() - edge).ClampNegativeToZero();
        self.offset.block_offset = edge;
        self.size.block_size = new_block_size;
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:116-119
    pub fn ShiftInlineEndEdgeTo(&mut self, edge: LayoutUnit) {
        self.size.inline_size = (edge - self.offset.inline_offset).ClampNegativeToZero();
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:121-124
    pub fn ShiftBlockEndEdgeTo(&mut self, edge: LayoutUnit) {
        self.size.block_size = (edge - self.offset.block_offset).ClampNegativeToZero();
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:126-135
    pub fn EnclosingRect(rect: &gfx::RectF) -> Self {
        let offset = LogicalOffset::new(
            LayoutUnit::FromFloatFloor(rect.x()),
            LayoutUnit::FromFloatFloor(rect.y()),
        );
        let size = LogicalSize::new(
            LayoutUnit::FromFloatCeil(rect.right()) - offset.inline_offset,
            LayoutUnit::FromFloatCeil(rect.bottom()) - offset.block_offset,
        );
        Self::new(offset, size)
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:137-139
    pub fn from_gfx_rect(rect: &gfx::Rect) -> Self {
        Self::new(
            LogicalOffset::new(
                LayoutUnit::from_signed(rect.x()),
                LayoutUnit::from_signed(rect.y()),
            ),
            LogicalSize::new(
                LayoutUnit::from_signed(rect.width()),
                LayoutUnit::from_signed(rect.height()),
            ),
        )
    }

    // cpp: layoutng_geometry/geometry/logical_rect.h:141
    // cpp: layoutng_geometry/geometry/logical_rect.cc:53-57
    pub fn ToString(&self) -> String {
        Format(
            "{},{} {}x{}",
            &[
                self.offset.inline_offset.ToString().into(),
                self.offset.block_offset.ToString().into(),
                self.size.inline_size.ToString().into(),
                self.size.block_size.ToString().into(),
            ],
        )
    }
}

// cpp: layoutng_geometry/geometry/logical_rect.h:31-41
// The deleted double overload and the testing-only integer overload have no
// production Rust equivalent; the public constructors require LayoutUnit.

// cpp: layoutng_geometry/geometry/logical_rect.h:63-67
// Equality is derived above.
impl std::ops::Add<LogicalOffset> for LogicalRect {
    type Output = Self;
    fn add(self, additional_offset: LogicalOffset) -> Self {
        Self::new(self.offset + additional_offset, self.size)
    }
}

// cpp: layoutng_geometry/geometry/logical_rect.cc:17-20
fn min_offset(a: LogicalOffset, b: LogicalOffset) -> LogicalOffset {
    LogicalOffset::new(
        if a.inline_offset < b.inline_offset {
            a.inline_offset
        } else {
            b.inline_offset
        },
        if a.block_offset < b.block_offset {
            a.block_offset
        } else {
            b.block_offset
        },
    )
}

// cpp: layoutng_geometry/geometry/logical_rect.cc:22-25
fn max_offset(a: LogicalOffset, b: LogicalOffset) -> LogicalOffset {
    LogicalOffset::new(
        if a.inline_offset > b.inline_offset {
            a.inline_offset
        } else {
            b.inline_offset
        },
        if a.block_offset > b.block_offset {
            a.block_offset
        } else {
            b.block_offset
        },
    )
}

// cpp: layoutng_geometry/geometry/logical_rect.h:144
// cpp: layoutng_geometry/geometry/logical_rect.cc:59-61
impl std::fmt::Display for LogicalRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ToString())
    }
}
