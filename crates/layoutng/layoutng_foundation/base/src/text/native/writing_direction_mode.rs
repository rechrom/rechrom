// C++: src/foundation/text/native/writing_direction_mode.h

use super::text_direction::{IsLtr, IsRtl, TextDirection};
use super::writing_mode::{
    IsFlippedBlocksWritingMode, IsFlippedLinesWritingMode, IsHorizontalWritingMode, WritingMode,
};

// C++ class WritingDirectionMode -> Rust struct + impl. The two enum fields
// remain in the same order and have the same u8 representation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WritingDirectionMode {
    writing_mode_: WritingMode,
    direction_: TextDirection,
}

impl WritingDirectionMode {
    // C++: writing_direction_mode.h, constructor.
    pub fn new(writing_mode: WritingMode, direction: TextDirection) -> Self {
        Self {
            writing_mode_: writing_mode,
            direction_: direction,
        }
    }

    // C++: writing_direction_mode.h, inline direction functions.
    pub fn Direction(&self) -> TextDirection {
        self.direction_
    }

    pub fn SetDirection(&mut self, direction: TextDirection) {
        self.direction_ = direction;
    }

    pub fn IsLtr(&self) -> bool {
        IsLtr(self.direction_)
    }

    pub fn IsRtl(&self) -> bool {
        IsRtl(self.direction_)
    }

    // C++: writing_direction_mode.h, block direction functions.
    pub fn GetWritingMode(&self) -> WritingMode {
        self.writing_mode_
    }

    pub fn SetWritingMode(&mut self, writing_mode: WritingMode) {
        self.writing_mode_ = writing_mode;
    }

    pub fn IsHorizontal(&self) -> bool {
        IsHorizontalWritingMode(self.writing_mode_)
    }

    pub fn IsFlippedBlocks(&self) -> bool {
        IsFlippedBlocksWritingMode(self.writing_mode_)
    }

    pub fn IsFlippedInlines(&self) -> bool {
        self.IsRtl() ^ (self.writing_mode_ == WritingMode::kSidewaysLr)
    }

    pub fn IsFlippedLines(&self) -> bool {
        IsFlippedLinesWritingMode(self.writing_mode_)
    }

    // C++: writing_direction_mode.h, IsFlippedX.
    pub fn IsFlippedX(&self) -> bool {
        if self.IsHorizontal() {
            return self.IsRtl();
        }
        self.IsFlippedBlocks()
    }

    // C++: writing_direction_mode.h, IsFlippedY.
    pub fn IsFlippedY(&self) -> bool {
        if self.IsHorizontal() {
            debug_assert!(!self.IsFlippedBlocks());
            return false;
        }
        self.IsFlippedInlines()
    }

    // C++: writing_direction_mode.h, IsHorizontalLtr.
    pub fn IsHorizontalLtr(&self) -> bool {
        self.IsHorizontal() && self.IsLtr()
    }

    // The directional methods declared next in the C++ header are defined
    // in writing_direction_mode_impl.rs, matching their C++ .cc definitions.
}

// C++: writing_direction_mode.h, operator==.
impl PartialEq for WritingDirectionMode {
    fn eq(&self, other: &Self) -> bool {
        self.writing_mode_ == other.writing_mode_ && self.direction_ == other.direction_
    }
}

impl Eq for WritingDirectionMode {}
