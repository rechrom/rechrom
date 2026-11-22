//! LayoutObject fragment-gap dispatch surface.
//!
//! The generic entry remains inlineable so profiles retain the concrete flex
//! or grid algorithm frame rather than stopping at a shared fragment wrapper.

use super::*;

impl LayoutObject {
    pub fn StitchedRowGapIndexBase(
        &self,
        _fragment: &PhysicalBoxFragment,
        gap_index: usize,
        _line_index: Option<usize>,
    ) -> usize {
        self.CheckIsNotDestroyed();
        gap_index
    }

    pub fn StitchedRowGapIndex(
        &self,
        fragment: &PhysicalBoxFragment,
        gap_index: usize,
        line_index: Option<usize>,
    ) -> usize {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectStitchedRowGapIndex(self, fragment, gap_index, line_index) }
    }
}
