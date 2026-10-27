// Source: Chromium SVGContentContainer::Layout. The existing fragment ABI uses
// LayoutBox storage, but SVG descendants are independently updated in SVG user
// coordinates. They never enter BlockLayoutAlgorithm or CSS in-flow placement.
// As for LayoutSVGRoot, descendants retain their own fragments. The boundary
// exports those children; they are not CSS inline/block children of this box.
#![allow(non_snake_case)]
use foundation::LayoutUnit;
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
