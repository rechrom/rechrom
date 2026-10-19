//! Reject producer blockers before entering canonical algorithms. Traversal
//! borrows the actual graph; the worklist retains allocation identities only.
use super::{native, PaintArtifact};
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
        node: Arc<native::TransformPaintPropertyNode>,
    },
    UnsupportedCanvasChild {
        node: Arc<native::EffectPaintPropertyNode>,
    },
    InvalidRecordRange {
        item: usize,
        begin: usize,
        end: usize,
        ops: usize,
    },
    MissingScrollCompositorData {
        node: Arc<native::ScrollPaintPropertyNode>,
    },
    MissingChunkCompositorData {
        chunk: usize,
    },
    UnsupportedDisplayItem {
        item: usize,
        item_type: u8,
    },
    EffectFilter {
        node: Arc<native::EffectPaintPropertyNode>,
        backdrop: bool,
        blocker: paint::property_filter_operations::FilterInputBlocker,
    },
    ClipPath {
        node: Arc<native::ClipPaintPropertyNode>,
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
            Self::UnsupportedCanvasChild { node } => write!(
                f,
                "effect {} lacks native canvas-child CustomData recording support",
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
    Transform(Arc<native::TransformPaintPropertyNode>),
    Clip(Arc<native::ClipPaintPropertyNode>),
    Effect(Arc<native::EffectPaintPropertyNode>),
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
    let mut seen = HashSet::new();
    let mut pending = Vec::new();
    pending.push(Node::Clip(viewport.viewport_clip.clone()));
    for node in &viewport.scroll_translation_nodes {
        pending.push(Node::Transform(node.clone()));
    }
    for (index, chunk) in artifact.chunks.iter().enumerate() {
        if chunk.compositor.is_none() {
            return Err(PaintInputError::MissingChunkCompositorData { chunk: index });
        }
        if chunk.begin_index > chunk.end_index
            || chunk.end_index as usize > artifact.display_items.len()
        {
            return Err(PaintInputError::InvalidChunkRange {
                chunk: index,
                begin: chunk.begin_index,
                end: chunk.end_index,
                items: artifact.display_items.len(),
            });
        }
        pending.push(Node::Transform(chunk.properties.transform.clone()));
        pending.push(Node::Clip(chunk.properties.clip.clone()));
        pending.push(Node::Effect(chunk.properties.effect.clone()));
    }
    for (index, item) in artifact.display_items.iter().enumerate() {
        if item.record_begin > item.record_end || item.record_end > artifact.items.len() {
            return Err(PaintInputError::InvalidRecordRange {
                item: index,
                begin: item.record_begin,
                end: item.record_end,
                ops: artifact.items.len(),
            });
        }
        use paint::display_item_id::DisplayItemIdType as T;
        if (T::kForeignLayerFirst.0..=T::kForeignLayerLast.0).contains(&item.id.r#type.0) {
            return Err(PaintInputError::UnsupportedDisplayItem {
                item: index,
                item_type: item.id.r#type.0,
            });
        }
    }
    for state in &artifact.recorded_properties {
        pending.push(Node::Transform(state.transform.clone()));
        pending.push(Node::Clip(state.clip.clone()));
        pending.push(Node::Effect(state.effect.clone()));
    }
    // These are only stable resource identities; record/shader owners stay in
    // the original artifact. Nested SVG pattern images are not all present in
    // the producer's direct-use invalidation index.
    let mut image_ids = artifact
        .image_dependencies
        .image_ids()
        .collect::<std::collections::BTreeSet<_>>();
    for op in artifact.items.iter() {
        if op.resource_id != 0 {
            image_ids.insert(op.resource_id);
        }
        if let Some(shader) = &op.paint_shader {
            super::native_metadata::visit_shader_images(shader, &mut |id| {
                image_ids.insert(id);
            });
        }
        for layer in &op.mask_layers {
            if layer.resource_id != 0 {
                image_ids.insert(layer.resource_id);
            }
            if let Some(shader) = &layer.paint_shader {
                super::native_metadata::visit_shader_images(shader, &mut |id| {
                    image_ids.insert(id);
                });
            }
        }
    }
    for resource_id in image_ids {
        let image = artifact
            .resources
            .as_ref()
            .and_then(|resources| {
                resources
                    .images
                    .iter()
                    .find(|image| image.id == resource_id)
            })
            .ok_or(PaintInputError::MissingImageResource { resource_id })?;
        if let paint::paint_engine::PaintImageContent::Document(content) = &image.content {
            let record = image
                .document_container_key
                .as_ref()
                .and_then(|key| content.exact_record_for(key))
                .and_then(|record| {
                    record
                        .as_any()
                        .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
                })
                .ok_or(PaintInputError::ImagePlayback {
                    resource_id,
                    reason: "document image has no published native PaintRecord",
                })?;
            ValidateArtifactGraph(&record.artifact, artifacts)?;
        }
        if image
            .ShouldAnimate()
            .map_err(|blocker| PaintInputError::ImageMetadata {
                resource_id,
                blocker: blocker.clone(),
            })?
        {
            return Err(PaintInputError::ImagePlayback {
                resource_id,
                reason: "animated image lacks its native per-frame timeline owner",
            });
        }
    }
    while let Some(node) = pending.pop() {
        match node {
            Node::Transform(node) => {
                if !seen.insert((0, Arc::as_ptr(&node) as usize)) {
                    continue;
                }
                if node
                    .direct_compositing_reasons
                    .iter()
                    .any(|reason| *reason == "StickyPosition")
                    && node.compositor.sticky_constraint.is_none()
                {
                    return Err(PaintInputError::MissingStickyConstraint { node: node.clone() });
                }
                if let Some(parent) = &node.parent {
                    pending.push(Node::Transform(parent.clone()));
                }
                if let Some(parent) = &node.scroll_parent_scroll_translation {
                    pending.push(Node::Transform(parent.clone()));
                }
                if let Some(scroll) = &node.scroll {
                    if scroll.parent.is_some() && !scroll.compositor.snap_container_data_absent {
                        return Err(PaintInputError::MissingScrollCompositorData {
                            node: scroll.clone(),
                        });
                    }
                    if let Some(clip) = &scroll.overflow_clip {
                        pending.push(Node::Clip(clip.clone()));
                    }
                }
            }
            Node::Clip(node) => {
                if !seen.insert((1, Arc::as_ptr(&node) as usize)) {
                    continue;
                }
                if let Some(reason) = &node.clip_path_error {
                    return Err(PaintInputError::ClipPath {
                        node: node.clone(),
                        reason: reason.clone(),
                    });
                }
                if let Some(parent) = &node.parent {
                    pending.push(Node::Clip(parent.clone()));
                }
                pending.push(Node::Transform(node.local_transform_space.clone()));
                if let Some(effect) = &node.pixel_moving_filter {
                    pending.push(Node::Effect(effect.clone()));
                }
            }
            Node::Effect(node) => {
                if !seen.insert((2, Arc::as_ptr(&node) as usize)) {
                    continue;
                }
                if node.compositor.canvas_child_state.is_some() {
                    return Err(PaintInputError::UnsupportedCanvasChild { node: node.clone() });
                }
                for (backdrop, filters) in [(false, &node.filters), (true, &node.backdrop_filters)]
                {
                    if let Some(blocker) = filters.blocker() {
                        return Err(PaintInputError::EffectFilter {
                            node: node.clone(),
                            backdrop,
                            blocker: blocker.clone(),
                        });
                    }
                }
                if let Some(parent) = &node.parent {
                    pending.push(Node::Effect(parent.clone()));
                }
                pending.push(Node::Transform(node.local_transform_space.clone()));
                if let Some(clip) = &node.output_clip {
                    pending.push(Node::Clip(clip.clone()));
                }
                if let Some(clip) = &node.backdrop_filter_clip {
                    pending.push(Node::Clip(clip.clone()));
                }
            }
        }
    }
    Ok(())
}
