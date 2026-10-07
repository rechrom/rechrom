// C++: layoutng_inline/layout_word_break.h/.cc.
// The owned empty String is distinct from a null String, as in StringImpl::empty_.
#![allow(non_snake_case)]

use foundation::{String, Traceable, Visitor};
use layoutng::internal::layout_node_metadata::{Element, Node};
use layoutng::internal::layout_text::LayoutText;
use std::ops::{Deref, DerefMut};

// cpp: layoutng_inline/layout_word_break.h:36-45
#[repr(C)]
pub struct LayoutWordBreak {
    text_: LayoutText,
}

const _: () = assert!(std::mem::offset_of!(LayoutWordBreak, text_) == 0);

impl Deref for LayoutWordBreak {
    type Target = LayoutText;
    fn deref(&self) -> &LayoutText {
        &self.text_
    }
}
impl DerefMut for LayoutWordBreak {
    fn deref_mut(&mut self) -> &mut LayoutText {
        &mut self.text_
    }
}

#[allow(non_snake_case)]
impl LayoutWordBreak {
    // cpp: layoutng_inline/layout_word_break.h:38
    // cpp: layoutng_inline/layout_word_break.cc:33-34
    pub fn new(node: &mut Element) -> Self {
        Self {
            text_: LayoutText::new(node as *mut Element as *mut Node, String::from("")),
        }
    }

    // cpp: layoutng_inline/layout_word_break.h:40-43
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutWordBreak"
    }

    // cpp: layoutng_inline/layout_word_break.cc:36-39
    pub fn IsWordBreak(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
}

impl Traceable for LayoutWordBreak {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.text_.Trace(visitor);
    }
}
