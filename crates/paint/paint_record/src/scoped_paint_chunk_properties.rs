//! Scoped PaintChunk property and identity replacement.
//!
//! Source entity: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/
//! scoped_paint_chunk_properties.h`.

use crate::display_item_id::DisplayItemId;
use crate::paint_chunker::DisplayItemClientInfo;
use crate::paint_context::{PaintContext, ScopedPaintChunkProperties};
use crate::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PropertyTreeState, TransformPaintPropertyNode,
};
use std::cell::RefCell;
use std::sync::Arc;

impl<'c, 'o> ScopedPaintChunkProperties<'c, 'o> {
    /// Source: `scoped_paint_chunk_properties.h:21-29`.
    pub fn with_client(
        context: &'c RefCell<PaintContext<'o>>,
        properties: PropertyTreeState,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
    ) -> Self {
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        borrowed.update_chunk_id_and_properties_raw(id, client, properties);
        Self { context, previous }
    }

    /// Source: `scoped_paint_chunk_properties.h:31-40,78-85`.
    pub fn transform(
        context: &'c RefCell<PaintContext<'o>>,
        transform: Arc<TransformPaintPropertyNode>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
    ) -> Self {
        let mut properties = context.borrow().chunk_properties();
        properties.transform = transform;
        Self::with_client(context, properties, id, client)
    }

    /// Source: `scoped_paint_chunk_properties.h:42-51,87-94`.
    pub fn clip(
        context: &'c RefCell<PaintContext<'o>>,
        clip: Arc<ClipPaintPropertyNode>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
    ) -> Self {
        let mut properties = context.borrow().chunk_properties();
        properties.clip = clip;
        Self::with_client(context, properties, id, client)
    }

    /// Source: `scoped_paint_chunk_properties.h:53-62,96-103`.
    pub fn effect(
        context: &'c RefCell<PaintContext<'o>>,
        effect: Arc<EffectPaintPropertyNode>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
    ) -> Self {
        let mut properties = context.borrow().chunk_properties();
        properties.effect = effect;
        Self::with_client(context, properties, id, client)
    }
}

/// Source: `scoped_paint_chunk_properties.h:68-75`.
///
/// Restoring the property state deliberately does not restore the previous
/// chunk ID. A following chunk receives a new scope ID or its first display
/// item's ID, exactly as required by the source lifetime contract.
impl Drop for ScopedPaintChunkProperties<'_, '_> {
    fn drop(&mut self) {
        self.context
            .borrow_mut()
            .update_chunk_properties(self.previous.clone());
    }
}
