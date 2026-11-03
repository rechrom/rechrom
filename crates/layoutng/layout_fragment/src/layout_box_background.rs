//! LayoutBox background coverage and pre-paint obscuration.

#![allow(non_snake_case)]

use super::super::scroll_types::IncludeScrollbarsInRect;
use super::*;
use foundation::{
    BlendMode, CompositeOperator, DynamicTo, EBorderStyle, EObjectFit, EVisibility,
    RuntimeEnabledFeatures,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::computed_style_constants::{
    EFillAttachment, EFillBox, EFillRepeat, EnclosingFillBox, IsSpecialClipFillBox,
};
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use layoutng_style::style::fill_layer::FillLayer;

// This is the LayoutBox-to-LayoutView path used by IntersectsVisibleViewport.
// It is the represented subset of layout_box.cc:1794-1930 and 3052-3120:
// static layout transforms, container offsets, scrolling, sticky/anchor
// translations, perspective admission, and box clips are all applied here.
// Filter expansion and 3D TransformState mapping require PaintLayer/property-
// tree products outside this package, so callers conservatively treat those
// objects as intersecting instead of fabricating a mapped rectangle.
fn MapVisualOverflowToLayoutView(
    box_: &LayoutBox,
    layout_view: *const LayoutObject,
) -> Option<PhysicalRect> {
    let mut state = TransformState::new(
        TransformState::kApplyTransformDirection,
        gfx::QuadF::from(gfx::RectF::from(box_.VisualOverflowRect())),
    );
    let mut current = box_ as *const LayoutBox;

    while current.cast::<LayoutObject>() != layout_view {
        let current_box = unsafe { &*current };
        if current_box.StyleRef().HasFilterInducingProperty() || current_box.HasReflection() {
            return None;
        }

        let mut container = current_box.Container();
        if container.is_null() {
            break;
        }
        while !unsafe { &*container }.IsBox() {
            container = unsafe { &*container }.Parent();
            if container.is_null() {
                break;
            }
        }
        if container.is_null() {
            break;
        }

        let container_object = unsafe { &*container };
        let container_box = unsafe { &*container.cast::<LayoutBox>() };
        let mut container_offset = current_box.PhysicalLocation();
        if current_box.IsStickyPositioned() {
            container_offset += current_box.StickyPositionOffset();
        } else if current_box.NeedsAnchorPositionScrollAdjustment() {
            container_offset += current_box.AnchorPositionScrollTranslationOffset();
        }

        let preserve_3d = container_object.StyleRef().Preserves3D()
            && container == current_box.NearestAncestorForElement();
        let accumulation = if preserve_3d {
            TransformState::kAccumulateTransform
        } else {
            TransformState::kFlattenTransform
        };
        let container_is_ancestor = container.cast_const() == layout_view;

        if current_box.ShouldUseTransformFromContainer(container) {
            let mut transform = current_box.CurrentTransform();
            transform.PostTranslate(
                container_offset.left.ToFloat(),
                container_offset.top.ToFloat(),
            );
            if !container_is_ancestor && container_box.IsScrollContainer() {
                let scroll = -container_box.ScrolledContentOffset();
                transform.PostTranslate(scroll.left.ToFloat(), scroll.top.ToFloat());
            }
            if container_object.HasPerspective()
                && container == current_box.NearestAncestorForElement()
            {
                let origin = container_box.PerspectiveOrigin(std::ptr::null());
                let mut perspective = gfx::Transform::default();
                perspective
                    .ApplyPerspectiveDepth(container_object.StyleRef().UsedPerspective() as f64);
                perspective.ApplyTransformOrigin(origin.x(), origin.y(), 0.0);
                transform = perspective * transform;
            }
            let matrix = transform.GetColMajor();
            if matrix[2] != 0.0
                || matrix[3] != 0.0
                || matrix[6] != 0.0
                || matrix[7] != 0.0
                || matrix[8] != 0.0
                || matrix[9] != 0.0
                || matrix[10] != 1.0
                || matrix[11] != 0.0
                || matrix[14] != 0.0
                || matrix[15] != 1.0
            {
                return None;
            }
            state.ApplyTransform(&transform, accumulation);
            if !preserve_3d {
                state.Flatten();
            }
            if !container_is_ancestor
                && container_object.HasClipRelatedProperty()
                && !container_box.ApplyBoxClips(
                    &mut state,
                    accumulation,
                    VisualRectFlags::default(),
                )
            {
                state.Flatten();
                return Some(PhysicalRect::EnclosingRect(
                    &state.LastPlanarQuad().BoundingBox(),
                ));
            }
        } else {
            state.Move(container_offset, accumulation);
            if !container_is_ancestor && container_object.HasClipRelatedProperty() {
                if container_box.IsScrollContainer() {
                    state.Move(-container_box.ScrolledContentOffset(), accumulation);
                }
                if !container_box.ApplyBoxClips(
                    &mut state,
                    accumulation,
                    VisualRectFlags::default(),
                ) {
                    state.Flatten();
                    return Some(PhysicalRect::EnclosingRect(
                        &state.LastPlanarQuad().BoundingBox(),
                    ));
                }
            }
        }
        current = container.cast::<LayoutBox>();
    }

    state.Flatten();
    Some(PhysicalRect::EnclosingRect(
        &state.LastPlanarQuad().BoundingBox(),
    ))
}

// Source: core/layout/layout_box.cc:1396-1482.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxPhysicalBackgroundRectProvider(
    box_: &LayoutBox,
    rect_type: BackgroundRectType,
) -> PhysicalRect {
    box_.CheckIsNotDestroyed();
    if rect_type == BackgroundRectType::kBackgroundKnownOpaqueRect
        && box_.BackgroundTransfersToViewDefault()
    {
        return PhysicalRect::default();
    }

    let style = box_.StyleRef();
    let background_color = style.ResolvedColor(style.BackgroundColor(), None);
    let mut background_box = None;
    let mut current = style.BackgroundLayers() as *const FillLayer;
    while !current.is_null() {
        let layer = unsafe { &*current };
        let mut current_clip = layer.Clip();
        if rect_type == BackgroundRectType::kBackgroundKnownOpaqueRect {
            if IsSpecialClipFillBox(current_clip)
                || layer.GetBlendMode() != BlendMode::kNormal
                || layer.Composite() != CompositeOperator::kCompositeSourceOver
            {
                current = layer.Next();
                continue;
            }
            let image = layer.GetImage();
            let image_is_opaque = !image.is_null()
                && matches!(
                    layer.Repeat().x,
                    EFillRepeat::kRepeatFill | EFillRepeat::kRoundFill
                )
                && matches!(
                    layer.Repeat().y,
                    EFillRepeat::kRepeatFill | EFillRepeat::kRoundFill
                )
                && unsafe { &*image }.KnownToBeOpaque(style);
            let color_is_opaque = layer.Next().is_null() && background_color.IsOpaque();
            if !image_is_opaque && !color_is_opaque {
                current = layer.Next();
                continue;
            }
        } else {
            let invisible_color_only_layer = layer.GetImage().is_null()
                && (!layer.Next().is_null() || background_color.IsFullyTransparent());
            if invisible_color_only_layer {
                current = layer.Next();
                continue;
            }
            if current_clip == EFillBox::kContent && layer.Attachment() == EFillAttachment::kLocal {
                current_clip = EFillBox::kPadding;
            }
        }
        if current_clip == EFillBox::kBorder && layer.Attachment() == EFillAttachment::kLocal {
            current_clip = EFillBox::kPadding;
        }
        background_box = Some(match background_box {
            Some(previous) => EnclosingFillBox(previous, current_clip),
            None => current_clip,
        });
        current = layer.Next();
    }

    let Some(mut background_box) = background_box else {
        return PhysicalRect::default();
    };
    if IsSpecialClipFillBox(background_box) {
        debug_assert_ne!(rect_type, BackgroundRectType::kBackgroundKnownOpaqueRect);
        background_box = EFillBox::kBorder;
    }
    if rect_type == BackgroundRectType::kBackgroundPaintedExtent
        && background_box == EFillBox::kBorder
        && box_.BackgroundClipBorderBoxIsEquivalentToPaddingBox()
    {
        background_box = EFillBox::kPadding;
    }
    match background_box {
        EFillBox::kBorder => box_.PhysicalBorderBoxRect(),
        EFillBox::kPadding => box_.PhysicalPaddingBoxRect(),
        EFillBox::kContent => box_.PhysicalContentBoxRect(),
        _ => unreachable!("special background clips are normalized above"),
    }
}

// Source: core/layout/layout_box.cc:2030-2033.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxBackgroundPaintedExtentProvider(box_: &LayoutBox) -> PhysicalRect {
    box_.PhysicalBackgroundRect(BackgroundRectType::kBackgroundPaintedExtent)
}

// Source: core/layout/layout_box.cc:2035-2063.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxBackgroundIsKnownToBeOpaqueInRectProvider(
    box_: &LayoutBox,
    local_rect: &PhysicalRect,
) -> bool {
    box_.CheckIsNotDestroyed();
    let style = box_.StyleRef();
    if style.HasEffectiveAppearance()
        || style.HasBorderRadius()
        || style.HasBorderShape()
        || box_.HasClipPath()
        || style.HasBlendMode()
    {
        return false;
    }
    box_.PhysicalBackgroundRect(BackgroundRectType::kBackgroundKnownOpaqueRect)
        .ContainsRect(local_rect)
}

// Source: core/layout/layout_box.cc:2065-2086.
fn IsCandidateForOpaquenessTest(child_box: &LayoutBox) -> bool {
    if child_box.HasLayer() {
        return false;
    }
    let style = child_box.StyleRef();
    if style.Visibility() != EVisibility::kVisible
        || !style.ShapeOutside().is_null()
        || child_box.StitchedSize().IsZero()
        || (child_box.IsLayoutReplaced() && style.HasBorderRadius())
    {
        return false;
    }
    true
}

// Source: core/layout/layout_image.cc:326-367.
fn LayoutImageForegroundIsKnownToBeOpaqueInRect(
    box_: &LayoutBox,
    local_rect: &PhysicalRect,
) -> bool {
    if box_.ChildPaintBlockedByDisplayLock() {
        return false;
    }
    let element = DynamicTo::<Element>(box_.GetNode());
    if element.is_null() {
        return false;
    }
    let Some(data) = unsafe { &*element }.InputElementData().as_ref() else {
        return false;
    };
    if !data.image_resource_is_loaded {
        return false;
    }
    if !box_.PhysicalContentBoxRect().ContainsRect(local_rect) {
        return false;
    }
    let style = box_.StyleRef();
    let background_clip = style.BackgroundClip();
    if background_clip == EFillBox::kBorder
        && style.HasBorder()
        && !style.BorderObscuresBackground()
    {
        return false;
    }
    if matches!(background_clip, EFillBox::kBorder | EFillBox::kPadding) && style.MayHavePadding() {
        return false;
    }
    if style.ObjectPosition() != &ComputedStyleInitialValues::InitialObjectPosition() {
        return false;
    }
    if !matches!(style.GetObjectFit(), EObjectFit::kFill | EObjectFit::kCover) {
        return false;
    }
    if style.ObjectViewBox().is_some() {
        return false;
    }
    // cpp: core/layout/layout_image.cc:363-365. Keep the event before the
    // final opacity query so a geometrically eligible translucent image is
    // represented too, exactly like inspector_paint_image_event::Data.
    if browser_tracing::enabled() {
        browser_tracing::instant(
            "devtools.timeline",
            "PaintImage",
            &[("node_id", unsafe { &*element }.InputId() as f64)],
        );
    }
    data.image_resource_known_to_be_opaque
}

// Source: core/layout/layout_box.cc:2088-2137.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxForegroundIsKnownToBeOpaqueInRectProvider(
    box_: &LayoutBox,
    local_rect: &PhysicalRect,
    max_depth_to_test: u32,
) -> bool {
    box_.CheckIsNotDestroyed();
    if box_.IsLayoutImage() {
        return LayoutImageForegroundIsKnownToBeOpaqueInRect(box_, local_rect);
    }
    if max_depth_to_test == 0 || box_.ChildPaintBlockedByDisplayLock() {
        return false;
    }
    let mut child = box_.SlowFirstChild();
    while !child.is_null() {
        let child_object = unsafe { &*child };
        if child_object.IsBox() {
            let child_box = unsafe { &*child.cast::<LayoutBox>() };
            if IsCandidateForOpaquenessTest(child_box) {
                debug_assert!(!child_box.IsPositioned());
                let mut child_local_rect = *local_rect;
                child_local_rect.Move(&-child_box.PhysicalLocation());
                if child_local_rect.Y() < LayoutUnit::default()
                    || child_local_rect.X() < LayoutUnit::default()
                {
                    return false;
                }
                if child_local_rect.Bottom() <= child_box.StitchedSize().height
                    && child_local_rect.Right() <= child_box.StitchedSize().width
                {
                    if RuntimeEnabledFeatures::CompositeBGColorAnimationEnabled()
                        && child_object.StyleRef().HasCurrentBackgroundColorAnimation()
                    {
                        return false;
                    }
                    if child_box.BackgroundIsKnownToBeOpaqueInRect(&child_local_rect)
                        || child_box.ForegroundIsKnownToBeOpaqueInRect(
                            &child_local_rect,
                            max_depth_to_test - 1,
                        )
                    {
                        return true;
                    }
                }
            }
        }
        child = child_object.NextSibling();
    }
    false
}

// Source: core/layout/layout_box.cc:2139-2172.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxComputeBackgroundIsKnownToBeObscuredProvider(
    box_: &LayoutBox,
) -> bool {
    box_.CheckIsNotDestroyed();
    // Source: core/layout/layout_image.cc:369-375.
    if box_.IsLayoutImage() {
        return box_.StyleRef().HasBackground()
            && LayoutImageForegroundIsKnownToBeOpaqueInRect(box_, &box_.BackgroundPaintedExtent());
    }
    if box_.ScrollsOverflow()
        || (box_.HasNonVisibleOverflow() && !box_.BackgroundClipBorderBoxIsEquivalentToPaddingBox())
        || !box_.StyleRef().HasBackground()
        || box_.IsLayoutView()
        || !box_.StyleRef().BoxShadow().is_null()
        || box_.IsFragmented()
    {
        return false;
    }
    box_.ForegroundIsKnownToBeOpaqueInRect(&box_.BackgroundPaintedExtent(), 4)
}

// Source: core/layout/layout_box.cc:2299-2315.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxIntersectsVisibleViewportProvider(box_: &LayoutBox) -> bool {
    box_.CheckIsNotDestroyed();

    // A Layout tree in this package is one local-frame tree.  Walking to its
    // outer LayoutView is the retained equivalent of View() followed by the
    // Frame::OwnerLayoutObject() loop; cross-frame owners remain a host input.
    let mut layout_view = box_ as *const LayoutBox as *mut LayoutObject;
    loop {
        let parent = unsafe { &*layout_view }.Parent();
        if parent.is_null() {
            break;
        }
        layout_view = parent;
    }
    debug_assert!(unsafe { &*layout_view }.IsLayoutView());
    if std::ptr::eq(box_ as *const LayoutBox as *const LayoutObject, layout_view) {
        return true;
    }

    let Some(rect) = MapVisualOverflowToLayoutView(box_, layout_view) else {
        return true;
    };

    let layout_view_box = unsafe { &*layout_view.cast::<LayoutBox>() };
    let scrollable_area = layout_view_box.GetScrollableArea();
    let visible_rect = if scrollable_area.is_null() {
        // FrameView always owns a ScrollableArea in Chromium.  Before the
        // extracted package installs its LayoutView area, the same immutable
        // viewport size arrives through LayoutInput.
        let viewport = layout_view_box.ViewportGeometryForLayout().size;
        PhysicalRect::from(gfx::Rect::new(
            gfx::Point::default(),
            gfx::Size::new(viewport.width, viewport.height),
        ))
    } else {
        PhysicalRect::from(
            unsafe { &*scrollable_area }
                .VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars),
        )
    };
    rect.IntersectsRect(&visible_rect)
}

// Source: core/layout/layout_box.cc:2317-2340.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxEnsureIsReadyForPaintInvalidationProvider(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    box_.EnsureIsReadyForPaintInvalidationBase();

    let new_obscured = box_.ComputeBackgroundIsKnownToBeObscured();
    if box_.BackgroundIsKnownToBeObscured() != new_obscured {
        box_.SetBackgroundIsKnownToBeObscured(new_obscured);
        box_.SetBackgroundNeedsFullPaintInvalidation();
    }

    if box_.MayNeedPaintInvalidationAnimatedBackgroundImage()
        && !box_.BackgroundIsKnownToBeObscured()
    {
        box_.SetBackgroundNeedsFullPaintInvalidation();
        box_.SetShouldDelayFullPaintInvalidation();
    }

    if box_.ShouldDelayFullPaintInvalidation() && box_.IntersectsVisibleViewport() {
        box_.ClearShouldDelayFullPaintInvalidation();
        debug_assert!(box_.ShouldDoFullPaintInvalidation());
    }
}

// Source: core/layout/layout_box.cc:4156-4204.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxBackgroundClipBorderBoxIsEquivalentToPaddingBoxProvider(
    box_: &LayoutBox,
) -> bool {
    box_.CheckIsNotDestroyed();
    let area = box_.GetScrollableArea();
    if !area.is_null() {
        for scrollbar in [
            unsafe { &*area }.HorizontalScrollbar(),
            unsafe { &*area }.VerticalScrollbar(),
        ] {
            if !scrollbar.is_null()
                && !unsafe { &*scrollbar }.IsOverlayScrollbar()
                && !unsafe { &*scrollbar }.IsOpaque()
            {
                return false;
            }
        }
    }

    let style = box_.StyleRef();
    let side_is_opaque = |width: i32,
                          color: &layoutng_style::css::style_color::StyleColor,
                          border_style: EBorderStyle| {
        width == 0
            || (style.ResolvedColor(color, None).IsOpaque() && border_style == EBorderStyle::kSolid)
    };
    side_is_opaque(
        style.BorderTopWidth(),
        style.BorderTopColor(),
        style.BorderTopStyle(),
    ) && side_is_opaque(
        style.BorderRightWidth(),
        style.BorderRightColor(),
        style.BorderRightStyle(),
    ) && side_is_opaque(
        style.BorderBottomWidth(),
        style.BorderBottomColor(),
        style.BorderBottomStyle(),
    ) && side_is_opaque(
        style.BorderLeftWidth(),
        style.BorderLeftColor(),
        style.BorderLeftStyle(),
    ) && style.IsScrollbarGutterAuto()
}
