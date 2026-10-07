// cpp: layoutng_geometry/geometry/bfc_rect.h:8-13
// Pending connection to //src/foundation:blink_geometry_api and blink_base.
use super::bfc_offset::BfcOffset;
use foundation::{LayoutUnit, String, StringBuilder};

// cpp: layoutng_geometry/geometry/bfc_rect.h:17-24
/// Start and end offsets of a rectangle in a block formatting context.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BfcRect {
    pub start_offset: BfcOffset,
    pub end_offset: BfcOffset,
}

#[allow(non_snake_case)]
impl BfcRect {
    pub fn new(start_offset: BfcOffset, end_offset: BfcOffset) -> Self {
        debug_assert!(end_offset.line_offset >= start_offset.line_offset);
        debug_assert!(end_offset.block_offset >= start_offset.block_offset);
        Self {
            start_offset,
            end_offset,
        }
    }

    // cpp: layoutng_geometry/geometry/bfc_rect.h:26-29
    pub fn LineStartOffset(&self) -> LayoutUnit {
        self.start_offset.line_offset
    }
    pub fn LineEndOffset(&self) -> LayoutUnit {
        self.end_offset.line_offset
    }
    pub fn BlockStartOffset(&self) -> LayoutUnit {
        self.start_offset.block_offset
    }
    pub fn BlockEndOffset(&self) -> LayoutUnit {
        self.end_offset.block_offset
    }

    // cpp: layoutng_geometry/geometry/bfc_rect.h:31-36
    pub fn BlockSize(&self) -> LayoutUnit {
        if self.end_offset.block_offset == LayoutUnit::Max() {
            return LayoutUnit::Max();
        }
        self.end_offset.block_offset - self.start_offset.block_offset
    }

    // cpp: layoutng_geometry/geometry/bfc_rect.h:37-44
    pub fn InlineSize(&self) -> LayoutUnit {
        if self.end_offset.line_offset == LayoutUnit::Max() {
            return if self.start_offset.line_offset == LayoutUnit::Max() {
                LayoutUnit::default()
            } else {
                LayoutUnit::Max()
            };
        }
        self.end_offset.line_offset - self.start_offset.line_offset
    }

    // cpp: layoutng_geometry/geometry/bfc_rect.cc:11-17
    pub fn ToString(&self) -> String {
        let mut builder = StringBuilder::new();
        builder.Append(self.start_offset.ToString());
        builder.Append('+');
        builder.Append(self.end_offset.ToString());
        builder.ToString()
    }
}

// cpp: layoutng_geometry/geometry/bfc_rect.h:46-56
// Equality is derived above; ostream insertion maps to fmt::Display.
// cpp: layoutng_geometry/geometry/bfc_rect.cc:19-21
impl std::fmt::Display for BfcRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ToString())
    }
}
