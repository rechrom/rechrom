//! Invalidation-owned source style and root-constraint projection entries.
//!
//! Specialized grid, float, fragment, trigger, mask, and font stages retain
//! their own owners below these frames. The outer transactions belong here
//! because they admit mutable layout input and construct the values used to
//! invalidate or reuse the resident tree.

use super::*;

// cpp: layoutng/internal/boundary/input.h:39-50
// cpp: layoutng/internal/boundary/input.cc:939-946
pub fn PrepareStyle(
    node: &NodeConstructionData,
    resolver: &mut dyn FontResolver,
    context: FragmentationType,
    layout_parent_style: *const StyleComputedStyle,
    layout_parent_kind: NodeKind,
) -> *const StyleComputedStyle {
    // StyleEngine already produced Chromium's immutable ComputedStyle. The
    // layout-input projection is still retained for DOM/paint metadata, but
    // layout must consume the original style object rather than translating
    // that projection back into a second style.
    if !node.computed_style.is_null() {
        return node.computed_style;
    }
    PrepareStyleProjection(
        node,
        resolver,
        context,
        layout_parent_style,
        layout_parent_kind,
    )
}

#[inline(never)]
fn PrepareStyleProjection(
    node: &NodeConstructionData,
    resolver: &mut dyn FontResolver,
    context: FragmentationType,
    layout_parent_style: *const StyleComputedStyle,
    layout_parent_kind: NodeKind,
) -> *const StyleComputedStyle {
    // Like Blink's CSSToLengthConversionData, zoom fixed CSS lengths before
    // passing them to layout. The DOM style remains in CSS units.
    let zoomed = crate::internal::css_zoom::ZoomedStyle(&node.style);
    let input = zoomed.as_ref();
    let defaults = ExtendedStyle::default();
    let extra = input.extended.as_deref().unwrap_or(&defaults);
    let mut builder = BeginStyle(input, extra);
    ApplyStyleListAndPosition(&mut builder, input, extra);
    ApplyStyleShape(&mut builder, extra);
    ApplyStyleBoxEdges(&mut builder, input, extra);
    ApplyColumnRule(&mut builder, input);
    ApplyStyleFlexAlignment(&mut builder, input, extra);
    ApplyStyleMulticolAndInitialLetter(&mut builder, input, extra);
    ApplyStyleGridTableAndBreaks(&mut builder, input, extra, context);
    ApplyStyleTransforms(&mut builder, input);
    ApplyStylePaintGrouping(&mut builder, input);
    mask_image_projection::ApplyMaskImages(&mut builder, input);
    ApplyStyleOverflowAndScrollbars(&mut builder, extra);
    ApplyTriggerScope(&mut builder, extra);
    ApplyStyleContainmentAndAspect(&mut builder, extra);
    ApplyStyleText(&mut builder, input, extra);
    FinishStyle(
        &mut builder,
        node,
        extra,
        resolver,
        layout_parent_style,
        layout_parent_kind,
    )
}

pub fn PrepareStyleDefault(
    input: &NodeConstructionData,
    resolver: &mut dyn FontResolver,
) -> *const StyleComputedStyle {
    PrepareStyle(
        input,
        resolver,
        FragmentationType::kFragmentPage,
        std::ptr::null(),
        NodeKind::kBox,
    )
}

// cpp: layoutng/internal/boundary/input.h:52-57
// cpp: layoutng/internal/boundary/input.cc:1591-1618
#[inline(never)]
pub fn PrepareConstraints(
    root_style: &StyleComputedStyle,
    space: &ConstraintSpace,
    contains_annotations: bool,
) -> LayoutConstraintSpace {
    let direction = WritingDirectionMode::new(root_style.GetWritingMode(), root_style.Direction());
    // cpp: web_frame_widget_impl.cc:2540-2568; local_frame_view.cc:1138-1146.
    // Chromium LayoutNG operates in LayoutZoomFactor coordinates. Device scale
    // is a separate raster input and must never enlarge the layout viewport.
    let layout_zoom = Number(space.layout_zoom_factor, "layout zoom", true);
    let width = LayoutUnit::from_f64(Number(
        space.available_size.width * layout_zoom,
        "available width",
        true,
    ));
    let height = LayoutUnit::from_f64(Number(
        space.available_size.height * layout_zoom,
        "available height",
        true,
    ));
    let mut available = if direction.IsHorizontal() {
        LogicalSize::new(width, height)
    } else {
        LogicalSize::new(height, width)
    };
    if let Some(block_size) = space.fragmentainer_block_size {
        // In paged layout the initial containing block uses the page area.
        available.block_size = LayoutUnit::from_f64(Number(
            block_size * layout_zoom,
            "fragmentainer block size",
            true,
        ));
    }
    let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
        direction.GetWritingMode(),
        direction,
        true,
        true,
        false,
    );
    builder.SetAvailableSize(available);
    builder.SetPercentageResolutionSize(available);
    builder.SetIsFixedInlineSize(true);
    builder.SetIsFixedBlockSize(true);
    builder.SetContainsAnnotations(contains_annotations);
    builder.ToConstraintSpace()
}

pub fn PrepareConstraintsDefault(
    root_style: &StyleComputedStyle,
    space: &ConstraintSpace,
) -> LayoutConstraintSpace {
    PrepareConstraints(root_style, space, false)
}
