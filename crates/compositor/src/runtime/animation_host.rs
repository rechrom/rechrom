//! The Page's cc::AnimationHost instances. Chromium has one AnimationHost
//! class serving the main and the impl instance (cc/animation/
//! animation_host.h); it is a shared cc owner in `layerize::commit`.
pub use layerize::commit::animation_host::*;
