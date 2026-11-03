//! Completeness of a paint traversal relative to its cull rect.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/paint_result.h:12-25`.

/// Used as the return type of paint methods whose output may be clipped by
/// their current cull rect.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PaintResult {
    /// The layer/object is fully painted, including the case where nothing
    /// needs painting for any paint rect.
    #[default]
    kFullyPainted,
    /// Some part lies outside the cull rect. The result must not be treated as
    /// complete when a later paint uses a different cull rect.
    kMayBeClippedByCullRect,
}

pub const kMaxPaintResult: PaintResult = PaintResult::kMayBeClippedByCullRect;

impl PaintResult {
    /// Chromium propagates the incomplete result from every painted child.
    pub fn Include(&mut self, child: Self) {
        if child == Self::kMayBeClippedByCullRect {
            *self = child;
        }
    }
}
