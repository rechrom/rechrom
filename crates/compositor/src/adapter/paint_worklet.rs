//! Paint worklet property values remain owned by the existing paint package.
pub trait PropertyValue: Default + Clone {
    fn reset(&mut self);
    fn has_value(&self) -> bool;
}
pub trait PaintWorkletInput<K> {
    fn GetPropertyKeys(&self) -> &[K];
    fn WorkletId(&self) -> u64;
}
pub trait PaintWorkletTypes {
    type PropertyKey: Ord + Clone;
    type PropertyValue: PropertyValue;
    type Input: PaintWorkletInput<Self::PropertyKey>;
    type PaintImageId;
}
