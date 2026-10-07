//! Identity and invalidation state of Blink's editing/CaretDisplayItemClient.
//! Geometry remains in the supported text-control caret geometry module.
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::PaintInvalidationReason;

// Blink FrameCaret owns a GC-allocated CaretDisplayItemClient
// (core/editing/frame_caret.cc:63). The standalone FrameCaret uses Box storage
// for the same persistent owner; moving FrameCaret does not move its client.
#[repr(C)]
#[derive(Default)]
pub struct CaretDisplayItemClient {
    display_item_client: DisplayItemClient,
}

#[allow(non_snake_case)]
impl CaretDisplayItemClient {
    pub fn Id(&self) -> u64 {
        self.display_item_client.Id() as u64
    }
    pub fn IsCacheable(&self) -> bool {
        self.display_item_client.IsCacheable()
    }
    pub fn IsJustCreated(&self) -> bool {
        self.display_item_client.IsJustCreated()
    }

    // ObjectPaintInvalidatorWithContext invalidates this independent client
    // with kCaret (core/editing/caret_display_item_client.cc:341,363).
    pub fn InvalidateForCaretPaint(&self) {
        self.display_item_client
            .Invalidate(PaintInvalidationReason::kCaret);
    }

    pub fn ValidateForCommittedPaint(&self) {
        self.display_item_client.ValidateForCommittedPaint();
    }
}
