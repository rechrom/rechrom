//! LayoutBlock paint invalidation during PrePaint.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/block_paint_invalidator.{h,cc}`.

use foundation::PaintInvalidationReason;
use layoutng_assembly::fragment_tree::FragmentPaintData;
use layoutng_assembly::internal::layout_input::Offset;

use crate::box_paint_invalidator::BoxPaintInvalidator;
use crate::paint_invalidator::PaintInvalidatorContext;

pub struct BlockPaintInvalidator;

impl BlockPaintInvalidator {
    /// Source: `block_paint_invalidator.cc:18-24`. Selection and drag-caret
    /// clients remain separate document-owned inputs; the LayoutBlock's box
    /// invalidation is preserved here as the first operation.
    pub fn ComputePaintInvalidationReason(
        fragment: &FragmentPaintData,
        previous: Option<&FragmentPaintData>,
        old_paint_offset: Option<Offset>,
        new_paint_offset: Offset,
        context: PaintInvalidatorContext,
        reason: PaintInvalidationReason,
    ) -> PaintInvalidationReason {
        BoxPaintInvalidator::ComputePaintInvalidationReason(
            fragment,
            previous,
            old_paint_offset,
            new_paint_offset,
            context,
            reason,
        )
    }
}
