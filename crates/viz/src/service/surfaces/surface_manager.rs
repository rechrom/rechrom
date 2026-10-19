// Source: components/viz/service/surfaces/surface_manager.h:1-54
// Source: components/viz/service/surfaces/surface_manager.cc:1-53
// Observer delivery, UMA, trace formatting and wall-clock injection are host
// services. This owner keeps Chromium's surface graph, allocation groups,
// temporary roots, activation propagation and garbage collection algorithms.
#![allow(non_snake_case)]

use super::{
    surface::Surface,
    surface_allocation_group::{AllocationGroupEffect, SurfaceAllocationGroup},
    surface_reference::SurfaceReference,
};
use crate::common::surfaces::{
    frame_sink_id::FrameSinkId,
    local_surface_id::{EmbedToken, LocalSurfaceId},
    surface_id::SurfaceId,
    surface_info::SurfaceInfo,
    surface_range::SurfaceRange,
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::Instant,
};

// Source: components/viz/service/surfaces/surface_manager.h:247-263
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemovedReason {
    EMBEDDED,
    DROPPED,
    SKIPPED,
    EXPIRED,
    COPIED,
}

#[derive(Clone, Copy, Debug, Default)]
struct TemporaryReferenceData {
    marked_as_old: bool,
}

// Source: components/viz/service/surfaces/surface_manager.h:314-392
pub struct SurfaceManager {
    activation_deadline_in_frames_: Option<u32>,
    embed_token_to_allocation_group_: BTreeMap<EmbedToken, SurfaceAllocationGroup>,
    frame_sink_id_to_allocation_groups_: BTreeMap<FrameSinkId, BTreeSet<EmbedToken>>,
    surfaces_: BTreeMap<SurfaceId, Surface>,
    surfaces_to_destroy_: BTreeMap<SurfaceId, Instant>,
    root_surface_id_: SurfaceId,
    display_root_surface_id_: Option<SurfaceId>,
    references_: BTreeMap<SurfaceId, BTreeSet<SurfaceId>>,
    reverse_references_: BTreeMap<SurfaceId, BTreeSet<SurfaceId>>,
    temporary_references_: BTreeMap<SurfaceId, TemporaryReferenceData>,
    surface_referenced_timestamps_: BTreeMap<SurfaceId, (Instant, u32)>,
    temporary_reference_ranges_: BTreeMap<FrameSinkId, Vec<LocalSurfaceId>>,
    allocation_groups_need_garbage_collection_: bool,
}

impl Default for SurfaceManager {
    fn default() -> Self {
        Self {
            activation_deadline_in_frames_: None,
            embed_token_to_allocation_group_: BTreeMap::new(),
            frame_sink_id_to_allocation_groups_: BTreeMap::new(),
            surfaces_: BTreeMap::new(),
            surfaces_to_destroy_: BTreeMap::new(),
            root_surface_id_: SurfaceId::new(
                FrameSinkId::default(),
                LocalSurfaceId::new(1, 1, EmbedToken::new(u64::MAX, u64::MAX)),
            ),
            display_root_surface_id_: None,
            references_: BTreeMap::new(),
            reverse_references_: BTreeMap::new(),
            temporary_references_: BTreeMap::new(),
            surface_referenced_timestamps_: BTreeMap::new(),
            temporary_reference_ranges_: BTreeMap::new(),
            allocation_groups_need_garbage_collection_: false,
        }
    }
}

impl SurfaceManager {
    // Source: components/viz/service/surfaces/surface_manager.h:55-87
    // Source: components/viz/service/surfaces/surface_manager.cc:54-120
    pub fn new() -> Self {
        Self::default()
    }
    pub fn SetActivationDeadlineInFramesForTesting(&mut self, deadline: Option<u32>) {
        self.activation_deadline_in_frames_ = deadline;
    }
    pub fn activation_deadline_in_frames(&self) -> Option<u32> {
        self.activation_deadline_in_frames_
    }
    pub fn SurfaceReferencesToString(&self) -> String {
        fn walk(manager: &SurfaceManager, id: SurfaceId, indent: &str, out: &mut String) {
            out.push_str(indent);
            out.push_str(&id.ToString());
            if manager.IsMarkedForDestruction(id) {
                out.push_str(" destroyed");
            }
            out.push('\n');
            if let Some(children) = manager.references_.get(&id) {
                let child_indent = format!("{indent}  ");
                for child in children {
                    walk(manager, *child, &child_indent, out);
                }
            }
        }
        let mut out = String::new();
        walk(self, self.root_surface_id_, "", &mut out);
        for id in self.temporary_references_.keys() {
            walk(self, *id, "* ", &mut out);
        }
        out
    }

    // Source: components/viz/service/surfaces/surface_manager.h:88-103
    // Source: components/viz/service/surfaces/surface_manager.cc:121-179
    pub fn CreateSurface(
        &mut self,
        info: SurfaceInfo,
        max_uncommitted: usize,
    ) -> Result<&mut Surface, String> {
        if !info.is_valid() {
            return Err("invalid SurfaceInfo".into());
        }
        let id = info.id();
        if self.surfaces_.contains_key(&id) {
            return Err("surface already exists".into());
        }
        let group = self.GetOrCreateAllocationGroupForSurfaceId(id)?;
        group.RegisterSurface(id);
        self.surfaces_
            .insert(id, Surface::new(info, max_uncommitted));
        self.AddTemporaryReference(id);
        Ok(self.surfaces_.get_mut(&id).expect("inserted surface"))
    }
    pub fn MarkSurfaceForDestruction(&mut self, id: SurfaceId) {
        assert!(self.surfaces_.contains_key(&id));
        self.surfaces_to_destroy_
            .entry(id)
            .or_insert_with(Instant::now);
    }
    pub fn InvalidateFrameSinkId(&mut self, id: FrameSinkId) {
        let tokens: Vec<_> = self
            .frame_sink_id_to_allocation_groups_
            .get(&id)
            .into_iter()
            .flatten()
            .copied()
            .collect();
        let mut effects = Vec::new();
        for token in tokens {
            if let Some(group) = self.embed_token_to_allocation_group_.get_mut(&token) {
                effects.extend(group.WillNotRegisterNewSurfaces());
            }
        }
        self.ApplyAllocationGroupEffects(effects);
        self.GarbageCollectSurfaces();
    }
    pub fn GetSurfaceForId(&self, id: SurfaceId) -> Option<&Surface> {
        self.surfaces_.get(&id)
    }
    pub fn GetSurfaceForIdMutable(&mut self, id: SurfaceId) -> Option<&mut Surface> {
        self.surfaces_.get_mut(&id)
    }

    // Source: components/viz/service/surfaces/surface_manager.h:104-120
    // Observer registration and damage notification are embedding callbacks;
    // activation below returns the same state transition directly to VizEngine.
    // Source: components/viz/service/surfaces/surface_manager.h:121-149
    // Source: components/viz/service/surfaces/surface_manager.cc:496-588
    pub fn SurfaceActivated(&mut self, id: SurfaceId) {
        let first = self
            .surfaces_
            .get_mut(&id)
            .is_some_and(Surface::TakeFirstActivation);
        let token = id.local_surface_id().embed_token();
        let effects = if let Some(group) = self.embed_token_to_allocation_group_.get_mut(&token) {
            if first {
                group.OnFirstSurfaceActivation(id)
            } else {
                group.MarkActive(id);
                Vec::new()
            }
        } else {
            Vec::new()
        };
        self.RecomputeActiveReferencedSurfaces(id);
        self.ApplyAllocationGroupEffects(effects);
    }
    pub fn SurfaceWillBeDrawn(&mut self, id: SurfaceId) {
        if let Some(surface) = self.surfaces_.get_mut(&id) {
            surface.OnWillBeDrawn();
        }
    }
    pub fn GetRootSurfaceId(&self) -> Option<SurfaceId> {
        self.display_root_surface_id_
    }
    pub fn SetRootSurfaceId(&mut self, id: SurfaceId) {
        if self.display_root_surface_id_ == Some(id) {
            return;
        }
        if let Some(old) = self.display_root_surface_id_.replace(id) {
            self.RemoveSurfaceReferenceImpl(SurfaceReference::new(self.root_surface_id_, old));
        }
        self.AddSurfaceReferenceImpl(SurfaceReference::new(self.root_surface_id_, id));
    }
    pub fn GetCreatedSurfaceIds(&self) -> Vec<SurfaceId> {
        self.surfaces_.keys().copied().collect()
    }

    // Source: components/viz/service/surfaces/surface_manager.h:150-182
    // Source: components/viz/service/surfaces/surface_manager.cc:180-317
    pub fn AddSurfaceReferences(&mut self, references: impl IntoIterator<Item = SurfaceReference>) {
        for reference in references {
            self.AddSurfaceReferenceImpl(reference);
        }
    }
    pub fn RemoveSurfaceReferences(
        &mut self,
        references: impl IntoIterator<Item = SurfaceReference>,
    ) {
        for reference in references {
            self.RemoveSurfaceReferenceImpl(reference);
        }
        self.GarbageCollectSurfaces();
    }
    pub fn GarbageCollectSurfaces(&mut self) -> Vec<Surface> {
        let live = self.GetLiveSurfaces();
        let doomed: Vec<_> = self
            .surfaces_to_destroy_
            .keys()
            .copied()
            .filter(|id| !live.contains(id))
            .collect();
        let mut removed = Vec::new();
        for id in doomed {
            self.surfaces_to_destroy_.remove(&id);
            if let Some(surface) = self.DestroySurface(id) {
                removed.push(surface);
            }
            self.surfaces_to_destroy_.remove(&id);
        }
        self.MaybeGarbageCollectAllocationGroups();
        removed
    }
    pub fn GetSurfacesReferencedByParent(&self, id: SurfaceId) -> Option<&BTreeSet<SurfaceId>> {
        self.references_.get(&id)
    }
    pub fn GetSurfacesThatReferenceChildForTesting(&self, id: SurfaceId) -> BTreeSet<SurfaceId> {
        self.reverse_references_
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }
    pub fn GetSurfaceReferencedTimestamp(&self, id: SurfaceId) -> Option<Instant> {
        self.surface_referenced_timestamps_
            .get(&id)
            .map(|value| value.0)
    }
    pub fn GetLatestInFlightSurface(&self, range: &SurfaceRange) -> Option<&Surface> {
        let end_token = range.end().local_surface_id().embed_token();
        if let Some(id) = self
            .embed_token_to_allocation_group_
            .get(&end_token)
            .and_then(|group| group.FindLatestActiveSurfaceInRange(range))
        {
            return self.surfaces_.get(&id);
        }
        let start = range.start()?;
        if start.local_surface_id().embed_token() == end_token {
            return None;
        }
        let id = self
            .embed_token_to_allocation_group_
            .get(&start.local_surface_id().embed_token())?
            .FindLatestActiveSurfaceInRange(range)?;
        self.surfaces_.get(&id)
    }

    // Source: components/viz/service/surfaces/surface_manager.h:183-246
    // Source: components/viz/service/surfaces/surface_manager.cc:440-460
    // Source: components/viz/service/surfaces/surface_manager.cc:642-757
    pub fn DropTemporaryReference(&mut self, id: SurfaceId) {
        self.RemoveTemporaryReferenceImpl(id, RemovedReason::DROPPED);
    }
    pub fn GetOrCreateAllocationGroupForSurfaceId(
        &mut self,
        id: SurfaceId,
    ) -> Result<&mut SurfaceAllocationGroup, String> {
        let token = id.local_surface_id().embed_token();
        if self
            .embed_token_to_allocation_group_
            .get(&token)
            .is_some_and(|group| group.submitter_frame_sink_id() != id.frame_sink_id())
        {
            return Err("Cannot reuse embed token across frame sinks".into());
        }
        if !self.embed_token_to_allocation_group_.contains_key(&token) {
            self.embed_token_to_allocation_group_.insert(
                token,
                SurfaceAllocationGroup::new(id.frame_sink_id(), token),
            );
            self.frame_sink_id_to_allocation_groups_
                .entry(id.frame_sink_id())
                .or_default()
                .insert(token);
        }
        Ok(self
            .embed_token_to_allocation_group_
            .get_mut(&token)
            .expect("inserted group"))
    }
    pub fn GetAllocationGroupForSurfaceId(&self, id: SurfaceId) -> Option<&SurfaceAllocationGroup> {
        self.embed_token_to_allocation_group_
            .get(&id.local_surface_id().embed_token())
            .filter(|group| group.submitter_frame_sink_id() == id.frame_sink_id())
    }
    pub fn SetAllocationGroupsNeedGarbageCollection(&mut self) {
        self.allocation_groups_need_garbage_collection_ = true;
    }
    pub fn HasBlockedEmbedder(&self, id: FrameSinkId) -> bool {
        self.frame_sink_id_to_allocation_groups_
            .get(&id)
            .is_some_and(|tokens| {
                tokens.iter().any(|token| {
                    self.embed_token_to_allocation_group_
                        .get(token)
                        .is_some_and(SurfaceAllocationGroup::HasBlockedEmbedder)
                })
            })
    }
    pub fn CommitFramesInRangeRecursively(
        &mut self,
        range: SurfaceRange,
        mut predicate: impl FnMut(SurfaceId, u32) -> bool,
    ) {
        let mut queue = VecDeque::from([range]);
        let mut seen = BTreeSet::new();
        while let Some(range) = queue.pop_front() {
            let mut ids = Vec::new();
            if let Some(start) = range.start() {
                if start.local_surface_id().embed_token()
                    != range.end().local_surface_id().embed_token()
                {
                    if let Some(group) = self.GetAllocationGroupForSurfaceId(start) {
                        ids.extend(
                            group
                                .surfaces()
                                .iter()
                                .copied()
                                .filter(|id| range.IsInRangeInclusive(*id)),
                        );
                    }
                }
            }
            if let Some(group) = self.GetAllocationGroupForSurfaceId(range.end()) {
                ids.extend(
                    group
                        .surfaces()
                        .iter()
                        .copied()
                        .filter(|id| range.IsInRangeInclusive(*id)),
                );
            }
            for id in ids {
                if !seen.insert(id) {
                    continue;
                }
                let id_for_predicate = id;
                if let Some(surface) = self.surfaces_.get_mut(&id) {
                    surface.CommitFrames(|index| predicate(id_for_predicate, index));
                    if let Some(frame) = surface.pending_frame().or_else(|| surface.active_frame())
                    {
                        queue.extend(frame.referenced_surfaces.iter().cloned());
                    }
                }
            }
        }
    }

    // Source: components/viz/service/surfaces/surface_manager.h:247-313
    // Source: components/viz/service/surfaces/surface_manager.cc:318-439
    fn GetLiveSurfaces(&self) -> BTreeSet<SurfaceId> {
        let mut live = BTreeSet::new();
        let mut queue = VecDeque::from([self.root_surface_id_]);
        for id in self.surfaces_.keys().copied() {
            if !self.IsMarkedForDestruction(id) && live.insert(id) {
                queue.push_back(id);
            }
        }
        for id in self.temporary_references_.keys().copied() {
            if live.insert(id) {
                queue.push_back(id);
            }
        }
        while let Some(parent) = queue.pop_front() {
            if let Some(children) = self.references_.get(&parent) {
                for child in children {
                    if live.insert(*child) {
                        queue.push_back(*child);
                    }
                }
            }
        }
        live
    }
    fn AddSurfaceReferenceImpl(&mut self, reference: SurfaceReference) {
        let parent = reference.parent_id();
        let child = reference.child_id();
        if parent.frame_sink_id() == child.frame_sink_id() && !parent.IsNewerThan(child) {
            return;
        }
        if !self.surfaces_.contains_key(&child) {
            return;
        }
        if self.references_.entry(parent).or_default().insert(child) {
            self.reverse_references_
                .entry(child)
                .or_default()
                .insert(parent);
            let entry = self
                .surface_referenced_timestamps_
                .entry(child)
                .or_insert_with(|| (Instant::now(), 0));
            entry.1 += 1;
        }
        if self.HasTemporaryReference(child) {
            self.RemoveTemporaryReferenceImpl(child, RemovedReason::EMBEDDED);
        }
    }
    fn RemoveSurfaceReferenceImpl(&mut self, reference: SurfaceReference) {
        let parent = reference.parent_id();
        let child = reference.child_id();
        let removed = self
            .references_
            .get_mut(&parent)
            .is_some_and(|children| children.remove(&child));
        if !removed {
            return;
        }
        if self
            .references_
            .get(&parent)
            .is_some_and(BTreeSet::is_empty)
        {
            self.references_.remove(&parent);
        }
        if let Some(parents) = self.reverse_references_.get_mut(&child) {
            parents.remove(&parent);
            if parents.is_empty() {
                self.reverse_references_.remove(&child);
            }
        }
        if let Some(value) = self.surface_referenced_timestamps_.get_mut(&child) {
            value.1 -= 1;
            if value.1 == 0 {
                self.surface_referenced_timestamps_.remove(&child);
            }
        }
    }
    fn HasTemporaryReference(&self, id: SurfaceId) -> bool {
        self.temporary_references_.contains_key(&id)
    }
    fn AddTemporaryReference(&mut self, id: SurfaceId) {
        assert!(!self.HasTemporaryReference(id));
        self.temporary_references_
            .insert(id, TemporaryReferenceData::default());
        self.temporary_reference_ranges_
            .entry(id.frame_sink_id())
            .or_default()
            .push(id.local_surface_id());
    }
    fn RemoveTemporaryReferenceImpl(&mut self, id: SurfaceId, _reason: RemovedReason) {
        let sink = id.frame_sink_id();
        if let Some(range) = self.temporary_reference_ranges_.get_mut(&sink) {
            range.retain(|local| {
                let candidate = SurfaceId::new(sink, *local);
                let remove = candidate.HasSameEmbedTokenAs(id) && !candidate.IsNewerThan(id);
                if remove {
                    self.temporary_references_.remove(&candidate);
                }
                !remove
            });
            if range.is_empty() {
                self.temporary_reference_ranges_.remove(&sink);
            }
        }
    }
    pub fn ExpireOldTemporaryReferences(&mut self) {
        // Source: components/viz/service/surfaces/surface_manager.cc:461-495
        let mut expired = Vec::new();
        for (id, data) in &mut self.temporary_references_ {
            if data.marked_as_old {
                expired.push(*id);
            } else if self.surfaces_to_destroy_.contains_key(id) {
                data.marked_as_old = true;
            }
        }
        for id in expired {
            self.RemoveTemporaryReferenceImpl(id, RemovedReason::EXPIRED);
        }
        self.GarbageCollectSurfaces();
    }
    pub fn IsMarkedForDestruction(&self, id: SurfaceId) -> bool {
        self.surfaces_to_destroy_.contains_key(&id)
    }

    // Source: components/viz/service/surfaces/surface_manager.cc:589-641
    pub fn DestroySurface(&mut self, id: SurfaceId) -> Option<Surface> {
        self.temporary_references_.remove(&id);
        self.surfaces_to_destroy_.remove(&id);
        if self.display_root_surface_id_ == Some(id) {
            self.display_root_surface_id_ = None;
            self.RemoveSurfaceReferenceImpl(SurfaceReference::new(self.root_surface_id_, id));
        }
        if let Some(children) = self.references_.remove(&id) {
            for child in children {
                if let Some(parents) = self.reverse_references_.get_mut(&child) {
                    parents.remove(&id);
                }
            }
        }
        if let Some(parents) = self.reverse_references_.remove(&id) {
            for parent in parents {
                if let Some(children) = self.references_.get_mut(&parent) {
                    children.remove(&id);
                }
            }
        }
        let token = id.local_surface_id().embed_token();
        if let Some(group) = self.embed_token_to_allocation_group_.get_mut(&token) {
            if group.UnregisterSurface(id).is_some() {
                self.allocation_groups_need_garbage_collection_ = true;
            }
        }
        self.surfaces_.remove(&id)
    }

    // Source: components/viz/service/surfaces/surface_manager.cc:651-732
    fn MaybeGarbageCollectAllocationGroups(&mut self) {
        if !self.allocation_groups_need_garbage_collection_ {
            return;
        }
        let doomed: Vec<_> = self
            .embed_token_to_allocation_group_
            .iter()
            .filter_map(|(token, group)| group.IsReadyToDestroy().then_some(*token))
            .collect();
        for token in doomed {
            if let Some(group) = self.embed_token_to_allocation_group_.remove(&token) {
                let sink = group.submitter_frame_sink_id();
                if let Some(tokens) = self.frame_sink_id_to_allocation_groups_.get_mut(&sink) {
                    tokens.remove(&token);
                    if tokens.is_empty() {
                        self.frame_sink_id_to_allocation_groups_.remove(&sink);
                    }
                }
            }
        }
        self.allocation_groups_need_garbage_collection_ = false;
    }

    // Source: components/viz/service/surfaces/surface_manager.cc:759-788
    fn ApplyAllocationGroupEffects(&mut self, effects: Vec<AllocationGroupEffect>) {
        let mut queue: VecDeque<_> = effects.into();
        while let Some(effect) = queue.pop_front() {
            match effect {
                AllocationGroupEffect::ActivateForInheritedDeadline(id) => {
                    if let Some(surface) = self.surfaces_.get_mut(&id) {
                        surface.ActivatePendingFrameForInheritedDeadline();
                    }
                    self.SurfaceActivated(id);
                }
                AllocationGroupEffect::ResolveActivationDependency {
                    embedder,
                    dependency,
                } => {
                    let activated = self
                        .surfaces_
                        .get_mut(&embedder)
                        .is_some_and(|surface| surface.OnActivationDependencyResolved(dependency));
                    if activated {
                        self.SurfaceActivated(embedder);
                    }
                }
                AllocationGroupEffect::NotifyChildActivated { embedder, child } => {
                    if self
                        .surfaces_
                        .get(&embedder)
                        .is_some_and(|surface| surface.OnChildActivatedForActiveFrame(child))
                    {
                        self.RecomputeActiveReferencedSurfaces(embedder);
                    }
                }
                AllocationGroupEffect::SetFallback(id) => {
                    let activated = self.surfaces_.get_mut(&id).is_some_and(|surface| {
                        let pending = surface.HasPendingFrame();
                        surface.SetIsFallbackAndMaybeActivate();
                        pending
                    });
                    if activated {
                        self.SurfaceActivated(id);
                    }
                }
                AllocationGroupEffect::SendAck(id) => {
                    if let Some(surface) = self.surfaces_.get_mut(&id) {
                        surface.SendAckToClient();
                    }
                }
                AllocationGroupEffect::MarkForDestruction => {
                    self.allocation_groups_need_garbage_collection_ = true;
                }
            }
        }
    }

    fn RecomputeActiveReferencedSurfaces(&mut self, parent: SurfaceId) {
        let ranges = self
            .surfaces_
            .get(&parent)
            .and_then(Surface::active_frame)
            .map(|frame| frame.referenced_surfaces.clone())
            .unwrap_or_default();
        let old_refs = self.references_.get(&parent).cloned().unwrap_or_default();
        let old_groups = self
            .surfaces_
            .get(&parent)
            .map(|surface| surface.referenced_allocation_groups().clone())
            .unwrap_or_default();
        let mut new_refs = BTreeSet::new();
        let mut new_groups = BTreeSet::new();
        let mut effects = Vec::new();
        for range in ranges {
            if let Some(surface) = self.GetLatestInFlightSurface(&range) {
                new_refs.insert(surface.surface_id());
            }
            let end = range.end();
            if let Ok(group) = self.GetOrCreateAllocationGroupForSurfaceId(end) {
                effects.extend(group.UpdateLastActiveReferenceAndMaybeActivate(end));
                new_groups.insert(end);
            }
            if range.HasDifferentEmbedTokens() {
                if let Some(start) = range.start() {
                    let selected_from_start =
                        new_refs.iter().any(|id| id.HasSameEmbedTokenAs(start));
                    if new_refs.is_empty() || selected_from_start {
                        if let Ok(group) = self.GetOrCreateAllocationGroupForSurfaceId(start) {
                            effects.extend(group.UpdateLastActiveReferenceAndMaybeActivate(start));
                            new_groups.insert(start);
                        }
                    }
                }
            }
        }
        for child in new_refs.difference(&old_refs).copied().collect::<Vec<_>>() {
            self.AddSurfaceReferenceImpl(SurfaceReference::new(parent, child));
        }
        for child in old_refs.difference(&new_refs).copied().collect::<Vec<_>>() {
            self.RemoveSurfaceReferenceImpl(SurfaceReference::new(parent, child));
        }
        for group_id in old_groups
            .difference(&new_groups)
            .copied()
            .collect::<Vec<_>>()
        {
            if let Some(group) = self
                .embed_token_to_allocation_group_
                .get_mut(&group_id.local_surface_id().embed_token())
            {
                if let Some(effect) = group.UnregisterActiveEmbedder(parent) {
                    queue_effect(&mut effects, effect);
                }
            }
        }
        for group_id in new_groups
            .difference(&old_groups)
            .copied()
            .collect::<Vec<_>>()
        {
            if let Some(group) = self
                .embed_token_to_allocation_group_
                .get_mut(&group_id.local_surface_id().embed_token())
            {
                group.RegisterActiveEmbedder(parent);
            }
        }
        if let Some(surface) = self.surfaces_.get_mut(&parent) {
            surface.SetResolvedActiveReferences(new_refs, new_groups);
        }
        self.ApplyAllocationGroupEffects(effects);
    }
}

fn queue_effect(effects: &mut Vec<AllocationGroupEffect>, effect: AllocationGroupEffect) {
    effects.push(effect);
}

// Source: components/viz/service/surfaces/surface_manager.h:393-412
// Source: components/viz/service/surfaces/surface_manager.cc:733-758
// Scoped observer traits and delegate forwarding stay in the embedding. The
// core exposes typed state changes directly to VizEngine.
