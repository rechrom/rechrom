// Chromium resource identity follows LayoutSVGResourceClipper ->
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
    // SVGResource::ClientEntry's cycle state remains with the native resource
    // identity. Document resource revision invalidates these cached results.
    pub(crate) client_cycle_cache: std::cell::RefCell<crate::svg_resources::ResourceCycleCache>,
}
impl LayoutSVGResourceContainer {
    pub(crate) fn new(element: *mut Element) -> Self {
        Self {
            hidden: LayoutSVGHiddenContainer::new(element),
            client_cycle_cache: Default::default(),
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
    /// core/layout/svg/layout_svg_resource_clipper.cc:230-235. Read the
    /// live element metadata, never a synthesized clip graph.
    pub fn UsesObjectBoundingBoxUnits(&self) -> bool {
        let element = foundation::DynamicTo::<Element>(self.GetNode());
        unsafe { element.as_ref() }
            .and_then(|e| e.InputElementData().as_ref())
            .and_then(|d| d.svg_clip_path_object_bounding_box)
            .expect("clipper requires SVGClipPathElement metadata")
    }
    pub fn ElementTransformIncludingMotion(
        &self,
    ) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform {
        crate::svg_viewport::LocalTransform(self)
    }
    /// core/layout/svg/layout_svg_resource_clipper.cc:237-247.
    pub fn CalculateClipTransform(
        &self,
        reference: foundation::gfx::RectF,
    ) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform {
        let mut transform = self.ElementTransformIncludingMotion();
        if self.UsesObjectBoundingBoxUnits() {
            transform.Translate(reference.x() as f64, reference.y() as f64);
            transform.Scale(reference.width() as f64, reference.height() as f64);
        }
        transform
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
