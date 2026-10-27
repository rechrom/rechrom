//! Invalidation-owned native style and root-constraint projection entries.
//!
//! Specialized grid, float, fragment, trigger, mask, and font stages retain
//! their own owners below these frames. The outer transactions belong here
//! because they admit mutable layout input and construct the values used to
//! invalidate or reuse the resident tree.

use super::*;

// cpp: layoutng/internal/boundary/native_input.h:39-50
// cpp: layoutng/internal/boundary/native_input.cc:939-946
pub fn PrepareNativeStyle(
    node: &NativeNodeConstructionData,
    resolver: &mut dyn NativeFontResolver,
    context: FragmentationType,
    layout_parent_style: *const NativeComputedStyle,
    layout_parent_kind: NodeKind,
) -> *const NativeComputedStyle {
    // StyleEngine already produced Chromium's immutable ComputedStyle. The
    // layout-input projection is still retained for DOM/paint metadata, but
    // layout must consume the original style object rather than translating
    // that projection back into a second style.
    if !node.style.native_style.is_null() {
        return node.style.native_style;
    }
    PrepareNativeStyleProjection(
        node,
        resolver,
        context,
        layout_parent_style,
        layout_parent_kind,
    )
}

#[inline(never)]
fn PrepareNativeStyleProjection(
    node: &NativeNodeConstructionData,
    resolver: &mut dyn NativeFontResolver,
    context: FragmentationType,
    layout_parent_style: *const NativeComputedStyle,
    layout_parent_kind: NodeKind,
) -> *const NativeComputedStyle {
    // Like Blink's CSSToLengthConversionData, zoom fixed CSS lengths before
    // passing them to layout. The DOM style remains in CSS units.
    let zoomed = crate::internal::css_zoom::ZoomedStyle(&node.style);
    let input = zoomed.as_ref();
    let defaults = ExtendedStyle::default();
    let extra = input.extended.as_deref().unwrap_or(&defaults);
    let mut builder = BeginNativeStyle(input, extra);
    ApplyNativeStyleListAndPosition(&mut builder, input, extra);
    ApplyNativeStyleShape(&mut builder, extra);
    ApplyNativeStyleBoxEdges(&mut builder, input, extra);
    ApplyNativeColumnRule(&mut builder, input);
    ApplyNativeStyleFlexAlignment(&mut builder, input, extra);
    ApplyNativeStyleMulticolAndInitialLetter(&mut builder, input, extra);
    ApplyNativeStyleGridTableAndBreaks(&mut builder, input, extra, context);
    ApplyNativeStyleTransforms(&mut builder, input);
    ApplyNativeStylePaintGrouping(&mut builder, input);
    mask_image_native::ApplyNativeMaskImages(&mut builder, input);
    ApplyNativeStyleOverflowAndScrollbars(&mut builder, extra);
    ApplyNativeTriggerScope(&mut builder, extra);
    ApplyNativeStyleContainmentAndAspect(&mut builder, extra);
    ApplyNativeStyleText(&mut builder, input, extra);
    FinishNativeStyle(
        &mut builder,
        node,
        extra,
        resolver,
        layout_parent_style,
        layout_parent_kind,
    )
}

pub fn PrepareNativeStyleDefault(
    input: &NativeNodeConstructionData,
    resolver: &mut dyn NativeFontResolver,
) -> *const NativeComputedStyle {
    PrepareNativeStyle(
        input,
        resolver,
        FragmentationType::kFragmentPage,
        std::ptr::null(),
        NodeKind::kBox,
    )
}

// cpp: layoutng/internal/boundary/native_input.h:52-57
// cpp: layoutng/internal/boundary/native_input.cc:1591-1618
#[inline(never)]
pub fn PrepareNativeConstraints(
    root_style: &ComputedStyle,
    space: &ConstraintSpace,
    contains_annotations: bool,
) -> NativeConstraintSpace {
    let direction = WritingDirectionMode::new(
        ConvertWritingMode(root_style.writing_mode),
        ConvertTextDirection(root_style.direction),
    );
    // cpp: web_frame_widget_impl.cc:2540-2568; local_frame_view.cc:1138-1146.
    // Native LayoutNG operates in LayoutZoomFactor coordinates. Device scale
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

pub fn PrepareNativeConstraintsDefault(
    root_style: &ComputedStyle,
    space: &ConstraintSpace,
) -> NativeConstraintSpace {
    PrepareNativeConstraints(root_style, space, false)
}
