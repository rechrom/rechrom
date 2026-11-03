// Source: Chromium SVGContentContainer::Layout. The existing fragment ABI uses
// LayoutBox storage, but SVG descendants are independently updated in SVG user
// coordinates. They never enter BlockLayoutAlgorithm or CSS in-flow placement.
// As for LayoutSVGRoot, descendants retain their own fragments. The boundary
// exports those children; they are not CSS inline/block children of this box.
#![allow(non_snake_case)]
use foundation::{gfx, DynamicTo, LayoutUnit};
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use layoutng_geometry::geometry::{fragment_geometry::FragmentGeometry, logical_size::LogicalSize};

pub(crate) fn LayoutContainer(
    object: &mut LayoutObject,
    info: &SVGLayoutInfo,
    width: f32,
    height: f32,
) -> SVGLayoutResult {
    let object_ptr = object as *mut LayoutObject;
    let style = object.StyleRef();
    let direction = style.GetWritingDirection();
    let mut constraints = ConstraintSpaceBuilder::new_without_parent_space(
        style.GetWritingMode(),
        direction,
        true,
        true,
        false,
    );
    let size = LogicalSize::new(
        LayoutUnit::FromFloatRound(width.max(0.0)),
        LayoutUnit::FromFloatRound(height.max(0.0)),
    );
    constraints.SetAvailableSize(size);
    let space = constraints.ToConstraintSpace();
    let node = BlockNode::new(object_ptr.cast());
    let geometry = FragmentGeometry {
        border_box_size: size,
        ..Default::default()
    };
    let mut builder =
        BoxFragmentBuilder::new(node.into(), style, &space, direction, std::ptr::null());
    builder.SetInitialFragmentGeometry(&geometry);
    builder.SetIsNewFormattingContext(true);
    let mut child = object.SlowFirstChild();
    let mut viewport_dependence = false;
    while let Some(child_object) = unsafe { child.as_mut() } {
        let result = child_object.UpdateSVGLayout(info);
        viewport_dependence |= result.has_viewport_dependence;
        child = child_object.NextSibling();
    }
    let result = builder.ToBoxFragment();
    unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutBox>() }
        .SetCachedLayoutResult(result, 0);
    object.ClearNeedsLayout();
    SVGLayoutResult::new(true, viewport_dependence)
}

/// SVGContentContainer::ComputeVisualOverflowRectIncludingFilters, projected
/// at the existing Layout -> Paint boundary after child SVG layout has
/// completed.  SVG containers own an anonymous zero-sized NG fragment, so its
/// CSS ink overflow is not their SVG visual rect.
pub(crate) fn VisualRect(object: &LayoutObject) -> gfx::RectF {
    if object.IsSVGHiddenContainer() {
        return gfx::RectF::default();
    }
    if object.IsSVGShape() {
        let element = DynamicTo::<Element>(object.GetNode());
        let Some(shape) = (unsafe { element.as_ref() })
            .and_then(|element| element.InputElementData().as_ref())
            .and_then(|data| data.svg_shape.as_ref())
        else {
            return gfx::RectF::default();
        };
        return gfx::RectF::new(
            gfx::PointF::new(shape.bounds_offset.x as f32, shape.bounds_offset.y as f32),
            gfx::SizeF::new(shape.bounds_width as f32, shape.bounds_height as f32),
        );
    }
    if object.IsSVGText() || object.IsSVGInline() || object.IsSVGInlineText() {
        let algorithms =
            layoutng_assembly::internal::layout_pass_scope::LayoutPassScope::Algorithms();
        if let Some(query) = unsafe { algorithms.as_ref() }
            .and_then(|algorithms| algorithms.svg_support.text_visual_rect)
        {
            return query(object);
        }
        return gfx::RectF::default();
    }
    if !object.IsSVGContainer() {
        return gfx::RectF::default();
    }
    let mut bounds = gfx::RectF::default();
    let mut child = object.SlowFirstChild();
    while let Some(child_object) = unsafe { child.as_ref() } {
        let child_bounds = VisualRect(child_object);
        if !child_bounds.IsEmpty() {
            bounds.Union(
                child_object
                    .LocalToSVGParentTransform()
                    .MapRect(child_bounds),
            );
        }
        child = child_object.NextSibling();
    }
    bounds
}
