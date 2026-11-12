//! Resource identity/lifecycle for LayoutSVGResourcePaintServer and its native
//! gradient/pattern specializations. Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Source: layout_svg_resource_paint_server.h:35-52; gradient.cc:74-100,174-180;
//! pattern.cc:49-79,115-124; resource_container.cc:73-92,276-299.
use crate::layout_svg_resource_clipper::LayoutSVGResourceContainer;
use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::{
    layout_node_metadata::Element,
    layout_object::{LayoutObject, LayoutObjectClass},
};
use layoutng_style::style::computed_style::ComputedStyle;
use std::ops::{Deref, DerefMut};
#[repr(C)]
pub struct LayoutSVGResourcePaintServer {
    resource: LayoutSVGResourceContainer,
    pattern: bool,
}
impl LayoutSVGResourcePaintServer {
    pub fn new(element: *mut Element, pattern: bool) -> Self {
        let resource = LayoutSVGResourceContainer::new(element);
        resource.SetRuntimeClass(LayoutObjectClass::SvgResourcePaintServer);
        Self { resource, pattern }
    }
    pub fn IsPattern(&self) -> bool {
        self.pattern
    }
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, style: &ComputedStyle) -> bool {
        if self.pattern {
            return !child.is_null()
                && unsafe { &*child }.IsSVG()
                && !unsafe { &*child }.IsSVGInline()
                && !unsafe { &*child }.IsSVGInlineText()
                && !unsafe { &*child }.IsSVGRoot();
        }
        // Gradient resources allow nested paint-server resources, not stops or
        // ordinary graphics LayoutObjects (gradient.cc:174-180).
        let _ = style;
        unsafe { child.as_ref() }
            .is_some_and(|child| child.RuntimeClass() == LayoutObjectClass::SvgResourcePaintServer)
    }
    pub fn GetName(&self) -> &'static str {
        if self.pattern {
            "LayoutSVGResourcePattern"
        } else {
            "LayoutSVGResourceGradient"
        }
    }
}
impl Deref for LayoutSVGResourcePaintServer {
    type Target = LayoutSVGResourceContainer;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}
impl DerefMut for LayoutSVGResourcePaintServer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}
impl Traceable for LayoutSVGResourcePaintServer {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.resource.Trace(visitor)
    }
}
