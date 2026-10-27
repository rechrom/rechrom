#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::layout_text::LayoutText;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::layout_svg_inline::LayoutSVGInline;

// cpp: layoutng_svg/layout_svg_tspan.h:28-42
#[repr(C)]
pub struct LayoutSVGTSpan {
    inline: LayoutSVGInline,
}

impl LayoutSVGTSpan {
    // cpp: layoutng_svg/layout_svg_tspan.cc:29-29
    pub fn new(element: *mut Element) -> Self {
        let inline = LayoutSVGInline::new(element);
        inline.SetRuntimeClass(LayoutObjectClass::SvgTSpan);
        Self { inline }
    }

    // cpp: layoutng_svg/layout_svg_tspan.h:32-40
    pub fn IsSVGTSpan(&self) -> bool {
        true
    }
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGTSpan"
    }

    // cpp: layoutng_svg/layout_svg_tspan.cc:31-41
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, _style: &ComputedStyle) -> bool {
        assert!(!child.is_null());
        let child_ref = unsafe { &*child };
        if child_ref.IsText() {
            return child_ref.IsSVGInlineText()
                && !unsafe { &*child.cast::<LayoutText>() }.HasEmptyText();
        }
        child_ref.IsSVGInline() && !child_ref.IsSVGTextPath()
    }
}

impl Deref for LayoutSVGTSpan {
    type Target = LayoutSVGInline;
    fn deref(&self) -> &Self::Target {
        &self.inline
    }
}
impl DerefMut for LayoutSVGTSpan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inline
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGTSpan, inline) == 0);
impl Traceable for LayoutSVGTSpan {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.inline.Trace(visitor);
    }
}
