//! Direct production Paint input values used by compositor bindings.
//!
//! The input remains the exact `Arc<PaintArtifact>` supplied by Paint. Chunks,
//! semantic items and replay operations are borrowed from their native owners;
//! record values retain an input Arc and a source range, never copied ops.

use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Range,
    sync::Arc,
};

pub use paint::paint_engine::{
    DisplayItem as PaintOp, PaintArtifact, PaintChunk, RecordedDisplayItem as DisplayItem,
    RecordedDisplayItemKind,
};
use paint::paint_property_tree as native;
mod artifact_nodes;
mod conversion;
mod conversion_nodes;
mod conversion_values;
mod scroll_nodes;
pub use artifact_nodes::{CpuCanvasSnapshot, CpuLayersAsJson};
mod reasons;
pub use conversion_values::CpuConversionRecorder;
pub use reasons::CpuCompositingReasons;
mod geometry_nodes;
mod invalidation;
pub use invalidation::*;
mod geometry_mapper;
pub use geometry_mapper::CpuGeometryMapper;
mod geometry_values;
mod pending_nodes;
mod pending_values;
pub use pending_values::*;
mod picture;
mod property_state;
mod raster;
pub use raster::*;
mod worklet;
pub use worklet::*;
mod image_values;
pub use image_values::CpuPaintImageInputError;
mod lowering;
pub use lowering::{CpuPaintColorFilter, CpuPaintShader, NativePaintSource};
mod native_services;
pub use native_services::{
    CpuNativePaintConfigurationError, CpuNativePaintDiagnostics, CpuNativePaintPlatform,
    CpuNativePaintValueServices, CpuNativeRasterTrackingServices,
};
mod native_discardable;
mod native_metadata;
mod native_solid;
pub use native_solid::{CpuPaintAnalysisTrace, CpuSolidColorBackend};
mod record;
pub use picture::*;
pub use record::CpuPaintImage as PaintImage;
pub use record::*;
mod subset;
mod validation;
pub use paint::paint_property_tree::PaintPropertyChangeType as PaintPropertyChange;
pub use subset::PaintChunkSubset;
pub use validation::{PaintInputError, ValidateArtifact};

/// Retains the actual producer artifact, including resources and recording basis.
#[derive(Clone)]
pub struct PaintInput {
    artifact: Arc<PaintArtifact>,
}

impl PaintInput {
    pub(crate) fn ViewportProperties<R:crate::runtime::types::CpuSkiaBackend>(&self)->crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::ViewportProperties<crate::runtime::types::CpuCompositorTypes<R>>{
        match self.artifact.compositor_viewport.as_ref().expect("validated native viewport").mode{
            paint::paint_engine::PaintCompositorViewportMode::SingleLayoutViewport=>crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::ViewportProperties{overscroll_elasticity_transform:None,page_scale:None,inner_scroll_translation:None,outer_clip:None,outer_scroll_translation:None},
        }
    }
    pub(crate) fn ScrollTranslationNodes<R>(&self) -> impl Iterator<Item = TransformNode<R>> + '_ {
        self.artifact
            .compositor_viewport
            .as_ref()
            .expect("validated native viewport")
            .scroll_translation_nodes
            .iter()
            .map(|node| TransformNode::from_arc(node.clone()))
    }
    pub fn new(artifact: Arc<PaintArtifact>) -> Result<Self, PaintInputError> {
        ValidateArtifact(&artifact)?;
        Ok(Self { artifact })
    }
    pub fn artifact(&self) -> &Arc<PaintArtifact> {
        &self.artifact
    }
    pub fn chunks(&self) -> &[PaintChunk] {
        &self.artifact.chunks
    }
    pub fn display_items(&self) -> &[DisplayItem] {
        &self.artifact.display_items
    }
    pub fn chunk(&self, index: usize) -> Option<&PaintChunk> {
        self.artifact.chunks.get(index)
    }
    pub fn items_for_chunk(&self, index: usize) -> Option<&[DisplayItem]> {
        let chunk = self.chunk(index)?;
        self.artifact
            .display_items
            .get(chunk.begin_index as usize..chunk.end_index as usize)
    }
    pub fn record(&self, index: usize) -> Option<PaintRecord> {
        let item = self.artifact.display_items.get(index)?;
        PaintRecord::from_item(&self.artifact, item)
    }
}

/// An immutable record lease over exactly the producer's resident replay range.
/// Retaining the artifact also retains the corresponding image/font resources;
/// it does not clone PaintArtifact, semantic items, chunks or replay commands.
#[derive(Clone)]
pub struct PaintRecord {
    artifact: Arc<PaintArtifact>,
    range: Range<usize>,
}

impl PaintRecord {
    pub fn from_artifact(artifact: Arc<PaintArtifact>) -> Self {
        let end = artifact.items.len();
        Self {
            artifact,
            range: 0..end,
        }
    }
    pub fn from_item(artifact: &Arc<PaintArtifact>, item: &DisplayItem) -> Option<Self> {
        let range = item.record_begin..item.record_end;
        artifact.items.get(range.clone())?;
        Some(Self {
            artifact: Arc::clone(artifact),
            range,
        })
    }
    pub fn artifact(&self) -> &Arc<PaintArtifact> {
        &self.artifact
    }
    pub fn ops(&self) -> &[PaintOp] {
        &self.artifact.items[self.range.clone()]
    }
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }
    pub fn len(&self) -> usize {
        self.range.len()
    }
}

/// Paint's persistent identity allocation; numeric node ids are diagnostic only.
/// The retained node handles below keep the allocation alive while in use.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PropertyNodeIdentity(usize);

impl PropertyNodeIdentity {
    fn of(lifecycle: &native::PaintPropertyNodeLifecycle) -> Self {
        Self(Arc::as_ptr(&lifecycle.identity) as usize)
    }
}

macro_rules! node_handle {
    ($handle:ident, $node:ident) => {
        /// Retains the actual native node; equality/hash concern persistent
        /// identity, independently of revision and all mutable-value metadata.
        pub struct $handle<R> {
            node: Arc<native::$node>,
            backend: PhantomData<fn() -> R>,
        }
        impl<R> Clone for $handle<R> {
            fn clone(&self) -> Self {
                Self::from_arc(Arc::clone(&self.node))
            }
        }
        impl<R> $handle<R> {
            pub fn from_arc(node: Arc<native::$node>) -> Self {
                Self {
                    node,
                    backend: PhantomData,
                }
            }
            pub fn as_arc(&self) -> &Arc<native::$node> {
                &self.node
            }
            pub fn node(&self) -> &native::$node {
                &self.node
            }
            pub fn identity(&self) -> PropertyNodeIdentity {
                PropertyNodeIdentity::of(&self.node.lifecycle)
            }
            pub fn cc_node_id(&self, sequence_number: i32) -> i32 {
                self.node.lifecycle.cc_node_id(sequence_number)
            }
            pub fn set_cc_node_id(&self, sequence_number: i32, node_id: i32) {
                self.node.lifecycle.set_cc_node_id(sequence_number, node_id);
            }
        }
        impl<R> PartialEq for $handle<R> {
            fn eq(&self, other: &Self) -> bool {
                self.identity() == other.identity()
            }
        }
        impl<R> Eq for $handle<R> {}
        impl<R> Hash for $handle<R> {
            fn hash<H: Hasher>(&self, hasher: &mut H) {
                self.identity().hash(hasher);
            }
        }
        impl<R> std::fmt::Debug for $handle<R> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($handle))
                    .field(&self.identity())
                    .finish()
            }
        }
    };
}
node_handle!(TransformNode, TransformPaintPropertyNode);
node_handle!(ClipNode, ClipPaintPropertyNode);
node_handle!(EffectNode, EffectPaintPropertyNode);
node_handle!(ScrollNode, ScrollPaintPropertyNode);

/// The native three-node state. Clone retains only its three actual Arc nodes.
/// No property node, directory, ancestry or recorded state is materialized.
pub struct PropertyTreeState<R> {
    transform_: TransformNode<R>,
    clip_: ClipNode<R>,
    effect_: EffectNode<R>,
}

impl<R> Clone for PropertyTreeState<R> {
    fn clone(&self) -> Self {
        Self {
            transform_: self.transform_.clone(),
            clip_: self.clip_.clone(),
            effect_: self.effect_.clone(),
        }
    }
}
impl<R> std::fmt::Debug for PropertyTreeState<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyTreeState")
            .field("transform", &self.transform_)
            .field("clip", &self.clip_)
            .field("effect", &self.effect_)
            .finish()
    }
}
impl<R> PartialEq for PropertyTreeState<R> {
    fn eq(&self, other: &Self) -> bool {
        self.transform_ == other.transform_
            && self.clip_ == other.clip_
            && self.effect_ == other.effect_
    }
}
impl<R> Eq for PropertyTreeState<R> {}
impl<R> Default for PropertyTreeState<R> {
    fn default() -> Self {
        Self::from_native(&native::PropertyTreeState::default())
    }
}

impl<R> PropertyTreeState<R> {
    pub fn from_native(state: &native::PropertyTreeState) -> Self {
        Self {
            transform_: TransformNode::from_arc(state.transform.clone()),
            clip_: ClipNode::from_arc(state.clip.clone()),
            effect_: EffectNode::from_arc(state.effect.clone()),
        }
    }
    pub fn native(&self) -> native::PropertyTreeState {
        native::PropertyTreeState {
            transform: self.transform_.node.clone(),
            clip: self.clip_.node.clone(),
            effect: self.effect_.node.clone(),
        }
    }
    pub fn transform(&self) -> TransformNode<R> {
        self.transform_.clone()
    }
    pub fn clip(&self) -> ClipNode<R> {
        self.clip_.clone()
    }
    pub fn effect(&self) -> EffectNode<R> {
        self.effect_.clone()
    }
    pub fn set_transform(&mut self, node: TransformNode<R>) {
        self.transform_ = node;
    }
    pub fn set_clip(&mut self, node: ClipNode<R>) {
        self.clip_ = node;
    }
    pub fn set_effect(&mut self, node: EffectNode<R>) {
        self.effect_ = node;
    }
    /// Chromium: property_tree_state.h:142-146 and the four property-node
    /// ClearChangedToRoot implementations. The sequence marker lives on each
    /// actual persistent Paint node identity, so shared ancestors are visited
    /// once without cloning or rewriting the artifact.
    pub fn clear_changed_to_root(&self, sequence_number: i32) {
        clear_transform_changed(self.transform_.as_arc(), sequence_number);
        clear_clip_changed(self.clip_.as_arc(), sequence_number);
        clear_effect_changed(self.effect_.as_arc(), sequence_number);
    }
}

fn clear_scroll_changed(node: &native::ScrollPaintPropertyNode, sequence_number: i32) {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.lifecycle.changed_sequence_number() == sequence_number {
            break;
        }
        node.lifecycle.clear_changed(sequence_number);
        current = node.parent.as_deref();
    }
}

fn clear_transform_changed(node: &native::TransformPaintPropertyNode, sequence_number: i32) {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.lifecycle.changed_sequence_number() == sequence_number {
            break;
        }
        node.lifecycle.clear_changed(sequence_number);
        if let Some(scroll) = node.scroll.as_deref() {
            clear_scroll_changed(scroll, sequence_number);
        }
        current = node.parent.as_deref();
    }
}

fn clear_clip_changed(node: &native::ClipPaintPropertyNode, sequence_number: i32) {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.lifecycle.changed_sequence_number() == sequence_number {
            break;
        }
        node.lifecycle.clear_changed(sequence_number);
        clear_transform_changed(&node.local_transform_space, sequence_number);
        current = node.parent.as_deref();
    }
}

fn clear_effect_changed(node: &native::EffectPaintPropertyNode, sequence_number: i32) {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.lifecycle.changed_sequence_number() == sequence_number {
            break;
        }
        node.lifecycle.clear_changed(sequence_number);
        clear_transform_changed(&node.local_transform_space, sequence_number);
        if let Some(clip) = node.output_clip.as_deref() {
            clear_clip_changed(clip, sequence_number);
        }
        current = node.parent.as_deref();
    }
}

// This is a direct native value projection, independent of a complete PAC
// family. The mapper implementation remains the Paint owner's responsibility.
impl<M, R> crate::adapter::paint_geometry::PaintChunkView<M> for PaintChunk
where
    M: crate::adapter::paint_geometry::ChunkGeometryMapper<
            PropertyTreeState = PropertyTreeState<R>,
        > + ?Sized,
{
    fn properties_unalias(&self) -> PropertyTreeState<R> {
        PropertyTreeState::from_native(&self.properties)
    }
    fn raster_effect_outset(&self) -> crate::adapter::paint_geometry::RasterEffectOutset {
        use crate::adapter::paint_geometry::RasterEffectOutset as Output;
        match self.raster_effect_outset {
            paint::paint_engine::RasterEffectOutset::kNone => Output::kNone,
            paint::paint_engine::RasterEffectOutset::kHalfPixel => Output::kHalfPixel,
            paint::paint_engine::RasterEffectOutset::kWholePixel => Output::kWholePixel,
        }
    }
}

impl<B> crate::adapter::paint_chunks_to_cc_layer::ConversionPaintArtifact<B> for PaintArtifact
where
    B: crate::adapter::paint_chunks_to_cc_layer::PaintChunksTypes<
        PaintChunk = PaintChunk,
        DisplayItem = DisplayItem,
    >,
{
    fn DisplayItemsForChunk<'a>(
        &'a self,
        chunk: &'a B::PaintChunk,
    ) -> impl ExactSizeIterator<Item = &'a B::DisplayItem> {
        self.display_items[chunk.begin_index as usize..chunk.end_index as usize].iter()
    }
}

impl<B, R> crate::adapter::paint_artifact_compositor::ArtifactPaintChunk<B> for PaintChunk
where
    B: crate::adapter::paint_artifact_compositor::PaintArtifactCompositorTypes<
        PaintChunk = PaintChunk,
        TransformNode = TransformNode<R>,
        ClipNode = ClipNode<R>,
        EffectNode = EffectNode<R>,
    >,
{
    fn CanMatchOldChunk(&self) -> bool {
        self.can_match_old_chunk()
    }
    fn SameRawTransform(&self, other: &PaintChunk) -> bool {
        self.properties
            .transform
            .lifecycle
            .same_node(&other.properties.transform.lifecycle)
    }
    fn UnaliasedTransform(&self) -> TransformNode<R> {
        TransformNode::from_arc(Arc::clone(&self.properties.transform))
    }
    fn UnaliasedClip(&self) -> ClipNode<R> {
        ClipNode::from_arc(Arc::clone(&self.properties.clip))
    }
    fn UnaliasedEffect(&self) -> EffectNode<R> {
        EffectNode::from_arc(Arc::clone(&self.properties.effect))
    }
    fn opaque_rect(&self) -> crate::adapter::geometry::Rect {
        // The native producer's integer-cell opacity proof is retained before
        // converting its scalar ABI to gfx's integer value representation.
        let rect = self.rect_known_to_be_opaque.enclosed();
        crate::adapter::geometry::Rect::new(
            rect.x as i32,
            rect.y as i32,
            rect.width as i32,
            rect.height as i32,
        )
    }
    fn SameRawProperties(&self, other: &PaintChunk) -> bool {
        self.properties.same_nodes(&other.properties)
    }
    fn IdToString(&self) -> String {
        format!("{:?}", self.id)
    }
    fn RawTransformForDebug(&self) -> TransformNode<R> {
        TransformNode::from_arc(Arc::clone(&self.properties.transform))
    }
    fn RawClipForDebug(&self) -> ClipNode<R> {
        ClipNode::from_arc(Arc::clone(&self.properties.clip))
    }
    fn RawEffectForDebug(&self) -> EffectNode<R> {
        EffectNode::from_arc(Arc::clone(&self.properties.effect))
    }
}
