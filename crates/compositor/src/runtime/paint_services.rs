//! Impl half of the production Paint value capabilities. The main paint
//! services (`CpuMainPaintServices`: PAC, chunk conversion, GeometryMapper,
//! main Layer services) are owned by layerize's LayerizeEngine
//! (MAIN_IMPL_SPLIT.md S1a-3 B3).
use super::types::{CpuCompositorTypes, CpuSkiaBackend};
use layerize::commit::paint::CpuPaintValueServices;
use std::sync::Arc;

/// Impl-thread paint value services: the immutable cc/paint value
/// capabilities that LayerTreeHostImpl's tile, raster and image-decode owners
/// replay committed PaintRecords with (cc/tiles/software_image_decode_cache.cc,
/// cc/raster/raster_source.cc). It shares the value capability with the main
/// half (cc/paint is used by both threads) and holds nothing main-owned.
pub struct CpuImplPaintServices<R: CpuSkiaBackend> {
    values: Arc<dyn CpuPaintValueServices<R>>,
}
impl<R: CpuSkiaBackend> CpuImplPaintServices<R> {
    /// The impl half's lease of the main services' shared value capability
    /// (`LayerizeEngine::PaintValues`).
    pub fn new(values: Arc<dyn CpuPaintValueServices<R>>) -> Self {
        Self { values }
    }
    pub(crate) fn PaintValues(&self) -> Arc<dyn CpuPaintValueServices<R>> {
        self.values.clone()
    }
}

// A non-test compile assertion proves the actual default family, rather than
// proving only a generic where-clause that no production type implements.
const _: fn() = || {
    // Separate bounds: a combined where-clause would add LayerizeFamily
    // param-env candidates next to the impl traits.
    fn require_main<B: layerize::commit::layerize_family::LayerizeFamily>() {}
    fn require_impl<
        B: crate::adapter::picture_layer_impl::PictureLayerImplTypes
            + crate::adapter::picture_layer_tiling::PictureLayerTilingTypes
            + crate::adapter::tile_manager::TileManagerTypes,
    >() {
    }
    require_main::<CpuCompositorTypes<crate::runtime::skia::SkRegion>>();
    require_impl::<CpuCompositorTypes<crate::runtime::skia::SkRegion>>();
};
