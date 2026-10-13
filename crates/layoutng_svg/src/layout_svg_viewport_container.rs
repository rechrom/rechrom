// Translation of core/layout/svg/layout_svg_viewport_container.{h,cc} at
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]
use crate::{
    layout_svg_group::LayoutSVGGroup, svg_container_layout::LayoutContainer, svg_viewport,
};
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::{gfx, Traceable, Visitor};
use layoutng_assembly::fragment_tree::SvgViewportGeometry;
use layoutng_assembly::internal::layout_input::SvgViewportLength;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use std::ops::{Deref, DerefMut};

#[repr(C)]
pub struct LayoutSVGViewportContainer {
    container: LayoutSVGGroup,
    viewport: gfx::RectF,
    content_transform: AffineTransform,
    local_to_parent_transform: AffineTransform,
}
impl LayoutSVGViewportContainer {
    pub fn new(element: *mut Element) -> Self {
        let container = LayoutSVGGroup::new(element);
        container.SetRuntimeClass(LayoutObjectClass::SvgViewportContainer);
        Self {
            container,
            viewport: gfx::RectF::default(),
            content_transform: AffineTransform::default(),
            local_to_parent_transform: AffineTransform::default(),
        }
    }
    pub fn Viewport(&self) -> gfx::RectF {
        self.viewport
    }
    pub fn LocalToSVGParentTransform(&self) -> AffineTransform {
        self.local_to_parent_transform
    }
    pub fn PaintGeometry(&self) -> SvgViewportGeometry {
        svg_viewport::Geometry(self, self.viewport, self.content_transform, true)
    }
    pub fn UpdateSVGLayout(&mut self, info: &SVGLayoutInfo) -> SVGLayoutResult {
        let element = unsafe { &*self.GetNode().cast::<Element>() };
        let input = element
            .InputElementData()
            .as_ref()
            .and_then(|e| e.svg_viewport)
            .expect("nested SVG viewport requires geometry input");
        let parent_size = svg_viewport::ResolveViewport(self);
        let resolve = |value: SvgViewportLength, dimension: f32| {
            let resolved = match value {
                SvgViewportLength::kUserUnits(value) => value,
                SvgViewportLength::kPercentage(value) => value * dimension as f64 / 100.0,
                SvgViewportLength::kUnsupported => svg_viewport::Unsupported(
                    "SVG viewport contextual/calculated or invalid length is not translated",
                ),
            };
            // SVGLengthContext clamps extreme CSS lengths. Until that branch
            // is translated, reject it rather than cast to infinity or publish
            // fragment geometry that disagrees with the SVG viewport.
            if !resolved.is_finite() || resolved.abs() > foundation::LayoutUnit::Max().ToDouble() {
                svg_viewport::Unsupported(
                    "SVG viewport length outside the supported finite LayoutUnit range",
                );
            }
            resolved as f32
        };
        self.viewport = gfx::RectF::new(
            gfx::PointF::new(
                resolve(input.x, parent_size.width()),
                resolve(input.y, parent_size.height()),
            ),
            gfx::SizeF::new(
                resolve(input.width, parent_size.width()),
                resolve(input.height, parent_size.height()),
            ),
        );
        self.content_transform = AffineTransform::new(
            1.0,
            0.0,
            0.0,
            1.0,
            self.viewport.x() as f64,
            self.viewport.y() as f64,
        );
        self.content_transform
            .PreConcat(svg_viewport::ViewBoxTransform(
                svg_viewport::ViewBox(self),
                self.viewport.size(),
            ));
        self.local_to_parent_transform = svg_viewport::LocalTransform(self);
        self.local_to_parent_transform
            .PreConcat(self.content_transform);
        let child_info = SVGLayoutInfo {
            viewport_changed: true,
            scale_factor_changed: true,
            ..*info
        };
        let width = self.viewport.width();
        let height = self.viewport.height();
        LayoutContainer(self, &child_info, width, height);
        SVGLayoutResult::new(true, true)
    }
}
impl Deref for LayoutSVGViewportContainer {
    type Target = LayoutSVGGroup;
    fn deref(&self) -> &Self::Target {
        &self.container
    }
}
impl DerefMut for LayoutSVGViewportContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}
impl Traceable for LayoutSVGViewportContainer {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.container.Trace(visitor);
    }
}
impl foundation::DowncastFrom<LayoutObject> for LayoutSVGViewportContainer {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGViewportContainer()
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGViewportContainer, container) == 0);
