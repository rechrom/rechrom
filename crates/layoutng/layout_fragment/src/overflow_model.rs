//! Fragment-owned retained overflow storage.

#![allow(non_snake_case)]

use foundation::{LayoutUnit, PhysicalOffset, PhysicalRect, Visitor};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;

pub struct BoxScrollableOverflowModel {
    scrollable_overflow_: PhysicalRect,
}

#[allow(non_snake_case)]
impl BoxScrollableOverflowModel {
    pub fn new(overflow_rect: &PhysicalRect) -> Self {
        Self {
            scrollable_overflow_: *overflow_rect,
        }
    }

    pub fn ScrollableOverflowRect(&self) -> &PhysicalRect {
        &self.scrollable_overflow_
    }
}

pub struct BoxVisualOverflowModel {
    self_visual_overflow_: PhysicalRect,
    contents_visual_overflow_: PhysicalRect,
    has_subpixel_visual_effect_outsets_: bool,
}

#[allow(non_snake_case)]
impl BoxVisualOverflowModel {
    pub fn new(self_visual_overflow_rect: &PhysicalRect) -> Self {
        Self {
            self_visual_overflow_: *self_visual_overflow_rect,
            contents_visual_overflow_: PhysicalRect::default(),
            has_subpixel_visual_effect_outsets_: false,
        }
    }

    pub fn SetSelfVisualOverflow(&mut self, rect: &PhysicalRect) {
        self.self_visual_overflow_ = *rect;
    }

    pub fn SelfVisualOverflowRect(&self) -> &PhysicalRect {
        &self.self_visual_overflow_
    }

    pub fn AddSelfVisualOverflow(&mut self, rect: &PhysicalRect) {
        self.self_visual_overflow_.Unite(rect);
    }

    pub fn ContentsVisualOverflowRect(&self) -> &PhysicalRect {
        &self.contents_visual_overflow_
    }

    pub fn AddContentsVisualOverflow(&mut self, rect: &PhysicalRect) {
        self.contents_visual_overflow_.Unite(rect);
    }

    pub fn Move(&mut self, dx: LayoutUnit, dy: LayoutUnit) {
        let offset = PhysicalOffset::new(dx, dy);
        self.self_visual_overflow_.Move(&offset);
        self.contents_visual_overflow_.Move(&offset);
    }

    pub fn SetHasSubpixelVisualEffectOutsets(&mut self, value: bool) {
        self.has_subpixel_visual_effect_outsets_ = value;
    }
    pub fn HasSubpixelVisualEffectOutsets(&self) -> bool {
        self.has_subpixel_visual_effect_outsets_
    }
}

#[derive(Clone, Copy, Default)]
pub struct PreviousOverflowData {
    pub previous_scrollable_overflow_rect: PhysicalRect,
    pub previous_visual_overflow_rect: PhysicalRect,
    pub previous_self_visual_overflow_rect: PhysicalRect,
}

#[derive(Default)]
pub struct BoxOverflowModel {
    pub scrollable_overflow: Option<BoxScrollableOverflowModel>,
    pub visual_overflow: Option<BoxVisualOverflowModel>,
    pub previous_overflow_data: Option<PreviousOverflowData>,
}

impl BoxOverflowModel {
    pub fn Trace(&self, _visitor: &mut Visitor) {}
}
