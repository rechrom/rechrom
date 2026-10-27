//! Fixed-cost LayoutView paint and compositing policy.
//!
//! These methods belong to the fragment/paint boundary, but remain inlineable
//! because they do not traverse retained state or perform input-sized work.

use super::*;

impl LayoutView {
    // cpp: layoutng/internal/layout_view_layout.cc:252-256
    pub fn BackgroundIsKnownToBeOpaqueInRect(&self, _local_rect: &PhysicalRect) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_view_layout.cc:263-266
    pub fn AdditionalCompositingReasons(&self) -> CompositingReasons {
        self.CheckIsNotDestroyed();
        CompositingReasons::default()
    }

    // cpp: layoutng/internal/layout_view_layout.cc:268-271
    pub fn HasTickmarks(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_view_layout.cc:273-276
    pub fn GetTickmarks(&self) -> foundation::Vector<gfx::Rect> {
        self.CheckIsNotDestroyed();
        foundation::Vector::default()
    }

    // cpp: layoutng/internal/layout_view.h:102-115
    pub fn LayerTypeRequired(&self) -> PaintLayerType {
        self.CheckIsNotDestroyed();
        PaintLayerType::kNormalPaintLayer
    }

    // cpp: layoutng/internal/layout_view.h:176-180
    pub fn CanHaveAdditionalCompositingReasons(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_view.h:358-363
    fn ComputeCanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }
}
