//! Direct production Paint input values used by the main-thread compositing
//! bindings (PaintArtifactCompositor, PaintChunksToCcLayer, raster
//! invalidation). The impl half keeps only its raster/tiling facets.
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
use paint_model::paint_property_tree;
// cc/paint values (PaintRecord, PaintOpBuffer, DisplayItemList facets) are
// commit vocabulary shared with the impl half.
pub use crate::commit::paint::*;
mod artifact_nodes;
mod artifact_record;
mod conversion;
mod conversion_nodes;
mod conversion_values;
mod scroll_nodes;
mod scrollbar;
pub use artifact_nodes::{CpuCanvasSnapshot, CpuLayersAsJson};
pub use scrollbar::{Scrollbar, ScrollbarCanvas};
mod reasons;
pub use conversion_values::CpuConversionRecorder;
pub use reasons::CpuCompositingReasons;
mod geometry_nodes;
mod invalidation;
pub use invalidation::*;
mod geometry_mapper;
pub use geometry_mapper::CpuGeometryMapper;
mod pending_nodes;
mod pending_values;
pub use pending_values::*;
mod property_state;
mod layer_services;
pub use layer_services::*;
mod subset;
mod validation;
pub use paint_model::paint_property_tree::PaintPropertyChangeType as PaintPropertyChange;
pub use subset::PaintChunkSubset;
pub use validation::{PaintInputError, ValidateArtifact};
// Skia CPU paint platform (moved from compositor/src/runtime/paint/services.rs).
mod services;
pub use services::{
    NewCpuMainPaintServices, PaintConfigurationError, SkiaPaintDiagnostics, SkiaPaintPlatform,
    SkiaPaintValueServices, SkiaRasterTrackingServices,
};

/// Retains the actual producer artifact, including resources and recording basis.
#[derive(Clone)]
pub struct PaintInput {
    artifact: Arc<PaintArtifact>,
}

impl PaintInput {
    pub fn ViewportProperties<R, B>(
        &self,
    ) -> crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::ViewportProperties<B>
    where
        B: crate::adapter::pending_layer::PendingLayerTypes<
            TransformNode = TransformNode<R>,
            ClipNode = ClipNode<R>,
        >,
    {
        let viewport = self
            .artifact
            .compositor_viewport
            .as_ref()
            .expect("validated native viewport");
        match viewport.mode {
            paint::paint_engine::PaintCompositorViewportMode::SingleLayoutViewport => {
                let outer = viewport
                    .layout_scroll_translation
                    .as_ref()
                    .map(|node| TransformNode::from_arc(node.clone()));
                crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::ViewportProperties{
                    overscroll_elasticity_transform:Some(TransformNode::from_arc(viewport.overscroll_elasticity_transform.clone())),
                    page_scale:Some(TransformNode::from_arc(viewport.page_scale.clone())),
                    inner_scroll_translation:outer.as_ref().map(|_|TransformNode::from_arc(viewport.visual_scroll_translation.clone())),
                    outer_clip:outer.as_ref().map(|_|ClipNode::from_arc(viewport.viewport_clip.clone())),
                    outer_scroll_translation:outer,
                }
            }
        }
    }
    pub fn ScrollTranslationNodes<R>(&self) -> impl Iterator<Item = TransformNode<R>> + '_ {
        self.artifact
            .compositor_viewport
            .as_ref()
            .expect("validated native viewport")
            .scroll_translation_nodes
            .iter()
            .map(|node| TransformNode::from_arc(node.clone()))
    }
    /// LocalFrameView::PushPaintArtifactToCompositor hands the artifact to
    /// PaintArtifactCompositor::Update directly (local_frame_view.cc:3348);
    /// Chromium checks the producer's invariants only with DCHECKs. The
    /// whole-graph walk below is that DCHECK: it runs in debug builds or when
    /// RECHROM_VALIDATE_PAINT_ARTIFACT is set (read once), not per frame in
    /// production.
    pub fn new(artifact: Arc<PaintArtifact>) -> Result<Self, PaintInputError> {
        static VALIDATE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if cfg!(debug_assertions)
            || *VALIDATE
                .get_or_init(|| std::env::var_os("RECHROM_VALIDATE_PAINT_ARTIFACT").is_some())
        {
            ValidateArtifact(&artifact)?;
        }
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
        paint::paint_engine::PaintRecordForItem(&self.artifact, item)
    }
}

/// A live Paint node's address (Chromium compares node pointers); numeric
/// node ids are diagnostic only. The handles below keep the node alive while
/// its identity is in use.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PropertyNodeIdentity(usize);


/// One node read under the document sequence's proof (decision D1: layerize
/// handles take the proof per call; LayerizeEngine runs on the document
/// sequence that owns the nodes).
#[inline]
pub(crate) fn read<T>(f: impl FnOnce(&paint_model::property_cell::PropertyAccess) -> T) -> T {
    paint_model::property_cell::with_access(f)
}


macro_rules! node_handle {
    ($handle:ident, $node:ident) => {
        /// Retains the actual live node (Chromium's `const Node*`); its links
        /// are read through the node itself. Equality and hash are the node's
        /// identity.
        pub struct $handle<R> {
            node: Arc<paint_property_tree::$node>,
            backend: PhantomData<fn() -> R>,
        }
        impl<R> Clone for $handle<R> {
            fn clone(&self) -> Self {
                Self::from_arc(Arc::clone(&self.node))
            }
        }
        impl<R> $handle<R> {
            pub fn from_arc(node: Arc<paint_property_tree::$node>) -> Self {
                Self {
                    node,
                    backend: PhantomData,
                }
            }
            /// This node's current parent.
            pub fn parent_handle(&self) -> Option<Self> {
                read(|a| self.node.Parent(a).map(|parent| Self::from_arc(parent.clone())))
            }
            /// Whether this node is `other` or one of its current ancestors.
            pub fn is_ancestor_of(&self, other: &Self) -> bool {
                // Source: paint_property_node.h:159-168 walks `const Node*`
                // parents. One proof covers the walk; parents are borrowed,
                // never retained per step.
                let target = Arc::as_ptr(&self.node);
                read(|a| {
                    let mut current: Option<&paint_property_tree::$node> = Some(&*other.node);
                    while let Some(node) = current {
                        if std::ptr::eq(node, target) {
                            return true;
                        }
                        current = node.Parent(a).map(|parent| &**parent);
                    }
                    false
                })
            }
            /// Lowest common ancestor over current parents, by identity.
            /// Source: paint_property_node.cc:12-60 (LowestCommonAncestorInternal
            /// compares depths, then walks both sides up by `const Node*`).
            pub fn lowest_common_ancestor(&self, other: &Self) -> Self {
                if Arc::ptr_eq(&self.node, &other.node) {
                    return self.clone();
                }
                read(|access| {
                    let depth = |node: &Arc<paint_property_tree::$node>| {
                        let mut depth = 0usize;
                        let mut current = node.Parent(access);
                        while let Some(node) = current {
                            depth += 1;
                            current = node.Parent(access);
                        }
                        depth
                    };
                    let mut a: &Arc<paint_property_tree::$node> = &self.node;
                    let mut b: &Arc<paint_property_tree::$node> = &other.node;
                    let (mut depth_a, mut depth_b) = (depth(a), depth(b));
                    while depth_a > depth_b {
                        a = a.Parent(access).expect("source common-root tree");
                        depth_a -= 1;
                    }
                    while depth_b > depth_a {
                        b = b.Parent(access).expect("source common-root tree");
                        depth_b -= 1;
                    }
                    while !Arc::ptr_eq(a, b) {
                        a = a.Parent(access).expect("source nonnull common ancestor");
                        b = b.Parent(access).expect("source nonnull common ancestor");
                    }
                    Self::from_arc(Arc::clone(a))
                })
            }
            pub fn as_arc(&self) -> &Arc<paint_property_tree::$node> {
                &self.node
            }
            pub fn node(&self) -> &paint_property_tree::$node {
                &self.node
            }
            pub fn identity(&self) -> PropertyNodeIdentity {
                PropertyNodeIdentity(Arc::as_ptr(&self.node) as *const u8 as usize)
            }
            pub fn cc_node_id(&self, sequence_number: i32) -> i32 {
                paint_model::property_cell::with_access(|a| self.node.lifecycle.cc_node_id(a, sequence_number))
            }
            pub fn set_cc_node_id(&self, sequence_number: i32, node_id: i32) {
                paint_model::property_cell::with_access(|a| self.node.lifecycle.set_cc_node_id(a, sequence_number, node_id));
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
        // PropertyTreeState::Root() (property_tree_state.h:275-280) is a
        // `const&` to the sequence's sentinels: retain each root node once.
        paint_property_tree::PropertyTreeState::WithRoot(Self::from_source)
    }
}

impl<R> PropertyTreeState<R> {
    /// The three-node state of `transform`, `clip` and `effect`
    /// (property_tree_state.h:91-96), without passing through the root.
    pub fn new(transform: TransformNode<R>, clip: ClipNode<R>, effect: EffectNode<R>) -> Self {
        Self {
            transform_: transform,
            clip_: clip,
            effect_: effect,
        }
    }
    pub fn from_source(state: &paint_property_tree::PropertyTreeState) -> Self {
        Self {
            transform_: TransformNode::from_arc(state.transform.clone()),
            clip_: ClipNode::from_arc(state.clip.clone()),
            effect_: EffectNode::from_arc(state.effect.clone()),
        }
    }
    pub fn as_source(&self) -> paint_property_tree::PropertyTreeState {
        paint_property_tree::PropertyTreeState {
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
        // PropertyTreeState::ClearChangedToRoot order (transform, clip,
        // effect) on the live nodes, without assembling a source state.
        paint_model::property_cell::with_access(|a| {
            self.transform_.node.ClearChangedToRoot(a, sequence_number);
            self.clip_.node.ClearChangedToRoot(a, sequence_number);
            self.effect_.node.ClearChangedToRoot(a, sequence_number);
        });
    }
}

fn clear_scroll_changed(node: &paint_property_tree::ScrollPaintPropertyNode, sequence_number: i32) {
    paint_model::property_cell::with_access(|a| node.ClearChangedToRoot(a, sequence_number));
}

fn clear_transform_changed(
    node: &paint_property_tree::TransformPaintPropertyNode,
    sequence_number: i32,
) {
    paint_model::property_cell::with_access(|a| node.ClearChangedToRoot(a, sequence_number));
}

fn clear_clip_changed(node: &paint_property_tree::ClipPaintPropertyNode, sequence_number: i32) {
    paint_model::property_cell::with_access(|a| node.ClearChangedToRoot(a, sequence_number));
}

fn clear_effect_changed(node: &paint_property_tree::EffectPaintPropertyNode, sequence_number: i32) {
    paint_model::property_cell::with_access(|a| node.ClearChangedToRoot(a, sequence_number));
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
        PropertyTreeState::from_source(&self.properties)
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
        paint::paint_engine::PaintChunk::CanMatchOldChunk(self)
    }
    fn SameRawTransform(&self, other: &PaintChunk) -> bool {
        Arc::ptr_eq(&self.properties.transform, &other.properties.transform)
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
