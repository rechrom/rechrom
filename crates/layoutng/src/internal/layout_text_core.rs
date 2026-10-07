#![allow(non_snake_case)]

use super::layout_text::LayoutText;

impl LayoutText {
    // cpp: layoutng/internal/layout_text_core.cc:5-8
    pub fn DetachAxHooks(&mut self) {
        self.CheckIsNotDestroyed();
        self.has_abstract_inline_text_box_ = false;
    }

    // cpp: layoutng/internal/layout_text_core.cc:10-12
    pub fn ClearBlockFlowCachedData(&self) {
        self.CheckIsNotDestroyed();
    }
}
