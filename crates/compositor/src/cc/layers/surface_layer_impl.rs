// Translation status: partial (resident range/quad path complete; submission
// visibility callback and debug rainbow border require their host services).
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/surface_layer_impl.h:1-20,26-83,94-106;
//         cc/layers/surface_layer_impl.cc:64-164,193-262.
use super::{
    append_quads_context::AppendQuadsContext,
    append_quads_data::AppendQuadsData,
    layer_impl::{LayerImpl, LayerImplVirtual},
};
use crate::adapter::{
    geometry::{Rect, ScaleToEnclosingRect},
    layer_impl::{LayerImplContext, LayerImplTypes, LayerType},
    layer_tree_impl::{Color4f, LayerTreeId},
    solid_color_layer_impl::SolidColorRenderPass,
    surface_layer_impl::{SurfaceLayerImplTypes, SurfaceRenderPass},
};

/// Resident implementation-side SurfaceLayer owner. Surface frames and
/// resources are deliberately not retained here; Viz resolves `surface_range_`.
pub struct SurfaceLayerImpl<B: LayerImplTypes> {
    base_: LayerImpl<B>,
    surface_range_: B::SurfaceRange,
    deadline_in_frames_: Option<u32>,
    stretch_content_to_fill_bounds_: bool,
    surface_hit_testable_: bool,
    has_pointer_events_none_: bool,
    is_reflection_: bool,
    override_child_paint_flags_: bool,
}

impl<B: LayerImplTypes> SurfaceLayerImpl<B> {
    pub fn base(&self) -> &LayerImpl<B> {
        &self.base_
    }
    pub fn base_mut(&mut self) -> &mut LayerImpl<B> {
        &mut self.base_
    }
    pub fn range(&self) -> &B::SurfaceRange {
        &self.surface_range_
    }
    pub fn GetLayerType(&self) -> LayerType {
        LayerType::kSurface
    }
}

impl<B: SurfaceLayerImplTypes> SurfaceLayerImpl<B> {
    pub fn Create(tree: LayerTreeId, id: i32, c: &mut LayerImplContext<'_, B>) -> Box<B::Layer> {
        Box::new(B::WrapSurfaceLayerImpl(Self::new(tree, id, c)))
    }

    pub fn new(tree: LayerTreeId, id: i32, c: &mut LayerImplContext<'_, B>) -> Self {
        Self {
            base_: LayerImpl::new(tree, id, c),
            surface_range_: B::DefaultSurfaceRange(),
            deadline_in_frames_: Some(0),
            stretch_content_to_fill_bounds_: false,
            surface_hit_testable_: false,
            has_pointer_events_none_: false,
            is_reflection_: false,
            override_child_paint_flags_: false,
        }
    }

    pub fn SetRange(
        &mut self,
        range: B::SurfaceRange,
        deadline: Option<u32>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if B::SurfaceRangesEqual(&self.surface_range_, &range)
            && self.deadline_in_frames_ == deadline
        {
            return;
        }
        self.surface_range_ = range;
        self.deadline_in_frames_ = deadline;
        self.base_.NoteLayerPropertyChanged(c);
    }

    pub fn SetStretchContentToFillBounds(&mut self, value: bool, c: &mut LayerImplContext<'_, B>) {
        if self.stretch_content_to_fill_bounds_ != value {
            self.stretch_content_to_fill_bounds_ = value;
            self.base_.NoteLayerPropertyChanged(c);
        }
    }

    pub fn SetIsReflection(&mut self, value: bool, c: &mut LayerImplContext<'_, B>) {
        if self.is_reflection_ != value {
            self.is_reflection_ = value;
            self.base_.NoteLayerPropertyChanged(c);
        }
    }

    pub fn CreateLayerImpl(
        &self,
        tree: LayerTreeId,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Self::Create(tree, self.base_.id(), c)
    }

    pub fn CopyPropertiesTo(
        &self,
        target: &mut Self,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        self.base_.CopyPropertiesTo(&mut target.base_, source, c);
        target.surface_range_ = self.surface_range_.clone();
        target.deadline_in_frames_ = self.deadline_in_frames_;
        target.stretch_content_to_fill_bounds_ = self.stretch_content_to_fill_bounds_;
        target.surface_hit_testable_ = self.surface_hit_testable_;
        target.has_pointer_events_none_ = self.has_pointer_events_none_;
        target.is_reflection_ = self.is_reflection_;
        target.override_child_paint_flags_ = self.override_child_paint_flags_;
    }

    pub fn AppendQuads(
        &mut self,
        _context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        let scale = c.tree.device_scale_factor();
        let bounds = self.base_.bounds(c);
        let quad_rect =
            ScaleToEnclosingRect(Rect::new(0, 0, bounds.width(), bounds.height()), scale);
        let mut visible = self
            .base_
            .draw_properties()
            .occlusion_in_content_space
            .GetUnoccludedContentRect(Rect::new(0, 0, bounds.width(), bounds.height()));
        visible = ScaleToEnclosingRect(visible, scale);
        visible.Intersect(quad_rect);
        if visible.IsEmpty() {
            return;
        }

        let state = pass.CreateAndAppendSharedQuadState();
        self.base_.PopulateScaledSharedQuadState(
            pass.SharedQuadStateMutable(state),
            scale,
            self.base_.contents_opaque(),
            c,
        );
        if B::SurfaceRangeIsValid(&self.surface_range_) {
            pass.AppendSurfaceDrawQuad(
                state,
                quad_rect,
                visible,
                &self.surface_range_,
                self.base_.background_color(),
                self.stretch_content_to_fill_bounds_,
                self.is_reflection_,
                true,
            );
            data.activation_dependencies
                .push(B::SurfaceRangeEnd(&self.surface_range_));
            if let Some(deadline) = self.deadline_in_frames_ {
                data.deadline_in_frames = Some(data.deadline_in_frames.unwrap_or(0).max(deadline));
            } else {
                data.use_default_lower_bound_deadline = true;
            }
        } else {
            let quad = pass.CreateAndAppendSolidColorDrawQuad();
            pass.SolidColorDrawQuadSetNew(
                quad,
                state,
                quad_rect,
                visible,
                self.base_.background_color(),
                false,
            );
        }
        self.deadline_in_frames_ = Some(0);
    }
}

pub struct SurfaceLayerImplVirtual;
impl<B: SurfaceLayerImplTypes> LayerImplVirtual<B> for SurfaceLayerImplVirtual {
    fn GetLayerType(&self, _base: &LayerImpl<B>) -> LayerType {
        LayerType::kSurface
    }
    fn CreateLayerImpl(
        &self,
        base: &LayerImpl<B>,
        tree: LayerTreeId,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        SurfaceLayerImpl::Create(tree, base.id(), c)
    }
}

#[allow(dead_code)]
fn _source_field_shape<B: SurfaceLayerImplTypes>(
    layer: &SurfaceLayerImpl<B>,
) -> (bool, bool, bool) {
    (
        layer.surface_hit_testable_,
        layer.has_pointer_events_none_,
        layer.override_child_paint_flags_,
    )
}
