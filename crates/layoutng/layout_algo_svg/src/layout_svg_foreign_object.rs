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
    pub fn UpdateSVGLayout(
        &mut self,
        _info: &layoutng_assembly::internal::svg_layout_info::SVGLayoutInfo,
    ) -> layoutng_assembly::internal::svg_layout_info::SVGLayoutResult {
        use layoutng_assembly::internal::{
            block_node::BlockNode, constraint_space_builder::ConstraintSpaceBuilder,
        };
        let style = self.StyleRef();
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            style.GetWritingMode(),
            style.GetWritingDirection(),
            true,
            true,
            false,
        );
        builder.SetAvailableSize(layoutng_geometry::geometry::logical_size::LogicalSize::default());
        BlockNode::new((self as *mut Self).cast()).Layout(
            &builder.ToConstraintSpace(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        layoutng_assembly::internal::svg_layout_info::SVGLayoutResult::new(true, true)
    }

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
