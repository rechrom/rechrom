//! Live feature services read by canonical LayerTreeSettings. The embedder
//! supplies process feature values; the settings type retains every source field.
pub trait LayerTreeSettingsFeatures {
    fn UseLayerListsByDefault(&self) -> bool;
    fn TreesInViz(&self) -> bool;
    fn TreeAnimationsInViz(&self) -> bool;
    fn DeferImplInvalidation(&self) -> bool;
    fn DeferImplInvalidationFrames(&self) -> i32;
    fn ScrollEventDispatchMode(&self) -> &str;
    fn WaitForLateScrollEvents(&self) -> bool;
    fn WaitForLateScrollEventsDeadlineRatio(&self) -> f64;
    fn ImageDecodeWorkingSetBytes(&self, for_renderer: bool) -> usize;
}
