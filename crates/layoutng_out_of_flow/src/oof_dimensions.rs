#![allow(non_snake_case)]

use foundation::{kIndefiniteSize, LayoutUnit};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_size::LogicalSize;

// cpp: layoutng_out_of_flow/oof_dimensions.h:12-13,27-30
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogicalOofDimensions {
    pub inset: BoxStrut,
    pub size: LogicalSize,
    pub margins: BoxStrut,
}

impl Default for LogicalOofDimensions {
    fn default() -> Self {
        Self {
            inset: BoxStrut::default(),
            size: LogicalSize::new(kIndefiniteSize, kIndefiniteSize),
            margins: BoxStrut::default(),
        }
    }
}

impl LogicalOofDimensions {
    // cpp: layoutng_out_of_flow/oof_dimensions.h:14-16
    pub fn MarginBoxInlineStart(&self) -> LayoutUnit {
        self.inset.inline_start - self.margins.inline_start
    }

    // cpp: layoutng_out_of_flow/oof_dimensions.h:17-19
    pub fn MarginBoxBlockStart(&self) -> LayoutUnit {
        self.inset.block_start - self.margins.block_start
    }

    // cpp: layoutng_out_of_flow/oof_dimensions.h:20-22
    pub fn MarginBoxInlineEnd(&self) -> LayoutUnit {
        self.inset.inline_start + self.size.inline_size + self.margins.inline_end
    }

    // cpp: layoutng_out_of_flow/oof_dimensions.h:23-25
    pub fn MarginBoxBlockEnd(&self) -> LayoutUnit {
        self.inset.block_start + self.size.block_size + self.margins.block_end
    }
}
