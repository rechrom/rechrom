//! Impl half of the production Paint value capabilities. The main paint
//! services (`CpuMainPaintServices`: PAC, chunk conversion, GeometryMapper,
//! main Layer services) live in layerize `runtime/paint_services.rs`
//! (MAIN_IMPL_SPLIT.md S1a-3 B3).
pub use layerize::runtime::paint_services::{
    CpuMainPaintServices, CpuPaintFeatures, CpuPaintPlatform,
};
use super::{
    paint,
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use std::sync::Arc;

/// Impl-thread paint value services: the immutable cc/paint value
/// capabilities that LayerTreeHostImpl's tile, raster and image-decode owners
/// replay committed PaintRecords with (cc/tiles/software_image_decode_cache.cc,
/// cc/raster/raster_source.cc). It shares the value capability with the main
/// half (cc/paint is used by both threads) and holds nothing main-owned.
pub struct CpuImplPaintServices<R: CpuSkiaBackend> {
    values: Arc<dyn paint::CpuPaintValueServices<R>>,
}
impl<R: CpuSkiaBackend> CpuImplPaintServices<R> {
    /// The impl half's lease of the main services' shared value capability.
    pub fn new(main: &CpuMainPaintServices<R>) -> Self {
        Self {
            values: main.PaintValues(),
        }
    }
    pub(crate) fn PaintValues(&self) -> Arc<dyn paint::CpuPaintValueServices<R>> {
        self.values.clone()
    }
}

// Source: cc/trees/tree_synchronizer.cc TRACE_EVENT scopes, which
// FinishCommit runs on impl with the main services lent to the commit.
impl<R: CpuSkiaBackend> crate::adapter::tree_sync::SynchronizationTrace
    for CpuMainPaintServices<R>
{
    fn TraceScope(&self, name: &str, count: Option<usize>) {
        self.Platform().TraceScope("cc", name, count)
    }
}

// A non-test compile assertion proves the actual default family, rather than
// proving only a generic where-clause that no production type implements.
const _: fn() = || {
    fn require<
        B: crate::adapter::paint_artifact_compositor::PaintArtifactCompositorTypes
            + crate::adapter::picture_layer_impl::PictureLayerImplTypes
            + crate::adapter::picture_layer_tiling::PictureLayerTilingTypes
            + crate::adapter::tile_manager::TileManagerTypes,
    >() {
    }
    require::<CpuCompositorTypes<crate::runtime::skia::SkRegion>>();
};
