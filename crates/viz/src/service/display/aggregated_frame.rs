// Source: components/viz/service/display/aggregated_frame.h:1-75
// Source: components/viz/service/display/aggregated_frame.cc:1-46
#![allow(non_snake_case)]

use crate::{
    common::{
        quads::aggregated_render_pass::AggregatedRenderPassOutput, surfaces::surface_id::SurfaceId,
    },
    native::SurfaceId as PublicSurfaceId,
};
use compositor_core::adapter::geometry::Rect;

#[derive(Clone, Debug, Default)]
pub struct DrawReceipt(pub(crate) Vec<(PublicSurfaceId, u64)>);

pub struct AggregatedFrame<'a> {
    pub(crate) output_rect: Rect,
    pub(crate) passes: Vec<AggregatedRenderPassOutput<'a>>,
    pub(crate) receipt: DrawReceipt,
    pub(crate) contained_surfaces: Vec<SurfaceId>,
    pub(crate) root_damage_rect: Rect,
    pub(crate) surface_damage_rects: Vec<Rect>,
}
impl<'a> AggregatedFrame<'a> {
    pub(crate) fn new(
        output_rect: Rect,
        passes: Vec<AggregatedRenderPassOutput<'a>>,
        receipt: DrawReceipt,
        contained_surfaces: Vec<SurfaceId>,
        root_damage_rect: Rect,
        surface_damage_rects: Vec<Rect>,
    ) -> Self {
        Self {
            output_rect,
            passes,
            receipt,
            contained_surfaces,
            root_damage_rect,
            surface_damage_rects,
        }
    }
    pub fn OutputRect(&self) -> Rect {
        self.output_rect
    }
    pub fn RenderPasses(&self) -> &[AggregatedRenderPassOutput<'a>] {
        &self.passes
    }
    pub fn Receipt(&self) -> DrawReceipt {
        self.receipt.clone()
    }
    pub fn ContainedSurfaces(&self) -> &[SurfaceId] {
        &self.contained_surfaces
    }
    pub fn RootDamageRect(&self) -> Rect {
        self.root_damage_rect
    }
    pub fn SurfaceDamageRects(&self) -> &[Rect] {
        &self.surface_damage_rects
    }
}
