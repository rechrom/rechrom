//! Form-control query view and the `LayoutMutation::FormControl` command
//! values.
//!
//! Commands (caps-lock indicator paint, inner-control modified style) cross
//! boundary 3 as `LayoutMutation::FormControl(FormControlMutation::..)`.
//! `ScrollSelectOptionIntoView` returns the scroll result Page applies, so it
//! is a borrowed view method instead.

pub use layout_assembly::layout_engine::FormControlMutation;
use layout_assembly::layout_engine::LayoutEngine as CoreLayoutEngine;
use layout_assembly::engine::scroll::scroll_source::NativeScrollResult;

use crate::LayoutEngine;

/// Borrowed form-control routes over the resident LayoutObject tree.
pub struct FormControlQueries<'a> {
    engine: &'a mut CoreLayoutEngine,
}

impl LayoutEngine {
    #[inline]
    pub fn FormControl(&mut self) -> FormControlQueries<'_> {
        FormControlQueries {
            engine: &mut self.engine,
        }
    }
}

impl FormControlQueries<'_> {
    /// ListBoxSelectType::ScrollToOptionTask, select_type.cc:1720-1754.
    #[inline]
    pub fn ScrollSelectOptionIntoView(&mut self, container: u64, option: u64) -> NativeScrollResult {
        self.engine.ScrollSelectOptionIntoView(container, option)
    }
}
