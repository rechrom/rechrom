// cpp: layoutng_geometry/geometry/static_position.h:8-13
// Pending foundation types: PhysicalOffset, PhysicalSize, WritingMode.
use super::logical_offset::LogicalOffset;
use super::writing_mode_converter::WritingModeConverter;
use foundation::{PhysicalOffset, PhysicalSize, WritingMode};

// cpp: layoutng_geometry/geometry/static_position.h:26-28
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineEdge {
    kInlineStart,
    kInlineCenter,
    kInlineEnd,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockEdge {
    kBlockStart,
    kBlockCenter,
    kBlockEnd,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalAlignmentDirection {
    kBlock,
    kInline,
}

// cpp: layoutng_geometry/geometry/static_position.h:19-48
#[derive(Clone, Copy, Debug)]
pub struct LogicalStaticPosition {
    pub offset: LogicalOffset,
    pub inline_edge: InlineEdge,
    pub block_edge: BlockEdge,
    pub align_self_direction: LogicalAlignmentDirection,
}

#[allow(non_snake_case)]
impl LogicalStaticPosition {
    pub fn from_offset(offset: LogicalOffset) -> Self {
        Self::new(
            offset,
            InlineEdge::kInlineStart,
            BlockEdge::kBlockStart,
            LogicalAlignmentDirection::kBlock,
        )
    }

    pub fn new(
        offset: LogicalOffset,
        inline_edge: InlineEdge,
        block_edge: BlockEdge,
        align_self_direction: LogicalAlignmentDirection,
    ) -> Self {
        Self {
            offset,
            inline_edge,
            block_edge,
            align_self_direction,
        }
    }

    // cpp: layoutng_geometry/geometry/static_position.h:151-164
    pub fn ConvertToPhysical(&self, converter: &WritingModeConverter) -> PhysicalStaticPosition {
        let physical_offset = converter.ToPhysicalOffset(self.offset, PhysicalSize::default());
        let (mut horizontal_edge, mut vertical_edge) = match converter.GetWritingMode() {
            // cpp: layoutng_geometry/geometry/static_position.h:165-172
            WritingMode::kHorizontalTb => (
                if (self.inline_edge == InlineEdge::kInlineStart) == converter.IsLtr() {
                    HorizontalEdge::kLeft
                } else {
                    HorizontalEdge::kRight
                },
                if self.block_edge == BlockEdge::kBlockStart {
                    VerticalEdge::kTop
                } else {
                    VerticalEdge::kBottom
                },
            ),
            // cpp: layoutng_geometry/geometry/static_position.h:173-180
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => (
                if self.block_edge == BlockEdge::kBlockEnd {
                    HorizontalEdge::kLeft
                } else {
                    HorizontalEdge::kRight
                },
                if (self.inline_edge == InlineEdge::kInlineStart) == converter.IsLtr() {
                    VerticalEdge::kTop
                } else {
                    VerticalEdge::kBottom
                },
            ),
            // cpp: layoutng_geometry/geometry/static_position.h:181-187
            WritingMode::kVerticalLr => (
                if self.block_edge == BlockEdge::kBlockStart {
                    HorizontalEdge::kLeft
                } else {
                    HorizontalEdge::kRight
                },
                if (self.inline_edge == InlineEdge::kInlineStart) == converter.IsLtr() {
                    VerticalEdge::kTop
                } else {
                    VerticalEdge::kBottom
                },
            ),
            // cpp: layoutng_geometry/geometry/static_position.h:188-195
            WritingMode::kSidewaysLr => (
                if self.block_edge == BlockEdge::kBlockStart {
                    HorizontalEdge::kLeft
                } else {
                    HorizontalEdge::kRight
                },
                if (self.inline_edge == InlineEdge::kInlineEnd) == converter.IsLtr() {
                    VerticalEdge::kTop
                } else {
                    VerticalEdge::kBottom
                },
            ),
        };

        // cpp: layoutng_geometry/geometry/static_position.h:197-227
        let physical_align_self_direction = match converter.GetWritingMode() {
            WritingMode::kHorizontalTb => {
                if self.inline_edge == InlineEdge::kInlineCenter {
                    horizontal_edge = HorizontalEdge::kHorizontalCenter;
                }
                if self.block_edge == BlockEdge::kBlockCenter {
                    vertical_edge = VerticalEdge::kVerticalCenter;
                }
                if self.align_self_direction == LogicalAlignmentDirection::kInline {
                    PhysicalAlignmentDirection::kHorizontal
                } else {
                    PhysicalAlignmentDirection::kVertical
                }
            }
            WritingMode::kVerticalRl
            | WritingMode::kSidewaysRl
            | WritingMode::kVerticalLr
            | WritingMode::kSidewaysLr => {
                if self.block_edge == BlockEdge::kBlockCenter {
                    horizontal_edge = HorizontalEdge::kHorizontalCenter;
                }
                if self.inline_edge == InlineEdge::kInlineCenter {
                    vertical_edge = VerticalEdge::kVerticalCenter;
                }
                if self.align_self_direction == LogicalAlignmentDirection::kInline {
                    PhysicalAlignmentDirection::kVertical
                } else {
                    PhysicalAlignmentDirection::kHorizontal
                }
            }
        };
        // cpp: layoutng_geometry/geometry/static_position.h:229-230
        PhysicalStaticPosition::new(
            physical_offset,
            horizontal_edge,
            vertical_edge,
            physical_align_self_direction,
        )
    }
}

// cpp: layoutng_geometry/geometry/static_position.h:30-39
impl Default for LogicalStaticPosition {
    fn default() -> Self {
        Self::from_offset(LogicalOffset::default())
    }
}

// cpp: layoutng_geometry/geometry/static_position.h:52-54
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalEdge {
    kLeft,
    kHorizontalCenter,
    kRight,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalEdge {
    kTop,
    kVerticalCenter,
    kBottom,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicalAlignmentDirection {
    kHorizontal,
    kVertical,
}

// cpp: layoutng_geometry/geometry/static_position.h:50-68
#[derive(Clone, Copy, Debug)]
pub struct PhysicalStaticPosition {
    pub offset: PhysicalOffset,
    pub horizontal_edge: HorizontalEdge,
    pub vertical_edge: VerticalEdge,
    pub align_self_direction: PhysicalAlignmentDirection,
}

#[allow(non_snake_case)]
impl PhysicalStaticPosition {
    pub fn new(
        offset: PhysicalOffset,
        horizontal_edge: HorizontalEdge,
        vertical_edge: VerticalEdge,
        align_self_direction: PhysicalAlignmentDirection,
    ) -> Self {
        Self {
            offset,
            horizontal_edge,
            vertical_edge,
            align_self_direction,
        }
    }

    // cpp: layoutng_geometry/geometry/static_position.h:70-84
    pub fn ConvertToLogical(&self, converter: &WritingModeConverter) -> LogicalStaticPosition {
        let logical_offset = converter.ToLogicalOffset(self.offset, PhysicalSize::default());
        let (mut inline_edge, mut block_edge) = match converter.GetWritingMode() {
            // cpp: layoutng_geometry/geometry/static_position.h:85-91
            WritingMode::kHorizontalTb => (
                if (self.horizontal_edge == HorizontalEdge::kLeft) == converter.IsLtr() {
                    InlineEdge::kInlineStart
                } else {
                    InlineEdge::kInlineEnd
                },
                if self.vertical_edge == VerticalEdge::kTop {
                    BlockEdge::kBlockStart
                } else {
                    BlockEdge::kBlockEnd
                },
            ),
            // cpp: layoutng_geometry/geometry/static_position.h:92-99
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => (
                if (self.vertical_edge == VerticalEdge::kTop) == converter.IsLtr() {
                    InlineEdge::kInlineStart
                } else {
                    InlineEdge::kInlineEnd
                },
                if self.horizontal_edge == HorizontalEdge::kRight {
                    BlockEdge::kBlockStart
                } else {
                    BlockEdge::kBlockEnd
                },
            ),
            // cpp: layoutng_geometry/geometry/static_position.h:100-106
            WritingMode::kVerticalLr => (
                if (self.vertical_edge == VerticalEdge::kTop) == converter.IsLtr() {
                    InlineEdge::kInlineStart
                } else {
                    InlineEdge::kInlineEnd
                },
                if self.horizontal_edge == HorizontalEdge::kLeft {
                    BlockEdge::kBlockStart
                } else {
                    BlockEdge::kBlockEnd
                },
            ),
            // cpp: layoutng_geometry/geometry/static_position.h:107-114
            WritingMode::kSidewaysLr => (
                if (self.vertical_edge == VerticalEdge::kBottom) == converter.IsLtr() {
                    InlineEdge::kInlineStart
                } else {
                    InlineEdge::kInlineEnd
                },
                if self.horizontal_edge == HorizontalEdge::kLeft {
                    BlockEdge::kBlockStart
                } else {
                    BlockEdge::kBlockEnd
                },
            ),
        };

        // cpp: layoutng_geometry/geometry/static_position.h:116-144
        let logical_align_self_direction = match converter.GetWritingMode() {
            WritingMode::kHorizontalTb => {
                if self.horizontal_edge == HorizontalEdge::kHorizontalCenter {
                    inline_edge = InlineEdge::kInlineCenter;
                }
                if self.vertical_edge == VerticalEdge::kVerticalCenter {
                    block_edge = BlockEdge::kBlockCenter;
                }
                if self.align_self_direction == PhysicalAlignmentDirection::kHorizontal {
                    LogicalAlignmentDirection::kInline
                } else {
                    LogicalAlignmentDirection::kBlock
                }
            }
            WritingMode::kVerticalRl
            | WritingMode::kSidewaysRl
            | WritingMode::kVerticalLr
            | WritingMode::kSidewaysLr => {
                if self.vertical_edge == VerticalEdge::kVerticalCenter {
                    inline_edge = InlineEdge::kInlineCenter;
                }
                if self.horizontal_edge == HorizontalEdge::kHorizontalCenter {
                    block_edge = BlockEdge::kBlockCenter;
                }
                if self.align_self_direction == PhysicalAlignmentDirection::kHorizontal {
                    LogicalAlignmentDirection::kBlock
                } else {
                    LogicalAlignmentDirection::kInline
                }
            }
        };
        // cpp: layoutng_geometry/geometry/static_position.h:146-147
        LogicalStaticPosition::new(
            logical_offset,
            inline_edge,
            block_edge,
            logical_align_self_direction,
        )
    }
}
