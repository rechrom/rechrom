// Source: Chromium SVGContentContainer::Layout. The existing fragment ABI uses
// LayoutBox storage, but SVG descendants are independently updated in SVG user
// coordinates. They never enter BlockLayoutAlgorithm or CSS in-flow placement.
// As for LayoutSVGRoot, descendants retain their own fragments. The boundary
// exports those children; they are not CSS inline/block children of this box.
#![allow(non_snake_case)]
use foundation::{gfx, LayoutUnit};
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::layout_box::LayoutBox;
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
        viewport_dependence |=
            crate::svg_resources::LayoutMarkerResourcesIfNeeded(child_object, info);
        let result = child_object.UpdateSVGLayout(info);
        child_object.SetHasViewportDependence(result.has_viewport_dependence);
        viewport_dependence |= result.has_viewport_dependence;
        child = child_object.NextSibling();
    }
    // SVGContentContainer::Layout updates retained bounds after all children.
    unsafe {
        &mut *(object as *mut LayoutObject).cast::<crate::layout_svg_group::LayoutSVGGroup>()
    }
    .UpdateObjectBounds();
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
        return unsafe {
            &*(object as *const LayoutObject).cast::<crate::layout_svg_shape::LayoutSVGShape>()
        }
        .DecoratedBoundingBox();
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

// SVGContentContainer::ComputeVisualOverflowRectIncludingFilters cc:225-240,
// and svg_content_container.cc:37-54. This is the source bounds traversal of
// native content, used by the SVGRoot virtual; no hit-test tree is projected.
pub(crate) fn ContentVisualOverflowIncludingFilters(root: &LayoutObject) -> gfx::RectF {
    let mut bounds = gfx::RectF::default();
    let mut child = root.SlowFirstChild();
    while let Some(object) = unsafe { child.as_ref() } {
        child = object.NextSibling();
        if !crate::svg_resources::IsObjectBoundingBoxValid(object) {
            continue;
        }
        let local = if object.IsSVGContainer() && !object.IsSVGForeignObject() {
            ContentVisualOverflowIncludingFilters(object)
        } else {
            crate::svg_resources::DecoratedBoundingBox(object)
        };
        let filtered =
            layoutng_assembly::internal::layout_pass_scope::LayoutPassScope::WithSVGResourceTree(
                |tree| crate::svg_resources::ApplyFiltersToRect(tree, object, local),
            )
            .unwrap_or_else(|| {
                assert!(
                    !object.StyleRef().HasFilter(),
                    "SVG filter bounds need the native resource query scope"
                );
                local
            });
        bounds.Union(object.LocalToSVGParentTransform().MapRect(filtered));
    }
    bounds
}
