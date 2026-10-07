#![allow(non_snake_case)]

use std::cell::Cell;

use super::display_item_client_types::{DisplayItemClientId, RasterEffectOutset};
use crate::graphics_types::graphics::dom_node_id::{kInvalidDOMNodeId, DOMNodeId};
use crate::{PaintInvalidationReason, Visitor};

const REASON_MASK: u8 = 0x1f;
const MARKED_FOR_VALIDATION: u8 = 1 << 5;

// The two C++ bitfields share one byte. The C++ virtual interface is
// dispatched by LayoutObject's Rust owner; this value preserves the mutable
// invalidation state and object identity used by layout/paint.
// cpp: foundation/graphics_types/graphics/paint/display_item_client.h:22-31,105-107
#[repr(C)]
pub struct DisplayItemClient {
    bits_: Cell<u8>,
}

impl Default for DisplayItemClient {
    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:25-28
    fn default() -> Self {
        Self {
            bits_: Cell::new(PaintInvalidationReason::kJustCreated as u8),
        }
    }
}

impl DisplayItemClient {
    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:33-35
    pub fn Id(&self) -> DisplayItemClientId {
        self as *const Self as DisplayItemClientId
    }

    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:42-44
    pub fn OwnerNodeId(&self, _is_internal_content: bool) -> DOMNodeId {
        kInvalidDOMNodeId
    }

    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:49-51
    pub fn VisualRectOutsetForRasterEffects(&self) -> RasterEffectOutset {
        RasterEffectOutset::kNone
    }

    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:58-63
    pub fn Invalidate(&self, reason: PaintInvalidationReason) {
        if reason > self.GetPaintInvalidationReason() {
            self.bits_
                .set((self.bits_.get() & !REASON_MASK) | reason as u8);
        }
    }

    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:65-67
    pub fn GetPaintInvalidationReason(&self) -> PaintInvalidationReason {
        // Only the enum discriminants in [0, kUncacheable] are ever written.
        unsafe { std::mem::transmute(self.bits_.get() & REASON_MASK) }
    }

    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:71-74
    pub fn IsJustCreated(&self) -> bool {
        self.GetPaintInvalidationReason() == PaintInvalidationReason::kJustCreated
    }

    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:79-82
    pub fn IsCacheable(&self) -> bool {
        self.GetPaintInvalidationReason() != PaintInvalidationReason::kUncacheable
    }

    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:86-88
    pub fn IsValid(&self) -> bool {
        self.GetPaintInvalidationReason() == PaintInvalidationReason::kNone
    }

    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:99-104
    pub(crate) fn MarkForValidation(&self) {
        self.bits_.set(self.bits_.get() | MARKED_FOR_VALIDATION);
    }

    pub(crate) fn IsMarkedForValidation(&self) -> bool {
        self.bits_.get() & MARKED_FOR_VALIDATION != 0
    }

    pub(crate) fn Validate(&self) {
        self.bits_.set(PaintInvalidationReason::kNone as u8);
    }

    /// The persistent paint owner calls this after committing an artifact,
    /// only for live clients used by that artifact. Uncacheable clients keep
    /// their invalidation reason, as in PaintController's cycle destructor.
    // cpp: platform/graphics/paint/paint_controller.cc:127-143
    pub fn ValidateForCommittedPaint(&self) {
        if self.IsCacheable() {
            self.Validate();
        }
    }

    // GarbageCollectedMixin has no fields to trace; concrete layout owners
    // trace their own members after calling this base method.
    // cpp: foundation/graphics_types/graphics/paint/display_item_client.h:22-23
    pub fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidation_keeps_the_strongest_reason_until_validation() {
        let client = DisplayItemClient::default();
        assert!(client.IsJustCreated());
        client.Invalidate(PaintInvalidationReason::kLayout);
        assert!(client.IsJustCreated());
        client.MarkForValidation();
        assert!(client.IsMarkedForValidation());
        client.Validate();
        assert!(client.IsValid());
        assert!(!client.IsMarkedForValidation());
        client.Invalidate(PaintInvalidationReason::kIncremental);
        client.Invalidate(PaintInvalidationReason::kStyle);
        assert_eq!(
            client.GetPaintInvalidationReason(),
            PaintInvalidationReason::kStyle
        );
    }
}
