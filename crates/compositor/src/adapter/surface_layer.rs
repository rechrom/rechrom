//! Main-thread SurfaceLayer constructor/property push boundary.
use super::{
    frame_data::FrameDataTypes,
    main_layer::{MainLayerImpl, MainLayerTypes},
    surface_layer_impl::SurfaceLayerImplTypes,
};
use crate::cc::trees::layer_tree_impl::{LayerTreeContext, LayerTreeImpl};

pub trait SurfaceLayerTypes:
    MainLayerTypes<Layer: SurfaceLayerImplTarget<Self>> + FrameDataTypes + SurfaceLayerImplTypes
{
}

pub trait SurfaceLayerImplTarget<B: SurfaceLayerTypes>: MainLayerImpl<B> {
    fn CreateSurface(
        tree: &mut LayerTreeImpl<B>,
        id: i32,
        context: &mut LayerTreeContext<'_, B>,
    ) -> Box<Self>;
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
    T: MainLayerTypes<Layer: SurfaceLayerImplTarget<T>> + FrameDataTypes + SurfaceLayerImplTypes
{
}
