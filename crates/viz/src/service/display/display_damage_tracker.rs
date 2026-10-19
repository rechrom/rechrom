// Source: components/viz/service/display/display_damage_tracker.h:1-29
// Source: components/viz/service/display/display_damage_tracker.cc:1-30
#![allow(non_snake_case)]

use crate::common::surfaces::surface_id::SurfaceId;
use compositor_core::adapter::{
    begin_frame::BeginFrameArgs, geometry::Rect, scheduler_host::BeginFrameAck,
};
use compositor_core::{runtime::frame::RenderPassId, NativeCompositorFrame};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HandleInteraction {
    Yes,
    No,
    #[default]
    NoChange,
}

#[derive(Clone, Default)]
pub struct SurfaceDamageState {
    pub producer_token: u32,
    pub damage_start_token: Option<u32>,
    pub continuous: bool,
    pub pass_damage: BTreeMap<RenderPassId, Rect>,
}

#[derive(Clone, Debug, Default)]
pub struct SurfaceBeginFrameState {
    pub last_args: BeginFrameArgs,
    pub last_ack: BeginFrameAck,
}

// Source: components/viz/service/display/display_damage_tracker.h:30-66
// Source: components/viz/service/display/display_damage_tracker.cc:31-88
pub struct DisplayDamageTracker {
    begin_frame_source_id_: Option<u64>,
    root_surface_id_: Option<SurfaceId>,
    root_frame_missing_: bool,
    expecting_resize_damage_: bool,
    has_surface_damage_due_to_interaction_: bool,
    earliest_input_timestamp_: Option<Instant>,
    surface_states_: BTreeMap<SurfaceId, SurfaceBeginFrameState>,
    surfaces_to_ack_on_next_draw_: Vec<SurfaceId>,
    states_: BTreeMap<SurfaceId, SurfaceDamageState>,
}
impl Default for DisplayDamageTracker {
    fn default() -> Self {
        Self {
            begin_frame_source_id_: None,
            root_surface_id_: None,
            root_frame_missing_: true,
            expecting_resize_damage_: false,
            has_surface_damage_due_to_interaction_: false,
            earliest_input_timestamp_: None,
            surface_states_: BTreeMap::new(),
            surfaces_to_ack_on_next_draw_: Vec::new(),
            states_: BTreeMap::new(),
        }
    }
}
impl DisplayDamageTracker {
    pub fn SetDisplayBeginFrameSourceId(&mut self, id: u64) {
        self.begin_frame_source_id_ = Some(id);
    }
    pub fn SetRootFrameMissing(&mut self, missing: bool) {
        self.root_frame_missing_ = missing;
    }
    pub fn SetNewRootSurface(&mut self, id: SurfaceId) {
        self.root_surface_id_ = Some(id);
        self.root_frame_missing_ = true;
        self.SetRootSurfaceDamaged();
    }
    pub fn SetRootSurfaceDamaged(&mut self) {
        if let Some(id) = self.root_surface_id_ {
            if let Some(state) = self.states_.get_mut(&id) {
                for damage in state.pass_damage.values_mut() {
                    *damage =
                        Rect::FromBounds(i32::MIN / 4, i32::MIN / 4, i32::MAX / 4, i32::MAX / 4);
                }
            }
        }
    }
    pub fn IsRootSurfaceValid(&self) -> bool {
        self.root_surface_id_.is_some()
    }
    pub fn DisplayResized(&mut self) {
        self.expecting_resize_damage_ = true;
    }
    pub fn root_frame_missing(&self) -> bool {
        self.root_frame_missing_
    }
    pub fn expecting_root_surface_damage_because_of_resize(&self) -> bool {
        self.expecting_resize_damage_
    }
    pub fn reset_expecting_root_surface_damage_because_of_resize(&mut self) {
        self.expecting_resize_damage_ = false;
    }

    // Source: components/viz/service/display/display_damage_tracker.h:67-97
    // Source: components/viz/service/display/display_damage_tracker.cc:89-140
    pub fn ProcessSurfaceDamage(
        &mut self,
        id: SurfaceId,
        ack: BeginFrameAck,
        display_damaged: bool,
        interaction: HandleInteraction,
        earliest_input: Option<Instant>,
    ) {
        self.has_surface_damage_due_to_interaction_ |= interaction == HandleInteraction::Yes;
        if display_damaged {
            if let Some(timestamp) = earliest_input {
                self.earliest_input_timestamp_ = Some(
                    self.earliest_input_timestamp_
                        .map_or(timestamp, |old| old.min(timestamp)),
                );
            }
        }
        if Some(id) == self.root_surface_id_ {
            self.expecting_resize_damage_ = false;
        }
        if ack.frame_id.sequence_number != 0 {
            let state = self.surface_states_.entry(id).or_default();
            if ack.frame_id >= state.last_ack.frame_id {
                state.last_ack = ack;
            }
        }
    }
    pub fn HasPendingSurfaces(
        &self,
        begin_frame_args: &BeginFrameArgs,
        mut surface_has_unacked_frame: impl FnMut(SurfaceId) -> bool,
    ) -> bool {
        self.surface_states_.iter().any(|(id, state)| {
            state.last_args.IsValid()
                && state.last_args.frame_id == begin_frame_args.frame_id
                && state.last_ack.frame_id != begin_frame_args.frame_id
                && !surface_has_unacked_frame(*id)
        })
    }
    pub fn HasDamageDueToInteraction(&self) -> bool {
        self.has_surface_damage_due_to_interaction_
    }
    pub fn GetEarliestInputGenerationTimeOfDamagedSurfaces(&self) -> Option<Instant> {
        self.earliest_input_timestamp_
    }
    pub fn DidFinishDisplayFrame(&mut self) {
        self.has_surface_damage_due_to_interaction_ = false;
        self.earliest_input_timestamp_ = None;
    }

    // Source: components/viz/service/display/display_damage_tracker.h:98-137
    // Source: components/viz/service/display/display_damage_tracker.cc:141-242
    pub fn OnSurfaceDamaged(
        &mut self,
        id: SurfaceId,
        ack: BeginFrameAck,
        interaction: HandleInteraction,
        contributes_to_display: bool,
        earliest_input: Option<Instant>,
    ) -> bool {
        let display_damaged =
            ack.has_damage && (contributes_to_display || Some(id) == self.root_surface_id_);
        if display_damaged {
            self.surfaces_to_ack_on_next_draw_.push(id);
        }
        self.ProcessSurfaceDamage(id, ack, display_damaged, interaction, earliest_input);
        display_damaged
    }
    pub fn OnSurfaceDamageExpected(&mut self, id: SurfaceId, args: BeginFrameArgs) -> bool {
        let relevant = self.CheckBeginFrameSourceId(args.frame_id.source_id);
        self.surface_states_.entry(id).or_default().last_args = args;
        relevant
    }
    pub fn RunDrawCallbacks(&mut self, contained: &BTreeSet<SurfaceId>) -> Vec<SurfaceId> {
        let mut result: BTreeSet<_> = std::mem::take(&mut self.surfaces_to_ack_on_next_draw_)
            .into_iter()
            .collect();
        result.extend(contained.iter().copied());
        result.into_iter().collect()
    }
    pub fn CheckBeginFrameSourceId(&self, source_id: u64) -> bool {
        self.begin_frame_source_id_.is_none_or(|id| id == source_id)
            || source_id == BeginFrameArgs::kManualSourceId
    }
    pub fn CheckForDisplayDamage(
        &self,
        id: SurfaceId,
        previously_contained: &BTreeSet<SurfaceId>,
    ) -> bool {
        previously_contained.contains(&id)
    }

    // Source: components/viz/service/display/display_damage_tracker.h:138-159
    // Source: components/viz/service/display/display_damage_tracker.cc:243-314
    pub fn OnSurfaceMarkedForDestruction(&mut self, id: SurfaceId) {
        self.surface_states_.remove(&id);
        self.states_.remove(&id);
    }

    /// CPU pass damage accumulation used by the current Engine adapter. This is
    /// the resolved-frame damage state consumed by SurfaceAggregator.
    pub fn UpdateFrame(
        &mut self,
        id: SurfaceId,
        frame: &NativeCompositorFrame,
        previous: Option<&NativeCompositorFrame>,
    ) -> SurfaceDamageState {
        let token = frame.metadata.frame_token;
        let old = self.states_.get(&id);
        let continuous = old.is_none_or(|state| {
            state.continuous && state.producer_token.checked_add(1) == Some(token)
        });
        let damage_start_token = old
            .and_then(|state| state.damage_start_token)
            .or(Some(token));
        let mut pass_damage = BTreeMap::new();
        for pass in &frame.render_pass_list {
            let damage = previous
                .and_then(|old_frame| {
                    old_frame
                        .render_pass_list
                        .iter()
                        .find(|old| old.id == pass.id)
                })
                .map(|old_pass| {
                    if old_pass.output_rect == pass.output_rect {
                        UnionRect(
                            old.and_then(|state| state.pass_damage.get(&pass.id))
                                .copied()
                                .unwrap_or_default(),
                            pass.damage_rect,
                        )
                    } else {
                        pass.output_rect
                    }
                })
                .unwrap_or(pass.damage_rect);
            pass_damage.insert(pass.id, damage);
        }
        let state = SurfaceDamageState {
            producer_token: token,
            damage_start_token,
            continuous,
            pass_damage,
        };
        self.states_.insert(id, state.clone());
        if Some(id) == self.root_surface_id_ {
            self.root_frame_missing_ = false;
            self.expecting_resize_damage_ = false;
        }
        state
    }
    pub fn Damage(&self, id: SurfaceId, pass: RenderPassId, fallback: Rect) -> Rect {
        self.states_
            .get(&id)
            .and_then(|state| state.pass_damage.get(&pass))
            .copied()
            .unwrap_or(fallback)
    }
    pub fn State(&self, id: SurfaceId) -> Option<&SurfaceDamageState> {
        self.states_.get(&id)
    }
    pub fn DidFinishFrame(&mut self, id: SurfaceId) {
        if let Some(state) = self.states_.get_mut(&id) {
            state.damage_start_token = None;
            state.continuous = true;
            for rect in state.pass_damage.values_mut() {
                *rect = Rect::default();
            }
        }
        self.DidFinishDisplayFrame();
    }
}

pub fn UnionRect(lhs: Rect, rhs: Rect) -> Rect {
    let mut result = lhs;
    result.Union(rhs);
    result
}
