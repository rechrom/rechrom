//! Hidden SVG mask resource; publication stays document-owned.
#![allow(non_snake_case)]
use crate::layout_svg_resource_clipper::LayoutSVGResourceContainer;
use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::{
    layout_node_metadata::Element,
    layout_object::{LayoutObject, LayoutObjectClass},
};
use std::ops::{Deref, DerefMut};
#[repr(C)]
pub struct LayoutSVGResourceMasker {
    resource: LayoutSVGResourceContainer,
}
impl LayoutSVGResourceMasker {
    pub fn new(element: *mut Element) -> Self {
        let resource = LayoutSVGResourceContainer::new(element);
        resource.SetRuntimeClass(LayoutObjectClass::SvgResourceMasker);
        Self { resource }
    }
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGResourceMasker"
    }
}
impl Deref for LayoutSVGResourceMasker {
    type Target = LayoutSVGResourceContainer;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}
impl DerefMut for LayoutSVGResourceMasker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}
impl Traceable for LayoutSVGResourceMasker {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.resource.Trace(visitor);
    }
}
impl foundation::DowncastFrom<LayoutObject> for LayoutSVGResourceMasker {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.RuntimeClass() == LayoutObjectClass::SvgResourceMasker
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGResourceMasker, resource) == 0);
