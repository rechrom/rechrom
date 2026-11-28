#![allow(non_camel_case_types)]

//! Source paint display-list ABI. The entry point and private painters are
//! connected after their owning source files have been translated.

use layout::fragment::ResidentBoxGeometry as _;
use std::sync::Arc;
pub mod drag_image;

// Immutable resource catalog retained by compositor PaintRecord leases.
pub use paint_input_types::PaintResources;
pub use layout::paint::RasterEffectOutset;

pub use paint_input_types::paint_input::{
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
    ImageDecodingMode as PaintImageDecodingMode, PaintImageAlphaType, PaintImageAnimationMetadata,
    PaintImageContent, PaintImageId as PaintImageResourceId,
    PaintImageMetadataUnavailable as ImageMetadataUnavailable,
};
pub use paint_input_types::layout_input::PaintImage as LayoutPaintImage;
pub use paint_record::drawing_display_item::CalculateRectKnownToBeOpaqueForRecord;

pub use crate::display_item_id::DisplayItemId as PaintChunkId;
pub use paint_chunk::{
    PaintChunk, PaintChunkBackgroundColorInfo, PaintChunkCompositorData, PaintChunks,
    PaintFrameToken, PaintHitTestData, PaintHitTestOpaqueness, PaintLayerSelectionData,
    PaintRegionCaptureData, PaintSelectionBound, PaintSelectionBoundType, PaintTrackedElementRect,
    PaintTrackedElementRects, PaintTrackedElementRelativeRect, PaintTrackedElementSubRect,
    PaintTrackedElementSubRectType,
};

pub use paint_commit::{artifact_access, artifact_writer, ArtifactAccess, ArtifactAccessScope};
pub use paint_commit::{
    DocumentPaintArtifactRecord, DocumentRecordArtifact, NewDocumentPaintArtifactRecord,
    PaintRecordForArtifact, PaintRecordForItem, ImageDependencyIndex, ImagePaintDependency, PaintArtifact,
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
        Arc::new(NewDocumentPaintArtifactRecord(
            Arc::new(PaintArtifact::default()),
            image_resource::IntrinsicSize {
                width: key.width,
                height: key.height,
            },
            image_resource::IntrinsicSize {
                width: key.width,
                height: key.height,
            },
            key.clone(),
        )) as Arc<dyn image_resource::DocumentPaintRecord>
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
            }].into(),
            ..Default::default()
        })),
        ..Default::default()
    };
    assert!(!artifact.HasExactDocumentImageContainerRecords());

    let resources = Arc::make_mut(artifact.resources.as_mut().unwrap());
    Arc::make_mut(&mut resources.images)[0].content = PaintImageContent::Document(Arc::new(
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
    Arc::make_mut(&mut updated.chunks.make_mut()[0].properties.transform)
        .state_mut()
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
    let artifacts = artifact_access();
    paint_model::property_cell::with_access(|a| {
        assert_eq!(old.chunks.get(&artifacts)[0].properties.transform.Matrix(a).values[13], 0.0);
        assert_eq!(
            scrolling.chunks.get(&artifacts)[0].properties.transform.Matrix(a).values[13],
            -12.0
        );
    });
    drop(artifacts);
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
        record_source: None,
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
        images: vec![paint_input_types::layout_input::PaintImage {
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
        }].into(),
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
    engine.persistent_data.CommitNewDisplayItems(old.clone());
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
    root: layout::paint::PaintFragmentCursor<'_>,
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
    for image in resources.images.iter() {
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

/// Immediate lifecycle notifications at the owning PrePaint/Paint commit
/// points. The receiver has no DOM, Layout or rendering operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintLifecyclePhase {
    InPrePaint,
    PrePaintClean,
    InPaint,
    PaintClean,
}
pub trait PaintLifecycleObserver {
    fn AdvanceTo(&self, phase: PaintLifecyclePhase);
    fn SetIntersectionObservationState(
        &self,
        state: foundation::intersection_observation_state::IntersectionObservationState,
    );
}

/// Call-scoped borrow of the native same-Page child LocalFrameView. Layout
/// retains geometry and FragmentPaintData; Paint retains only opaque leases.
pub struct LocalFrameViewPaintInput<'a> {
    pub owner: layout::paint::PaintFragmentCursor<'a>,
    pub layout: &'a layout::paint::LayoutPaintOutput,
    pub is_cross_origin_to_parent: bool,
    pub blocking_wheel_targets: Arc<[u64]>,
    pub lifecycle: &'a dyn PaintLifecycleObserver,
}

fn WalkPrePaint(
    pre_paint: &mut PrePaintTreeWalk,
    fragments: PaintFragmentSource<'_>,
    source_lease: layout::paint::PaintFragmentLease,
) -> Box<crate::pre_paint_tree_walk::PaintTreeNode<'static>> {
    // Chromium refreshes PaintLayer descendant-dependent flags immediately
    // before PrePaint. Doing this at Layout publication alone is too early:
    // later lifecycle work may dirty the layer tree before this walk.
    fragments.Cursor().UpdateRootLayerDescendantDependentFlags();
    {
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
        browser_tracing::instant(
            "prepaint",
            "PrePaint.Rebind",
            &[
                ("rebind_walk", stats.rebind_walk as u8 as f64),
                ("rebound_nodes", stats.rebound_nodes as f64),
                ("post_walk", stats.post_walk as u8 as f64),
            ],
        );
        let ms = |ns: u64| ns as f64 / 1e6;
        let shift_parts = prepaint::pre_paint_tree_walk::TakeShiftPartNs();
        browser_tracing::instant(
            "prepaint",
            "PrePaint.NodeProfile",
            &[
                ("admission_ms", ms(stats.profile_admission_ns)),
                ("build_ms", ms(stats.profile_build_ns)),
                ("invalidation_ms", ms(stats.profile_invalidation_ns)),
                ("children_setup_ms", ms(stats.profile_children_setup_ns)),
                ("child_loop_ms", ms(stats.profile_child_loop_ns)),
                ("tail_ms", ms(stats.profile_tail_ns)),
            ],
        );
        browser_tracing::instant(
            "prepaint",
            "PrePaint.Shift",
            &[
                ("shifted_nodes", stats.shifted_nodes as f64),
                ("check_mismatches", stats.shift_check_mismatches as f64),
                ("branch_ms", ms(stats.shift_branch_ns)),
                ("refresh_ms", ms(shift_parts[0])),
                ("context_ms", ms(shift_parts[1])),
                ("rest_ms", ms(shift_parts[2])),
            ],
        );
        let r = stats.shift_rejects;
        browser_tracing::instant(
            "prepaint",
            "PrePaint.ShiftRejects",
            &[
                ("no_basis", r[0] as f64),
                ("own_inputs_changed", r[1] as f64),
                ("own_property_nodes", r[2] as f64),
                ("identity", r[3] as f64),
                ("paint_layer", r[4] as f64),
                ("svg", r[5] as f64),
                ("own_clip_cull_transform", r[6] as f64),
                ("other_context", r[7] as f64),
            ],
        );
        let (k, c) = (stats.build_kind_ns, stats.build_kind_nodes);
        browser_tracing::instant(
            "prepaint",
            "PrePaint.BuildKinds",
            &[
                ("transform_ms", ms(k[0])),
                ("transform_n", c[0] as f64),
                ("clip_ms", ms(k[1])),
                ("clip_n", c[1] as f64),
                ("effect_ms", ms(k[2])),
                ("effect_n", c[2] as f64),
                ("svg_ms", ms(k[3])),
                ("svg_n", c[3] as f64),
            ],
        );
        browser_tracing::instant(
            "prepaint",
            "PrePaint.BuildKinds2",
            &[
                ("layer_ms", ms(k[4])),
                ("layer_n", c[4] as f64),
                ("plain_ms", ms(k[5])),
                ("plain_n", c[5] as f64),
            ],
        );
        tree
    }
}

/// A completed resident PrePaint pass. This token carries the same layout
/// lifetime/identity lease; it contains no copied fragment or Paint tree.
pub struct PreparedPrePaint {
    source: layout::paint::PaintFragmentLease,
}

/// Paint-property values admitted by Blink's deferred transform/opacity
/// update path. The nodes retain their canonical property identity; this is
/// not a copied compositor tree or a replacement PaintArtifact.
#[derive(Clone, Debug, Default)]
pub struct CompositedAnimationPropertyUpdate {
    pub transforms: Vec<Arc<paint_model::paint_property_tree::TransformPaintPropertyNode>>,
    pub effects: Vec<Arc<paint_model::paint_property_tree::EffectPaintPropertyNode>>,
    pub bindings: Vec<prepaint::CompositedAnimationPropertyBinding>,
}

/// The local border box transform and effect an animated object applies
/// itself, read from its FragmentData (fragment_data.h:104-110,137-142):
/// the transform when one of the object's own ObjectPaintProperties
/// transform slots is its LocalBorderBoxProperties transform, and the
/// effect when its own Effect node carries a non-unit opacity (the
/// opacity Effect, object_paint_properties.h:511-523). These are the
/// PaintTreeNode `applies_transform` / `applies_opacity` facts derived from
/// the slots instead of the node copies.
fn OwnAnimatedPropertyStarts(
    data: &prepaint::fragment_data::PrePaintFragmentData,
) -> (
    Option<Arc<paint_model::paint_property_tree::TransformPaintPropertyNode>>,
    Option<Arc<paint_model::paint_property_tree::EffectPaintPropertyNode>>,
) {
    let (Some(properties), Some(state)) =
        (data.PaintProperties(), data.LocalBorderBoxPropertiesRef())
    else {
        return (None, None);
    };
    let transform = properties
        .ForTransformNodes(|node| Arc::ptr_eq(node, &state.transform))
        .then(|| state.transform.clone());
    let effect = properties
        .Effect()
        .is_some_and(|effect| {
            paint_model::property_cell::with_access(|a| effect.state(a).opacity != 1.0)
        })
        .then(|| state.effect.clone());
    (transform, effect)
}

fn CollectCompositedAnimationPropertyUpdate(
    tree: &crate::pre_paint_tree_walk::PaintTreeNode<'_>,
    targeted_nodes: &[u64],
) -> Option<CompositedAnimationPropertyUpdate> {
    use paint_model::paint_property_tree::{
        EffectPaintPropertyNode, PaintPropertyChangeType, TransformPaintPropertyNode,
    };
    use std::collections::HashSet;

    #[derive(Default)]
    struct Collector {
        valid: bool,
        rejection: u8,
        rejected_change: PaintPropertyChangeType,
        matched_targets: HashSet<u64>,
        transforms: Vec<Arc<TransformPaintPropertyNode>>,
        effects: Vec<Arc<EffectPaintPropertyNode>>,
        bindings: Vec<prepaint::CompositedAnimationPropertyBinding>,
        seen_transforms: HashSet<usize>,
        seen_effects: HashSet<usize>,
        seen_transform_bindings: HashSet<u64>,
        seen_effect_bindings: HashSet<u64>,
    }

    impl Collector {
        fn transform(&mut self, node_id: u64, node: &Arc<TransformPaintPropertyNode>) {
            if !self.valid
                || !self
                    .seen_transforms
                    .insert(Arc::as_ptr(&node.lifecycle.identity) as usize)
            {
                return;
            }
            let (active, element_id, has_direct_reasons, parent) =
                paint_model::property_cell::with_access(|a| {
                    let state = node.state(a);
                    (
                        state.HasActiveTransformAnimation(),
                        state.compositor.compositor_element_id,
                        state.HasDirectCompositingReasons(),
                        state.parent.clone(),
                    )
                });
            if active {
                let internal = element_id.GetInternalValue();
                if foundation::CompositorElementId::IsValidInternalValue(internal)
                    && self.seen_transform_bindings.insert(internal)
                {
                    self.bindings
                        .push(prepaint::CompositedAnimationPropertyBinding {
                            node_id,
                            property: prepaint::CompositedAnimationProperty::Transform,
                            element_id,
                        });
                }
                match paint_model::property_cell::with_access(|a| node.lifecycle.node_changed(a)) {
                    PaintPropertyChangeType::Unchanged => {}
                    PaintPropertyChangeType::ChangedOnlyCompositedValues => {
                        self.transforms.push(node.clone());
                    }
                    PaintPropertyChangeType::ChangedOnlySimpleValues if has_direct_reasons => {
                        self.transforms.push(node.clone());
                    }
                    change => {
                        self.valid = false;
                        self.rejection = 1;
                        self.rejected_change = change;
                    }
                }
            }
            if let Some(parent) = &parent {
                self.transform(node_id, parent);
            }
        }

        fn effect(&mut self, node_id: u64, node: &Arc<EffectPaintPropertyNode>) {
            if !self.valid
                || !self
                    .seen_effects
                    .insert(Arc::as_ptr(&node.lifecycle.identity) as usize)
            {
                return;
            }
            let (active, element_id, has_direct_reasons, parent) =
                paint_model::property_cell::with_access(|a| {
                    let state = node.state(a);
                    (
                        state.HasActiveOpacityAnimation(),
                        state.compositor.compositor_element_id,
                        state.HasDirectCompositingReasons(),
                        state.parent.clone(),
                    )
                });
            if active {
                let internal = element_id.GetInternalValue();
                if foundation::CompositorElementId::IsValidInternalValue(internal)
                    && self.seen_effect_bindings.insert(internal)
                {
                    self.bindings
                        .push(prepaint::CompositedAnimationPropertyBinding {
                            node_id,
                            property: prepaint::CompositedAnimationProperty::Opacity,
                            element_id,
                        });
                }
                match paint_model::property_cell::with_access(|a| node.lifecycle.node_changed(a)) {
                    PaintPropertyChangeType::Unchanged
                    | PaintPropertyChangeType::ChangedOnlyCompositedValues => {}
                    PaintPropertyChangeType::ChangedOnlySimpleValues if has_direct_reasons => {
                        self.effects.push(node.clone());
                    }
                    change => {
                        self.valid = false;
                        self.rejection = 2;
                        self.rejected_change = change;
                    }
                }
            }
            if let Some(parent) = &parent {
                self.effect(node_id, parent);
            }
        }

        fn tree(
            &mut self,
            node: &crate::pre_paint_tree_walk::PaintTreeNode<'_>,
            targeted_nodes: &HashSet<u64>,
        ) {
            let node_id = node
                .fragment
                .as_deref()
                .map_or(0, |fragment| fragment.ViewNodeId());
            if targeted_nodes.contains(&node_id) {
                self.matched_targets.insert(node_id);
                // Blink's pending sets contain LayoutObjects, not every
                // property ancestor reachable through their state. Inspect
                // only the local slots this object applies: its
                // FragmentData's ObjectPaintProperties (fragment_data.h:
                // 104-110; object_paint_properties.h:503-523 update the
                // object's own Transform and Effect nodes). Hand-built nodes
                // without FragmentData keep their own copies.
                let (transform, effect) = match node.FragmentData() {
                    Some(data) => OwnAnimatedPropertyStarts(data),
                    None => (
                        node.applies_transform
                            .then(|| node.properties.nodes.transform.clone()),
                        node.applies_opacity
                            .then(|| node.properties.nodes.effect.clone()),
                    ),
                };
                if let Some(transform) = &transform {
                    self.transform(node_id, transform);
                }
                if let Some(effect) = &effect {
                    self.effect(node_id, effect);
                }
            }
            for child in &node.children {
                self.tree(child, targeted_nodes);
            }
        }
    }

    let targeted_nodes = targeted_nodes.iter().copied().collect::<HashSet<_>>();
    let mut collector = Collector {
        valid: true,
        ..Default::default()
    };
    collector.tree(tree, &targeted_nodes);
    if collector.matched_targets.len() != targeted_nodes.len() {
        collector.valid = false;
        collector.rejection = 3;
    }
    let mut trace =
        browser_tracing::span("prepaint", "PrePaint.CompositedAnimationPropertyAdmission");
    trace.set("admitted", collector.valid as u8 as f64);
    trace.set("rejection", collector.rejection as f64);
    trace.set("rejected_change", collector.rejected_change as u8 as f64);
    trace.set("targeted_nodes", targeted_nodes.len() as f64);
    trace.set("matched_targets", collector.matched_targets.len() as f64);
    trace.set("transform_updates", collector.transforms.len() as f64);
    trace.set("effect_updates", collector.effects.len() as f64);
    (collector.valid
        && (!collector.transforms.is_empty()
            || !collector.effects.is_empty()
            || !collector.bindings.is_empty()))
    .then_some(CompositedAnimationPropertyUpdate {
        transforms: collector.transforms,
        effects: collector.effects,
        bindings: collector.bindings,
    })
}

fn RecordPaint(
    pre_paint: &mut PrePaintTreeWalk,
    fragments: PaintFragmentSource<'_>,
    source_lease: layout::paint::PaintFragmentLease,
    editing: EditingPaintState,
    previous: Option<Arc<PaintArtifact>>,
    previous_subsequences: paint_record::SubsequencesData,
    foreign_layers: &foundation::TrustedIdentityHashMap<u64, Arc<dyn ForeignLayer>>,
    lifecycle: Option<&dyn PaintLifecycleObserver>,
    prepared: Option<PreparedPrePaint>,
    reuse_admissible: bool,
) -> (
    Option<PaintRecording>,
    paint_record::SubsequencesData,
    crate::performance::PaintPerformanceSnapshot,
) {
    let mut trace = browser_tracing::span("paint", "Paint.RecordPaint");
    ValidateReplayResources(fragments.Cursor().DocumentResources().map(|resources| &**resources));
    let paint_tree = if let Some(prepared) = prepared {
        assert!(
            prepared.source.PtrEq(&source_lease),
            "Paint uses the exact prepared Layout result"
        );
        pre_paint
            .retained_tree
            .take()
            .expect("completed PrePaint retains its native tree")
    } else {
        if let Some(lifecycle) = lifecycle {
            lifecycle.AdvanceTo(PaintLifecyclePhase::InPrePaint);
        }
        let tree = WalkPrePaint(pre_paint, fragments, source_lease);
        if let Some(lifecycle) = lifecycle {
            lifecycle.SetIntersectionObservationState(pre_paint.IntersectionObservationState());
            lifecycle.AdvanceTo(PaintLifecyclePhase::PrePaintClean);
        }
        tree
    };
    if let Some(lifecycle) = lifecycle {
        lifecycle.AdvanceTo(PaintLifecyclePhase::InPaint);
    }
    let mut output = PaintRecording::default();

    output.compositor_viewport = pre_paint.ExportCompositorViewport(&fragments);
    output.resources = fragments.Cursor().DocumentResources().cloned();
    // Same-Page LocalFrames record into this controller. Their immutable
    // resource identities therefore enter its single artifact catalog too;
    // encoded pixels, decoded frames and font bytes remain shared owners.
    for view in pre_paint.LocalFrameViews() {
        let root = view.Root();
        let Some(resources) = root.DocumentResources() else {
            continue;
        };
        ValidateReplayResources(Some(resources));
        if let Some(catalog) = output.resources.as_mut() {
            if Arc::ptr_eq(catalog, resources) {
                continue;
            }
            let catalog = Arc::make_mut(catalog);
            if !catalog.fonts.ptr_eq(&resources.fonts) {
                for face in &resources.fonts {
                    if !catalog.fonts.contains(face) {
                        catalog.fonts.push(face.clone());
                    }
                }
            }
            // The catalogs are shared immutable slices; append the missing
            // entries in one rebuild instead of growing them in place.
            let mut added_images: Vec<image_resource::PaintImage> = Vec::new();
            for image in resources.images.iter() {
                if let Some(existing) = catalog
                    .images
                    .iter()
                    .chain(added_images.iter())
                    .find(|existing| existing.id == image.id)
                {
                    assert_eq!(
                        existing.revision, image.revision,
                        "shared PaintImage identity must have one revision"
                    );
                } else {
                    added_images.push(image.clone());
                }
            }
            if !added_images.is_empty() {
                catalog.images = catalog.images.iter().cloned().chain(added_images).collect();
            }
            let mut added_canvases: Vec<canvas::CanvasPaintResource> = Vec::new();
            for canvas in resources.canvases.iter() {
                if !catalog
                    .canvases
                    .iter()
                    .chain(added_canvases.iter())
                    .any(|existing| existing.resource_id == canvas.resource_id)
                {
                    added_canvases.push(canvas.clone());
                }
            }
            if !added_canvases.is_empty() {
                catalog.canvases = catalog
                    .canvases
                    .iter()
                    .cloned()
                    .chain(added_canvases)
                    .collect();
            }
        } else {
            output.resources = Some(resources.clone());
        }
    }
    // cpp: third_party/blink/renderer/core/frame/local_frame_view.cc:3183-3226
    //      LocalFrameView::PaintTree creates a PaintController and walks the
    //      paint tree only when the LayoutView layer
    //      SelfOrDescendantNeedsRepaint(); otherwise the persistent data keeps
    //      the previous PaintArtifact. Live property nodes already carry this
    //      PrePaint's values, so a property-only frame records nothing.
    if previous.as_ref().is_some_and(|previous| {
        reuse_admissible
            && !pre_paint.RootNeedsRepaint(&paint_tree)
            && previous.compositor_viewport == output.compositor_viewport
            && match (&previous.resources, &output.resources) {
                (None, None) => true,
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                _ => false,
            }
    }) {
        trace.set("reused_artifact", 1.0);
        {
            let _trace = browser_tracing::span("prepaint", "PrePaint.RetainTree");
            pre_paint.RetainTree(paint_tree);
        }
        // No PaintController ran, so nothing is committed: the persistent
        // data keeps the very same artifact object (`None` here), which is
        // what RasterInvalidator's identity check relies on
        // (raster_invalidator.cc:293-301) and what keeps the shared artifact
        // free of any per-frame write.
        return (
            None,
            previous_subsequences,
            crate::performance::PaintPerformanceSnapshot {
                prepaint: pre_paint.stats,
                ..Default::default()
            },
        );
    }
    let caret_geometry = editing.caret.and_then(|state| {
        state
            .native
            .map(|native| (native.local_rect(), native.shape))
    });
    let drag_caret_geometry = editing
        .drag_caret
        .and_then(|state| state.native.map(|native| native.local_rect()));
    let mut paint_context = PaintContext::WithEditingAndPersistentCache(
        &mut output,
        editing,
        previous,
        previous_subsequences,
    );
    paint_context.BindForeignLayers(foreign_layers);
    paint_context.drag_caret_local_rect = drag_caret_geometry;
    // RecordPaint's root cursor is the documentElement Layout result. Preserve
    // its exact computed opacity as the document-global Paint Timing fact used
    // by every nested PaintLayerPainter scope.
    paint_context.SetDocumentElementIsInvisible(fragments.paint.style.opacity() == 0.0);
    // RECHROM_PREPAINT_FRAGMENT_DATA_CHECK: the resident PrePaint nodes are
    // lent to the paint walk as its oracle (the SVG resource directory
    // included); production painting reads only the fragment cursor.
    let traversal_check = crate::paint_layer_painter::TraversalCheckEnabled();
    if traversal_check {
        paint_context
            .BindSvgResourceTree(paint_record::paint_object::PaintObject::new(&paint_tree));
    }
    paint_context.BindPaintTimingRoot(paint_record::paint_object::PaintObject::FromCursor(
        fragments.Cursor(),
        paint_record::paint_object::PaintWalkContext::default(),
    ));
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
        {
            // LocalFrameView::PaintTree walks the live tree under the source
            // lease; painters read each fragment's LayoutObject directly.
            let _resident_sources = layout::paint::ResidentPaintSourceScope::new();
            // FramePainter::Paint of the root LocalFrameView: its LayoutView
            // layer, the embedded same-Page frames through PrePaint's
            // LocalFrameViews.
            let frames: &dyn crate::paint_layer_painter::LocalFrameRoots = &*pre_paint;
            PaintLayerPainter::new(fragments.Cursor(), &context, Some(frames))
                .WithOracle(traversal_check.then_some(&*paint_tree))
                .Paint();
        }
        let walk = context.borrow().walk_metrics;
        trace.set("layers", walk.layer_visits as f64);
        trace.set("phase_visits", walk.phase_visits as f64);
        trace.set("skipped_hidden", walk.skipped_hidden as f64);
        trace.set("skipped_phase", walk.skipped_phase as f64);
        trace.set("subsequence_layers", walk.subsequence_layers as f64);
        trace.set("subsequence_needs_repaint", walk.subsequence_needs_repaint as f64);
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
    trace.set("recorded_drawings", reuse.recorded_drawings as f64);
    trace.set("recorded_scrollbars", reuse.recorded_scrollbars as f64);
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
    browser_tracing::instant(
        "paint_reuse",
        "PaintReuse.Subsequences",
        &[
            ("attempts", reuse.subsequence_attempts as f64),
            ("hits", reuse.subsequence_hits as f64),
            ("rejected_client", reuse.subsequence_rejected_client as f64),
            ("missing_index", reuse.subsequence_rejected_missing_index as f64),
            ("tombstone", reuse.subsequence_rejected_tombstone as f64),
            ("invalid", reuse.subsequence_rejected_invalid as f64),
        ],
    );
    drop(context);
    // Source: PaintController::CommitNewDisplayItems and
    // PaintController::AppendSubsequenceByMoving. Cached chunks and display
    // items keep references to the live ObjectPaintProperties nodes, which
    // PrePaint updated in place, so they observe current values and parents
    // without any rebinding.
    let (record, chunk) = {
        let access = artifact_access();
        (
            paint_record::PaintRecordMetrics {
                paint_ops: output.items.len(),
                display_items: output.display_items.get(&access).len(),
            },
            paint_chunk::MeasureChunks(
                output
                    .chunks
                    .get(&access)
                    .iter()
                    .map(|chunk| (chunk.begin_index, chunk.end_index)),
            ),
        )
    };
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
    trace.set("display_items", record.display_items as f64);
    trace.set("chunks", chunk.chunks as f64);
    if browser_tracing::profile_input() {
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
        Some(output),
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
    foreign_layers: foundation::TrustedIdentityHashMap<u64, Arc<dyn ForeignLayer>>,
    foreign_layers_changed: bool,
    recorded_since_commit: bool,
    /// A PaintController cycle committed a new artifact since the last
    /// CommitPaintResult. Chromium validates clients and clears
    /// `client_is_just_created` only at the end of such a cycle
    /// (PaintController::~PaintController, paint_controller.cc:127-143);
    /// a frame that reused the artifact writes nothing into it.
    repainted_since_commit: bool,
    /// `property_cell::PropertyChangeEpoch` when the current artifact object
    /// was published. A kept artifact is republished as a new header (every
    /// recording Arc shared) only when live property values changed since,
    /// so the compositor push still runs for them; otherwise it keeps its
    /// identity.
    published_property_epoch: u64,
    /// The editing state the current artifact was recorded with.
    last_editing: Option<EditingPaintState>,
}
impl PaintEngine {
    pub fn new() -> Self {
        Self::default()
    }
    fn PaintInternal(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        editing: EditingPaintState,
        lifecycle: Option<&dyn PaintLifecycleObserver>,
        document: Option<canvas::CanvasDocumentIdentity>,
        prepared: Option<PreparedPrePaint>,
    ) {
        let _trace = browser_tracing::span("paint", "PaintEngine.Paint");
        let layout_result = PaintFragmentSource::FromCursor(layout_output.PaintCursor());
        let source_lease = layout_output.PaintLease();
        let previous = self.persistent_data.GetPaintArtifact().cloned();
        let previous_subsequences = self.persistent_data.TakeSubsequences();
        // Inputs that reach the recording without a PaintLayer repaint bit:
        // the editing state, foreign layer bindings and embedded frames.
        // A second recording before CommitPaintResult (document-image
        // container re-recording) always records.
        let reuse_admissible = !self.foreign_layers_changed
            && !self.recorded_since_commit
            && !self.pre_paint.HasLocalFrameViews()
            && self.last_editing.as_ref() == Some(&editing);
        self.last_editing = Some(editing.clone());
        self.foreign_layers_changed = false;
        self.recorded_since_commit = true;
        let (result, subsequences, performance) = RecordPaint(
            &mut self.pre_paint,
            layout_result,
            source_lease,
            editing,
            previous,
            previous_subsequences,
            &self.foreign_layers,
            lifecycle,
            prepared,
            reuse_admissible,
        );
        // This recording consumed every PrePaint walk so far; the retained
        // scroll path may again trust the live dirty bits against it.
        self.pre_paint.MarkPrePaintRecorded();
        if self.pre_paint.stats.post_walk || self.retained_scroll_state.is_none() {
            self.retained_scroll_state = None;
            self.retained_scroll_state_initialized = false;
        }
        self.performance = performance;
        let property_epoch = paint_model::property_cell::PropertyChangeEpoch();
        let property_values_changed =
            std::mem::replace(&mut self.published_property_epoch, property_epoch) != property_epoch;
        let artifact = match result {
            // PaintController::CommitNewDisplayItems: a new artifact object
            // replaces the current one (paint_controller.cc:84-91,823-848);
            // its clients are validated by the next CommitPaintResult.
            Some(mut recording) => {
                recording.canvas_document = document;
                self.repainted_since_commit = true;
                paint_commit::CommitArtifact(recording)
            }
            // LocalFrameView::PaintTree without a repaint keeps the current
            // artifact object (local_frame_view.cc:3185-3227), and nothing
            // is written into it. Chromium's PrePaint pushes changed live
            // property values to PAC directly; here they reach Layerize with
            // the next artifact push, which Page skips for an artifact object
            // it already pushed. So when live property values changed since
            // this object was published, republish the same recording under
            // a new header: a shallow clone sharing every recording Arc
            // (items, display items, chunks, resources), never a chunk copy.
            None => {
                let current = self
                    .persistent_data
                    .GetPaintArtifactMut()
                    .expect("artifact reuse requires a current artifact");
                if property_values_changed || current.canvas_document != document {
                    let mut header = PaintRecording::clone(&**current);
                    header.canvas_document = document;
                    *current = Arc::new(header);
                }
                current.clone()
            }
        };
        self.persistent_data
            .CommitNewDisplayItemsWithSubsequences(artifact, subsequences);
        if let Some(lifecycle) = lifecycle {
            lifecycle.AdvanceTo(PaintLifecyclePhase::PaintClean);
        }
    }
}

/// Painting: PrePaint, Paint and the commit of the `PaintArtifact`
/// (Blink LocalFrameView::RunPrePaintLifecyclePhase and
/// RunPaintLifecyclePhase, core/frame/local_frame_view.cc).
pub trait PaintLifecycle {
    /// Update resident properties and record a new artifact. Publish only once
    /// this pass completes; a failed paint cannot expose the previous result
    /// as if it represented the new inputs.
    fn Paint(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        editing: EditingPaintState,
    );
    /// Publish actual EmbeddedContentView owners for the next cross-document
    /// walk. This never records a child artifact or changes child lifecycle.
    fn SetLocalFrameViews(&mut self, views: &[LocalFrameViewPaintInput<'_>]);
    fn SetBlockingWheelEventHandlerTargets(&mut self, targets: Arc<[u64]>);
    fn PrePaintWithLocalFrameViews(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        views: &[LocalFrameViewPaintInput<'_>],
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> PreparedPrePaint;
    fn PaintAfterPrePaintWithLocalFrameViews(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        views: &[LocalFrameViewPaintInput<'_>],
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
        document: canvas::CanvasDocumentIdentity,
        prepared: PreparedPrePaint,
    );
    fn PaintWithLocalFrameViews(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        views: &[LocalFrameViewPaintInput<'_>],
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
        document: canvas::CanvasDocumentIdentity,
    );
    /// LocalFrameView::UpdateAllLifecyclePhasesExceptPaint: execute the same
    /// resident PrePaint walk used by Paint, leaving display-item recording
    /// and artifact publication to the scheduled rendering lifecycle.
    fn PrePaintWithLifecycle(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> PreparedPrePaint;
    fn PaintAfterPrePaint(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
        document: canvas::CanvasDocumentIdentity,
        prepared: PreparedPrePaint,
    );
    fn PaintWithLifecycle(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
        document: canvas::CanvasDocumentIdentity,
    );
    /// Publish the resident compositor layer used by VideoPainter's normal
    /// foreground path. Passing `None` detaches the node without affecting
    /// unrelated paint resources or the package graph.
    fn SetForeignLayer(&mut self, node_id: u64, layer: Option<Arc<dyn ForeignLayer>>);
    fn GetPrePaintUpdateStats(&self) -> crate::pre_paint_tree_walk::PrePaintUpdateStats;
    fn GetPaintResult(&self) -> Option<&Arc<PaintArtifact>>;
    fn GetPerformanceSnapshot(&self) -> crate::performance::PaintPerformanceSnapshot;
    /// Complete Paint-owned client validation and publish an immutable output.
    /// Layout consumes the opaque client receipt at its own boundary; Page
    /// never receives mutable access to the artifact or the client set.
    fn CommitPaintResult(&mut self) -> PaintOutput;
}

impl PaintLifecycle for PaintEngine {
    fn Paint(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        editing: EditingPaintState,
    ) {
        self.PaintInternal(layout_output, editing, None, None, None);
    }
    fn SetLocalFrameViews(&mut self, views: &[LocalFrameViewPaintInput<'_>]) {
        self.pre_paint.SetLocalFrameViews(views.iter().map(|view| {
            prepaint::pre_paint_tree_walk::EmbeddedLocalFrameView::new(
                view.owner,
                view.layout,
                view.is_cross_origin_to_parent,
            )
            .WithBlockingWheelEventHandlerTargets(view.blocking_wheel_targets.clone())
        }));
    }
    fn SetBlockingWheelEventHandlerTargets(&mut self, targets: Arc<[u64]>) {
        self.pre_paint.SetBlockingWheelEventHandlerTargets(targets);
    }

    fn PrePaintWithLocalFrameViews(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        views: &[LocalFrameViewPaintInput<'_>],
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> PreparedPrePaint {
        self.SetLocalFrameViews(views);
        let prepared = self.PrePaintWithLifecycle(layout_output, lifecycle);
        // Only children actually reached by the resident PrePaint walk become
        // clean. A display-locked frame owner must retain its dirty lifecycle.
        for view in views {
            if self
                .pre_paint
                .LocalFramePrePaintCompleted(view.owner.paint.DisplayItemClientId())
            {
                view.lifecycle.AdvanceTo(PaintLifecyclePhase::InPrePaint);
                view.lifecycle
                    .SetIntersectionObservationState(self.pre_paint.IntersectionObservationState());
                view.lifecycle.AdvanceTo(PaintLifecyclePhase::PrePaintClean);
            }
        }
        prepared
    }

    fn PaintAfterPrePaintWithLocalFrameViews(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        views: &[LocalFrameViewPaintInput<'_>],
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
        document: canvas::CanvasDocumentIdentity,
        prepared: PreparedPrePaint,
    ) {
        for view in views {
            if self
                .pre_paint
                .LocalFramePrePaintCompleted(view.owner.paint.DisplayItemClientId())
            {
                view.lifecycle.AdvanceTo(PaintLifecyclePhase::InPaint);
            }
        }
        self.PaintAfterPrePaint(layout_output, editing, lifecycle, document, prepared);
        for view in views {
            if self
                .pre_paint
                .LocalFramePrePaintCompleted(view.owner.paint.DisplayItemClientId())
            {
                view.lifecycle.AdvanceTo(PaintLifecyclePhase::PaintClean);
            }
        }
    }

    fn PaintWithLocalFrameViews(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        views: &[LocalFrameViewPaintInput<'_>],
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
        document: canvas::CanvasDocumentIdentity,
    ) {
        let prepared = self.PrePaintWithLocalFrameViews(layout_output, views, lifecycle);
        self.PaintAfterPrePaintWithLocalFrameViews(
            layout_output,
            views,
            editing,
            lifecycle,
            document,
            prepared,
        );
    }

    fn PrePaintWithLifecycle(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> PreparedPrePaint {
        lifecycle.AdvanceTo(PaintLifecyclePhase::InPrePaint);
        let tree = WalkPrePaint(
            &mut self.pre_paint,
            PaintFragmentSource::FromCursor(layout_output.PaintCursor()),
            layout_output.PaintLease(),
        );
        self.pre_paint.RetainTree(tree);
        lifecycle.SetIntersectionObservationState(self.pre_paint.IntersectionObservationState());
        lifecycle.AdvanceTo(PaintLifecyclePhase::PrePaintClean);
        PreparedPrePaint {
            source: layout_output.PaintLease(),
        }
    }

    fn PaintAfterPrePaint(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
        document: canvas::CanvasDocumentIdentity,
        prepared: PreparedPrePaint,
    ) {
        self.PaintInternal(
            layout_output,
            editing,
            Some(lifecycle),
            Some(document),
            Some(prepared),
        );
    }
    fn PaintWithLifecycle(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        editing: EditingPaintState,
        lifecycle: &dyn PaintLifecycleObserver,
        document: canvas::CanvasDocumentIdentity,
    ) {
        self.PaintInternal(
            layout_output,
            editing,
            Some(lifecycle),
            Some(document),
            None,
        );
    }

    fn SetForeignLayer(&mut self, node_id: u64, layer: Option<Arc<dyn ForeignLayer>>) {
        self.foreign_layers_changed = true;
        if let Some(layer) = layer {
            self.foreign_layers.insert(node_id, layer);
        } else {
            self.foreign_layers.remove(&node_id);
        }
    }

    fn GetPrePaintUpdateStats(&self) -> crate::pre_paint_tree_walk::PrePaintUpdateStats {
        self.pre_paint.stats
    }

    fn GetPaintResult(&self) -> Option<&Arc<PaintArtifact>> {
        self.persistent_data.GetPaintArtifact()
    }

    fn GetPerformanceSnapshot(&self) -> crate::performance::PaintPerformanceSnapshot {
        self.performance
    }

    fn CommitPaintResult(&mut self) -> PaintOutput {
        let mut trace = browser_tracing::span("paint_commit", "PaintCommit.Publish");
        self.recorded_since_commit = false;
        let repainted = std::mem::take(&mut self.repainted_since_commit);
        let generation = self.pre_paint.property_node_store.CompositorUpdateGeneration();
        let current = self
            .persistent_data
            .GetPaintArtifactMut()
            .expect("Paint must complete before commit");
        // Source: paint_controller.cc:127-143. Only the PaintController cycle
        // that produced a new artifact validates its clients and clears its
        // chunks' client_is_just_created; that artifact has not been handed
        // to Page or Layerize yet, so the write is in place. A frame that
        // kept the current artifact (LocalFrameView::PaintTree without
        // repaint, local_frame_view.cc:3185-3227) has no clients to validate
        // and leaves the shared artifact untouched.
        let clients = if repainted {
            let artifact = Arc::make_mut(current);
            let clients = crate::PaintController::ValidateForCommittedPaint(artifact);
            artifact.property_tree_update_generation = generation;
            clients
        } else {
            // LocalFrameView::SetPaintArtifactCompositorNeedsUpdate's typed
            // value (local_frame_view.cc:3545-3563). It versions the shared
            // artifact only when PrePaint actually changed property nodes
            // beyond composited values; the copy is the shallow artifact
            // header, every recording Arc stays shared.
            if current.property_tree_update_generation != generation {
                Arc::make_mut(current).property_tree_update_generation = generation;
            }
            std::collections::HashSet::new()
        };
        let recording_revision = current.recording_revision;
        self.performance.commit = paint_commit::PaintCommitMetrics {
            clients: clients.len(),
            recording_revision,
        };
        trace.set("clients", clients.len() as f64);
        trace.set("repainted", repainted as u8 as f64);
        trace.set("recording_revision", recording_revision as f64);
        // ~PaintController validates the used clients through their own
        // pointers (paint_controller.cc:127-143), never by walking the
        // LayoutObject tree. The resident client directory resolves each id
        // to its live client; only the ids it cannot resolve (the caret
        // clients, or a client created after the directory's last rebuild)
        // go to the Layout commit, which validates them on its source
        // owners.
        let unresolved = self.pre_paint.ValidateLiveClientsForCommittedPaint(&clients);
        trace.set("unresolved_clients", unresolved.len() as f64);
        self.pre_paint.CommitClients(clients.iter().copied());
        PaintOutput::new(
            self.persistent_data
                .GetPaintArtifact()
                .expect("committed Paint result")
                .clone(),
            layout::paint::PaintClientCommit::FromClientSet(unresolved),
        )
    }
}

/// Resource-only updates of the committed artifact: new resource catalogs
/// and resource mutations without a new paint (Blink ImageResourceContent
/// observers invalidate only the records that sample the resource).
pub trait PaintResourceUpdates {
    /// Publish a new immutable resource catalog while retaining the recorded
    /// display items and paint properties. This is the document-image analogue
    /// of ImageResourceContent invalidation: stable resource ids let the
    /// compositor damage only records that sample changed pixels.
    fn UpdatePaintResources(&mut self, resources: Arc<PaintResources>) -> bool;
    fn ApplyMutation(&mut self, mutation: PaintMutation) -> Option<PaintMutationResult>;
    /// Apply resource-only changes to the artifact this engine's persistent
    /// PaintController data owns. Chromium's PaintControllerPersistentData
    /// is the only document-side owner of the current PaintArtifact
    /// (paint_controller.h:85-99); LocalFrameView reads it there
    /// (local_frame_view.cc:4399-4406) and the embedder never holds its own
    /// reference. The update is in place; Arc copy-on-write happens only
    /// while Layerize still retains the previous artifact (the
    /// PaintArtifactCompositor's retained reference). Returns false when no
    /// artifact exists or a resource mutation is rejected.
    fn PublishResourceUpdate(&mut self, update: PaintResourceUpdate) -> bool;
}

impl PaintResourceUpdates for PaintEngine {
    fn UpdatePaintResources(&mut self, resources: Arc<PaintResources>) -> bool {
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
    fn ApplyMutation(&mut self, mutation: PaintMutation) -> Option<PaintMutationResult> {
        let artifact = self.persistent_data.GetPaintArtifactMut()?;
        let artifact = Arc::make_mut(artifact);
        artifact.ApplyResourceMutation(mutation)
    }

    fn PublishResourceUpdate(&mut self, update: PaintResourceUpdate) -> bool {
        match update {
            PaintResourceUpdate::Snapshot(resources) => self.UpdatePaintResources(resources),
            PaintResourceUpdate::Mutations(mutations) => mutations
                .into_iter()
                .all(|mutation| self.ApplyMutation(mutation).is_some()),
        }
    }
}

/// Direct property updates that skip a full paint: scroll offsets, the
/// visual viewport and composited animations (Blink
/// PaintPropertyTreeBuilder::DirectlyUpdateTransformAndOpacity and
/// PaintLayerScrollableArea::UpdateScrollOffset's direct scroll-translation
/// update, core/paint/paint_property_tree_builder.cc).
pub trait PaintPropertyUpdates {
    fn ApplyAnimationMutation(&mut self, mutation: animation::PaintMutation);
    fn AnimationTargetedNodes(&self) -> &[u64];
    /// Source: LocalFrameView::ExecuteAllPendingUpdates and
    /// PaintPropertyTreeBuilder::DirectlyUpdate{TransformMatrix,OpacityValue}.
    /// First execute Chromium's targeted pending transform/opacity update on
    /// the canonical retained ObjectPaintProperties slots. Any structural,
    /// owner, or compositing-reason change rejects that path and runs the
    /// ordinary complete PrePaint walk, leaving it prepared for Paint.
    fn PrePaintCompositedAnimationUpdate(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        targeted_nodes: &[u64],
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> (
        Option<PreparedPrePaint>,
        Option<CompositedAnimationPropertyUpdate>,
    );
    /// Typed handoff for KeyframeModels created by the same complete
    /// PrePaint/Paint lifecycle as their compositor property nodes.
    fn CompositedAnimationPropertyBindings(
        &self,
        targeted_nodes: &[u64],
    ) -> Vec<prepaint::CompositedAnimationPropertyBinding>;
    /// Apply a content-only image mutation to the committed artifact. Stable
    /// display items remain shared; the retained dependency index identifies
    /// exactly which paint clients LayerTile must invalidate.
    /// VisualViewport native scalar input. PrePaint updates its existing
    /// property slot during the ordinary walk; it never allocates a second id.
    fn SetVisualViewportState(&mut self, state: paint_model::PaintVisualViewportState);
    fn SetVisualViewportScrollOffset(
        &mut self,
        element: foundation::CompositorElementId,
        offset: foundation::gfx::Vector2dF,
    );
    /// Requires the source retained-geometry proof from Page, just as the
    /// existing scroll update path. A rejection leaves the artifact unchanged.
    #[doc(hidden)]
    fn TryUpdateScrollProperties(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
    ) -> Result<
        crate::scroll_paint_update::ScrollPropertyUpdate,
        crate::scroll_paint_update::ScrollPaintInvalidation,
    >;
}

impl PaintPropertyUpdates for PaintEngine {
    fn ApplyAnimationMutation(&mut self, mutation: animation::PaintMutation) {
        match mutation {
            animation::PaintMutation::SetTargetedProperties { node_ids } => {
                self.animation_targeted_nodes = node_ids;
            }
            animation::PaintMutation::Commit => self.animation_targeted_nodes.clear(),
        }
    }

    fn AnimationTargetedNodes(&self) -> &[u64] {
        &self.animation_targeted_nodes
    }

    fn PrePaintCompositedAnimationUpdate(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
        targeted_nodes: &[u64],
        lifecycle: &dyn PaintLifecycleObserver,
    ) -> (
        Option<PreparedPrePaint>,
        Option<CompositedAnimationPropertyUpdate>,
    ) {
        lifecycle.AdvanceTo(PaintLifecyclePhase::InPrePaint);
        if let Some(update) = self
            .pre_paint
            .DirectlyUpdateCompositedAnimationProperties(targeted_nodes)
        {
            lifecycle.AdvanceTo(PaintLifecyclePhase::PrePaintClean);
            return (
                None,
                Some(CompositedAnimationPropertyUpdate {
                    transforms: update.transforms,
                    effects: update.effects,
                    bindings: update.bindings,
                }),
            );
        }
        let tree = WalkPrePaint(
            &mut self.pre_paint,
            PaintFragmentSource::FromCursor(layout_output.PaintCursor()),
            layout_output.PaintLease(),
        );
        let update = CollectCompositedAnimationPropertyUpdate(&tree, targeted_nodes);
        self.pre_paint.RetainTree(tree);
        lifecycle.SetIntersectionObservationState(self.pre_paint.IntersectionObservationState());
        lifecycle.AdvanceTo(PaintLifecyclePhase::PrePaintClean);
        (
            Some(PreparedPrePaint {
                source: layout_output.PaintLease(),
            }),
            update,
        )
    }

    fn CompositedAnimationPropertyBindings(
        &self,
        targeted_nodes: &[u64],
    ) -> Vec<prepaint::CompositedAnimationPropertyBinding> {
        self.pre_paint
            .CompositedAnimationPropertyBindings(targeted_nodes)
    }

    fn SetVisualViewportState(&mut self, state: paint_model::PaintVisualViewportState) {
        self.pre_paint.SetVisualViewportState(state);
    }
    fn SetVisualViewportScrollOffset(
        &mut self,
        element: foundation::CompositorElementId,
        offset: foundation::gfx::Vector2dF,
    ) {
        self.pre_paint
            .SetVisualViewportScrollOffset(element, offset);
    }

    #[doc(hidden)]
    fn TryUpdateScrollProperties(
        &mut self,
        layout_output: &layout::paint::LayoutPaintOutput,
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
        if result.is_ok() {
            let generation = self.pre_paint.property_node_store.CompositorUpdateGeneration();
            if let Some(artifact) = self
                .persistent_data
                .GetPaintArtifactMut()
                .filter(|artifact| artifact.property_tree_update_generation != generation)
            {
                Arc::make_mut(artifact).property_tree_update_generation = generation;
            }
        }
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

/// Read-only PaintLayer queries for hit testing and cursor selection (Blink
/// PaintLayer::HitTestLayer and PaintLayerClipper, core/paint/paint_layer.cc).
pub trait PaintLayerQueries {
    /// The PaintLayer `source` names, iterated over Layout's own PaintLayer
    /// children and PaintLayerStackingNode lists
    /// (paint_layer_paint_order_iterator.cc:39-79), which Paint shares.
    fn ReverseResidentLayerOrder<'a>(
        &'a self,
        source: layout::paint::PaintFragmentCursor<'a>,
        selection: crate::resident_hit_test::ResidentLayerSelection,
    ) -> Option<crate::resident_hit_test::ResidentPaintLayerOrderCursor<'a>>;
    fn ResidentLayerOrder<'a>(
        &'a self,
        source: layout::paint::PaintFragmentCursor<'a>,
        selection: crate::resident_hit_test::ResidentLayerSelection,
        reverse: bool,
    ) -> Option<crate::resident_hit_test::ResidentPaintLayerOrderCursor<'a>>;
    fn ResidentOverlayControlsAfter<'a>(
        &'a self,
        source: layout::paint::PaintFragmentCursor<'a>,
        child: layout::paint::PaintFragmentCursor<'_>,
    ) -> Option<crate::resident_hit_test::ResidentPaintLayerOrderCursor<'a>>;
    /// Borrow committed PrePaint data for this exact resident Layout fragment.
    /// The lease prevents a paint lifecycle mutation while the query runs.
    fn ResidentHitTest(
        &self,
        source: layout::paint::PaintFragmentCursor<'_>,
    ) -> Option<crate::resident_hit_test::ResidentPaintHitTest<'_>>;
    fn ResidentLayerHitTest(
        &self,
        source: layout::paint::PaintFragmentCursor<'_>,
    ) -> Option<crate::resident_hit_test::ResidentPaintHitTest<'_>>;
    fn CalculateResidentLayerClipRects(
        &self,
        source: layout::paint::PaintFragmentCursor<'_>,
        layer: crate::resident_hit_test::ResidentPaintHitTest<'_>,
        root: crate::resident_hit_test::ResidentPaintHitTest<'_>,
        fragment: crate::resident_hit_test::ResidentPaintHitTest<'_>,
        options: crate::resident_layer_clipper::ResidentLayerClipOptions,
    ) -> crate::resident_layer_clipper::ResidentLayerClipRects;
}

impl PaintLayerQueries for PaintEngine {
    fn ReverseResidentLayerOrder<'a>(
        &'a self,
        source: layout::paint::PaintFragmentCursor<'a>,
        selection: crate::resident_hit_test::ResidentLayerSelection,
    ) -> Option<crate::resident_hit_test::ResidentPaintLayerOrderCursor<'a>> {
        self.ResidentLayerOrder(source, selection, true)
    }
    fn ResidentLayerOrder<'a>(
        &'a self,
        source: layout::paint::PaintFragmentCursor<'a>,
        selection: crate::resident_hit_test::ResidentLayerSelection,
        reverse: bool,
    ) -> Option<crate::resident_hit_test::ResidentPaintLayerOrderCursor<'a>> {
        // The query names a PaintLayer, which belongs to its LayoutObject:
        // it is resolved by object identity, so an inline-level layer
        // reached through its FragmentItem cursor names its own PaintLayer.
        // Layout keeps the stacking node's lists current before PrePaint
        // (PaintLayer::UpdateDescendantDependentFlags), so no Paint-side
        // list preparation precedes the query.
        self.pre_paint
            .ResidentLayerOrder(source, selection, None, reverse)
    }

    fn ResidentOverlayControlsAfter<'a>(
        &'a self,
        source: layout::paint::PaintFragmentCursor<'a>,
        child: layout::paint::PaintFragmentCursor<'_>,
    ) -> Option<crate::resident_hit_test::ResidentPaintLayerOrderCursor<'a>> {
        // Both cursors name PaintLayers; resolve them by object identity
        // like ResidentLayerOrder.
        let all = crate::resident_hit_test::ResidentLayerSelection {
            negative_z: true,
            normal_flow: true,
            positive_z: true,
        };
        self.pre_paint
            .ResidentLayerOrder(source, all, Some(child), false)
    }

    fn ResidentHitTest(
        &self,
        source: layout::paint::PaintFragmentCursor<'_>,
    ) -> Option<crate::resident_hit_test::ResidentPaintHitTest<'_>> {
        self.pre_paint.ResidentHitTest(source)
    }

    fn ResidentLayerHitTest(
        &self,
        source: layout::paint::PaintFragmentCursor<'_>,
    ) -> Option<crate::resident_hit_test::ResidentPaintHitTest<'_>> {
        self.pre_paint.ResidentLayerHitTest(source)
    }

    fn CalculateResidentLayerClipRects(
        &self,
        source: layout::paint::PaintFragmentCursor<'_>,
        layer: crate::resident_hit_test::ResidentPaintHitTest<'_>,
        root: crate::resident_hit_test::ResidentPaintHitTest<'_>,
        fragment: crate::resident_hit_test::ResidentPaintHitTest<'_>,
        options: crate::resident_layer_clipper::ResidentLayerClipOptions,
    ) -> crate::resident_layer_clipper::ResidentLayerClipRects {
        self.pre_paint
            .CalculateResidentLayerClipRects(source, layer, root, fragment, options)
    }
}
