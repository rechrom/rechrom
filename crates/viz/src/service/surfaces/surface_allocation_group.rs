// Source: components/viz/service/surfaces/surface_allocation_group.h:1-30
// Source: components/viz/service/surfaces/surface_allocation_group.cc:1-16
// C++ ownership pointers are stable SurfaceIds in the Rust owner graph. Client
// callbacks are returned as typed effects so SurfaceManager applies them after
// releasing the allocation-group borrow.
#![allow(non_snake_case)]

use crate::common::surfaces::{
    frame_sink_id::FrameSinkId, local_surface_id::EmbedToken, surface_id::SurfaceId,
    surface_range::SurfaceRange,
};
use std::collections::{BTreeMap, BTreeSet};

// Source: components/viz/service/surfaces/surface_allocation_group.h:31-40
// Source: components/viz/service/surfaces/surface_allocation_group.cc:17-29
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationGroupEffect {
    ActivateForInheritedDeadline(SurfaceId),
    ResolveActivationDependency {
        embedder: SurfaceId,
        dependency: SurfaceId,
    },
    NotifyChildActivated {
        embedder: SurfaceId,
        child: SurfaceId,
    },
    SetFallback(SurfaceId),
    SendAck(SurfaceId),
    MarkForDestruction,
}

// Source: components/viz/service/surfaces/surface_allocation_group.h:134-187
#[derive(Default)]
pub struct SurfaceAllocationGroup {
    submitter_: FrameSinkId,
    embed_token_: EmbedToken,
    surfaces_: Vec<SurfaceId>,
    active_surfaces_: BTreeSet<SurfaceId>,
    blocked_embedders_: BTreeMap<SurfaceId, SurfaceId>,
    active_embedders_: BTreeSet<SurfaceId>,
    last_active_reference_: Option<SurfaceId>,
    last_reference_: Option<SurfaceId>,
    will_not_register_new_surfaces_: bool,
}

impl SurfaceAllocationGroup {
    // Source: components/viz/service/surfaces/surface_allocation_group.h:31-50
    // Source: components/viz/service/surfaces/surface_allocation_group.cc:17-35
    pub fn new(submitter: FrameSinkId, embed_token: EmbedToken) -> Self {
        Self {
            submitter_: submitter,
            embed_token_: embed_token,
            ..Self::default()
        }
    }
    pub fn submitter_frame_sink_id(&self) -> FrameSinkId {
        self.submitter_
    }
    pub fn embed_token(&self) -> EmbedToken {
        self.embed_token_
    }
    pub fn IsReadyToDestroy(&self) -> bool {
        self.surfaces_.is_empty()
            && self.active_embedders_.is_empty()
            && self.blocked_embedders_.is_empty()
    }

    // Source: components/viz/service/surfaces/surface_allocation_group.h:51-69
    // Source: components/viz/service/surfaces/surface_allocation_group.cc:36-68
    pub fn RegisterSurface(&mut self, surface: SurfaceId) {
        assert_eq!(surface.frame_sink_id(), self.submitter_);
        assert_eq!(surface.local_surface_id().embed_token(), self.embed_token_);
        assert!(self
            .surfaces_
            .last()
            .is_none_or(|last| surface.IsNewerThan(*last)));
        self.surfaces_.push(surface);
    }
    pub fn UnregisterSurface(&mut self, surface: SurfaceId) -> Option<AllocationGroupEffect> {
        let index = self
            .surfaces_
            .iter()
            .position(|candidate| *candidate == surface)
            .expect("registered surface");
        self.surfaces_.remove(index);
        self.active_surfaces_.remove(&surface);
        self.IsReadyToDestroy()
            .then_some(AllocationGroupEffect::MarkForDestruction)
    }
    pub fn RegisterBlockedEmbedder(&mut self, embedder: SurfaceId, dependency: SurfaceId) {
        self.blocked_embedders_.insert(embedder, dependency);
    }
    pub fn UnregisterBlockedEmbedder(
        &mut self,
        embedder: SurfaceId,
        did_activate: bool,
    ) -> Option<AllocationGroupEffect> {
        assert!(self.blocked_embedders_.remove(&embedder).is_some());
        (!did_activate && self.IsReadyToDestroy())
            .then_some(AllocationGroupEffect::MarkForDestruction)
    }

    // Source: components/viz/service/surfaces/surface_allocation_group.h:70-82
    // Source: components/viz/service/surfaces/surface_allocation_group.cc:69-83
    pub fn HasBlockedEmbedder(&self) -> bool {
        !self.blocked_embedders_.is_empty()
    }
    pub fn RegisterActiveEmbedder(&mut self, surface: SurfaceId) {
        assert!(self.active_embedders_.insert(surface));
    }
    pub fn UnregisterActiveEmbedder(
        &mut self,
        surface: SurfaceId,
    ) -> Option<AllocationGroupEffect> {
        assert!(self.active_embedders_.remove(&surface));
        self.IsReadyToDestroy()
            .then_some(AllocationGroupEffect::MarkForDestruction)
    }

    // Source: components/viz/service/surfaces/surface_allocation_group.h:83-105
    // Source: components/viz/service/surfaces/surface_allocation_group.cc:84-150
    pub fn UpdateLastActiveReferenceAndMaybeActivate(
        &mut self,
        surface_id: SurfaceId,
    ) -> Vec<AllocationGroupEffect> {
        assert_eq!(self.submitter_, surface_id.frame_sink_id());
        assert_eq!(
            self.embed_token_,
            surface_id.local_surface_id().embed_token()
        );
        if self
            .last_active_reference_
            .is_some_and(|last| last.IsSameOrNewerThan(surface_id))
        {
            return Vec::new();
        }
        self.last_active_reference_ = Some(surface_id);
        let mut effects = Vec::new();
        if let Some(candidate) = self.FindLatestSurfaceUpTo(surface_id) {
            if !self.active_surfaces_.contains(&candidate) {
                effects.push(AllocationGroupEffect::ActivateForInheritedDeadline(
                    candidate,
                ));
            }
        }
        effects.extend(self.UpdateLastReferenceAndMaybeActivate(surface_id));
        effects
    }
    pub fn UpdateLastPendingReferenceAndMaybeActivate(
        &mut self,
        surface_id: SurfaceId,
    ) -> Vec<AllocationGroupEffect> {
        self.UpdateLastReferenceAndMaybeActivate(surface_id)
    }
    pub fn GetLastActiveReference(&self) -> Option<SurfaceId> {
        self.last_active_reference_
    }
    pub fn GetLastReference(&self) -> Option<SurfaceId> {
        self.last_reference_
    }
    pub fn FindLatestActiveSurfaceInRange(&self, range: &SurfaceRange) -> Option<SurfaceId> {
        if range.end().local_surface_id().embed_token() == self.embed_token_ {
            assert_eq!(self.submitter_, range.end().frame_sink_id());
            let candidate = self.FindLatestActiveSurfaceUpTo(range.end())?;
            return range
                .start()
                .is_none_or(|start| !start.IsNewerThan(candidate))
                .then_some(candidate);
        }
        let start = range.start()?;
        assert_eq!(self.embed_token_, start.local_surface_id().embed_token());
        assert_ne!(
            self.embed_token_,
            range.end().local_surface_id().embed_token()
        );
        assert_eq!(self.submitter_, start.frame_sink_id());
        let candidate = self
            .surfaces_
            .iter()
            .rev()
            .copied()
            .find(|id| self.active_surfaces_.contains(id))?;
        (!start.IsNewerThan(candidate)).then_some(candidate)
    }

    // Source: components/viz/service/surfaces/surface_allocation_group.h:106-124
    // Source: components/viz/service/surfaces/surface_allocation_group.cc:151-198
    // Latency payload movement stays on FrameData. Notification order remains
    // here and is emitted as effects for the manager to apply.
    pub fn MarkActive(&mut self, surface: SurfaceId) {
        if self.surfaces_.contains(&surface) {
            self.active_surfaces_.insert(surface);
        }
    }
    pub fn OnFirstSurfaceActivation(&mut self, surface: SurfaceId) -> Vec<AllocationGroupEffect> {
        self.MarkActive(surface);
        let mut effects: Vec<_> = self
            .active_embedders_
            .iter()
            .copied()
            .map(|embedder| AllocationGroupEffect::NotifyChildActivated {
                embedder,
                child: surface,
            })
            .collect();
        let resolved: Vec<_> = self
            .blocked_embedders_
            .iter()
            .filter_map(|(embedder, dependency)| {
                (!dependency.IsNewerThan(surface)).then_some((*embedder, *dependency))
            })
            .collect();
        for (embedder, dependency) in resolved {
            self.blocked_embedders_.remove(&embedder);
            effects.push(AllocationGroupEffect::ResolveActivationDependency {
                embedder,
                dependency,
            });
        }
        effects
    }
    pub fn WillNotRegisterNewSurfaces(&mut self) -> Vec<AllocationGroupEffect> {
        self.will_not_register_new_surfaces_ = true;
        std::mem::take(&mut self.blocked_embedders_)
            .into_iter()
            .map(
                |(embedder, dependency)| AllocationGroupEffect::ResolveActivationDependency {
                    embedder,
                    dependency,
                },
            )
            .collect()
    }
    pub fn AckLatestActiveUnackedFrame(
        &self,
        has_unacked_frame: impl FnOnce(SurfaceId) -> bool,
    ) -> Option<AllocationGroupEffect> {
        let reference = self.last_active_reference_?;
        let latest =
            self.FindLatestActiveSurfaceInRange(&SurfaceRange::new(Some(reference), reference))?;
        has_unacked_frame(latest).then_some(AllocationGroupEffect::SendAck(latest))
    }

    // Source: components/viz/service/surfaces/surface_allocation_group.h:125-153
    // Source: components/viz/service/surfaces/surface_allocation_group.cc:199-270
    pub fn last_created_surface(&self) -> Option<SurfaceId> {
        self.surfaces_.last().copied()
    }
    pub fn surfaces(&self) -> &[SurfaceId] {
        &self.surfaces_
    }
    pub fn FindLatestSurfaceUpTo(&self, surface_id: SurfaceId) -> Option<SurfaceId> {
        assert_eq!(self.submitter_, surface_id.frame_sink_id());
        assert_eq!(
            self.embed_token_,
            surface_id.local_surface_id().embed_token()
        );
        let first = *self.surfaces_.first()?;
        if !surface_id.IsSameOrNewerThan(first) {
            return None;
        }
        let index = self
            .surfaces_
            .partition_point(|candidate| surface_id.IsSameOrNewerThan(*candidate));
        self.surfaces_.get(index.saturating_sub(1)).copied()
    }
    pub fn FindLatestActiveSurfaceUpTo(&self, surface_id: SurfaceId) -> Option<SurfaceId> {
        let latest = self.FindLatestSurfaceUpTo(surface_id)?;
        let index = self
            .surfaces_
            .iter()
            .position(|candidate| *candidate == latest)?;
        self.surfaces_[..=index]
            .iter()
            .rev()
            .copied()
            .find(|candidate| self.active_surfaces_.contains(candidate))
    }
    pub fn MaybeMarkForDestruction(&self) -> Option<AllocationGroupEffect> {
        self.IsReadyToDestroy()
            .then_some(AllocationGroupEffect::MarkForDestruction)
    }
    pub fn UpdateLastReferenceAndMaybeActivate(
        &mut self,
        surface_id: SurfaceId,
    ) -> Vec<AllocationGroupEffect> {
        if self
            .last_reference_
            .is_some_and(|last| last.IsSameOrNewerThan(surface_id))
        {
            return Vec::new();
        }
        self.last_reference_ = Some(surface_id);
        let Some(candidate) = self.FindLatestSurfaceUpTo(surface_id) else {
            return Vec::new();
        };
        (candidate != surface_id)
            .then_some(AllocationGroupEffect::SetFallback(candidate))
            .into_iter()
            .collect()
    }
}

// Source: components/viz/service/surfaces/surface_allocation_group.h:154-191
// Source: components/viz/service/surfaces/surface_allocation_group.cc:271-272
// Rust drop checks are enforced by SurfaceManager removing a group only after
// IsReadyToDestroy. The fields above preserve Chromium's state partition.
