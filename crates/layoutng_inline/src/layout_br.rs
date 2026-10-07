// C++: layoutng_inline/layout_br.h/.cc.
// LayoutText is the first field, preserving the C++ public base at offset zero.
#![allow(non_snake_case)]

use foundation::{String, Traceable, Visitor};
use layoutng::internal::layout_node_metadata::{Element, Node};
use layoutng::internal::layout_text::LayoutText;
use std::ops::{Deref, DerefMut};

// cpp: layoutng_inline/layout_br.h:14-27
#[repr(C)]
pub struct LayoutBR {
    text_: LayoutText,
}

const _: () = assert!(std::mem::offset_of!(LayoutBR, text_) == 0);

impl Deref for LayoutBR {
    type Target = LayoutText;
    fn deref(&self) -> &LayoutText {
        &self.text_
    }
}
impl DerefMut for LayoutBR {
    fn deref_mut(&mut self) -> &mut LayoutText {
        &mut self.text_
    }
}

#[allow(non_snake_case)]
impl LayoutBR {
    // cpp: layoutng_inline/layout_br.h:16
    // cpp: layoutng_inline/layout_br.cc:9-9
    pub fn new(node: &mut Element) -> Self {
        Self {
            text_: LayoutText::new(node as *mut Element as *mut Node, String::from("\n")),
        }
    }

    // cpp: layoutng_inline/layout_br.h:18-21
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutBR"
    }

    // cpp: layoutng_inline/layout_br.h:23-26
    pub fn IsBR(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
}

impl Traceable for LayoutBR {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.text_.Trace(visitor);
    }
}
