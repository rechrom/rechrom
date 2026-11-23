//! The binding of the main half to the one concrete compositor family
//! (MAIN_IMPL_SPLIT.md, "Plan correction", option h).
//!
//! The family type stays in the compositor, because it also implements impl
//! traits of the compositor sub-crates (orphan rule). The compositor
//! implements this trait for it. Main traits that move into layerize are
//! implemented by blanket impls over `F: LayerizeFamily`, and main code that
//! named the concrete family becomes generic over `F`, using `F::Skia` where
//! it used the Skia backend parameter `R`.
//!
//! Chromium has one concrete cc::Layer / cc::LayerImpl family; this trait is
//! the template-parameter projection of it, not a second model.
use super::{
    commit_state::CommitStateTypes, push_properties::PushPropertiesTypes,
    region_backend::SkRegionBackend, skia_value_backend::MainSkiaBackend,
};

pub trait LayerizeFamily: PushPropertiesTypes + CommitStateTypes {
    /// The external Skia value backend (the compositor's `R`).
    type Skia: MainSkiaBackend + SkRegionBackend + Send + Sync + 'static;
}
