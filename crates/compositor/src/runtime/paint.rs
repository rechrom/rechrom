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
use paint::paint_property_tree;
mod artifact_nodes;
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
pub use lowering::{CpuPaintColorFilter, CpuPaintShader, PaintSource};
mod services;
pub use services::{
    PaintConfigurationError, SkiaPaintDiagnostics, SkiaPaintPlatform, SkiaPaintValueServices,
    SkiaRasterTrackingServices,
};
mod discardable_image_map;
mod metadata;
mod solid_color_analyzer;
pub use solid_color_analyzer::{CpuPaintAnalysisTrace, CpuSolidColorBackend};
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
    /// The same immutable replay range of the same artifact allocation.
    pub fn SameSource(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.artifact, &other.artifact) && self.range == other.range
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
    fn of(lifecycle: &paint_property_tree::PaintPropertyNodeLifecycle) -> Self {
        Self(Arc::as_ptr(&lifecycle.identity) as usize)
    }
}

/// The artifact's published property trees a handle resolves its links
/// through (see paint_property_tree::PropertyNodeResolver); None follows the
/// Arc links themselves.
pub type HandleTrees = Option<Arc<paint_property_tree::FrozenPropertyTrees>>;

thread_local! {
    /// The published trees of the artifact this thread is compositing; a
    /// handle made without trees (a display item's or hit-test region's
    /// scroll translation) resolves its links through them. As in Chromium,
    /// an older artifact's nodes then read the current node values.
    static INSTALLED_TREES: std::cell::RefCell<HandleTrees> = const { std::cell::RefCell::new(None) };
}

/// Install the published trees of the artifact being composited on this thread.
pub fn InstallPropertyTrees(trees: HandleTrees) {
    INSTALLED_TREES.with(|installed| *installed.borrow_mut() = trees);
}

fn installed_trees() -> HandleTrees {
    INSTALLED_TREES.with(|installed| installed.borrow().clone())
}

macro_rules! resolve_link {
    ($trees:expr, $kind:ident, $link:expr) => {
        match $trees {
            Some(trees) => {
                paint_property_tree::PropertyNodeResolver::$kind(&**trees, $link).clone()
            }
            None => $link.clone(),
        }
    };
}

macro_rules! node_handle {
    ($handle:ident, $node:ident, $kind:ident) => {
        /// Retains the actual native node; equality/hash concern persistent
        /// identity, independently of revision and all mutable-value metadata.
        pub struct $handle<R> {
            node: Arc<paint_property_tree::$node>,
            trees: HandleTrees,
            backend: PhantomData<fn() -> R>,
        }
        impl<R> Clone for $handle<R> {
            fn clone(&self) -> Self {
                Self::from_arc_in(Arc::clone(&self.node), self.trees.clone())
            }
        }
        impl<R> $handle<R> {
            /// A handle resolving through the installed artifact's trees.
            pub fn from_arc(node: Arc<paint_property_tree::$node>) -> Self {
                Self::from_arc_in(node, installed_trees())
            }
            /// A handle whose links resolve through `trees`.
            pub fn from_arc_in(node: Arc<paint_property_tree::$node>, trees: HandleTrees) -> Self {
                Self {
                    node,
                    trees,
                    backend: PhantomData,
                }
            }
            pub fn trees(&self) -> &HandleTrees {
                &self.trees
            }
            /// This node's current parent.
            pub fn parent_handle(&self) -> Option<Self> {
                self.node.parent.as_ref().map(|parent| self.link_same(parent))
            }
            /// Whether this node is `other` or one of its current ancestors.
            pub fn is_ancestor_of(&self, other: &Self) -> bool {
                let mut current = Some(other.clone());
                while let Some(node) = current {
                    if node == *self {
                        return true;
                    }
                    current = node.parent_handle();
                }
                false
            }
            /// Lowest common ancestor over current parents, by identity.
            pub fn lowest_common_ancestor(&self, other: &Self) -> Self {
                let trees = self.trees.clone().or_else(|| other.trees.clone());
                let a = Self::from_arc_in(self.node.clone(), trees.clone());
                let b = Self::from_arc_in(other.node.clone(), trees);
                let depth = |node: &Self| {
                    let mut depth = 0;
                    let mut current = node.parent_handle();
                    while let Some(node) = current {
                        depth += 1;
                        current = node.parent_handle();
                    }
                    depth
                };
                let (mut a, mut b, mut depth_a, mut depth_b) = {
                    let (da, db) = (depth(&a), depth(&b));
                    (a, b, da, db)
                };
                while depth_a > depth_b {
                    a = a.parent_handle().expect("source common-root tree");
                    depth_a -= 1;
                }
                while depth_b > depth_a {
                    b = b.parent_handle().expect("source common-root tree");
                    depth_b -= 1;
                }
                while a != b {
                    a = a.parent_handle().expect("source nonnull common ancestor");
                    b = b.parent_handle().expect("source nonnull common ancestor");
                }
                a
            }
            /// The current node a same-kind link (parent) of this node names.
            pub fn link_same(&self, link: &Arc<paint_property_tree::$node>) -> Self {
                Self::from_arc_in(resolve_link!(&self.trees, $kind, link), self.trees.clone())
            }
            /// The current node a transform link of this node names.
            pub fn link_transform(
                &self,
                link: &Arc<paint_property_tree::TransformPaintPropertyNode>,
            ) -> TransformNode<R> {
                TransformNode::from_arc_in(resolve_link!(&self.trees, transform, link), self.trees.clone())
            }
            pub fn link_clip(&self, link: &Arc<paint_property_tree::ClipPaintPropertyNode>) -> ClipNode<R> {
                ClipNode::from_arc_in(resolve_link!(&self.trees, clip, link), self.trees.clone())
            }
            pub fn link_effect(
                &self,
                link: &Arc<paint_property_tree::EffectPaintPropertyNode>,
            ) -> EffectNode<R> {
                EffectNode::from_arc_in(resolve_link!(&self.trees, effect, link), self.trees.clone())
            }
            pub fn link_scroll(
                &self,
                link: &Arc<paint_property_tree::ScrollPaintPropertyNode>,
            ) -> ScrollNode<R> {
                ScrollNode::from_arc_in(resolve_link!(&self.trees, scroll, link), self.trees.clone())
            }
            pub fn as_arc(&self) -> &Arc<paint_property_tree::$node> {
                &self.node
            }
            pub fn node(&self) -> &paint_property_tree::$node {
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
node_handle!(TransformNode, TransformPaintPropertyNode, transform);
node_handle!(ClipNode, ClipPaintPropertyNode, clip);
node_handle!(EffectNode, EffectPaintPropertyNode, effect);
node_handle!(ScrollNode, ScrollPaintPropertyNode, scroll);

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
        Self::from_source(&paint_property_tree::PropertyTreeState::default())
    }
}

impl<R> PropertyTreeState<R> {
    pub fn from_source(state: &paint_property_tree::PropertyTreeState) -> Self {
        Self::from_source_in(state, None)
    }
    /// A state whose nodes resolve their links through `trees`.
    pub fn from_source_in(state: &paint_property_tree::PropertyTreeState, trees: HandleTrees) -> Self {
        Self {
            transform_: TransformNode::from_arc_in(state.transform.clone(), trees.clone()),
            clip_: ClipNode::from_arc_in(state.clip.clone(), trees.clone()),
            effect_: EffectNode::from_arc_in(state.effect.clone(), trees),
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
        self.as_source()
            .ClearChangedToRootIn(self.transform_.trees(), sequence_number);
    }
}

fn clear_scroll_changed(node: &paint_property_tree::ScrollPaintPropertyNode, sequence_number: i32) {
    node.ClearChangedToRoot(sequence_number);
}

fn clear_transform_changed(
    node: &paint_property_tree::TransformPaintPropertyNode,
    trees: &HandleTrees,
    sequence_number: i32,
) {
    node.ClearChangedToRootIn(trees, sequence_number);
}

fn clear_clip_changed(node: &paint_property_tree::ClipPaintPropertyNode, sequence_number: i32) {
    node.ClearChangedToRoot(sequence_number);
}

fn clear_effect_changed(node: &paint_property_tree::EffectPaintPropertyNode, sequence_number: i32) {
    node.ClearChangedToRoot(sequence_number);
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
        PropertyTreeState::from_source_in(&self.properties, self.property_trees.0.clone())
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
