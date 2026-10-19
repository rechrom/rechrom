//! Pure CPU-family entry points into canonical HostImpl's live field factory.
//! All native queries, mutation and release control flow remain in that stem.
use super::types::{CpuCompositorTypes, CpuSkiaBackend};
use crate::adapter::{
    host_picture::{HostPictureFields, HostPictureRegistrationOwnerFields},
    layer_tree_impl::LayerTreeScrollbarRegistrationHost,
    picture_layer_impl::{
        PictureLayerImplServices, PictureLayerImplTilingEnvironment, PictureLayerReadHost,
        PictureLayerTreeHost,
    },
};
pub fn WithPictureContextParts<R: CpuSkiaBackend>(
    fields: HostPictureFields<'_, CpuCompositorTypes<R>>,
    callback: &mut dyn FnMut(
        &dyn PictureLayerReadHost<CpuCompositorTypes<R>>,
        &mut dyn PictureLayerTreeHost<CpuCompositorTypes<R>>,
        &mut dyn PictureLayerImplServices<CpuCompositorTypes<R>>,
        &mut dyn PictureLayerImplTilingEnvironment<CpuCompositorTypes<R>>,
    ),
) {
    crate::cc::trees::layer_tree_host_impl::WithHostPictureContextParts(fields, callback)
}
pub fn WithLayerContextParts<R: CpuSkiaBackend>(
    fields: HostPictureFields<'_, CpuCompositorTypes<R>>,
    registration: HostPictureRegistrationOwnerFields<'_, CpuCompositorTypes<R>>,
    callback: &mut dyn FnMut(
        &dyn PictureLayerReadHost<CpuCompositorTypes<R>>,
        &mut dyn PictureLayerTreeHost<CpuCompositorTypes<R>>,
        &mut dyn PictureLayerImplServices<CpuCompositorTypes<R>>,
        &mut dyn PictureLayerImplTilingEnvironment<CpuCompositorTypes<R>>,
        &mut dyn LayerTreeScrollbarRegistrationHost<CpuCompositorTypes<R>>,
    ),
) {
    crate::cc::trees::layer_tree_host_impl::WithHostLayerContextParts(
        fields,
        registration,
        callback,
    )
}
