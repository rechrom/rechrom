//! Reject producer blockers before entering canonical algorithms. Traversal
//! borrows the actual graph; the worklist retains allocation identities only.
use super::{paint_property_tree, PaintArtifact};
use std::{collections::HashSet, sync::Arc};
#[derive(Debug)]
pub enum PaintInputError {
    MissingImageResource {
        resource_id: u64,
    },
    ImageMetadata {
        resource_id: u64,
        blocker: paint::paint_engine::ImageMetadataUnavailable,
    },
    ImagePlayback {
        resource_id: u64,
        reason: &'static str,
    },
    MissingViewportMetadata,
    InvalidViewportMetadata {
        reason: &'static str,
    },
    MissingStickyConstraint {
        node: Arc<paint_property_tree::TransformPaintPropertyNode>,
    },
    InvalidCanvasChild {
        node: Arc<paint_property_tree::EffectPaintPropertyNode>,
    },
    InvalidRecordRange {
        item: usize,
        begin: usize,
        end: usize,
        ops: usize,
    },
    MissingScrollCompositorData {
        node: Arc<paint_property_tree::ScrollPaintPropertyNode>,
    },
    MissingChunkCompositorData {
        chunk: usize,
    },
    UnsupportedDisplayItem {
        item: usize,
        item_type: u8,
    },
    EffectFilter {
        node: Arc<paint_property_tree::EffectPaintPropertyNode>,
        backdrop: bool,
        blocker: paint_model::property_filter_operations::FilterInputBlocker,
    },
    ClipPath {
        node: Arc<paint_property_tree::ClipPaintPropertyNode>,
        reason: String,
    },
    InvalidChunkRange {
        chunk: usize,
        begin: u32,
        end: u32,
        items: usize,
    },
}
impl std::fmt::Display for PaintInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImagePlayback {
                resource_id,
                reason,
            } => write!(f, "Paint image {resource_id}: {reason}"),
            Self::MissingImageResource { resource_id } => {
                write!(f, "paint image {resource_id} has no actual resource owner")
            }
            Self::ImageMetadata {
                resource_id,
                blocker,
            } => write!(f, "paint image {resource_id}: {blocker}"),
            Self::MissingViewportMetadata => write!(
                f,
                "PaintArtifact lacks its producer's compositor viewport mode and native identities"
            ),
            Self::InvalidViewportMetadata { reason } => {
                write!(f, "PaintArtifact compositor viewport: {reason}")
            }
            Self::MissingStickyConstraint { node } => write!(
                f,
                "transform {} lacks the native compositor sticky constraint",
                node.id
            ),
            Self::InvalidCanvasChild { node } => write!(
                f,
                "effect {} has an incomplete canvas-child property state",
                node.id
            ),
            Self::InvalidRecordRange {
                item,
                begin,
                end,
                ops,
            } => write!(
                f,
                "display item {item} has record range {begin}..{end} outside {ops} operations"
            ),
            Self::MissingScrollCompositorData { node } => write!(
                f,
                "scroll {} lacks complete native scroll compositing/snap metadata",
                node.id
            ),
            Self::MissingChunkCompositorData { chunk } => {
                write!(f, "chunk {chunk} lacks producer compositor derived data")
            }
            Self::UnsupportedDisplayItem { item, item_type } => write!(
                f,
                "display item {item} type {item_type} lacks its native compositor layer owner"
            ),
            Self::EffectFilter {
                node,
                backdrop,
                blocker,
            } => write!(
                f,
                "effect {} {}filter: {}",
                node.id,
                if *backdrop { "backdrop " } else { "" },
                blocker
            ),
            Self::ClipPath { node, reason } => write!(f, "clip {}: {}", node.id, reason),
            Self::InvalidChunkRange {
                chunk,
                begin,
                end,
                items,
            } => write!(
                f,
                "chunk {chunk} has range {begin}..{end} outside {items} display items"
            ),
        }
    }
}
impl std::error::Error for PaintInputError {}
enum Node {
    Transform(Arc<paint_property_tree::TransformPaintPropertyNode>),
    Clip(Arc<paint_property_tree::ClipPaintPropertyNode>),
    Effect(Arc<paint_property_tree::EffectPaintPropertyNode>),
}
pub fn ValidateArtifact(artifact: &PaintArtifact) -> Result<(), PaintInputError> {
    let mut artifacts = HashSet::new();
    ValidateArtifactGraph(artifact, &mut artifacts)
}

fn ValidateArtifactGraph(
    artifact: &PaintArtifact,
    artifacts: &mut HashSet<usize>,
) -> Result<(), PaintInputError> {
    if !artifacts.insert(artifact as *const PaintArtifact as usize) {
        return Ok(());
    }
    let viewport = artifact
        .compositor_viewport
        .as_ref()
        .ok_or(PaintInputError::MissingViewportMetadata)?;
    let layout_scroll = viewport.layout_scroll_translation.as_ref().ok_or(
        PaintInputError::InvalidViewportMetadata {
            reason: "producer did not publish its actual LayoutView scroll translation",
        },
    )?;
    if !viewport
        .scroll_translation_nodes
        .iter()
        .any(|node| Arc::ptr_eq(node, layout_scroll))
    {
        return Err(PaintInputError::InvalidViewportMetadata {
            reason: "LayoutView scroll translation is absent from the producer scroll-node list",
        });
    }
    if viewport
        .scroll_translation_nodes
        .iter()
        .any(|node| paint_model::property_cell::with_access(|access| node.ScrollNode(access).is_none()))
    {
        return Err(PaintInputError::InvalidViewportMetadata {
            reason:
                "producer scroll-translation list contains a transform without a Paint scroll node",
        });
    }
    let mut seen = HashSet::new();
    let mut pending = Vec::new();
    pending.push(Node::Clip(viewport.viewport_clip.clone()));
    for node in &viewport.scroll_translation_nodes {
        pending.push(Node::Transform(node.clone()));
    }
    let access = ::paint::artifact_access();
    let display_items = artifact.display_items.get(&access);
    for (index, chunk) in artifact.chunks.get(&access).iter().enumerate() {
        if chunk.compositor.is_none() {
            return Err(PaintInputError::MissingChunkCompositorData { chunk: index });
        }
        if chunk.begin_index > chunk.end_index
            || chunk.end_index as usize > display_items.len()
        {
            return Err(PaintInputError::InvalidChunkRange {
                chunk: index,
                begin: chunk.begin_index,
                end: chunk.end_index,
                items: display_items.len(),
            });
        }
        pending.push(Node::Transform(chunk.properties.transform.clone()));
        pending.push(Node::Clip(chunk.properties.clip.clone()));
        pending.push(Node::Effect(chunk.properties.effect.clone()));
    }
    for (index, item) in display_items.iter().enumerate() {
        if item.record_begin > item.record_end || item.record_end > artifact.items.len() {
            return Err(PaintInputError::InvalidRecordRange {
                item: index,
                begin: item.record_begin,
                end: item.record_end,
                ops: artifact.items.len(),
            });
        }
        use paint_model::display_item_id::DisplayItemIdType as T;
        if (T::kForeignLayerFirst.0..=T::kForeignLayerLast.0).contains(&item.id.r#type.0)
            && (item.kind != super::RecordedDisplayItemKind::ForeignLayer
                || item.foreign_layer.is_none())
        {
            return Err(PaintInputError::UnsupportedDisplayItem {
                item: index,
                item_type: item.id.r#type.0,
            });
        }
    }
    // These are only stable resource identities; record/shader owners stay in
    // the original artifact. Nested SVG pattern images are not all present in
    // the producer's direct-use invalidation index.
    let mut image_ids = artifact
        .image_dependencies
        .image_ids()
        .collect::<std::collections::BTreeSet<_>>();
    for op in artifact.AllOps(&access) {
        if op.resource_id != 0 {
            image_ids.insert(op.resource_id);
        }
        if let Some(shader) = &op.paint_shader {
            super::metadata::visit_shader_images(shader, &mut |id| {
                image_ids.insert(id);
            });
        }
        for layer in &op.mask_layers {
            if layer.resource_id != 0 {
                image_ids.insert(layer.resource_id);
            }
            if let Some(shader) = &layer.paint_shader {
                super::metadata::visit_shader_images(shader, &mut |id| {
                    image_ids.insert(id);
                });
            }
        }
    }
    for resource_id in image_ids {
        let resources = artifact
            .resources
            .as_ref()
            .ok_or(PaintInputError::MissingImageResource { resource_id })?;
        let Some(image) = resources
            .images
            .iter()
            .find(|image| image.id == resource_id)
        else {
            if resources
                .canvases
                .iter()
                .any(|canvas| canvas.resource_id == resource_id)
            {
                continue;
            }
            return Err(PaintInputError::MissingImageResource { resource_id });
        };
        if let paint::paint_engine::PaintImageContent::Document(content) = &image.content {
            let record = image
                .document_container_key
                .as_ref()
                .and_then(|key| content.exact_record_for(key))
                .and_then(|record| {
                    paint::paint_engine::downcast_document_paint_record::<
                        paint::paint_engine::DocumentPaintArtifactRecord,
                    >(record)
                })
                .ok_or(PaintInputError::ImagePlayback {
                    resource_id,
                    reason: "document image has no published PaintRecord",
                })?;
            let artifact = paint::paint_engine::DocumentRecordArtifact(record).ok_or(
                PaintInputError::ImagePlayback {
                    resource_id,
                    reason: "document image record has no Paint recording",
                },
            )?;
            ValidateArtifactGraph(artifact, artifacts)?;
        }
        if image
            .ShouldAnimate()
            .map_err(|blocker| PaintInputError::ImageMetadata {
                resource_id,
                blocker: blocker.clone(),
            })?
            && image.BitmapAnimationFrames().is_none()
        {
            return Err(PaintInputError::ImagePlayback {
                resource_id,
                reason: "animated image lacks its per-frame timeline owner",
            });
        }
    }
    while let Some(node) = pending.pop() {
        match node {
            Node::Transform(node) => {
                if !seen.insert((0, Arc::as_ptr(&node) as usize)) {
                    continue;
                }
                let (missing_sticky, links) = paint_model::property_cell::with_access(|access| {
                    let state = node.state(access);
                    (
                        state.RequiresCompositingForStickyPosition()
                            && state.GetStickyConstraint().is_none(),
                        state.links(),
                    )
                });
                if missing_sticky {
                    return Err(PaintInputError::MissingStickyConstraint { node: node.clone() });
                }
                if let Some(parent) = &links.parent {
                    pending.push(Node::Transform(parent.clone()));
                }
                if let Some(parent) = &links.scroll_parent_scroll_translation {
                    pending.push(Node::Transform(parent.clone()));
                }
                if let Some(scroll) = &links.scroll {
                    let (missing_snap_data, overflow_clip) =
                        paint_model::property_cell::with_access(|access| {
                            let state = scroll.state(access);
                            (
                                state.parent.is_some()
                                    && !state.compositor.snap_container_data_absent
                                    && state.compositor.snap_container_data.is_none(),
                                state.overflow_clip.clone(),
                            )
                        });
                    if missing_snap_data {
                        return Err(PaintInputError::MissingScrollCompositorData {
                            node: scroll.clone(),
                        });
                    }
                    if let Some(clip) = overflow_clip {
                        pending.push(Node::Clip(clip));
                    }
                }
            }
            Node::Clip(node) => {
                if !seen.insert((1, Arc::as_ptr(&node) as usize)) {
                    continue;
                }
                let (error, parent, transform, effect) =
                    paint_model::property_cell::with_access(|access| {
                        let state = node.state(access);
                        (
                            state.clip_path_error.clone(),
                            state.parent.clone(),
                            state.local_transform_space.clone(),
                            state.pixel_moving_filter.clone(),
                        )
                    });
                if let Some(reason) = error {
                    return Err(PaintInputError::ClipPath {
                        node: node.clone(),
                        reason,
                    });
                }
                if let Some(parent) = parent {
                    pending.push(Node::Clip(parent));
                }
                pending.push(Node::Transform(transform));
                if let Some(effect) = effect {
                    pending.push(Node::Effect(effect));
                }
            }
            Node::Effect(node) => {
                if !seen.insert((2, Arc::as_ptr(&node) as usize)) {
                    continue;
                }
                let (invalid_canvas_child, blocker, links) =
                    paint_model::property_cell::with_access(|access| {
                        let state = node.state(access);
                        let blocker = [(false, &state.filters), (true, &state.backdrop_filters)]
                            .into_iter()
                            .find_map(|(backdrop, filters)| {
                                filters.blocker().map(|blocker| (backdrop, blocker.clone()))
                            });
                        (
                            state.compositor.canvas_child_state.is_some()
                                && !state.HasCanvasChildState(),
                            blocker,
                            state.links(),
                        )
                    });
                if invalid_canvas_child {
                    return Err(PaintInputError::InvalidCanvasChild { node: node.clone() });
                }
                if let Some((backdrop, blocker)) = blocker {
                    return Err(PaintInputError::EffectFilter {
                        node: node.clone(),
                        backdrop,
                        blocker,
                    });
                }
                if let Some(parent) = links.parent {
                    pending.push(Node::Effect(parent));
                }
                pending.push(Node::Transform(links.local_transform_space));
                if let Some(clip) = links.output_clip {
                    pending.push(Node::Clip(clip));
                }
                if let Some(clip) = links.backdrop_filter_clip {
                    pending.push(Node::Clip(clip));
                }
            }
        }
    }
    Ok(())
}
impl From<PaintInputError> for crate::commit::layerize_error::LayerizeError {
    fn from(error: PaintInputError) -> Self {
        use crate::commit::layerize_error::LayerizeErrorKind as K;
        use PaintInputError as E;
        let kind = match &error {
            E::MissingImageResource { .. } => K::MissingImageResource,
            E::ImageMetadata { .. } => K::ImageMetadata,
            E::ImagePlayback { .. } => K::ImagePlayback,
            E::MissingViewportMetadata => K::MissingViewportMetadata,
            E::InvalidViewportMetadata { .. } => K::InvalidViewportMetadata,
            E::MissingStickyConstraint { .. } => K::MissingStickyConstraint,
            E::InvalidCanvasChild { .. } => K::InvalidCanvasChild,
            E::InvalidRecordRange { .. } => K::InvalidRecordRange,
            E::MissingScrollCompositorData { .. } => K::MissingScrollCompositorData,
            E::MissingChunkCompositorData { .. } => K::MissingChunkCompositorData,
            E::UnsupportedDisplayItem { .. } => K::UnsupportedDisplayItem,
            E::EffectFilter { .. } => K::EffectFilter,
            E::ClipPath { .. } => K::ClipPath,
            E::InvalidChunkRange { .. } => K::InvalidChunkRange,
        };
        Self {
            kind,
            detail: error.to_string(),
        }
    }
}
