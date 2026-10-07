// cpp: layoutng_geometry/geometry/bfc_offset.h:10-12
// Pending connection to //src/foundation:blink_geometry_api and blink_base.
use foundation::{LayoutUnit, StrCat, String};

// cpp: layoutng_geometry/geometry/bfc_offset.h:16-24
#[derive(Clone, Copy, Debug, Default)]
pub struct BfcDelta {
    pub line_offset_delta: LayoutUnit,
    pub block_offset_delta: LayoutUnit,
}

impl BfcDelta {
    pub fn new(line_offset_delta: LayoutUnit, block_offset_delta: LayoutUnit) -> Self {
        Self {
            line_offset_delta,
            block_offset_delta,
        }
    }
}

// cpp: layoutng_geometry/geometry/bfc_offset.h:26-38
/// Offset relative to a block formatting context, independent of text direction.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BfcOffset {
    pub line_offset: LayoutUnit,
    pub block_offset: LayoutUnit,
}

#[allow(non_snake_case)]
impl BfcOffset {
    pub fn new(line_offset: LayoutUnit, block_offset: LayoutUnit) -> Self {
        Self {
            line_offset,
            block_offset,
        }
    }

    // cpp: layoutng_geometry/geometry/bfc_offset.cc:11-13
    pub fn ToString(&self) -> String {
        StrCat(&[
            self.line_offset.ToString().into(),
            String::from("x"),
            self.block_offset.ToString().into(),
        ])
    }
}

// cpp: layoutng_geometry/geometry/bfc_offset.h:40-48
impl std::ops::AddAssign<BfcDelta> for BfcOffset {
    fn add_assign(&mut self, delta: BfcDelta) {
        *self = *self + delta;
    }
}

impl std::ops::Add<BfcDelta> for BfcOffset {
    type Output = Self;
    fn add(self, delta: BfcDelta) -> Self {
        Self::new(
            self.line_offset + delta.line_offset_delta,
            self.block_offset + delta.block_offset_delta,
        )
    }
}

// cpp: layoutng_geometry/geometry/bfc_offset.h:50-53
// Pairwise equality is represented by the PartialEq derive on BfcOffset.

// cpp: layoutng_geometry/geometry/bfc_offset.h:55-58
// cpp: layoutng_geometry/geometry/bfc_offset.cc:15-17
impl std::fmt::Display for BfcOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ToString())
    }
}
