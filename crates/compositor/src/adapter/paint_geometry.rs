//! Live geometry-mapper boundary used by Blink PaintArtifact layerization.
//! The Paint owner supplies its resident transform nodes and canonical mapper;
//! compositor does not clone the property tree or implement another mapper.

use super::geometry::RectF;
pub use paint_model::geometry_mapper_source::traits::*;
pub trait MaskGeometryMapper {
    type TransformPaintPropertyNode: ?Sized;

    fn SourceToDestinationRectFromRoot(
        &self,
        transform: &Self::TransformPaintPropertyNode,
        rect: &mut RectF,
    );
}

pub use compositor_layerize::paint_geometry::{ChunkGeometryMapper, RasterEffectOutset};

/// Facade-owned projection of an external Paint chunk. Keeping this trait in
/// the facade permits native Paint types to implement it without orphan-rule
/// coupling between the responsibility packages.
pub trait PaintChunkView<M: ChunkGeometryMapper + ?Sized> {
    fn properties_unalias(&self) -> M::PropertyTreeState;
    fn raster_effect_outset(&self) -> RasterEffectOutset;
}
