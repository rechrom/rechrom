// Translation status: complete
// Copyright 2015 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.h:1-68;
// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:1-10.
use crate::adapter::{
    geometry::Rect,
    raster_invalidator::{
        ClientIsOldOrNew, DisplayItemClientId, RasterInvalidationItem, RasterInvalidationSink,
        RasterVisualRectMapper,
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
    pub fn new(invalidator: &'a mut S, old: &'a [I], new: &'a [I], mapper: &'a M) -> Self {
        Self {
            invalidator_: invalidator,
            old_display_items_: old,
            new_display_items_: new,
            mapper_: mapper,
            old_display_items_index_: HashMap::new(),
        }
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
            let mut reason = new.GetPaintInvalidationReason();
            if !IsFullPaintInvalidationReason(reason) && old_index < latest {
                reason = Reason::kReordered;
            }
            let old = &self.old_display_items_[old_index];
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
    fn AddRasterInvalidation(
        &mut self,
        id: DisplayItemClientId,
        rect: Rect,
        reason: Reason,
        old: ClientIsOldOrNew,
    ) {
        let rect = self
            .invalidator_
            .ClipByLayerBounds(self.mapper_.MapVisualRect(rect));
        self.invalidator_
            .AddRasterInvalidation(rect, id, reason, old);
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:144-187.
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
                    ClientIsOldOrNew::kClientIsOld,
                );
            }
            return;
        }
        if old.IsEmpty() {
            self.AddRasterInvalidation(id, new, Reason::kAppeared, ClientIsOldOrNew::kClientIsNew);
            return;
        }
        if reason == Reason::kJustCreated {
            self.AddRasterInvalidation(
                id,
                old,
                Reason::kDisappeared,
                ClientIsOldOrNew::kClientIsOld,
            );
            self.AddRasterInvalidation(id, new, Reason::kAppeared, ClientIsOldOrNew::kClientIsNew);
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
                ClientIsOldOrNew::kClientIsNew,
            );
        }
        let bottom = ComputeBottomDelta(new, old, new);
        if !bottom.IsEmpty() {
            self.AddRasterInvalidation(
                id,
                bottom,
                Reason::kIncremental,
                ClientIsOldOrNew::kClientIsNew,
            );
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:240-254.
    fn GenerateFullRasterInvalidation(
        &mut self,
        id: DisplayItemClientId,
        old: Rect,
        new: Rect,
        reason: Reason,
    ) {
        if !new.Contains(old) {
            self.AddRasterInvalidation(id, old, reason, ClientIsOldOrNew::kClientIsNew);
            if old.Contains(new) {
                return;
            }
        }
        self.AddRasterInvalidation(id, new, reason, ClientIsOldOrNew::kClientIsNew);
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

// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:100,134,143,188,218,239.
// Source function separators have no runtime instructions.

// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:134.
// Definition separator has no runtime instructions.

// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:143.
// Definition separator has no runtime instructions.

// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:188.
// Definition separator has no runtime instructions.

// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:218.
// Definition separator has no runtime instructions.

// Source: third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.cc:239.
// Definition separator has no runtime instructions.
