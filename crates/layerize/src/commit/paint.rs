//! cc/paint as commit vocabulary. The values live in the cc_paint root
//! (Chromium's separate cc/paint component); both halves name them through
//! this path. The main-side bindings stay here: the PaintArtifact handoff
//! (records are built from it by Paint's PaintRecordForArtifact /
//! PaintRecordForItem) and the recording-source / picture-recorder facets.

pub use cc_paint::*;
pub use paint::paint_engine::{PaintArtifact, RecordedDisplayItem as DisplayItem};

mod picture;
pub use picture::*;
#[cfg(test)]
mod lowering_tests;
