// Native resource identity follows LayoutSVGResourceClipper ->
// LayoutSVGResourceContainer -> LayoutSVGHiddenContainer. Immutable clip data
// and consumer invalidation cross the existing document/Layout boundary;
// this owner does not create a second mutable copy of that resource graph.
#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};

use crate::layout_svg_hidden_container::LayoutSVGHiddenContainer;

#[repr(C)]
pub struct LayoutSVGResourceContainer {
    hidden: LayoutSVGHiddenContainer,
}
impl LayoutSVGResourceContainer {
    pub(crate) fn new(element: *mut Element) -> Self {
        Self {
            hidden: LayoutSVGHiddenContainer::new(element),
        }
    }
    pub fn IsSVGResourceContainer(&self) -> bool {
        true
    }
}
impl Deref for LayoutSVGResourceContainer {
    type Target = LayoutSVGHiddenContainer;
    fn deref(&self) -> &Self::Target {
        &self.hidden
    }
}
impl DerefMut for LayoutSVGResourceContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hidden
    }
}
impl Traceable for LayoutSVGResourceContainer {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.hidden.Trace(visitor);
    }
}
impl foundation::DowncastFrom<LayoutObject> for LayoutSVGResourceContainer {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGResourceContainer()
    }
}

#[repr(C)]
pub struct LayoutSVGResourceClipper {
    resource: LayoutSVGResourceContainer,
}
impl LayoutSVGResourceClipper {
    pub fn new(element: *mut Element) -> Self {
        let resource = LayoutSVGResourceContainer::new(element);
        resource.SetRuntimeClass(LayoutObjectClass::SvgResourceClipper);
        Self { resource }
    }
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGResourceClipper"
    }
}
impl Deref for LayoutSVGResourceClipper {
    type Target = LayoutSVGResourceContainer;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}
impl DerefMut for LayoutSVGResourceClipper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}
impl Traceable for LayoutSVGResourceClipper {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.resource.Trace(visitor);
    }
}
impl foundation::DowncastFrom<LayoutObject> for LayoutSVGResourceClipper {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.RuntimeClass() == LayoutObjectClass::SvgResourceClipper
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGResourceContainer, hidden) == 0);
const _: () = assert!(std::mem::offset_of!(LayoutSVGResourceClipper, resource) == 0);
