#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{DowncastFrom, Traceable, Visitor};
use layoutng_assembly::internal::layout_box_model_object::PaintLayerType;
use layoutng_assembly::internal::layout_inline::LayoutInline;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::layout_text::LayoutText;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_svg/layout_svg_inline.h:30-51
#[repr(C)]
pub struct LayoutSVGInline {
    inline: LayoutInline,
}

impl LayoutSVGInline {
    // cpp: layoutng_svg/layout_svg_inline.cc:9-11
    pub fn new(element: *mut Element) -> Self {
        let mut inline = LayoutInline::new(element);
        inline.SetRuntimeClass(LayoutObjectClass::SvgInline);
        inline.SetAlwaysCreateLineBoxes(true);
        Self { inline }
    }

    // cpp: layoutng_svg/layout_svg_inline.h:33-48
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGInline"
    }
    pub fn LayerTypeRequired(&self) -> PaintLayerType {
        PaintLayerType::kNoPaintLayer
    }
    pub fn IsSVG(&self) -> bool {
        true
    }
    pub fn IsSVGInline(&self) -> bool {
        true
    }

    // cpp: layoutng_svg/layout_svg_inline.cc:13-24
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, style: &ComputedStyle) -> bool {
        assert!(!child.is_null());
        let child_ref = unsafe { &*child };
        if child_ref.IsText() {
            return child_ref.IsSVGInlineText()
                && !unsafe { &*child.cast::<LayoutText>() }.HasEmptyText();
        }
        if !child_ref.IsSVGInline() && !child_ref.IsSVGInlineText() {
            return false;
        }
        self.IsChildAllowedBase(child, style)
    }
}

impl Deref for LayoutSVGInline {
    type Target = LayoutInline;
    fn deref(&self) -> &Self::Target {
        &self.inline
    }
}
impl DerefMut for LayoutSVGInline {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inline
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGInline, inline) == 0);
impl Traceable for LayoutSVGInline {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.inline.Trace(visitor);
    }
}
impl DowncastFrom<LayoutObject> for LayoutSVGInline {
    // cpp: layoutng_svg/layout_svg_inline.h:52-57
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGInline()
    }
}
