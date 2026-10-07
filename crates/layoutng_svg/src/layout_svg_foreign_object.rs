#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_style::style::computed_style::ComputedStyle;

use crate::layout_svg_group::LayoutSVGGroup;

// cpp: layoutng_svg/layout_svg_foreign_object.h:11-20
#[repr(C)]
pub struct LayoutSVGForeignObject {
    group: LayoutSVGGroup,
}

impl LayoutSVGForeignObject {
    // cpp: layoutng_svg/layout_svg_foreign_object.cc:5-6
    pub fn new(element: *mut Element) -> Self {
        let group = LayoutSVGGroup::new(element);
        group.SetRuntimeClass(LayoutObjectClass::SvgForeignObject);
        Self { group }
    }

    // cpp: layoutng_svg/layout_svg_foreign_object.h:15-15
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGForeignObject"
    }

    // cpp: layoutng_svg/layout_svg_foreign_object.cc:8-14
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, _style: &ComputedStyle) -> bool {
        !child.is_null() && !unsafe { &*child }.IsSVGChild()
    }

    // cpp: layoutng_svg/layout_svg_foreign_object.h:19-19
    pub fn IsSVGForeignObject(&self) -> bool {
        true
    }
}

impl Deref for LayoutSVGForeignObject {
    type Target = LayoutSVGGroup;
    fn deref(&self) -> &Self::Target {
        &self.group
    }
}
impl DerefMut for LayoutSVGForeignObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGForeignObject, group) == 0);
impl Traceable for LayoutSVGForeignObject {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.group.Trace(visitor);
    }
}
