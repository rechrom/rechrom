// Source: components/viz/service/surfaces/surface.h:1-96
// Source: components/viz/service/surfaces/surface.cc:1-90
// Platform callbacks, Perfetto, copy-output payloads and transition services
// are adapter capabilities. This owner retains Chromium's uncommitted,
// pending and active frame lifecycle and emits resources/effects explicitly.
#![allow(non_snake_case)]

use crate::common::{
    quads::frame_deadline::FrameDeadline,
    surfaces::{surface_id::SurfaceId, surface_info::SurfaceInfo, surface_range::SurfaceRange},
};
use crate::service::surfaces::surface_dependency_deadline::SurfaceDependencyDeadline;
use compositor_core::adapter::geometry::Rect;
use compositor_core::runtime::frame::RenderPassId;
use compositor_core::runtime::resources::RasterReturnedResource;
use compositor_core::{NativeCompositorFrame, NativeTransferableResource};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::Instant;

// Source: components/viz/service/surfaces/surface.h:97-137
// Source: components/viz/service/surfaces/surface.cc:60-116
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueFrameResult {
    REJECTED,
    ACCEPTED_ACTIVE,
    ACCEPTED_PENDING,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingFrameDataChangeReason {
    kNotSet,
    kCommitFrameReset,
    kCommitFrameDependencies,
    kActivatePendingFrameReset,
    kSurfaceDestruction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameActivationReason {
    kNotSet,
    kDependencyResolved,
    kDeadline,
    kCommitWithNoDependencies,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationHelper {
    pub frame_token: u32,
    completed: bool,
}
impl PresentationHelper {
    pub fn new(frame_token: u32) -> Self {
        Self {
            frame_token,
            completed: false,
        }
    }
    pub fn DidPresent(&mut self) {
        self.completed = true;
    }
    pub fn completed(&self) -> bool {
        self.completed
    }
}

// Source: components/viz/service/surfaces/surface.h:340-359
// Source: components/viz/service/surfaces/surface.cc:476-492
pub struct FrameData {
    pub frame: NativeCompositorFrame,
    pub frame_index: u32,
    pub submission: u64,
    pub producer_token: u32,
    pub damage_start_token: Option<u32>,
    pub damage_continuous: bool,
    pub pending_damage: BTreeMap<RenderPassId, Rect>,
    pub activation_dependencies: BTreeSet<SurfaceId>,
    pub referenced_surfaces: Vec<SurfaceRange>,
    pub acknowledged: bool,
    pub frame_drawn: bool,
    pub will_be_notified_of_presentation: bool,
}
impl FrameData {
    pub fn SendAckIfNeeded(&mut self) -> bool {
        if self.acknowledged {
            false
        } else {
            self.acknowledged = true;
            true
        }
    }
}

// Source: components/viz/service/surfaces/surface.h:402-467
pub struct Surface {
    surface_info_: SurfaceInfo,
    previous_frame_surface_id_: Option<SurfaceId>,
    deadline_: SurfaceDependencyDeadline,
    pending_frame_data_: Option<FrameData>,
    active_frame_data_: Option<FrameData>,
    uncommitted_frames_: VecDeque<FrameData>,
    seen_first_frame_activation_: bool,
    first_activation_notified_: bool,
    seen_first_surface_embedding_: bool,
    active_referenced_surfaces_: BTreeSet<SurfaceId>,
    referenced_allocation_groups_: BTreeSet<SurfaceId>,
    activation_dependencies_: BTreeSet<SurfaceId>,
    blocking_allocation_groups_: BTreeSet<SurfaceId>,
    is_fallback_: bool,
    is_latency_info_taken_: bool,
    pending_copy_surface_id_: Option<SurfaceId>,
    max_uncommitted_frames_: usize,
    last_sent_frame_token_: u32,
    pending_frame_data_change_reason_: PendingFrameDataChangeReason,
    frame_activation_reason_: FrameActivationReason,
    retired_resources_: Vec<RasterReturnedResource>,
    closed_: bool,
}

impl Surface {
    // Source: components/viz/service/surfaces/surface.h:138-182
    // Source: components/viz/service/surfaces/surface.cc:117-202
    pub fn new(info: SurfaceInfo, max_uncommitted_frames: usize) -> Self {
        Self {
            surface_info_: info,
            previous_frame_surface_id_: None,
            deadline_: Default::default(),
            pending_frame_data_: None,
            active_frame_data_: None,
            uncommitted_frames_: VecDeque::new(),
            seen_first_frame_activation_: false,
            first_activation_notified_: false,
            seen_first_surface_embedding_: false,
            active_referenced_surfaces_: BTreeSet::new(),
            referenced_allocation_groups_: BTreeSet::new(),
            activation_dependencies_: BTreeSet::new(),
            blocking_allocation_groups_: BTreeSet::new(),
            is_fallback_: false,
            is_latency_info_taken_: false,
            pending_copy_surface_id_: None,
            max_uncommitted_frames_: max_uncommitted_frames,
            last_sent_frame_token_: 0,
            pending_frame_data_change_reason_: PendingFrameDataChangeReason::kNotSet,
            frame_activation_reason_: FrameActivationReason::kNotSet,
            retired_resources_: Vec::new(),
            closed_: false,
        }
    }
    pub fn surface_id(&self) -> SurfaceId {
        self.surface_info_.id()
    }
    pub fn surface_info(&self) -> &SurfaceInfo {
        &self.surface_info_
    }
    pub fn previous_frame_surface_id(&self) -> Option<SurfaceId> {
        self.previous_frame_surface_id_
    }
    pub fn SetDependencyDeadline(&mut self, deadline: FrameDeadline, default_frames: Option<u32>) {
        self.deadline_.Set(deadline, default_frames);
    }
    pub fn has_deadline(&self) -> bool {
        self.deadline_.has_deadline()
    }
    pub fn SetPreviousFrameSurface(&mut self, surface: SurfaceId) {
        assert!(self.HasActiveFrame() || self.HasPendingFrame());
        self.previous_frame_surface_id_ = Some(surface);
    }

    // Source: components/viz/service/surfaces/surface.h:183-192
    // Source: components/viz/service/surfaces/surface.cc:252-383
    pub fn QueueFrame(&mut self, frame: FrameData) -> QueueFrameResult {
        if self.closed_ || !self.FrameMatchesSurface(&frame.frame) {
            return QueueFrameResult::REJECTED;
        }
        if self.max_uncommitted_frames_ == 0 {
            return self.CommitFrame(frame);
        }
        if self.uncommitted_frames_.len() == self.max_uncommitted_frames_ {
            if let Some(dropped) = self.uncommitted_frames_.pop_front() {
                self.RetireFrame(dropped);
            }
        }
        self.uncommitted_frames_.push_back(frame);
        if self.uncommitted_frames_.len() < self.max_uncommitted_frames_ {
            if let Some(back) = self.uncommitted_frames_.back_mut() {
                back.SendAckIfNeeded();
            }
        }
        QueueFrameResult::ACCEPTED_PENDING
    }
    fn FrameMatchesSurface(&self, frame: &NativeCompositorFrame) -> bool {
        let Some(root) = frame.render_pass_list.last() else {
            return false;
        };
        root.output_rect.width == self.surface_info_.size_in_pixels().width()
            && root.output_rect.height == self.surface_info_.size_in_pixels().height()
            && frame.metadata.device_scale_factor == self.surface_info_.device_scale_factor()
    }
    pub fn CommitFrame(&mut self, mut frame: FrameData) -> QueueFrameResult {
        self.is_latency_info_taken_ = false;
        if self.active_frame_data_.is_some() || self.pending_frame_data_.is_some() {
            self.previous_frame_surface_id_ = Some(self.surface_id());
        }
        self.pending_frame_data_change_reason_ = PendingFrameDataChangeReason::kCommitFrameReset;
        if let Some(previous_pending) = self.pending_frame_data_.take() {
            self.RetireFrame(previous_pending);
        }
        self.UpdateActivationDependencies(&frame);
        frame.activation_dependencies = self.activation_dependencies_.clone();
        if self.activation_dependencies_.is_empty() || self.is_fallback_ {
            self.ActivateFrame(frame, FrameActivationReason::kCommitWithNoDependencies);
            QueueFrameResult::ACCEPTED_ACTIVE
        } else {
            self.pending_frame_data_change_reason_ =
                PendingFrameDataChangeReason::kCommitFrameDependencies;
            self.pending_frame_data_ = Some(frame);
            QueueFrameResult::ACCEPTED_PENDING
        }
    }
    pub fn CommitFrames(&mut self, mut predicate: impl FnMut(u32) -> bool) {
        assert!(self.max_uncommitted_frames_ > 0);
        while self
            .uncommitted_frames_
            .front()
            .is_some_and(|frame| predicate(frame.frame_index))
        {
            let frame = self.uncommitted_frames_.pop_front().expect("front checked");
            self.CommitFrame(frame);
        }
        if self.uncommitted_frames_.len() < self.max_uncommitted_frames_ {
            if let Some(back) = self.uncommitted_frames_.back_mut() {
                back.SendAckIfNeeded();
            }
        }
    }

    // Source: components/viz/service/surfaces/surface.h:193-227
    // Source: components/viz/service/surfaces/surface.cc:44-58
    // Source: components/viz/service/surfaces/surface.cc:384-438
    // Copy-output request storage/execution is excluded from the CPU Viz core;
    // Renderer owns pixel readback. Frame selection remains on Surface.
    pub fn HasCopyOutputRequests(&self) -> bool {
        false
    }
    pub fn active_frame(&self) -> Option<&FrameData> {
        self.active_frame_data_.as_ref()
    }
    pub fn active_frame_mut(&mut self) -> Option<&mut FrameData> {
        self.active_frame_data_.as_mut()
    }
    pub fn SetActiveFrameForViewTransition(&mut self, frame: NativeCompositorFrame) -> bool {
        let Some(active) = self.active_frame_data_.as_mut() else {
            return false;
        };
        active.frame = frame;
        true
    }

    // Source: components/viz/service/surfaces/surface.h:228-257
    // Source: components/viz/service/surfaces/surface.cc:856-975
    pub fn pending_frame(&self) -> Option<&FrameData> {
        self.pending_frame_data_.as_ref()
    }
    pub fn GetActiveFrameIndex(&self) -> Option<u32> {
        self.active_frame_data_
            .as_ref()
            .map(|frame| frame.frame_index)
    }
    pub fn TakePresentationHelperForPresentNotification(&mut self) -> Option<PresentationHelper> {
        let active = self.active_frame_data_.as_mut()?;
        if active.will_be_notified_of_presentation {
            return None;
        }
        active.will_be_notified_of_presentation = true;
        Some(PresentationHelper::new(active.frame.metadata.frame_token))
    }
    pub fn SendAckToClient(&mut self) -> bool {
        self.active_frame_data_
            .as_mut()
            .is_some_and(FrameData::SendAckIfNeeded)
    }
    pub fn MarkAsDrawn(&mut self) {
        if let Some(active) = self.active_frame_data_.as_mut() {
            active.frame_drawn = true;
        }
    }

    // Source: components/viz/service/surfaces/surface.h:258-310
    // Source: components/viz/service/surfaces/surface.cc:223-251
    // Source: components/viz/service/surfaces/surface.cc:439-475
    pub fn active_referenced_surfaces(&self) -> &BTreeSet<SurfaceId> {
        &self.active_referenced_surfaces_
    }
    pub fn referenced_allocation_groups(&self) -> &BTreeSet<SurfaceId> {
        &self.referenced_allocation_groups_
    }
    pub fn activation_dependencies(&self) -> &BTreeSet<SurfaceId> {
        &self.activation_dependencies_
    }
    pub fn HasActiveFrame(&self) -> bool {
        self.active_frame_data_.is_some()
    }
    pub fn HasPendingFrame(&self) -> bool {
        self.pending_frame_data_.is_some()
    }
    pub fn has_active_frame(&self) -> bool {
        self.HasActiveFrame()
    }
    pub fn has_pending_frame(&self) -> bool {
        self.HasPendingFrame()
    }
    pub fn HasUndrawnActiveFrame(&self) -> bool {
        self.active_frame_data_
            .as_ref()
            .is_some_and(|frame| !frame.frame_drawn)
    }
    pub fn HasUnackedActiveFrame(&self) -> bool {
        self.active_frame_data_
            .as_ref()
            .is_some_and(|frame| !frame.acknowledged)
    }
    pub fn TakeFirstActivation(&mut self) -> bool {
        if !self.seen_first_frame_activation_ || self.first_activation_notified_ {
            false
        } else {
            self.first_activation_notified_ = true;
            true
        }
    }
    pub fn OnChildActivatedForActiveFrame(&self, activated: SurfaceId) -> bool {
        self.active_frame_data_.as_ref().is_some_and(|frame| {
            frame
                .referenced_surfaces
                .iter()
                .any(|range| range.IsInRangeInclusive(activated))
        })
    }
    pub fn ActivatePendingFrameForInheritedDeadline(&mut self) {
        assert!(!self.HasActiveFrame());
        self.ActivatePendingFrameForDeadline();
    }
    pub fn is_latency_info_taken(&self) -> bool {
        self.is_latency_info_taken_
    }
    pub fn OnActivationDependencyResolved(&mut self, dependency: SurfaceId) -> bool {
        assert!(self.activation_dependencies_.remove(&dependency));
        self.blocking_allocation_groups_.remove(&dependency);
        if self.activation_dependencies_.is_empty() {
            self.ActivatePendingFrame(FrameActivationReason::kDependencyResolved);
            true
        } else {
            false
        }
    }
    pub fn SetIsFallbackAndMaybeActivate(&mut self) {
        self.is_fallback_ = true;
        if self.HasPendingFrame() {
            self.ActivatePendingFrameForDeadline();
        }
    }
    pub fn ActivateIfDeadlinePassed(&mut self, now: Instant) -> bool {
        if self.HasPendingFrame() && self.deadline_.HasDeadlinePassedAt(now) {
            self.ActivatePendingFrameForDeadline();
            true
        } else {
            false
        }
    }

    // Source: components/viz/service/surfaces/surface.h:311-339
    // Source: components/viz/service/surfaces/surface.cc:569-598
    pub fn GetFirstUncommitedFrameIndex(&self) -> Option<u32> {
        self.uncommitted_frames_
            .front()
            .map(|frame| frame.frame_index)
    }
    pub fn GetUncommitedFrameIndexNewerThan(&self, frame_index: u32) -> Option<u32> {
        self.uncommitted_frames_
            .iter()
            .find_map(|frame| (frame.frame_index > frame_index).then_some(frame.frame_index))
    }
    pub fn ResetPendingCopySurfaceId(&mut self) {
        assert!(self.pending_copy_surface_id_.take().is_some());
    }
    pub fn ClearNonRootCopyRequests(&mut self) {}

    // Source: components/viz/service/surfaces/surface.h:360-400
    // Source: components/viz/service/surfaces/surface.cc:203-222
    // Source: components/viz/service/surfaces/surface.cc:493-568
    // Source: components/viz/service/surfaces/surface.cc:599-746
    pub fn SetResolvedActiveReferences(
        &mut self,
        references: BTreeSet<SurfaceId>,
        allocation_groups: BTreeSet<SurfaceId>,
    ) {
        self.active_referenced_surfaces_ = references;
        self.referenced_allocation_groups_ = allocation_groups;
        if let Some(copy) = self.pending_copy_surface_id_ {
            self.active_referenced_surfaces_.insert(copy);
        }
    }
    fn ActivatePendingFrame(&mut self, reason: FrameActivationReason) {
        let Some(frame) = self.pending_frame_data_.take() else {
            return;
        };
        self.pending_frame_data_change_reason_ =
            PendingFrameDataChangeReason::kActivatePendingFrameReset;
        self.deadline_.Cancel();
        self.ActivateFrame(frame, reason);
    }
    pub fn ActivatePendingFrameForDeadline(&mut self) {
        if !self.HasPendingFrame() {
            return;
        }
        self.activation_dependencies_.clear();
        self.blocking_allocation_groups_.clear();
        self.ActivatePendingFrame(FrameActivationReason::kDeadline);
    }
    pub fn ActivateFrame(&mut self, frame: FrameData, reason: FrameActivationReason) {
        if let Some(previous) = self.active_frame_data_.take() {
            self.RetireFrame(previous);
        }
        self.activation_dependencies_.clear();
        self.blocking_allocation_groups_.clear();
        self.active_referenced_surfaces_ = frame
            .referenced_surfaces
            .iter()
            .map(SurfaceRange::end)
            .collect();
        self.frame_activation_reason_ = reason;
        self.seen_first_frame_activation_ = true;
        self.last_sent_frame_token_ = self
            .last_sent_frame_token_
            .max(frame.frame.metadata.frame_token);
        self.active_frame_data_ = Some(frame);
    }
    fn UpdateActivationDependencies(&mut self, frame: &FrameData) {
        self.activation_dependencies_ = frame.activation_dependencies.clone();
        self.blocking_allocation_groups_ = self.activation_dependencies_.clone();
    }

    // Source: components/viz/service/surfaces/surface.h:381-400
    // Source: components/viz/service/surfaces/surface.cc:747-855
    pub fn ResolveFrameDeadline(
        &self,
        current: FrameDeadline,
        default_deadline: Option<u32>,
        referenced_by_active_embedder: bool,
    ) -> FrameDeadline {
        if self.is_fallback_ || (!self.HasActiveFrame() && referenced_by_active_embedder) {
            return FrameDeadline::MakeZero();
        }
        let mut frames = current.deadline_in_frames();
        if default_deadline.is_none() || current.use_default_lower_bound_deadline() {
            frames = frames.max(default_deadline.unwrap_or(u32::MAX));
        }
        FrameDeadline::new(
            current
                .frame_start_time()
                .expect("a non-zero FrameDeadline has a start time"),
            frames,
            current.frame_interval(),
            false,
        )
    }

    // Source: components/viz/service/surfaces/surface.h:394-400
    // Source: components/viz/service/surfaces/surface.cc:976-1045
    fn RetireFrame(&mut self, mut frame: FrameData) {
        frame.SendAckIfNeeded();
        self.retired_resources_
            .extend(Self::ReturnedResources(frame));
    }
    pub fn TakeReturnedResources(&mut self) -> Vec<RasterReturnedResource> {
        std::mem::take(&mut self.retired_resources_)
    }
    pub fn ReturnedResources(frame: FrameData) -> Vec<RasterReturnedResource> {
        frame
            .frame
            .resource_list
            .iter()
            .map(NativeTransferableResource::ToReturnedResource)
            .collect()
    }

    // Source: components/viz/service/surfaces/surface.cc:1005-1017
    // Copy request lifetime is represented by Renderer readback ownership; no
    // copy-request objects enter this core package.
    pub fn ClearCopyRequests(&mut self, _keep_root: bool) {}

    // Source: components/viz/service/surfaces/surface.cc:1046-1068
    pub fn OnWillBeDrawn(&mut self) {
        self.seen_first_surface_embedding_ = true;
        self.MarkAsDrawn();
    }

    pub fn TakeActiveFrame(&mut self) -> Option<FrameData> {
        self.active_frame_data_.take()
    }
    pub fn Close(&mut self) {
        self.closed_ = true;
        self.pending_frame_data_change_reason_ = PendingFrameDataChangeReason::kSurfaceDestruction;
        while let Some(frame) = self.uncommitted_frames_.pop_front() {
            self.RetireFrame(frame);
        }
        if let Some(frame) = self.pending_frame_data_.take() {
            self.RetireFrame(frame);
        }
        if let Some(frame) = self.active_frame_data_.take() {
            self.RetireFrame(frame);
        }
        self.activation_dependencies_.clear();
        self.blocking_allocation_groups_.clear();
        self.deadline_.Cancel();
    }
}

// Source: components/viz/service/surfaces/surface.h:401-471
// Source: components/viz/service/surfaces/surface.cc:1068
// Field ownership is declared above; Rust's Drop/Option/VecDeque replace C++
// weak pointers, scoped observations and circular deque lifetime plumbing.
