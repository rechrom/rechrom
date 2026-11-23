// Copyright 2015 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.{h,cc}.
use crate::adapter::{
    geometry::Rect,
    raster_invalidator::{
        ClientIsOldOrNew, DisplayItemClientId, RasterInvalidationItem, RasterInvalidationSink,
        RasterVisualRectMapper, RoundedOpaqueBacking,
    },
};
use foundation::{IsFullPaintInvalidationReason, PaintInvalidationReason as Reason};
use std::collections::HashMap;
// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.h:15-65.
// Source raw ranges/iterator addresses map to borrowed resident slices and
// stable positions in those slices, without copying items or their records.
pub struct DisplayItemRasterInvalidator<
    'a,
    I: RasterInvalidationItem,
    S: RasterInvalidationSink + ?Sized,
    M: RasterVisualRectMapper + ?Sized,
> {
    invalidator_: &'a mut S,
    old_display_items_: &'a [I],
    new_display_items_: &'a [I],
    old_image_content_revisions_: &'a [Vec<(u64, u64)>],
    new_image_content_revisions_: &'a [Vec<(u64, u64)>],
    mapper_: &'a M,
    old_display_items_index_: HashMap<DisplayItemClientId, Vec<usize>>,
}
impl<
        'a,
        I: RasterInvalidationItem,
        S: RasterInvalidationSink + ?Sized,
        M: RasterVisualRectMapper + ?Sized,
    > DisplayItemRasterInvalidator<'a, I, S, M>
{
    // Source: display_item_raster_invalidator.h:32-33.
    const kClientIsOld: ClientIsOldOrNew = ClientIsOldOrNew::kClientIsOld;
    const kClientIsNew: ClientIsOldOrNew = ClientIsOldOrNew::kClientIsNew;

    pub fn new(invalidator: &'a mut S, old: &'a [I], new: &'a [I], mapper: &'a M) -> Self {
        Self {
            invalidator_: invalidator,
            old_display_items_: old,
            new_display_items_: new,
            old_image_content_revisions_: &[],
            new_image_content_revisions_: &[],
            mapper_: mapper,
            old_display_items_index_: HashMap::new(),
        }
    }
    pub fn WithImageContentRevisions(
        mut self,
        old: &'a [Vec<(u64, u64)>],
        new: &'a [Vec<(u64, u64)>],
    ) -> Self {
        self.old_image_content_revisions_ = old;
        self.new_image_content_revisions_ = new;
        self
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:11-99.
    pub fn Generate(&mut self) {
        struct OldAndNewDisplayItems {
            old_visual_rect: Rect,
            new_visual_rect: Rect,
            reason: Reason,
        }
        impl Default for OldAndNewDisplayItems {
            fn default() -> Self {
                Self {
                    old_visual_rect: Rect::default(),
                    new_visual_rect: Rect::default(),
                    reason: Reason::kNone,
                }
            }
        }
        let mut clients: HashMap<DisplayItemClientId, OldAndNewDisplayItems> = HashMap::new();
        let mut matched = vec![false; self.old_display_items_.len()];
        let mut next = 0;
        let mut latest = next;
        for new_index in 0..self.new_display_items_.len() {
            let new = &self.new_display_items_[new_index];
            let old_index = self.MatchNewDisplayItemInOldChunk(new, &mut next);
            let Some(old_index) = old_index else {
                if new.DrawsContent() {
                    let value = clients.entry(new.ClientId()).or_default();
                    value.new_visual_rect.Union(new.VisualRect());
                    if value.reason == Reason::kNone {
                        value.reason = if new.IsCacheable() {
                            Reason::kAppeared
                        } else {
                            Reason::kUncacheable
                        };
                    }
                }
                continue;
            };
            let old = &self.old_display_items_[old_index];
            let mut reason = new.GetPaintInvalidationReason();
            // A culled DrawingDisplayItem keeps its semantic id and visual
            // rect, but its PaintRecord is empty.  When the interest rect
            // moves, the matching item can therefore appear or disappear
            // without Layout assigning a client invalidation reason.  Treat
            // that record-presence transition exactly like an unmatched
            // display item; otherwise ContentLayerClientImpl may retain the
            // preceding recording and newly exposed tiles raster as clear.
            // A source tombstone means this item was moved from the cache. Its
            // subtype bytes now belong to `new`, so the deliberately-cleared
            // base draws-content bit is not a content transition.
            if reason == Reason::kNone
                && !old.IsTombstone()
                && old.DrawsContent() != new.DrawsContent()
            {
                reason = if new.DrawsContent() {
                    Reason::kAppeared
                } else {
                    Reason::kDisappeared
                };
            }
            if reason == Reason::kNone
                && self
                    .old_image_content_revisions_
                    .get(old_index)
                    .map(Vec::as_slice)
                    .unwrap_or_default()
                    != self
                        .new_image_content_revisions_
                        .get(new_index)
                        .map(Vec::as_slice)
                        .unwrap_or_default()
            {
                reason = Reason::kImage;
            }
            if !IsFullPaintInvalidationReason(reason) && old_index < latest {
                reason = Reason::kReordered;
            }
            if reason != Reason::kNone && (old.DrawsContent() || new.DrawsContent()) {
                let value = clients.entry(new.ClientId()).or_default();
                if old.IsTombstone() || old.DrawsContent() {
                    value.old_visual_rect.Union(old.VisualRect());
                }
                if new.DrawsContent() {
                    value.new_visual_rect.Union(new.VisualRect());
                }
                value.reason = reason;
            }
            debug_assert!(!matched[old_index]);
            matched[old_index] = true;
            if old.IsTombstone() {
                latest = latest.max(old_index);
            }
        }
        for (i, old) in self.old_display_items_.iter().enumerate() {
            if matched[i] {
                continue;
            }
            if old.DrawsContent() || old.IsTombstone() {
                clients
                    .entry(old.ClientId())
                    .or_default()
                    .old_visual_rect
                    .Union(old.VisualRect());
            }
        }
        for (id, value) in clients {
            self.GenerateRasterInvalidation(
                id,
                value.old_visual_rect,
                value.new_visual_rect,
                value.reason,
            );
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:101-133.
    #[inline(always)]
    fn MatchNewDisplayItemInOldChunk(&mut self, new: &I, next: &mut usize) -> Option<usize> {
        if !new.IsCacheable() {
            return None;
        }
        while *next < self.old_display_items_.len() {
            let old = &self.old_display_items_[*next];
            if !old.IsCacheable() {
                *next += 1;
                continue;
            }
            if old.GetId() == new.GetId() {
                let matched = *next;
                *next += 1;
                return Some(matched);
            }
            self.old_display_items_index_
                .entry(old.ClientId())
                .or_default()
                .push(*next);
            *next += 1;
        }
        let entries = self.old_display_items_index_.get(&new.ClientId())?;
        for &i in entries {
            if self.old_display_items_[i].GetId() == new.GetId() {
                return Some(i);
            }
        }
        None
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:135-142.
    #[inline(always)]
    fn AddRasterInvalidation(
        &mut self,
        id: DisplayItemClientId,
        rect: Rect,
        reason: Reason,
        old: ClientIsOldOrNew,
    ) {
        let source_rect = rect;
        let mut mapped_rect = self.mapper_.MapVisualRect(source_rect);
        let inner_rect = self
            .mapper_
            .MapVisualRectWithoutRasterEffectOutset(source_rect);
        let mut requires_full_replay = false;
        let mut full_replay_rect = Rect::default();
        let has_rounded_clip = self.mapper_.HasRoundedClip();
        // A changed drawing can also change antialias coverage in an enclosing
        // decoration or rounded clip. Chromium invalidates the corresponding
        // display item as part of the same chunk. Preserve the same result when
        // Layout only marks the descendant.
        if !mapped_rect.IsEmpty()
            && IsFullPaintInvalidationReason(reason)
            && (has_rounded_clip || reason == Reason::kImage)
        {
            let items = if old == Self::kClientIsOld {
                self.old_display_items_
            } else {
                self.new_display_items_
            };
            let changed_index = items
                .iter()
                .position(|item| item.ClientId() == id)
                .unwrap_or(items.len());
            let counterpart_items = if old == Self::kClientIsOld {
                self.new_display_items_
            } else {
                self.old_display_items_
            };
            let mut backing_rect = Rect::default();
            let mut image_decoration_rect = Rect::default();
            for item in &items[..changed_index] {
                // Cached display items are represented in the new list by a
                // tombstone whose subtype bytes have moved out. Resolve that
                // stable id against the other list before consulting drawing
                // and opacity metadata.
                let resolved = if item.IsTombstone() {
                    counterpart_items
                        .iter()
                        .find(|candidate| candidate.GetId() == item.GetId())
                        .unwrap_or(item)
                } else {
                    item
                };
                let visual_rect = resolved.VisualRect();
                if !resolved.DrawsContent() || !visual_rect.Contains(source_rect) {
                    continue;
                }
                if has_rounded_clip && !resolved.RectKnownToBeOpaque().IsEmpty() {
                    backing_rect.Union(visual_rect);
                }
                // An image decode does not invalidate its containing CSS box
                // decoration in Rechrom's flat artifact. Replaying only the
                // image can start the partial-raster clip on the decoration's
                // antialiased border. Select the nearest containing decoration
                // so the border and image are quantized in the same context.
                if reason == Reason::kImage && resolved.IsBoxDecorationBackground() {
                    let visual_area = i64::from(visual_rect.width) * i64::from(visual_rect.height);
                    let selected_area = i64::from(image_decoration_rect.width)
                        * i64::from(image_decoration_rect.height);
                    if image_decoration_rect.IsEmpty() || visual_area < selected_area {
                        image_decoration_rect = visual_rect;
                    }
                }
            }
            if has_rounded_clip && !backing_rect.IsEmpty() {
                // Replay and copy the complete rounded backing.  The mapped
                // descendant can already include a raster-effect outset, but
                // that outset is not a substitute for the backing's border
                // coverage (notably for fractional CSS geometry).
                let mapped_backing = self.mapper_.MapVisualRect(backing_rect);
                mapped_rect.Union(mapped_backing);
                full_replay_rect.Union(mapped_backing);
                requires_full_replay = true;
            }
            if !image_decoration_rect.IsEmpty() {
                let mapped_decoration = self.mapper_.MapVisualRect(image_decoration_rect);
                mapped_rect.Union(mapped_decoration);
                full_replay_rect.Union(mapped_decoration);
                requires_full_replay = true;
            }
            if has_rounded_clip {
                if let Some(layer_backing) = self.invalidator_.FindRoundedOpaqueBacking(inner_rect)
                {
                    // A partial-raster clip that starts on any antialiased edge of
                    // the backing can quantize that edge differently from the
                    // original full-tile raster. This applies to integral edges as
                    // well as fractional right edges, and remains true when a
                    // shadow/surface sibling overlaps the backing. The raster
                    // backend intersects this replay region with the actual dirty
                    // rect before copying from its full-context scratch target, so
                    // requesting the complete backing does not overwrite clean
                    // pixels.
                    mapped_rect.Union(layer_backing.rect);
                    full_replay_rect.Union(layer_backing.rect);
                    requires_full_replay = true;
                }
                // An asynchronously decoded image is initially rastered as
                // empty content, then repainted inside its rounded overflow
                // clip. Preserve one pixel of AA context when no decoration
                // or opaque backing supplies the edge.
                if reason == Reason::kImage && self.mapper_.NeedsRoundedClipAAOutset() {
                    let mut rounded_clip_context = inner_rect;
                    rounded_clip_context.Inset(-1, -1, -1, -1);
                    mapped_rect.Union(rounded_clip_context);
                    full_replay_rect.Union(rounded_clip_context);
                    requires_full_replay = true;
                }
            }
        }
        let rect = self.invalidator_.ClipByLayerBounds(mapped_rect);
        let inner_rect = self.invalidator_.ClipByLayerBounds(inner_rect);
        if std::env::var_os("RECHROM_TRACE_PARTIAL_RASTER").is_some() {
            eprintln!(
                "display-item-invalidation id={} reason={:?} old={:?} source={:?} mapped={:?} clipped={:?}",
                id, reason, old, source_rect, mapped_rect, rect
            );
        }
        if requires_full_replay {
            if full_replay_rect.IsEmpty() {
                full_replay_rect = rect;
            }
            self.invalidator_
                .AddRasterInvalidationForFullReplay(
                    rect,
                    inner_rect,
                    full_replay_rect,
                    id,
                    reason,
                    old,
                );
        } else {
            self.invalidator_
                .AddRasterInvalidation(rect, id, reason, old);
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:144-187.
    #[inline(always)]
    fn GenerateRasterInvalidation(
        &mut self,
        id: DisplayItemClientId,
        old: Rect,
        new: Rect,
        mut reason: Reason,
    ) {
        if new.IsEmpty() {
            if !old.IsEmpty() {
                self.AddRasterInvalidation(
                    id,
                    old,
                    Reason::kDisappeared,
                    Self::kClientIsOld,
                );
            }
            return;
        }
        if old.IsEmpty() {
            self.AddRasterInvalidation(id, new, Reason::kAppeared, Self::kClientIsNew);
            return;
        }
        if reason == Reason::kJustCreated {
            self.AddRasterInvalidation(
                id,
                old,
                Reason::kDisappeared,
                Self::kClientIsOld,
            );
            self.AddRasterInvalidation(id, new, Reason::kAppeared, Self::kClientIsNew);
            return;
        }
        if !IsFullPaintInvalidationReason(reason) && (old.x != new.x || old.y != new.y) {
            reason = Reason::kLayout;
        }
        if IsFullPaintInvalidationReason(reason) {
            self.GenerateFullRasterInvalidation(id, old, new, reason);
            return;
        }
        debug_assert_eq!((old.x, old.y), (new.x, new.y));
        self.GenerateIncrementalRasterInvalidation(id, old, new);
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:219-238.
    #[inline(always)]
    fn GenerateIncrementalRasterInvalidation(
        &mut self,
        id: DisplayItemClientId,
        old: Rect,
        new: Rect,
    ) {
        debug_assert_eq!((old.x, old.y), (new.x, new.y));
        let right = ComputeRightDelta(new, old, new);
        if !right.IsEmpty() {
            self.AddRasterInvalidation(
                id,
                right,
                Reason::kIncremental,
                Self::kClientIsNew,
            );
        }
        let bottom = ComputeBottomDelta(new, old, new);
        if !bottom.IsEmpty() {
            self.AddRasterInvalidation(
                id,
                bottom,
                Reason::kIncremental,
                Self::kClientIsNew,
            );
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:240-254.
    #[inline(always)]
    fn GenerateFullRasterInvalidation(
        &mut self,
        id: DisplayItemClientId,
        old: Rect,
        new: Rect,
        reason: Reason,
    ) {
        if !new.Contains(old) {
            self.AddRasterInvalidation(id, old, reason, Self::kClientIsNew);
            if old.Contains(new) {
                return;
            }
        }
        self.AddRasterInvalidation(id, new, reason, Self::kClientIsNew);
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:189-217.
fn ComputeRightDelta(location: Rect, old: Rect, new: Rect) -> Rect {
    let delta = new.width - old.width;
    if delta > 0 {
        return Rect::new(location.x + old.width, location.y, delta, new.height);
    }
    if delta < 0 {
        return Rect::new(location.x + new.width, location.y, -delta, old.height);
    }
    Rect::default()
}
fn ComputeBottomDelta(location: Rect, old: Rect, new: Rect) -> Rect {
    let delta = new.height - old.height;
    if delta > 0 {
        return Rect::new(location.x, location.y + old.height, new.width, delta);
    }
    if delta < 0 {
        return Rect::new(location.x, location.y + new.height, old.width, -delta);
    }
    Rect::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct Item {
        id: u8,
        client: usize,
        rect: Rect,
        reason: Reason,
        draws: bool,
        cacheable: bool,
        tombstone: bool,
        opaque_rect: Rect,
        box_decoration: bool,
    }

    impl Item {
        fn drawing(id: u8, client: usize, rect: Rect) -> Self {
            Self {
                id,
                client,
                rect,
                reason: Reason::kNone,
                draws: true,
                cacheable: true,
                tombstone: false,
                opaque_rect: Rect::default(),
                box_decoration: false,
            }
        }

        fn tombstone(mut self) -> Self {
            self.draws = false;
            self.tombstone = true;
            self
        }
    }

    impl RasterInvalidationItem for Item {
        type Id = u8;
        fn GetId(&self) -> Self::Id {
            self.id
        }
        fn ClientId(&self) -> DisplayItemClientId {
            self.client
        }
        fn DrawsContent(&self) -> bool {
            self.draws
        }
        fn IsCacheable(&self) -> bool {
            self.cacheable
        }
        fn IsTombstone(&self) -> bool {
            self.tombstone
        }
        fn ScrollbarNeedsUpdateDisplay(&self) -> Option<bool> {
            None
        }
        fn VisualRect(&self) -> Rect {
            self.rect
        }
        fn RectKnownToBeOpaque(&self) -> Rect {
            self.opaque_rect
        }
        fn IsBoxDecorationBackground(&self) -> bool {
            self.box_decoration
        }
        fn GetPaintInvalidationReason(&self) -> Reason {
            self.reason
        }
    }

    #[derive(Default)]
    struct Sink {
        invalidations: Vec<(Rect, usize, Reason, ClientIsOldOrNew)>,
        full_replays: Vec<(Rect, Rect)>,
        replay_regions: Vec<Rect>,
        layer_backings: Vec<Rect>,
        overlapping_layer_backing: bool,
    }

    impl RasterInvalidationSink for Sink {
        fn ClipByLayerBounds(&self, rect: Rect) -> Rect {
            rect
        }
        fn FindRoundedOpaqueBacking(&self, rect: Rect) -> Option<RoundedOpaqueBacking> {
            let rect = self
                .layer_backings
                .iter()
                .copied()
                .filter(|backing| backing.Contains(rect))
                .min_by_key(|backing| i64::from(backing.width) * i64::from(backing.height))?;
            Some(RoundedOpaqueBacking {
                rect,
                has_overlapping_sibling: self.overlapping_layer_backing,
            })
        }
        fn AddRasterInvalidation(
            &mut self,
            rect: Rect,
            id: DisplayItemClientId,
            reason: Reason,
            old_or_new: ClientIsOldOrNew,
        ) {
            self.invalidations.push((rect, id, reason, old_or_new));
        }
        fn AddRasterInvalidationForFullReplay(
            &mut self,
            rect: Rect,
            inner: Rect,
            replay_rect: Rect,
            id: DisplayItemClientId,
            reason: Reason,
            old_or_new: ClientIsOldOrNew,
        ) {
            self.full_replays.push((rect, inner));
            self.replay_regions.push(replay_rect);
            self.AddRasterInvalidation(rect, id, reason, old_or_new);
        }
    }

    struct IdentityMapper;
    impl RasterVisualRectMapper for IdentityMapper {
        fn MapVisualRect(&self, rect: Rect) -> Rect {
            rect
        }
    }

    struct RoundedMapper;
    impl RasterVisualRectMapper for RoundedMapper {
        fn MapVisualRect(&self, rect: Rect) -> Rect {
            rect
        }
        fn NeedsRoundedClipAAOutset(&self) -> bool {
            true
        }
        fn HasRoundedClip(&self) -> bool {
            true
        }
    }

    struct RoundedRasterOutsetMapper;
    impl RasterVisualRectMapper for RoundedRasterOutsetMapper {
        fn MapVisualRect(&self, mut rect: Rect) -> Rect {
            rect.Inset(-1, -1, -1, -1);
            rect
        }
        fn MapVisualRectWithoutRasterEffectOutset(&self, rect: Rect) -> Rect {
            rect
        }
        fn HasRoundedClip(&self) -> bool {
            true
        }
    }

    struct FractionalRightRoundedMapper;
    impl RasterVisualRectMapper for FractionalRightRoundedMapper {
        fn MapVisualRect(&self, rect: Rect) -> Rect {
            rect
        }
        fn HasRoundedClip(&self) -> bool {
            true
        }
        fn RoundedClipHasFractionalRightEdge(&self) -> bool {
            true
        }
    }

    fn generate(old: &[Item], new: &[Item]) -> Sink {
        let mut sink = Sink::default();
        DisplayItemRasterInvalidator::new(&mut sink, old, new, &IdentityMapper).Generate();
        sink
    }

    #[test]
    fn cached_tombstones_preserve_source_reorder_detection() {
        let first_rect = Rect::new(100, 100, 100, 100);
        let second_rect = Rect::new(100, 100, 50, 200);
        let old = [
            Item::drawing(1, 1, first_rect).tombstone(),
            Item::drawing(2, 2, second_rect).tombstone(),
        ];
        let new = [
            Item::drawing(2, 2, second_rect),
            Item::drawing(1, 1, first_rect),
        ];
        let result = generate(&old, &new);
        assert_eq!(
            result.invalidations,
            vec![(
                first_rect,
                1,
                Reason::kReordered,
                ClientIsOldOrNew::kClientIsNew
            )]
        );
    }

    #[test]
    fn appearance_and_disappearance_use_old_and_new_client_sides() {
        let old_rect = Rect::new(1, 2, 30, 40);
        let appeared = generate(&[], &[Item::drawing(1, 7, old_rect)]);
        assert_eq!(
            appeared.invalidations,
            vec![(
                old_rect,
                7,
                Reason::kAppeared,
                ClientIsOldOrNew::kClientIsNew
            )]
        );
        let disappeared = generate(&[Item::drawing(1, 7, old_rect)], &[]);
        assert_eq!(
            disappeared.invalidations,
            vec![(
                old_rect,
                7,
                Reason::kDisappeared,
                ClientIsOldOrNew::kClientIsOld
            )]
        );
    }

    #[test]
    fn same_origin_size_change_generates_source_right_and_bottom_deltas() {
        let old = [Item::drawing(1, 3, Rect::new(10, 20, 30, 40))];
        let mut resized = Item::drawing(1, 3, Rect::new(10, 20, 35, 46));
        resized.reason = Reason::kIncremental;
        let new = [resized];
        let result = generate(&old, &new);
        assert_eq!(
            result.invalidations,
            vec![
                (
                    Rect::new(40, 20, 5, 46),
                    3,
                    Reason::kIncremental,
                    ClientIsOldOrNew::kClientIsNew
                ),
                (
                    Rect::new(10, 60, 35, 6),
                    3,
                    Reason::kIncremental,
                    ClientIsOldOrNew::kClientIsNew
                )
            ]
        );
    }

    #[test]
    fn full_invalidation_unions_all_items_for_one_client() {
        let first = Rect::new(100, 100, 300, 300);
        let second = Rect::new(100, 150, 300, 300);
        let old = [Item::drawing(1, 8, first), Item::drawing(2, 8, second)];
        let mut new = old.clone();
        for item in &mut new {
            item.reason = Reason::kLayout;
        }
        let result = generate(&old, &new);
        assert_eq!(
            result.invalidations,
            vec![(
                Rect::new(100, 100, 300, 350),
                8,
                Reason::kLayout,
                ClientIsOldOrNew::kClientIsNew
            )]
        );
    }

    #[test]
    fn full_invalidation_without_backing_stays_inside_rounded_clip() {
        let rect = Rect::new(10, 20, 30, 40);
        let old = [Item::drawing(1, 8, rect)];
        let mut changed = Item::drawing(1, 8, rect);
        changed.reason = Reason::kLayout;
        let mut sink = Sink::default();
        DisplayItemRasterInvalidator::new(&mut sink, &old, &[changed], &RoundedMapper).Generate();
        assert_eq!(
            sink.invalidations,
            vec![(
                rect,
                8,
                Reason::kLayout,
                ClientIsOldOrNew::kClientIsNew
            )]
        );
        assert!(sink.full_replays.is_empty());
    }

    #[test]
    fn decoded_image_replays_rounded_clip_aa_context_without_opaque_backing() {
        let rect = Rect::new(10, 20, 30, 40);
        let old = [Item::drawing(1, 8, rect)];
        let mut changed = Item::drawing(1, 8, rect);
        changed.reason = Reason::kImage;
        let mut sink = Sink::default();
        DisplayItemRasterInvalidator::new(&mut sink, &old, &[changed], &RoundedMapper).Generate();
        let context = Rect::new(9, 19, 32, 42);
        assert_eq!(
            sink.invalidations,
            vec![(
                context,
                8,
                Reason::kImage,
                ClientIsOldOrNew::kClientIsNew
            )]
        );
        assert_eq!(sink.full_replays, vec![(context, rect)]);
        assert_eq!(sink.replay_regions, vec![context]);
    }

    #[test]
    fn decoded_image_replays_containing_box_decoration_without_rounded_clip() {
        let decoration_rect = Rect::new(9, 19, 32, 42);
        let image_rect = Rect::new(10, 20, 30, 40);
        let mut decoration = Item::drawing(1, 7, decoration_rect);
        decoration.box_decoration = true;
        let old = [decoration.clone(), Item::drawing(2, 8, image_rect)];
        let mut changed = Item::drawing(2, 8, image_rect);
        changed.reason = Reason::kImage;
        let new = [decoration, changed];
        let mut sink = Sink::default();
        DisplayItemRasterInvalidator::new(&mut sink, &old, &new, &IdentityMapper).Generate();
        assert_eq!(
            sink.invalidations,
            vec![(
                decoration_rect,
                8,
                Reason::kImage,
                ClientIsOldOrNew::kClientIsNew
            )]
        );
        assert_eq!(sink.full_replays, vec![(decoration_rect, image_rect)]);
        assert_eq!(sink.replay_regions, vec![decoration_rect]);
    }

    #[test]
    fn full_invalidation_replays_rounded_opaque_backing() {
        let backing_rect = Rect::new(9, 19, 32, 42);
        let changed_rect = Rect::new(10, 20, 30, 40);
        let mut backing = Item::drawing(1, 7, backing_rect);
        backing.opaque_rect = Rect::new(12, 23, 24, 34);
        let old = [backing.clone(), Item::drawing(2, 8, changed_rect)];
        let mut changed = Item::drawing(2, 8, changed_rect);
        changed.reason = Reason::kLayout;
        let new = [backing, changed];
        let mut sink = Sink::default();
        DisplayItemRasterInvalidator::new(&mut sink, &old, &new, &RoundedMapper).Generate();
        assert_eq!(
            sink.invalidations,
            vec![(
                backing_rect,
                8,
                Reason::kLayout,
                ClientIsOldOrNew::kClientIsNew
            )]
        );
        assert_eq!(sink.full_replays, vec![(backing_rect, changed_rect)]);
    }

    #[test]
    fn full_invalidation_uses_smallest_cross_chunk_opaque_backing() {
        let layer_rect = Rect::new(0, 0, 100, 100);
        let backing_rect = Rect::new(9, 19, 32, 42);
        let changed_rect = Rect::new(10, 20, 30, 40);
        let old = [Item::drawing(2, 8, changed_rect)];
        let mut changed = Item::drawing(2, 8, changed_rect);
        changed.reason = Reason::kLayout;
        let mut sink = Sink {
            layer_backings: vec![layer_rect, backing_rect],
            ..Sink::default()
        };
        DisplayItemRasterInvalidator::new(&mut sink, &old, &[changed], &RoundedMapper).Generate();
        assert_eq!(sink.full_replays, vec![(backing_rect, changed_rect)]);
        assert_eq!(sink.invalidations[0].0, backing_rect);
    }

    #[test]
    fn overlapping_cross_chunk_backing_replays_complete_fractional_backing() {
        let backing_rect = Rect::new(9, 19, 32, 42);
        let changed_rect = Rect::new(10, 20, 30, 40);
        let old = [Item::drawing(2, 8, changed_rect)];
        let mut changed = Item::drawing(2, 8, changed_rect);
        changed.reason = Reason::kLayout;
        let mut sink = Sink {
            layer_backings: vec![backing_rect],
            overlapping_layer_backing: true,
            ..Sink::default()
        };
        DisplayItemRasterInvalidator::new(
            &mut sink,
            &old,
            &[changed],
            &FractionalRightRoundedMapper,
        )
        .Generate();
        assert_eq!(sink.full_replays, vec![(backing_rect, changed_rect)]);
        assert_eq!(sink.invalidations[0].0, backing_rect);
        assert_eq!(sink.replay_regions, vec![backing_rect]);
    }

    #[test]
    fn overlapping_cross_chunk_backing_replays_complete_integral_backing() {
        let backing_rect = Rect::new(9, 19, 32, 42);
        let changed_rect = Rect::new(10, 20, 30, 40);
        let old = [Item::drawing(2, 8, changed_rect)];
        let mut changed = Item::drawing(2, 8, changed_rect);
        changed.reason = Reason::kLayout;
        let mut sink = Sink {
            layer_backings: vec![backing_rect],
            overlapping_layer_backing: true,
            ..Sink::default()
        };
        DisplayItemRasterInvalidator::new(&mut sink, &old, &[changed], &RoundedMapper).Generate();
        assert_eq!(sink.full_replays, vec![(backing_rect, changed_rect)]);
        assert_eq!(sink.invalidations[0].0, backing_rect);
        assert_eq!(sink.replay_regions, vec![backing_rect]);
    }

    #[test]
    fn cached_backing_tombstone_uses_old_opaque_metadata() {
        let backing_rect = Rect::new(9, 19, 32, 42);
        let changed_rect = Rect::new(10, 20, 30, 40);
        let mut backing = Item::drawing(1, 7, backing_rect);
        backing.opaque_rect = Rect::new(12, 23, 24, 34);
        let old = [backing.clone(), Item::drawing(2, 8, changed_rect)];
        let mut cached_backing = backing.tombstone();
        cached_backing.opaque_rect = Rect::default();
        let mut changed = Item::drawing(2, 8, changed_rect);
        changed.reason = Reason::kLayout;
        let new = [cached_backing, changed];
        let mut sink = Sink::default();
        DisplayItemRasterInvalidator::new(&mut sink, &old, &new, &RoundedMapper).Generate();
        assert_eq!(sink.full_replays, vec![(backing_rect, changed_rect)]);
        assert_eq!(sink.invalidations[0].0, backing_rect);
    }

    #[test]
    fn existing_raster_outset_still_requests_rounded_backing_replay() {
        let backing_rect = Rect::new(9, 19, 32, 42);
        let changed_rect = Rect::new(10, 20, 30, 40);
        let mut backing = Item::drawing(1, 7, backing_rect);
        backing.opaque_rect = Rect::new(12, 23, 24, 34);
        let old = [backing.clone(), Item::drawing(2, 8, changed_rect)];
        let mut changed = Item::drawing(2, 8, changed_rect);
        changed.reason = Reason::kLayout;
        let new = [backing, changed];
        let mut sink = Sink::default();
        DisplayItemRasterInvalidator::new(
            &mut sink,
            &old,
            &new,
            &RoundedRasterOutsetMapper,
        )
        .Generate();
        let mapped_backing_rect = Rect::new(8, 18, 34, 44);
        assert_eq!(
            sink.full_replays,
            vec![(mapped_backing_rect, changed_rect)]
        );
        assert_eq!(sink.invalidations[0].0, mapped_backing_rect);
    }

    #[test]
    fn just_created_never_treats_reused_client_address_as_the_old_client() {
        let old_rect = Rect::new(10, 20, 30, 40);
        let new_rect = Rect::new(50, 60, 70, 80);
        let old = [Item::drawing(1, 11, old_rect)];
        let mut replacement = Item::drawing(1, 11, new_rect);
        replacement.reason = Reason::kJustCreated;
        let result = generate(&old, &[replacement]);
        assert_eq!(
            result.invalidations,
            vec![
                (
                    old_rect,
                    11,
                    Reason::kDisappeared,
                    ClientIsOldOrNew::kClientIsOld
                ),
                (
                    new_rect,
                    11,
                    Reason::kAppeared,
                    ClientIsOldOrNew::kClientIsNew
                )
            ]
        );
    }

    #[test]
    fn retained_image_revision_change_stays_symmetric_without_touching_tombstones() {
        let rect = Rect::new(5, 6, 20, 30);
        let old = Item::drawing(1, 4, rect);
        let new = Item::drawing(1, 4, rect);
        let old_revisions = vec![vec![(9, 1)]];
        let new_revisions = vec![vec![(9, 2)]];
        let run = |old_items: &[Item],
                   new_items: &[Item],
                   old_revisions: &[Vec<(u64, u64)>],
                   new_revisions: &[Vec<(u64, u64)>]| {
            let mut sink = Sink::default();
            DisplayItemRasterInvalidator::new(
                &mut sink,
                old_items,
                new_items,
                &IdentityMapper,
            )
            .WithImageContentRevisions(old_revisions, new_revisions)
            .Generate();
            sink
        };
        let forward = run(
            std::slice::from_ref(&old),
            std::slice::from_ref(&new),
            &old_revisions,
            &new_revisions,
        );
        let backward = run(
            std::slice::from_ref(&new),
            std::slice::from_ref(&old),
            &new_revisions,
            &old_revisions,
        );
        for result in [forward, backward] {
            assert_eq!(
                result.invalidations,
                vec![(
                    rect,
                    4,
                    Reason::kImage,
                    ClientIsOldOrNew::kClientIsNew
                )]
            );
        }
    }
}
