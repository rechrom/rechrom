#![allow(non_camel_case_types)]

//! Source paint display-list ABI. The entry point and private painters are
//! connected after their owning source files have been translated.

use std::sync::Arc;

use layoutng_assembly::fragment_tree::PaintResources;
pub use layoutng_assembly::fragment_tree::RasterEffectOutset;

pub use layoutng_assembly::internal::paint_input::{
    PaintCornerRadii, PaintFilterOperation, PaintMaskComposite, PaintMaskMode, PaintShader,
    PaintShaderKind, SvgPatternCommand, SvgPatternData, SvgPatternImage, SvgStrokeLineCap,
    SvgStrokeLineJoin,
};

pub use paint_model::PaintPhase;

pub use paint_record::paint_context::PaintContext;
pub use paint_record::PaintOpType as DisplayItemType;

pub use paint_model::PaintRect;

pub use paint_record::{
    CaretGeometry, CaretPosition, DecorationPaintPass, DisplayItem, DisplayItemList,
    DisplayMaskLayer, EditingPaintState, ForeignLayer, ForeignLayerDisplayItemData,
    ForeignLayerSize, NewForeignLayerDisplayItem, RecordForeignLayer, RecordedDisplayItem,
    RecordedDisplayItemKind, ScrollbarDisplayItemData,
};
use paint_record::{PaintControllerPersistentData, PaintRecording};

// Public producer analysis also serves canonical PaintRecords downstream.
pub use image_resource::{
    downcast_document_paint_record, ContainerKey as DocumentImageContainerKey,
    ImageDecodingMode as PaintImageDecodingMode, PaintImageAnimationMetadata, PaintImageContent,
    PaintImageId as PaintImageResourceId,
    PaintImageMetadataUnavailable as ImageMetadataUnavailable,
};
pub use layoutng_assembly::internal::layout_input::PaintImage as LayoutPaintImage;
pub use paint_record::drawing_display_item::CalculateRectKnownToBeOpaqueForRecord;

pub use crate::display_item_id::DisplayItemId as PaintChunkId;
pub use paint_chunk::{
    PaintChunk, PaintChunkBackgroundColorInfo, PaintChunkCompositorData, PaintChunks,
    PaintFrameToken, PaintHitTestData, PaintHitTestOpaqueness, PaintLayerSelectionData,
    PaintRegionCaptureData, PaintSelectionBound, PaintSelectionBoundType, PaintTrackedElementRect,
    PaintTrackedElementRects, PaintTrackedElementRelativeRect, PaintTrackedElementSubRect,
    PaintTrackedElementSubRectType,
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
            images: vec![LayoutPaintImage {
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
        chunks: vec![PaintChunk::default()].into(),
        ..Default::default()
    });
    let mut scrolling = old.clone();
    let updated = Arc::make_mut(&mut scrolling);
    // This ownership test models a new property snapshot only; it invokes no
    // source identity, layout or recording algorithm.
    Arc::make_mut(&mut updated.chunks[0].properties.transform)
        .matrix
        .values[13] = -12.0;
    assert!(
        Arc::ptr_eq(&old.items, &scrolling.items),
        "property changes share the immutable PaintRecord operations"
    );
    assert!(old.display_items.PtrEq(&scrolling.display_items));
    assert!(
        !old.chunks.PtrEq(&scrolling.chunks),
        "only the changed chunk directory detaches"
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
        scrollbar: None,
        foreign_layer: None,
        kind: RecordedDisplayItemKind::Drawing,
        id: Default::default(),
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
        display_items: vec![display].into(),
        resources: Some(resources),
        image_dependencies: dependencies,
        recording_revision: 99,
        ..Default::default()
    });
    let mut engine = PaintEngine::new();
    engine.persistent_data.AdoptPaintArtifact(old.clone());
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
    assert!(old.display_items.PtrEq(&next.display_items));
    assert!(old.chunks.PtrEq(&next.chunks));
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
use crate::paint_layer_painter::PaintLayerPainter;
use crate::pre_paint_tree_walk::{PaintFragmentSource, PrePaintTreeWalk};
use std::cell::RefCell;
use std::collections::HashSet;
pub fn FragmentClientRectsByNode(
    root: layoutng_assembly::layout_engine::PaintFragmentCursor<'_>,
) -> std::collections::HashMap<u64, Vec<PaintRect>> {
    crate::client_rects::CollectFragmentClientRects(root)
}

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

fn RecordPaint(
    pre_paint: &mut PrePaintTreeWalk,
    fragments: PaintFragmentSource<'_>,
    source_lease: layoutng_assembly::layout_engine::PaintFragmentLease,
    editing: EditingPaintState,
    previous: Option<Arc<PaintArtifact>>,
    previous_subsequences: paint_record::SubsequencesData,
) -> (
    PaintRecording,
    paint_record::SubsequencesData,
    crate::performance::PaintPerformanceSnapshot,
) {
    let mut trace = browser_tracing::span("paint", "Paint.RecordPaint");
    ValidateReplayResources(fragments.paint.resources.as_deref());
    let paint_tree = {
        let mut trace = browser_tracing::span("prepaint", "PrePaint.Walk");
        let tree = pre_paint.Walk(fragments, source_lease);
        let stats = pre_paint.stats;
        trace.set("computed_nodes", stats.computed_nodes as f64);
        trace.set("reused_nodes", stats.reused_nodes as f64);
        trace.set(
            "invalidation_only_nodes",
            stats.invalidation_only_nodes as f64,
        );
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
    output.resources = fragments.paint.resources.clone();
    // Resolve caret ownership through Layout's source fragment links. The
    // resident PaintTree stores derived paint state, never source topology.
    fn caret_owner(
        node: PaintFragmentSource<'_>,
        id: u64,
    ) -> Option<layoutng_assembly::layout_engine::PaintFragmentCursor<'_>> {
        if node.node_id == id && node.paint.text_control_caret_metrics.is_some() {
            return Some(node.Cursor());
        }
        node.Children()
            .into_iter()
            .find_map(|child| caret_owner(child, id))
    }
    let caret_geometry = editing.caret.and_then(|state| {
        let owner = caret_owner(fragments, state.node_id)?;
        // core/layout/inline/caret_rect.cc:426-430 keeps the insertion caret
        // as a bar while an IME composition is active.
        let shape = if state.composition.is_some() {
            layoutng_assembly::internal::caret_rect::CaretShape::kBar
        } else {
            owner.paint.style.caret_shape()
        };
        let rect = layoutng_assembly::caret::geometry::TextControlCaretRect(
            owner,
            state.offset,
            state.empty,
            shape,
        )?;
        Some((rect, shape))
    });
    let mut paint_context = PaintContext::WithEditingAndPersistentCache(
        &mut output,
        editing,
        previous,
        previous_subsequences,
    );
    // RecordPaint's root cursor is the documentElement Layout result. Preserve
    // its exact computed opacity as the document-global Paint Timing fact used
    // by every nested PaintLayerPainter scope.
    paint_context.SetDocumentElementIsInvisible(fragments.paint.style.opacity() == 0.0);
    paint_context.BindTextSources(fragments.Cursor());
    if let Some((rect, shape)) = caret_geometry {
        paint_context.caret_local_rect = Some(rect);
        paint_context.caret_shape = Some(shape);
    }
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
    let subsequences = {
        let _trace = browser_tracing::span("paint_chunk", "PaintChunk.Finish");
        context.borrow_mut().Finish()
    };
    trace.set(
        "cached_display_items",
        context.borrow().CacheMetrics().reused as f64,
    );
    trace.set(
        "cache_candidates",
        context.borrow().CacheMetrics().candidates as f64,
    );
    trace.set(
        "cache_attempts",
        context.borrow().CacheMetrics().attempts as f64,
    );
    trace.set(
        "cache_rejected_uncacheable",
        context.borrow().CacheMetrics().rejected_uncacheable as f64,
    );
    trace.set(
        "cache_rejected_just_created",
        context.borrow().CacheMetrics().rejected_just_created as f64,
    );
    trace.set(
        "cache_rejected_invalid",
        context.borrow().CacheMetrics().rejected_invalid as f64,
    );
    trace.set(
        "cache_rejected_missing_id",
        context.borrow().CacheMetrics().rejected_missing_id as f64,
    );
    trace.set(
        "cache_rejected_missing_previous",
        context.borrow().CacheMetrics().rejected_missing_previous as f64,
    );
    let reuse = context.borrow().CacheMetrics();
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
            ("missing_id", reuse.rejected_missing_id as f64),
            ("missing_previous", reuse.rejected_missing_previous as f64),
        ],
    );
    drop(context);
    // Source: PaintController::CommitNewDisplayItems,
    // PaintController::AppendSubsequenceByMoving and ObjectPaintProperties.
    // Cached source objects keep pointers to nodes that Chromium updates in
    // place. Publish their immutable Rust equivalents against this pass's
    // current owner slots, including wrapper ancestry and transform sidecars.
    if reuse.reused != 0
        || output
            .chunks
            .iter()
            .any(|chunk| chunk.is_moved_from_cached_subsequence)
    {
        let _trace = browser_tracing::span("paint_commit", "PaintCommit.RebindPropertyStates");
        let mut rebinder = paint_model::paint_property_tree::ArtifactPropertyStateRebinder::new(
            &pre_paint.property_node_store,
        );
        for chunk in &mut output.chunks {
            // Current/home chunks can share an owner node whose immutable
            // payload still points at an earlier ancestor revision, while the
            // layer also contains a moved/guest chunk. Canonicalize both sides
            // of that ancestry comparison in this publication pass.
            rebinder.RebindState(&mut chunk.properties);
            if let Some(scroll_translation) = chunk
                .compositor
                .as_mut()
                .and_then(|data| data.hit_test_data.as_mut())
                .and_then(|data| data.scroll_translation.as_mut())
            {
                *scroll_translation = rebinder.RebindTransform(scroll_translation);
            }
        }
        // Individual cached drawings are moved without a moved-subsequence
        // marker. Their DisplayItem-owned scroll translation still observes
        // ObjectPaintProperties in place in Chromium.
        for item in &mut output.display_items {
            if let Some(scroll_translation) = item.scroll_translation.as_mut() {
                *scroll_translation = rebinder.RebindTransform(scroll_translation);
            }
        }
    }
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
            "prepaint-update computed_nodes={} reused_nodes={} invalidation_only_nodes={} skipped_subtrees={} post_walk={}",
            stats.computed_nodes,
            stats.reused_nodes,
            stats.invalidation_only_nodes,
            stats.skipped_subtrees,
            stats.post_walk
        );
    }
    (
        output,
        subsequences,
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
    persistent_data: PaintControllerPersistentData,
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
        let layout_result = PaintFragmentSource::FromCursor(layout_output.PaintCursor());
        let source_lease = layout_output.PaintLease();
        let previous = self.persistent_data.GetPaintArtifact().cloned();
        let previous_subsequences = self.persistent_data.GetSubsequences().clone();
        self.pre_paint.SetRetainedOwnerLifetimes(false);
        let (result, subsequences, performance) = RecordPaint(
            &mut self.pre_paint,
            layout_result,
            source_lease,
            editing,
            previous,
            previous_subsequences,
        );
        if self.pre_paint.stats.post_walk || self.retained_scroll_state.is_none() {
            self.retained_scroll_state = None;
            self.retained_scroll_state_initialized = false;
        }
        self.performance = performance;
        self.persistent_data.CommitNewDisplayItemsWithSubsequences(
            paint_commit::CommitArtifact(result),
            subsequences,
        );
    }

    pub fn GetPrePaintUpdateStats(&self) -> crate::pre_paint_tree_walk::PrePaintUpdateStats {
        self.pre_paint.stats
    }

    pub fn GetPaintResult(&self) -> Option<&Arc<PaintArtifact>> {
        self.persistent_data.GetPaintArtifact()
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
        let Some(artifact) = self.persistent_data.GetPaintArtifactMut() else {
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
        let artifact = self.persistent_data.GetPaintArtifactMut()?;
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
        self.persistent_data.AdoptPaintArtifact(artifact);
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
        self.persistent_data.GetPaintArtifact().cloned()
    }

    /// Complete Paint-owned client validation and publish an immutable output.
    /// Layout consumes the opaque client receipt at its own boundary; Page
    /// never receives mutable access to the artifact or the client set.
    pub fn CommitPaintResult(&mut self) -> PaintOutput {
        let mut trace = browser_tracing::span("paint_commit", "PaintCommit.Publish");
        let artifact = Arc::make_mut(
            self.persistent_data
                .GetPaintArtifactMut()
                .expect("Paint must complete before commit"),
        );
        let clients = crate::PaintController::ValidateForCommittedPaint(artifact);
        self.performance.commit = paint_commit::PaintCommitMetrics {
            clients: clients.len(),
            recording_revision: artifact.recording_revision,
        };
        trace.set("clients", clients.len() as f64);
        trace.set("recording_revision", artifact.recording_revision as f64);
        self.pre_paint.CommitClients(clients.iter().copied());
        PaintOutput::new(
            self.persistent_data
                .GetPaintArtifact()
                .expect("committed Paint result")
                .clone(),
            layoutng_assembly::layout_engine::PaintClientCommit::FromClientSetAndDelayedInvalidations(
                clients,
                self.pre_paint
                    .PendingDelayedPaintInvalidations()
                    .clone(),
            ),
        )
    }

    /// Move Page's current frame reference back before updating the artifact.
    /// This prevents Engine/Page sharing alone from causing a deep copy.
    #[doc(hidden)]
    pub fn AdoptPaintResult(&mut self, artifact: Arc<PaintArtifact>) {
        self.persistent_data.AdoptPaintArtifact(artifact);
    }

    /// Requires the source retained-geometry proof from Page, just as the
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
        let Some(artifact) = self.persistent_data.GetPaintArtifactMut() else {
            return Err(crate::scroll_paint_update::ScrollPaintInvalidation::MissingCoverage);
        };
        trace.set("artifact_strong_count", Arc::strong_count(artifact) as f64);
        // Chromium scroll refresh versions the changed nodes and ancestor paths.
        // Keep the resident tree as the old geometry/context baseline: the
        // next ordinary Walk must compare it, not force unrelated subtrees.
        let result = crate::scroll_paint_update::TryUpdateScrollPropertiesWithPrePaint(
            PaintFragmentSource::FromCursor(layout_output.PaintCursor()),
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
