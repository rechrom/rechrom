use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    CaretGeometry, DisplayItem, DisplayItemList, DisplayItemListJsonOption, RecordedDisplayItem,
};
use image_resource::downcast_document_paint_record;
use layoutng_assembly::fragment_tree::PaintResources;
use layoutng_assembly::internal::paint_input::{PaintShader, PaintShaderKind, SvgPatternCommand};
use paint_chunk::{PaintChunk, PaintChunks};
use paint_model::PaintCompositorViewport;
use paint_model::PaintRect;

/// Shared immutable artifact storage. Cloning a published PaintRecording
/// keeps its backing instead of copying display items, chunks, client maps, or
/// property directories. Mutation remains confined to a fresh Paint pass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SharedArtifactData<T>(Arc<T>);

impl<T> From<T> for SharedArtifactData<T> {
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl From<Vec<RecordedDisplayItem>> for SharedArtifactData<DisplayItemList> {
    fn from(value: Vec<RecordedDisplayItem>) -> Self {
        DisplayItemList::from(value).into()
    }
}

impl<T> std::ops::Deref for SharedArtifactData<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> SharedArtifactData<T> {
    pub fn PtrEq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Clone> std::ops::DerefMut for SharedArtifactData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

impl<'a, T> IntoIterator for &'a SharedArtifactData<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.as_ref().into_iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut SharedArtifactData<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        Arc::make_mut(&mut self.0).into_iter()
    }
}

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

// cc/paint/paint_shader.cc:319-349 distinguishes an image shader from a
// PaintRecord-backed shader.  An SVG vector pattern's resource_id names the
// paint server; it is not a PaintImage id.  Only the bitmap-pattern variant
// owns that direct image, while a vector pattern recursively owns images in
// its recorded commands.
fn VisitShaderImageIds(shader: &PaintShader, visit: &mut dyn FnMut(image_resource::ImageId)) {
    let Some(pattern) = shader.pattern.as_ref() else {
        if shader.kind == PaintShaderKind::kPattern && shader.resource_id != 0 {
            visit(shader.resource_id);
        }
        return;
    };
    VisitPatternImageIds(&pattern.commands, visit);
}

fn VisitPatternImageIds(
    commands: &[SvgPatternCommand],
    visit: &mut dyn FnMut(image_resource::ImageId),
) {
    for command in commands {
        match command {
            SvgPatternCommand::DrawImage(image) => {
                if image.resource_id != 0 {
                    visit(image.resource_id);
                }
            }
            SvgPatternCommand::DrawShape(shape) | SvgPatternCommand::DrawGlyph(shape) => {
                if let Some(shader) = &shape.fill_server {
                    VisitShaderImageIds(shader, visit);
                }
                if let Some(shader) = &shape.stroke_server {
                    VisitShaderImageIds(shader, visit);
                }
            }
            SvgPatternCommand::DrawText(text) => {
                if let Some(shader) = &text.paint.fill_server {
                    VisitShaderImageIds(shader, visit);
                }
                if let Some(shader) = &text.paint.stroke_server {
                    VisitShaderImageIds(shader, visit);
                }
            }
            SvgPatternCommand::SaveLayerMask(mask) => {
                VisitPatternImageIds(&mask.commands, visit);
            }
            _ => {}
        }
    }
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
                    VisitShaderImageIds(shader, &mut |id| {
                        ids.insert(id);
                    });
                }
                for mask in &item.mask_layers {
                    if mask.resource_id != 0 {
                        ids.insert(mask.resource_id);
                    }
                    if let Some(shader) = &mask.paint_shader {
                        VisitShaderImageIds(shader, &mut |id| {
                            ids.insert(id);
                        });
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
                                VisitShaderImageIds(shader, &mut |id| {
                                    ids.insert(id);
                                });
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

#[derive(Clone, Default)]
pub struct PaintRecording {
    /// Source viewport identities published by Paint. None is missing producer
    /// input; the compositor must not invent a visual/layout viewport mode.
    pub compositor_viewport: Option<PaintCompositorViewport>,
    /// Source: paint_artifact.cc:44-49. Captured from the original Paint client
    /// together with its debug name at recording time.
    pub client_owner_node_ids: SharedArtifactData<HashMap<usize, i32>>,
    // Source: paint_artifact.h:62-68,81-87; paint_artifact.cc:29-41.
    pub client_debug_names: SharedArtifactData<HashMap<usize, Arc<str>>>,
    /// Immutable PaintRecord operations shared by published artifacts. Property
    /// updates retain this backing; an actual operation edit uses Arc::make_mut
    /// to preserve old frame snapshots (Chromium PaintRecord sharing intent).
    pub items: Arc<Vec<DisplayItem>>,
    pub display_items: SharedArtifactData<DisplayItemList>,
    pub chunks: SharedArtifactData<PaintChunks>,
    /// Identity of the immutable display-item recording and its recording
    /// coordinate basis. A retained property-only scroll keeps this value;
    /// recording or changing any draw operation assigns a new value.
    ///
    /// Zero is reserved for externally assembled/test artifacts, for which
    /// downstream caches must retain their complete value proof.
    pub recording_revision: u64,
    pub resources: Option<Arc<PaintResources>>,
    pub image_dependencies: Arc<ImageDependencyIndex>,
    /// Resource revision proof is a small sidecar. Image changes replace this
    /// value without rewriting or copying the immutable recording.
    pub image_content_revisions: Arc<Vec<Vec<(image_resource::ImageId, u64)>>>,
    /// Chunk indices whose immutable records sample a changed resource in this
    /// artifact version. This replaces mutation of PaintChunk cache flags.
    pub resource_changed_chunks: Arc<HashSet<usize>>,
    /// Resource identities whose LayoutImage::Paint entry point ran while
    /// producing this artifact. This is deliberately not derived from image
    /// draw operations: ImagePainter may early-return after LayoutImage has
    /// already performed its animated-image UseCounter check.
    pub painted_layout_image_resource_ids: SharedArtifactData<HashSet<image_resource::ImageId>>,
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
            && self.image_content_revisions == other.image_content_revisions
            && self.resource_changed_chunks == other.resource_changed_chunks
            && self.painted_layout_image_resource_ids == other.painted_layout_image_resource_ids
            && self.chunks == other.chunks
            && self
                .chunks
                .iter()
                .zip(&other.chunks)
                .all(|(a, b)| a.properties.same_values(&b.properties))
            && match (&self.resources, &other.resources) {
                (None, None) => true,
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                _ => false,
            }
    }
}

impl PaintRecording {
    pub fn PaintedLayoutImageResourceIds(
        &self,
    ) -> impl Iterator<Item = image_resource::ImageId> + '_ {
        self.painted_layout_image_resource_ids.iter().copied()
    }
    // Source: paint_artifact.h:38.
    pub fn IsEmpty(&self) -> bool {
        self.chunks.is_empty()
    }

    // Source: paint_artifact.h:40-43.
    pub fn GetDisplayItemList(&self) -> &DisplayItemList {
        &self.display_items
    }

    // Source: paint_artifact.h:40-43. PaintController is the mutable caller.
    pub fn GetDisplayItemListMut(&mut self) -> &mut DisplayItemList {
        &mut self.display_items
    }

    // Source: paint_artifact.h:45-46.
    pub fn GetPaintChunks(&self) -> &PaintChunks {
        &self.chunks
    }

    // Source: paint_artifact.h:45-46. PaintController is the mutable caller.
    pub fn GetPaintChunksMut(&mut self) -> &mut PaintChunks {
        &mut self.chunks
    }

    // Source: paint_artifact.h:48-53.
    pub fn DisplayItemsInChunk(&self, chunk_index: usize) -> &[RecordedDisplayItem] {
        let chunk = self
            .chunks
            .get(chunk_index)
            .expect("paint chunk index must be valid");
        self.display_items
            .ItemsInRange(chunk.begin_index as usize, chunk.end_index as usize)
    }

    // Source: paint_artifact.cc:14-23.
    pub fn ApproximateUnsharedMemoryUsage(&self) -> usize {
        let mut total = std::mem::size_of::<Self>() + self.display_items.MemoryUsageInBytes()
            - std::mem::size_of::<DisplayItemList>()
            + self.chunks.capacity() * std::mem::size_of::<PaintChunk>();
        for chunk in &self.chunks {
            let chunk_size = chunk.MemoryUsageInBytes();
            debug_assert!(chunk_size >= std::mem::size_of::<PaintChunk>());
            total += chunk_size - std::mem::size_of::<PaintChunk>();
        }
        total
    }

    // Source: paint_artifact.h:62; paint_artifact.cc:31-35.
    pub fn RecordDebugInfo(&mut self, client_id: usize, name: Arc<str>, owner_node_id: i32) {
        self.client_debug_names.insert(client_id, name);
        self.client_owner_node_ids.insert(client_id, owner_node_id);
    }

    // Source: paint_artifact.h:63-68; paint_artifact.cc:37-42.
    pub fn ClientDebugName(&self, client_id: usize) -> &str {
        self.client_debug_names
            .get(&client_id)
            .map_or("", |name| name.as_ref())
    }

    // Source: paint_artifact.h:69; paint_artifact.cc:44-50.
    pub fn ClientOwnerNodeId(&self, client_id: usize) -> i32 {
        self.client_owner_node_ids
            .get(&client_id)
            .copied()
            .unwrap_or(0)
    }

    // Source: paint_artifact.h:70; paint_artifact.cc:52-62.
    pub fn IdAsString(&self, id: &paint_model::display_item_id::DisplayItemId) -> String {
        let debug_name = self.ClientDebugName(id.client_id as usize);
        if debug_name.is_empty() {
            id.ToString()
        } else {
            format!(
                "{:#x}:{}:{}:{}",
                id.client_id, debug_name, id.r#type, id.fragment
            )
        }
    }

    // Source: paint_chunk.cc:113-116. This method lives with PaintArtifact in
    // Rust to preserve the paint_chunk -> paint_record package direction.
    pub fn PaintChunkToString(&self, chunk: &PaintChunk, concise: bool) -> String {
        chunk.ToStringWithId(&self.IdAsString(&chunk.id), concise)
    }

    // Source: paint_artifact.cc:64-94.
    pub fn ToJSON(&self) -> serde_json::Value {
        self.ChunksAsJSON(0, self.chunks.len(), DisplayItemListJsonOption::kDefault)
    }

    pub fn ChunksAsJSON(
        &self,
        start_chunk_index: usize,
        end_chunk_index: usize,
        option: DisplayItemListJsonOption,
    ) -> serde_json::Value {
        assert!(start_chunk_index <= end_chunk_index);
        assert!(end_chunk_index <= self.chunks.len());
        let chunks = self.chunks[start_chunk_index..end_chunk_index]
            .iter()
            .enumerate()
            .map(|(offset, chunk)| {
                let chunk_index = start_chunk_index + offset;
                let mut object = serde_json::Map::new();
                object.insert(
                    "chunk".into(),
                    serde_json::Value::String(format!(
                        "{} {}",
                        self.ClientDebugName(chunk.id.client_id as usize),
                        self.IdAsString(&chunk.id)
                    )),
                );
                object.insert(
                    "state".into(),
                    serde_json::Value::String(chunk.properties.ToString()),
                );
                object.insert(
                    "bounds".into(),
                    serde_json::Value::String(format!(
                        "{},{} {}x{}",
                        chunk.bounds.x, chunk.bounds.y, chunk.bounds.width, chunk.bounds.height
                    )),
                );
                if option == DisplayItemListJsonOption::kShowPaintRecords {
                    object.insert(
                        "chunkData".into(),
                        serde_json::Value::String(self.PaintChunkToString(chunk, false)),
                    );
                }
                object.insert(
                    "displayItems".into(),
                    DisplayItemList::DisplayItemsAsJSON(
                        self,
                        chunk.begin_index as usize,
                        self.DisplayItemsInChunk(chunk_index),
                        option,
                    ),
                );
                serde_json::Value::Object(object)
            })
            .collect();
        serde_json::Value::Array(chunks)
    }

    // Source: paint_artifact.cc:96-100. Rechrom's extra fields are derived
    // members of the same artifact and are cleared with their source owners.
    pub fn clear(&mut self) {
        Arc::make_mut(&mut self.items).clear();
        self.display_items.clear();
        self.chunks.clear();
        self.client_debug_names.clear();
        self.client_owner_node_ids.clear();
        self.image_dependencies = Arc::new(ImageDependencyIndex::default());
        self.image_content_revisions = Arc::new(Vec::new());
        self.resource_changed_chunks = Arc::new(HashSet::new());
        self.painted_layout_image_resource_ids.clear();
        self.compositor_viewport = None;
        self.recording_revision = 0;
        self.resources = None;
        self.caret = None;
    }

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
    /// dependency index and resource catalog. The proof is a separate immutable
    /// sidecar, so resource updates never mutate/copy display items or chunks.
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
            .image_content_revisions
            .iter()
            .zip(&revisions)
            .enumerate()
            .filter_map(|(index, (old, revision))| (old != revision).then_some(index))
            .collect();
        if self.image_content_revisions.len() != revisions.len() {
            // A newly recorded artifact has no prior sidecar; every dependency
            // is already covered by ordinary recording invalidation.
            self.image_content_revisions = Arc::new(revisions);
            self.resource_changed_chunks = Arc::new(HashSet::new());
            return Vec::new();
        }
        self.image_content_revisions = Arc::new(revisions);
        self.resource_changed_chunks = Arc::new(
            self.chunks
                .iter()
                .enumerate()
                .filter_map(|(chunk_index, chunk)| {
                    changed
                        .iter()
                        .any(|record| {
                            chunk.begin_index as usize <= *record
                                && *record < chunk.end_index as usize
                        })
                        .then_some(chunk_index)
                })
                .collect(),
        );
        changed
    }

    pub fn ImageContentRevisionsInChunk(
        &self,
        chunk_index: usize,
    ) -> &[Vec<(image_resource::ImageId, u64)>] {
        let chunk = &self.chunks[chunk_index];
        &self.image_content_revisions[chunk.begin_index as usize..chunk.end_index as usize]
    }

    pub fn ChunkHasResourceChange(&self, chunk_index: usize) -> bool {
        self.resource_changed_chunks.contains(&chunk_index)
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

    /// Whether this immutable artifact carries the exact source document
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

// Source: paint_artifact.cc:102-104.
impl std::fmt::Display for PaintRecording {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string_pretty(&self.ToJSON()).map_err(|_| std::fmt::Error)?;
        formatter.write_str(&json)
    }
}

#[cfg(test)]
mod paint_artifact_tests {
    use super::*;
    use paint_model::display_item_id::{DisplayItemId, DisplayItemIdType};

    fn artifact() -> PaintRecording {
        let mut chunk = PaintChunk::default();
        chunk.id = DisplayItemId {
            client_id: 17,
            r#type: DisplayItemIdType::kLayerChunk,
            fragment: 3,
        };
        chunk.bounds = PaintRect {
            x: 1.0,
            y: 2.0,
            width: 30.0,
            height: 40.0,
        };
        let mut artifact = PaintRecording {
            chunks: vec![chunk].into(),
            items: Arc::new(vec![DisplayItem::default()]),
            ..Default::default()
        };
        artifact.RecordDebugInfo(17, Arc::<str>::from("artifact-client"), 29);
        artifact
    }

    #[test]
    fn json_and_display_use_resident_chunk_and_client_state() {
        let artifact = artifact();
        let json = artifact.ToJSON();
        let chunk = &json.as_array().unwrap()[0];
        assert!(chunk["chunk"].as_str().unwrap().contains("artifact-client"));
        assert_eq!(chunk["bounds"], "1,2 30x40");
        assert_eq!(chunk["displayItems"], serde_json::json!([]));
        let pretty = artifact.to_string();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&pretty).unwrap(),
            json
        );
    }

    #[test]
    fn memory_usage_counts_list_and_chunk_capacity() {
        let artifact = artifact();
        assert!(artifact.ApproximateUnsharedMemoryUsage() >= std::mem::size_of::<PaintRecording>());
    }

    #[test]
    fn clear_removes_source_and_derived_artifact_state() {
        let mut artifact = artifact();
        artifact.painted_layout_image_resource_ids.insert(9);
        artifact.recording_revision = 44;
        artifact.clear();
        assert!(artifact.IsEmpty());
        assert!(artifact.GetDisplayItemList().IsEmpty());
        assert!(artifact.items.is_empty());
        assert!(artifact.client_debug_names.is_empty());
        assert!(artifact.painted_layout_image_resource_ids.is_empty());
        assert_eq!(artifact.recording_revision, 0);
    }

    #[test]
    fn vector_pattern_identity_is_not_an_image_dependency() {
        use layoutng_assembly::internal::paint_input::{
            PaintShaderKind, SvgPatternData, SvgPatternImage,
        };

        let vector_pattern = PaintShader {
            kind: PaintShaderKind::kPattern,
            // SVGPaintServer's stable LayoutObject identity, not PaintImage.
            resource_id: 3160,
            pattern: Some(Arc::new(SvgPatternData {
                commands: vec![SvgPatternCommand::DrawImage(SvgPatternImage {
                    resource_id: 7,
                    ..Default::default()
                })],
                ..Default::default()
            })),
            ..Default::default()
        };
        let bitmap_pattern = PaintShader {
            kind: PaintShaderKind::kPattern,
            resource_id: 8,
            ..Default::default()
        };
        let items = vec![
            DisplayItem {
                paint_shader: Some(vector_pattern),
                ..Default::default()
            },
            DisplayItem {
                paint_shader: Some(bitmap_pattern),
                ..Default::default()
            },
        ];
        let record = RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar: None,
            foreign_layer: None,
            kind: crate::RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: PaintRect::default(),
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: layoutng_assembly::fragment_tree::RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: PaintRect::default(),
            record_background: Default::default(),
            record_has_text: false,
            record_begin: 0,
            record_end: items.len(),
            scroll_translation: None,
        };
        let index = ImageDependencyIndex::build(&items, &[record], &[]);
        let mut ids = index.image_ids().collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(ids, vec![7, 8]);
        assert!(index.dependencies(3160).is_empty());
    }
}
