#![allow(non_snake_case)]

use foundation::TrustedIdentityHashMap as HashMap;
use layoutng_assembly::fragment_tree::PaintGlyphRun;
use layoutng_assembly::internal::paint_input::PaintBlendMode;
use layoutng_assembly::layout_engine::NativePaintFragmentCursor;
use std::sync::Arc;

use crate::border_shape_utils::UniformCornerRadius;
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::paint_chunker::{DisplayItemClientInfo, PaintChunker};
use crate::paint_engine::{
    DisplayItem, DisplayItemType, PaintPhase, RasterEffectOutset, RecordedDisplayItem,
    RecordedDisplayItemKind,
};
use crate::paint_property_tree::PropertyTreeState;
use crate::pre_paint_tree_walk::{
    EnclosingRectInPropertySpace, MaskAdjustedOpacity, PaintTreeNode,
};
use crate::PaintRecording;
use crate::PaintRect;
use layoutng_assembly::internal::layout_input::Offset;

// Source: drawing_display_item.cc:64-81,116-157. This is calculated once at
// the real record producer, retaining the native flags/color/visual bounds.
fn DrawingBackgroundColor(
    id: DisplayItemId,
    visual: PaintRect,
    record: &[DisplayItem],
) -> crate::paint_engine::PaintChunkBackgroundColorInfo {
    use crate::paint_engine::PaintChunkBackgroundColorInfo;
    let empty = PaintChunkBackgroundColorInfo::default();
    if record.is_empty() {
        return empty;
    }
    let mut solid = record.len() == 1;
    for op in record {
        let is_flags = (op.r#type as u8) >= DisplayItemType::kDrawRect as u8;
        let fill = !matches!(
            op.r#type,
            DisplayItemType::kStrokeEllipse
                | DisplayItemType::kStrokePath
                | DisplayItemType::kStrokeRect
                | DisplayItemType::kStrokeLine
                | DisplayItemType::kStrokeWavyLine
        );
        if !is_flags
            || !fill
            || op.is_shadow
            || op.blur_radius != 0.
            || !op.filters.is_empty()
            || op.paint_shader.is_some()
            || op.blend_mode != PaintBlendMode::kNormal
        {
            if ![
                DisplayItemIdType::kBoxDecorationBackground,
                DisplayItemIdType::kDocumentBackground,
                DisplayItemIdType::kDocumentRootBackdrop,
                DisplayItemIdType::kScrollCorner,
            ]
            .contains(&id.r#type)
            {
                return empty;
            }
            continue;
        }
        let r = match op.r#type {
            DisplayItemType::kDrawRect => op.rect,
            DisplayItemType::kDrawRoundedRect => {
                solid = false;
                op.rect
            }
            _ => return empty,
        };
        let mut value = PaintChunkBackgroundColorInfo::default();
        value.background_color = op.color;
        value.background_color.alpha *= op.opacity;
        value.background_area = r.width as f32 * r.height as f32;
        value.background_is_solid_color = solid
            && r.x <= visual.x
            && r.y <= visual.y
            && r.x + r.width >= visual.x + visual.width
            && r.y + r.height >= visual.y + visual.height;
        return value;
    }
    empty
}

// drawing_display_item.h:70-98 and drawing_display_item.cc:64-81,285-320.
// Only one fill rect/rrect op without a shader or pixel-moving paint flags
// qualifies. Images, text, strokes and multi-op records retain painter ink bounds.
fn TightenDrawingVisualRect(
    visual_rect: PaintRect,
    record: &[DisplayItem],
    delta: Offset,
) -> (PaintRect, bool) {
    if let [op] = record {
        if matches!(
            op.r#type,
            DisplayItemType::kDrawRect | DisplayItemType::kDrawRoundedRect
        ) && op.paint_shader.is_none()
            && op.filters.is_empty()
            && op.blend_mode == PaintBlendMode::kNormal
        {
            return (EnclosingRectInPropertySpace(op.rect, delta), true);
        }
        // SkCanvas image draws cannot write outside their destination/tile
        // rectangle unless an image filter, shadow or blur expands the op.
        // Treating ordinary DrawImage records as unknown made LayerTile fall
        // back to the whole viewport for each animated image layer.
        if matches!(
            op.r#type,
            DisplayItemType::kDrawImageRect | DisplayItemType::kDrawTiledImage
        ) && op.filters.is_empty()
            && !op.is_shadow
            && op.blur_radius == 0.0
        {
            return (EnclosingRectInPropertySpace(op.rect, delta), true);
        }
    }
    (EnclosingRectInPropertySpace(visual_rect, delta), false)
}

// drawing_display_item.cc:172-282. Keep this bounded and independent of image
// pixel contents. The standalone ABI has no certified shader/image opacity,
// so only solid fill rectangles supply a new proof. Unknown state is rejected.
pub fn CalculateRectKnownToBeOpaqueForRecord(
    visual_rect: PaintRect,
    record: &[DisplayItem],
) -> PaintRect {
    const OP_COUNT_LIMIT: usize = 8;
    if record.is_empty() {
        return PaintRect::default();
    }
    let mut opaque = PaintRect::default();
    let mut clip = visual_rect.enclosed();
    for op in record.iter().take(OP_COUNT_LIMIT) {
        match op.r#type {
            DisplayItemType::kSave => continue,
            DisplayItemType::kClipRect => {
                clip = clip.intersection(op.rect.enclosed());
                continue;
            }
            // Source drawing_display_item.cc stops when the bounded prefix
            // reaches any other non-draw operation, including Restore.
            kind if (kind as u8) < DisplayItemType::kDrawRect as u8 => break,
            _ => {}
        }
        if op.blend_mode != PaintBlendMode::kNormal
            || !op.filters.is_empty()
            || !op.mask_layers.is_empty()
            || op.r#type == DisplayItemType::kDrawMask
        {
            continue;
        }
        if op.paint_shader.is_some()
            || op.color.alpha != 1.0
            || op.opacity != 1.0
            || op.is_shadow
            || op.blur_radius != 0.0
        {
            continue;
        }
        let rect = match op.r#type {
            DisplayItemType::kDrawRect => op.rect.enclosed(),
            DisplayItemType::kDrawRoundedRect => {
                let r = op.corner_radii;
                if [
                    r.top_left.x,
                    r.top_left.y,
                    r.top_right.x,
                    r.top_right.y,
                    r.bottom_left.x,
                    r.bottom_left.y,
                    r.bottom_right.x,
                    r.bottom_right.y,
                ]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0)
                {
                    continue;
                }
                // The contained rectangle excludes all four rounded corners.
                // Chromium uses .3, conservatively above (2 - sqrt(2)) / 2.
                let left = r.top_left.x.max(r.bottom_left.x) * 0.3;
                let top = r.top_left.y.max(r.top_right.y) * 0.3;
                let right = r.top_right.x.max(r.bottom_right.x) * 0.3;
                let bottom = r.bottom_left.y.max(r.bottom_right.y) * 0.3;
                PaintRect {
                    x: op.rect.x + left,
                    y: op.rect.y + top,
                    width: op.rect.width - left - right,
                    height: op.rect.height - top - bottom,
                }
                .enclosed()
            }
            _ => continue,
        };
        opaque = PaintRect::maximum_covered_rect(opaque, rect).intersection(clip);
        if opaque == visual_rect.enclosed() {
            break;
        }
    }
    opaque
}

// cpp: paint/paint_context.h:8-26
pub struct PaintContext<'o> {
    output: &'o mut PaintRecording,
    previous: Option<Arc<PaintRecording>>,
    cached_items: HashMap<DisplayItemId, Vec<usize>>,
    /// Borrowed native text sources exist only for this synchronous Paint
    /// recording. The committed PaintArtifact retains final immutable drawing
    /// payloads, never LayoutResult or FragmentItem pointers.
    native_text_sources: HashMap<u64, NativePaintFragmentCursor<'o>>,
    /// Multiple paint phases and decorations for one FragmentItem share the
    /// single payload built from ShapeResult during this recording.
    native_text_runs: HashMap<u64, Arc<Vec<PaintGlyphRun>>>,
    pub cached_display_items: usize,
    /// PaintController-style cache diagnostics. These are traced on
    /// Paint.RecordPaint so a performance capture can distinguish invalidated
    /// clients from unstable DisplayItem identities without changing cache
    /// behavior.
    pub cache_candidates: usize,
    pub cache_attempts: usize,
    pub cache_rejected_uncacheable: usize,
    pub cache_rejected_just_created: usize,
    pub cache_rejected_invalid: usize,
    pub cache_rejected_geometry: usize,
    pub cache_rejected_missing_id: usize,
    pub cache_rejected_missing_previous: usize,
    pub walk_metrics: paint_model::PaintWalkMetrics,
    pub caret: Option<crate::paint_engine::CaretPosition>,
    pub document_selection: Option<layoutng_assembly::editing_state::DocumentSelection>,
    pub caret_local_rect: Option<layoutng_assembly::caret::geometry::LocalCaretRect>,
    /// Editing ranges resolved through the current native FragmentItem links
    /// before PrePaint retains its derived tree. Empty during normal page
    /// painting, so resize does no document-wide caret projection.
    pub text_range_rects: Vec<(
        u64,
        (u32, u32),
        layoutng_assembly::caret::geometry::LocalCaretRect,
    )>,
    chunker: PaintChunker,
    drawing_recorder: Option<(
        DisplayItemId,
        DisplayItemClientInfo,
        PropertyTreeState,
        PaintRect,
        usize,
        RecordedDisplayItemKind,
        Option<crate::paint_engine::ScrollbarDisplayItemData>,
        Option<std::sync::Arc<crate::paint_property_tree::TransformPaintPropertyNode>>,
        RasterEffectOutset,
        bool,
        Offset,
        Option<usize>,
    )>,
    next_property_id: u64,
}

impl<'o> PaintContext<'o> {
    pub fn Finish(&mut self) {
        self.chunker.Finish(&mut self.output.chunks);
    }
    // cpp: paint/paint_context.h:10
    pub fn new(output: &'o mut PaintRecording) -> Self {
        Self {
            output,
            previous: None,
            cached_items: HashMap::default(),
            native_text_sources: HashMap::default(),
            native_text_runs: HashMap::default(),
            cached_display_items: 0,
            cache_candidates: 0,
            cache_attempts: 0,
            cache_rejected_uncacheable: 0,
            cache_rejected_just_created: 0,
            cache_rejected_invalid: 0,
            cache_rejected_geometry: 0,
            cache_rejected_missing_id: 0,
            cache_rejected_missing_previous: 0,
            walk_metrics: Default::default(),
            caret: None,
            document_selection: None,
            caret_local_rect: None,
            text_range_rects: Vec::new(),
            chunker: PaintChunker::default(),
            drawing_recorder: None,
            next_property_id: 1,
        }
    }

    pub fn WithEditing(
        output: &'o mut PaintRecording,
        editing: crate::paint_engine::EditingPaintState,
    ) -> Self {
        Self::WithEditingAndCache(output, editing, None)
    }

    pub fn WithEditingAndCache(
        output: &'o mut PaintRecording,
        editing: crate::paint_engine::EditingPaintState,
        previous: Option<Arc<PaintRecording>>,
    ) -> Self {
        let mut cached_items: HashMap<DisplayItemId, Vec<usize>> = HashMap::default();
        if let Some(artifact) = &previous {
            for (index, item) in artifact.display_items.iter().enumerate() {
                if item.is_cacheable
                    && !item.is_tombstone
                    && item.kind == RecordedDisplayItemKind::Drawing
                {
                    cached_items.entry(item.id).or_default().push(index);
                }
            }
            for indices in cached_items.values_mut() {
                indices.reverse();
            }
        }
        let cache_candidates = cached_items.values().map(Vec::len).sum();
        Self {
            output,
            previous,
            cached_items,
            native_text_sources: HashMap::default(),
            native_text_runs: HashMap::default(),
            cached_display_items: 0,
            cache_candidates,
            cache_attempts: 0,
            cache_rejected_uncacheable: 0,
            cache_rejected_just_created: 0,
            cache_rejected_invalid: 0,
            cache_rejected_geometry: 0,
            cache_rejected_missing_id: 0,
            cache_rejected_missing_previous: 0,
            walk_metrics: Default::default(),
            caret: editing.caret,
            document_selection: editing.document_selection,
            caret_local_rect: None,
            text_range_rects: Vec::new(),
            chunker: PaintChunker::default(),
            drawing_recorder: None,
            next_property_id: 1,
        }
    }

    pub fn WithCaret(
        output: &'o mut PaintRecording,
        caret: Option<crate::paint_engine::CaretPosition>,
    ) -> Self {
        Self::WithEditing(
            output,
            crate::paint_engine::EditingPaintState {
                caret,
                document_selection: None,
            },
        )
    }
    pub fn SetCaretGeometry(&mut self, geometry: crate::paint_engine::CaretGeometry) {
        self.output.caret = Some(geometry);
    }

    /// Index native FragmentItem cursors without copying their shape or child
    /// topology. The cursor lifetime is deliberately tied to this PaintContext
    /// and therefore cannot escape into the committed artifact.
    pub fn BindNativeTextSources(&mut self, root: NativePaintFragmentCursor<'o>) {
        fn collect<'a>(
            cursor: NativePaintFragmentCursor<'a>,
            output: &mut HashMap<u64, NativePaintFragmentCursor<'a>>,
        ) {
            if matches!(
                cursor.kind,
                layoutng_assembly::fragment_tree::FragmentKind::kText
                    | layoutng_assembly::fragment_tree::FragmentKind::kGeneratedText
            ) {
                output.insert(cursor.fragment_instance_id, cursor);
            }
            for child in cursor.Children() {
                collect(child, output);
            }
        }
        collect(root, &mut self.native_text_sources);
    }

    pub fn HasNativeTextSource(&self, fragment_instance_id: u64) -> bool {
        self.native_text_sources.contains_key(&fragment_instance_id)
    }

    /// Resolve ShapeResult only after a cache miss has committed Paint to
    /// recording this text item. Later phases share the same immutable owner.
    pub fn TextGlyphRuns(&mut self, fragment_instance_id: u64) -> Arc<Vec<PaintGlyphRun>> {
        if let Some(runs) = self.native_text_runs.get(&fragment_instance_id) {
            return Arc::clone(runs);
        }
        let runs = self
            .native_text_sources
            .get(&fragment_instance_id)
            .copied()
            .and_then(NativePaintFragmentCursor::PaintGlyphRuns)
            .unwrap_or_else(|| Arc::new(Vec::new()));
        self.native_text_runs
            .insert(fragment_instance_id, Arc::clone(&runs));
        runs
    }

    // paint_controller.cc:UseCachedItemIfPossible. A valid cacheable client
    // moves its old semantic DrawingDisplayItem into the new artifact while
    // the current PrePaint property state is retained by the new PaintChunk.
    pub fn UseCachedDrawingIfPossible(
        &mut self,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
        client_visual_rect: PaintRect,
    ) -> bool {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        let id = DisplayItemId {
            client_id: fragment.paint.display_item_client_id,
            r#type: item_type,
            fragment: fragment.paint.display_item_fragment,
        };
        self.UseCachedDrawingWithClient(
            id,
            DisplayItemClientInfo::from_fragment(fragment),
            Some((
                fragment.paint.display_item_client_owner_node_id,
                fragment.paint.display_item_client_debug_name.clone(),
            )),
            Some(client_visual_rect),
        )
    }

    pub fn UseCachedDrawingWithClient(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        debug: Option<(i32, Arc<str>)>,
        client_visual_rect: Option<PaintRect>,
    ) -> bool {
        self.cache_attempts += 1;
        let invalidated = !matches!(
            client.invalidation_reason,
            Some(foundation::PaintInvalidationReason::kNone)
        );
        if paint_reuse::DecideDisplayItemReuse(
            true,
            client.is_cacheable,
            client.is_just_created,
            invalidated,
            true,
        ) == paint_reuse::CacheDecision::Record
        {
            if !client.is_cacheable {
                self.cache_rejected_uncacheable += 1;
            } else if client.is_just_created {
                self.cache_rejected_just_created += 1;
            } else if invalidated {
                self.cache_rejected_invalid += 1;
            }
            return false;
        }
        let Some(index) = self
            .cached_items
            .get_mut(&id)
            .and_then(|indices| indices.pop())
        else {
            self.cache_rejected_missing_id += 1;
            return false;
        };
        let Some(previous) = &self.previous else {
            self.cache_rejected_missing_previous += 1;
            return false;
        };
        let old = &previous.display_items[index];
        let geometry_matches = client_visual_rect.is_none_or(|rect| rect == old.client_visual_rect);
        if paint_reuse::DecideDisplayItemReuse(
            true,
            client.is_cacheable,
            client.is_just_created,
            invalidated,
            geometry_matches,
        ) == paint_reuse::CacheDecision::Record
        {
            self.cache_rejected_geometry += 1;
            return false;
        }
        let old_record = &previous.items[old.record_begin..old.record_end];
        let record_begin = self.output.items.len();
        Arc::make_mut(&mut self.output.items).extend_from_slice(old_record);
        let record_end = self.output.items.len();
        if let Some((owner, name)) = debug {
            self.output
                .client_owner_node_ids
                .insert(id.client_id as usize, owner);
            self.output
                .client_debug_names
                .insert(id.client_id as usize, name);
        }
        self.chunker.increment_display_item(
            &mut self.output.chunks,
            id,
            client,
            old.visual_rect,
            old.draws_content,
            old.raster_effect_outset,
            old.visual_rect_is_accurate,
            old.record_rect_known_to_be_opaque,
        );
        if old.draws_content {
            self.chunker.UpdateDrawingDerivedData(
                &mut self.output.chunks,
                old.record_background,
                old.record_has_text,
                old.visual_rect,
            );
        }
        let mut moved = old.clone();
        moved.paint_invalidation_reason = Some(foundation::PaintInvalidationReason::kNone);
        moved.record_begin = record_begin;
        moved.record_end = record_end;
        self.output.display_items.push(moved);
        self.cached_display_items += 1;
        true
    }

    // cpp: paint/paint_context.h:22
    // cpp: paint/paint_context.cc:60-80
    pub fn Append(&mut self, mut item: DisplayItem, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        item.fragment_instance_id = fragment.fragment_instance_id;
        // Blink records save/restore/clip inside PaintRecords, not as drawing
        // DisplayItems. They must neither consume a pending scope ID nor create
        // a fallback identity. The existing flat replay ABI retains the ops.
        if self.drawing_recorder.is_some()
            || (item.r#type as u8) < (DisplayItemType::kDrawRect as u8)
        {
            std::sync::Arc::make_mut(&mut self.output.items).push(item);
            return;
        }
        // Blink's fallback ID is a semantic DisplayItem ID, not a Skia opcode.
        // The standalone stream remains flat; scopes supply the outer ID.
        let id = DisplayItemId {
            client_id: fragment.paint.display_item_client_id,
            r#type: DisplayItemIdType::PaintPhaseToDrawingType(item.phase),
            fragment: fragment.paint.display_item_fragment,
        };
        let client = DisplayItemClientInfo::from_fragment(fragment);
        self.output.client_owner_node_ids.insert(
            id.client_id as usize,
            fragment.paint.display_item_client_owner_node_id,
        );
        self.output.client_debug_names.insert(
            id.client_id as usize,
            fragment.paint.display_item_client_debug_name.clone(),
        );
        let (visual_rect, visual_rect_is_accurate) = TightenDrawingVisualRect(
            item.rect,
            std::slice::from_ref(&item),
            node.paint_snap_offset,
        );
        let raster_effect_outset = fragment.paint.display_item_raster_effect_outset;
        let rect_known_to_be_opaque =
            CalculateRectKnownToBeOpaqueForRecord(visual_rect, std::slice::from_ref(&item));
        self.chunker.increment_display_item(
            &mut self.output.chunks,
            id,
            client,
            visual_rect,
            true,
            raster_effect_outset,
            visual_rect_is_accurate,
            rect_known_to_be_opaque,
        );
        let record_background =
            DrawingBackgroundColor(id, visual_rect, std::slice::from_ref(&item));
        let record_has_text = item.r#type == DisplayItemType::kDrawGlyphRun;
        self.chunker.UpdateDrawingDerivedData(
            &mut self.output.chunks,
            record_background,
            record_has_text,
            visual_rect,
        );
        self.output.display_items.push(RecordedDisplayItem {
            is_cacheable: client.is_cacheable,
            is_tombstone: false,
            paint_invalidation_reason: client.invalidation_reason,
            image_content_revisions: Vec::new(),
            scrollbar_needs_update_display: client.scrollbar_needs_update_display,
            scrollbar: None,
            kind: RecordedDisplayItemKind::Drawing,
            scroll_translation: None,
            id,
            client_visual_rect: item.rect,
            visual_rect,
            visual_rect_is_accurate,
            draws_content: true,
            raster_effect_outset,
            record_rect_known_to_be_opaque: rect_known_to_be_opaque,
            record_background,
            record_has_text,
            record_begin: self.output.items.len(),
            record_end: self.output.items.len() + 1,
        });
        std::sync::Arc::make_mut(&mut self.output.items).push(item);
    }

    pub fn InDrawingRecorder(&self) -> bool {
        self.drawing_recorder.is_some()
    }

    pub fn BeginDrawingRecorder(
        &mut self,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
        visual_rect: PaintRect,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        self.output.client_owner_node_ids.insert(
            fragment.paint.display_item_client_id as usize,
            fragment.paint.display_item_client_owner_node_id,
        );
        self.output.client_debug_names.insert(
            fragment.paint.display_item_client_id as usize,
            fragment.paint.display_item_client_debug_name.clone(),
        );
        self.BeginDrawingRecorderWithClient(
            DisplayItemId {
                client_id: fragment.paint.display_item_client_id,
                r#type: item_type,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo::from_fragment(fragment),
            visual_rect,
            node.paint_snap_offset,
        );
        if let Some(recorder) = &mut self.drawing_recorder {
            recorder.8 = fragment.paint.display_item_raster_effect_outset;
            recorder.9 = fragment.paint.self_ink_overflow.is_some()
                || fragment.paint.box_self_visual_overflow.is_some();
        }
    }

    pub fn BeginDrawingRecorderWithClient(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        visual_rect: PaintRect,
        delta: Offset,
    ) {
        assert!(!self.InDrawingRecorder(), "DrawingRecorders cannot nest");
        self.drawing_recorder = Some((
            id,
            client,
            self.chunk_properties(),
            visual_rect,
            self.output.items.len(),
            RecordedDisplayItemKind::Drawing,
            None,
            None,
            RasterEffectOutset::kNone,
            true,
            delta,
            None,
        ));
    }

    pub fn BeginScrollbarDisplayItem(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        visual_rect: PaintRect,
        scroll_translation: std::sync::Arc<crate::paint_property_tree::TransformPaintPropertyNode>,
        scrollbar: crate::paint_engine::ScrollbarDisplayItemData,
        delta: Offset,
    ) {
        assert!(
            !self.InDrawingRecorder(),
            "ScrollbarDisplayItem is an outer item"
        );
        self.drawing_recorder = Some((
            id,
            client,
            self.chunk_properties(),
            visual_rect,
            self.output.items.len(),
            RecordedDisplayItemKind::Scrollbar,
            Some(scrollbar),
            Some(scroll_translation),
            RasterEffectOutset::kNone,
            true,
            delta,
            None,
        ));
    }

    pub fn EndDrawingRecorder(&mut self) {
        let (
            id,
            client,
            properties,
            visual_rect,
            record_begin,
            kind,
            scrollbar,
            scroll_translation,
            raster_effect_outset,
            visual_rect_is_accurate,
            delta,
            _cached_item,
        ) = self
            .drawing_recorder
            .take()
            .expect("DrawingRecorder must be active");
        let record_end = self.output.items.len();
        let client_visual_rect = visual_rect;
        let (mut visual_rect, tightened) = if kind == RecordedDisplayItemKind::Drawing {
            TightenDrawingVisualRect(
                visual_rect,
                &self.output.items[record_begin..record_end],
                delta,
            )
        } else {
            (EnclosingRectInPropertySpace(visual_rect, delta), false)
        };
        // Source: paint_chunks_to_cc_layer.cc:816-827. A reference filter can
        // produce pixels outside every source DrawingDisplayItem (or with no
        // source pixels at all). PendingLayer is sized from PaintChunk bounds
        // before PaintChunksToCcLayer records its synthetic empty visual op, so
        // retain that same producer-owned output bound on the resident chunk.
        // This prevents cc from allocating only the unfiltered source extent.
        if kind == RecordedDisplayItemKind::Drawing {
            if let Some(filtered) = properties.effect.compositor.filter_output_bounds {
                visual_rect.union(filtered);
            }
        }
        let visual_rect_is_accurate = visual_rect_is_accurate || tightened;
        // DrawingDisplayItem uses !record.empty(), including state ops.
        // ScrollbarDisplayItem is independently drawable, without a PaintRecord.
        let draws_content =
            kind == RecordedDisplayItemKind::Scrollbar || record_begin != record_end;
        self.chunker.update_properties(properties);
        let rect_known_to_be_opaque = if kind == RecordedDisplayItemKind::Drawing {
            CalculateRectKnownToBeOpaqueForRecord(
                visual_rect,
                &self.output.items[record_begin..record_end],
            )
        } else {
            PaintRect::default()
        };
        self.chunker.increment_display_item(
            &mut self.output.chunks,
            id,
            client,
            visual_rect,
            draws_content,
            raster_effect_outset,
            visual_rect_is_accurate,
            rect_known_to_be_opaque,
        );
        let mut record_background = Default::default();
        let mut record_has_text = false;
        if kind == RecordedDisplayItemKind::Drawing {
            let record = &self.output.items[record_begin..record_end];
            if draws_content {
                record_background = DrawingBackgroundColor(id, visual_rect, record);
                record_has_text = record
                    .iter()
                    .any(|op| op.r#type == DisplayItemType::kDrawGlyphRun);
                self.chunker.UpdateDrawingDerivedData(
                    &mut self.output.chunks,
                    record_background,
                    record_has_text,
                    visual_rect,
                );
            }
        }
        self.output.display_items.push(RecordedDisplayItem {
            is_cacheable: client.is_cacheable,
            is_tombstone: false,
            paint_invalidation_reason: client.invalidation_reason,
            image_content_revisions: Vec::new(),
            scrollbar_needs_update_display: client.scrollbar_needs_update_display,
            scrollbar,
            kind,
            scroll_translation,
            id,
            client_visual_rect,
            visual_rect,
            visual_rect_is_accurate,
            draws_content,
            raster_effect_outset,
            record_rect_known_to_be_opaque: rect_known_to_be_opaque,
            record_background,
            record_has_text,
            record_begin,
            record_end,
        });
    }

    // Source: paint_controller.cc:RecordScrollHitTestData and
    // paint_chunker.cc:RecordScrollHitTestData. The caller supplies the actual
    // border-box state and ScrollTranslation node from PrePaint.
    pub fn RecordScrollHitTestData(
        &mut self,
        node: &PaintTreeNode<'_>,
        rect: PaintRect,
        opaqueness: crate::paint_engine::PaintHitTestOpaqueness,
        scroll_translation: std::sync::Arc<crate::paint_property_tree::TransformPaintPropertyNode>,
        scrolling_contents_cull_rect: PaintRect,
    ) {
        let fragment = node.fragment.as_deref().expect("scroll hit-test fragment");
        let id = DisplayItemId {
            client_id: fragment.paint.display_item_client_id,
            r#type: DisplayItemIdType::kScrollHitTest,
            fragment: fragment.paint.display_item_fragment,
        };
        let client = DisplayItemClientInfo::from_fragment(fragment);
        self.output.client_owner_node_ids.insert(
            id.client_id as usize,
            fragment.paint.display_item_client_owner_node_id,
        );
        self.output.client_debug_names.insert(
            id.client_id as usize,
            fragment.paint.display_item_client_debug_name.clone(),
        );
        self.chunker.RecordScrollHitTestData(
            &mut self.output.chunks,
            id,
            client,
            rect,
            opaqueness,
            crate::paint_engine::PaintHitTestData {
                touch_action_rects: Vec::new(),
                wheel_event_rects: Vec::new(),
                xr_regions: Vec::new(),
                scroll_hit_test_rect: rect,
                scroll_translation: Some(scroll_translation),
                scrolling_contents_cull_rect,
            },
        );
    }

    pub fn SetNextPropertyId(&mut self, next: u64) {
        self.next_property_id = next;
    }

    pub fn AllocatePropertyId(&mut self) -> u64 {
        let id = self.next_property_id;
        self.next_property_id += 1;
        id
    }

    pub fn chunk_properties(&self) -> PropertyTreeState {
        self.chunker.current_properties().clone()
    }

    pub fn update_chunk_properties(&mut self, properties: PropertyTreeState) {
        self.chunker.update_properties(properties);
    }

    pub fn update_chunk_id_and_properties(
        &mut self,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        self.chunker.update_id_and_properties(
            DisplayItemId {
                client_id: fragment.paint.display_item_client_id,
                r#type: item_type,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo::from_fragment(fragment),
            node.properties.nodes.clone(),
        );
    }

    // cpp: paint/paint_context.h:12
    // cpp: paint/paint_context.cc:82-86
    pub fn BeginNode(&mut self, node: &PaintTreeNode<'_>) {
        self.BeginGeometry(node);
        self.BeginEffects(node);
        self.BeginEscapableGeometry(node, None);
    }

    // cpp: paint/paint_context.h:16
    // cpp: paint/paint_context.cc:88-130
    pub fn BeginGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.establishes_paint_state
            || (!fragment.paint.has_source && fragment.paint.text_combine.is_none())
        {
            return;
        }
        if node.applies_transform {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            for transform in &node.local_transforms {
                let mut item = DisplayItem {
                    r#type: DisplayItemType::kConcat,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                };
                item.transform = *transform;
                self.Append(item, node);
            }
        }
        if node.applies_root_clip {
            if let Some(root_clip) = node.root_clip {
                self.Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase: PaintPhase::kForeground,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    node,
                );
                self.Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipRect,
                        phase: PaintPhase::kForeground,
                        node_id: fragment.node_id,
                        rect: root_clip,
                        // paint_chunks_to_cc_layer.cc:589-595: property clips
                        // use AA for both plain and rounded rectangles.
                        antialias: true,
                        ..Default::default()
                    },
                    node,
                );
            }
        }
        if node.applies_clip_path && node.local_clip_path_children.len() <= 1 {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            let mut item = DisplayItem {
                r#type: DisplayItemType::kClipPath,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                antialias: true,
                ..Default::default()
            };
            item.path = node.local_clip_path.clone();
            item.even_odd = node.clip_path_even_odd;
            self.Append(item, node);
        }
    }

    // cpp: paint/paint_context.h:18-20
    // cpp: paint/paint_context.cc:132-169
    pub fn BeginEscapableGeometry(
        &mut self,
        node: &PaintTreeNode<'_>,
        clip_override: Option<PaintRect>,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source
            || !fragment.paint.establishes_paint_state
            || !node.applies_overflow_clip
            || node.local_clip.is_none()
        {
            return;
        }
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            },
            node,
        );
        let local_clip = node.local_clip.expect("overflow clip checked above");
        let clip = clip_override.unwrap_or(local_clip);
        let mut radii = node.local_clip_radii;
        if clip_override.is_some() && local_clip.width > 0.0 && local_clip.height > 0.0 {
            let scale_x = clip.width / local_clip.width;
            let scale_y = clip.height / local_clip.height;
            radii.top_left.x *= scale_x.abs();
            radii.top_left.y *= scale_y.abs();
            radii.top_right.x *= scale_x.abs();
            radii.top_right.y *= scale_y.abs();
            radii.bottom_right.x *= scale_x.abs();
            radii.bottom_right.y *= scale_y.abs();
            radii.bottom_left.x *= scale_x.abs();
            radii.bottom_left.y *= scale_y.abs();
        }
        let has_radius = radii.HasRadius();
        self.Append(
            DisplayItem {
                r#type: if has_radius {
                    DisplayItemType::kClipRoundedRect
                } else {
                    DisplayItemType::kClipRect
                },
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                rect: clip,
                corner_radius: UniformCornerRadius(&radii),
                corner_radii: radii,
                antialias: true,
                ..Default::default()
            },
            node,
        );
    }

    // cpp: paint/paint_context.h:14
    // cpp: paint/paint_context.cc:171-223
    pub fn BeginEffects(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source || !fragment.paint.establishes_paint_state {
            return;
        }
        let style = &*fragment.paint.style;
        if node.applies_compositing_layer {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayer,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_backdrop_filter
            && !node.applies_compositing_layer
            && !node.applies_blend
            && !node.applies_opacity
            && !node.applies_mask
        {
            // Backdrop execution belongs to the retained Effect node. This
            // redundant wrapper only keeps flat PaintRecord effect boundaries
            // aligned for PaintChunksToCcLayer lowering.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayer,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_blend {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    blend_mode: style.blend_mode,
                    opacity: if node.applies_opacity {
                        MaskAdjustedOpacity(style)
                    } else {
                        1.0
                    },
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_opacity && !node.applies_blend {
            // SkCanvas.h saveLayer bounds is optional. Chromium fills it from
            // all chunk drawable bounds after effect conversion (not this
            // owner's box). Until that union is available, omit the hint: a
            // scrolling child/OOF can extend beyond the owner, and SkCanvas's
            // current bounded device would otherwise discard real pixels.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerAlpha,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    opacity: MaskAdjustedOpacity(style),
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_mask {
            // Isolate the content Effect. Its mask is a later, independently
            // recorded source drawing under the child DstIn Mask effect.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayer,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            // Match the content-only MaskClip before background recording,
            // independently of the later overflow/scroll geometry scopes.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    rect: node.ResolvedMaskClipRect().expect("resolved MaskClip"),
                    antialias: true,
                    ..Default::default()
                },
                node,
            );
        }
        self.BeginCssClip(node);
        if node.applies_filter {
            let mut item = DisplayItem {
                r#type: DisplayItemType::kSaveLayerFilter,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            };
            // PrePaint owns size-dependent filter resolution (including the
            // physical box-reflection offset and fetched filter resources).
            // Recording the raw style chain here would feed percentages to
            // Raster a second time and bypass that typed boundary.
            item.filters = node.recording_filters.clone();
            item.filter_reference_box = node.properties.nodes.effect.filter_reference_box;
            item.filter_reference_scale = node.properties.nodes.effect.filter_reference_scale;
            self.Append(item, node);
        }
    }

    // cpp: paint/paint_context.h:13
    // cpp: paint/paint_context.cc:225-229
    pub fn EndNode(&mut self, node: &PaintTreeNode<'_>) {
        self.EndEscapableGeometry(node);
        self.EndEffects(node);
        self.EndGeometry(node);
    }

    // cpp: paint/paint_context.h:15
    // cpp: paint/paint_context.cc:231-265
    pub fn EndFilter(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source || !fragment.paint.establishes_paint_state {
            return;
        }
        if node.applies_filter {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
    }

    pub fn EndMaskClip(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if fragment.paint.has_source && fragment.paint.establishes_paint_state && node.applies_mask
        {
            // The mask source uses its original ancestor clip, not MaskClip.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
    }

    pub fn EndEffects(&mut self, node: &PaintTreeNode<'_>) {
        self.EndFilter(node);
        self.EndCssClip(node);
        self.EndMaskClip(node);
        self.EndEffectsAfterFilter(node);
    }

    pub fn BeginCssClip(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source
            || !fragment.paint.establishes_paint_state
            || !node.applies_css_clip
        {
            return;
        }
        let rect = node.local_css_clip.expect("CSS clip checked above");
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            },
            node,
        );
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kClipRect,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                rect,
                antialias: true,
                ..Default::default()
            },
            node,
        );
    }

    pub fn EndCssClip(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source
            || !fragment.paint.establishes_paint_state
            || !node.applies_css_clip
        {
            return;
        }
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            },
            node,
        );
    }

    pub fn EndEffectsAfterFilter(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source || !fragment.paint.establishes_paint_state {
            return;
        }
        if node.applies_mask {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_opacity && !node.applies_blend {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_blend {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_backdrop_filter
            && !node.applies_compositing_layer
            && !node.applies_blend
            && !node.applies_opacity
            && !node.applies_mask
        {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_compositing_layer {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
    }

    // cpp: paint/paint_context.h:17
    // cpp: paint/paint_context.cc:267-289
    pub fn EndGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.establishes_paint_state
            || (!fragment.paint.has_source && fragment.paint.text_combine.is_none())
        {
            return;
        }
        if node.applies_clip_path && node.local_clip_path_children.len() <= 1 {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_root_clip {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_transform {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
    }

    // cpp: paint/paint_context.h:21
    // cpp: paint/paint_context.cc:291-301
    pub fn EndEscapableGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source
            || !fragment.paint.establishes_paint_state
            || !node.applies_overflow_clip
            || node.local_clip.is_none()
        {
            return;
        }
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            },
            node,
        );
    }
}

/// Translated scope behavior from Blink scoped_paint_chunk_properties.h.
/// Restoring properties deliberately does not restore the previous chunk ID.
pub struct ScopedPaintChunkProperties<'c, 'o> {
    context: &'c std::cell::RefCell<PaintContext<'o>>,
    previous: PropertyTreeState,
}

impl<'c, 'o> ScopedPaintChunkProperties<'c, 'o> {
    pub fn state(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        properties: PropertyTreeState,
    ) -> Self {
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        borrowed.update_chunk_properties(properties);
        Self { context, previous }
    }

    pub fn caret(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        caret: crate::paint_engine::CaretPosition,
        node: &PaintTreeNode<'_>,
    ) -> Self {
        use crate::paint_property_tree::EffectPaintPropertyNode;
        let fragment = node.fragment.as_deref().expect("caret fragment");
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        let mut properties = previous.clone();
        // BoxFragmentPainter::PaintCaretsIfNeeded applies OverflowClip while
        // retaining the border-box transform (not ScrollTranslation).
        properties.clip = node.contents_properties.nodes.clip.clone();
        let id = borrowed.next_property_id;
        borrowed.next_property_id += 1;
        // FrameCaret::PaintCaret + CaretEffectNodeState, frame_caret.cc:86,282.
        properties.effect = std::sync::Arc::new(
            EffectPaintPropertyNode {
                compositor: Default::default(),
                backdrop_filter_bounds: None,
                self_or_ancestor_participates_in_view_transition: false,
                lifecycle: Default::default(),
                id,
                parent: Some(previous.effect.clone()),
                local_transform_space: previous.transform.clone(),
                output_clip: None,
                filter_reference_box: None,
                filter_reference_scale: 1.0,
                opacity: if caret.visible { 1.0 } else { 0.001 },
                blend_mode: layoutng_assembly::internal::paint_input::PaintBlendMode::kNormal,
                filters: Default::default(),
                backdrop_filters: Default::default(),
                backdrop_filter_has_reference: false,
                backdrop_filter_clip: None,
                isolates_blending: false,
                has_mask: false,
                is_mask: false,
                direct_compositing_reasons: vec!["ActiveOpacityAnimation"],
            }
            .with_filter_presence()
            .with_inherited_view_transition(),
        );
        borrowed.chunker.update_id_and_properties(
            DisplayItemId {
                client_id: caret.display_item_client_id,
                r#type: DisplayItemIdType::kCaret,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo {
                is_cacheable: caret.display_item_client_is_cacheable,
                invalidation_reason: Some(caret.display_item_client_invalidation_reason),
                scrollbar_needs_update_display: None,
                is_just_created: caret.display_item_client_is_just_created,
            },
            properties,
        );
        Self { context, previous }
    }

    pub fn layer(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
    ) -> Self {
        let fragment = node.fragment.as_deref().expect("paint layer fragment");
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        if fragment.paint.paint_layer_client_id != 0 {
            borrowed.chunker.update_id_and_properties(
                DisplayItemId {
                    client_id: fragment.paint.paint_layer_client_id,
                    r#type: item_type,
                    fragment: fragment.paint.display_item_fragment,
                },
                DisplayItemClientInfo {
                    is_cacheable: fragment.paint.paint_layer_client_is_cacheable,
                    invalidation_reason: fragment.paint.paint_layer_client_invalidation_reason,
                    scrollbar_needs_update_display: None,
                    is_just_created: fragment.paint.paint_layer_client_is_just_created.get(),
                },
                node.properties.nodes.clone(),
            );
        } else {
            borrowed.update_chunk_properties(node.properties.nodes.clone());
        }
        Self { context, previous }
    }

    pub fn mask(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
    ) -> Self {
        let fragment = node.fragment.as_deref().expect("mask fragment");
        let properties = node
            .mask_properties
            .as_ref()
            .expect("Mask paint properties")
            .clone();
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        let layer_client = fragment.paint.paint_layer_client_id;
        let (client_id, client) = if layer_client != 0 {
            (
                layer_client,
                DisplayItemClientInfo {
                    is_cacheable: fragment.paint.paint_layer_client_is_cacheable,
                    invalidation_reason: fragment.paint.paint_layer_client_invalidation_reason,
                    scrollbar_needs_update_display: None,
                    is_just_created: fragment.paint.paint_layer_client_is_just_created.get(),
                },
            )
        } else {
            // Standalone owners without a native PaintLayer use their real
            // drawing client. The recorder supplies the same legal fallback.
            (
                fragment.paint.display_item_client_id,
                DisplayItemClientInfo::from_fragment(fragment),
            )
        };
        borrowed.chunker.update_id_and_properties(
            DisplayItemId {
                client_id,
                r#type: DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kMask),
                fragment: fragment.paint.display_item_fragment,
            },
            client,
            properties,
        );
        Self { context, previous }
    }

    pub fn contents(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        phase: PaintPhase,
    ) -> Self {
        let fragment = node.fragment.as_deref().expect("contents fragment");
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        borrowed.chunker.update_id_and_properties(
            DisplayItemId {
                client_id: fragment.paint.display_item_client_id,
                r#type: DisplayItemIdType::PaintPhaseToClipType(phase),
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo::from_fragment(fragment),
            node.contents_properties.nodes.clone(),
        );
        Self { context, previous }
    }

    pub fn properties_only(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
    ) -> Self {
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        borrowed.update_chunk_properties(node.properties.nodes.clone());
        Self { context, previous }
    }

    /// TextCombinePainter records parent decorations after the anonymous
    /// wrapper's scale/skew transform has been restored.  Mirror that canvas
    /// state in the paint chunk property tree as well.
    pub fn text_combine_parent(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        phase: PaintPhase,
    ) -> Self {
        let fragment = node.fragment.as_deref().expect("text combine fragment");
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        let mut properties = node.properties.nodes.clone();
        properties.transform = properties
            .transform
            .parent
            .clone()
            .expect("text combine transform must have a parent");
        borrowed.chunker.update_id_and_properties(
            DisplayItemId {
                client_id: fragment.paint.display_item_client_id,
                r#type: DisplayItemIdType::PaintPhaseToDrawingType(phase),
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo::from_fragment(fragment),
            properties,
        );
        Self { context, previous }
    }

    pub fn new(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
    ) -> Self {
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        borrowed.update_chunk_id_and_properties(node, item_type);
        Self { context, previous }
    }
}

impl Drop for ScopedPaintChunkProperties<'_, '_> {
    fn drop(&mut self) {
        self.context
            .borrow_mut()
            .update_chunk_properties(self.previous.clone());
    }
}

#[cfg(test)]
mod opaque_record_tests {
    use super::*;
    use layoutng_assembly::internal::layout_input_types::Color;
    use layoutng_assembly::internal::paint_input::{
        PaintCornerRadii, PaintCornerRadius, PaintShader,
    };

    #[test]
    fn drawing_cache_uses_display_item_client_validity() {
        let id = DisplayItemId {
            client_id: 9,
            r#type: DisplayItemIdType::kBoxDecorationBackground,
            fragment: 0,
        };
        let visual_rect = PaintRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        let mut previous = PaintRecording::default();
        previous.items = Arc::new(vec![DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect: visual_rect,
            ..Default::default()
        }]);
        previous.display_items.push(RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            image_content_revisions: Vec::new(),
            scrollbar_needs_update_display: None,
            scrollbar: None,
            kind: RecordedDisplayItemKind::Drawing,
            id,
            client_visual_rect: visual_rect,
            visual_rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: CalculateRectKnownToBeOpaqueForRecord(
                visual_rect,
                previous.items.as_slice(),
            ),
            record_background: DrawingBackgroundColor(id, visual_rect, previous.items.as_slice()),
            record_has_text: false,
            record_begin: 0,
            record_end: 1,
            scroll_translation: None,
        });
        let previous = Arc::new(previous);
        let client = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar_needs_update_display: None,
            is_cacheable: true,
            is_just_created: false,
        };

        let mut cached = PaintRecording::default();
        let mut context = PaintContext::WithEditingAndCache(
            &mut cached,
            Default::default(),
            Some(previous.clone()),
        );
        assert!(context.UseCachedDrawingWithClient(id, client, None, None));
        drop(context);
        assert_eq!(cached.display_items[0].visual_rect, visual_rect);

        let mut moved = PaintRecording::default();
        let mut context = PaintContext::WithEditingAndCache(
            &mut moved,
            Default::default(),
            Some(previous.clone()),
        );
        let moved_rect = PaintRect {
            x: visual_rect.x + 1.0,
            ..visual_rect
        };
        assert!(!context.UseCachedDrawingWithClient(id, client, None, Some(moved_rect),));
        assert_eq!(context.cache_rejected_geometry, 1);

        let mut invalid = PaintRecording::default();
        let mut context =
            PaintContext::WithEditingAndCache(&mut invalid, Default::default(), Some(previous));
        let invalid_client = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kLayout),
            ..client
        };
        assert!(!context.UseCachedDrawingWithClient(id, invalid_client, None, None));
    }

    #[test]
    fn bounded_opaque_record_proof_preserves_clips_rounding_and_uncovered_gaps() {
        let visual = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let solid = DisplayItem {
            rect: visual,
            color: Color {
                alpha: 1.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let clipped = vec![
            DisplayItem {
                r#type: DisplayItemType::kSave,
                ..Default::default()
            },
            DisplayItem {
                r#type: DisplayItemType::kClipRect,
                rect: PaintRect {
                    x: 10.2,
                    y: 15.2,
                    width: 30.6,
                    height: 40.6,
                },
                ..Default::default()
            },
            solid.clone(),
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                ..Default::default()
            },
        ];
        let expected = PaintRect {
            x: 11.0,
            y: 16.0,
            width: 29.0,
            height: 39.0,
        };
        assert_eq!(
            CalculateRectKnownToBeOpaqueForRecord(visual, &clipped),
            expected
        );
        let radius = PaintCornerRadius { x: 10.0, y: 10.0 };
        let rounded = DisplayItem {
            r#type: DisplayItemType::kDrawRoundedRect,
            corner_radii: PaintCornerRadii {
                top_left: radius,
                top_right: radius,
                bottom_left: radius,
                bottom_right: radius,
            },
            ..solid.clone()
        };
        assert_eq!(
            CalculateRectKnownToBeOpaqueForRecord(visual, &[rounded]),
            PaintRect {
                x: 3.0,
                y: 3.0,
                width: 94.0,
                height: 94.0
            }
        );
        for unknown in [
            DisplayItem {
                color: Color {
                    alpha: 0.5,
                    ..Default::default()
                },
                ..solid.clone()
            },
            DisplayItem {
                paint_shader: Some(PaintShader::default()),
                ..solid.clone()
            },
            DisplayItem {
                r#type: DisplayItemType::kDrawImageRect,
                ..solid.clone()
            },
            DisplayItem {
                blend_mode: PaintBlendMode::kMultiply,
                ..solid.clone()
            },
        ] {
            assert!(CalculateRectKnownToBeOpaqueForRecord(visual, &[unknown]).is_empty());
        }
        let mut destroying = clipped;
        destroying.push(DisplayItem {
            r#type: DisplayItemType::kSaveLayerDstIn,
            ..Default::default()
        });
        assert!(!CalculateRectKnownToBeOpaqueForRecord(visual, &destroying).is_empty());
        assert!(!CalculateRectKnownToBeOpaqueForRecord(visual, &vec![solid.clone(); 8]).is_empty());
        assert!(!CalculateRectKnownToBeOpaqueForRecord(visual, &vec![solid.clone(); 9]).is_empty());
        let left = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let adjacent = PaintRect { x: 10.0, ..left };
        let separate = PaintRect { x: 20.0, ..left };
        assert_eq!(
            PaintRect::maximum_covered_rect(left, adjacent),
            PaintRect {
                width: 20.0,
                ..left
            }
        );
        assert_eq!(PaintRect::maximum_covered_rect(left, separate), left);
        let client = DisplayItemClientInfo {
            is_cacheable: true,
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar_needs_update_display: None,
            is_just_created: false,
        };
        let mut chunker = PaintChunker::default();
        let mut chunks = Vec::new();
        for opaque in [left, adjacent, separate] {
            chunker.increment_display_item(
                &mut chunks,
                DisplayItemId::default(),
                client,
                visual,
                true,
                RasterEffectOutset::kNone,
                true,
                opaque,
            );
        }
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0].rect_known_to_be_opaque,
            PaintRect {
                width: 30.0,
                ..left
            }
        );
    }
}
