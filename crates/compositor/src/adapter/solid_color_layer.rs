//! External SkPicture recording plus same-package derived constructor binding.
//! All opacity/draw/quad decisions remain in the canonical source modules.
use super::{
    layer_tree_impl::Color4f,
    main_layer::{MainLayerImpl, MainLayerTypes},
};
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
/// Same-package type erasure of the SolidColorLayerImpl push target. Its
/// constructor is reached through `main_layer::LayerImplFactory`.
pub trait SolidColorLayerImplTarget<B: SolidColorLayerTypes>: MainLayerImpl<B> {}
