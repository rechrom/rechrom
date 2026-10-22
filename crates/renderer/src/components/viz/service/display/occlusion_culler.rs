// Source: components/viz/service/display/occlusion_culler.h:1-50
// Source: components/viz/service/display/occlusion_culler.cc:1-569
//! Display-stage overdraw removal performed after surface aggregation.
//!
//! Overlay-specific exclusions and rounded-corner tessellation stay conservative:
//! those groups do not contribute occlusion until their corresponding display
//! capabilities exist. Fully and partially occluded quads follow Chromium's
//! target-space/content-space algorithm.
#![allow(non_snake_case)]

use crate::components::viz::common::display::renderer_settings::OcclusionCullerSettings;
use compositor_core::{
    adapter::{
        geometry::{Rect, Transform as _},
        occlusion::OcclusionTransform as _,
        property_types::PropertyTransform as _,
    },
    cc::base::{math_util::MathUtil, region::Region},
    runtime::{frame::QuadMaterial, skia::SkRegion},
};
use std::collections::BTreeSet;
use viz::{
    common::quads::aggregated_render_pass::{
        NativeDrawQuad, NativeSharedQuadState, NativeTransform,
    },
    native::AggregatedFrame,
};

type NativeRegion = Region<SkRegion>;

#[derive(Clone, Copy, Debug, Default)]
pub struct OcclusionCullStats {
    pub quads_before: usize,
    pub quads_after: usize,
    pub fully_culled: usize,
    pub reduced: usize,
    pub split: usize,
    pub opaque_groups: usize,
}

pub struct OcclusionCuller {
    settings: OcclusionCullerSettings,
    device_scale_factor: f32,
}

impl OcclusionCuller {
    pub fn new(settings: OcclusionCullerSettings) -> Self {
        Self {
            settings,
            device_scale_factor: 1.0,
        }
    }

    fn CanContributeToOcclusion(sqs: &NativeSharedQuadState, is_render_pass: bool) -> bool {
        !is_render_pass
            && sqs.opacity == 1.0
            && sqs.are_contents_opaque
            && matches!(
                sqs.blend_mode,
                compositor_core::adapter::occlusion::SkBlendMode::kSrcOver
                    | compositor_core::adapter::occlusion::SkBlendMode::kSrc
            )
            && sqs
                .quad_to_target_transform
                .NonDegeneratePreserves2dAxisAlignment()
            // Chromium computes an inscribed occluder for rounded corners. Until
            // that exact RRect helper is owned here, refusing the whole group is
            // the conservative equivalent.
            && !sqs.mask_filter_info.HasRoundedCorners()
    }

    fn IsRenderPass(quad: &NativeDrawQuad) -> bool {
        matches!(quad.material, QuadMaterial::CompositorRenderPass { .. })
    }

    fn IsSimpleInvertible(transform: &NativeTransform) -> bool {
        transform.NonDegeneratePreserves2dAxisAlignment() && transform.IsInvertible()
    }

    fn RegionArea(region: &NativeRegion) -> i64 {
        region
            .into_iter()
            .map(|rect| i64::from(rect.width.max(0)) * i64::from(rect.height.max(0)))
            .sum()
    }

    fn AddOccluder(
        &self,
        sqs: &NativeSharedQuadState,
        is_render_pass: bool,
        occlusion: &mut NativeRegion,
    ) -> bool {
        if !Self::CanContributeToOcclusion(sqs, is_render_pass) {
            return false;
        }
        let mut region = NativeRegion::new(MathUtil::MapEnclosedRectWith2dAxisAlignedTransform(
            &sqs.quad_to_target_transform,
            sqs.visible_quad_layer_rect,
        ));
        if let Some(clip) = sqs.clip_rect {
            region.Intersect(&clip);
        }
        if Self::RegionArea(&region) <= i64::from(self.settings.occluder_minium_visible_quad_size)
        {
            return false;
        }
        occlusion.Union(&region);
        self.ReduceOccluderComplexity(occlusion);
        true
    }

    fn ReduceOccluderComplexity(&self, region: &mut NativeRegion) {
        while region.GetRegionComplexity() > self.settings.maximum_occluder_complexity {
            let Some(smallest) = region
                .into_iter()
                .min_by_key(|rect| i64::from(rect.width.max(0)) * i64::from(rect.height.max(0)))
            else {
                break;
            };
            region.Subtract(&smallest);
        }
    }

    fn ReducedVisibleRects(&self, region: &NativeRegion) -> Option<Vec<Rect>> {
        let mut reduced = Vec::<Rect>::new();
        for rect in region {
            if let Some(existing) = reduced.iter_mut().find(|other| SharesEdge(**other, rect)) {
                existing.Union(rect);
            } else {
                reduced.push(rect);
            }
            if reduced.len() >= self.settings.quad_split_limit.max(1) as usize {
                return None;
            }
        }
        Some(reduced)
    }

    fn ShouldSplit(&self, bounds: Rect, rects: &[Rect]) -> bool {
        if rects.len() < 2 {
            return false;
        }
        let bounds_area = i64::from(bounds.width.max(0)) * i64::from(bounds.height.max(0));
        let rect_area: i64 = rects
            .iter()
            .map(|rect| i64::from(rect.width.max(0)) * i64::from(rect.height.max(0)))
            .sum();
        let reduced = bounds_area.saturating_sub(rect_area) as f32;
        reduced * self.device_scale_factor * self.device_scale_factor
            > self.settings.minimum_fragments_reduced as f32
    }

    pub fn RemoveOverdrawQuads(&mut self, frame: &mut AggregatedFrame) -> OcclusionCullStats {
        let mut stats = OcclusionCullStats::default();
        stats.quads_before = frame
            .render_pass_list
            .iter()
            .map(|pass| pass.quad_list.len())
            .sum();

        // Chromium skips culling a pass when a render-pass draw quad can move
        // pixels through foreground or backdrop filters. Aggregated quads in
        // this model reference the child pass, so collect the child IDs first.
        let filtered_passes: BTreeSet<u64> = frame
            .render_pass_list
            .iter()
            .filter(|pass| !pass.filters.IsEmpty() || !pass.backdrop_filters.IsEmpty())
            .map(|pass| pass.id)
            .collect();

        for pass in &mut frame.render_pass_list {
            if pass.quad_list.len() <= 1 || filtered_passes.contains(&pass.id) {
                continue;
            }

            let mut occlusion = NativeRegion::default();
            let mut occlusion_in_content = NativeRegion::default();
            let mut last_sqs_index: Option<usize> = None;
            let mut last_sqs_is_render_pass = false;
            let mut current_sqs_intersects = false;
            let mut index = 0usize;

            while index < pass.quad_list.len() {
                let Some(sqs_handle) = pass.quad_list[index].geometry.shared_quad_state else {
                    index += 1;
                    continue;
                };
                let sqs_index = sqs_handle.Index();
                let is_render_pass = Self::IsRenderPass(&pass.quad_list[index]);

                if last_sqs_index != Some(sqs_index) {
                    if let Some(previous) = last_sqs_index {
                        if self.AddOccluder(
                            &pass.shared_quad_state_list[previous],
                            last_sqs_is_render_pass,
                            &mut occlusion,
                        ) {
                            stats.opaque_groups += 1;
                        }
                    }
                    last_sqs_index = Some(sqs_index);
                    last_sqs_is_render_pass = is_render_pass;
                    occlusion_in_content.Clear();

                    let sqs = &pass.shared_quad_state_list[sqs_index];
                    let current_target = MathUtil::MapEnclosingClippedRect(
                        &sqs.quad_to_target_transform,
                        sqs.visible_quad_layer_rect,
                    );
                    current_sqs_intersects = occlusion.Intersects(&current_target);
                    if current_sqs_intersects && Self::IsSimpleInvertible(&sqs.quad_to_target_transform)
                    {
                        let inverse = sqs.quad_to_target_transform.GetCheckedInverse();
                        for target_rect in &occlusion {
                            if current_target.Intersects(target_rect) {
                                let content_rect =
                                    MathUtil::MapEnclosedRectWith2dAxisAlignedTransform(
                                        &inverse,
                                        target_rect,
                                    );
                                occlusion_in_content.Union(&content_rect);
                            }
                        }
                    }
                }

                if !current_sqs_intersects {
                    index += 1;
                    continue;
                }

                let visible = pass.quad_list[index].geometry.visible_rect;
                if occlusion_in_content.Contains(&visible) {
                    pass.quad_list[index].geometry.visible_rect = Rect::default();
                    stats.fully_culled += 1;
                    index += 1;
                    continue;
                }

                if !is_render_pass && occlusion_in_content.Intersects(&visible) {
                    let mut visible_region = NativeRegion::new(visible);
                    visible_region.Subtract(&occlusion_in_content);
                    let bounds = visible_region.bounds();
                    pass.quad_list[index].geometry.visible_rect = bounds;
                    stats.reduced += 1;

                    if let Some(rects) = self.ReducedVisibleRects(&visible_region) {
                        if self.ShouldSplit(bounds, &rects) {
                            let original = pass.quad_list[index].clone();
                            let count = rects.len();
                            let replacements = rects.into_iter().map(|visible_rect| {
                                let mut copy = original.clone();
                                copy.geometry.visible_rect = visible_rect;
                                copy
                            });
                            pass.quad_list.splice(index..=index, replacements);
                            stats.split += count.saturating_sub(1);
                            index += count;
                            continue;
                        }
                    }
                } else if occlusion_in_content.IsEmpty()
                    && occlusion.Contains(&MathUtil::MapEnclosingClippedRect(
                        &pass.shared_quad_state_list[sqs_index].quad_to_target_transform,
                        visible,
                    ))
                {
                    pass.quad_list[index].geometry.visible_rect = Rect::default();
                    stats.fully_culled += 1;
                }
                index += 1;
            }
        }

        stats.quads_after = frame
            .render_pass_list
            .iter()
            .map(|pass| pass.quad_list.len())
            .sum();
        stats
    }
}

fn SharesEdge(a: Rect, b: Rect) -> bool {
    (a.y == b.y && a.height == b.height && (a.right() == b.x || b.right() == a.x))
        || (a.x == b.x && a.width == b.width && (a.bottom() == b.y || b.bottom() == a.y))
}
