// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/layout_view.h:75-95,132-135,210,417-419
// Source: third_party/blink/renderer/core/layout/layout_view.cc:160-231

use super::{
    hit_test_cache::HitTestCache,
    hit_test_location::HitTestLocation,
    hit_test_result::{HitTestResult, NodeRef, ScrollbarRef},
};
use foundation::{gfx, PhysicalOffset, PhysicalRect, PhysicalSize};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use std::cell::{Cell, RefCell};

/// Durable interaction state attached to one resident LayoutView by its Frame
/// owner. It contains the source cache and counters, never a geometry model.
#[derive(Default)]
pub struct LayoutViewHitTestState {
    hit_test_count: Cell<u32>,
    hit_test_cache_hits: Cell<u32>,
    pub(super) hit_test_cache: RefCell<HitTestCache<()>>,
    binding: Cell<Option<(usize, u64)>>,
}
impl LayoutViewHitTestState {
    pub fn HitTestCount(&self) -> u32 {
        self.hit_test_count.get()
    }
    pub fn HitTestCacheHits(&self) -> u32 {
        self.hit_test_cache_hits.get()
    }
    pub fn ClearHitTestCache(&self) {
        self.hit_test_cache.borrow_mut().Clear();
    }
    /// The embedding clears on source lifecycle invalidation. Revision binding
    /// also prevents a newly prepared lease from reusing a previous epoch.
    pub fn BindPreparedOwner(&self, identity: usize, revision: u64) -> bool {
        if let Some((owner, old_revision)) = self.binding.get() {
            if owner != identity {
                return false;
            }
            if old_revision != revision {
                self.ClearHitTestCache();
            }
        }
        self.binding.set(Some((identity, revision)));
        true
    }
}

/// Only the values read by LayoutView::HitTestNoLifecycleUpdate. The resident
/// adapter borrows the original frame/view/layer owners and performs no update.
pub trait LayoutViewHitTestSource<'r> {
    type Error;
    fn State(&self) -> &LayoutViewHitTestState;
    fn DomTreeVersion(&self) -> u64;
    fn FrameSize(&self) -> Option<gfx::Size>;
    fn DocumentRect(&self) -> PhysicalRect;
    fn DocumentToFrameOffset(&self) -> PhysicalOffset;
    fn HasActiveUnboundedElements(&self) -> bool;
    fn LayerHitTest(
        &self,
        location: &HitTestLocation,
        result: &mut HitTestResult<'r, ()>,
        area: &PhysicalRect,
    ) -> Result<bool, Self::Error>;
    /// Scrollbar::GetLayoutBox()->GetNode(), replacing a Document node by its
    /// actual documentElement. None is a real ownerless scrollbar, not an error.
    fn ScrollbarNode(&self, scrollbar: &ScrollbarRef) -> Result<Option<NodeRef>, Self::Error>;
}

// Source: layout_view.cc:160-231. Rect/list semantics are retained by the
// unchanged request and HitTestCache; only successful point hits enter cache.
pub fn HitTestNoLifecycleUpdate<'r, S: LayoutViewHitTestSource<'r>>(
    source: &S,
    location: &HitTestLocation,
    result: &mut HitTestResult<'r, ()>,
) -> Result<bool, S::Error> {
    let state = source.State();
    state
        .hit_test_count
        .set(state.hit_test_count.get().wrapping_add(1));
    let dom_tree_version = source.DomTreeVersion();
    let mut cache_result = result.clone();
    if state.hit_test_cache.borrow().LookupCachedResult(
        location,
        &mut cache_result,
        dom_tree_version,
    ) {
        state
            .hit_test_cache_hits
            .set(state.hit_test_cache_hits.get().wrapping_add(1));
        result.Assign(&cache_result);
        return Ok(true);
    }
    let mut area = PhysicalRect::default();
    if let Some(size) = source.FrameSize() {
        area.size = PhysicalSize::FromSize(&size);
        if result.GetHitTestRequest().IgnoreClipping() {
            let mut document_rect = source.DocumentRect();
            document_rect.Move(&source.DocumentToFrameOffset());
            area.Unite(&document_rect);
        } else if foundation::RuntimeEnabledFeatures::UnboundedElementEnabled()
            && source.HasActiveUnboundedElements()
        {
            area = PhysicalRect::from(foundation::InfiniteIntRect());
        }
    }
    let hit_layer = source.LayerHitTest(location, result, &area)?;
    if let Some(scrollbar) = result.GetScrollbar().cloned() {
        result.SetInnerNode(None);
        result.SetURLElement(None);
        if let Some(node) = source.ScrollbarNode(&scrollbar)? {
            result.SetURLElement(
                crate::core::dom::node_hit_test::EnclosingLinkEventParentOrSelf(&node),
            );
            result.SetInnerNode(Some(node));
        }
    }
    if hit_layer {
        state
            .hit_test_cache
            .borrow_mut()
            .AddCachedResult(location, result, dom_tree_version);
    }
    Ok(hit_layer)
}
