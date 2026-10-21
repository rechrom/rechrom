// Copyright 2012 The Chromium Authors. BSD-style license.
//! CPU translation of components/viz/service/display/direct_renderer.{h,cc}.
//! Root identity is supplied by VizOutput::RootDestination: each supplied root
//! is a host composition plane, rather than assuming only the final pass is root.
#![allow(non_snake_case)]
use super::bsp_walk_action::{BspWalkAction, BspWalkActionDrawPolygon, PolygonDrawBackend};
use super::{
    bsp_tree::BspTree, draw_polygon::DrawPolygon, render_pass_alpha_type::RenderPassAlphaType,
};
use crate::components::viz::common::display::renderer_settings::{
    DebugRendererSettings, RendererSettings,
};
use compositor_core::{
    adapter::geometry::{PointF, Rect, RectF, Size},
    cc::base::math_util::MathUtil,
    runtime::frame::{QuadMaterial, RenderPassId},
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
};
use viz::{
    common::quads::aggregated_render_pass::{NativeDrawQuad, NativeSharedQuadState},
    native::{AggregatedRenderPass, VizOutput},
};

// Source: components/viz/service/display/direct_renderer.h:211-223,276-324.
#[derive(Clone, Debug)]
pub struct RenderPassRequirements {
    pub size: Size,
    pub generate_mipmap: bool,
    pub alpha_type: RenderPassAlphaType,
    pub is_scanout: bool,
}
pub trait DirectRendererBackend {
    /// Embedding adapter: the target's concrete storage belongs to the backend.
    /// Borrowed targets do not escape Render and require no pixel snapshot.
    type Target<'a>;
    fn Configure(&mut self, settings: &RendererSettings);
    fn ValidateTarget(&self, input: &VizOutput, target: &Self::Target<'_>) -> io::Result<Size>;
    fn PublishFrame(
        &mut self,
        input: &VizOutput,
        target: Self::Target<'_>,
    ) -> io::Result<crate::RenderUpdate>;

    fn CanPartialSwap(&self) -> bool;
    fn UpdateRenderPassTextures(
        &mut self,
        input: &VizOutput,
        r: &BTreeMap<RenderPassId, RenderPassRequirements>,
    ) -> io::Result<()>;
    fn AllocateRenderPassResourceIfNeeded(
        &mut self,
        id: RenderPassId,
        r: &RenderPassRequirements,
    ) -> io::Result<()>;
    fn IsRenderPassResourceAllocated(&self, id: RenderPassId) -> bool;
    fn GetRenderPassBackingPixelSize(&self, id: RenderPassId) -> Size;
    fn GetRenderPassBackingDrawnRect(&self, id: RenderPassId) -> Rect;
    fn SetRenderPassBackingDrawnRect(&mut self, id: RenderPassId, rect: Rect);
    fn BeginDrawingFrame(&mut self, input: &VizOutput) -> io::Result<()>;
    fn BeginDrawingRenderPass(
        &mut self,
        pass: &AggregatedRenderPass,
        needs_clear: bool,
        update: Rect,
        viewport: Size,
    ) -> io::Result<()>;
    fn SetScissorTestRect(&mut self, rect: Rect);
    fn EnsureScissorTestDisabled(&mut self);
    fn DoDrawQuad(
        &mut self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        index: usize,
        clip: Option<&[PointF; 4]>,
    ) -> io::Result<()>;
    fn FinishDrawingRenderPass(&mut self) -> io::Result<()>;
    fn FinishDrawingFrame(&mut self) -> io::Result<()>;
    // Rust failure cleanup: invalidate partially drawn backings; C++ has no
    // fallible draw return path corresponding to this backend error boundary.
    fn AbortDrawingFrame(&mut self);
    fn GetCurrentFramebufferDamage(&self, _id: RenderPassId) -> Rect {
        Rect::default()
    }
    fn DidChangeVisibility(&mut self, _visible: bool) {}
}
// Source: components/viz/service/display/direct_renderer.h:136-152,351-451.
// DrawingFrame borrows the canonical aggregate; no pass/quad snapshot is retained.
pub struct DrawingFrame<'a> {
    pub input: &'a VizOutput,
    pub device_viewport_size: Size,
    pub root_damage_rect: Rect,
}
pub struct DirectRenderer<B> {
    pub backend: B,
    pub settings: RendererSettings,
    pub debug_settings: DebugRendererSettings,
    pub use_partial_swap: bool,
    pub allow_undamaged_nonroot_render_pass_to_skip: bool,
    pub rpdq_filter_lookup_optimizations: bool,
    pub buffer_queue_per_render_pass: bool,
    pub max_texture_size: i32,
    pub supports_viewporter: bool,
    device_viewport_size: Size,
    last_viewport_resize_time: Option<std::time::Instant>,
    backdrop_filter_output_rects: BTreeMap<RenderPassId, Rect>,
    has_pixel_moving_foreground_filters: bool,
    pub skipped_render_pass_ids: BTreeSet<RenderPassId>,
    pub last_root_render_pass_scissor_rect: Rect,
    visible: bool,
    initialized: bool,
    next_frame_needs_full_frame_redraw: bool,
    reshape_size: Option<Size>,
    output_surface_clip_rect: Option<Rect>,
    enlarge_pass_texture_amount: Size,
    pub total_pixels_rendered_this_frame: u64,
}
impl<B: DirectRendererBackend> DirectRenderer<B> {
    // Source: components/viz/service/display/direct_renderer.cc:86-106.
    pub fn new(
        settings: RendererSettings,
        debug_settings: DebugRendererSettings,
        backend: B,
    ) -> Self {
        Self {
            backend,
            settings,
            debug_settings,
            use_partial_swap: false,
            allow_undamaged_nonroot_render_pass_to_skip: true,
            rpdq_filter_lookup_optimizations: true,
            buffer_queue_per_render_pass: false,
            max_texture_size: 0,
            supports_viewporter: false,
            device_viewport_size: Size::default(),
            last_viewport_resize_time: None,
            backdrop_filter_output_rects: BTreeMap::new(),
            has_pixel_moving_foreground_filters: false,
            skipped_render_pass_ids: BTreeSet::new(),
            last_root_render_pass_scissor_rect: Rect::default(),
            visible: false,
            initialized: false,
            next_frame_needs_full_frame_redraw: false,
            reshape_size: None,
            output_surface_clip_rect: None,
            enlarge_pass_texture_amount: Size::default(),
            total_pixels_rendered_this_frame: 0,
        }
    }
    pub fn Initialize(&mut self) {
        self.use_partial_swap = self.settings.partial_swap_enabled && self.backend.CanPartialSwap();
        self.initialized = true;
    }
    // Source: components/viz/service/display/direct_renderer.cc:137-151.
    pub fn SetOutputSurfaceClipRect(&mut self, rect: Rect) {
        self.output_surface_clip_rect = Some(rect)
    }
    pub fn SetVisible(&mut self, visible: bool) {
        assert!(self.initialized);
        if self.visible == visible {
            return;
        }
        self.visible = visible;
        self.backend.DidChangeVisibility(visible)
    }
    pub fn ReallocatedFrameBuffers(&mut self) {
        self.next_frame_needs_full_frame_redraw = true
    }
    // Source: components/viz/service/display/direct_renderer.cc:158-198.
    // CPU backend has no pass bypass (base CanPassBeDrawnDirectly returns nullopt).
    pub fn DecideRenderPassAllocationsForFrame(
        &mut self,
        input: &VizOutput,
        skip_root: bool,
    ) -> io::Result<()> {
        let mut requirements = BTreeMap::new();
        for pass in input.Frame().RenderPasses() {
            let root = input.RootDestination(pass.RenderPassId()).is_some();
            if root && skip_root {
                continue;
            }
            requirements.insert(
                pass.RenderPassId(),
                self.CalculateRenderPassRequirements(pass, root),
            );
        }
        self.backend.UpdateRenderPassTextures(input, &requirements)
    }
    // Source: components/viz/service/display/direct_renderer.cc:200-217,233-238,263-291,357-376,391-395,410-449,472-489.
    // CPU output is fixed-size; overlay/ink/platform reshape branches are excluded.
    pub fn DrawFrame(&mut self, input: &VizOutput, viewport: Size) -> io::Result<()> {
        let result = self.DrawFrameInternal(input, viewport);
        self.backdrop_filter_output_rects.clear();
        self.has_pixel_moving_foreground_filters = false;
        if result.is_err() {
            self.next_frame_needs_full_frame_redraw = true;
            self.backend.AbortDrawingFrame();
        }
        result
    }
    fn DrawFrameInternal(&mut self, input: &VizOutput, viewport: Size) -> io::Result<()> {
        assert!(self.visible);
        let surface_size = self.CalculateSizeForOutputSurface(viewport);
        let reshape =
            self.reshape_size != Some(surface_size) || self.next_frame_needs_full_frame_redraw;
        self.reshape_size = Some(surface_size);
        self.next_frame_needs_full_frame_redraw = false;
        self.skipped_render_pass_ids.clear();
        self.total_pixels_rendered_this_frame = 0;
        self.backend.BeginDrawingFrame(input)?;
        // Source: components/viz/service/display/direct_renderer.cc:265-291.
        if !self.rpdq_filter_lookup_optimizations {
            let mut candidates = BTreeMap::new();
            for pass in input.Frame().RenderPasses() {
                candidates.insert(
                    pass.RenderPassId(),
                    MathUtil::MapEnclosingClippedRect(
                        &pass.transform_to_root_target,
                        pass.output_rect,
                    ),
                );
                for quad in &pass.quad_list {
                    if let QuadMaterial::CompositorRenderPass { render_pass_id, .. } =
                        &quad.material
                    {
                        let child = input
                            .Frame()
                            .RenderPasses()
                            .iter()
                            .find(|p| p.RenderPassId() == *render_pass_id)
                            .ok_or_else(|| io::Error::other("missing aggregate child pass"))?;
                        if child.filters.HasFilterThatMovesPixels() {
                            self.has_pixel_moving_foreground_filters = true;
                        }
                        if child.backdrop_filters.HasFilterThatMovesPixels() {
                            self.backdrop_filter_output_rects.insert(
                                *render_pass_id,
                                *candidates.get(render_pass_id).ok_or_else(|| {
                                    io::Error::other("aggregate child does not precede parent")
                                })?,
                            );
                        }
                    }
                }
            }
        }
        self.DecideRenderPassAllocationsForFrame(input, false)?;
        let frame = DrawingFrame {
            input,
            device_viewport_size: viewport,
            root_damage_rect: Rect::default(),
        };
        // Preserve aggregate dependency order. Each host root uses the source root
        // branch at its own position; roots are not moved behind later child passes.
        for pass in input.Frame().RenderPasses() {
            let root = input.RootDestination(pass.RenderPassId()).is_some();
            if !root {
                self.DrawRenderPassAndExecuteCopyRequests(&frame, pass, false, pass.damage_rect)?;
                continue;
            }
            let mut damage = pass.damage_rect;
            let surface = pass.output_rect;
            damage.Intersect(surface);
            // A CPU root plane can acquire a fresh bitmap without changing
            // the host viewport. Adapt OutputSurface reallocation semantics:
            // include backend framebuffer invalidation before the root skip.
            let framebuffer_damage = self
                .backend
                .GetCurrentFramebufferDamage(pass.RenderPassId());
            let backing_invalid = self
                .backend
                .GetRenderPassBackingDrawnRect(pass.RenderPassId())
                != pass.output_rect;
            let skip = damage.IsEmpty()
                && framebuffer_damage.IsEmpty()
                && !backing_invalid
                && self.use_partial_swap
                && !reshape;
            let full = reshape || backing_invalid || (!self.use_partial_swap && !skip);
            if full {
                damage = surface
            }
            if !skip {
                self.DrawRenderPassAndExecuteCopyRequests(&frame, pass, true, damage)?
            }
        }

        self.backend.FinishDrawingFrame()?;
        self.backdrop_filter_output_rects.clear();
        self.has_pixel_moving_foreground_filters = false;
        Ok(())
    }
    // Source: components/viz/service/display/direct_renderer.cc:123-129,517-564.
    pub fn MoveFromDrawToWindowSpace(pass: &AggregatedRenderPass, r: Rect) -> Rect {
        Rect::new(
            r.x.saturating_sub(pass.output_rect.x),
            r.y.saturating_sub(pass.output_rect.y),
            r.width,
            r.height,
        )
    }
    pub fn ShouldSkipQuad(
        &self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        quad: &NativeDrawQuad,
        scissor: Rect,
    ) -> bool {
        if scissor.IsEmpty() {
            return true;
        }
        let shared = shared(pass, quad);
        let mut target = expanded_rect(input, quad);
        target = MathUtil::MapEnclosingClippedRect(&shared.quad_to_target_transform, target);
        if let Some(clip) = shared.clip_rect {
            target.Intersect(clip)
        }
        target.Intersect(scissor);
        target.IsEmpty()
    }
    pub fn SetScissorStateForQuad(
        &mut self,
        pass: &AggregatedRenderPass,
        quad: &NativeDrawQuad,
        scissor: Rect,
        use_scissor: bool,
    ) {
        let clip = shared(pass, quad).clip_rect;
        if use_scissor {
            let mut r = scissor;
            if let Some(clip) = clip {
                r.Intersect(clip)
            }
            self.backend
                .SetScissorTestRect(Self::MoveFromDrawToWindowSpace(pass, r));
            return;
        } else if let Some(clip) = clip {
            self.backend
                .SetScissorTestRect(Self::MoveFromDrawToWindowSpace(pass, clip));
            return;
        }
        self.backend.EnsureScissorTestDisabled()
    }
    // Source: components/viz/service/display/direct_renderer.cc:566-584,591-604.
    fn FlushPolygons(
        &mut self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        list: &mut VecDeque<DrawPolygon>,
        scissor: Rect,
        clipped: bool,
    ) -> io::Result<()> {
        if list.is_empty() {
            return Ok(());
        }
        let mut tree = BspTree::new(list);
        let mut adapter = PolygonRenderer {
            renderer: self,
            input,
            pass,
            result: Ok(()),
        };
        let mut action = BspWalkActionDrawPolygon::new(&mut adapter, &scissor, clipped);
        tree.traverse_with_action_handler(&mut |poly| action.call(poly));
        adapter.result
    }
    // Source: components/viz/service/display/direct_renderer.cc:566-584.
    fn DoDrawPolygon(
        &mut self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        poly: &DrawPolygon,
        scissor: Rect,
        clipped: bool,
    ) -> io::Result<()> {
        let index = poly.original_ref;
        self.SetScissorStateForQuad(pass, &pass.quad_list[index], scissor, clipped);
        if !poly.is_split {
            return self.backend.DoDrawQuad(input, pass, index, None);
        }
        let mut quads = Vec::new();
        poly.to_quads_2d(&mut quads);
        for quad in quads {
            self.backend
                .DoDrawQuad(input, pass, index, Some(&quad.points))?;
        }
        Ok(())
    }

    // Source: components/viz/service/display/direct_renderer.cc:606-622.
    // cc623-702 copy requests cannot enter VizOutput (no copy capability).
    fn DrawRenderPassAndExecuteCopyRequests(
        &mut self,
        frame: &DrawingFrame<'_>,
        pass: &AggregatedRenderPass,
        root: bool,
        damage: Rect,
    ) -> io::Result<()> {
        for _ in 0..self.settings.slow_down_compositing_scale_factor {
            self.DrawRenderPass(frame, pass, root, damage)?
        }
        Ok(())
    }
    // Source: components/viz/service/display/direct_renderer.cc:740-874.
    fn DrawRenderPass(
        &mut self,
        frame: &DrawingFrame<'_>,
        pass: &AggregatedRenderPass,
        root: bool,
        damage: Rect,
    ) -> io::Result<()> {
        let mut trace = browser_tracing::span("renderer", "Renderer.DrawRenderPass");
        trace.set("root", root as u8 as f64);
        trace.set("quads", pass.Quads().len() as f64);
        trace.set("width", pass.output_rect.width as f64);
        trace.set("height", pass.output_rect.height as f64);
        if self.CanSkipRenderPass(pass, root) {
            self.skipped_render_pass_ids.insert(pass.RenderPassId());
            return Ok(());
        }
        let surface = pass.output_rect;
        let mut scissor = surface;
        if self.use_partial_swap {
            scissor.Intersect(self.ComputeScissorRectForRenderPass(frame.input, pass, root, damage))
        }
        if root {
            if let Some(clip) = self.output_surface_clip_rect {
                scissor.Intersect(clip)
            }
        }
        if !root && scissor.IsEmpty() {
            return Ok(());
        }
        self.EnsureRenderPassAllocated(pass, root)?;
        if !root
            && !self
                .backend
                .IsRenderPassResourceAllocated(pass.RenderPassId())
        {
            return Ok(());
        }
        self.total_pixels_rendered_this_frame +=
            scissor.width.max(0) as u64 * scissor.height.max(0) as u64;
        trace.set(
            "damage_pixels",
            scissor.width.max(0) as f64 * scissor.height.max(0) as f64,
        );
        let clipped = !scissor.Contains(surface);
        let clear = !root || self.settings.should_clear_root_render_pass;
        let update = Self::MoveFromDrawToWindowSpace(pass, if clipped { scissor } else { surface });
        let viewport = Size::new(surface.width, surface.height);
        self.backend
            .BeginDrawingRenderPass(pass, clear, update, viewport)?;
        if root {
            self.last_root_render_pass_scissor_rect = scissor
        }
        let mut polygons = VecDeque::new();
        let mut next_polygon_id = 0;
        let mut last_sorting_context_id = 0;
        for index in (0..pass.quad_list.len()).rev() {
            let quad = &pass.quad_list[index];
            if self.ShouldSkipQuad(
                frame.input,
                pass,
                quad,
                if clipped { scissor } else { surface },
            ) {
                continue;
            }
            let shared = shared(pass, quad);
            if last_sorting_context_id != shared.sorting_context_id {
                last_sorting_context_id = shared.sorting_context_id;
                self.FlushPolygons(frame.input, pass, &mut polygons, scissor, clipped)?
            }
            if shared.sorting_context_id != 0 {
                let poly = DrawPolygon::new(
                    index,
                    RectF::from(quad.geometry.visible_rect),
                    &shared.quad_to_target_transform,
                    next_polygon_id,
                );
                next_polygon_id += 1;
                let normal = poly.normal;
                let length_squared = (normal.x as f64) * (normal.x as f64)
                    + (normal.y as f64) * (normal.y as f64)
                    + (normal.z as f64) * (normal.z as f64);
                if length_squared > 0. {
                    polygons.push_back(poly)
                }
                continue;
            }
            self.SetScissorStateForQuad(pass, quad, scissor, clipped);
            self.backend.DoDrawQuad(frame.input, pass, index, None)?
        }
        self.FlushPolygons(frame.input, pass, &mut polygons, scissor, clipped)?;
        self.backend.FinishDrawingRenderPass()?;
        if !root
            && self
                .backend
                .GetRenderPassBackingDrawnRect(pass.RenderPassId())
                != pass.output_rect
        {
            debug_assert_eq!(scissor, pass.output_rect);
            self.backend
                .SetRenderPassBackingDrawnRect(pass.RenderPassId(), pass.output_rect)
        }
        Ok(())
    }
    // Source: components/viz/service/display/direct_renderer.cc:876-897.
    pub fn CanSkipRenderPass(&self, pass: &AggregatedRenderPass, root: bool) -> bool {
        if root {
            return false;
        }
        if pass.cache_render_pass || self.allow_undamaged_nonroot_render_pass_to_skip {
            if pass.has_damage_from_contributing_content {
                return false;
            }
            return self
                .backend
                .IsRenderPassResourceAllocated(pass.RenderPassId());
        }
        false
    }
    // Source: components/viz/service/display/direct_renderer.cc:899-958,975-990.
    pub fn CalculateRenderPassRequirements(
        &self,
        pass: &AggregatedRenderPass,
        root: bool,
    ) -> RenderPassRequirements {
        if root {
            assert!(!pass.generate_mipmap);
        }
        RenderPassRequirements {
            size: if root {
                Size::new(pass.output_rect.width, pass.output_rect.height)
            } else {
                self.CalculateTextureSizeForRenderPass(pass)
            },
            generate_mipmap: !root && pass.generate_mipmap,
            alpha_type: if root && !pass.has_transparent_background {
                RenderPassAlphaType::kOpaque
            } else {
                RenderPassAlphaType::kPremul
            },
            is_scanout: root,
        }
    }
    fn EnsureRenderPassAllocated(
        &mut self,
        pass: &AggregatedRenderPass,
        root: bool,
    ) -> io::Result<()> {
        let mut r = self.CalculateRenderPassRequirements(pass, root);
        if !root {
            r.size = Size::new(
                r.size
                    .width()
                    .saturating_add(self.enlarge_pass_texture_amount.width()),
                r.size
                    .height()
                    .saturating_add(self.enlarge_pass_texture_amount.height()),
            )
        }
        self.backend
            .AllocateRenderPassResourceIfNeeded(pass.RenderPassId(), &r)
    }
    // Source: components/viz/service/display/direct_renderer.cc:992-1074.
    // The aggregate owns child filters; RPDQ child IDs resolve that canonical
    // value rather than duplicating filter state onto the renderer's quads.
    fn ExpandDamageForPixelMovingFilters(
        &self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        damage: &mut Rect,
    ) {
        if damage.IsEmpty() {
            return;
        }
        if !self.rpdq_filter_lookup_optimizations
            && self.backdrop_filter_output_rects.is_empty()
            && !self.has_pixel_moving_foreground_filters
        {
            return;
        }
        for quad in &pass.quad_list {
            let QuadMaterial::CompositorRenderPass { render_pass_id, .. } = &quad.material else {
                continue;
            };
            let Some(child) = input
                .Frame()
                .RenderPasses()
                .iter()
                .find(|p| p.RenderPassId() == *render_pass_id)
            else {
                continue;
            };
            let shared = shared(pass, quad);
            if self.rpdq_filter_lookup_optimizations
                && !child.backdrop_filters.IsEmpty()
                && child.backdrop_filters.HasFilterThatMovesPixels()
            {
                let r = MathUtil::MapEnclosingClippedRect(
                    &shared.quad_to_target_transform,
                    quad.geometry.rect,
                );
                if damage.Intersects(r) {
                    damage.Union(r)
                }
            }
            if !self.rpdq_filter_lookup_optimizations {
                if let Some(r) = self.backdrop_filter_output_rects.get(render_pass_id) {
                    if damage.Intersects(*r) {
                        damage.Union(*r);
                    }
                }
            }
            if child.filters.HasFilterThatMovesPixels() {
                let mut r = MathUtil::MapEnclosingClippedRect(
                    &shared.quad_to_target_transform,
                    expanded_rect(input, quad),
                );
                if let Some(clip) = shared.clip_rect {
                    r.Intersect(clip)
                }
                if damage.Intersects(r) {
                    damage.Union(r)
                }
            }
        }
    }
    // Source: components/viz/service/display/direct_renderer.cc:1076-1174.
    fn ComputeScissorRectForRenderPass(
        &self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        root: bool,
        damage: Rect,
    ) -> Rect {
        if root {
            let mut damage = damage;
            damage.Union(
                self.backend
                    .GetCurrentFramebufferDamage(pass.RenderPassId()),
            );
            self.ExpandDamageForPixelMovingFilters(input, pass, &mut damage);
            return damage;
        }
        if self
            .backend
            .GetRenderPassBackingDrawnRect(pass.RenderPassId())
            == pass.output_rect
        {
            if self.buffer_queue_per_render_pass {
                let mut damage = pass.damage_rect;
                damage.Union(
                    self.backend
                        .GetCurrentFramebufferDamage(pass.RenderPassId()),
                );
                self.ExpandDamageForPixelMovingFilters(input, pass, &mut damage);
                damage.Intersect(pass.output_rect);
                return damage;
            }
            pass.damage_rect
        } else {
            pass.output_rect
        }
    }
    // Source: components/viz/service/display/direct_renderer.cc:1176-1202.
    pub fn CalculateTextureSizeForRenderPass(&self, pass: &AggregatedRenderPass) -> Size {
        let mut width = pass.output_rect.width;
        let mut height = pass.output_rect.height;
        if !self.settings.dont_round_texture_sizes_for_pixel_tests {
            width = round_up(width, 64);
            height = round_up(height, 64);
            if self.max_texture_size > 0 {
                width = width.min(self.max_texture_size);
                height = height.min(self.max_texture_size);
            }
        }
        Size::new(width, height)
    }
    // Source: components/viz/service/display/direct_renderer.cc:1204-1267.
    pub fn CalculateSizeForOutputSurface(&mut self, requested: Size) -> Size {
        let surface = self.reshape_size.unwrap_or_default();
        if requested == surface
            || !self.supports_viewporter
            || surface.width() == 0 && surface.height() == 0
            || self.settings.dont_round_texture_sizes_for_pixel_tests
        {
            self.device_viewport_size = requested;
            return requested;
        }
        if self.device_viewport_size == requested
            && self
                .last_viewport_resize_time
                .is_some_and(|time| time.elapsed() >= std::time::Duration::from_secs(1))
        {
            return requested;
        }
        let mut width = surface.width();
        let mut height = surface.height();
        const MULTIPLE: i32 = 256;
        if requested.width() > width {
            width = round_up(requested.width().saturating_add(MULTIPLE - 1), MULTIPLE);
        }
        if requested.height() > height {
            height = round_up(requested.height().saturating_add(MULTIPLE - 1), MULTIPLE);
        }
        if requested != self.device_viewport_size {
            self.last_viewport_resize_time = Some(std::time::Instant::now());
        }
        if width > self.max_texture_size {
            width = self.max_texture_size;
        }
        if height > self.max_texture_size {
            height = self.max_texture_size;
        }
        self.device_viewport_size = requested;
        Size::new(width, height)
    }
    // Source: components/viz/service/display/direct_renderer.cc:497-499.
    pub fn GetTargetDamageBoundingRect(&self) -> Rect {
        Rect::default()
    }
    // Source: components/viz/service/display/direct_renderer.cc:1274-1277.
    pub fn HasAllocatedResourcesForTesting(&self, id: RenderPassId) -> bool {
        self.backend.IsRenderPassResourceAllocated(id)
    }
    pub fn SetEnlargePassTextureAmountForTesting(&mut self, amount: Size) {
        self.enlarge_pass_texture_amount = amount
    }
}
// Source: components/viz/service/display/direct_renderer.cc:1279-1285.
// C++ static geometry helper; no access to the manager or backend storage.
pub fn ShouldApplyRoundedCorner(quad: &NativeDrawQuad, shared: &NativeSharedQuadState) -> bool {
    let target = MathUtil::MapClippedRect(
        &shared.quad_to_target_transform,
        RectF::from(quad.geometry.visible_rect),
    );
    crate::components::viz::common::viz_utils::QuadRoundedCornersBoundsIntersects(shared, target)
}
fn shared<'a>(pass: &'a AggregatedRenderPass, quad: &NativeDrawQuad) -> &'a NativeSharedQuadState {
    pass.TrySharedQuadState(
        quad.geometry
            .shared_quad_state
            .expect("validated shared state"),
    )
    .expect("validated shared state")
}
fn expanded_rect(input: &VizOutput, quad: &NativeDrawQuad) -> Rect {
    let QuadMaterial::CompositorRenderPass {
        render_pass_id,
        filters_scale,
        filters_origin,
        ..
    } = &quad.material
    else {
        return quad.geometry.visible_rect;
    };
    let Some(pass) = input
        .Frame()
        .RenderPasses()
        .iter()
        .find(|p| p.RenderPassId() == *render_pass_id)
    else {
        return quad.geometry.visible_rect;
    };
    if !pass.filters.HasFilterThatMovesPixels() {
        return quad.geometry.visible_rect;
    }
    let r = quad.geometry.visible_rect;
    let r = pass.filters.MapRectWithScaleAndOrigin(
        foundation::gfx::Rect::new(
            foundation::gfx::Point::new(r.x, r.y),
            foundation::gfx::Size::new(r.width, r.height),
        ),
        foundation::gfx::Vector2dF::new(filters_scale.x(), filters_scale.y()),
        foundation::gfx::PointF::new(filters_origin.x(), filters_origin.y()),
    );
    Rect::new(r.x(), r.y(), r.width(), r.height())
}

// Private adapter for the translated BspWalkActionDrawPolygon. It resolves
// handles against borrowed canonical passes and carries backend failures out of
// the source visitor's void callback without swallowing them.
struct PolygonRenderer<'a, B> {
    renderer: &'a mut DirectRenderer<B>,
    input: &'a VizOutput,
    pass: &'a AggregatedRenderPass,
    result: io::Result<()>,
}
impl<B: DirectRendererBackend> PolygonDrawBackend for PolygonRenderer<'_, B> {
    fn polygon_transform(&self, index: usize) -> &viz::native::NativeTransform {
        &shared(self.pass, &self.pass.quad_list[index]).quad_to_target_transform
    }
    fn do_draw_polygon(&mut self, item: &DrawPolygon, scissor: Rect, clipped: bool) {
        if self.result.is_ok() {
            self.result = self
                .renderer
                .DoDrawPolygon(self.input, self.pass, item, scissor, clipped);
        }
    }
}

fn round_up(value: i32, multiple: i32) -> i32 {
    // CheckedRoundUp's source checked arithmetic rejects overflow.
    value
        .checked_add(multiple - 1)
        .and_then(|v| (v / multiple).checked_mul(multiple))
        .expect("render pass dimension overflow")
}

// SOURCE_EXCLUDED: direct_renderer.cc:1-85: C++ includes, UMA enum and default
// value constructors; native Rust values/imports express ownership instead.
// SOURCE_EXCLUDED: direct_renderer.cc:112-121: target/device AxisTransform2d
// projection is supplied by SoftwareRenderer::BeginDrawingRenderPass canvas.
// SOURCE_EXCLUDED: direct_renderer.cc:153-156,240-260,299-356,377-390,397-409,
// 468-471,475-476: OutputSurface reshape, overlay/GPU/platform composition.
// SOURCE_EXCLUDED: direct_renderer.cc:218-231,704-738,1309-1311,1319-1331:
// delegated ink and Ozone overlay testing require external platform owners.
// SOURCE_EXCLUDED: direct_renderer.cc:235,293-298,362-369,933-938,944-946,
// 960-973,1287-1289,1298-1307: display color-space/HDR/output format policy;
// VizOutput and CPU RenderTarget carry no display-color-space capability.
// SOURCE_EXCLUDED: direct_renderer.cc:623-702: CopyOutputRequest geometry,
// readback, tracked element mapping. Canonical aggregate exposes no requests.
// SOURCE_EXCLUDED: direct_renderer.cc:1291-1296: gradient mask query belongs to
// GPU renderer callers; source SoftwareRenderer only applies rounded corners.
// SOURCE_EXCLUDED: direct_renderer.cc:1269-1272,1313-1317: raw pointer testing
// frame mutation and optional trace hooks; frame lifetime is statically borrowed.
// ADAPTED: direct_renderer.cc:210-217,422-448,480-489: multiple root planes are
// selected by VizOutput::RootDestination, canonical pass list stays borrowed,
// per-pass roots retain dependency order and frame destruction drops borrows.
// ADAPTED: direct_renderer.cc:899-958,975-990: host root planes use CPU backing
// resources, so renderer allocation applies to root as well as nonroot passes.
// Root scanout/color fields describe source requirements; CPU never scans out.
