use foundation::LayoutUnit;

// cpp: layoutng/internal/exclusions/line_layout_opportunity.h:15-71
#[derive(Clone, Copy, Debug, Default)]
pub struct LineLayoutOpportunity {
    pub line_left_offset: LayoutUnit,
    pub line_right_offset: LayoutUnit,
    pub float_line_left_offset: LayoutUnit,
    pub float_line_right_offset: LayoutUnit,
    pub bfc_block_offset: LayoutUnit,
    pub line_block_size: LayoutUnit,
}

#[allow(non_snake_case)]
impl LineLayoutOpportunity {
    // cpp: layoutng/internal/exclusions/line_layout_opportunity.h:22-23
    pub fn with_inline_size(inline_size: LayoutUnit) -> Self {
        Self {
            line_right_offset: inline_size,
            float_line_right_offset: inline_size,
            ..Self::default()
        }
    }

    // cpp: layoutng/internal/exclusions/line_layout_opportunity.h:24-35
    pub fn new(
        line_left_offset: LayoutUnit,
        line_right_offset: LayoutUnit,
        float_line_left_offset: LayoutUnit,
        float_line_right_offset: LayoutUnit,
        bfc_block_offset: LayoutUnit,
        line_block_size: LayoutUnit,
    ) -> Self {
        Self {
            line_left_offset,
            line_right_offset,
            float_line_left_offset,
            float_line_right_offset,
            bfc_block_offset,
            line_block_size,
        }
    }

    // cpp: layoutng/internal/exclusions/line_layout_opportunity.h:55-58
    pub fn AvailableInlineSize(&self) -> LayoutUnit {
        debug_assert!(self.line_right_offset >= self.line_left_offset);
        self.line_right_offset - self.line_left_offset
    }

    // cpp: layoutng/internal/exclusions/line_layout_opportunity.h:60-63
    pub fn AvailableFloatInlineSize(&self) -> LayoutUnit {
        debug_assert!(self.float_line_right_offset >= self.float_line_left_offset);
        self.float_line_right_offset - self.float_line_left_offset
    }

    // cpp: layoutng/internal/exclusions/line_layout_opportunity.h:65-70
    pub fn IsEqualToAvailableFloatInlineSize(&self, inline_size: LayoutUnit) -> bool {
        debug_assert!(self.float_line_right_offset >= self.float_line_left_offset);
        self.float_line_left_offset + inline_size == self.float_line_right_offset
    }
}
