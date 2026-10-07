use foundation::{LayoutUnit, WritingDirectionMode};
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize};

use crate::physical_fragment::PhysicalFragment;

// A borrowed physical fragment preserves the C++ reference lifetime. The
// STACK_ALLOCATED macro has no direct Rust equivalent.
// cpp: layoutng_fragment_tree/logical_fragment.h:18-19
// cpp: layoutng_fragment_tree/logical_fragment.h:45-46
pub struct LogicalFragment<'a> {
    physical_fragment_: &'a PhysicalFragment,
    writing_direction_: WritingDirectionMode,
}

#[allow(non_snake_case)]
impl<'a> LogicalFragment<'a> {
    // cpp: layoutng_fragment_tree/logical_fragment.h:22-25
    pub fn new(
        writing_direction: WritingDirectionMode,
        physical_fragment: &'a PhysicalFragment,
    ) -> Self {
        Self {
            physical_fragment_: physical_fragment,
            writing_direction_: writing_direction,
        }
    }

    // cpp: layoutng_fragment_tree/logical_fragment.h:28-31
    pub fn InlineSize(&self) -> LayoutUnit {
        if self.writing_direction_.IsHorizontal() {
            self.physical_fragment_.Size().width
        } else {
            self.physical_fragment_.Size().height
        }
    }

    // cpp: layoutng_fragment_tree/logical_fragment.h:32-35
    pub fn BlockSize(&self) -> LayoutUnit {
        if self.writing_direction_.IsHorizontal() {
            self.physical_fragment_.Size().height
        } else {
            self.physical_fragment_.Size().width
        }
    }

    // cpp: layoutng_fragment_tree/logical_fragment.h:36-39
    pub fn Size(&self) -> LogicalSize {
        ToLogicalSize(
            self.physical_fragment_.Size(),
            self.writing_direction_.GetWritingMode(),
        )
    }

    // cpp: layoutng_fragment_tree/logical_fragment.h:40-42
    pub fn GetWritingDirection(&self) -> WritingDirectionMode {
        self.writing_direction_
    }
}
