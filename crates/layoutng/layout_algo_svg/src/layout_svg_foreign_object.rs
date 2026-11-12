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
    viewport: foundation::gfx::RectF,
    overridden_location: foundation::PhysicalOffset,
    local_transform: foundation::blink_geometry::transforms::affine_transform::AffineTransform,
}

impl LayoutSVGForeignObject {
    pub fn UpdateSVGLayout(
        &mut self,
        _info: &layoutng_assembly::internal::svg_layout_info::SVGLayoutInfo,
    ) -> layoutng_assembly::internal::svg_layout_info::SVGLayoutResult {
        use layoutng_assembly::internal::{
            block_node::BlockNode, constraint_space_builder::ConstraintSpaceBuilder,
        };
        self.local_transform = crate::svg_viewport::LocalTransform(self);
        let style = self.StyleRef();
        let parent = crate::svg_viewport::ResolveViewport(self);
        let value = |length: &foundation::Length, dimension| {
            crate::svg_length_functions::ValueForLengthWithStyle(length, style, dimension)
        };
        let viewport = foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(
                value(style.X(), parent.width()),
                value(style.Y(), parent.height()),
            ),
            foundation::gfx::SizeF::new(
                value(style.Width(), parent.width()).max(0.),
                value(style.Height(), parent.height()).max(0.),
            ),
        );
        let zoom = style.EffectiveZoom();
        let mode = style.GetWritingMode();
        let direction = style.GetWritingDirection();
        self.viewport = viewport;
        self.overridden_location = foundation::PhysicalOffset::FromPointFFloor(
            &foundation::gfx::PointF::new(viewport.x() * zoom, viewport.y() * zoom),
        );
        let physical = foundation::PhysicalSize::new(
            foundation::LayoutUnit::FromFloatRound(viewport.width() * zoom),
            foundation::LayoutUnit::FromFloatRound(viewport.height() * zoom),
        );
        let size = layoutng_geometry::geometry::logical_size::LogicalSize::new(
            if foundation::IsHorizontalWritingMode(mode) {
                physical.width
            } else {
                physical.height
            },
            if foundation::IsHorizontalWritingMode(mode) {
                physical.height
            } else {
                physical.width
            },
        );
        let mut builder =
            ConstraintSpaceBuilder::new_without_parent_space(mode, direction, true, true, false);
        builder.SetAvailableSize(size);
        builder.SetIsFixedInlineSize(true);
        builder.SetIsFixedBlockSize(true);
        BlockNode::new((self as *mut Self).cast()).Layout(
            &builder.ToConstraintSpace(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        layoutng_assembly::internal::svg_layout_info::SVGLayoutResult::new(true, true)
    }

    pub fn ObjectBoundingBox(&self) -> foundation::gfx::RectF {
        self.viewport
    }
    pub fn PhysicalLocation(&self) -> foundation::PhysicalOffset {
        self.overridden_location
    }
    pub fn LocalToSVGParentTransform(
        &self,
    ) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform {
        let mut transform = self.local_transform;
        let inverse_zoom = 1.0 / self.StyleRef().EffectiveZoom() as f64;
        transform.Scale(inverse_zoom, inverse_zoom);
        transform
    }
    pub fn DecoratedBoundingBox(&self) -> foundation::gfx::RectF {
        foundation::gfx::RectF::from(foundation::PhysicalRect::new(
            self.PhysicalLocation(),
            self.StitchedSize(),
        ))
    }

    // cpp: layoutng_svg/layout_svg_foreign_object.cc:5-6
    pub fn new(element: *mut Element) -> Self {
        let group = LayoutSVGGroup::new(element);
        group.SetRuntimeClass(LayoutObjectClass::SvgForeignObject);
        Self {
            group,
            viewport: Default::default(),
            overridden_location: Default::default(),
            local_transform: Default::default(),
        }
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
