//! Main-thread SurfaceLayer constructor/property push boundary.
use super::main_layer::{MainLayerImpl, MainLayerTypes};
use crate::commit::surface_range::SurfaceRangeTypes;

pub trait SurfaceLayerTypes:
    MainLayerTypes + SurfaceRangeTypes
{
}

pub trait SurfaceLayerImplTarget<B: SurfaceLayerTypes>: MainLayerImpl<B> {
    fn SurfaceSetRange(
        &mut self,
        range: B::SurfaceRange,
        deadline: Option<u32>,
        tree: &mut Self::ImplContext<'_>,
    );
    fn SurfaceSetStretchContentToFillBounds(&mut self, value: bool, tree: &mut Self::ImplContext<'_>);
    fn SurfaceSetIsReflection(&mut self, value: bool, tree: &mut Self::ImplContext<'_>);
}

impl<T> SurfaceLayerTypes for T where
    T: MainLayerTypes + SurfaceRangeTypes
{
}
