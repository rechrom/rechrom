#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{DynamicTo, LayoutUnit, PhysicalRect, PhysicalSize, Traceable, Visitor};
use layoutng_assembly::internal::layout_algorithm_set::SvgRootSizingInfo;
use layoutng_assembly::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::layout_object_child_list::LayoutObjectChildList;
use layoutng_assembly::internal::layout_replaced::LayoutReplaced;
use layoutng_assembly::internal::svg_layout_info::SVGLayoutInfo;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::natural_sizing_info::PhysicalNaturalSizingInfo;

// cpp: layoutng_svg/layout_svg_root.h:10-50
#[repr(C)]
pub struct LayoutSVGRoot {
    replaced: LayoutReplaced,
    children: LayoutObjectChildList,
    container_size: PhysicalSize,
    viewport: foundation::gfx::RectF,
    content_transform: foundation::blink_geometry::transforms::affine_transform::AffineTransform,
}

impl LayoutSVGRoot {
    // cpp: layoutng_svg/svg_layout_objects.cc:55-56
    pub fn new(element: *mut Element) -> Self {
        let replaced = LayoutReplaced::new(element);
        replaced.SetRuntimeClass(LayoutObjectClass::SvgRoot);
        Self {
            replaced,
            children: LayoutObjectChildList::default(),
            container_size: PhysicalSize::default(),
            viewport: Default::default(),
            content_transform: Default::default(),
        }
    }

    // cpp: layoutng_svg/layout_svg_root.h:19-27
    pub fn IsEmbeddedThroughFrameContainingSVGDocument(&self) -> bool {
        false
    }
    pub fn LogicalSizeScaleFactorForPercentageLengths(&self) -> f64 {
        1.0
    }
    pub fn FirstChild(&self) -> *mut LayoutObject {
        self.children.FirstChild()
    }
    pub fn LastChild(&self) -> *mut LayoutObject {
        self.children.LastChild()
    }
    pub fn GetContainerSize(&self) -> PhysicalSize {
        self.container_size
    }
    // core/layout/svg/layout_svg_root.h: SVGImage supplies this before layout.
    pub fn SetContainerSize(&mut self, size: PhysicalSize) {
        if self.container_size == size {
            return;
        }
        self.container_size = size;
        self.SetNeedsLayoutAndFullPaintInvalidation(
            &raw const layoutng_assembly::internal::layout_invalidation_reason::kSizeChanged,
        );
    }
    pub fn ViewportSize(&self) -> foundation::gfx::SizeF {
        self.viewport.size()
    }
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGRoot"
    }
    pub fn CanHaveChildren(&self) -> bool {
        true
    }
    pub fn IsSVG(&self) -> bool {
        true
    }
    pub fn IsSVGRoot(&self) -> bool {
        true
    }
    pub fn ShouldApplyObjectViewBox(&self) -> bool {
        false
    }
    pub fn VirtualChildren(&self) -> &LayoutObjectChildList {
        &self.children
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:63-68
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, _style: &ComputedStyle) -> bool {
        !child.is_null() && {
            let child = unsafe { &*child };
            child.IsSVGText()
                || child.IsSVGShape()
                || child.IsSVGContainer()
                || child.IsSVGForeignObject()
        }
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:70-76
    pub fn AddChild(&mut self, child: *mut LayoutObject, before_child: *mut LayoutObject) {
        assert!(!child.is_null());
        assert!(self.IsChildAllowed(child, unsafe { &*child }.StyleRef()));
        self.AddChildBase(child, before_child);
    }
    pub fn RemoveChild(&mut self, child: *mut LayoutObject) {
        self.RemoveChildBase(child);
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:79-103
    pub fn GetNaturalDimensions(&self) -> PhysicalNaturalSizingInfo {
        let element = DynamicTo::<Element>(self.GetNode());
        assert!(!element.is_null());
        let mut result = PhysicalNaturalSizingInfo::None();
        let Some(data) = unsafe { &*element }.InputElementData().as_ref() else {
            return result;
        };
        let zoom = self.StyleRef().EffectiveZoom() as f64;
        if let Some(width) = data.natural_width {
            result.size.width = LayoutUnit::from_f64(width * zoom);
            result.has_width = true;
        }
        if let Some(height) = data.natural_height {
            result.size.height = LayoutUnit::from_f64(height * zoom);
            result.has_height = true;
        }
        if let Some(ratio) = data.natural_aspect_ratio.filter(|ratio| *ratio > 0.0) {
            result.aspect_ratio = PhysicalSize::new(
                LayoutUnit::from_f64(ratio * 1000.0),
                LayoutUnit::from_signed(1000),
            );
        } else if result.has_width && result.has_height && !result.size.IsEmpty() {
            result.aspect_ratio = result.size;
        }
        result
    }

    // cpp: layoutng_svg/svg_layout_objects_layout.cc:15-34
    pub fn ContentTransform(
        &self,
    ) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform {
        self.content_transform
    }
    pub fn PaintGeometry(&self) -> layoutng_assembly::fragment_tree::SvgViewportGeometry {
        crate::svg_viewport::Geometry(self, self.viewport, self.content_transform, false)
    }
    pub fn LayoutRoot(&mut self, content_rect: &PhysicalRect) {
        self.viewport = foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(
                content_rect.offset.left.ToFloat(),
                content_rect.offset.top.ToFloat(),
            ),
            foundation::gfx::SizeF::new(
                content_rect.size.width.ToFloat(),
                content_rect.size.height.ToFloat(),
            ),
        );
        self.content_transform =
            foundation::blink_geometry::transforms::affine_transform::AffineTransform::new(
                1.0,
                0.0,
                0.0,
                1.0,
                self.viewport.x() as f64,
                self.viewport.y() as f64,
            );
        if crate::svg_viewport::ViewBox(self).is_some() {
            self.content_transform
                .PreConcat(crate::svg_viewport::ViewBoxTransform(
                    crate::svg_viewport::ViewBox(self),
                    self.viewport.size(),
                ));
        } else {
            let zoom = self.StyleRef().EffectiveZoom() as f64;
            self.content_transform.Scale(zoom, zoom);
        }
        let mut object = (self as *mut Self).cast::<LayoutObject>();
        while !object.is_null() {
            let box_model = DynamicTo::<LayoutBoxModelObject>(object);
            if !box_model.is_null() {
                unsafe { &mut *box_model }.UpdateTransformForLayout();
            }
            object = unsafe { &*object }.Parent();
        }
        let info = SVGLayoutInfo {
            force_layout: true,
            viewport_changed: true,
            ..SVGLayoutInfo::default()
        };
        let mut child = self.FirstChild();
        while !child.is_null() {
            unsafe { &mut *child }.UpdateSVGLayout(&info);
            child = unsafe { &*child }.NextSibling();
        }
    }
}

impl Deref for LayoutSVGRoot {
    type Target = LayoutReplaced;
    fn deref(&self) -> &Self::Target {
        &self.replaced
    }
}
impl DerefMut for LayoutSVGRoot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.replaced
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGRoot, replaced) == 0);
impl Traceable for LayoutSVGRoot {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.children.Trace(visitor);
        self.replaced.Trace(visitor);
    }
}
impl foundation::DowncastFrom<LayoutObject> for LayoutSVGRoot {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGRoot()
    }
}

pub fn SvgRootAddChild(
    object: &mut LayoutObject,
    child: *mut LayoutObject,
    before: *mut LayoutObject,
) {
    unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGRoot>() }.AddChild(child, before);
}
pub fn SvgRootRemoveChild(object: &mut LayoutObject, child: *mut LayoutObject) {
    unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGRoot>() }.RemoveChild(child);
}
pub fn SvgRootChildren(object: &LayoutObject) -> *mut LayoutObjectChildList {
    unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGRoot>() }.VirtualChildren()
        as *const LayoutObjectChildList as *mut LayoutObjectChildList
}
pub fn SvgRootNaturalDimensions(object: &LayoutObject) -> PhysicalNaturalSizingInfo {
    unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGRoot>() }.GetNaturalDimensions()
}
pub fn SvgRootLayout(object: &mut LayoutObject, rect: &PhysicalRect) {
    unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGRoot>() }.LayoutRoot(rect);
}
pub fn SvgRootSetContainerSize(object: &mut LayoutObject, size: PhysicalSize) {
    unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGRoot>() }.SetContainerSize(size);
}
pub fn QuerySvgRootSizing(object: &LayoutObject) -> SvgRootSizingInfo {
    let root = unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGRoot>() };
    let size = root.GetContainerSize();
    SvgRootSizingInfo {
        container_width: size.width,
        container_height: size.height,
        embedded_through_frame: root.IsEmbeddedThroughFrameContainingSVGDocument(),
        logical_size_scale_factor: root.LogicalSizeScaleFactorForPercentageLengths(),
    }
}
