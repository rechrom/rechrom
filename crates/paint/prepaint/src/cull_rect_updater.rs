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

use layout::fragment::ResidentBoxGeometry as _;
use layout::paint::{FragmentPaintData, PaintFragmentCursor};
use layout::views::paint_layer::PaintLayer;
use paint_input_types::PaintResources;
use paint_input_types::layout_input::Offset;
use paint_model::cull_rect::{ApplyScrollExpansion, IsSmallScroller, LocalPixelDistanceToExpand};
use paint_model::paint_property_tree::{PaintPropertyChangeType, PropertyTreeState};
use paint_model::{CullRect, PaintRect, PaintResult};

use crate::fragment_data::PrePaintFragmentData;

/// Source: `cull_rect_updater.cc:31-36`.
///
/// The pinned desktop feature defaults use a DPR coefficient of one, so the
/// source expression `1 + (dpr - 1) * coefficient` reduces to DPR.
/// The device scale is LocalFrame::DevicePixelRatio, a document fact.
#[inline]
pub fn ExpansionRatio(resources: Option<&PaintResources>) -> f64 {
    resources
        .map_or(1.0, |resources| resources.device_pixel_ratio)
        .max(f64::MIN_POSITIVE)
}

#[inline]
fn ContainerRect(fragment: &FragmentPaintData) -> PaintRect {
    let size = fragment.ViewScrollGeometry().1.unwrap_or(fragment.ViewSize());
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
        width: fragment.ViewScrollGeometry().0.width.max(0.0),
        height: fragment.ViewScrollGeometry().0.height.max(0.0),
    }
}

#[inline]
fn CanExpandForScroll(fragment: &FragmentPaintData) -> bool {
    let container = ContainerRect(fragment);
    let contents = ContentsRect(fragment);
    fragment.ViewHasScrollContainer()
        && (contents.width > container.width || contents.height > container.height)
}

/// Source: `cull_rect.cc:48-68,75-103` with the pinned feature defaults.
#[inline]
pub fn PixelDistanceToExpand(
    fragment: &FragmentPaintData,
    resources: Option<&PaintResources>,
    root_scroller: bool,
) -> f64 {
    let ratio = ExpansionRatio(resources);
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
pub fn HasScrolledEnough(
    fragment: &FragmentPaintData,
    resources: Option<&PaintResources>,
    current: Offset,
) -> bool {
    let Some(origin) = fragment.paint.LastCullRectUpdateScrollPosition() else {
        return false;
    };
    let container = ContainerRect(fragment);
    let contents = ContentsRect(fragment);
    let ratio = ExpansionRatio(resources);
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

/// Inputs already computed by ObjectPaintProperties during the same PrePaint
/// lifecycle. Special recording reuses them without reinterpreting CSS.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CullRectDecisionInputs {
    pub has_filter_that_moves_pixels: bool,
    pub has_perspective: bool,
    pub has_active_transform_animation: bool,
    pub has_non_composited_transform: bool,
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
/// Blink stores retained cull rectangles on `FragmentData`. Rechrom's
/// `PaintTreeNode` still holds the rects Paint reads; `SetFragmentCullRects`
/// also stores them on the occurrence's `PrePaintFragmentData`.
#[derive(Default)]
pub struct CullRectUpdater;

impl CullRectUpdater {
    /// Public lifecycle entry used by the browser's retained scroll state.
    ///
    /// Keeping this on the updater prevents the browser package from owning a
    /// second, divergent cull-refresh rule.
    #[inline]
    pub fn HasScrolledEnough(
        fragment: &FragmentPaintData,
        resources: Option<&PaintResources>,
        current: Offset,
    ) -> bool {
        HasScrolledEnough(fragment, resources, current)
    }

    /// Source: `cull_rect_updater.cc:437-519`.
    ///
    /// Like FragmentData::GetCullRect(), the stored rectangle is in
    /// LocalBorderBoxProperties transform space. The incoming parent cull is
    /// already in `parent_state` because it is the parent's stored contents
    /// cull; retaining that invariant avoids a root-space proxy ABI.
    pub fn ComputeFragmentCullRect(
        parent_cull: Option<PaintRect>,
        old_cull: Option<Option<PaintRect>>,
        root_state: &PropertyTreeState,
        parent_state: &PropertyTreeState,
        local_state: &PropertyTreeState,
        expansion_ratio: f64,
    ) -> (Option<PaintRect>, bool) {
        let mut cull = parent_cull.map_or_else(CullRect::Infinite, CullRect::FromRect);
        let mut expanded = false;
        if parent_state != local_state {
            let old =
                old_cull.map(|stored| stored.map_or_else(CullRect::Infinite, CullRect::FromRect));
            expanded = paint_model::property_cell::with_access(|access| cull.ApplyPaintProperties(
                access,
                root_state,
                parent_state,
                local_state,
                old,
                expansion_ratio,
            ));
        }
        (cull.Rect(), expanded)
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
        let mut contents_cull = cull_rect.map_or_else(CullRect::Infinite, CullRect::FromRect);
        let mut expanded = false;
        if local_state != contents_state {
            let old = old_contents_cull
                .map(|stored| stored.map_or_else(CullRect::Infinite, CullRect::FromRect));
            expanded = paint_model::property_cell::with_access(|access| contents_cull.ApplyPaintProperties(
                access,
                root_state,
                local_state,
                contents_state,
                old,
                expansion_ratio,
            ));
        }
        (contents_cull.Rect(), expanded)
    }

    /// Source: `cull_rect_updater.cc:51-90` (SetFragmentCullRect,
    /// SetFragmentContentsCullRect) as `UpdateForSelf` (cc:393-435) applies
    /// them to each FragmentData of a PaintLayer's object. Returns whether
    /// the contents cull rect changed (UpdateForSelf's
    /// `force_update_children`).
    ///
    /// Dual write: the walk still computes the rects (PaintPropertyTreeBuilder,
    /// ShiftRetainedNode) and keeps them on PaintTreeNode, which Paint reads;
    /// this stores the same values (`None` is infinite) on the occurrence's
    /// FragmentData. As in Chromium only PaintLayer objects get cull rects
    /// here; every other FragmentData keeps its default empty CullRect
    /// (cull_rect.h). Not repeated here, so behaviour stays the walk's:
    /// SetLayerNeedsRepaintOnCullRectChange (cc:44-49), raised by the walk's
    /// CullRectChangeNeedsRepaint against the node's own retained rects, and
    /// PaintLayerScrollableArea::DidUpdateCullRect (cc:84-86). The
    /// OverriddenCullRectScope save/restore branch (`g_original_cull_rects`)
    /// belongs to special painting (special_paint.rs), which does not store.
    ///
    /// # Safety
    /// PrePaint is the single writer of the occurrence's PrePaint state
    /// (`PaintFragmentCursor::PrePaintDataMut`); no other reference to it may
    /// be live.
    pub(crate) unsafe fn SetFragmentCullRects(
        cursor: PaintFragmentCursor<'_>,
        cull_rect: Option<PaintRect>,
        contents_cull_rect: Option<PaintRect>,
    ) -> bool {
        // Hand-built fixtures have no result state and so no FragmentData.
        if cursor.PrePaintSlot().is_none() || cursor.OwnPaintLayer().is_none() {
            return false;
        }
        let fragment = unsafe { cursor.PrePaintDataMut::<PrePaintFragmentData>() };
        // cc:51-65 SetFragmentCullRect.
        let cull_rect = RetainedCullRect(cull_rect);
        if cull_rect != fragment.GetCullRect() {
            fragment.SetCullRect(cull_rect);
        }
        // cc:67-90 SetFragmentContentsCullRect.
        let contents_cull_rect = RetainedCullRect(contents_cull_rect);
        if contents_cull_rect == fragment.GetContentsCullRect() {
            return false;
        }
        fragment.SetContentsCullRect(contents_cull_rect);
        true
    }

    /// Source: `cull_rect_updater.cc:558-656` PaintPropertiesChanged: the
    /// PaintLayer cull-rect bits a property change of `cursor`'s object sets.
    ///
    /// `change` is the maximum change of this object's own property nodes
    /// (PaintPropertyTreeBuilder's `properties_changed_`); Rechrom keeps no
    /// per-tree split, so an effect-only change also counts, where Chromium
    /// reads only transform, clip and scroll (cc:589-596). Not translated
    /// yet: the HasScrolledEnough term (cc:596, still decided by the walk's
    /// retained scroll state), the scroll-change SetNeedsRepaint (cc:619-622,
    /// a repaint the walk does not issue today) and the LayoutView
    /// fixed-position children branch (cc:624-645). Nothing reads the bits
    /// for scheduling yet; FinishUpdate clears them after the walk.
    pub(crate) fn PaintPropertiesChanged(
        cursor: PaintFragmentCursor<'_>,
        fragment: &FragmentPaintData,
        change: PaintPropertyChangeType,
        inputs: CullRectDecisionInputs,
        previous_paint_result: PaintResult,
    ) {
        let layer = cursor.OwnPaintLayer();
        // cc:573-587: an infinite layer whose stored rects are already
        // infinite needs no update.
        if layer.is_some()
            && ShouldUseInfiniteCullRect(
                fragment,
                false,
                inputs.has_filter_that_moves_pixels,
                inputs.has_perspective,
                inputs.has_active_transform_animation,
                inputs.has_non_composited_transform,
                previous_paint_result,
            )
            .use_infinite
            && cursor
                .PrePaintData::<PrePaintFragmentData>()
                .is_some_and(|stored| {
                    stored.GetCullRect().IsInfinite() && stored.GetContentsCullRect().IsInfinite()
                })
        {
            return;
        }
        // cc:589-607.
        if change < PaintPropertyChangeType::ChangedOnlySimpleValues {
            return;
        }
        // cc:609-646.
        if let Some(layer) = layer {
            layer.SetNeedsCullRectUpdate();
            return;
        }
        // cc:649-655.
        if cursor.SourceHasChildren() {
            if let Some(enclosing) = cursor.EnclosingPaintLayer() {
                enclosing.SetForcesChildrenCullRectUpdate();
            }
        }
    }

    /// Source: `cull_rect_updater.cc:279-280,353-354`: CullRectUpdater::Update
    /// (run after PrePaintTreeWalk, LocalFrameView::RunPrePaintLifecyclePhase)
    /// clears each updated layer's bits after its subtree. Rechrom updated the
    /// cull rects during the walk itself, so this pass follows
    /// UpdateRecursively's traversal (cc:291-390) over the dirty bits only to
    /// clear them. `root` is the walk's LayoutView fragment.
    pub(crate) fn FinishUpdate(root: PaintFragmentCursor<'_>) {
        if let Some(layer) = root.OwnPaintLayer() {
            ClearCullRectUpdateRecursively(layer);
        }
    }
}

/// Rechrom's retained cull value (`None` is infinite) as FragmentData's
/// CullRect.
#[inline]
fn RetainedCullRect(rect: Option<PaintRect>) -> CullRect {
    rect.map_or_else(CullRect::Infinite, CullRect::FromRect)
}

/// UpdateRecursively / UpdateForDescendants reduced to their bit clearing.
fn ClearCullRectUpdateRecursively(layer: &PaintLayer) {
    // cc:293-294 returns before clearing.
    if layer.IsUnderSVGHiddenContainer() {
        return;
    }
    // cc:332-339 (the forced-children propagation only reaches layers whose
    // own bits are already covered by DescendantNeedsCullRectUpdate).
    if layer.ForcesChildrenCullRectUpdate() || layer.DescendantNeedsCullRectUpdate() {
        // cc:363-366: DisplayLockContext forces the update on unlock, so a
        // locked subtree keeps its bits.
        if !layer.GetLayoutObject().ChildPaintBlockedByDisplayLock() {
            let mut child = layer.FirstChild();
            while let Some(current) = unsafe { child.as_ref() } {
                ClearCullRectUpdateRecursively(current);
                child = current.NextSibling();
            }
        }
    }
    // cc:353-354.
    layer.ClearNeedsCullRectUpdate();
}

#[cfg(test)]
mod tests {
    use super::*;
    use paint_model::geometry_mapper::TranslationTransform;
    use paint_model::paint_property_tree::{ClipPaintPropertyNode, TransformPaintPropertyNode};
    use std::sync::Arc;

    #[test]
    fn paint_offset_translation_keeps_fragment_culls_in_local_property_space() {
        let root = PropertyTreeState::default();
        let translated = Arc::new(TransformPaintPropertyNode {
            lifecycle: Default::default(),
            id: 1,
            parent: Some(root.transform.clone()),
            matrix: TranslationTransform(0.0, 52.0),
            ..(*root.transform).clone()
        });
        let border = PropertyTreeState {
            transform: translated.clone(),
            clip: root.clip.clone(),
            effect: root.effect.clone(),
        };
        let mut overflow_clip = (*root.clip).clone();
        overflow_clip.lifecycle = Default::default();
        overflow_clip.id = 2;
        let state = overflow_clip.state_mut();
        state.parent = Some(root.clip.clone());
        state.local_transform_space = translated;
        state.rect = Some(PaintRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 52.0,
        });
        let overflow_clip = Arc::new(overflow_clip.with_layout_clips());
        let contents = PropertyTreeState {
            transform: border.transform.clone(),
            clip: overflow_clip,
            effect: root.effect.clone(),
        };
        let viewport = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 200.0,
        };

        let (self_cull, _) = CullRectUpdater::ComputeFragmentCullRect(
            Some(viewport),
            None,
            &root,
            &root,
            &border,
            1.0,
        );
        assert_eq!(
            self_cull,
            Some(PaintRect {
                x: 0.0,
                y: -52.0,
                width: 320.0,
                height: 200.0,
            })
        );

        let (contents_cull, _) = CullRectUpdater::ComputeFragmentContentsCullRect(
            self_cull, None, &root, &border, &contents, 1.0,
        );
        assert_eq!(
            contents_cull,
            Some(PaintRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 52.0,
            })
        );

        let (child_cull, _) = CullRectUpdater::ComputeFragmentCullRect(
            contents_cull,
            None,
            &root,
            &contents,
            &contents,
            1.0,
        );
        assert_eq!(child_cull, contents_cull);
    }

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
