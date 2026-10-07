#![allow(non_snake_case)]

use foundation::{LayoutUnit, PhysicalOffset, PhysicalRect, Visitor};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;

// cpp: layoutng/internal/overflow_model.h:95-109
pub struct BoxScrollableOverflowModel {
    scrollable_overflow_: PhysicalRect,
}

#[allow(non_snake_case)]
impl BoxScrollableOverflowModel {
    // cpp: layoutng/internal/overflow_model.h:97-98
    pub fn new(overflow_rect: &PhysicalRect) -> Self {
        Self {
            scrollable_overflow_: *overflow_rect,
        }
    }

    // cpp: layoutng/internal/overflow_model.h:103-105
    pub fn ScrollableOverflowRect(&self) -> &PhysicalRect {
        &self.scrollable_overflow_
    }
}

// cpp: layoutng/internal/overflow_model.h:111-153
pub struct BoxVisualOverflowModel {
    self_visual_overflow_: PhysicalRect,
    contents_visual_overflow_: PhysicalRect,
    has_subpixel_visual_effect_outsets_: bool,
}

#[allow(non_snake_case)]
impl BoxVisualOverflowModel {
    // cpp: layoutng/internal/overflow_model.h:113-114
    pub fn new(self_visual_overflow_rect: &PhysicalRect) -> Self {
        Self {
            self_visual_overflow_: *self_visual_overflow_rect,
            contents_visual_overflow_: PhysicalRect::default(),
            has_subpixel_visual_effect_outsets_: false,
        }
    }

    // cpp: layoutng/internal/overflow_model.h:118-120
    pub fn SetSelfVisualOverflow(&mut self, rect: &PhysicalRect) {
        self.self_visual_overflow_ = *rect;
    }

    // cpp: layoutng/internal/overflow_model.h:122-124
    pub fn SelfVisualOverflowRect(&self) -> &PhysicalRect {
        &self.self_visual_overflow_
    }

    // cpp: layoutng/internal/overflow_model.h:125-127
    pub fn AddSelfVisualOverflow(&mut self, rect: &PhysicalRect) {
        self.self_visual_overflow_.Unite(rect);
    }

    // cpp: layoutng/internal/overflow_model.h:129-131
    pub fn ContentsVisualOverflowRect(&self) -> &PhysicalRect {
        &self.contents_visual_overflow_
    }

    // cpp: layoutng/internal/overflow_model.h:132-134
    pub fn AddContentsVisualOverflow(&mut self, rect: &PhysicalRect) {
        self.contents_visual_overflow_.Unite(rect);
    }

    // cpp: layoutng/internal/overflow_model.h:136-140
    pub fn Move(&mut self, dx: LayoutUnit, dy: LayoutUnit) {
        let offset = PhysicalOffset::new(dx, dy);
        self.self_visual_overflow_.Move(&offset);
        self.contents_visual_overflow_.Move(&offset);
    }

    // cpp: layoutng/internal/overflow_model.h:142-147
    pub fn SetHasSubpixelVisualEffectOutsets(&mut self, value: bool) {
        self.has_subpixel_visual_effect_outsets_ = value;
    }
    pub fn HasSubpixelVisualEffectOutsets(&self) -> bool {
        self.has_subpixel_visual_effect_outsets_
    }
}

// cpp: layoutng/internal/overflow_model.h:161-165
#[derive(Clone, Copy, Default)]
pub struct PreviousOverflowData {
    pub previous_scrollable_overflow_rect: PhysicalRect,
    pub previous_visual_overflow_rect: PhysicalRect,
    pub previous_self_visual_overflow_rect: PhysicalRect,
}

// cpp: layoutng/internal/overflow_model.h:155-169
#[derive(Default)]
pub struct BoxOverflowModel {
    pub scrollable_overflow: Option<BoxScrollableOverflowModel>,
    pub visual_overflow: Option<BoxVisualOverflowModel>,
    pub previous_overflow_data: Option<PreviousOverflowData>,
}

impl BoxOverflowModel {
    // cpp: layoutng/internal/overflow_model.h:168
    pub fn Trace(&self, _visitor: &mut Visitor) {}
}
