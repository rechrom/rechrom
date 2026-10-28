use std::{collections::HashMap, sync::Arc};

use crate::{CaretGeometry, DisplayItem, DisplayItemList, RecordedDisplayItem};
use image_resource::downcast_document_paint_record;
use layoutng_assembly::fragment_tree::PaintResources;
use layoutng_assembly::internal::paint_input::PaintShader;
use paint_chunk::{PaintChunk, PaintChunks};
use paint_model::PaintCompositorViewport;
use paint_model::PaintRect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImagePaintDependency {
    pub record_index: usize,
    pub visual_rect: PaintRect,
}

/// Built while recording paint. Resource-only updates use this retained index
/// instead of walking the outer Fragment tree to rediscover image clients.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageDependencyIndex {
    by_image: HashMap<image_resource::ImageId, Vec<ImagePaintDependency>>,
}

impl ImageDependencyIndex {
    /// Borrows the existing producer dependency identities. No image catalog
    /// or command list is copied for compositor input validation.
    pub fn image_ids(&self) -> impl Iterator<Item = image_resource::ImageId> + '_ {
        self.by_image.keys().copied()
    }
    pub fn dependencies(&self, image_id: image_resource::ImageId) -> &[ImagePaintDependency] {
        self.by_image.get(&image_id).map_or(&[], Vec::as_slice)
    }

    pub fn build(
        items: &[DisplayItem],
        records: &[RecordedDisplayItem],
        chunks: &[PaintChunk],
    ) -> Self {
        let mut by_image: HashMap<_, Vec<_>> = HashMap::new();
        for (record_index, record) in records.iter().enumerate() {
            let mut ids = std::collections::HashSet::new();
            for item in &items[record.record_begin..record.record_end] {
                if item.resource_id != 0 {
                    ids.insert(item.resource_id);
                }
                if let Some(shader) = &item.paint_shader {
                    if shader.resource_id != 0 {
                        ids.insert(shader.resource_id);
                    }
                }
                for mask in &item.mask_layers {
                    if mask.resource_id != 0 {
                        ids.insert(mask.resource_id);
                    }
                    if let Some(shader) = &mask.paint_shader {
                        if shader.resource_id != 0 {
                            ids.insert(shader.resource_id);
                        }
                    }
                }
            }
            if let Some(chunk) = chunks.iter().find(|chunk| {
                chunk.begin_index as usize <= record_index
                    && record_index < chunk.end_index as usize
            }) {
                let mut effect = Some(chunk.properties.effect.as_ref());
                while let Some(node) = effect {
                    for filter in [&node.filters, &node.backdrop_filters] {
                        for layer in filter.reflection_mask_layers() {
                            if let Some(image) = &layer.image {
                                if image.id != 0 {
                                    ids.insert(image.id);
                                }
                            }
                            if let Some(shader) = &layer.paint_shader {
                                if shader.resource_id != 0 {
                                    ids.insert(shader.resource_id);
                                }
                            }
                        }
                    }
                    effect = node.parent.as_deref();
                }
            }
            for image_id in ids {
                by_image
                    .entry(image_id)
                    .or_default()
                    .push(ImagePaintDependency {
                        record_index,
                        visual_rect: record.visual_rect,
                    });
            }
        }
        Self { by_image }
    }
}

/// Concrete immutable record emitted by document_image and consumed only by
/// raster replay. It contains no retained DOM, layout engine or timeline.
#[derive(Clone)]
pub struct DocumentPaintArtifactRecord {
    pub artifact: Arc<PaintRecording>,
    pub intrinsic_size: image_resource::IntrinsicSize,
    /// Coordinate space used by the nested recording. Responsive SVG may
    /// paint at a container size different from its intrinsic dimensions.
    pub record_size: image_resource::IntrinsicSize,
    pub container_key: image_resource::ContainerKey,
}

#[derive(Clone)]
pub enum PaintMutation {
    ImageChanged {
        image_id: image_resource::ImageId,
        revision: u64,
        frame: Arc<image_resource::DocumentImageFrame>,
    },
}

/// Resource-only changes applied to an already committed artifact. The Page
/// chooses which update entered the pipeline; PaintEngine owns immutable
/// artifact replacement and validation.
pub enum PaintResourceUpdate {
    Snapshot(Arc<PaintResources>),
    Mutations(Vec<PaintMutation>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintMutationResult {
    pub damaged_records: Vec<ImagePaintDependency>,
}

// cpp: paint/paint_engine.h:227-239
#[derive(Clone, Default)]
pub struct PaintRecording {
    /// Source viewport identities published by Paint. None is missing producer
    /// input; the compositor must not invent a visual/layout viewport mode.
    pub compositor_viewport: Option<PaintCompositorViewport>,
    /// Source: paint_artifact.cc:44-49. Captured from the original Paint client
    /// together with its debug name at recording time.
    pub client_owner_node_ids: std::collections::HashMap<usize, i32>,
    // Source: paint_artifact.h:62-68,81-87; paint_artifact.cc:29-41.
    pub client_debug_names: std::collections::HashMap<usize, std::sync::Arc<str>>,
    /// Immutable PaintRecord operations shared by published artifacts. Property
    /// updates retain this backing; an actual operation edit uses Arc::make_mut
    /// to preserve old frame snapshots (Chromium PaintRecord sharing intent).
    pub items: Arc<Vec<DisplayItem>>,
    pub display_items: DisplayItemList,
    pub chunks: PaintChunks,
    /// Property snapshots when the flat records were generated. Scroll-only
    /// updates change chunk properties without changing this coordinate basis.
    pub recorded_properties: Vec<paint_model::paint_property_tree::PropertyTreeState>,
    /// Identity of the immutable display-item recording and its recording
    /// coordinate basis. A retained property-only scroll keeps this value;
    /// recording or changing any draw operation assigns a new value.
    ///
    /// Zero is reserved for externally assembled/test artifacts, for which
    /// downstream caches must retain their complete value proof.
    pub recording_revision: u64,
    pub resources: Option<Arc<PaintResources>>,
    pub image_dependencies: Arc<ImageDependencyIndex>,
    pub caret: Option<CaretGeometry>,
}

impl PartialEq for PaintRecording {
    fn eq(&self, other: &Self) -> bool {
        self.caret == other.caret
            && self.compositor_viewport == other.compositor_viewport
            && self.client_owner_node_ids == other.client_owner_node_ids
            && self.client_debug_names == other.client_debug_names
            && (Arc::ptr_eq(&self.items, &other.items) || self.items == other.items)
            && self.display_items == other.display_items
            && self.image_dependencies == other.image_dependencies
            && self.chunks == other.chunks
            && self
                .chunks
                .iter()
                .zip(&other.chunks)
                .all(|(a, b)| a.properties.same_values(&b.properties))
            && self
                .chunks
                .iter()
                .zip(&other.chunks)
                .enumerate()
                .all(|(index, (a, b))| {
                    self.recorded_properties
                        .get(index)
                        .unwrap_or(&a.properties)
                        .same_values(
                            other
                                .recorded_properties
                                .get(index)
                                .unwrap_or(&b.properties),
                        )
                })
            && match (&self.resources, &other.resources) {
                (None, None) => true,
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                _ => false,
            }
    }
}

impl PaintRecording {
    /// Replace only the immutable replay-resource snapshot. Recorded ops and
    /// property state stay shared; the dependency proof marks affected chunks.
    pub fn ReplaceResources(&mut self, resources: Arc<PaintResources>) {
        self.resources = Some(resources);
        self.RefreshImageContentRevisions();
    }

    /// Apply one validated, content-only image revision to this immutable
    /// artifact value. Arc copy-on-write remains the facade's publication job.
    pub fn ApplyResourceMutation(
        &mut self,
        mutation: PaintMutation,
    ) -> Option<PaintMutationResult> {
        let PaintMutation::ImageChanged {
            image_id,
            revision,
            frame,
        } = mutation;
        if frame.resource_id != image_id || frame.revision != revision {
            return None;
        }
        let old_resources = self.resources.as_ref()?.as_ref();
        let old = old_resources
            .images
            .iter()
            .find(|image| image.id == image_id)?;
        if old.width != frame.intrinsic_size.width || old.height != frame.intrinsic_size.height {
            return None;
        }
        if revision <= old.revision {
            if revision == old.revision
                && !matches!(&old.content,
                    image_resource::PaintImageContent::Document(content)
                    if content.record_for(&frame.container_key)
                        .is_some_and(|record| Arc::ptr_eq(record, &frame.record)))
            {
                return None;
            }
            return Some(PaintMutationResult::default());
        }
        let damaged_records = self.image_dependencies.dependencies(image_id).to_vec();
        let mut resources = old_resources.clone();
        let image = resources
            .images
            .iter_mut()
            .find(|image| image.id == image_id)?;
        image.revision = revision;
        image.UpdateDocumentFrameMetadata(&frame);
        image.document_container_key = Some(frame.container_key.clone());
        let image_resource::PaintImageContent::Document(content) = &image.content else {
            return None;
        };
        image.content = image_resource::PaintImageContent::Document(Arc::new(
            content.with_variant(frame.container_key.clone(), frame.record.clone()),
        ));
        self.resources = Some(Arc::new(resources));
        self.RefreshImageContentRevisions();
        Some(PaintMutationResult { damaged_records })
    }

    /// Rebuild the per-record image-content proof from the canonical retained
    /// dependency index and resource catalog. Returns the records whose sampled
    /// content changed and clears cached-subsequence reuse for their chunks.
    pub fn RefreshImageContentRevisions(&mut self) -> Vec<usize> {
        let mut revisions = vec![Vec::new(); self.display_items.len()];
        if let Some(resources) = self.resources.as_deref() {
            for image in &resources.images {
                for dependency in self.image_dependencies.dependencies(image.id) {
                    if let Some(record) = revisions.get_mut(dependency.record_index) {
                        record.push((image.id, image.revision));
                    }
                }
            }
        }
        for record in &mut revisions {
            record.sort_unstable();
            record.dedup();
        }
        let changed: Vec<_> = self
            .display_items
            .iter()
            .zip(&revisions)
            .enumerate()
            .filter_map(|(index, (item, revision))| {
                (item.image_content_revisions != *revision).then_some(index)
            })
            .collect();
        for (item, revision) in self.display_items.iter_mut().zip(revisions) {
            item.image_content_revisions = revision;
        }
        for chunk in &mut self.chunks {
            if changed.iter().any(|index| {
                chunk.begin_index as usize <= *index && *index < chunk.end_index as usize
            }) {
                chunk.is_moved_from_cached_subsequence = false;
            }
        }
        changed
    }

    pub fn DocumentImageContainerRequirements(
        &self,
    ) -> Vec<(image_resource::ImageId, image_resource::ContainerKey)> {
        fn pattern_requirements(
            shader: &PaintShader,
            output: &mut Vec<(image_resource::ImageId, image_resource::ContainerKey)>,
        ) {
            let Some(pattern) = shader.pattern.as_ref() else {
                return;
            };
            for command in &pattern.commands {
                match command {
                    layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawImage(
                        image,
                    ) => {
                        if let Some(key) = &image.document_container_key {
                            let pair = (image.resource_id, key.clone());
                            if image.resource_id != 0 && !output.contains(&pair) {
                                output.push(pair);
                            }
                        }
                    }
                    layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawShape(
                        shape,
                    )
                    | layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawGlyph(
                        shape,
                    ) => {
                        if let Some(shader) = &shape.fill_server {
                            pattern_requirements(shader, output);
                        }
                        if let Some(shader) = &shape.stroke_server {
                            pattern_requirements(shader, output);
                        }
                    }
                    layoutng_assembly::internal::paint_input::SvgPatternCommand::SaveLayerMask(
                        mask,
                    ) => {
                        let nested = PaintShader {
                            pattern: Some(Arc::new(
                                layoutng_assembly::internal::paint_input::SvgPatternData {
                                    commands: mask.commands.clone(),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        };
                        pattern_requirements(&nested, output);
                    }
                    _ => {}
                }
            }
        }
        let mut output = Vec::new();
        for item in self.items.iter() {
            if let Some(key) = &item.document_image_container_key {
                let pair = (item.resource_id, key.clone());
                if item.resource_id != 0 && !output.contains(&pair) {
                    output.push(pair);
                }
            }
            if let Some(shader) = &item.paint_shader {
                pattern_requirements(shader, &mut output);
            }
            for mask in &item.mask_layers {
                if let Some(key) = &mask.document_image_container_key {
                    let pair = (mask.resource_id, key.clone());
                    if mask.resource_id != 0 && !output.contains(&pair) {
                        output.push(pair);
                    }
                }
                if let Some(shader) = &mask.paint_shader {
                    pattern_requirements(shader, &mut output);
                }
            }
        }
        // Reflection masks live on effect nodes rather than display items.
        // They use the same per-container SVG records and must be fulfilled
        // before the second recording snapshots their resource payloads.
        let mut visited = std::collections::HashSet::new();
        for chunk in &self.chunks {
            let mut effect = Some(chunk.properties.effect.as_ref());
            while let Some(node) = effect {
                if !visited.insert(node as *const _) {
                    break;
                }
                for filter in [&node.filters, &node.backdrop_filters] {
                    for layer in filter.reflection_mask_layers() {
                        if let (Some(image), Some(key)) =
                            (&layer.image, &layer.document_image_container_key)
                        {
                            let pair = (image.id, key.clone());
                            if image.id != 0 && !output.contains(&pair) {
                                output.push(pair);
                            }
                        }
                        if let Some(shader) = &layer.paint_shader {
                            pattern_requirements(shader, &mut output);
                        }
                    }
                }
                effect = node.parent.as_deref();
            }
        }
        output
    }

    /// Whether this immutable artifact carries the exact native document
    /// record named by every display item. A newer Page catalog cannot certify
    /// an older artifact snapshot.
    pub fn HasExactDocumentImageContainerRecords(&self) -> bool {
        let requirements = self.DocumentImageContainerRequirements();
        let Some(resources) = self.resources.as_ref() else {
            return requirements.is_empty();
        };
        requirements.iter().all(|(resource_id, container)| {
            resources
                .images
                .iter()
                .find(|image| image.id == *resource_id)
                .is_some_and(|image| match &image.content {
                    image_resource::PaintImageContent::Document(content) => content
                        .exact_record_for(container)
                        .and_then(|record| {
                            downcast_document_paint_record::<DocumentPaintArtifactRecord>(record)
                        })
                        .is_some(),
                    image_resource::PaintImageContent::Bitmap(_) => false,
                })
        })
    }
}
