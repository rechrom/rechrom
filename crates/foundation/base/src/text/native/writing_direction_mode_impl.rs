// C++: src/foundation/text/native/writing_direction_mode.cc

use std::fmt;

use crate::blink_geometry::geometry::{LogicalDirection, PhysicalDirection};

use super::text_direction::TextDirection;
use super::writing_direction_mode::WritingDirectionMode;
use super::writing_mode::IsHorizontalWritingMode;

use LogicalDirection::{kBlockEnd, kBlockStart, kInlineEnd, kInlineStart};
use PhysicalDirection::{kDown, kLeft, kRight, kUp};

// C++: writing_direction_mode.cc, kInlineStartMap through kLineUnderMap.
const kInlineStartMap: [PhysicalDirection; 5] = [kLeft, kUp, kUp, kUp, kDown];
const kInlineEndMap: [PhysicalDirection; 5] = [kRight, kDown, kDown, kDown, kUp];
const kBlockStartMap: [PhysicalDirection; 5] = [kUp, kRight, kLeft, kRight, kLeft];
const kBlockEndMap: [PhysicalDirection; 5] = [kDown, kLeft, kRight, kLeft, kRight];
const kLineOverMap: [PhysicalDirection; 5] = [kUp, kRight, kRight, kRight, kLeft];
const kLineUnderMap: [PhysicalDirection; 5] = [kDown, kLeft, kLeft, kLeft, kRight];

// C++: writing_direction_mode.cc, kTopMap through kLeftMap.
const kTopMap: [LogicalDirection; 5] = [
    kBlockStart,
    kInlineStart,
    kInlineStart,
    kInlineStart,
    kInlineEnd,
];
const kRightMap: [LogicalDirection; 5] =
    [kInlineEnd, kBlockStart, kBlockEnd, kBlockStart, kBlockEnd];
const kBottomMap: [LogicalDirection; 5] =
    [kBlockEnd, kInlineEnd, kInlineEnd, kInlineEnd, kInlineStart];
const kLeftMap: [LogicalDirection; 5] =
    [kInlineStart, kBlockEnd, kBlockStart, kBlockEnd, kBlockStart];

impl WritingDirectionMode {
    // C++: writing_direction_mode.cc, InlineStart.
    pub fn InlineStart(&self) -> PhysicalDirection {
        if self.Direction() == TextDirection::kLtr {
            return kInlineStartMap[self.GetWritingMode() as usize];
        }
        kInlineEndMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, InlineEnd.
    pub fn InlineEnd(&self) -> PhysicalDirection {
        if self.Direction() == TextDirection::kLtr {
            return kInlineEndMap[self.GetWritingMode() as usize];
        }
        kInlineStartMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, BlockStart.
    pub fn BlockStart(&self) -> PhysicalDirection {
        kBlockStartMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, BlockEnd.
    pub fn BlockEnd(&self) -> PhysicalDirection {
        kBlockEndMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, LineOver.
    pub fn LineOver(&self) -> PhysicalDirection {
        kLineOverMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, LineUnder.
    pub fn LineUnder(&self) -> PhysicalDirection {
        kLineUnderMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, Top.
    pub fn Top(&self) -> LogicalDirection {
        if self.IsLtr() || IsHorizontalWritingMode(self.GetWritingMode()) {
            return kTopMap[self.GetWritingMode() as usize];
        }
        kBottomMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, Right.
    pub fn Right(&self) -> LogicalDirection {
        if self.IsLtr() || !IsHorizontalWritingMode(self.GetWritingMode()) {
            return kRightMap[self.GetWritingMode() as usize];
        }
        kLeftMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, Bottom.
    pub fn Bottom(&self) -> LogicalDirection {
        if self.IsLtr() || IsHorizontalWritingMode(self.GetWritingMode()) {
            return kBottomMap[self.GetWritingMode() as usize];
        }
        kTopMap[self.GetWritingMode() as usize]
    }

    // C++: writing_direction_mode.cc, Left.
    pub fn Left(&self) -> LogicalDirection {
        if self.IsLtr() || !IsHorizontalWritingMode(self.GetWritingMode()) {
            return kLeftMap[self.GetWritingMode() as usize];
        }
        kRightMap[self.GetWritingMode() as usize]
    }
}

// C++: writing_direction_mode.cc, operator<<.
impl fmt::Display for WritingDirectionMode {
    fn fmt(&self, ostream: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(ostream, "{} {}", self.GetWritingMode(), self.Direction())
    }
}
