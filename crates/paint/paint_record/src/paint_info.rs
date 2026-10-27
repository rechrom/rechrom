#![allow(non_snake_case)]

use std::cell::RefCell;

use crate::paint_context::PaintContext;
use crate::paint_engine::PaintPhase;

// cpp: paint/paint_info.h:7-24
#[derive(Clone, Copy)]
pub struct PaintInfo<'c, 'o> {
    pub context: &'c RefCell<PaintContext<'o>>,
    pub phase: PaintPhase,
    pub descendant_painting_blocked: bool,
}

impl<'c, 'o> PaintInfo<'c, 'o> {
    pub fn new(context: &'c RefCell<PaintContext<'o>>, phase: PaintPhase) -> Self {
        Self {
            context,
            phase,
            descendant_painting_blocked: false,
        }
    }

    // cpp: paint/paint_info.h:15-22
    pub fn ForDescendants(&self) -> Self {
        let mut result = *self;
        if result.phase == PaintPhase::kDescendantOutlinesOnly {
            result.phase = PaintPhase::kOutline;
        } else if result.phase == PaintPhase::kDescendantBlockBackgroundsOnly {
            result.phase = PaintPhase::kBlockBackground;
        }
        result
    }
}
