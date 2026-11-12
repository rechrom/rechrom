#![allow(non_snake_case)]

use foundation::TrustedIdentityHashMap as HashMap;
use layoutng_assembly::fragment_tree::PaintGlyphRun;
use layoutng_assembly::internal::paint_input::PaintBlendMode;
use layoutng_assembly::layout_engine::PaintFragmentCursor;
use std::sync::Arc;

use crate::contoured_border_geometry::UniformCornerRadius;
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::drawing_display_item::{
    CalculateRectKnownToBeOpaqueForRecord, DrawingBackgroundColor, TightenVisualRect,
};
use crate::paint_chunker::DisplayItemClientInfo;
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

struct TextPaintAggregation {
    node_id: u64,
    identity: u64,
    rect: PaintRect,
    properties: PropertyTreeState,
    color_is_fully_transparent: bool,
    has_text_shadow: bool,
    has_text_stroke: bool,
    ignore_depth: i32,
    document_element_is_invisible: bool,
}
/// Matches ScopedPaintTimingDetectorBlockPaintHook's stack lifetime. This
/// guard owns no layout geometry and runs no callback into another engine.
pub struct ScopedTextPaintAggregation<'c, 'o> {
    context: &'c std::cell::RefCell<PaintContext<'o>>,
    active: bool,
}
impl<'c, 'o> ScopedTextPaintAggregation<'c, 'o> {
    pub fn new(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        foreground: bool,
    ) -> Self {
        let active = foreground && context.borrow_mut().BeginTextPaintAggregation(node);
        Self { context, active }
    }
}
impl Drop for ScopedTextPaintAggregation<'_, '_> {
    fn drop(&mut self) {
        if self.active {
            self.context.borrow_mut().EndTextPaintAggregation();
        }
    }
}

pub struct PaintContext<'o> {
    output: &'o mut PaintRecording,
    paint_controller: crate::PaintController,
    /// Borrowed source text sources exist only for this synchronous Paint
    /// recording. The committed PaintArtifact retains final immutable drawing
    /// payloads, never LayoutResult or FragmentItem pointers.
    text_sources: HashMap<u64, PaintFragmentCursor<'o>>,
    svg_resource_root: Option<&'o PaintTreeNode<'static>>,
    /// Multiple paint phases and decorations for one FragmentItem share the
    /// single payload built from ShapeResult during this recording.
    text_runs: HashMap<u64, Arc<Vec<PaintGlyphRun>>>,
    pub walk_metrics: paint_model::PaintWalkMetrics,
    pub caret: Option<crate::paint_engine::CaretPosition>,
    pub drag_caret: Option<crate::paint_engine::CaretPosition>,
    pub drag_caret_local_rect: Option<layoutng_assembly::caret::geometry::LocalCaretRect>,
    pub text_control_selection:
        Option<layoutng_assembly::editing_state::TextControlSelectionPaintState>,
    pub document_selection: Option<layoutng_assembly::editing_state::DocumentSelection>,
    pub document_composition_markers:
        std::rc::Rc<[layoutng_assembly::editing_state::DocumentCompositionMarker]>,
    pub selection_handles_visible: bool,
    pub caret_local_rect: Option<layoutng_assembly::caret::geometry::LocalCaretRect>,
    /// Effective FrameCaret shape for this recording. IME composition forces
    /// this to bar independently of the owner's computed caret-shape.
    pub caret_shape: Option<layoutng_assembly::internal::caret_rect::CaretShape>,
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
    paint_under_invalidation_checking: bool,
    skipping_cache_count: usize,
    document_element_is_invisible: bool,
    /// Resident compositor-owned layers keyed by their source DOM/Layout
    /// node.  Paint retains only the type-erased strong owner, matching
    /// Blink's LayoutObject -> cc::Layer hand-off without introducing a
    /// compositor dependency below paint_record.
    foreign_layers: HashMap<u64, Arc<dyn crate::ForeignLayer>>,
    text_paint_aggregations: Vec<TextPaintAggregation>,
    paint_timing_root_properties: Option<PropertyTreeState>,
}

impl<'o> PaintContext<'o> {
    pub fn BindPaintTimingRoot(&mut self, root: &PaintTreeNode<'_>) {
        self.paint_timing_root_properties = Some(root.properties.nodes.clone());
    }
    fn BeginTextPaintAggregation(&mut self, node: &PaintTreeNode<'_>) -> bool {
        use crate::ignore_paint_timing_scope::IgnorePaintTimingScope as Ignore;
        if Ignore::IgnoreDepth() > 1 {
            return false;
        }
        let Some((node_id, identity)) = node
            .SourceCursor()
            .and_then(|cursor| cursor.TextPaintTimingAggregator())
        else {
            return false;
        };
        let fragment = node.fragment.as_deref().expect("source Paint fragment");
        let style = &fragment.paint.style;
        self.text_paint_aggregations.push(TextPaintAggregation {
            node_id,
            identity,
            rect: PaintRect::default(),
            properties: node.properties.nodes.clone(),
            color_is_fully_transparent: style.color().alpha <= 0.0,
            has_text_shadow: style.text_shadow_count() != 0,
            has_text_stroke: style.text_stroke_width() != 0.0,
            ignore_depth: Ignore::IgnoreDepth(),
            document_element_is_invisible: Ignore::IsDocumentElementInvisible(),
        });
        true
    }
    pub fn NotifyTextPaint(&mut self, visual_rect: PaintRect) {
        if crate::ignore_paint_timing_scope::IgnorePaintTimingScope::ShouldIgnore() {
            return;
        }
        if let Some(aggregation) = self.text_paint_aggregations.last_mut() {
            aggregation.rect.union(visual_rect);
        }
    }
    fn EndTextPaintAggregation(&mut self) {
        use paint_model::geometry_mapper_source::{
            float_clip_rect::FloatClipRect,
            geometry_mapper::GeometryMapper,
            native::{NativeGeometryMapperContext, NativeGeometryState},
        };
        let aggregation = self
            .text_paint_aggregations
            .pop()
            .expect("matching text paint scope");
        if aggregation.rect.is_empty() {
            return;
        }
        let root = self
            .paint_timing_root_properties
            .as_ref()
            .expect("timing root bound before native Paint");
        let rect = aggregation.rect;
        let mut mapped = FloatClipRect::FromRect(paint_model::source_geometry::RectF::new(
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        ));
        let context = NativeGeometryMapperContext {
            singular_transform_fix:
                foundation::RuntimeEnabledFeatures::GeometryMapperSingularTransformFixEnabled(),
        };
        GeometryMapper::<NativeGeometryMapperContext>::LocalToAncestorVisualRect(
            &context,
            &NativeGeometryState::from_state(&aggregation.properties),
            &NativeGeometryState::from_state(root),
            &mut mapped,
            foundation::OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            Default::default(),
        );
        let mapped = mapped.Rect();
        self.output
            .text_paint_observations
            .push(paint_model::paint_timing::TextPaintObservation {
                node_id: aggregation.node_id,
                layout_object_identity: aggregation.identity,
                frame_visual_rect: aggregation.rect,
                local_root_visual_rect: PaintRect {
                    x: mapped.x() as f64,
                    y: mapped.y() as f64,
                    width: mapped.width() as f64,
                    height: mapped.height() as f64,
                },
                color_is_fully_transparent: aggregation.color_is_fully_transparent,
                has_text_shadow: aggregation.has_text_shadow,
                has_text_stroke: aggregation.has_text_stroke,
                ignore_depth: aggregation.ignore_depth,
                document_element_is_invisible: aggregation.document_element_is_invisible,
            });
    }

    /// Record the LayoutImage paint entry independently from whether its
    /// ImagePainter later emits a DrawImage operation.
    pub fn RecordPaintedLayoutImage(&mut self, image_id: image_resource::ImageId) {
        if image_id != 0 {
            self.output
                .painted_layout_image_resource_ids
                .insert(image_id);
        }
    }

    /// Source: paint_controller.cc:884-887. Paint timing suppression is
    /// thread-local in Blink and in Rechrom's translated scope.
    pub fn SetFirstPainted(&mut self) {
        if crate::ignore_paint_timing_scope::IgnorePaintTimingScope::IgnoreDepth() == 0 {
            self.output.first_painted = true;
        }
    }

    /// Source: html_canvas_painter.cc:48-50. Contentful timing is a document
    /// signal and remains independent from PaintController::SetFirstPainted.
    pub fn SetFirstContentfulPaint(&mut self) {
        if crate::ignore_paint_timing_scope::IgnorePaintTimingScope::IgnoreDepth() == 0 {
            self.output.first_contentful_painted = true;
        }
    }

    /// Source: paint_controller.cc:893-897 and video_painter.cc:58-62.
    pub fn SetImagePainted(&mut self) {
        if crate::ignore_paint_timing_scope::IgnorePaintTimingScope::IgnoreDepth() == 0 {
            self.output.image_painted = true;
        }
    }

    /// Source: paint_controller.cc:889-892 and text_painter.cc:419-422.
    pub fn SetTextPainted(&mut self) {
        if crate::ignore_paint_timing_scope::IgnorePaintTimingScope::IgnoreDepth() == 0 {
            self.output.text_painted = true;
        }
    }

    pub fn Finish(&mut self) -> crate::paint_controller::SubsequencesData {
        self.paint_controller.Finish(self.output)
    }

    /// Source: platform/graphics/paint/paint_controller.cc:154-157.
    /// SVG reference filters need a property-bearing chunk even when their
    /// container contributes no ordinary drawing item.
    pub fn EnsureChunk(&mut self) {
        self.paint_controller.EnsureChunk(self.output);
    }

    pub(crate) fn RecordSelectionBounds(
        &mut self,
        start: Option<paint_chunk::PaintSelectionBound>,
        end: Option<paint_chunk::PaintSelectionBound>,
    ) {
        self.paint_controller
            .RecordSelection(self.output, start, end);
    }

    pub(crate) fn RecordAnySelectionWasPainted(&mut self) {
        self.paint_controller
            .RecordAnySelectionWasPainted(self.output);
    }
    pub fn new(output: &'o mut PaintRecording) -> Self {
        Self {
            output,
            paint_controller: crate::PaintController::default(),
            text_sources: HashMap::default(),
            svg_resource_root: None,
            text_runs: HashMap::default(),
            walk_metrics: Default::default(),
            caret: None,
            drag_caret: None,
            drag_caret_local_rect: None,
            text_control_selection: None,
            document_selection: None,
            document_composition_markers: Default::default(),
            selection_handles_visible: false,
            caret_local_rect: None,
            caret_shape: None,
            drawing_recorder: None,
            next_property_id: 1,
            paint_under_invalidation_checking:
                foundation::RuntimeEnabledFeatures::PaintUnderInvalidationCheckingEnabled(),
            skipping_cache_count: 0,
            document_element_is_invisible: false,
            foreign_layers: HashMap::default(),
            text_paint_aggregations: Vec::new(),
            paint_timing_root_properties: None,
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
        Self::WithEditingAndPersistentCache(
            output,
            editing,
            previous,
            crate::paint_controller::SubsequencesData::default(),
        )
    }

    pub fn WithEditingAndPersistentCache(
        output: &'o mut PaintRecording,
        editing: crate::paint_engine::EditingPaintState,
        previous: Option<Arc<PaintRecording>>,
        previous_subsequences: crate::paint_controller::SubsequencesData,
    ) -> Self {
        let paint_controller =
            crate::PaintController::WithPreviousAndSubsequences(previous, previous_subsequences);
        paint_controller.PrepareNewArtifact(output);
        Self {
            output,
            paint_controller,
            text_sources: HashMap::default(),
            svg_resource_root: None,
            text_runs: HashMap::default(),
            walk_metrics: Default::default(),
            caret: editing.caret,
            drag_caret: editing.drag_caret,
            drag_caret_local_rect: None,
            text_control_selection: editing.text_control_selection,
            document_selection: editing.document_selection,
            document_composition_markers: editing.document_composition_markers,
            selection_handles_visible: editing.selection_handles_visible,
            caret_local_rect: None,
            caret_shape: None,
            drawing_recorder: None,
            next_property_id: 1,
            paint_under_invalidation_checking:
                foundation::RuntimeEnabledFeatures::PaintUnderInvalidationCheckingEnabled(),
            skipping_cache_count: 0,
            document_element_is_invisible: false,
            foreign_layers: HashMap::default(),
            text_paint_aggregations: Vec::new(),
            paint_timing_root_properties: None,
        }
    }

    pub fn BindForeignLayers(&mut self, layers: &HashMap<u64, Arc<dyn crate::ForeignLayer>>) {
        self.foreign_layers = layers.clone();
    }

    pub fn ForeignLayerForNode(&self, node_id: u64) -> Option<Arc<dyn crate::ForeignLayer>> {
        self.foreign_layers.get(&node_id).cloned()
    }

    pub fn PaintUnderInvalidationCheckingEnabled(&self) -> bool {
        self.paint_under_invalidation_checking
    }

    pub fn SetPaintUnderInvalidationCheckingForTesting(&mut self, enabled: bool) {
        self.paint_under_invalidation_checking = enabled;
    }

    /// Source: paint_layer_painter.cc:338-348. PaintEngine resolves the
    /// documentElement once from the immutable root fragment; every nested
    /// layer scope consumes the same document-global fact.
    pub fn SetDocumentElementIsInvisible(&mut self, invisible: bool) {
        self.document_element_is_invisible = invisible;
    }

    pub fn DocumentElementIsInvisible(&self) -> bool {
        self.document_element_is_invisible
    }

    // Source: display_item_cache_skipper.h:18-26 and
    // paint_controller.h:248-261.
    pub fn BeginSkippingCache(&mut self) {
        self.skipping_cache_count += 1;
    }

    pub fn EndSkippingCache(&mut self) {
        assert!(self.skipping_cache_count > 0, "cache skipper is unbalanced");
        self.skipping_cache_count -= 1;
    }

    pub fn IsSkippingCache(&self) -> bool {
        self.skipping_cache_count != 0
    }

    fn ApplyCacheSkipping(&self, mut client: DisplayItemClientInfo) -> DisplayItemClientInfo {
        if self.skipping_cache_count != 0 {
            client.is_cacheable = false;
            client.invalidation_reason = Some(foundation::PaintInvalidationReason::kUncacheable);
        }
        client
    }

    pub fn WithCaret(
        output: &'o mut PaintRecording,
        caret: Option<crate::paint_engine::CaretPosition>,
    ) -> Self {
        Self::WithEditing(
            output,
            crate::paint_engine::EditingPaintState {
                caret,
                drag_caret: None,
                text_control_selection: None,
                document_selection: None,
                selection_handles_visible: false,
                document_composition_markers: Default::default(),
            },
        )
    }
    pub fn SetCaretGeometry(&mut self, geometry: crate::paint_engine::CaretGeometry) {
        self.output.caret = Some(geometry);
    }

    /// Index source FragmentItem cursors without copying their shape or child
    /// topology. The cursor lifetime is deliberately tied to this PaintContext
    /// and therefore cannot escape into the committed artifact.
    pub fn BindSvgResourceTree(&mut self, root: &'o PaintTreeNode<'static>) {
        self.svg_resource_root = Some(root);
    }
    pub fn SvgResourceNode(&self, client: u64) -> Option<&'o PaintTreeNode<'static>> {
        self.svg_resource_root?.SvgResourceNode(client)
    }
    pub fn BindTextSources(&mut self, root: PaintFragmentCursor<'o>) {
        fn collect<'a>(
            cursor: PaintFragmentCursor<'a>,
            output: &mut HashMap<u64, PaintFragmentCursor<'a>>,
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
        collect(root, &mut self.text_sources);
    }

    pub fn HasTextSource(&self, fragment_instance_id: u64) -> bool {
        self.text_sources.contains_key(&fragment_instance_id)
    }

    pub fn TextSource(&self, fragment_instance_id: u64) -> Option<PaintFragmentCursor<'o>> {
        self.text_sources.get(&fragment_instance_id).copied()
    }

    /// Resolve ShapeResult only after a cache miss has committed Paint to
    /// recording this text item. Later phases share the same immutable owner.
    pub fn TextGlyphRuns(&mut self, fragment_instance_id: u64) -> Arc<Vec<PaintGlyphRun>> {
        if let Some(runs) = self.text_runs.get(&fragment_instance_id) {
            return Arc::clone(runs);
        }
        let runs = self
            .text_sources
            .get(&fragment_instance_id)
            .copied()
            .and_then(PaintFragmentCursor::PaintGlyphRuns)
            .unwrap_or_else(|| Arc::new(Vec::new()));
        self.text_runs
            .insert(fragment_instance_id, Arc::clone(&runs));
        runs
    }

    pub fn CacheMetrics(&self) -> paint_reuse::PaintReuseMetrics {
        self.paint_controller.Metrics()
    }

    // paint_controller.cc:UseCachedItemIfPossible. A valid cacheable client
    // moves its old semantic DrawingDisplayItem into the new artifact while
    // the current PrePaint property state is retained by the new PaintChunk.
    pub fn UseCachedDrawingIfPossible(
        &mut self,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
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
        )
    }

    pub fn UseCachedDrawingWithClient(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        debug: Option<(i32, Arc<str>)>,
    ) -> bool {
        let Some(cached) = self.paint_controller.TakeCachedDrawing(
            id,
            client,
            self.paint_under_invalidation_checking,
        ) else {
            return false;
        };
        let old = &cached.item;
        let record_begin = self.output.items.len();
        Arc::make_mut(&mut self.output.items).extend_from_slice(&cached.record);
        let record_end = self.output.items.len();
        if let Some((owner, name)) = debug {
            self.output
                .RecordDebugInfo(id.client_id as usize, name, owner);
        }
        self.paint_controller.ProcessNewItem(
            self.output,
            id,
            client,
            old.visual_rect,
            old.draws_content,
            old.raster_effect_outset,
            old.visual_rect_is_accurate,
            old.record_rect_known_to_be_opaque,
            old.record_background,
            old.record_has_text,
        );
        let mut moved = old.clone();
        moved.paint_invalidation_reason = Some(foundation::PaintInvalidationReason::kNone);
        moved.record_begin = record_begin;
        moved.record_end = record_end;
        self.output.display_items.push(moved);
        true
    }

    /// Source: `paint_controller.cc:337-409` through
    /// `SubsequenceRecorder::UseCachedSubsequenceIfPossible`.
    pub fn UseCachedSubsequenceIfPossible(
        &mut self,
        client_id: u64,
        client: DisplayItemClientInfo,
        debug: Option<(i32, Arc<str>)>,
    ) -> Option<paint_model::PaintResult> {
        if let Some((owner, name)) = debug {
            self.output.RecordDebugInfo(client_id as usize, name, owner);
        }
        self.paint_controller.UseCachedSubsequenceIfPossible(
            self.output,
            client_id,
            client,
            self.paint_under_invalidation_checking,
        )
    }

    /// Source: `paint_controller.cc:411-418`.
    pub fn BeginSubsequence(&mut self, client_id: u64, debug: Option<(i32, Arc<str>)>) -> usize {
        if let Some((owner, name)) = debug {
            self.output.RecordDebugInfo(client_id as usize, name, owner);
        }
        self.paint_controller
            .BeginSubsequence(client_id, self.output)
    }

    /// Source: `paint_controller.cc:420-460`.
    pub fn EndSubsequence(&mut self, subsequence_index: usize) {
        self.paint_controller
            .EndSubsequence(subsequence_index, self.output);
    }

    pub fn SetSubsequencePaintResult(
        &mut self,
        subsequence_index: usize,
        result: paint_model::PaintResult,
    ) {
        self.paint_controller
            .SetSubsequencePaintResult(subsequence_index, result);
    }

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
        let client = self.ApplyCacheSkipping(DisplayItemClientInfo::from_fragment(fragment));
        self.output.RecordDebugInfo(
            id.client_id as usize,
            fragment.paint.display_item_client_debug_name.clone(),
            fragment.paint.display_item_client_owner_node_id,
        );
        let (visual_rect, visual_rect_is_accurate) = TightenVisualRect(
            item.rect,
            std::slice::from_ref(&item),
            node.paint_snap_offset,
        );
        let raster_effect_outset = fragment.paint.display_item_raster_effect_outset;
        let rect_known_to_be_opaque =
            CalculateRectKnownToBeOpaqueForRecord(visual_rect, std::slice::from_ref(&item));
        let record_background =
            DrawingBackgroundColor(id, visual_rect, std::slice::from_ref(&item));
        let record_has_text = item.r#type == DisplayItemType::kDrawGlyphRun;
        self.paint_controller.ProcessNewItem(
            self.output,
            id,
            client,
            visual_rect,
            true,
            raster_effect_outset,
            visual_rect_is_accurate,
            rect_known_to_be_opaque,
            record_background,
            record_has_text,
        );
        self.output.display_items.push(RecordedDisplayItem {
            is_cacheable: client.is_cacheable,
            is_tombstone: false,
            paint_invalidation_reason: client.invalidation_reason,
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            scroll_translation: None,
            id,
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
        self.paint_controller
            .CheckNewItem(self.output, self.skipping_cache_count != 0);
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
        self.output.RecordDebugInfo(
            fragment.paint.display_item_client_id as usize,
            fragment.paint.display_item_client_debug_name.clone(),
            fragment.paint.display_item_client_owner_node_id,
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

    pub fn RegisterAuxiliaryClient(
        &mut self,
        id: u64,
        owner_node_id: i32,
        debug_name: std::sync::Arc<str>,
    ) {
        self.output
            .RecordDebugInfo(id as usize, debug_name, owner_node_id);
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
        scroll_translation: Option<
            std::sync::Arc<crate::paint_property_tree::TransformPaintPropertyNode>,
        >,
        scrollbar: crate::paint_engine::ScrollbarDisplayItemData,
        raster_effect_outset: RasterEffectOutset,
        delta: Offset,
    ) {
        assert!(
            !self.InDrawingRecorder(),
            "ScrollbarDisplayItem is an outer item"
        );
        assert!(
            id.r#type == DisplayItemIdType::kScrollbarHorizontal
                || id.r#type == DisplayItemIdType::kScrollbarVertical
        );
        assert!(RecordedDisplayItem::IsScrollbarElementId(
            scrollbar.element_id
        ));
        debug_assert!(
            scroll_translation
                .as_ref()
                .is_none_or(|translation| translation.scroll.is_some()),
            "ScrollbarDisplayItem scroll translation must own a ScrollNode"
        );
        self.drawing_recorder = Some((
            id,
            client,
            self.chunk_properties(),
            visual_rect,
            self.output.items.len(),
            RecordedDisplayItemKind::Scrollbar,
            Some(scrollbar),
            scroll_translation,
            raster_effect_outset,
            true,
            delta,
            None,
        ));
    }

    /// Source: `drawing_recorder.h:76-79`.
    pub fn UniteDrawingRecorderVisualRect(&mut self, rect: PaintRect) {
        let recorder = self
            .drawing_recorder
            .as_mut()
            .expect("DrawingRecorder must be active");
        recorder.3.union(rect);
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
        let client = self.ApplyCacheSkipping(client);
        let record_end = self.output.items.len();
        let (mut visual_rect, tightened) = if kind == RecordedDisplayItemKind::Drawing {
            TightenVisualRect(
                visual_rect,
                &self.output.items[record_begin..record_end],
                delta,
            )
        } else {
            (EnclosingRectInPropertySpace(visual_rect, delta), false)
        };
        // Source: paint_chunks_to_cc_layer.cc:806-814. A reference filter can
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
        self.paint_controller
            .UpdateCurrentPaintChunkProperties(properties);
        let rect_known_to_be_opaque = if kind == RecordedDisplayItemKind::Drawing {
            CalculateRectKnownToBeOpaqueForRecord(
                visual_rect,
                &self.output.items[record_begin..record_end],
            )
        } else {
            PaintRect::default()
        };
        let mut record_background = Default::default();
        let mut record_has_text = false;
        if kind == RecordedDisplayItemKind::Drawing {
            let record = &self.output.items[record_begin..record_end];
            if draws_content {
                record_background = DrawingBackgroundColor(id, visual_rect, record);
                record_has_text = record
                    .iter()
                    .any(|op| op.r#type == DisplayItemType::kDrawGlyphRun);
            }
        }
        self.paint_controller.ProcessNewItem(
            self.output,
            id,
            client,
            visual_rect,
            draws_content,
            raster_effect_outset,
            visual_rect_is_accurate,
            rect_known_to_be_opaque,
            record_background,
            record_has_text,
        );
        self.output.display_items.push(RecordedDisplayItem {
            is_cacheable: client.is_cacheable,
            is_tombstone: false,
            paint_invalidation_reason: client.invalidation_reason,
            scrollbar,
            foreign_layer: None,
            kind,
            scroll_translation,
            id,
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
        self.paint_controller
            .CheckNewItem(self.output, self.skipping_cache_count != 0);
    }

    /// Package-boundary half of `PaintController::CreateAndAppend` used by
    /// `RecordForeignLayer`; the source-shaped orchestration remains in
    /// `foreign_layer_display_item.rs`.
    pub(crate) fn AppendForeignLayerDisplayItem(
        &mut self,
        mut item: RecordedDisplayItem,
        client: DisplayItemClientInfo,
    ) {
        assert!(!self.InDrawingRecorder());
        assert_eq!(item.kind, RecordedDisplayItemKind::ForeignLayer);
        assert!(item.id.r#type.IsForeignLayerType());
        let client = self.ApplyCacheSkipping(client);
        item.is_cacheable = client.is_cacheable;
        item.paint_invalidation_reason = client.invalidation_reason;
        item.record_begin = self.output.items.len();
        item.record_end = item.record_begin;
        self.paint_controller.ProcessNewItem(
            self.output,
            item.id,
            client,
            item.visual_rect,
            true,
            item.raster_effect_outset,
            item.visual_rect_is_accurate,
            PaintRect::default(),
            Default::default(),
            false,
        );
        self.output.display_items.push(item);
        self.paint_controller
            .CheckNewItem(self.output, self.skipping_cache_count != 0);
    }

    // Source: paint_controller.cc:204-218 and paint_chunker.cc:300-339. The
    // caller supplies the actual
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
        self.output.RecordDebugInfo(
            id.client_id as usize,
            fragment.paint.display_item_client_debug_name.clone(),
            fragment.paint.display_item_client_owner_node_id,
        );
        self.paint_controller.RecordScrollHitTestData(
            self.output,
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

    /// Source: object_painter.cc:142-158 and paint_controller.cc:159-174.
    /// The caller selects the source visual rect and SVG/box opacity policy;
    /// this boundary supplies the canonical display-item identity and Layout
    /// event-handler facts retained on the immutable fragment.
    pub fn RecordHitTestData(
        &mut self,
        node: &PaintTreeNode<'_>,
        rect: PaintRect,
        opaqueness: crate::paint_engine::PaintHitTestOpaqueness,
    ) {
        let fragment = node.fragment.as_deref().expect("hit-test fragment");
        self.RecordHitTestDataWithClient(
            node,
            fragment.paint.display_item_client_id,
            DisplayItemClientInfo::from_fragment(fragment),
            rect,
            opaqueness,
        );
    }

    pub(crate) fn RecordHitTestDataWithClient(
        &mut self,
        node: &PaintTreeNode<'_>,
        client_id: u64,
        client: DisplayItemClientInfo,
        rect: PaintRect,
        opaqueness: crate::paint_engine::PaintHitTestOpaqueness,
    ) {
        let fragment = node.fragment.as_deref().expect("hit-test fragment");
        let id = DisplayItemId {
            client_id,
            r#type: DisplayItemIdType::kHitTest,
            fragment: fragment.paint.display_item_fragment,
        };
        if client_id == fragment.paint.display_item_client_id {
            self.output.RecordDebugInfo(
                id.client_id as usize,
                fragment.paint.display_item_client_debug_name.clone(),
                fragment.paint.display_item_client_owner_node_id,
            );
        }
        self.paint_controller.RecordHitTestData(
            self.output,
            id,
            client,
            rect,
            fragment.paint.effective_allowed_touch_action,
            fragment.paint.inside_blocking_wheel_event_handler,
            opaqueness,
        );
    }

    /// Source: svg_model_object_painter.cc:40-53 and
    /// paint_controller.cc:176-188.
    pub fn RecordRegionCaptureData(
        &mut self,
        node: &PaintTreeNode<'_>,
        crop_id: [u64; 2],
        rect: PaintRect,
    ) {
        let fragment = node.fragment.as_deref().expect("region-capture fragment");
        self.RecordRegionCaptureDataWithClient(
            node,
            fragment.paint.display_item_client_id,
            DisplayItemClientInfo::from_fragment(fragment),
            crop_id,
            rect,
        );
    }

    pub(crate) fn RecordRegionCaptureDataWithClient(
        &mut self,
        node: &PaintTreeNode<'_>,
        client_id: u64,
        client: DisplayItemClientInfo,
        crop_id: [u64; 2],
        rect: PaintRect,
    ) {
        debug_assert_ne!(crop_id, [0; 2]);
        let fragment = node.fragment.as_deref().expect("region-capture fragment");
        let id = DisplayItemId {
            client_id,
            r#type: DisplayItemIdType::kRegionCapture,
            fragment: fragment.paint.display_item_fragment,
        };
        if client_id == fragment.paint.display_item_client_id {
            self.output.RecordDebugInfo(
                id.client_id as usize,
                fragment.paint.display_item_client_debug_name.clone(),
                fragment.paint.display_item_client_owner_node_id,
            );
        }
        self.paint_controller
            .RecordRegionCaptureData(self.output, id, client, crop_id, rect);
    }

    /// Source: box_painter.cc:30-33 and paint_controller.cc:190-202.
    pub fn RecordTrackedElementData(
        &mut self,
        node: &PaintTreeNode<'_>,
        element_paint_rect: PaintRect,
        sub_rects: &[layoutng_assembly::internal::layout_input::TrackedElementSubRect],
    ) {
        let fragment = node.fragment.as_deref().expect("tracked-element fragment");
        self.RecordTrackedElementDataWithClient(
            node,
            fragment.paint.display_item_client_id,
            DisplayItemClientInfo::from_fragment(fragment),
            element_paint_rect,
            sub_rects,
        );
    }

    pub(crate) fn RecordTrackedElementDataWithClient(
        &mut self,
        node: &PaintTreeNode<'_>,
        client_id: u64,
        client: DisplayItemClientInfo,
        element_paint_rect: PaintRect,
        sub_rects: &[layoutng_assembly::internal::layout_input::TrackedElementSubRect],
    ) {
        use layoutng_assembly::internal::layout_input::{
            TrackedElementFrameToken, TrackedElementSubRectType,
        };
        use paint_chunk::{
            PaintFrameToken, PaintTrackedElementRelativeRect, PaintTrackedElementSubRect,
            PaintTrackedElementSubRectType,
        };

        let frame_token = |token: TrackedElementFrameToken| match token {
            TrackedElementFrameToken::Local(value) => PaintFrameToken::Local(value),
            TrackedElementFrameToken::Remote(value) => PaintFrameToken::Remote(value),
        };
        let converted = sub_rects
            .iter()
            .map(|sub_rect| PaintTrackedElementSubRect {
                feature: sub_rect.feature,
                id: sub_rect.id,
                should_add_to_compositor_frame_metadata: sub_rect
                    .should_add_to_compositor_frame_metadata,
                should_exclude_fixed_and_sticky_occlusions: sub_rect
                    .should_exclude_fixed_and_sticky_occlusions,
                sub_rect: sub_rect
                    .sub_rect
                    .map(|relative| PaintTrackedElementRelativeRect {
                        rect: PaintRect {
                            x: f64::from(relative.x),
                            y: f64::from(relative.y),
                            width: f64::from(relative.width),
                            height: f64::from(relative.height),
                        },
                        r#type: match relative.r#type {
                            TrackedElementSubRectType::IntersectWithElementRect => {
                                PaintTrackedElementSubRectType::IntersectWithElementRect
                            }
                            TrackedElementSubRectType::NoIntersection => {
                                PaintTrackedElementSubRectType::NoIntersection
                            }
                        },
                    }),
                frame_token: sub_rect.frame_token.map(frame_token),
                parent_frame_token: sub_rect.parent_frame_token.map(PaintFrameToken::Local),
            })
            .collect::<Vec<_>>();
        let fragment = node.fragment.as_deref().expect("tracked-element fragment");
        let id = DisplayItemId {
            client_id,
            r#type: DisplayItemIdType::kTrackedElement,
            fragment: fragment.paint.display_item_fragment,
        };
        if client_id == fragment.paint.display_item_client_id {
            self.output.RecordDebugInfo(
                id.client_id as usize,
                fragment.paint.display_item_client_debug_name.clone(),
                fragment.paint.display_item_client_owner_node_id,
            );
        }
        self.paint_controller.RecordTrackedElementData(
            self.output,
            id,
            client,
            element_paint_rect,
            &converted,
        );
    }

    // Source: scrollable_area_painter.cc:97-114 and
    // paint_controller.cc:204-218. A resizer uses the scroll-corner client,
    // its own display-item type, and deliberately has no scroll translation:
    // the metadata exists only to force touch resizing onto the main thread.
    pub fn RecordResizerScrollHitTestData(
        &mut self,
        node: &PaintTreeNode<'_>,
        client_id: u64,
        client: DisplayItemClientInfo,
        rect: PaintRect,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("resizer scroll hit-test fragment");
        let id = DisplayItemId {
            client_id,
            r#type: DisplayItemIdType::kResizerScrollHitTest,
            fragment: fragment.paint.display_item_fragment,
        };
        let mut data = crate::paint_engine::PaintHitTestData::default();
        data.scroll_hit_test_rect = rect;
        self.paint_controller.RecordScrollHitTestData(
            self.output,
            id,
            client,
            rect,
            crate::paint_engine::PaintHitTestOpaqueness::Mixed,
            data,
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
        self.paint_controller.CurrentPaintChunkProperties().clone()
    }

    pub fn update_chunk_properties(&mut self, properties: PropertyTreeState) {
        self.paint_controller
            .UpdateCurrentPaintChunkProperties(properties);
    }

    pub fn SetCurrentEffectivelyInvisible(&mut self, invisible: bool) {
        self.paint_controller
            .SetCurrentEffectivelyInvisible(invisible);
    }

    pub fn CurrentEffectivelyInvisible(&self) -> bool {
        self.paint_controller.CurrentEffectivelyInvisible()
    }

    pub fn NumNewChunks(&self) -> usize {
        self.paint_controller.NumNewChunks(self.output)
    }

    pub fn LastChunkProperties(&self) -> Option<PropertyTreeState> {
        self.paint_controller
            .LastChunk(self.output)
            .map(|chunk| chunk.properties.clone())
    }

    pub fn LastChunkBounds(&self) -> Option<PaintRect> {
        self.paint_controller
            .LastChunk(self.output)
            .map(|chunk| chunk.bounds)
    }

    pub fn CurrentChunkIsNonEmptyAndTransparentToHitTest(&self) -> bool {
        self.paint_controller
            .CurrentChunkIsNonEmptyAndTransparentToHitTest(self.output)
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
        self.paint_controller
            .UpdateCurrentPaintChunkIdAndProperties(
                DisplayItemId {
                    client_id: fragment.paint.display_item_client_id,
                    r#type: item_type,
                    fragment: fragment.paint.display_item_fragment,
                },
                DisplayItemClientInfo::from_fragment(fragment),
                node.properties.nodes.clone(),
            );
    }

    pub(crate) fn update_chunk_id_and_properties_raw(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        properties: PropertyTreeState,
    ) {
        self.paint_controller
            .UpdateCurrentPaintChunkIdAndProperties(id, client, properties);
    }

    pub fn BeginNode(&mut self, node: &PaintTreeNode<'_>) {
        self.BeginGeometry(node);
        self.BeginEffects(node);
        self.BeginEscapableGeometry(node, None);
    }

    pub fn BeginGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.establishes_paint_state
            || (!fragment.paint.HasSource() && fragment.paint.text_combine.is_none())
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

    pub fn BeginEscapableGeometry(
        &mut self,
        node: &PaintTreeNode<'_>,
        clip_override: Option<PaintRect>,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.HasSource()
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
        if clip_override.is_none() {
            if let Some(inner_clip) = node.local_inner_border_radius_clip {
                let inner_radii = node.local_inner_border_radius_radii;
                self.Append(
                    DisplayItem {
                        r#type: if inner_radii.HasRadius() {
                            DisplayItemType::kClipRoundedRect
                        } else {
                            DisplayItemType::kClipRect
                        },
                        phase: PaintPhase::kForeground,
                        node_id: fragment.node_id,
                        rect: inner_clip,
                        corner_radius: UniformCornerRadius(&inner_radii),
                        corner_radii: inner_radii,
                        antialias: true,
                        ..Default::default()
                    },
                    node,
                );
            }
        }
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

    pub fn BeginEffects(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.HasSource() || !fragment.paint.establishes_paint_state {
            return;
        }
        let style = &fragment.paint.style;
        if node.applies_backdrop_filter
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
                    blend_mode: style.blend_mode(),
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
            // PaintLayer::UpdateCompositorFilterOperationsForFilter publishes
            // one canonical value on the Effect node. Flat PaintRecords retain
            // that value instead of asking Raster to rebuild authored filters.
            item.canonical_filters = node.properties.nodes.effect.filters.clone();
            self.Append(item, node);
        }
    }

    pub fn EndNode(&mut self, node: &PaintTreeNode<'_>) {
        self.EndEscapableGeometry(node);
        self.EndEffects(node);
        self.EndGeometry(node);
    }

    pub fn EndFilter(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.HasSource() || !fragment.paint.establishes_paint_state {
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
        if fragment.paint.HasSource() && fragment.paint.establishes_paint_state && node.applies_mask
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
        if !fragment.paint.HasSource()
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
        if !fragment.paint.HasSource()
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
        if !fragment.paint.HasSource() || !fragment.paint.establishes_paint_state {
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
    }

    pub fn EndGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.establishes_paint_state
            || (!fragment.paint.HasSource() && fragment.paint.text_combine.is_none())
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

    pub fn EndEscapableGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.HasSource()
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
    pub(crate) context: &'c std::cell::RefCell<PaintContext<'o>>,
    pub(crate) previous: PropertyTreeState,
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

    pub fn drag_caret(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        caret: crate::paint_engine::CaretPosition,
        node: &PaintTreeNode<'_>,
    ) -> Self {
        let fragment = node.fragment.as_deref().expect("drag caret fragment");
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        let mut properties = previous.clone();
        properties.clip = node.contents_properties.nodes.clip.clone();
        borrowed.update_chunk_id_and_properties_raw(
            DisplayItemId {
                client_id: caret.display_item_client_id,
                r#type: DisplayItemIdType::kDragCaret,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo {
                is_cacheable: caret.display_item_client_is_cacheable,
                invalidation_reason: Some(caret.display_item_client_invalidation_reason),
                is_just_created: caret.display_item_client_is_just_created,
            },
            properties,
        );
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
                filter_viewport: None,
                filter_space_origin: Offset::default(),
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
                direct_compositing_reasons: foundation::CompositingReasons::FromReason(
                    foundation::CompositingReason::kActiveOpacityAnimation,
                ),
            }
            .with_filter_presence()
            .with_inherited_view_transition(),
        );
        borrowed.update_chunk_id_and_properties_raw(
            DisplayItemId {
                client_id: caret.display_item_client_id,
                r#type: DisplayItemIdType::kCaret,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo {
                is_cacheable: caret.display_item_client_is_cacheable,
                invalidation_reason: Some(caret.display_item_client_invalidation_reason),
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
            borrowed.update_chunk_id_and_properties_raw(
                DisplayItemId {
                    client_id: fragment.paint.paint_layer_client_id,
                    r#type: item_type,
                    fragment: fragment.paint.display_item_fragment,
                },
                DisplayItemClientInfo {
                    is_cacheable: fragment.paint.paint_layer_client_is_cacheable,
                    invalidation_reason: fragment.paint.paint_layer_client_invalidation_reason,
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
        Self::mask_with_type(
            context,
            node,
            DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kMask),
        )
    }

    pub fn mask_with_type(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
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
                    is_just_created: fragment.paint.paint_layer_client_is_just_created.get(),
                },
            )
        } else {
            // Standalone owners without a source PaintLayer use their real
            // drawing client. The recorder supplies the same legal fallback.
            (
                fragment.paint.display_item_client_id,
                DisplayItemClientInfo::from_fragment(fragment),
            )
        };
        borrowed.update_chunk_id_and_properties_raw(
            DisplayItemId {
                client_id,
                r#type: item_type,
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
        borrowed.update_chunk_id_and_properties_raw(
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
        borrowed.update_chunk_id_and_properties_raw(
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

#[cfg(test)]
mod opaque_record_tests {
    use super::*;
    use layoutng_assembly::internal::layout_input_types::Color;
    use layoutng_assembly::internal::paint_input::{
        PaintCornerRadii, PaintCornerRadius, PaintShader,
    };
    use paint_chunk::PaintChunker;

    #[test]
    fn text_paint_timing_signal_honors_ignore_scope() {
        let mut output = PaintRecording::default();
        {
            let mut context = PaintContext::new(&mut output);
            context.SetTextPainted();
        }
        assert!(output.text_painted);

        let mut ignored = PaintRecording::default();
        {
            let _scope = crate::ignore_paint_timing_scope::IgnorePaintTimingScope::new();
            crate::ignore_paint_timing_scope::IgnorePaintTimingScope::IncrementIgnoreDepth();
            let mut context = PaintContext::new(&mut ignored);
            context.SetTextPainted();
        }
        assert!(!ignored.text_painted);
    }

    #[test]
    fn scoped_chunk_transform_changes_only_transform_and_restores_properties() {
        use crate::paint_property_tree::TransformPaintPropertyNode;

        let mut output = PaintRecording::default();
        let context = std::cell::RefCell::new(PaintContext::new(&mut output));
        let root = context.borrow().chunk_properties();
        let transform = std::sync::Arc::new(TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            lifecycle: Default::default(),
            id: 1,
            parent: Some(root.transform.clone()),
            matrix: Default::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: None,
            scroll_parent_scroll_translation: None,
            sticky_constraint: None,
            direct_compositing_reasons: Default::default(),
        });
        let id = DisplayItemId {
            client_id: 9,
            r#type: DisplayItemIdType::kLayerChunk,
            fragment: 2,
        };
        let client = DisplayItemClientInfo {
            is_cacheable: true,
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_just_created: false,
        };
        {
            let _scope =
                ScopedPaintChunkProperties::transform(&context, transform.clone(), id, client);
            let current = context.borrow().chunk_properties();
            assert!(std::sync::Arc::ptr_eq(&current.transform, &transform));
            assert!(std::sync::Arc::ptr_eq(&current.clip, &root.clip));
            assert!(std::sync::Arc::ptr_eq(&current.effect, &root.effect));
        }
        let restored = context.borrow().chunk_properties();
        assert!(restored.same_nodes(&root));
    }

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
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            id,
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
            is_cacheable: true,
            is_just_created: false,
        };

        let mut cached = PaintRecording::default();
        let mut context = PaintContext::WithEditingAndCache(
            &mut cached,
            Default::default(),
            Some(previous.clone()),
        );
        assert!(context.UseCachedDrawingWithClient(id, client, None));
        drop(context);
        assert_eq!(cached.display_items[0].visual_rect, visual_rect);

        let mut invalid = PaintRecording::default();
        let mut context =
            PaintContext::WithEditingAndCache(&mut invalid, Default::default(), Some(previous));
        let invalid_client = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kLayout),
            ..client
        };
        assert!(!context.UseCachedDrawingWithClient(id, invalid_client, None));
    }

    #[test]
    fn under_invalidation_cache_skipper_repaints_and_marks_new_items_uncacheable() {
        let mut output = PaintRecording::default();
        let mut context = PaintContext::new(&mut output);
        context.SetPaintUnderInvalidationCheckingForTesting(true);
        let client = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_cacheable: true,
            is_just_created: false,
        };
        let id = DisplayItemId {
            client_id: 7,
            r#type: DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kForeground),
            fragment: 0,
        };
        assert!(!context.UseCachedDrawingWithClient(id, client, None));

        context.BeginSkippingCache();
        let skipped = context.ApplyCacheSkipping(client);
        assert!(!skipped.is_cacheable);
        assert_eq!(
            skipped.invalidation_reason,
            Some(foundation::PaintInvalidationReason::kUncacheable)
        );
        context.EndSkippingCache();
        assert!(context.ApplyCacheSkipping(client).is_cacheable);
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
