// cpp: layoutng_geometry/geometry/scroll_offset_range.h:10-11
// Pending connection to //src/foundation:blink_geometry_api and text_values_api.
use foundation::{LayoutUnit, PhysicalOffset, WritingDirectionMode, WritingMode};

// cpp: layoutng_geometry/geometry/scroll_offset_range.h:15-22
/// Missing bounds represent unbounded scroll translation in that direction.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PhysicalScrollRange {
    pub x_min: Option<LayoutUnit>,
    pub x_max: Option<LayoutUnit>,
    pub y_min: Option<LayoutUnit>,
    pub y_max: Option<LayoutUnit>,
}

#[allow(non_snake_case)]
impl PhysicalScrollRange {
    // cpp: layoutng_geometry/geometry/scroll_offset_range.h:23-27
    pub fn Contains(&self, offset: &PhysicalOffset) -> bool {
        self.x_min.map_or(true, |min| offset.left >= min)
            && self.x_max.map_or(true, |max| offset.left <= max)
            && self.y_min.map_or(true, |min| offset.top >= min)
            && self.y_max.map_or(true, |max| offset.top <= max)
    }

    // cpp: layoutng_geometry/geometry/scroll_offset_range.h:29-42
    pub fn Move(&mut self, offset: &PhysicalOffset) {
        if let Some(x_min) = &mut self.x_min {
            *x_min += offset.left;
        }
        if let Some(x_max) = &mut self.x_max {
            *x_max += offset.left;
        }
        if let Some(y_min) = &mut self.y_min {
            *y_min += offset.top;
        }
        if let Some(y_max) = &mut self.y_max {
            *y_max += offset.top;
        }
    }
}

// cpp: layoutng_geometry/geometry/scroll_offset_range.h:44-48
// Pairwise equality is represented by PartialEq above.

// cpp: layoutng_geometry/geometry/scroll_offset_range.h:50-55
#[derive(Clone, Copy, Debug, Default)]
pub struct LogicalScrollRange {
    pub inline_min: Option<LayoutUnit>,
    pub inline_max: Option<LayoutUnit>,
    pub block_min: Option<LayoutUnit>,
    pub block_max: Option<LayoutUnit>,
}

#[allow(non_snake_case)]
impl LogicalScrollRange {
    // cpp: layoutng_geometry/geometry/scroll_offset_range.h:57-62
    pub fn ToPhysical(&self, mode: WritingDirectionMode) -> PhysicalScrollRange {
        if mode.IsHorizontalLtr() {
            return PhysicalScrollRange {
                x_min: self.inline_min,
                x_max: self.inline_max,
                y_min: self.block_min,
                y_max: self.block_max,
            };
        }
        self.SlowToPhysical(mode)
    }

    // cpp: layoutng_geometry/geometry/scroll_offset_range.h:64-66
    // cpp: layoutng_geometry/geometry/scroll_offset_range.cc:17-46
    fn SlowToPhysical(&self, mode: WritingDirectionMode) -> PhysicalScrollRange {
        match mode.GetWritingMode() {
            WritingMode::kHorizontalTb => {
                debug_assert!(!mode.IsLtr());
                PhysicalScrollRange {
                    x_min: Negate(self.inline_max),
                    x_max: Negate(self.inline_min),
                    y_min: self.block_min,
                    y_max: self.block_max,
                }
            }
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                if mode.IsLtr() {
                    return PhysicalScrollRange {
                        x_min: Negate(self.block_max),
                        x_max: Negate(self.block_min),
                        y_min: self.inline_min,
                        y_max: self.inline_max,
                    };
                }
                PhysicalScrollRange {
                    x_min: Negate(self.block_max),
                    x_max: Negate(self.block_min),
                    y_min: Negate(self.inline_max),
                    y_max: Negate(self.inline_min),
                }
            }
            WritingMode::kVerticalLr => {
                if mode.IsLtr() {
                    return PhysicalScrollRange {
                        x_min: self.block_min,
                        x_max: self.block_max,
                        y_min: self.inline_min,
                        y_max: self.inline_max,
                    };
                }
                PhysicalScrollRange {
                    x_min: self.block_min,
                    x_max: self.block_max,
                    y_min: Negate(self.inline_max),
                    y_max: Negate(self.inline_min),
                }
            }
            WritingMode::kSidewaysLr => {
                if mode.IsLtr() {
                    return PhysicalScrollRange {
                        x_min: self.block_min,
                        x_max: self.block_max,
                        y_min: Negate(self.inline_max),
                        y_max: Negate(self.inline_min),
                    };
                }
                PhysicalScrollRange {
                    x_min: self.block_min,
                    x_max: self.block_max,
                    y_min: self.inline_min,
                    y_max: self.inline_max,
                }
            }
        }
    }
}

// cpp: layoutng_geometry/geometry/scroll_offset_range.cc:9-15
#[allow(non_snake_case)]
fn Negate(bound: Option<LayoutUnit>) -> Option<LayoutUnit> {
    bound.map(|value| -value)
}
