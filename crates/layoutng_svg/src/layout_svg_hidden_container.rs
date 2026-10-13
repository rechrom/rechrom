// Source: core/layout/svg/layout_svg_hidden_container.{h,cc}, Chromium
// 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};

use crate::layout_svg_group::LayoutSVGGroup;

/// Non-rendered SVG content still owns styled, laid-out descendants. The
/// group base supplies the existing SVG child list and fragment storage ABI;
/// its transformable-container and paint semantics are not inherited.
#[repr(C)]
pub struct LayoutSVGHiddenContainer {
    container: LayoutSVGGroup,
}

impl LayoutSVGHiddenContainer {
    pub fn new(element: *mut Element) -> Self {
        let container = LayoutSVGGroup::new(element);
        container.SetRuntimeClass(LayoutObjectClass::SvgHiddenContainer);
        Self { container }
    }

    pub fn GetName(&self) -> &'static str {
        "LayoutSVGHiddenContainer"
    }
    pub fn IsSVGHiddenContainer(&self) -> bool {
        true
    }
    pub fn IsSVGTransformableContainer(&self) -> bool {
        false
    }

    pub fn UpdateSVGLayout(&mut self, info: &SVGLayoutInfo) -> SVGLayoutResult {
        let child_info = SVGLayoutInfo {
            force_layout: self.SelfNeedsFullLayout(),
            ..*info
        };
        let result = crate::svg_container_layout::LayoutContainer(self, &child_info, 0.0, 0.0);
        // The represented hidden owners, defs and clipPath, have no own SVG
        // relative-length attributes. Their descendants retain dependence.
        SVGLayoutResult::new(false, result.has_viewport_dependence)
    }
}

impl Deref for LayoutSVGHiddenContainer {
    type Target = LayoutSVGGroup;
    fn deref(&self) -> &Self::Target {
        &self.container
    }
}
impl DerefMut for LayoutSVGHiddenContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}
impl Traceable for LayoutSVGHiddenContainer {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.container.Trace(visitor);
    }
}
impl foundation::DowncastFrom<LayoutObject> for LayoutSVGHiddenContainer {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGHiddenContainer()
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGHiddenContainer, container) == 0);
