//! External SkPicture recording plus same-package derived constructor binding.
//! All opacity/draw/quad decisions remain in the canonical source modules.
use super::{
    layer_tree_impl::Color4f,
    main_layer::{MainLayerImpl, MainLayerTypes},
};
use crate::cc::trees::layer_tree_impl::{LayerTreeContext, LayerTreeImpl};
use std::sync::Arc;
pub trait SolidColorLayerTypes:
    MainLayerTypes<
    DerivedServices: SolidColorLayerServices<Self>,
    Layer: SolidColorLayerImplTarget<Self>,
>
{
    type SolidPictureRecorder: SolidColorPictureRecorder<Self>;
}
pub trait SolidColorPictureRecorder<B: SolidColorLayerTypes> {
    fn BeginRecording(&mut self, width: i32, height: i32);
    fn DrawColor(&mut self, color: Color4f);
    fn FinishRecordingAsPicture(self) -> Arc<B::Picture>;
}
pub trait SolidColorLayerServices<B: SolidColorLayerTypes> {
    fn CreateSolidPictureRecorder(&self) -> B::SolidPictureRecorder;
}
/// Pure same-package constructor/type erasure. The binding must invoke the
/// canonical SolidColorLayerImpl::Create with the actual scoped impl context,
/// then wrap that owned derived allocation; it contains no constructor policy.
pub trait SolidColorLayerImplTarget<B: SolidColorLayerTypes>: MainLayerImpl<B> {
    fn CreateSolidColor(
        tree: &mut LayerTreeImpl<B>,
        id: i32,
        context: &mut LayerTreeContext<'_, B>,
    ) -> Box<Self>;
}
