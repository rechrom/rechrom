//! Chunk identity/state algorithm translated from Blink paint_chunker.cc.
//! Chunk ranges address semantic DisplayItems, independent of replay op counts.
use layoutng_assembly::fragment_tree::RasterEffectOutset;
use paint_model::display_item_id::DisplayItemId;
use paint_model::paint_property_tree::PropertyTreeState;
use paint_model::PaintRect;

pub use crate::DisplayItemClientInfo;
use crate::{
    PaintChunk, PaintChunkBackgroundColorInfo, PaintChunkCompositorData, PaintHitTestData,
    PaintHitTestOpaqueness, PaintSelectionBound, PaintTrackedElementRect,
    PaintTrackedElementSubRect, K_MIN_BACKGROUND_COLOR_COVERAGE_RATIO,
};

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

    pub fn current_effectively_invisible(&self) -> bool {
        self.current_effectively_invisible
    }

    pub fn force_new_chunk(&mut self) {
        self.will_force_new_chunk = true;
        self.next_chunk_id = None;
    }

    /// Source: `paint_chunker.cc:52-57` and the cached-subsequence move
    /// constructor in `paint_chunk.h:60-82`.
    pub fn AppendByMoving(&mut self, chunks: &mut Vec<PaintChunk>, mut chunk: PaintChunk) {
        self.FinalizeLastChunkProperties(chunks);
        let begin_index = chunks.last().map_or(0, |last| last.end_index);
        let size = chunk.size();
        chunk.begin_index = begin_index;
        chunk.end_index = begin_index + size;
        chunk.client_is_just_created = false;
        chunk.is_moved_from_cached_subsequence = true;
        chunks.push(chunk);
    }

    // Source: `paint_chunker.h:115-118`.
    pub fn EnsureChunk(&mut self, chunks: &mut Vec<PaintChunk>) -> bool {
        let (id, client) = self
            .next_chunk_id
            .expect("EnsureChunk requires a pending scoped chunk id");
        self.ensure_current_chunk(chunks, id, client)
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
            compositor: Some(Box::new(PaintChunkCompositorData::new())),
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

    // Source: `paint_chunker.cc:91-112,138-150`. An empty record still contributes its
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
        let forces_new_chunk = crate::ForcesNewChunk(id.r#type);
        if forces_new_chunk {
            self.force_new_chunk();
        }
        let created = self.ensure_current_chunk(chunks, id, client);
        chunks.last_mut().unwrap().end_index += 1;
        // Chromium treats every ordinary display item's visual rect as
        // transparent to hit testing while extending the chunk bounds.  This
        // must go through UnionBounds: a drawing outside previously recorded
        // opaque hit-test data makes the enlarged chunk mixed rather than
        // allowing the opaque region to cover the un-hit-testable gap.
        self.UnionBounds(
            chunks,
            visual_rect,
            PaintHitTestOpaqueness::Transparent,
        );
        let chunk = chunks.last_mut().unwrap();
        chunk.bounds_are_complete &= visual_rect_is_accurate;
        if draws_content {
            chunk.drawable_bounds.union(visual_rect);
        }
        // Source: `paint_chunker.cc:110-112`: never union across unpainted gaps.
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

    // Source: `paint_chunker.cc:152-194`.
    pub fn AddHitTestDataToCurrentChunk(
        &mut self,
        chunks: &mut Vec<PaintChunk>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        rect: PaintRect,
        touch_action: foundation::TouchAction,
        blocking_wheel: bool,
        opaqueness: PaintHitTestOpaqueness,
    ) -> bool {
        let created = self.ensure_current_chunk(chunks, id, client);
        self.UnionBounds(chunks, rect, opaqueness);
        let chunk = chunks.last_mut().expect("hit-test chunk");
        if touch_action != foundation::TouchAction::kAuto {
            let touch_action_rects = &mut chunk.EnsureHitTestData().touch_action_rects;
            let bits = touch_action.bits() as u32;
            if touch_action_rects
                .last()
                .is_none_or(|(last_rect, last_bits)| {
                    !last_rect.contains(rect) || *last_bits != bits
                })
            {
                touch_action_rects.push((rect, bits));
            }
        }
        if blocking_wheel {
            let wheel_event_rects = &mut chunk.EnsureHitTestData().wheel_event_rects;
            if wheel_event_rects
                .last()
                .is_none_or(|last_rect| !last_rect.contains(rect))
            {
                wheel_event_rects.push(rect);
            }
        }
        created
    }

    // Source: `paint_chunker.cc:196-204`.
    pub fn CurrentChunkIsNonEmptyAndTransparentToHitTest(&self, chunks: &[PaintChunk]) -> bool {
        if self.will_force_new_chunk
            || chunks
                .last()
                .is_none_or(|last| !last.properties.same_nodes(&self.current_properties))
        {
            return false;
        }
        let chunk = chunks.last().expect("current chunk");
        !chunk.bounds.is_empty()
            && chunk.compositor.as_deref().is_some_and(|metadata| {
                metadata.hit_test_opaqueness == PaintHitTestOpaqueness::Transparent
            })
    }

    // Source: `paint_chunker.cc:206-220`.
    pub fn AddRegionCaptureDataToCurrentChunk(
        &mut self,
        chunks: &mut Vec<PaintChunk>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        crop_id: [u64; 2],
        rect: PaintRect,
    ) -> bool {
        debug_assert_ne!(crop_id, [0; 2]);
        let created = self.ensure_current_chunk(chunks, id, client);
        let metadata = chunks
            .last_mut()
            .expect("region-capture chunk")
            .compositor
            .as_deref_mut()
            .expect("region-capture compositor metadata");
        let data = metadata
            .region_capture_data
            .get_or_insert_with(|| Box::new(Vec::new()));
        if let Some(entry) = data.iter_mut().find(|(old_id, _)| *old_id == crop_id) {
            entry.1 = rect;
        } else {
            data.push((crop_id, rect));
            data.sort_by_key(|(id, _)| *id);
        }
        created
    }

    // Source: `paint_chunker.cc:222-244`.
    pub fn AddTrackedElementDataToCurrentChunk(
        &mut self,
        chunks: &mut Vec<PaintChunk>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        element_paint_rect: PaintRect,
        tracked_element_sub_rects: &[PaintTrackedElementSubRect],
    ) -> bool {
        let created = self.ensure_current_chunk(chunks, id, client);
        let metadata = chunks
            .last_mut()
            .expect("tracked-element chunk")
            .compositor
            .as_deref_mut()
            .expect("tracked-element compositor metadata");
        let data = metadata
            .tracked_element_rects
            .get_or_insert_with(|| Box::new(Vec::new()));
        for sub_rect in tracked_element_sub_rects {
            debug_assert_ne!(sub_rect.id, [0; 2]);
            data.push(PaintTrackedElementRect {
                feature: sub_rect.feature,
                rect: sub_rect.GetEffectiveBounds(element_paint_rect),
                id: sub_rect.id,
                should_add_to_compositor_frame_metadata: sub_rect
                    .should_add_to_compositor_frame_metadata,
                should_exclude_fixed_and_sticky_occlusions: sub_rect
                    .should_exclude_fixed_and_sticky_occlusions,
                frame_token: sub_rect.frame_token,
                parent_frame_token: sub_rect.parent_frame_token,
            });
        }
        created
    }

    // Source: `paint_chunker.cc:246-289`.
    pub fn AddSelectionToCurrentChunk(
        &mut self,
        chunks: &mut [PaintChunk],
        start: Option<PaintSelectionBound>,
        end: Option<PaintSelectionBound>,
    ) {
        let selection = chunks
            .last_mut()
            .expect("selection must have painted a current chunk")
            .EnsureLayerSelectionData();
        if let Some(start) = start {
            debug_assert!(selection.start.is_none());
            selection.start = Some(start);
        }
        if let Some(end) = end {
            debug_assert!(selection.end.is_none());
            selection.end = Some(end);
        }
    }

    // Source: `paint_chunker.cc:291-298`.
    pub fn RecordAnySelectionWasPainted(&mut self, chunks: &mut [PaintChunk]) {
        chunks
            .last_mut()
            .expect("selection must have painted a current chunk")
            .EnsureLayerSelectionData()
            .any_selection_was_painted = true;
    }

    // Source: `paint_chunker.cc:300-339`. Scroll hit-test data is a metadata-only
    // paint chunk in the scroller's border-box property state. It deliberately
    // owns no DisplayItem; PAC consumes the typed hit-test payload directly.
    pub fn CreateScrollHitTestChunk(
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
        self.UnionBounds(chunks, rect, opaqueness);
        *chunks
            .last_mut()
            .expect("scroll hit-test chunk")
            .EnsureHitTestData() = data;
        self.force_new_chunk();
    }

    // Source: `paint_chunker.cc:341-348`.
    fn UnionBounds(
        &mut self,
        chunks: &mut [PaintChunk],
        rect: PaintRect,
        opaqueness: PaintHitTestOpaqueness,
    ) {
        let chunk = chunks.last_mut().expect("current chunk");
        let metadata = chunk
            .compositor
            .as_deref_mut()
            .expect("current chunk compositor metadata");
        metadata.hit_test_opaqueness =
            UnionHitTestOpaqueness(chunk.bounds, metadata.hit_test_opaqueness, rect, opaqueness);
        chunk.bounds.union(rect);
    }

    // Source: `paint_chunker.cc:110-123,350-388`. The record analysis result comes from
    // its actual producer; the canonical chunk retains its own derived fields.
    pub fn UpdateDrawingDerivedData(
        &mut self,
        chunks: &mut [PaintChunk],
        candidate: PaintChunkBackgroundColorInfo,
        has_text: bool,
        visual_rect: PaintRect,
    ) {
        let chunk = chunks.last_mut().expect("source current chunk");
        let m = chunk
            .compositor
            .as_deref_mut()
            .expect("source source metadata");
        let min_area = K_MIN_BACKGROUND_COLOR_COVERAGE_RATIO
            * chunk.bounds.width as f32
            * chunk.bounds.height as f32;
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
            .expect("source source metadata");
        if chunk.end_index - chunk.begin_index > 1
            || m.background_area != chunk.bounds.width as f32 * chunk.bounds.height as f32
        {
            m.background_is_solid_color = false;
        }
    }
    // Source: `paint_chunker.cc:15-16`.
    pub fn Finish(&mut self, chunks: &mut [PaintChunk]) {
        self.FinalizeLastChunkProperties(chunks);
    }
}

// Source primitive: `cc/input/hit_test_opaqueness.cc:15-17,27-57`.
fn UnionHitTestOpaqueness(
    rect1: PaintRect,
    opaqueness1: PaintHitTestOpaqueness,
    rect2: PaintRect,
    opaqueness2: PaintHitTestOpaqueness,
) -> PaintHitTestOpaqueness {
    if rect1.is_empty() {
        return opaqueness2;
    }
    if rect2.is_empty() {
        return opaqueness1;
    }
    let rank = |opaqueness| match opaqueness {
        PaintHitTestOpaqueness::Transparent => 0,
        PaintHitTestOpaqueness::Mixed => 1,
        PaintHitTestOpaqueness::Opaque => 2,
    };
    if rank(opaqueness1) < rank(opaqueness2) {
        return UnionHitTestOpaqueness(rect2, opaqueness2, rect1, opaqueness1);
    }
    match opaqueness1 {
        PaintHitTestOpaqueness::Transparent => PaintHitTestOpaqueness::Transparent,
        PaintHitTestOpaqueness::Mixed => PaintHitTestOpaqueness::Mixed,
        PaintHitTestOpaqueness::Opaque => {
            if opaqueness2 == PaintHitTestOpaqueness::Opaque {
                let mut union = rect1;
                union.union(rect2);
                if union == PaintRect::maximum_covered_rect(rect1, rect2) {
                    PaintHitTestOpaqueness::Opaque
                } else {
                    PaintHitTestOpaqueness::Mixed
                }
            } else if rect1.contains(rect2) {
                PaintHitTestOpaqueness::Opaque
            } else {
                PaintHitTestOpaqueness::Mixed
            }
        }
    }
}

// Source primitive: ui/gfx/color_utils.cc:577-614. Quantize to source SkColor,
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
    use paint_model::display_item_id::DisplayItemIdType;
    use paint_model::paint_property_tree::TransformPaintPropertyNode;
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
            direct_compositing_reasons: Default::default(),
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
        assert!(chunks[2].Matches(&chunks[2]));
        let mut created = chunks[2].clone();
        created.client_is_just_created = true;
        assert!(!created.Matches(&chunks[2]));
        let mut uncacheable = chunks[2].clone();
        uncacheable.is_cacheable = false;
        assert!(!chunks[2].Matches(&uncacheable));
    }

    #[test]
    fn translated_chunk_metadata_paths_share_the_current_chunk() {
        let client = DisplayItemClientInfo {
            is_cacheable: true,
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_just_created: false,
        };
        let hit_test_id = id(44, DisplayItemIdType::kHitTest, 0);
        let outer = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
        };
        let inner = PaintRect {
            x: 2.0,
            y: 2.0,
            width: 4.0,
            height: 4.0,
        };
        let separate = PaintRect { x: 30.0, ..inner };
        let mut chunker = PaintChunker::default();
        let mut chunks = Vec::new();
        assert!(chunker.AddHitTestDataToCurrentChunk(
            &mut chunks,
            hit_test_id,
            client,
            outer,
            foundation::TouchAction::kPanX,
            true,
            PaintHitTestOpaqueness::Opaque,
        ));
        assert!(!chunker.increment_display_item(
            &mut chunks,
            id(45, DisplayItemIdType::kBoxDecorationBackground, 0),
            client,
            separate,
            true,
            RasterEffectOutset::kNone,
            true,
            PaintRect::default(),
        ));
        assert_eq!(
            chunks[0].compositor.as_ref().unwrap().hit_test_opaqueness,
            PaintHitTestOpaqueness::Mixed
        );
        assert!(!chunker.AddHitTestDataToCurrentChunk(
            &mut chunks,
            hit_test_id,
            client,
            inner,
            foundation::TouchAction::kPanX,
            true,
            PaintHitTestOpaqueness::Transparent,
        ));
        assert_eq!(chunks[0].EnsureHitTestData().touch_action_rects.len(), 1);
        assert_eq!(chunks[0].EnsureHitTestData().wheel_event_rects.len(), 1);
        assert!(chunker.CurrentChunkIsNonEmptyAndTransparentToHitTest(&chunks) == false);
        assert!(!chunker.AddHitTestDataToCurrentChunk(
            &mut chunks,
            hit_test_id,
            client,
            separate,
            foundation::TouchAction::kAuto,
            false,
            PaintHitTestOpaqueness::Transparent,
        ));
        assert_eq!(
            chunks[0].compositor.as_ref().unwrap().hit_test_opaqueness,
            PaintHitTestOpaqueness::Mixed
        );

        let capture_id = id(44, DisplayItemIdType::kRegionCapture, 0);
        assert!(!chunker.AddRegionCaptureDataToCurrentChunk(
            &mut chunks,
            capture_id,
            client,
            [1, 2],
            inner,
        ));
        assert!(!chunker.AddRegionCaptureDataToCurrentChunk(
            &mut chunks,
            capture_id,
            client,
            [1, 2],
            separate,
        ));
        assert_eq!(
            chunks[0]
                .compositor
                .as_ref()
                .unwrap()
                .region_capture_data
                .as_ref()
                .unwrap()
                .as_slice(),
            &[([1, 2], separate)]
        );
    }
}
