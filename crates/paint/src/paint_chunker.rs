//! Chunk identity/state algorithm translated from Blink paint_chunker.cc.
//! Chunk ranges address semantic DisplayItems, independent of replay op counts.
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::paint_engine::{PaintChunk, PaintRect, RasterEffectOutset};
use crate::paint_property_tree::PropertyTreeState;
use layoutng_assembly::fragment_tree::FragmentNode;

#[derive(Clone, Copy, Debug)]
pub struct DisplayItemClientInfo {
    pub is_cacheable: bool,
    pub is_just_created: bool,
}

impl DisplayItemClientInfo {
    pub(crate) fn from_fragment(fragment: &FragmentNode) -> Self {
        Self {
            is_cacheable: fragment.paint.display_item_client_is_cacheable,
            is_just_created: fragment.paint.display_item_client_is_just_created,
        }
    }
}

pub struct PaintChunker {
    current_properties: PropertyTreeState,
    next_chunk_id: Option<(DisplayItemId, DisplayItemClientInfo)>,
    will_force_new_chunk: bool,
    current_effectively_invisible: bool,
}

impl Default for PaintChunker {
    fn default() -> Self {
        Self {
            current_properties: PropertyTreeState::default(),
            next_chunk_id: None,
            will_force_new_chunk: true,
            current_effectively_invisible: false,
        }
    }
}

impl PaintChunker {
    pub fn current_properties(&self) -> &PropertyTreeState {
        &self.current_properties
    }

    // paint_chunker.cc: UpdateCurrentPaintChunkProperties(properties).
    pub fn update_properties(&mut self, properties: PropertyTreeState) {
        if !self.current_properties.same_nodes(&properties) {
            self.next_chunk_id = None;
            self.current_properties = properties;
        }
    }

    // Preserve a pending outer scope ID while its property state is unchanged.
    // paint_chunker.cc: UpdateCurrentPaintChunkProperties(id, client, properties).
    pub fn update_id_and_properties(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        properties: PropertyTreeState,
    ) {
        if self.next_chunk_id.is_none() || !self.current_properties.same_nodes(&properties) {
            self.next_chunk_id = Some((id, client));
        }
        self.current_properties = properties;
    }

    pub fn set_current_effectively_invisible(&mut self, invisible: bool) {
        self.current_effectively_invisible = invisible;
    }

    pub fn force_new_chunk(&mut self) {
        self.will_force_new_chunk = true;
        self.next_chunk_id = None;
    }

    // paint_chunker.cc: EnsureCurrentChunk(). Property node identity determines
    // segmentation; neither property values nor command positions enter IDs.
    pub fn ensure_current_chunk(
        &mut self,
        chunks: &mut Vec<PaintChunk>,
        fallback_id: DisplayItemId,
        client: DisplayItemClientInfo,
    ) -> bool {
        let create = self.will_force_new_chunk
            || chunks
                .last()
                .is_none_or(|last| !last.properties.same_nodes(&self.current_properties));
        if !create {
            return false;
        }
        let (id, client) = self.next_chunk_id.take().unwrap_or((fallback_id, client));
        let begin = chunks.last().map_or(0, |chunk| chunk.end_index);
        chunks.push(PaintChunk {
            id,
            begin_index: begin,
            end_index: begin,
            bounds: PaintRect::default(),
            drawable_bounds: PaintRect::default(),
            rect_known_to_be_opaque: PaintRect::default(),
            bounds_are_complete: true,
            raster_effect_outset: RasterEffectOutset::kNone,
            properties: self.current_properties.clone(),
            is_cacheable: client.is_cacheable,
            client_is_just_created: client.is_just_created,
            is_moved_from_cached_subsequence: false,
            effectively_invisible: self.current_effectively_invisible,
        });
        self.will_force_new_chunk = false;
        true
    }

    pub fn increment_display_item_index(
        &mut self,
        chunks: &mut Vec<PaintChunk>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
    ) -> bool {
        self.increment_display_item(
            chunks,
            id,
            client,
            PaintRect::default(),
            false,
            RasterEffectOutset::kNone,
            false,
            PaintRect::default(),
        )
    }

    // paint_chunker.cc:104-106,138-139. An empty record still contributes its
    // VisualRect to bounds; only DrawsContent contributes drawable_bounds.
    pub fn increment_display_item(
        &mut self,
        chunks: &mut Vec<PaintChunk>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        visual_rect: PaintRect,
        draws_content: bool,
        raster_effect_outset: RasterEffectOutset,
        visual_rect_is_accurate: bool,
        rect_known_to_be_opaque: PaintRect,
    ) -> bool {
        let forces_new_chunk = (DisplayItemIdType::kForeignLayerFirst.0
            ..=DisplayItemIdType::kForeignLayerLast.0)
            .contains(&id.r#type.0)
            || id.r#type == DisplayItemIdType::kScrollbarHorizontal
            || id.r#type == DisplayItemIdType::kScrollbarVertical;
        if forces_new_chunk {
            self.force_new_chunk();
        }
        let created = self.ensure_current_chunk(chunks, id, client);
        let chunk = chunks.last_mut().unwrap();
        chunk.end_index += 1;
        chunk.bounds.union(visual_rect);
        chunk.bounds_are_complete &= visual_rect_is_accurate;
        if draws_content {
            chunk.drawable_bounds.union(visual_rect);
        }
        // paint_chunker.cc:118-120, never union across unpainted gaps.
        chunk.rect_known_to_be_opaque =
            PaintRect::maximum_covered_rect(chunk.rect_known_to_be_opaque, rect_known_to_be_opaque);
        if raster_effect_outset as u8 > chunk.raster_effect_outset as u8 {
            chunk.raster_effect_outset = raster_effect_outset;
        }
        if forces_new_chunk {
            self.force_new_chunk();
        }
        created
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display_item_id::DisplayItemIdType;
    use crate::paint_property_tree::TransformPaintPropertyNode;
    use std::sync::Arc;

    fn id(client_id: u64, item_type: DisplayItemIdType, fragment: u32) -> DisplayItemId {
        DisplayItemId {
            client_id,
            r#type: item_type,
            fragment,
        }
    }

    #[test]
    fn translated_chunk_identity_scope_fallback_and_matching() {
        let client = DisplayItemClientInfo {
            is_cacheable: true,
            is_just_created: false,
        };
        let root = PropertyTreeState::default();
        let mut child = root.clone();
        child.transform = Arc::new(TransformPaintPropertyNode {
            lifecycle: Default::default(),
            id: 1,
            parent: Some(root.transform.clone()),
            matrix: Default::default(),
            origin: [0.0; 3],
            scroll: None,
            direct_compositing_reasons: Vec::new(),
        });
        let outer = id(10, DisplayItemIdType::kLayerChunk, 0);
        let inner = id(20, DisplayItemIdType::kLayerChunk, 3);
        let drawing = id(30, DisplayItemIdType::kBoxDecorationBackground, 4);
        let mut chunker = PaintChunker::default();
        let mut chunks = Vec::new();
        chunker.update_id_and_properties(outer, client, root.clone());
        chunker.update_id_and_properties(inner, client, root.clone());
        assert!(chunker.ensure_current_chunk(&mut chunks, drawing, client));
        assert_eq!(chunks[0].id, outer); // Preserve the outer ID.
        assert!(!chunker.ensure_current_chunk(&mut chunks, drawing, client));
        chunker.update_id_and_properties(inner, client, child);
        assert!(chunker.ensure_current_chunk(&mut chunks, drawing, client));
        assert_eq!(chunks[1].id, inner);
        chunker.update_properties(root); // Restore state, not the old ID.
        assert!(chunker.ensure_current_chunk(&mut chunks, drawing, client));
        assert_eq!(chunks[2].id, drawing);
        assert!(chunks[2].matches(&chunks[2]));
        let mut created = chunks[2].clone();
        created.client_is_just_created = true;
        assert!(!created.matches(&chunks[2]));
        let mut uncacheable = chunks[2].clone();
        uncacheable.is_cacheable = false;
        assert!(!chunks[2].matches(&uncacheable));
    }
}
