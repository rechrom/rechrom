//! Cull-rect lifecycle owned by PrePaint.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/cull_rect_updater.{h,cc}` and
//! `third_party/blink/renderer/platform/graphics/paint/cull_rect.{h,cc}`.
//!
//! Rechrom's package boundary exposes immutable source fragment paint input
//! instead of Blink's mutable `PaintLayer`/`FragmentData` graph. This owner
//! retains the same sliding-window state and performs the supported root and
//! scroll-container cull decisions. Unsupported printing, embedded-frame,
//! view-transition, canvas-subtree and unbounded-element branches remain
//! absent rather than being approximated here.

use layoutng_assembly::fragment_tree::FragmentPaintData;
use layoutng_assembly::internal::layout_input::Offset;
use paint_model::cull_rect::{ApplyScrollExpansion, IsSmallScroller, LocalPixelDistanceToExpand};
use paint_model::geometry_mapper::SourceToDestinationRect;
use paint_model::paint_property_tree::PropertyTreeState;
use paint_model::{CullRect, PaintRect, PaintResult};

/// Source: `cull_rect_updater.cc:31-36`.
///
/// The pinned desktop feature defaults use a DPR coefficient of one, so the
/// source expression `1 + (dpr - 1) * coefficient` reduces to DPR.
#[inline]
pub fn ExpansionRatio(fragment: &FragmentPaintData) -> f64 {
    fragment
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.device_pixel_ratio)
        .max(f64::MIN_POSITIVE)
}

#[inline]
fn ContainerRect(fragment: &FragmentPaintData) -> PaintRect {
    let size = fragment.paint.client_size.unwrap_or(fragment.size);
    PaintRect {
        x: 0.0,
        y: 0.0,
        width: size.width.max(0.0),
        height: size.height.max(0.0),
    }
}

#[inline]
fn ContentsRect(fragment: &FragmentPaintData) -> PaintRect {
    PaintRect {
        x: 0.0,
        y: 0.0,
        width: fragment.paint.scroll_size.width.max(0.0),
        height: fragment.paint.scroll_size.height.max(0.0),
    }
}

#[inline]
fn CanExpandForScroll(fragment: &FragmentPaintData) -> bool {
    let container = ContainerRect(fragment);
    let contents = ContentsRect(fragment);
    fragment.paint.scroll_container.is_some()
        && (contents.width > container.width || contents.height > container.height)
}

/// Source: `cull_rect.cc:48-68,75-103` with the pinned feature defaults.
#[inline]
pub fn PixelDistanceToExpand(fragment: &FragmentPaintData, root_scroller: bool) -> f64 {
    let ratio = ExpansionRatio(fragment);
    LocalPixelDistanceToExpand(
        IsSmallScroller(ContainerRect(fragment), root_scroller, ratio),
        ratio,
    )
    .0
}

/// Supported scroll branch of `CullRect::HasScrolledEnough`.
///
/// Source: `cull_rect_updater.cc:193-206` and `cull_rect.cc:507-530`.
#[inline]
pub fn HasScrolledEnough(fragment: &FragmentPaintData, current: Offset) -> bool {
    let Some(origin) = fragment.paint_cull_scroll_offset else {
        return false;
    };
    let container = ContainerRect(fragment);
    let contents = ContentsRect(fragment);
    let ratio = ExpansionRatio(fragment);
    let small = IsSmallScroller(container, fragment.paint.IsLayoutView(), ratio);
    let can_expand = CanExpandForScroll(fragment);
    let old = if can_expand {
        ApplyScrollExpansion(container, contents, (origin.x, origin.y), small, ratio).0
    } else {
        CullRect::FromRect(PaintRect {
            x: origin.x,
            y: origin.y,
            ..container
        })
    };
    old.HasScrolledEnough(
        (current.x - origin.x, current.y - origin.y),
        can_expand,
        small,
        ratio,
        contents,
    )
}

/// Result of Chromium's `ShouldUseInfiniteCullRect` decision. The second bit
/// is deliberately separate: only canvas subtrees and perspective propagate
/// the unconditional-infinite rule to descendants. A moving filter makes its
/// own layer infinite but does not bypass a descendant's local overflow clip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct InfiniteCullRectDecision {
    pub use_infinite: bool,
    pub subtree_should_use_infinite: bool,
}

/// Supported branches of `cull_rect_updater.cc:92-190`.
///
/// The global InfiniteCullRect runtime override, unclipped LocalFrame root,
/// ElementCanvasTransform and view-transition pseudo-element branches are not
/// exported by the current Layout boundary and remain explicitly unsupported
/// in the translation ledger. Previous-result repaint scheduling on an actual
/// cull change is handled by the retained walk after this updater returns.
pub(super) fn ShouldUseInfiniteCullRect(
    fragment: &FragmentPaintData,
    inherited_subtree_should_use_infinite: bool,
    has_filter_that_moves_pixels: bool,
    has_perspective: bool,
    has_active_transform_animation: bool,
    has_non_composited_transform: bool,
    previous_paint_result: PaintResult,
) -> InfiniteCullRectDecision {
    if inherited_subtree_should_use_infinite {
        return InfiniteCullRectDecision {
            use_infinite: true,
            subtree_should_use_infinite: true,
        };
    }
    if fragment.paint.IsInclusiveDescendantOfUnboundedElement() {
        return InfiniteCullRectDecision {
            use_infinite: true,
            subtree_should_use_infinite: false,
        };
    }
    if fragment.paint.CanvasDrawElementEnabled() && fragment.paint.IsInCanvasSubtree() {
        return InfiniteCullRectDecision {
            use_infinite: true,
            subtree_should_use_infinite: true,
        };
    }
    if fragment.paint.IsOverscrollAreaParent() {
        return InfiniteCullRectDecision {
            use_infinite: true,
            subtree_should_use_infinite: false,
        };
    }
    if has_filter_that_moves_pixels && !fragment.paint.IsPrinting() {
        return InfiniteCullRectDecision {
            use_infinite: true,
            subtree_should_use_infinite: false,
        };
    }
    if has_perspective {
        return InfiniteCullRectDecision {
            use_infinite: true,
            subtree_should_use_infinite: true,
        };
    }
    if has_active_transform_animation
        || (has_non_composited_transform && previous_paint_result == PaintResult::kFullyPainted)
    {
        return InfiniteCullRectDecision {
            use_infinite: true,
            subtree_should_use_infinite: false,
        };
    }
    InfiniteCullRectDecision::default()
}

/// PrePaint cull updater.
///
/// Blink stores retained cull rectangles on `FragmentData`; Rechrom's
/// `PaintTreeNode` is that retained fragment-local owner.
#[derive(Default)]
pub struct CullRectUpdater;

impl CullRectUpdater {
    /// Public lifecycle entry used by the browser's retained scroll state.
    ///
    /// Keeping this on the updater prevents the browser package from owning a
    /// second, divergent cull-refresh rule.
    #[inline]
    pub fn HasScrolledEnough(fragment: &FragmentPaintData, current: Offset) -> bool {
        HasScrolledEnough(fragment, current)
    }

    fn CullRectInState(
        stored_cull: Option<PaintRect>,
        root_state: &PropertyTreeState,
        state: &PropertyTreeState,
    ) -> CullRect {
        let Some(rect) = stored_cull else {
            return CullRect::Infinite();
        };
        if root_state
            .transform
            .lifecycle
            .same_node(&state.transform.lifecycle)
        {
            return CullRect::FromRect(rect);
        }
        CullRect::FromRect(
            SourceToDestinationRect(&root_state.transform, &state.transform, rect)
                .unwrap_or_default(),
        )
    }

    fn StoreCullRect(
        cull: CullRect,
        state: &PropertyTreeState,
        root_state: &PropertyTreeState,
    ) -> Option<PaintRect> {
        let rect = cull.Rect()?;
        if state
            .transform
            .lifecycle
            .same_node(&root_state.transform.lifecycle)
        {
            return Some(rect);
        }
        Some(
            SourceToDestinationRect(&state.transform, &root_state.transform, rect)
                .unwrap_or_default(),
        )
    }

    /// Source: `cull_rect_updater.cc:437-519`.
    ///
    /// Blink stores the result in the fragment's local property space. The
    /// current PaintTreeNode ABI stores cull rectangles in root space because
    /// its painters compare them with root-mapped visual rects; conversion at
    /// this boundary preserves the source update algorithm and package ABI.
    pub fn ComputeFragmentCullRect(
        parent_cull: Option<PaintRect>,
        old_cull: Option<Option<PaintRect>>,
        root_state: &PropertyTreeState,
        parent_state: &PropertyTreeState,
        local_state: &PropertyTreeState,
        expansion_ratio: f64,
    ) -> (Option<PaintRect>, bool) {
        let mut cull = Self::CullRectInState(parent_cull, root_state, parent_state);
        let mut expanded = false;
        if parent_state != local_state {
            let old = old_cull.map(|stored| Self::CullRectInState(stored, root_state, local_state));
            expanded = cull.ApplyPaintProperties(
                root_state,
                parent_state,
                local_state,
                old,
                expansion_ratio,
            );
        }
        (Self::StoreCullRect(cull, local_state, root_state), expanded)
    }

    /// Source: `cull_rect_updater.cc:521-541`.
    pub fn ComputeFragmentContentsCullRect(
        cull_rect: Option<PaintRect>,
        old_contents_cull: Option<Option<PaintRect>>,
        root_state: &PropertyTreeState,
        local_state: &PropertyTreeState,
        contents_state: &PropertyTreeState,
        expansion_ratio: f64,
    ) -> (Option<PaintRect>, bool) {
        let mut contents_cull = Self::CullRectInState(cull_rect, root_state, local_state);
        let mut expanded = false;
        if local_state != contents_state {
            let old = old_contents_cull
                .map(|stored| Self::CullRectInState(stored, root_state, contents_state));
            expanded = contents_cull.ApplyPaintProperties(
                root_state,
                local_state,
                contents_state,
                old,
                expansion_ratio,
            );
        }
        (
            Self::StoreCullRect(contents_cull, contents_state, root_state),
            expanded,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_and_perspective_propagate_but_filter_and_overscroll_do_not() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.canvas_draw_element_enabled = true;
        fragment.paint.is_in_canvas_subtree = true;
        let canvas = ShouldUseInfiniteCullRect(
            &fragment,
            false,
            false,
            false,
            false,
            false,
            PaintResult::kMayBeClippedByCullRect,
        );
        assert_eq!(
            canvas,
            InfiniteCullRectDecision {
                use_infinite: true,
                subtree_should_use_infinite: true,
            }
        );

        fragment.paint.is_in_canvas_subtree = false;
        fragment.paint.is_overscroll_area_parent = true;
        let overscroll = ShouldUseInfiniteCullRect(
            &fragment,
            false,
            false,
            false,
            false,
            false,
            PaintResult::kMayBeClippedByCullRect,
        );
        assert!(overscroll.use_infinite);
        assert!(!overscroll.subtree_should_use_infinite);

        fragment.paint.is_overscroll_area_parent = false;
        let filter = ShouldUseInfiniteCullRect(
            &fragment,
            false,
            true,
            false,
            false,
            false,
            PaintResult::kMayBeClippedByCullRect,
        );
        assert!(filter.use_infinite);
        assert!(!filter.subtree_should_use_infinite);

        let perspective = ShouldUseInfiniteCullRect(
            &fragment,
            false,
            false,
            true,
            false,
            false,
            PaintResult::kMayBeClippedByCullRect,
        );
        assert!(perspective.use_infinite);
        assert!(perspective.subtree_should_use_infinite);
    }

    #[test]
    fn printing_keeps_moving_filter_cull_finite() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.is_printing = true;
        assert_eq!(
            ShouldUseInfiniteCullRect(
                &fragment,
                false,
                true,
                false,
                false,
                false,
                PaintResult::kMayBeClippedByCullRect,
            ),
            InfiniteCullRectDecision::default()
        );
    }

    #[test]
    fn active_and_previously_fully_painted_non_composited_transforms_are_infinite() {
        let fragment = FragmentPaintData::default();
        let active = ShouldUseInfiniteCullRect(
            &fragment,
            false,
            false,
            false,
            true,
            false,
            PaintResult::kMayBeClippedByCullRect,
        );
        assert!(active.use_infinite);
        assert!(!active.subtree_should_use_infinite);

        let fully_painted = ShouldUseInfiniteCullRect(
            &fragment,
            false,
            false,
            false,
            false,
            true,
            PaintResult::kFullyPainted,
        );
        assert!(fully_painted.use_infinite);
        assert!(!fully_painted.subtree_should_use_infinite);

        assert_eq!(
            ShouldUseInfiniteCullRect(
                &fragment,
                false,
                false,
                false,
                false,
                true,
                PaintResult::kMayBeClippedByCullRect,
            ),
            InfiniteCullRectDecision::default()
        );
        assert_eq!(
            ShouldUseInfiniteCullRect(
                &fragment,
                false,
                false,
                false,
                false,
                false,
                PaintResult::kFullyPainted,
            ),
            InfiniteCullRectDecision::default()
        );
    }
}
