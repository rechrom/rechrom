#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use layoutng_geometry::geometry::logical_size::LogicalSize;

// cpp: layoutng_svg/layout_svg_shape.h:10-22
#[repr(C)]
pub struct LayoutSVGShape {
    flow: LayoutBlockFlow,
}

impl LayoutSVGShape {
    // cpp: layoutng_svg/layout_svg_shape.cc:10-10
    pub fn new(element: *mut Element) -> Self {
        let flow = LayoutBlockFlow::new(element.cast());
        flow.SetRuntimeClass(LayoutObjectClass::SvgShape);
        Self { flow }
    }

    // cpp: layoutng_svg/layout_svg_shape.h:14-14
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGShape"
    }

    // cpp: layoutng_svg/layout_svg_shape.h:15-15
    pub fn CreatesNewFormattingContext(&self) -> bool {
        true
    }

    // cpp: layoutng_svg/layout_svg_shape.h:16-16
    pub fn CanHaveChildren(&self) -> bool {
        false
    }

    // cpp: layoutng_svg/layout_svg_shape.h:20-21
    pub fn IsSVG(&self) -> bool {
        true
    }

    pub fn IsSVGShape(&self) -> bool {
        true
    }

    // cpp: layoutng_svg/layout_svg_shape.cc:12-19
    pub fn UpdateSVGLayout(&mut self, _info: &SVGLayoutInfo) -> SVGLayoutResult {
        let style = self.StyleRef();
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            style.GetWritingMode(),
            style.GetWritingDirection(),
            true,
            true,
            false,
        );
        builder.SetAvailableSize(LogicalSize::default());
        let space = builder.ToConstraintSpace();
        BlockNode::new((self as *mut Self).cast()).Layout(
            &space,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        SVGLayoutResult::new(true, true)
    }
}

impl Deref for LayoutSVGShape {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

impl DerefMut for LayoutSVGShape {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutSVGShape, flow) == 0);

impl Traceable for LayoutSVGShape {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}

// cpp: layoutng_svg/layout_svg_shape.h:24-29
impl foundation::DowncastFrom<LayoutObject> for LayoutSVGShape {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGShape()
    }
}
