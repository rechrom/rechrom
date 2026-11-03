//! Fixed-cost LayoutView paint and compositing policy.
//!
//! These methods belong to the fragment/paint boundary, but remain inlineable
//! because they do not traverse retained state or perform input-sized work.

use super::*;

impl LayoutView {
    pub fn BackgroundIsKnownToBeOpaqueInRect(&self, _local_rect: &PhysicalRect) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn AdditionalCompositingReasons(&self) -> CompositingReasons {
        self.CheckIsNotDestroyed();
        CompositingReasons::default()
    }

    pub fn HasTickmarks(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn GetTickmarks(&self) -> foundation::Vector<gfx::Rect> {
        self.CheckIsNotDestroyed();
        foundation::Vector::default()
    }

    pub fn LayerTypeRequired(&self) -> PaintLayerType {
        self.CheckIsNotDestroyed();
        PaintLayerType::kNormalPaintLayer
    }

    pub fn CanHaveAdditionalCompositingReasons(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    fn ComputeCanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }
}
