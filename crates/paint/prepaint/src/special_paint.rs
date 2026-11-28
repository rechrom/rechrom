//! Source core/paint/cull_rect_updater.cc:232-277,437-541,667-697.
//! A synchronous scope over the committed FragmentData owners, reached
//! through the fragment cursor. Only the cull values changed by special
//! painting are saved; no source/tree/style copy.
use crate::cull_rect_updater::{CullRectUpdater, RetainedCullRect, ShouldUseInfiniteCullRect};
use crate::fragment_data::PrePaintFragmentData;
use crate::pre_paint_tree_walk::PrePaintTreeWalk;
use foundation::TrustedIdentityHashMap as HashMap;
use layout::input::Position;
use layout::paint::PaintFragmentCursor;
use paint_model::{
    paint_property_tree::{ClipPaintPropertyNode, PropertyTreeState},
    CullRect, PaintRect,
};
use std::sync::Arc;

/// Source cull_rect_updater.cc:51-90,667-697 `g_original_cull_rects`: the
/// PaintLayer FragmentData cull rects (cull rect, contents cull rect) a
/// special paint replaced, restored when the scope ends.
struct OriginalCullRects {
    cursor: PaintFragmentCursor<'static>,
    fragment_data: (CullRect, CullRect),
}

/// The occurrence's FragmentData for writing when CullRectUpdater stores cull
/// rects on it: only objects with a PaintLayer (SetFragmentCullRect,
/// cull_rect_updater.cc:51-90 via UpdateForSelf, cc:393-435). Hand-built
/// fixtures have no result state.
///
/// # Safety
/// The caller is the single writer of the occurrence's PrePaint state for
/// the returned borrow (`PaintFragmentCursor::PrePaintDataMut`).
unsafe fn LayerFragmentDataMut<'a>(
    cursor: PaintFragmentCursor<'a>,
) -> Option<&'a mut PrePaintFragmentData> {
    if cursor.PrePaintSlot().is_none() || cursor.OwnPaintLayer().is_none() {
        return None;
    }
    Some(unsafe { cursor.PrePaintDataMut::<PrePaintFragmentData>() })
}
#[derive(Clone)]
struct ContainerCull {
    state: PropertyTreeState,
    cull: Option<PaintRect>,
    subtree_infinite: bool,
    fragmented: bool,
}
#[derive(Clone)]
struct Context {
    current: ContainerCull,
    absolute: ContainerCull,
    fixed: ContainerCull,
}

struct OverriddenCullRectScope<'f> {
    original: Vec<OriginalCullRects>,
    /// The embedded LocalFrameViews' LayoutView by owner client: a frame's
    /// objects follow their owner's children, as in the PrePaint walk.
    frames: &'f HashMap<u64, PaintFragmentCursor<'static>>,
}
impl Drop for OverriddenCullRectScope<'_> {
    fn drop(&mut self) {
        for saved in self.original.iter().rev() {
            // WithSpecialPaint borrows PrePaint exclusively; the committed
            // walk's source lease keeps every occurrence alive until the
            // scope restores all values.
            if let Some(data) = unsafe { LayerFragmentDataMut(saved.cursor) } {
                let (cull_rect, contents_cull_rect) = saved.fragment_data;
                data.SetCullRect(cull_rect);
                data.SetContentsCullRect(contents_cull_rect);
            }
        }
    }
}
impl OverriddenCullRectScope<'_> {
    fn Override(
        &mut self,
        cursor: PaintFragmentCursor<'static>,
        context: &Context,
        root: &PropertyTreeState,
        view_cull: Option<PaintRect>,
        view_contents_clip: &Arc<ClipPaintPropertyNode>,
        starting: bool,
    ) {
        let fragment = cursor.PaintData();
        if fragment.paint.IsFragmentLessBox() || fragment.paint.IsSVGHiddenContainer() {
            return;
        }
        // Only occurrences a PrePaint walk produced FragmentData for.
        let Some(data) = cursor
            .PrePaintData::<PrePaintFragmentData>()
            .filter(|data| data.WasWalked())
        else {
            return;
        };
        // UpdateRecursively, source.cc:305-310. Containing-block contexts
        // remain distinct from the logical/stacking traversal parent.
        let mut context = context.clone();
        if !starting {
            match fragment.paint.Position() {
                Position::kAbsolute => context.current = context.absolute.clone(),
                Position::kFixed => context.current = context.fixed.clone(),
                _ => {}
            }
        }
        // The visited object's FragmentData (fragment_data.h:104-110,137-177)
        // supplies its property tree states and PaintOffsetTranslation, as
        // CullRectUpdater reads them (cc:437-541).
        let local_border_box = data.LocalBorderBoxProperties();
        let contents_state = data.ContentsProperties();
        let paint_offset_translation = data
            .PaintProperties()
            .and_then(|properties| properties.PaintOffsetTranslation())
            .cloned();
        // The containing-block and display-lock facts are the LayoutObject /
        // PaintLayer's own (cc:305-310,349-382), read live.
        let layer_facts = fragment.paint.LivePaintLayerFacts();
        let can_contain_absolute = fragment.paint.PaintLayerCanContainAbsolutePositionObjects();
        let can_contain_fixed = fragment.paint.PaintLayerCanContainFixedPositionObjects();
        let record = data.WalkRecord();
        let [has_filter_that_moves_pixels, has_perspective, has_active_transform_animation, has_non_composited_transform] =
            record.cull_decision_inputs;
        let infinite = ShouldUseInfiniteCullRect(
            fragment,
            context.current.subtree_infinite,
            has_filter_that_moves_pixels,
            has_perspective,
            has_active_transform_animation,
            has_non_composited_transform,
            // PaintLayer::PreviousPaintResult (cull_rect_updater.cc:221-230),
            // which PaintLayerPainter writes; an object without a painting
            // layer keeps the conservative result.
            if record.is_paint_layer {
                cursor
                    .OwnPaintLayer()
                    .map_or(paint_model::PaintResult::kMayBeClippedByCullRect, |layer| {
                        layer.PreviousPaintResult()
                    })
            } else {
                paint_model::PaintResult::kMayBeClippedByCullRect
            },
        );
        let self_cull = if infinite.use_infinite {
            None
        } else if starting {
            context.current.cull
        } else {
            // source.cc:417-421,497-509: a fragmented containing layer has
            // no single parent FragmentData; map an infinite cull from root.
            let (mut input, mut parent) = if context.current.fragmented {
                (None, root)
            } else {
                (context.current.cull, &context.current.state)
            };
            let mut local_state = local_border_box.clone();
            // source.cc:454-486: a viewport-fixed paint-offset translation
            // uses the viewport clip, ignoring movable non-container clips.
            if fragment.paint.Position() == Position::kFixed
                && paint_model::property_cell::with_access(|access| {
                    paint_offset_translation
                        .as_ref()
                        .and_then(|translation| translation.Parent(access))
                        .is_some_and(|parent| Arc::ptr_eq(parent, &root.transform))
                })
            {
                local_state.clip = view_contents_clip.clone();
                parent = root;
                input = view_cull;
            }

            CullRectUpdater::ComputeFragmentCullRect(input, None, root, parent, &local_state, 0.0).0
        };
        let contents = if infinite.use_infinite {
            None
        } else {
            CullRectUpdater::ComputeFragmentContentsCullRect(
                self_cull,
                None,
                root,
                &local_border_box,
                &contents_state,
                0.0,
            )
            .0
        };
        // cc:51-90 with g_is_updating_for_overridden_cull_rect: a PaintLayer
        // object's FragmentData cull rects are replaced (painters read them
        // there), saving the originals for the scope's destructor. Other
        // objects paint with their paint parent's contents cull rect.
        if let Some(data) = unsafe { LayerFragmentDataMut(cursor) } {
            let cull_rect = RetainedCullRect(self_cull);
            let contents_cull_rect = RetainedCullRect(contents);
            let original = (data.GetCullRect(), data.GetContentsCullRect());
            if original != (cull_rect, contents_cull_rect) {
                data.SetCullRect(cull_rect);
                data.SetContentsCullRect(contents_cull_rect);
                self.original.push(OriginalCullRects {
                    cursor,
                    fragment_data: original,
                });
            }
        }
        // source.cc:349-361,378-382. Starting layer can contain every
        // descendant for this override, independent of ordinary CSS ownership.
        context.current = ContainerCull {
            state: contents_state,
            cull: contents,
            subtree_infinite: infinite.subtree_should_use_infinite,
            fragmented: layer_facts.is_fragmented,
        };
        if starting || can_contain_absolute {
            context.absolute = context.current.clone();
        }
        if starting || can_contain_fixed {
            context.fixed = context.current.clone();
        }
        if layer_facts.child_paint_blocked_by_display_lock {
            return;
        }
        for child in cursor.Children() {
            self.Override(child, &context, root, view_cull, view_contents_clip, false);
        }
        if let Some(&frame) = self.frames.get(&fragment.paint.DisplayItemClientId()) {
            self.Override(frame, &context, root, view_cull, view_contents_clip, false);
        }
    }
}
impl PrePaintTreeWalk {
    /// One source PaintLayer's first FragmentData, found through the
    /// committed walk's occurrences. Zero requests the LocalFrameView paint
    /// root. `input_rect` is in this layer's LocalBorderBoxProperties
    /// transform space. The callback receives the layer's and the root's
    /// fragment cursors and cannot retain them beyond this scope.
    pub fn WithSpecialPaint<T>(
        &mut self,
        layer_client_id: u64,
        mut input_rect: PaintRect,
        offset_input_by_paint_offset: bool,
        query: impl for<'n> FnOnce(
            PaintFragmentCursor<'n>,
            PaintFragmentCursor<'n>,
            &'n PrePaintTreeWalk,
        ) -> T,
    ) -> Option<T> {
        let root = self.RetainedRoot()?;
        // SAFETY: the committed walk's source lease keeps the root's result
        // alive until the next walk, which cannot run while PrePaint is
        // borrowed by this scope.
        let root: PaintFragmentCursor<'static> = unsafe { std::mem::transmute(root) };
        let layer = if layer_client_id == 0 {
            root
        } else {
            let mut found = None;
            self.VisitWalkedOccurrences(root, &mut |cursor, data| {
                if found.is_none()
                    && data.WalkRecord().is_paint_layer
                    && cursor.PaintData().paint.PaintLayerClientId() == layer_client_id
                {
                    found = Some(cursor);
                }
            });
            // SAFETY: as `root`.
            unsafe { std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'static>>(found?) }
        };
        let root_state = self.viewport_properties_for_special_paint();
        let view_cull = self.viewport_cull_for_special_paint();
        let view_contents_clip = self.viewport_contents_clip_for_special_paint();
        let frames: HashMap<u64, PaintFragmentCursor<'static>> = self
            .LocalFrameViews()
            .map(|view| {
                // SAFETY: as `root`; the frames' leases are retained with it.
                let frame: PaintFragmentCursor<'static> = unsafe { std::mem::transmute(view.Root()) };
                (view.OwnerClient(), frame)
            })
            .collect();
        let mut scope = OverriddenCullRectScope {
            original: Vec::new(),
            frames: &frames,
        };
        if offset_input_by_paint_offset {
            // FirstFragment().PaintOffset() (data_transfer.cc:131-134), in
            // the flat replay space the cull rects and recorded ops use.
            if let Some(data) = layer.PrePaintData::<PrePaintFragmentData>() {
                let local = data.PaintOffset();
                let snap = data.FlatReplay().paint_snap_offset;
                input_rect.x += local.left.ToDouble() - snap.x;
                input_rect.y += local.top.ToDouble() - snap.y;
            }
        }
        // Source gfx::ToEnclosingRect, including gfx's clamped int geometry.
        let rect = foundation::gfx::ToEnclosingRect(foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(input_rect.x as f32, input_rect.y as f32),
            foundation::gfx::SizeF::new(input_rect.width as f32, input_rect.height as f32),
        ));
        let input_rect = PaintRect {
            x: rect.x() as f64,
            y: rect.y() as f64,
            width: rect.width() as f64,
            height: rect.height() as f64,
        };

        let current = ContainerCull {
            state: root_state.clone(),
            cull: Some(input_rect),
            subtree_infinite: false,
            fragmented: false,
        };
        let context = Context {
            current: current.clone(),
            absolute: current.clone(),
            fixed: current,
        };
        scope.Override(
            layer,
            &context,
            &root_state,
            if layer_client_id == 0 {
                Some(input_rect)
            } else {
                view_cull
            },
            &view_contents_clip,
            true,
        );
        // The walk is lent back read-only (its LocalFrameViews for the paint
        // walk); the scope holds cursors only.
        let result = query(layer, root, &*self);
        drop(scope);
        Some(result)
    }
}
