// Translation status: complete
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/tile_based_layer_impl.h:1-37,49-53,64,99,691-695.
// Rust modules replace the include guards/includes/namespace. Generic T is the
// same-package native Tiling concept; the concrete base remains resident.
use super::{
    append_quads_context::AppendQuadsContext, append_quads_data::AppendQuadsData,
    draw_mode::DrawMode, layer_impl::LayerImpl,
};
use crate::{
    adapter::{
        geometry::{self, Rect, RectF, Size, SizeF, Transform, Vector2d},
        layer_impl::{LayerImplContext, LayerImplSharedQuadState},
        layer_tree_impl::{Color4f, LayerTreeId, ResourceId},
        property_tree::PropertyTreeTransform,
        solid_color_layer_impl::SolidColorRenderPass,
        tile_based_layer_impl::*,
    },
    cc::{
        base::math_util::MathUtil,
        debug::{debug_colors::DebugColors, layer_tree_debug_state::DebugBorderType},
        tiles::{
            tile_draw_info::Mode,
            tile_priority::TileResolution,
            tiling_internal::TileLike,
            tiling_set_coverage_iterator::{TilingSetCoverageIterator, TilingWithCover},
        },
        trees::{occlusion::Occlusion, property_ids::kInvalidPropertyNodeId},
    },
};
use std::marker::PhantomData;
type Coverage<'a, B, T> = TilingSetCoverageIterator<
    'a,
    T,
    <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend,
>;
// Source: cc/layers/tile_based_layer_impl.h:38-42.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilingResolution {
    kHigh,
    kAboveHigh,
    kBelowHigh,
}
// Source: cc/layers/tile_based_layer_impl.h:44-47.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IterationStatus {
    kContinue,
    kStop,
}
// Native TileLike plus the exact Tile queries consumed by this template. This
// same-package concept is implemented by canonical Tile, never a raster owner.
pub trait TileBasedTile: TileLike {
    fn draw_mode(&self) -> Mode;
    fn GetResourceId(&self) -> Option<ResourceId>;
    fn GetResourceSize(&self) -> Option<Size>;
    fn GetSolidColor(&self) -> Option<Color4f>;
}
// Source: cc/layers/tile_based_layer_impl.h:97,135-187,208-222.
// The derived receiver is a call-scoped borrow of the actual fields beside the
// base. Cover borrows its resident tilings; hooks keep native virtual dispatch.
// Default no-op/false bodies below are the source's actual base virtual bodies.
pub trait TileBasedLayerDelegate<B: TileBasedLayerImplTypes, T: TilingWithCover> {
    // A context lifetime is independent of the resident derived-field borrow.
    type Context<'context>: TileBasedLayerContext<B>;
    // Source: cc/layers/layer_impl.cc:329-339. Inherited virtual dispatch is
    // selected from the actual derived sibling receiver, before the base helper.
    fn GetDebugBorderProperties(
        &self,
        base: &LayerImpl<B>,
        c: &LayerImplContext<'_, B>,
    ) -> (Color4f, f32) {
        base.GetDebugBorderProperties(c)
    }
    fn GetMaximumContentsScaleForUseInAppendQuads(&self) -> f32;
    fn RecordedBounds(&self) -> Rect;
    fn IsDirectlyCompositedImage(&self) -> bool;
    fn GetTilingResolutionForDebugBorders(&self, tiling: &T) -> TilingResolution;
    fn Cover(&self, coverage_rect: Rect, coverage_scale: f32, ideal: f32) -> Coverage<'_, B, T>;
    fn GetIdealContentsScaleKey(&self) -> f32;
    fn WillProcessReadyToDrawTile(&self, _iter: &Coverage<'_, B, T>) {}
    fn ShouldUpdateApproximatedVisibleContentArea(&self, _resolution: TileResolution) -> bool {
        false
    }
    fn ShouldReportTileAsMissing(&self, _geometry: Rect, _viewport: Rect) -> bool {
        false
    }
    fn GetScaledViewportForTilePriority(&self, _scale: f32) -> Rect {
        Rect::default()
    }
    fn DidAppendQuad(
        &self,
        _base: &LayerImpl<B>,
        _quad: &B::DrawQuad,
        _iter: &Coverage<'_, B, T>,
        _data: &mut AppendQuadsData<B::SurfaceId>,
        _checkerboard: bool,
        _c: &mut Self::Context<'_>,
    ) {
    }
    fn ValidateTilingSetForContentsResourceId(&self) -> bool {
        true
    }
    fn AppendQuadsForResourcelessSoftwareDraw(
        &self,
        base: &TileBasedLayerImpl<B, T>,
        context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        shared: B::SharedQuadStateHandle,
        occlusion: &Occlusion<B::Transform>,
        c: &mut Self::Context<'_>,
    );
    fn WillAppendQuads(&self, _base: &mut TileBasedLayerImpl<B, T>) {}
    fn SanityCheckTilingState(
        &self,
        _base: &TileBasedLayerImpl<B, T>,
        _c: &LayerImplContext<'_, B>,
    ) {
    }
}
// Source: cc/layers/tile_based_layer_impl.h:229-243.
pub struct TileBasedLayerImpl<B: TileBasedLayerImplTypes, T: TilingWithCover> {
    base_: LayerImpl<B>,
    is_backdrop_filter_mask_: bool,
    solid_color_: Option<Color4f>,
    last_append_quads_scales_: Vec<f32>,
    damage_rect_: Rect,
    nearest_neighbor_: bool,
    produced_tile_last_append_quads_: bool,
    tiling_type_: PhantomData<fn() -> T>,
}
impl<B: TileBasedLayerImplTypes, T: TilingWithCover> TileBasedLayerImpl<B, T> {
    // Source: cc/layers/tile_based_layer_impl.h:100-101,229-243.
    pub fn new(tree: LayerTreeId, id: i32, c: &mut LayerImplContext<'_, B>) -> Self {
        Self {
            base_: LayerImpl::new(tree, id, c),
            is_backdrop_filter_mask_: false,
            solid_color_: None,
            last_append_quads_scales_: Vec::new(),
            damage_rect_: Rect::default(),
            nearest_neighbor_: false,
            produced_tile_last_append_quads_: true,
            tiling_type_: PhantomData,
        }
    }
    // The actual resident base, not a projected layer record.
    pub fn base(&self) -> &LayerImpl<B> {
        &self.base_
    }
    pub fn base_mut(&mut self) -> &mut LayerImpl<B> {
        &mut self.base_
    }
    // Source: cc/layers/tile_based_layer_impl.h:54-62.
    pub fn SetIsBackdropFilterMask(&mut self, value: bool, c: &mut LayerImplContext<'_, B>) {
        if self.is_backdrop_filter_mask() == value {
            return;
        }
        self.is_backdrop_filter_mask_ = value;
        self.base_
            .SetNeedsPushProperties(LayerImpl::<B>::kChangedGeneralProperty, c);
    }
    pub(crate) fn SplitTileClient(
        &mut self,
    ) -> (
        super::layer_impl::LayerImplTileClient<'_>,
        &crate::cc::layers::draw_properties::DrawProperties<B::Transform>,
        &bool,
    ) {
        let (client, draw) = self.base_.SplitTileClient();
        (client, draw, &self.is_backdrop_filter_mask_)
    }
    pub fn is_backdrop_filter_mask(&self) -> bool {
        self.is_backdrop_filter_mask_
    }
    // Source: cc/layers/tile_based_layer_impl.h:68-72.
    pub fn GetDamageRect(&self) -> Rect {
        self.damage_rect_
    }
    pub fn ResetChangeTracking(&mut self) {
        self.base_.ResetChangeTracking();
        self.damage_rect_ = Rect::default();
    }
    // Source: cc/layers/tile_based_layer_impl.h:74-77.
    pub fn GetEnclosingVisibleRectInTargetSpace(
        &self,
        delegate: &impl TileBasedLayerDelegate<B, T>,
        c: &LayerImplContext<'_, B>,
    ) -> Rect {
        self.base_.GetScaledEnclosingVisibleRectInTargetSpace(
            delegate.GetMaximumContentsScaleForUseInAppendQuads(),
            c,
        )
    }
    // Source: cc/layers/tile_based_layer_impl.h:82-95,103,106-125.
    pub fn GetNearestNeighbor(&self) -> bool {
        self.nearest_neighbor_
    }
    pub fn SetSolidColor(&mut self, color: Option<Color4f>) {
        self.solid_color_ = color;
    }
    pub fn GetLastAppendQuadsScalesForTesting(&mut self) -> &mut Vec<f32> {
        &mut self.last_append_quads_scales_
    }
    pub fn set_produced_tile_last_append_quads(&mut self, value: bool) {
        self.produced_tile_last_append_quads_ = value;
    }
    pub fn produced_tile_last_append_quads(&self) -> bool {
        self.produced_tile_last_append_quads_
    }
    pub fn solid_color(&self) -> Option<Color4f> {
        self.solid_color_
    }
    pub fn ClearLastAppendQuadsScales(&mut self) {
        self.last_append_quads_scales_.clear();
    }
    pub fn AddScaleToLastAppendQuadsScales(&mut self, scale: f32) {
        if self
            .last_append_quads_scales_
            .last()
            .is_none_or(|last| *last != scale)
        {
            self.last_append_quads_scales_.push(scale);
        }
    }
    /// Same inherited base and damage storage, split for the source's tile
    /// callback while a resident tile in its Picture owner is still borrowed.
    pub(crate) fn SplitTileNotification(
        &mut self,
    ) -> (super::layer_impl::LayerImplTileClient<'_>, &mut Rect) {
        (self.base_.SplitTileClient().0, &mut self.damage_rect_)
    }
    pub fn UnionWithExistingDamage(&mut self, rect: Rect) {
        self.damage_rect_.Union(rect);
    }
    pub fn SetNearestNeighbor(&mut self, value: bool) {
        self.nearest_neighbor_ = value;
    }
    pub fn LastAppendQuadsScalesContains(&self, scale: f32) -> bool {
        self.last_append_quads_scales_.contains(&scale)
    }
    // Source: cc/layers/tile_based_layer_impl.h:127-133,245-281.
    // The source receiver reads only the derived Cover/RecordedBounds virtuals,
    // so no base borrow is retained while the caller's functor updates the base.
    pub fn ForEachVisibleTile<D: TileBasedLayerDelegate<B, T>>(
        delegate: &D,
        coverage: Rect,
        max_scale: f32,
        ideal: f32,
        occlusion: &Occlusion<B::Transform>,
        mut functor: impl FnMut(&Coverage<'_, B, T>, Rect) -> IterationStatus,
    ) {
        let recorded = geometry::ScaleToEnclosingRect(delegate.RecordedBounds(), max_scale);
        let mut iter = delegate.Cover(coverage, max_scale, ideal);
        while iter.IsValid() {
            let geometry = iter.geometry_rect();
            if !recorded.Intersects(geometry) {
                iter.Advance();
                continue;
            }
            let visible = occlusion.GetUnoccludedContentRect(geometry);
            if visible.IsEmpty() {
                iter.Advance();
                continue;
            }
            if functor(&iter, visible) == IterationStatus::kStop {
                break;
            }
            iter.Advance();
        }
    }
}
impl<B: TileBasedLayerImplTypes, T: TilingWithCover> TileBasedLayerImpl<B, T>
where
    T::Tile: TileBasedTile,
{
    // Source: cc/layers/tile_based_layer_impl.h:65-67,283-464.
    pub fn AppendQuads<D: TileBasedLayerDelegate<B, T>>(
        &mut self,
        delegate: &D,
        context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        c: &mut D::Context<'_>,
    ) {
        if self.is_backdrop_filter_mask() {
            return;
        }
        if let Some(color) = self.solid_color() {
            self.AppendSolidQuad(delegate, pass, data, color, c);
            return;
        }
        let shared = pass.CreateAndAppendSharedQuadState();
        self.base_.PopulateScaledSharedQuadState(
            pass.SharedQuadStateMutable(shared),
            delegate.GetMaximumContentsScaleForUseInAppendQuads(),
            self.base_.contents_opaque(),
            &c.BaseContext(),
        );
        if delegate.IsDirectlyCompositedImage() {
            let size = self.base_.bounds(&c.BaseContext());
            let mut bounds = MathUtil::MapEnclosingClippedRect(
                &self.base_.draw_properties().target_space_transform,
                Rect::new(0, 0, size.width(), size.height()),
            );
            if self.base_.is_clipped() {
                bounds.Intersect(self.base_.draw_properties().clip_rect);
            }
            if let Some(clip) = pass.SharedQuadState(shared).clip_rect() {
                bounds.Intersect(clip);
            }
            pass.SharedQuadStateMutable(shared)
                .set_clip_rect(Some(bounds));
        }
        let occlusion = self
            .base_
            .draw_properties()
            .occlusion_in_content_space
            .GetOcclusionWithGivenDrawTransform(
                pass.SharedQuadState(shared).quad_to_target_transform(),
            );
        if context.draw_mode == DrawMode::DRAW_MODE_RESOURCELESS_SOFTWARE {
            delegate.AppendQuadsForResourcelessSoftwareDraw(
                self, context, pass, data, shared, &occlusion, c,
            );
            return;
        }
        let mut offset = Vector2d::default();
        if Transform::IsScaleOrTranslation(pass.SharedQuadState(shared).quad_to_target_transform())
        {
            let visible = pass.SharedQuadState(shared).visible_quad_layer_rect();
            offset = Vector2d::new(-visible.x, -visible.y);
        }
        let debug_rect = OffsetRect(pass.SharedQuadState(shared).quad_layer_rect(), offset);
        self.AppendLayerDebugBorderQuad(delegate, pass, debug_rect, shared, data, &c.BaseContext());
        let device_scale = c.BaseContext().tree.device_scale_factor();
        let max_scale = delegate.GetMaximumContentsScaleForUseInAppendQuads();
        let ideal = delegate.GetIdealContentsScaleKey();
        if self
            .base_
            .ShowDebugBorders(DebugBorderType::LAYER, &c.BaseContext())
        {
            let mut iter = delegate.Cover(
                pass.SharedQuadState(shared).visible_quad_layer_rect(),
                max_scale,
                ideal,
            );
            while iter.IsValid() {
                let (color, width) =
                    if let Some(tile) = iter.current_tile().filter(|tile| tile.IsReadyToDraw()) {
                        match tile.draw_mode() {
                            Mode::SOLID_COLOR_MODE => (
                                DebugColors::SolidColorTileBorderColor(),
                                DebugColors::SolidColorTileBorderWidth(device_scale) as f32,
                            ),
                            Mode::OOM_MODE => (
                                DebugColors::OOMTileBorderColor(),
                                DebugColors::OOMTileBorderWidth(device_scale) as f32,
                            ),
                            Mode::RESOURCE_MODE => match delegate
                                .GetTilingResolutionForDebugBorders(iter.CurrentTiling().unwrap())
                            {
                                TilingResolution::kHigh => (
                                    DebugColors::HighResTileBorderColor(),
                                    DebugColors::HighResTileBorderWidth(device_scale) as f32,
                                ),
                                TilingResolution::kAboveHigh => (
                                    DebugColors::AboveHighResTileBorderColor(),
                                    DebugColors::AboveHighResTileBorderWidth(device_scale) as f32,
                                ),
                                TilingResolution::kBelowHigh => (
                                    DebugColors::BelowHighResTileBorderColor(),
                                    DebugColors::BelowHighResTileBorderWidth(device_scale) as f32,
                                ),
                            },
                        }
                    } else {
                        (
                            DebugColors::MissingTileBorderColor(),
                            DebugColors::MissingTileBorderWidth(device_scale) as f32,
                        )
                    };
                let quad = pass.CreateAndAppendDebugBorderDrawQuad();
                let geometry = OffsetRect(iter.geometry_rect(), offset);
                pass.DebugBorderDrawQuadSetNew(quad, shared, geometry, geometry, color, width);
                iter.Advance();
            }
        }
        self.ClearLastAppendQuadsScales();
        delegate.WillAppendQuads(self);
        let viewport = delegate.GetScaledViewportForTilePriority(max_scale);
        let mut missing = 0;
        Self::ForEachVisibleTile(
            delegate,
            pass.SharedQuadState(shared).visible_quad_layer_rect(),
            max_scale,
            ideal,
            &occlusion,
            |iter, visible| {
                let geometry = OffsetRect(iter.geometry_rect(), offset);
                let offset_visible = OffsetRect(visible, offset);
                let blending = !self.base_.contents_opaque();
                data.visible_layer_area += i64::from(visible.width) * i64::from(visible.height);
                if !self.AppendQuadForTile(
                    delegate,
                    iter,
                    context,
                    pass,
                    data,
                    shared,
                    &occlusion,
                    geometry,
                    offset_visible,
                    visible,
                    blending,
                    max_scale,
                    viewport,
                    c,
                ) {
                    missing += 1;
                }
                IterationStatus::kContinue
            },
        );
        if missing != 0 {
            data.num_missing_tiles += missing;
            c.BaseContext().services.CheckerboardTrace(missing);
        }
        let state = pass.SharedQuadStateMutable(shared);
        PropertyTreeTransform::Translate(
            state.quad_to_target_transform_mut(),
            -offset.x as f32,
            -offset.y as f32,
        );
        state.set_quad_layer_rect(OffsetRect(state.quad_layer_rect(), offset));
        state.set_visible_quad_layer_rect(OffsetRect(state.visible_quad_layer_rect(), offset));
        delegate.SanityCheckTilingState(self, &c.BaseContext());
    }
    // Source: cc/layers/tile_based_layer_impl.h:73,466-513.
    pub fn HasMissingTiles<D: TileBasedLayerDelegate<B, T>>(
        &self,
        delegate: &D,
        c: &LayerImplContext<'_, B>,
    ) -> bool {
        if self.is_backdrop_filter_mask() {
            return false;
        }
        if self.solid_color().is_some() {
            return false;
        }
        let max_scale = delegate.GetMaximumContentsScaleForUseInAppendQuads();
        let ideal = delegate.GetIdealContentsScaleKey();
        let bounds = c
            .services
            .ScaleToCeiledSize(self.base_.bounds(c), max_scale);
        let mut visible =
            geometry::ScaleToEnclosingRect(self.base_.visible_layer_rect(), max_scale);
        visible.Intersect(Rect::new(0, 0, bounds.width(), bounds.height()));
        if visible.IsEmpty() {
            return false;
        }
        let occlusion = self
            .base_
            .draw_properties()
            .occlusion_in_content_space
            .GetOcclusionWithGivenDrawTransform(&self.base_.GetScaledDrawTransform(max_scale));
        let viewport = delegate.GetScaledViewportForTilePriority(max_scale);
        let mut missing = false;
        Self::ForEachVisibleTile(
            delegate,
            visible,
            max_scale,
            ideal,
            &occlusion,
            |iter, _visible| {
                if iter.current_tile().is_none_or(|tile| !tile.IsReadyToDraw()) {
                    if delegate.ShouldReportTileAsMissing(iter.geometry_rect(), viewport) {
                        missing = true;
                        return IterationStatus::kStop;
                    }
                }
                IterationStatus::kContinue
            },
        );
        missing
    }
    // Source: cc/layers/tile_based_layer_impl.h:78-80,515-567.
    pub fn GetContentsResourceId(
        &self,
        delegate: &impl TileBasedLayerDelegate<B, T>,
        resource: &mut ResourceId,
        size: &mut Size,
        uv: &mut SizeF,
        c: &LayerImplContext<'_, B>,
    ) {
        *resource = ResourceId::default();
        if !self.is_backdrop_filter_mask() {
            return;
        }
        if !delegate.ValidateTilingSetForContentsResourceId() {
            return;
        }
        let scale = delegate.GetMaximumContentsScaleForUseInAppendQuads();
        let bounds = self.base_.bounds(c);
        let content =
            geometry::ScaleToEnclosingRect(Rect::new(0, 0, bounds.width(), bounds.height()), scale);
        let iter = delegate.Cover(content, scale, delegate.GetIdealContentsScaleKey());
        if !iter.IsValid() || iter.current_tile().is_none() {
            return;
        }
        debug_assert_eq!(iter.geometry_rect(), content);
        let tile = iter.current_tile().unwrap();
        let resource_opt = tile.GetResourceId();
        let size_opt = tile.GetResourceSize();
        let (Some(id), Some(actual_size)) = (resource_opt, size_opt) else {
            return;
        };
        *resource = id;
        *size = actual_size;
        let requested_size = iter
            .CurrentTiling()
            .unwrap()
            .tiling_data()
            .tiling_rect()
            .size();
        let requested = SizeF::new(
            requested_size.width() as f32,
            requested_size.height() as f32,
        );
        debug_assert!(requested.width() <= size.width() as f32);
        debug_assert!(requested.height() <= size.height() as f32);
        *uv = SizeF::new(
            requested.width() / size.width() as f32,
            requested.height() / size.height() as f32,
        );
    }
    // Source: cc/layers/tile_based_layer_impl.h:188-206,569-635.
    fn AppendQuadForTile<D: TileBasedLayerDelegate<B, T>>(
        &mut self,
        delegate: &D,
        iter: &Coverage<'_, B, T>,
        _context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        shared: B::SharedQuadStateHandle,
        _occlusion: &Occlusion<B::Transform>,
        geometry: Rect,
        offset_visible: Rect,
        visible: Rect,
        blending: bool,
        _max_scale: f32,
        viewport: Rect,
        c: &mut D::Context<'_>,
    ) -> bool {
        let tile = iter.current_tile();
        let mut has_quad = false;
        if let Some(tile) = tile.filter(|tile| tile.IsReadyToDraw()) {
            delegate.WillProcessReadyToDrawTile(iter);
            if let Some(resource) = tile.GetResourceId() {
                let quad = pass.CreateAndAppendTileDrawQuad();
                pass.TileDrawQuadSetNew(
                    quad,
                    shared,
                    geometry,
                    offset_visible,
                    blending,
                    resource,
                    iter.texture_rect(),
                    self.GetNearestNeighbor(),
                    !c.BaseContext().tree.settings().enable_edge_anti_aliasing,
                );
                delegate.DidAppendQuad(&self.base_, pass.DrawQuad(quad), iter, data, false, c);
                has_quad = true;
            } else if let Some(color) = tile.GetSolidColor() {
                let alpha = color.alpha * pass.SharedQuadState(shared).opacity();
                if alpha >= f32::EPSILON {
                    let quad = pass.CreateAndAppendSolidColorDrawQuad();
                    pass.SolidColorDrawQuadSetNew(
                        quad,
                        shared,
                        geometry,
                        offset_visible,
                        color,
                        !c.BaseContext().tree.settings().enable_edge_anti_aliasing,
                    );
                    delegate.DidAppendQuad(&self.base_, pass.DrawQuad(quad), iter, data, false, c);
                }
                has_quad = true;
            }
        }
        if !has_quad {
            let mut color = self.base_.safe_opaque_background_color();
            if self
                .base_
                .ShowDebugBorders(DebugBorderType::LAYER, &c.BaseContext())
            {
                color = DebugColors::DefaultCheckerboardColor();
            }
            let quad = pass.CreateAndAppendSolidColorDrawQuad();
            pass.SolidColorDrawQuadSetNew(quad, shared, geometry, offset_visible, color, false);
            delegate.DidAppendQuad(&self.base_, pass.DrawQuad(quad), iter, data, true, c);
            return !delegate.ShouldReportTileAsMissing(iter.geometry_rect(), viewport);
        }
        if delegate.ShouldUpdateApproximatedVisibleContentArea(iter.resolution()) {
            data.approximated_visible_content_area +=
                i64::from(visible.width) * i64::from(visible.height);
        }
        self.produced_tile_last_append_quads_ = true;
        self.AddScaleToLastAppendQuadsScales(iter.CurrentTiling().unwrap().contents_scale_key());
        true
    }
    // Source: cc/layers/layer_impl.cc:329-339. The source inherited virtual
    // call uses the real derived sibling fields; the primitive quad helper is
    // still the unique canonical LayerImpl implementation.
    fn AppendLayerDebugBorderQuad(
        &self,
        delegate: &impl TileBasedLayerDelegate<B, T>,
        pass: &mut B::RenderPass,
        rect: Rect,
        shared: B::SharedQuadStateHandle,
        data: &mut AppendQuadsData<B::SurfaceId>,
        c: &LayerImplContext<'_, B>,
    ) {
        let (color, width) = delegate.GetDebugBorderProperties(&self.base_, c);
        self.base_
            .AppendDebugBorderQuadWithColor(pass, rect, shared, data, color, width, c);
    }
    // Source: cc/layers/tile_based_layer_impl.h:224-227,637-671.
    fn AppendSolidQuad<D: TileBasedLayerDelegate<B, T>>(
        &self,
        delegate: &D,
        pass: &mut B::RenderPass,
        data: &mut AppendQuadsData<B::SurfaceId>,
        color: Color4f,
        c: &mut D::Context<'_>,
    ) {
        let scale = 1.;
        let shared = pass.CreateAndAppendSharedQuadState();
        self.base_.PopulateScaledSharedQuadState(
            pass.SharedQuadStateMutable(shared),
            scale,
            self.base_.contents_opaque(),
            &c.BaseContext(),
        );
        let size = self.base_.bounds(&c.BaseContext());
        self.AppendLayerDebugBorderQuad(
            delegate,
            pass,
            Rect::new(0, 0, size.width(), size.height()),
            shared,
            data,
            &c.BaseContext(),
        );
        let visible = pass.SharedQuadState(shared).visible_quad_layer_rect();
        let occlusion = &self.base_.draw_properties().occlusion_in_content_space;
        let blend = c
            .BaseContext()
            .tree
            .property_trees()
            .effect_tree()
            .Node(self.base_.effect_tree_index())
            .blend_mode;
        super::solid_color_layer_impl::SolidColorLayerImpl::<B>::AppendSolidQuads(
            pass,
            occlusion,
            shared,
            visible,
            color,
            !c.BaseContext().tree.settings().enable_edge_anti_aliasing,
            blend,
            data,
        );
    }
}
impl<B: TileBasedLayerImplTypes, T: TilingWithCover> TileBasedLayerImpl<B, T> {
    // Source: cc/layers/tile_based_layer_impl.h:104,673-690.
    pub fn CalculateCullRectInLayerSpace(&self, c: &LayerImplContext<'_, B>) -> Option<Rect> {
        let trees = c.tree.property_trees();
        let scroll = trees.scroll_tree();
        if self.base_.scroll_tree_index() != kInvalidPropertyNodeId {
            let node = scroll.Node(self.base_.scroll_tree_index());
            if self.base_.transform_tree_index() == node.transform_id {
                if let Some(rect) = scroll.ScrollingContentsCullRect(node.element_id) {
                    let mut rect = RectF::new(
                        rect.x as f32,
                        rect.y as f32,
                        rect.width as f32,
                        rect.height as f32,
                    );
                    let offset = self.base_.offset_to_transform_parent();
                    rect.Offset(-offset.x(), -offset.y());
                    return Some(geometry::ToEnclosingRect(rect));
                }
            }
        }
        None
    }
}
// Source: ui/gfx/geometry/rect.cc:125-130; ui/gfx/geometry/rect.h:159-164.
// Native gfx::Rect::Offset primitive: clamp coordinates through the same Rect
// constructor. This helper owns only the source local by-value rectangle.
fn OffsetRect(rect: Rect, offset: Vector2d) -> Rect {
    Rect::new(
        rect.x.saturating_add(offset.x),
        rect.y.saturating_add(offset.y),
        rect.width,
        rect.height,
    )
}

// Native Tile supplies the actual resident values to the Tiling concept.
impl<B> TileBasedTile for crate::cc::tiles::tile::Tile<B>
where
    B: crate::adapter::tile::TileTypes<ResourceId = crate::adapter::layer_tree_impl::ResourceId>,
{
    fn draw_mode(&self) -> Mode {
        self.draw_mode()
    }
    fn GetResourceId(&self) -> Option<ResourceId> {
        self.GetResourceId()
    }
    fn GetResourceSize(&self) -> Option<Size> {
        self.GetResourceSize()
    }
    fn GetSolidColor(&self) -> Option<Color4f> {
        self.GetSolidColor()
    }
}
