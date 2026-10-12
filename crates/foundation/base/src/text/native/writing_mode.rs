// C++: src/foundation/text/native/writing_mode.h

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WritingMode {
    kHorizontalTb = 0,
    kVerticalRl = 1,
    kVerticalLr = 2,
    kSidewaysRl = 3,
    kSidewaysLr = 4,
}

impl WritingMode {
    // C++: writing_mode.h, kMaxWritingMode = kSidewaysLr.
    pub const kMaxWritingMode: Self = Self::kSidewaysLr;
}

// C++: writing_mode.h, IsHorizontalWritingMode.
pub fn IsHorizontalWritingMode(writing_mode: WritingMode) -> bool {
    writing_mode == WritingMode::kHorizontalTb
}

// C++: writing_mode.h, IsVerticalWritingMode.
pub fn IsVerticalWritingMode(writing_mode: WritingMode) -> bool {
    writing_mode == WritingMode::kVerticalLr || writing_mode == WritingMode::kVerticalRl
}

// C++: writing_mode.h, IsFlippedLinesWritingMode.
pub fn IsFlippedLinesWritingMode(writing_mode: WritingMode) -> bool {
    writing_mode == WritingMode::kVerticalLr
}

// C++: writing_mode.h, ToLineWritingMode.
pub fn ToLineWritingMode(writing_mode: WritingMode) -> WritingMode {
    if !IsFlippedLinesWritingMode(writing_mode) {
        writing_mode
    } else {
        WritingMode::kVerticalRl
    }
}

// C++: writing_mode.h, IsFlippedBlocksWritingMode.
pub fn IsFlippedBlocksWritingMode(writing_mode: WritingMode) -> bool {
    writing_mode == WritingMode::kVerticalRl || writing_mode == WritingMode::kSidewaysRl
}

// C++: writing_mode.h, IsParallelWritingMode.
pub fn IsParallelWritingMode(a: WritingMode, b: WritingMode) -> bool {
    (a == WritingMode::kHorizontalTb) == (b == WritingMode::kHorizontalTb)
}

// C++: writing_mode.h, IsHorizontalTypographicMode.
pub fn IsHorizontalTypographicMode(writing_mode: WritingMode) -> bool {
    writing_mode == WritingMode::kHorizontalTb
        || writing_mode == WritingMode::kSidewaysLr
        || writing_mode == WritingMode::kSidewaysRl
}
