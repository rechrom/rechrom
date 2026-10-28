#![allow(non_camel_case_types)]

//! Source paint display-list ABI. The entry point and private painters are
//! connected after their owning source files have been translated.

use std::sync::Arc;

use layoutng_assembly::fragment_tree::PaintResources;
pub use layoutng_assembly::fragment_tree::RasterEffectOutset;
use layoutng_assembly::internal::layout_input::{Offset, Size};

pub use layoutng_assembly::internal::paint_input::{
    PaintCornerRadii, PaintFilterOperation, PaintMaskComposite, PaintMaskMode, PaintShader,
    SvgPatternCommand, SvgStrokeLineCap, SvgStrokeLineJoin,
};

pub use paint_model::PaintPhase;

pub use paint_record::PaintOpType as DisplayItemType;

pub use paint_model::PaintRect;

use paint_record::PaintRecording;
pub use paint_record::{
    CaretGeometry, CaretPosition, DisplayItem, DisplayItemList, DisplayMaskLayer,
    EditingPaintState, RecordedDisplayItem, RecordedDisplayItemKind, ScrollbarDisplayItemData,
};

// Public producer analysis also serves canonical PaintRecords downstream.
pub use crate::paint_context::CalculateRectKnownToBeOpaqueForRecord;
pub use image_resource::{
    downcast_document_paint_record, ContainerKey as DocumentImageContainerKey, PaintImageContent,
    PaintImageId as NativePaintImageId, PaintImageMetadataUnavailable as ImageMetadataUnavailable,
};
pub use layoutng_assembly::internal::layout_input::PaintImage as NativePaintImage;

pub use crate::display_item_id::DisplayItemId as PaintChunkId;
pub use paint_chunk::{
    PaintChunk, PaintChunkBackgroundColorInfo, PaintChunkCompositorData, PaintChunks,
    PaintFrameToken, PaintHitTestData, PaintHitTestOpaqueness, PaintLayerSelectionData,
    PaintSelectionBound, PaintSelectionBoundType, PaintTrackedElementRect,
};

pub use paint_commit::{
    DocumentPaintArtifactRecord, ImageDependencyIndex, ImagePaintDependency, PaintArtifact,
    PaintCompositorViewport, PaintCompositorViewportMode, PaintMutation, PaintMutationResult,
    PaintOutput, PaintResourceUpdate,
};

#[cfg(test)]
#[test]
fn document_image_container_proof_uses_the_artifacts_own_catalog() {
    let mut requested = image_resource::ContainerKey::new(17, 17, 1.0, 2.0);
    requested.preferred_color_scheme_is_explicit = true;
    let initial = image_resource::ContainerKey::new(24, 24, 1.0, 2.0);
    let record = |key: &image_resource::ContainerKey| {
        Arc::new(DocumentPaintArtifactRecord {
            artifact: Arc::new(PaintArtifact::default()),
            intrinsic_size: image_resource::IntrinsicSize {
                width: key.width,
                height: key.height,
            },
            record_size: image_resource::IntrinsicSize {
                width: key.width,
                height: key.height,
            },
            container_key: key.clone(),
        }) as Arc<dyn image_resource::DocumentPaintRecord>
    };
    let mut artifact = PaintArtifact {
        items: vec![DisplayItem {
            r#type: DisplayItemType::kDrawImageRect,
            resource_id: 7,
            document_image_container_key: Some(requested.clone()),
            ..Default::default()
        }]
        .into(),
        resources: Some(Arc::new(PaintResources {
            images: vec![NativePaintImage {
                id: 7,
                document_container_key: Some(initial.clone()),
                content: PaintImageContent::Document(Arc::new(
                    image_resource::DocumentPaintContent::new(initial.clone(), record(&initial)),
                )),
                ..Default::default()
            }],
            ..Default::default()
        })),
        ..Default::default()
    };
    assert!(!artifact.HasExactDocumentImageContainerRecords());

    let resources = Arc::make_mut(artifact.resources.as_mut().unwrap());
    resources.images[0].content = PaintImageContent::Document(Arc::new(
        image_resource::DocumentPaintContent::new(requested.clone(), record(&requested)),
    ));
    assert!(artifact.HasExactDocumentImageContainerRecords());
}

#[cfg(test)]
#[test]
fn artifact_property_updates_share_ops_and_real_op_edits_isolate_old_frames() {
    let old = Arc::new(PaintArtifact {
        items: vec![DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect: PaintRect {
                x: 3.0,
                y: 4.0,
                width: 20.0,
                height: 10.0,
            },
            ..Default::default()
        }]
        .into(),
        chunks: vec![PaintChunk::default()],
        ..Default::default()
    });
    let mut scrolling = old.clone();
    let updated = Arc::make_mut(&mut scrolling);
    // This ownership test models a new property snapshot only; it invokes no
    // native identity, layout or recording algorithm.
    Arc::make_mut(&mut updated.chunks[0].properties.transform)
        .matrix
        .values[13] = -12.0;
    assert!(
        Arc::ptr_eq(&old.items, &scrolling.items),
        "property changes share the immutable PaintRecord operations"
    );
    assert_eq!(old.chunks[0].properties.transform.matrix.values[13], 0.0);
    assert_eq!(
        scrolling.chunks[0].properties.transform.matrix.values[13],
        -12.0
    );
    Arc::make_mut(&mut Arc::make_mut(&mut scrolling).items)[0]
        .rect
        .x = 9.0;
    assert!(!Arc::ptr_eq(&old.items, &scrolling.items));
    assert_eq!(
        old.items[0].rect.x, 3.0,
        "a real operation edit leaves the old frame immutable"
    );
    assert_eq!(scrolling.items[0].rect.x, 9.0);
    assert_eq!(old.items.len(), scrolling.items.len());
}

#[cfg(test)]
#[test]
fn document_image_mutation_retains_recording_and_targets_its_client() {
    use image_resource::{ContainerKey, DocumentImageFrame, IntrinsicSize, PaintImageContent};
    let draw = DisplayItem {
        r#type: DisplayItemType::kDrawImageRect,
        resource_id: 7,
        rect: PaintRect {
            x: 32.0,
            y: 48.0,
            width: 80.0,
            height: 40.0,
        },
        ..Default::default()
    };
    let display = RecordedDisplayItem {
        is_cacheable: true,
        is_tombstone: false,
        paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
        image_content_revisions: Vec::new(),
        scrollbar_needs_update_display: None,
        scrollbar: None,
        kind: RecordedDisplayItemKind::Drawing,
        id: Default::default(),
        client_visual_rect: draw.rect,
        visual_rect: draw.rect,
        visual_rect_is_accurate: true,
        draws_content: true,
        raster_effect_outset: RasterEffectOutset::kNone,
        record_rect_known_to_be_opaque: PaintRect::default(),
        record_background: PaintChunkBackgroundColorInfo::default(),
        record_has_text: false,
        record_begin: 0,
        record_end: 1,
        scroll_translation: None,
    };
    let items: Arc<Vec<_>> = Arc::new(vec![draw]);
    let dependencies = Arc::new(ImageDependencyIndex::build(
        &items,
        std::slice::from_ref(&display),
        &[],
    ));
    let old_record: Arc<dyn image_resource::DocumentPaintRecord> = Arc::new(1u8);
    let resources = Arc::new(PaintResources {
        images: vec![layoutng_assembly::internal::layout_input::PaintImage {
            id: 7,
            revision: 1,
            width: 80,
            height: 40,
            content: PaintImageContent::Document(Arc::new(
                image_resource::DocumentPaintContent::new(
                    ContainerKey::new(80, 40, 1.0, 1.0),
                    old_record,
                ),
            )),
            ..Default::default()
        }],
        ..Default::default()
    });
    let old = Arc::new(PaintArtifact {
        items: items.clone(),
        display_items: vec![display],
        resources: Some(resources),
        image_dependencies: dependencies,
        recording_revision: 99,
        ..Default::default()
    });
    let mut engine = PaintEngine::new();
    engine.paint_result = Some(old.clone());
    let next_record: Arc<dyn image_resource::DocumentPaintRecord> = Arc::new(2u8);
    let frame = Arc::new(DocumentImageFrame {
        animation_info: image_resource::PaintImageAnimationInfo::confirmed_static(),
        completion_state: image_resource::PaintImageCompletionState::Done,
        resource_id: 7,
        revision: 2,
        intrinsic_size: IntrinsicSize {
            width: 80,
            height: 40,
        },
        container_key: ContainerKey::new(80, 40, 1.0, 1.0),
        record: next_record,
    });
    let result = engine
        .ApplyMutation(PaintMutation::ImageChanged {
            image_id: 7,
            revision: 2,
            frame: frame.clone(),
        })
        .unwrap();
    let next = engine.GetPaintResult().unwrap();
    assert_eq!(result.damaged_records.len(), 1);
    assert_eq!(
        result.damaged_records[0].visual_rect,
        PaintRect {
            x: 32.0,
            y: 48.0,
            width: 80.0,
            height: 40.0
        }
    );
    assert!(Arc::ptr_eq(&old.items, &next.items));
    assert_eq!(old.recording_revision, next.recording_revision);
    assert_eq!(old.resources.as_ref().unwrap().images[0].revision, 1);
    assert_eq!(next.resources.as_ref().unwrap().images[0].revision, 2);
    let subsumed = engine
        .ApplyMutation(PaintMutation::ImageChanged {
            image_id: 7,
            revision: 2,
            frame,
        })
        .expect("the committed immutable revision subsumes its queued invalidation");
    assert!(subsumed.damaged_records.is_empty());
}
use crate::paint_context::PaintContext;
use crate::paint_layer_painter::PaintLayerPainter;
use crate::pre_paint_tree_walk::{PaintFragmentSource, PrePaintTreeWalk};
use std::cell::RefCell;
use std::collections::HashSet;
pub fn NativeFragmentClientRectsByNode(
    root: layoutng_assembly::layout_engine::NativePaintFragmentCursor<'_>,
) -> std::collections::HashMap<u64, Vec<PaintRect>> {
    crate::client_rects::CollectNativeFragmentClientRects(root)
}

// cpp: paint/paint_engine.cc:33-54
fn SameReplayResources(a: &PaintResources, b: &PaintResources) -> bool {
    if a.device_pixel_ratio != b.device_pixel_ratio
        || a.preferred_color_scheme != b.preferred_color_scheme
        || a.layout_zoom_factor != b.layout_zoom_factor
        || a.viewport != b.viewport
        || a.fonts.len() != b.fonts.len()
        || a.images.len() != b.images.len()
    {
        return false;
    }
    let same_fonts = a.fonts.ptr_eq(&b.fonts)
        || a.fonts
            .iter()
            .zip(&b.fonts)
            .all(|(left, right)| left.face_index == right.face_index && left.bytes == right.bytes);
    same_fonts && a.images == b.images
}

// cpp: paint/paint_engine.cc:56-91
fn ValidateReplayResources(resources: Option<&PaintResources>) {
    let Some(resources) = resources else {
        return;
    };
    if !resources.device_pixel_ratio.is_finite()
        || resources.device_pixel_ratio <= 0.0
        || !resources.layout_zoom_factor.is_finite()
        || resources.layout_zoom_factor <= 0.0
        || !resources.ArtifactToDeviceScale().is_finite()
    {
        panic!("PaintResources device and layout scales must be finite and positive");
    }
    if let Some(viewport) = resources.viewport {
        if viewport.size.width < 0 || viewport.size.height < 0 {
            panic!("PaintResources viewport size must be nonnegative");
        }
    }
    for font in &resources.fonts {
        if font.bytes.is_empty() {
            panic!("PaintResources font bytes are empty");
        }
    }
    let mut image_ids = HashSet::new();
    for image in &resources.images {
        let width = image.width as usize;
        let height = image.height as usize;
        let dimensions_fit = width != 0 && height != 0 && width <= usize::MAX / 4 / height;
        let bitmap_valid = image
            .BitmapPixels()
            .is_none_or(|pixels| dimensions_fit && pixels.len() == width * height * 4);
        if image.id == 0
            || !image_ids.insert(image.id)
            || image.width == 0
            || image.height == 0
            || !image.resolution_scale.is_finite()
            || image.resolution_scale <= 0.0
            || !dimensions_fit
            || !bitmap_valid
        {
            panic!("PaintResources images must have unique ids and valid RGBA data");
        }
    }
}

// cpp: paint/paint_engine.cc:93-112
fn ValidateResourceCatalog(
    fragment: PaintFragmentSource<'_>,
    catalog: Option<&PaintResources>,
    validated_catalogs: &mut HashSet<*const PaintResources>,
) {
    if let Some(candidate) = fragment.paint.resources.as_deref() {
        let pointer = candidate as *const PaintResources;
        if !validated_catalogs.contains(&pointer) {
            if catalog.is_none_or(|root| !SameReplayResources(candidate, root)) {
                panic!("all FragmentPaintData paint data must use the root replay resources");
            }
            validated_catalogs.insert(pointer);
        }
    }
    for child in fragment.Children() {
        ValidateResourceCatalog(child, catalog, validated_catalogs);
    }
}
fn RecordPaint(
    pre_paint: &mut PrePaintTreeWalk,
    fragments: PaintFragmentSource<'_>,
    editing: EditingPaintState,
    item_capacity: usize,
    display_item_capacity: usize,
    chunk_capacity: usize,
    previous: Option<Arc<PaintArtifact>>,
) -> (PaintRecording, crate::performance::PaintPerformanceSnapshot) {
    let mut trace = browser_tracing::span("paint", "Paint.RecordPaint");
    ValidateReplayResources(fragments.paint.resources.as_deref());
    let mut validated_catalogs = HashSet::new();
    if let Some(catalog) = fragments.paint.resources.as_deref() {
        validated_catalogs.insert(catalog as *const PaintResources);
    }
    ValidateResourceCatalog(
        fragments,
        fragments.paint.resources.as_deref(),
        &mut validated_catalogs,
    );
    let paint_tree = {
        let mut trace = browser_tracing::span("prepaint", "PrePaint.Walk");
        let tree = pre_paint.Walk(fragments);
        let stats = pre_paint.stats;
        trace.set("computed_nodes", stats.computed_nodes as f64);
        trace.set("reused_nodes", stats.reused_nodes as f64);
        trace.set(
            "invalidation_only_nodes",
            stats.invalidation_only_nodes as f64,
        );
        trace.set("geometry_only_nodes", stats.geometry_only_nodes as f64);
        trace.set(
            "same_geometry_rebuilds",
            stats.same_geometry_rebuilds as f64,
        );
        trace.set(
            "property_geometry_changes",
            stats.property_geometry_changes as f64,
        );
        trace.set(
            "property_geometry_changes_with_own_keys",
            stats.property_geometry_changes_with_own_keys as f64,
        );
        trace.set(
            "property_geometry_changes_needing_property_update",
            stats.property_geometry_changes_needing_property_update as f64,
        );
        trace.set("skipped_subtrees", stats.skipped_subtrees as f64);
        trace.set("post_walk", stats.post_walk as u8 as f64);
        tree
    };
    let mut output = PaintRecording::default();
    output.compositor_viewport = pre_paint.ExportCompositorViewport(&fragments);
    Arc::make_mut(&mut output.items).reserve(item_capacity);
    output.display_items.reserve(display_item_capacity);
    output.chunks.reserve(chunk_capacity);
    output.resources = fragments.paint.resources.clone();
    // Resolve caret ownership through Layout's native fragment links. The
    // resident PaintTree stores derived paint state, never source topology.
    fn caret_owner(
        node: PaintFragmentSource<'_>,
        id: u64,
    ) -> Option<layoutng_assembly::layout_engine::NativePaintFragmentCursor<'_>> {
        if node.node_id == id && node.paint.text_control_caret_metrics.is_some() {
            return Some(node.Native());
        }
        node.Children()
            .into_iter()
            .find_map(|child| caret_owner(child, id))
    }
    let caret_local_rect = editing.caret.and_then(|state| {
        layoutng_assembly::caret::geometry::TextControlCaretRect(
            caret_owner(fragments, state.node_id)?,
            state.offset,
            state.empty,
        )
    });
    // Chromium keeps ShapeResult on FragmentItem and asks it for range
    // geometry only while painting an active selection/composition. Resolve
    // those few ranges through the live native cursors before PrePaint retains
    // its derived tree; ordinary resize publishes no per-character arrays.
    fn editing_text_range_rects(
        node: PaintFragmentSource<'_>,
        editing: EditingPaintState,
        output: &mut Vec<(
            u64,
            (u32, u32),
            layoutng_assembly::caret::geometry::LocalCaretRect,
        )>,
    ) {
        if matches!(
            node.kind,
            layoutng_assembly::fragment_tree::FragmentKind::kText
                | layoutng_assembly::fragment_tree::FragmentKind::kGeneratedText
        ) {
            let mut ranges = Vec::with_capacity(2);
            if let Some(caret) = editing
                .caret
                .filter(|caret| node.paint.text_control_host == Some(caret.node_id))
            {
                if let Some(range) = caret.selection {
                    ranges.push(range);
                }
                if let Some(range) = caret.composition {
                    ranges.push(range);
                }
            } else if node.paint.text_control_host.is_none() && !node.paint.style.user_select_none {
                if let Some(selection) = editing
                    .document_selection
                    .filter(|selection| !selection.Collapsed())
                {
                    let (start, end) = selection.Ordered();
                    let order = node.paint.logical_tree_order;
                    if order >= start.tree_order && order <= end.tree_order {
                        if let (Some(mut range_start), Some(mut range_end)) =
                            (node.text_start, node.text_end)
                        {
                            if order == start.tree_order {
                                range_start = range_start.max(start.offset);
                            }
                            if order == end.tree_order {
                                range_end = range_end.min(end.offset);
                            }
                            if range_start < range_end {
                                ranges.push((range_start, range_end));
                            }
                        }
                    }
                }
            }
            for range in ranges {
                if let Some(rect) = node.Native().TextRangeRect(range.0, range.1) {
                    output.push((
                        node.fragment_instance_id,
                        range,
                        layoutng_assembly::caret::geometry::LocalCaretRect {
                            offset: Offset {
                                x: rect.x,
                                y: rect.y,
                            },
                            size: Size {
                                width: rect.width,
                                height: rect.height,
                            },
                        },
                    ));
                }
            }
        }
        for child in node.Children() {
            editing_text_range_rects(child, editing, output);
        }
    }
    let mut text_range_rects = Vec::new();
    if editing
        .caret
        .is_some_and(|caret| caret.selection.is_some() || caret.composition.is_some())
        || editing
            .document_selection
            .is_some_and(|selection| !selection.Collapsed())
    {
        editing_text_range_rects(fragments, editing, &mut text_range_rects);
    }
    let mut paint_context = PaintContext::WithEditingAndCache(&mut output, editing, previous);
    paint_context.BindNativeTextSources(fragments.Native());
    paint_context.caret_local_rect = caret_local_rect;
    paint_context.text_range_rects = text_range_rects;
    let context = RefCell::new(paint_context);
    context
        .borrow_mut()
        .SetNextPropertyId(pre_paint.next_property_id);
    {
        // Traversal drives recording, so the enclosing duration belongs to
        // paint_walk. Recording work remains attributable to paint_record by
        // its crate symbols and the work-volume summary emitted below.
        let mut trace = browser_tracing::span("paint_walk", "PaintWalk.Traverse");
        PaintLayerPainter::new(&paint_tree, &context, Vec::new()).Paint();
        let walk = context.borrow().walk_metrics;
        trace.set("layers", walk.layer_visits as f64);
        trace.set("phase_visits", walk.phase_visits as f64);
        trace.set("skipped_hidden", walk.skipped_hidden as f64);
        trace.set("skipped_phase", walk.skipped_phase as f64);
    }
    {
        let _trace = browser_tracing::span("paint_chunk", "PaintChunk.Finish");
        context.borrow_mut().Finish();
    }
    trace.set(
        "cached_display_items",
        context.borrow().cached_display_items as f64,
    );
    trace.set("cache_candidates", context.borrow().cache_candidates as f64);
    trace.set("cache_attempts", context.borrow().cache_attempts as f64);
    trace.set(
        "cache_rejected_uncacheable",
        context.borrow().cache_rejected_uncacheable as f64,
    );
    trace.set(
        "cache_rejected_just_created",
        context.borrow().cache_rejected_just_created as f64,
    );
    trace.set(
        "cache_rejected_invalid",
        context.borrow().cache_rejected_invalid as f64,
    );
    trace.set(
        "cache_rejected_geometry",
        context.borrow().cache_rejected_geometry as f64,
    );
    trace.set(
        "cache_rejected_missing_id",
        context.borrow().cache_rejected_missing_id as f64,
    );
    trace.set(
        "cache_rejected_missing_previous",
        context.borrow().cache_rejected_missing_previous as f64,
    );
    let reuse = paint_reuse::PaintReuseMetrics {
        candidates: context.borrow().cache_candidates,
        attempts: context.borrow().cache_attempts,
        reused: context.borrow().cached_display_items,
        rejected_uncacheable: context.borrow().cache_rejected_uncacheable,
        rejected_just_created: context.borrow().cache_rejected_just_created,
        rejected_invalid: context.borrow().cache_rejected_invalid,
        rejected_geometry: context.borrow().cache_rejected_geometry,
        rejected_missing_id: context.borrow().cache_rejected_missing_id,
        rejected_missing_previous: context.borrow().cache_rejected_missing_previous,
    };
    let walk = context.borrow().walk_metrics;
    browser_tracing::instant(
        "paint_reuse",
        "PaintReuse.Summary",
        &[
            ("candidates", reuse.candidates as f64),
            ("attempts", reuse.attempts as f64),
            ("reused", reuse.reused as f64),
            ("hit_rate", reuse.hit_rate()),
            ("invalid", reuse.rejected_invalid as f64),
            ("geometry", reuse.rejected_geometry as f64),
            ("missing_id", reuse.rejected_missing_id as f64),
            ("missing_previous", reuse.rejected_missing_previous as f64),
        ],
    );
    drop(context);
    let record = paint_record::PaintRecordMetrics {
        paint_ops: output.items.len(),
        display_items: output.display_items.len(),
    };
    let chunk = paint_chunk::MeasureChunks(
        output
            .chunks
            .iter()
            .map(|chunk| (chunk.begin_index, chunk.end_index)),
    );
    browser_tracing::instant(
        "paint_record",
        "PaintRecord.Summary",
        &[
            ("paint_ops", record.paint_ops as f64),
            ("display_items", record.display_items as f64),
        ],
    );
    browser_tracing::instant(
        "paint_chunk",
        "PaintChunk.Summary",
        &[
            ("chunks", chunk.chunks as f64),
            ("display_items", chunk.display_items as f64),
            ("max_items", chunk.max_items_per_chunk as f64),
        ],
    );
    {
        let _trace = browser_tracing::span("paint_commit", "PaintCommit.BuildMetadata");
        paint_commit::FinalizeArtifact(&mut output);
    }
    {
        let _trace = browser_tracing::span("prepaint", "PrePaint.RetainTree");
        pre_paint.RetainTree(paint_tree);
    }
    trace.set("items", output.items.len() as f64);
    trace.set("display_items", output.display_items.len() as f64);
    trace.set("chunks", output.chunks.len() as f64);
    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
        let stats = pre_paint.stats;
        eprintln!(
            "prepaint-update computed_nodes={} reused_nodes={} invalidation_only_nodes={} geometry_only_nodes={} skipped_subtrees={} post_walk={}",
            stats.computed_nodes,
            stats.reused_nodes,
            stats.invalidation_only_nodes,
            stats.geometry_only_nodes,
            stats.skipped_subtrees,
            stats.post_walk
        );
    }
    (
        output,
        crate::performance::PaintPerformanceSnapshot {
            prepaint: pre_paint.stats,
            walk,
            reuse,
            record,
            chunk,
            commit: Default::default(),
        },
    )
}

/// Resident paint facade. PrePaint's property ownership directory and the
/// committed artifact have independent lifetimes, like ObjectPaintProperties
/// and PaintControllerPersistentData in Blink. Completed nodes retain local
/// geometry plus the shared immutable Layout paint payload; no layout-root
/// owner or borrowed snapshot is retained.
#[derive(Default)]
pub struct PaintEngine {
    pre_paint: PrePaintTreeWalk,
    retained_scroll_state: Option<crate::scroll_paint_update::RetainedScrollPaintState>,
    retained_scroll_state_initialized: bool,
    paint_result: Option<Arc<PaintArtifact>>,
    performance: crate::performance::PaintPerformanceSnapshot,
    animation_targeted_nodes: Vec<u64>,
}
impl PaintEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ApplyAnimationMutation(&mut self, mutation: animation::PaintMutation) {
        match mutation {
            animation::PaintMutation::SetTargetedProperties { node_ids } => {
                self.animation_targeted_nodes = node_ids;
            }
            animation::PaintMutation::Commit => self.animation_targeted_nodes.clear(),
        }
    }

    pub fn AnimationTargetedNodes(&self) -> &[u64] {
        &self.animation_targeted_nodes
    }

    /// Update resident properties and record a new artifact. Publish only once
    /// this pass completes; a failed paint cannot expose the previous result
    /// as if it represented the new inputs.
    pub fn Paint(
        &mut self,
        layout_output: &layoutng_assembly::layout_engine::LayoutPaintOutput,
        editing: EditingPaintState,
    ) {
        let _trace = browser_tracing::span("paint", "PaintEngine.Paint");
        let layout_result = PaintFragmentSource::FromNative(layout_output.PaintCursor());
        let previous = self.paint_result.clone();
        let capacities = previous.as_ref().map_or((0, 0, 0), |old| {
            (old.items.len(), old.display_items.len(), old.chunks.len())
        });
        self.pre_paint.SetRetainedNativeOwnerLifetimes(false);
        let (result, performance) = RecordPaint(
            &mut self.pre_paint,
            layout_result,
            editing,
            capacities.0,
            capacities.1,
            capacities.2,
            previous,
        );
        if self.pre_paint.stats.post_walk || self.retained_scroll_state.is_none() {
            self.retained_scroll_state = None;
            self.retained_scroll_state_initialized = false;
        }
        self.performance = performance;
        self.paint_result = Some(paint_commit::CommitArtifact(result));
    }

    pub fn GetPrePaintUpdateStats(&self) -> crate::pre_paint_tree_walk::PrePaintUpdateStats {
        self.pre_paint.stats
    }

    pub fn GetPaintResult(&self) -> Option<&Arc<PaintArtifact>> {
        self.paint_result.as_ref()
    }

    pub fn GetPerformanceSnapshot(&self) -> crate::performance::PaintPerformanceSnapshot {
        self.performance
    }

    /// Publish a new immutable resource catalog while retaining the recorded
    /// display items and paint properties. This is the document-image analogue
    /// of ImageResourceContent invalidation: stable resource ids let the
    /// compositor damage only records that sample changed pixels.
    pub fn UpdatePaintResources(&mut self, resources: Arc<PaintResources>) -> bool {
        ValidateReplayResources(Some(&resources));
        let Some(artifact) = self.paint_result.as_mut() else {
            return false;
        };
        if artifact
            .resources
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &resources))
        {
            return true;
        }
        let artifact = Arc::make_mut(artifact);
        artifact.ReplaceResources(resources);
        true
    }

    /// Apply a content-only image mutation to the committed artifact. Stable
    /// display items remain shared; the retained dependency index identifies
    /// exactly which paint clients LayerTile must invalidate.
    pub fn ApplyMutation(&mut self, mutation: PaintMutation) -> Option<PaintMutationResult> {
        let artifact = self.paint_result.as_mut()?;
        let artifact = Arc::make_mut(artifact);
        artifact.ApplyResourceMutation(mutation)
    }

    /// Reconcile resource-only changes with the artifact currently published
    /// by Page. This keeps Arc COW/adoption and resource revision validation
    /// inside PaintEngine while older compositor snapshots remain immutable.
    pub fn PublishResourceUpdate(
        &mut self,
        artifact: Arc<PaintArtifact>,
        update: PaintResourceUpdate,
    ) -> Option<Arc<PaintArtifact>> {
        self.paint_result = Some(artifact);
        match update {
            PaintResourceUpdate::Snapshot(resources) => {
                self.UpdatePaintResources(resources).then_some(())?;
            }
            PaintResourceUpdate::Mutations(mutations) => {
                for mutation in mutations {
                    self.ApplyMutation(mutation)?;
                }
            }
        }
        self.paint_result.clone()
    }

    /// Complete Paint-owned client validation and publish an immutable output.
    /// Layout consumes the opaque client receipt at its own boundary; Page
    /// never receives mutable access to the artifact or the client set.
    pub fn CommitPaintResult(&mut self) -> PaintOutput {
        let mut trace = browser_tracing::span("paint_commit", "PaintCommit.Publish");
        let artifact = Arc::make_mut(
            self.paint_result
                .as_mut()
                .expect("Paint must complete before commit"),
        );
        let clients = paint_commit::CollectClientIds(
            artifact.display_items.iter().map(|item| item.id),
            artifact.chunks.iter().map(|chunk| chunk.id),
        );
        self.performance.commit = paint_commit::PaintCommitMetrics {
            clients: clients.len(),
            recording_revision: artifact.recording_revision,
        };
        trace.set("clients", clients.len() as f64);
        trace.set("recording_revision", artifact.recording_revision as f64);
        // cpp: platform/graphics/paint/paint_controller.cc:141-143
        for chunk in &mut artifact.chunks {
            chunk.client_is_just_created = false;
        }
        self.pre_paint.CommitClients(clients.iter().copied());
        PaintOutput::new(
            self.paint_result
                .as_ref()
                .expect("committed Paint result")
                .clone(),
            layoutng_assembly::layout_engine::PaintClientCommit::FromClientSet(clients),
        )
    }

    /// Move Page's current frame reference back before updating the artifact.
    /// This prevents Engine/Page sharing alone from causing a deep copy.
    #[doc(hidden)]
    pub fn AdoptPaintResult(&mut self, artifact: Arc<PaintArtifact>) {
        self.paint_result = Some(artifact);
    }

    /// Requires the native retained-geometry proof from Page, just as the
    /// existing scroll update path. A rejection leaves the artifact unchanged.
    #[doc(hidden)]
    pub fn TryUpdateScrollProperties(
        &mut self,
        layout_output: &layoutng_assembly::layout_engine::LayoutPaintOutput,
    ) -> Result<
        crate::scroll_paint_update::ScrollPropertyUpdate,
        crate::scroll_paint_update::ScrollPaintInvalidation,
    > {
        let mut trace = browser_tracing::span("paint", "PaintEngine.TryUpdateScrollProperties");
        trace.set("records_reused", 0.0);
        let Some(artifact) = self.paint_result.as_mut() else {
            return Err(crate::scroll_paint_update::ScrollPaintInvalidation::MissingCoverage);
        };
        trace.set("artifact_strong_count", Arc::strong_count(artifact) as f64);
        // Native scroll refresh versions the changed nodes and ancestor paths.
        // Keep the resident tree as the old geometry/context baseline: the
        // next ordinary Walk must compare it, not force unrelated subtrees.
        let result = crate::scroll_paint_update::TryUpdateScrollPropertiesWithPrePaint(
            PaintFragmentSource::FromNative(layout_output.PaintCursor()),
            Arc::make_mut(artifact),
            &mut self.pre_paint,
            &mut self.retained_scroll_state,
            &mut self.retained_scroll_state_initialized,
        );
        trace.set("records_reused", result.is_ok() as u8 as f64);
        if let Ok(update) = &result {
            trace.set("updated_chunks", update.updated_chunks as f64);
            trace.set(
                "updated_scrollbar_records",
                update.updated_scrollbar_records as f64,
            );
        }
        result
    }
}
