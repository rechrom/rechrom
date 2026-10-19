// Source: components/viz/service/display/resolved_frame_data.h:1-35
// Source: components/viz/service/display/resolved_frame_data.cc:1-53
// DisplayResourceProvider registration is represented by strict validation and
// identity resource mapping in the CPU path. Offset lookups enter as values.
#![allow(non_snake_case)]

use crate::{common::surfaces::surface_id::SurfaceId, service::surfaces::surface::FrameData};
use compositor_core::adapter::{
    geometry::{Rect, Vector2dF},
    input_services::OffsetTag,
    layer_tree_impl::ResourceId,
};
use compositor_core::runtime::frame::{QuadMaterial, RenderPassId};
use compositor_core::{NativeCompositorFrame, NativeCompositorRenderPass};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    error::Error,
    fmt,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameValidationError {
    MissingRenderPass,
    DuplicateRenderPass,
    DuplicateResource,
    MissingSharedQuadState,
    MissingResource,
    InvalidRenderPassOrder,
}
impl fmt::Display for FrameValidationError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{self:?}")
    }
}
impl Error for FrameValidationError {}

// Source: components/viz/service/display/resolved_frame_data.h:36-68
// Source: components/viz/service/display/resolved_frame_data.cc:54-61
#[derive(Clone, Copy, Debug, Default)]
pub struct ResolvedQuadData {
    pub remapped_resource_id: ResourceId,
}

#[derive(Clone)]
pub struct FixedPassData<'a> {
    pub render_pass: Option<&'a NativeCompositorRenderPass>,
    pub embed_count: usize,
    pub remapped_id: u64,
    pub render_pass_id: RenderPassId,
    pub is_root: bool,
    pub draw_quads: Vec<ResolvedQuadData>,
}

// Source: components/viz/service/display/resolved_frame_data.h:69-129
// Source: components/viz/service/display/resolved_frame_data.cc:62-71
#[derive(Clone, Debug, Default)]
pub struct AggregationPassData {
    pub embedded_passes: BTreeSet<RenderPassId>,
    pub in_copy_request_pass: bool,
    pub in_pixel_moving_filter_pass: bool,
    pub in_cached_render_pass: bool,
    pub has_damage: bool,
    pub will_draw: bool,
    pub added_damage: Rect,
    pub prevent_merge: bool,
}
impl AggregationPassData {
    pub fn Reset(&mut self) {
        *self = Self::default();
    }
}

// Source: components/viz/service/display/resolved_frame_data.h:130-152
// Source: components/viz/service/display/resolved_frame_data.cc:72-81
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MergeState {
    #[default]
    kInitState,
    kNotMerged,
    kAlwaysMerged,
    kSomeTimesMerged,
}
#[derive(Clone, Debug, Default)]
pub struct PersistentPassData {
    pub parent_clip_rect: Rect,
    pub merge_state: MergeState,
}

// Source: components/viz/service/display/resolved_frame_data.h:153-215
// Source: components/viz/service/display/resolved_frame_data.cc:82-108
#[derive(Clone)]
pub struct ResolvedPassData<'a> {
    fixed_: FixedPassData<'a>,
    aggregation_: AggregationPassData,
    current_persistent_data_: PersistentPassData,
    previous_persistent_data_: PersistentPassData,
}
impl<'a> ResolvedPassData<'a> {
    pub fn render_pass(&self) -> &'a NativeCompositorRenderPass {
        self.fixed_
            .render_pass
            .expect("render pass is set during aggregation")
    }
    pub fn remapped_id(&self) -> u64 {
        self.fixed_.remapped_id
    }
    pub fn render_pass_id(&self) -> RenderPassId {
        self.fixed_.render_pass_id
    }
    pub fn is_root(&self) -> bool {
        self.fixed_.is_root
    }
    pub fn draw_quads(&self) -> &[ResolvedQuadData] {
        &self.fixed_.draw_quads
    }
    pub fn embed_count(&self) -> usize {
        self.fixed_.embed_count
    }
    pub fn IsUnembedded(&self) -> bool {
        !self.fixed_.is_root && self.fixed_.embed_count == 0
    }
    pub fn aggregation(&self) -> &AggregationPassData {
        &self.aggregation_
    }
    pub fn aggregation_mut(&mut self) -> &mut AggregationPassData {
        &mut self.aggregation_
    }
    pub fn current_persistent_data(&mut self) -> &mut PersistentPassData {
        &mut self.current_persistent_data_
    }
    pub fn previous_persistent_data(&self) -> &PersistentPassData {
        &self.previous_persistent_data_
    }
    pub fn CopyAndResetPersistentPassData(&mut self) {
        self.previous_persistent_data_ = self.current_persistent_data_.clone();
        self.current_persistent_data_ = PersistentPassData::default();
    }
    pub fn SetCompositorRenderPass(&mut self, pass: &'a NativeCompositorRenderPass) {
        assert_eq!(pass.id, self.fixed_.render_pass_id);
        self.fixed_.render_pass = Some(pass);
    }
    pub fn ResetCompositorRenderPass(&mut self) {
        self.fixed_.render_pass = None;
    }
}

// Source: components/viz/service/display/resolved_frame_data.h:216-236
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameDamageType {
    kFull,
    kFrame,
    kNone,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OffsetTagData {
    pub last_offset: Vector2dF,
    pub current_offset: Vector2dF,
    pub last_containing_rect: Rect,
    pub current_containing_rect: Rect,
    pub defined_in_frame: bool,
}

// Source: components/viz/service/display/resolved_frame_data.h:237-325
// Source: components/viz/service/display/resolved_frame_data.cc:109-166
pub struct ResolvedFrameData<'a> {
    surface_id_: SurfaceId,
    frame_: &'a FrameData,
    valid_: bool,
    frame_index_: u32,
    previous_frame_index_: u32,
    passes_: Vec<ResolvedPassData<'a>>,
    pass_map_: BTreeMap<RenderPassId, usize>,
    aggregated_id_map_: BTreeMap<RenderPassId, u64>,
    offset_tag_data_: BTreeMap<OffsetTag, OffsetTagData>,
    offset_tag_added_damage_: Rect,
    used_in_aggregation_: bool,
}
impl<'a> ResolvedFrameData<'a> {
    pub fn new(surface_id: SurfaceId, frame: &'a FrameData) -> Result<Self, FrameValidationError> {
        Self::new_with_previous(surface_id, frame, 0, None)
    }
    pub fn new_with_previous(
        surface_id: SurfaceId,
        frame: &'a FrameData,
        previous_frame_index: u32,
        previous_root_pass_id: Option<u64>,
    ) -> Result<Self, FrameValidationError> {
        ValidateFrame(&frame.frame)?;
        let mut resolved = Self {
            surface_id_: surface_id,
            frame_: frame,
            valid_: false,
            frame_index_: frame.frame_index,
            previous_frame_index_: previous_frame_index,
            passes_: Vec::new(),
            pass_map_: BTreeMap::new(),
            aggregated_id_map_: BTreeMap::new(),
            offset_tag_data_: BTreeMap::new(),
            offset_tag_added_damage_: Rect::default(),
            used_in_aggregation_: true,
        };
        resolved.UpdateActiveFrame(previous_root_pass_id)?;
        Ok(resolved)
    }
    pub fn surface_id(&self) -> SurfaceId {
        self.surface_id_
    }
    pub fn frame(&self) -> &'a FrameData {
        self.frame_
    }
    pub fn is_valid(&self) -> bool {
        self.valid_
    }
    pub fn previous_frame_index(&self) -> u32 {
        self.previous_frame_index_
    }
    pub fn SetFullDamageForNextAggregation(&mut self) {
        self.previous_frame_index_ = 0;
    }
    pub fn passes(&self) -> &[ResolvedPassData<'a>] {
        &self.passes_
    }
    pub fn root_pass(&self) -> &ResolvedPassData<'a> {
        self.passes_.last().expect("validated frame has root")
    }
    pub fn pass(&self, id: RenderPassId) -> Option<&ResolvedPassData<'a>> {
        self.pass_map_.get(&id).map(|index| &self.passes_[*index])
    }

    // Source: components/viz/service/display/resolved_frame_data.h:326-401
    // Source: components/viz/service/display/resolved_frame_data.cc:167-300
    fn UpdateActiveFrame(
        &mut self,
        previous_root_pass_id: Option<u64>,
    ) -> Result<(), FrameValidationError> {
        let frame = &self.frame_.frame;
        let resources: HashSet<_> = frame
            .resource_list
            .iter()
            .map(|resource| resource.id)
            .collect();
        let mut next_aggregate_id = 1u64;
        for (index, pass) in frame.render_pass_list.iter().enumerate() {
            let is_root = index + 1 == frame.render_pass_list.len();
            if self.pass_map_.contains_key(&pass.id) {
                self.SetInvalid();
                return Err(FrameValidationError::DuplicateRenderPass);
            }
            let remapped_id = if is_root {
                previous_root_pass_id.unwrap_or_else(|| {
                    let id = next_aggregate_id;
                    next_aggregate_id += 1;
                    id
                })
            } else {
                let id = next_aggregate_id;
                next_aggregate_id += 1;
                id
            };
            self.aggregated_id_map_.insert(pass.id, remapped_id);
            let mut draw_quads = Vec::with_capacity(pass.quad_list.len());
            for quad in &pass.quad_list {
                let resource = match quad.material {
                    QuadMaterial::Tile { resource_id, .. } => resource_id,
                    QuadMaterial::CompositorRenderPass {
                        render_pass_id,
                        mask_resource_id,
                        ..
                    } => {
                        let Some(child) = self.pass_map_.get(&render_pass_id).copied() else {
                            self.SetInvalid();
                            return Err(FrameValidationError::InvalidRenderPassOrder);
                        };
                        self.passes_[child].fixed_.embed_count += 1;
                        mask_resource_id
                    }
                    _ => ResourceId::default(),
                };
                if resource.0 != 0 && !resources.contains(&resource) {
                    self.SetInvalid();
                    return Err(FrameValidationError::MissingResource);
                }
                draw_quads.push(ResolvedQuadData {
                    remapped_resource_id: resource,
                });
            }
            let resolved = ResolvedPassData {
                fixed_: FixedPassData {
                    render_pass: Some(pass),
                    embed_count: 0,
                    remapped_id,
                    render_pass_id: pass.id,
                    is_root,
                    draw_quads,
                },
                aggregation_: AggregationPassData::default(),
                current_persistent_data_: PersistentPassData::default(),
                previous_persistent_data_: PersistentPassData::default(),
            };
            self.pass_map_.insert(pass.id, self.passes_.len());
            self.passes_.push(resolved);
        }
        self.valid_ = true;
        Ok(())
    }

    // Source: components/viz/service/display/resolved_frame_data.cc:301-360
    // Offset values are injected by the compositor host. Rechrom's current
    // SharedQuadState has no OffsetTag field, so this method retains persistent
    // value/damage state without fabricating tagged geometry.
    pub fn UpdateOffsetTag(&mut self, tag: OffsetTag, offset: Vector2dF, containing_rect: Rect) {
        let data = self.offset_tag_data_.entry(tag).or_default();
        data.current_offset = offset;
        data.current_containing_rect.Union(containing_rect);
        data.defined_in_frame = true;
    }

    // Source: components/viz/service/display/resolved_frame_data.cc:361-444
    pub fn ComputeOffsetTagContainingRects(&mut self) {
        let output = self.GetOutputRect();
        for data in self.offset_tag_data_.values_mut() {
            data.current_containing_rect.Intersect(output);
        }
    }

    // Source: components/viz/service/display/resolved_frame_data.cc:445-508
    pub fn RecomputeOffsetTagDamage(&mut self) {
        self.offset_tag_added_damage_ = Rect::default();
        let surface_damage = self.BaseSurfaceDamage();
        let output = self.GetOutputRect();
        if surface_damage == output {
            return;
        }
        for data in self.offset_tag_data_.values() {
            if data.current_offset != data.last_offset {
                self.offset_tag_added_damage_
                    .Union(data.current_containing_rect);
                self.offset_tag_added_damage_
                    .Union(data.last_containing_rect);
            } else if data.current_offset != Vector2dF::default() {
                let mut shifted_damage = data.current_containing_rect;
                shifted_damage.Intersect(surface_damage);
                self.offset_tag_added_damage_.Union(shifted_damage);
                if !data.last_containing_rect.IsEmpty()
                    && !data
                        .current_containing_rect
                        .Contains(data.last_containing_rect)
                {
                    self.offset_tag_added_damage_
                        .Union(data.last_containing_rect);
                }
            }
        }
        self.offset_tag_added_damage_.Intersect(output);
    }

    // Source: components/viz/service/display/resolved_frame_data.cc:509-583
    pub fn SetInvalid(&mut self) {
        self.frame_index_ = self.frame_.frame_index;
        self.pass_map_.clear();
        self.passes_.clear();
        self.valid_ = false;
    }
    pub fn WasUsedInAggregation(&self) -> bool {
        self.used_in_aggregation_
    }
    pub fn ResetAfterAggregation(&mut self) {
        for pass in &mut self.passes_ {
            pass.aggregation_mut().Reset();
            pass.CopyAndResetPersistentPassData();
            pass.ResetCompositorRenderPass();
        }
        self.previous_frame_index_ = self.frame_index_;
        self.used_in_aggregation_ = false;
        self.offset_tag_data_
            .retain(|_, data| data.defined_in_frame);
        for data in self.offset_tag_data_.values_mut() {
            data.last_containing_rect = data.current_containing_rect;
            data.current_containing_rect = Rect::default();
            data.last_offset = data.current_offset;
            data.current_offset = Vector2dF::default();
            data.defined_in_frame = false;
        }
        self.offset_tag_added_damage_ = Rect::default();
    }
    pub fn WillDraw(&self) -> bool {
        self.root_pass().aggregation().will_draw
    }
    pub fn GetRenderPassDataById(&self, id: RenderPassId) -> &ResolvedPassData<'a> {
        self.pass(id).expect("resolved render pass id")
    }
    pub fn GetRootRenderPassData(&self) -> &ResolvedPassData<'a> {
        self.root_pass()
    }

    // Source: components/viz/service/display/resolved_frame_data.cc:584-657
    pub fn GetFrameDamageType(&self) -> FrameDamageType {
        assert!(self.valid_ && self.used_in_aggregation_);
        if self.previous_frame_index_ == self.frame_index_ {
            FrameDamageType::kNone
        } else if self.previous_frame_index_ > 0
            && self.frame_index_ == self.previous_frame_index_.wrapping_add(1)
        {
            FrameDamageType::kFrame
        } else {
            FrameDamageType::kFull
        }
    }
    fn BaseSurfaceDamage(&self) -> Rect {
        match self.GetFrameDamageType() {
            FrameDamageType::kFull => self.GetOutputRect(),
            FrameDamageType::kFrame => self.root_pass().render_pass().damage_rect,
            FrameDamageType::kNone => Rect::default(),
        }
    }
    pub fn GetSurfaceDamage(&self) -> Rect {
        let mut damage = self.BaseSurfaceDamage();
        damage.Union(self.offset_tag_added_damage_);
        damage
    }
    pub fn GetOutputRect(&self) -> Rect {
        self.root_pass().render_pass().output_rect
    }
    pub fn ReuseActiveFrame(&mut self) {
        assert_eq!(self.frame_.frame.render_pass_list.len(), self.passes_.len());
        for (resolved, pass) in self
            .passes_
            .iter_mut()
            .zip(&self.frame_.frame.render_pass_list)
        {
            resolved.SetCompositorRenderPass(pass);
        }
        for data in self.offset_tag_data_.values_mut() {
            data.current_containing_rect = data.last_containing_rect;
        }
    }
}

// Source: components/viz/service/display/resolved_frame_data.h:402-405
pub fn ValidateFrame(frame: &NativeCompositorFrame) -> Result<(), FrameValidationError> {
    if frame.render_pass_list.is_empty() {
        return Err(FrameValidationError::MissingRenderPass);
    }
    let mut passes = BTreeMap::new();
    for pass in &frame.render_pass_list {
        if passes.insert(pass.id, ()).is_some() {
            return Err(FrameValidationError::DuplicateRenderPass);
        }
    }
    let mut resources = HashSet::new();
    for resource in &frame.resource_list {
        if !resources.insert(resource.id) {
            return Err(FrameValidationError::DuplicateResource);
        }
    }
    for (parent_index, pass) in frame.render_pass_list.iter().enumerate() {
        for quad in &pass.quad_list {
            let Some(handle) = quad.geometry.shared_quad_state else {
                return Err(FrameValidationError::MissingSharedQuadState);
            };
            if pass.TrySharedQuadState(handle).is_none() {
                return Err(FrameValidationError::MissingSharedQuadState);
            }
            match quad.material {
                QuadMaterial::Tile { resource_id, .. } if !resources.contains(&resource_id) => {
                    return Err(FrameValidationError::MissingResource);
                }
                QuadMaterial::CompositorRenderPass {
                    render_pass_id,
                    mask_resource_id,
                    ..
                } => {
                    let Some(child_index) = frame
                        .render_pass_list
                        .iter()
                        .position(|candidate| candidate.id == render_pass_id)
                    else {
                        return Err(FrameValidationError::MissingRenderPass);
                    };
                    if child_index >= parent_index {
                        return Err(FrameValidationError::InvalidRenderPassOrder);
                    }
                    if mask_resource_id.0 != 0 && !resources.contains(&mask_resource_id) {
                        return Err(FrameValidationError::MissingResource);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
