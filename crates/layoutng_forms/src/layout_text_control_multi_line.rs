#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::LayoutObjectClass;

// cpp: layoutng_forms/layout_text_control_multi_line.h:12-13
#[repr(C)]
pub struct LayoutTextControlMultiLine {
    flow: LayoutBlockFlow,
}

impl LayoutTextControlMultiLine {
    // cpp: layoutng_forms/layout_text_control_multi_line.h:15-15
    // cpp: layoutng_forms/layout_text_control_multi_line.cc:11-12
    pub fn new(element: *mut Element) -> Self {
        let flow = LayoutBlockFlow::new(element.cast());
        flow.SetRuntimeClass(LayoutObjectClass::TextControlMultiLine);
        Self { flow }
    }

    // cpp: layoutng_forms/layout_text_control_multi_line.h:18-21
    pub fn IsTextArea(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng_forms/layout_text_control_multi_line.h:23-26
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutTextControlMultiLine"
    }

    // cpp: layoutng_forms/layout_text_control_multi_line.h:28-31
    pub fn CreatesNewFormattingContext(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
}

impl Deref for LayoutTextControlMultiLine {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

impl DerefMut for LayoutTextControlMultiLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutTextControlMultiLine, flow) == 0);

impl Traceable for LayoutTextControlMultiLine {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}
