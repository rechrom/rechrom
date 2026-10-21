// Root module wiring for delivered external source translations.
// Unresolved true native dependencies remain visible; zero Skia row credit.
pub mod lock;
pub mod lock_impl;
pub mod lock_impl_posix;
pub mod lock_metrics_recorder;
pub mod lock_metrics_recorder_tags;
pub mod lock_subtle;
