//! Chunk identity/state algorithm translated from Blink paint_chunker.cc.
//! Chunk ranges address semantic DisplayItems, independent of replay op counts.
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::paint_engine::{
    PaintChunk, PaintHitTestData, PaintHitTestOpaqueness, PaintRect, RasterEffectOutset,
};
use crate::paint_property_tree::PropertyTreeState;
use layoutng_assembly::fragment_tree::FragmentNode;

#[derive(Clone, Copy, Debug)]
pub struct DisplayItemClientInfo {
    pub invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub scrollbar_needs_update_display: Option<bool>,
    pub is_cacheable: bool,
    pub is_just_created: bool,
}

impl DisplayItemClientInfo {
    pub(crate) fn from_fragment(fragment: &FragmentNode) -> Self {
        Self {
            invalidation_reason: fragment.paint.display_item_client_invalidation_reason,
            scrollbar_needs_update_display: None,
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
        self.FinalizeLastChunkProperties(chunks);
        let begin = chunks.last().map_or(0, |chunk| chunk.end_index);
        chunks.push(PaintChunk {
            compositor: Some(Box::new(
                crate::paint_engine::PaintChunkCompositorData::new(),
            )),
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

    // Source: paint_chunker.cc:305-348. Scroll hit-test data is a metadata-only
    // paint chunk in the scroller's border-box property state. It deliberately
    // owns no DisplayItem; PAC consumes the typed hit-test payload directly.
    pub fn RecordScrollHitTestData(
        &mut self,
        chunks: &mut Vec<PaintChunk>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        rect: PaintRect,
        opaqueness: PaintHitTestOpaqueness,
        data: PaintHitTestData,
    ) {
        self.force_new_chunk();
        let created = self.ensure_current_chunk(chunks, id, client);
        debug_assert!(created);
        let chunk = chunks.last_mut().expect("scroll hit-test chunk");
        chunk.bounds.union(rect);
        let compositor = chunk
            .compositor
            .as_deref_mut()
            .expect("scroll hit-test compositor metadata");
        compositor.hit_test_opaqueness = opaqueness;
        compositor.hit_test_data = Some(Box::new(data));
        self.force_new_chunk();
    }

    // Source: paint_chunker.cc:350-393. The record analysis result comes from
    // its actual producer; the canonical chunk retains its own derived fields.
    pub fn UpdateDrawingDerivedData(
        &mut self,
        chunks: &mut [PaintChunk],
        candidate: crate::paint_engine::PaintChunkBackgroundColorInfo,
        has_text: bool,
        visual_rect: PaintRect,
    ) {
        let chunk = chunks.last_mut().expect("source current chunk");
        let m = chunk
            .compositor
            .as_deref_mut()
            .expect("source native metadata");
        let min_area = 0.5 * chunk.bounds.width as f32 * chunk.bounds.height as f32;
        if candidate.background_area >= m.background_area || candidate.background_area >= min_area {
            if m.background_area >= min_area && candidate.background_color.alpha != 1. {
                m.background_area = candidate.background_area;
                m.background_color =
                    ResultingPaintColor(candidate.background_color, m.background_color);
            } else {
                m.background_color = candidate.background_color;
                m.background_area = candidate.background_area;
                m.background_is_solid_color = candidate.background_is_solid_color;
            }
        }
        if m.text_known_to_be_on_opaque_background {
            if has_text {
                m.has_text = true;
                let r = chunk.rect_known_to_be_opaque;
                m.text_known_to_be_on_opaque_background = r.x <= visual_rect.x
                    && r.y <= visual_rect.y
                    && r.x + r.width >= visual_rect.x + visual_rect.width
                    && r.y + r.height >= visual_rect.y + visual_rect.height;
            }
        } else {
            debug_assert!(m.has_text);
        }
    }
    pub fn FinalizeLastChunkProperties(&mut self, chunks: &mut [PaintChunk]) {
        let Some(chunk) = chunks.last_mut() else {
            return;
        };
        if chunk.is_moved_from_cached_subsequence {
            return;
        }
        let m = chunk
            .compositor
            .as_deref_mut()
            .expect("source native metadata");
        if chunk.end_index - chunk.begin_index > 1
            || m.background_area != chunk.bounds.width as f32 * chunk.bounds.height as f32
        {
            m.background_is_solid_color = false;
        }
    }
    // Source: paint_chunker.cc:15-20.
    pub fn Finish(&mut self, chunks: &mut [PaintChunk]) {
        self.FinalizeLastChunkProperties(chunks);
    }
}

// Source primitive: ui/gfx/color_utils.cc:577-614. Quantize to native SkColor,
// source alpha normalization, ClampRound and conversion back to SkColor4f.
fn ResultingPaintColor(
    f: layoutng_assembly::internal::layout_input_types::Color,
    b: layoutng_assembly::internal::layout_input_types::Color,
) -> layoutng_assembly::internal::layout_input_types::Color {
    use layoutng_assembly::internal::layout_input_types::Color;
    let q = |v: f32| (v * 255.).round().clamp(0., 255.);
    let a = q(f.alpha) / 255.;
    let fa = [q(f.red), q(f.green), q(f.blue), 255.];
    let ba = [q(b.red), q(b.green), q(b.blue), q(b.alpha)];
    let v = if a == 0. {
        ba
    } else if a == 1. {
        fa
    } else {
        let n = 255. * a + ba[3] * (1. - a);
        if n == 0. {
            [0.; 4]
        } else {
            let fw = 255. * a / n;
            let bw = ba[3] * (1. - a) / n;
            [
                (fa[0] * fw + ba[0] * bw).round(),
                (fa[1] * fw + ba[1] * bw).round(),
                (fa[2] * fw + ba[2] * bw).round(),
                n.round(),
            ]
        }
    };
    Color {
        red: v[0] / 255.,
        green: v[1] / 255.,
        blue: v[2] / 255.,
        alpha: v[3] / 255.,
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
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar_needs_update_display: None,
            is_just_created: false,
        };
        let root = PropertyTreeState::default();
        let mut child = root.clone();
        child.transform = Arc::new(TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            scroll_parent_scroll_translation: None,
            lifecycle: Default::default(),
            id: 1,
            parent: Some(root.transform.clone()),
            matrix: Default::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: None,
            sticky_constraint: None,
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
