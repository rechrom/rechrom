//! The native Filter/Marker resource identities needed by SVGResource cycle
//! traversal. Source: layout_svg_resource_filter.cc:30-43,68-82;
//! layout_svg_resource_marker.cc:46-191. Marker geometry borrows its native
//! descendants; this resource identity never consumes flattened paint data.
use crate::layout_svg_resource_clipper::LayoutSVGResourceContainer;
use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::{
    layout_node_metadata::Element,
    layout_object::{LayoutObject, LayoutObjectClass},
};
use layoutng_style::style::computed_style::ComputedStyle;
use std::ops::{Deref, DerefMut};
#[repr(C)]
pub struct LayoutSVGResourceAuxiliary {
    resource: LayoutSVGResourceContainer,
    filter: bool,
    viewport: foundation::gfx::SizeF,
    local_transform: foundation::blink_geometry::transforms::affine_transform::AffineTransform,
    is_in_layout: std::cell::Cell<bool>,
}
impl LayoutSVGResourceAuxiliary {
    pub fn new(element: *mut Element, filter: bool) -> Self {
        let resource = LayoutSVGResourceContainer::new(element);
        resource.SetRuntimeClass(LayoutObjectClass::SvgResourceAuxiliary);
        Self {
            resource,
            filter,
            viewport: Default::default(),
            local_transform: Default::default(),
            is_in_layout: Default::default(),
        }
    }
    pub fn IsFilter(&self) -> bool {
        self.filter
    }

    fn MarkerData(&self) -> layoutng_assembly::internal::layout_input::SvgMarkerData {
        let element = foundation::DynamicTo::<Element>(self.GetNode());
        unsafe { element.as_ref() }
            .and_then(|element| element.InputElementData().as_ref())
            .and_then(|data| data.svg_marker.clone())
            .expect("native SVGMarkerElement owns its animated marker data")
    }
    pub fn Viewport(&self) -> foundation::gfx::RectF {
        foundation::gfx::RectF::new(Default::default(), self.viewport)
    }
    pub fn LocalToSVGParentTransform(
        &self,
    ) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform {
        self.local_transform
    }
    pub fn ReferencePoint(&self) -> foundation::gfx::PointF {
        let data = self.MarkerData();
        foundation::gfx::PointF::new(
            crate::svg_length_context::Value(self, &data.lengths.ref_x, true, data.container_sizes),
            crate::svg_length_context::Value(
                self,
                &data.lengths.ref_y,
                false,
                data.container_sizes,
            ),
        )
    }
    pub fn ShouldPaint(&self) -> bool {
        crate::svg_viewport::ViewBox(self)
            .is_none_or(|viewbox| viewbox.width > 0. && viewbox.height > 0.)
    }
    pub(crate) fn MarkerTransformation(
        &self,
        position: &crate::svg_marker_data::MarkerPosition,
        stroke_width: f32,
    ) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform {
        use layoutng_assembly::internal::layout_input::SvgMarkerOrient as Orient;
        let data = self.MarkerData();
        let scale = if data.stroke_width_units {
            stroke_width
        } else {
            1.
        };
        let angle = match data.orient {
            Orient::Angle(angle) => angle as f64,
            Orient::Auto => position.angle as f64,
            Orient::AutoStartReverse => {
                position.angle as f64
                    + if position.kind == crate::svg_marker_data::MarkerType::Start {
                        180.
                    } else {
                        0.
                    }
            }
        };
        let mut transform =
            foundation::blink_geometry::transforms::affine_transform::AffineTransform::default();
        transform
            .Translate(position.origin.x() as f64, position.origin.y() as f64)
            .Rotate(angle)
            .Scale(scale as f64, scale as f64);
        let reference = self.local_transform.MapPoint(&self.ReferencePoint());
        transform.Translate(-reference.x() as f64, -reference.y() as f64);
        transform
    }
    pub(crate) fn MarkerBoundaries(
        &self,
        marker_transform:&foundation::blink_geometry::transforms::affine_transform::AffineTransform,
    ) -> foundation::gfx::RectF {
        // marker.cc:89-101. LayoutSVGContainer::VisualRectInLocalSVGCoordinates
        // is its resident decorated bounds, before this marker's viewBox map.
        marker_transform.MapRect(self.local_transform.MapRect(self.ObjectBounds().decorated))
    }
    pub fn UpdateSVGLayout(
        &mut self,
        info: &layoutng_assembly::internal::svg_layout_info::SVGLayoutInfo,
    ) -> layoutng_assembly::internal::svg_layout_info::SVGLayoutResult {
        if self.filter {
            return self.resource.UpdateSVGLayout(info);
        }
        if self.is_in_layout.replace(true) {
            return layoutng_assembly::internal::svg_layout_info::SVGLayoutResult::new(
                false, false,
            );
        }
        struct Reset(*const std::cell::Cell<bool>);
        impl Drop for Reset {
            fn drop(&mut self) {
                unsafe { &*self.0 }.set(false);
            }
        }
        let _reset = Reset(&self.is_in_layout);
        let data = self.MarkerData();
        self.viewport = foundation::gfx::SizeF::new(
            crate::svg_length_context::Value(self, &data.lengths.width, true, data.container_sizes),
            crate::svg_length_context::Value(
                self,
                &data.lengths.height,
                false,
                data.container_sizes,
            ),
        );
        self.local_transform = crate::svg_viewport::ViewBoxTransform(
            crate::svg_viewport::ViewBox(self),
            self.viewport,
        );
        // The marker follows LayoutSVGContainer, bypassing the hidden-layout
        // override, and owns its own reentrant resource-layout guard.
        let mut result = crate::svg_container_layout::LayoutContainer(self, info, 0., 0.);
        result.has_viewport_dependence |= [
            &data.lengths.width,
            &data.lengths.height,
            &data.lengths.ref_x,
            &data.lengths.ref_y,
        ]
        .into_iter()
        .any(|value| crate::svg_length_context::IsRelative(value));
        result
    }
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, _style: &ComputedStyle) -> bool {
        // Filter primitives use the retained typed Filter graph; there is no
        // represented SVGFilterPrimitive LayoutObject class to attach here.
        if self.filter {
            return false;
        }
        unsafe { child.as_ref() }.is_some_and(|child| {
            child.IsSVG() && !child.IsSVGInline() && !child.IsSVGInlineText() && !child.IsSVGRoot()
        })
    }
}
impl Deref for LayoutSVGResourceAuxiliary {
    type Target = LayoutSVGResourceContainer;
    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}
impl DerefMut for LayoutSVGResourceAuxiliary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}
impl Traceable for LayoutSVGResourceAuxiliary {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.resource.Trace(visitor)
    }
}
