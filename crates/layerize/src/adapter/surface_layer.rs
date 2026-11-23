//! Main-thread SurfaceLayer constructor/property push boundary.
use super::main_layer::{MainLayerImpl, MainLayerTypes};
use crate::commit::surface_range::SurfaceRangeTypes;

pub trait SurfaceLayerTypes:
    MainLayerTypes<Layer: SurfaceLayerImplTarget<Self>> + SurfaceRangeTypes
{
}

pub trait SurfaceLayerImplTarget<B: SurfaceLayerTypes>: MainLayerImpl<B> {
    fn SurfaceSetRange(
        &mut self,
        range: B::SurfaceRange,
        deadline: Option<u32>,
        tree: &mut B::ImplContext<'_>,
    );
    fn SurfaceSetStretchContentToFillBounds(&mut self, value: bool, tree: &mut B::ImplContext<'_>);
    fn SurfaceSetIsReflection(&mut self, value: bool, tree: &mut B::ImplContext<'_>);
}

impl<T> SurfaceLayerTypes for T where
    T: MainLayerTypes<Layer: SurfaceLayerImplTarget<T>> + SurfaceRangeTypes
{
}
