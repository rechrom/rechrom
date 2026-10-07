use foundation::{LayoutUnit, PhysicalRect, Visitor};

// cpp: layoutng/internal/devtools_flex_info.h:14-27
pub struct DevtoolsFlexInfo {
    pub lines: Vec<DevtoolsFlexInfoLine>,
}

// cpp: layoutng/internal/devtools_flex_info.h:15-20
pub struct DevtoolsFlexInfoItem {
    pub rect: PhysicalRect,
    pub baseline: LayoutUnit,
}

impl DevtoolsFlexInfoItem {
    pub fn new(rect: PhysicalRect, baseline: LayoutUnit) -> Self {
        Self { rect, baseline }
    }
}

// cpp: layoutng/internal/devtools_flex_info.h:21-23
pub struct DevtoolsFlexInfoLine {
    pub items: Vec<DevtoolsFlexInfoItem>,
}

#[allow(non_snake_case)]
impl DevtoolsFlexInfo {
    // cpp: layoutng/internal/devtools_flex_info.h:26-26
    pub fn Trace(&self, _visitor: &mut Visitor) {}
}
