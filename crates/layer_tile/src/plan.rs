use paint::paint_engine::{DisplayItem, PaintArtifact, PaintRect};
use paint::paint_property_tree::PropertyTreeState;
use std::sync::Arc;
use std::time::Instant;

/// Backend-proven replay in the original chunk's property coordinates. Chunk
/// identity remains native while the record is lowered into raster space.
#[derive(Clone)]
pub struct RasterRecordContent {
    pub record_index: usize,
    pub items: Arc<[DisplayItem]>,
    pub visual_rect: PaintRect,
    /// Source alpha proof after PaintChunksToCcLayer coordinate lowering.
    pub rect_known_to_be_opaque: PaintRect,
    pub bounds_are_complete: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerTreeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameConfig {
    /// Visible root property-tree space; never used as a layer's bounds/origin.
    pub viewport: PaintRect,
    pub raster_scale: f64,
    /// Current pending-tree viewport. Tiles in this target are foreground
    /// work required before activation.
    pub activation_scroll: Option<CompositorScrollOffset>,
    /// Explicit compositor sample time. LayerTile combines it with retained
    /// visible-viewport history to compute skewport; it never reads a clock.
    pub frame_time: Option<Instant>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompositorScrollOffset {
    pub scroll_node_id: u64,
    /// Additional CSS-pixel translation relative to the committed property.
    pub translation_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsupportedReason {
    InvalidFrameGeometry,
    MissingSemanticRecords,
    InvalidChunkRange,
    InvalidRecordRange,
    NonTranslationTransform,
    FractionalPixelPlacement,
    UnsupportedClip,
    OpacityEffect,
    BlendEffect,
    FilterEffect,
    MaskEffect,
    ForeignLayer,
    TileBudgetExceeded,
}

#[derive(Clone)]
pub struct FramePlan {
    pub layer_tree_id: LayerTreeId,
    pub frame_id: u64,
    pub config: FrameConfig,
    /// Paint order after proven-disjoint layer reordering. Each layer retains
    /// the source order of its merged chunks/records.
    pub layers: Vec<LayerPlan>,
    /// Diagnostic count only. Raster work is published separately through
    /// `RasterBatch`; a display frame never owns executable backend work.
    pub raster_task_count: usize,
    pub unsupported: Option<UnsupportedReason>,
    pub(crate) source: Arc<PaintArtifact>,
    pub(crate) raster_records: Arc<[RasterRecordContent]>,
    pub(crate) tasks: Vec<RasterTask>,
    pub(crate) retired_tiles: Vec<TileId>,
}

impl std::fmt::Debug for FramePlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FramePlan")
            .field("frame_id", &self.frame_id)
            .field("config", &self.config)
            .field("layers", &self.layers)
            .field("raster_task_count", &self.raster_task_count)
            .field("unsupported", &self.unsupported)
            .field("raster_records", &self.raster_records.len())
            .finish()
    }
}

#[allow(non_snake_case)]
impl FramePlan {
    pub fn GetPaintArtifact(&self) -> &PaintArtifact {
        &self.source
    }
    pub fn GetRasterRecords(&self) -> &[RasterRecordContent] {
        &self.raster_records
    }

    pub fn TileIsReady(&self, tile: &TilePlacement) -> bool {
        tile.ready
    }
}

/// Immutable raster work selected by LayerTile. NOW/activation work is the
/// prefix ending at `required_task_count`; the remaining tasks are speculative
/// SOON work and may finish later.
#[derive(Clone, Debug)]
pub struct RasterBatch {
    pub layer_tree_id: LayerTreeId,
    pub frame_id: u64,
    pub tasks: Arc<[RasterTask]>,
    pub required_task_count: usize,
}

#[derive(Clone, Debug)]
pub struct TileResourceRelease {
    pub layer_tree_id: LayerTreeId,
    pub frame_id: u64,
    pub tile_ids: Arc<[TileId]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RasterCompletion {
    pub layer_tree_id: LayerTreeId,
    pub frame_id: u64,
    pub tile_id: TileId,
    pub generation: u64,
}

/// One atomic pending-tree publication. The display snapshot and executable
/// raster work are separate typed values. Resource retirement is deliberately
/// absent: only successful activation can publish a `TileResourceRelease`.
#[derive(Clone, Debug)]
pub struct PendingTreeUpdate {
    pub frame_plan: FramePlan,
    pub raster_batch: RasterBatch,
}

#[derive(Clone, Debug)]
pub struct LayerPlan {
    pub id: LayerId,
    /// Ordering role, without prescribing a pixel format/background color.
    /// A backend changing its backing policy for this role must reraster;
    /// manager already invalidates tiles when the role changes.
    pub is_first_layer: bool,
    /// Backend backing policy derived from the actual whole-frame effect tree.
    /// Includes content inside an isolated parent of a real DstIn mask child,
    /// even when that mask has no visible tiles. This is not a source node ID.
    pub requires_transparent_backing: bool,
    pub chunk_indices: Vec<usize>,
    /// Raster-upcast transforms/clips, with the original effect Arc and parent
    /// chain intact. A normal-blend opacity effect can span several layers:
    /// compose those layers into one transparent group surface, then apply
    /// that node's opacity once on group exit. Match groups by node lifecycle
    /// identity, not the diagnostic numeric ID or opacity value. Nested parent
    /// effects are applied outside their children. Tiles themselves contain
    /// pre-effect content; their backing must not include a page background
    /// that lies outside the effect group.
    /// An `is_mask` effect is the real Mask/DstIn child of its content effect.
    /// Its normal persistent tiles contain mask source pixels. Composite the
    /// complete mask child into its isolated parent with DstIn once, including
    /// transparent exterior; never apply the mask separately to content tiles.
    pub properties: PropertyTreeState,
    /// Actual chunk union in property-tree coordinates, even if incomplete.
    pub bounds: PaintRect,
    pub bounds_are_complete: bool,
    /// Complete drawing support in retained raster coordinates, including each
    /// record's raster-effect outset. Like cc's recorded bounds, this limits
    /// output quad geometry without changing the resident tile grid or UVs.
    /// None keeps unknown support conservative. The white-backed first layer
    /// also uses None: its initialized pixels extend beyond drawing ink.
    pub raster_content_bounds: Option<PaintRect>,
    /// Maximum fully covered opaque rectangle in retained raster coordinates.
    /// External effects/clips still apply at composition.
    pub rect_known_to_be_opaque: PaintRect,
    /// Resident grid anchor in the layer's raster space, retained per scale
    /// while the native layer and raster transform owner still match. Its
    /// initial value encloses complete bounds on the device pixel grid;
    /// initially unknown ink uses (0,0) in the retained local space.
    /// Logical bounds keep fractional ink coordinates. Tiling (i,j) starts
    /// here, never at the viewport; unknown ink can need negative indices.
    pub raster_origin: (f64, f64),
    /// External compositor placement after eligible scroll nodes snap in
    /// device space. Ordinary CSS translations keep their fractional phase.
    /// Raster origin's pixel enclosure never changes this translation.
    pub root_translation: (f64, f64),
    /// Impl-side scroll-tree override while the Page commit trails input. The
    /// renderer resolves it through the same property tree for content,
    /// descendant clips and effect output clips atomically. It never changes
    /// raster coordinates or tile identity.
    pub compositor_scroll: Option<CompositorScrollOffset>,
    /// Conservative bounding intersection of external clips, for allocation
    /// and culling only. It does not encode rounded coverage. Composition must
    /// apply plain `properties.clip` rects as enclosing compositor scissors,
    /// rounded/path clips with AA coverage, and each
    /// `properties.effect` output-clip chain at its group boundary, following
    /// Chromium's synthetic non-trivial-clip effects/render surfaces.
    /// Those native nodes remain owned through the property's Arcs.
    pub root_clip: Option<PaintRect>,
    pub record_indices: Vec<usize>,
    /// Resident interest-area grid. `tiles` is its currently drawable subset;
    /// impl-side scrolling can promote ready entries without a Page commit.
    pub interest_tiles: Vec<TilePlacement>,
    pub tiles: Vec<TilePlacement>,
}

#[derive(Clone, Debug)]
pub struct TilePlacement {
    pub tile_id: TileId,
    pub generation: u64,
    pub tile_index: (i32, i32),
    /// Zero-based normalized tiling coordinates relative to raster_origin.
    pub tiling_rect: PaintRect,
    /// Actual PaintRecord coordinates, suitable for translation into a tile.
    pub tile_rect: PaintRect,
    pub raster_scale: f64,
    pub pixel_size: (u32, u32),
    /// Readiness is snapshotted when LayerTile publishes or refreshes a plan.
    /// Raster cannot mutate this value through a back-reference.
    pub ready: bool,
}

#[derive(Clone, Debug)]
pub struct RasterTask {
    pub tile_id: TileId,
    /// cc Tile::invalidated_id analogue. A dirty tile is a new object; the
    /// normalized grid position is stable independently of its resource ID.
    pub previous_tile_id: Option<TileId>,
    pub layer_id: LayerId,
    pub generation: u64,
    pub tile_index: (i32, i32),
    pub tiling_rect: PaintRect,
    pub tile_rect: PaintRect,
    pub raster_scale: f64,
    pub pixel_size: (u32, u32),
    /// Pending-tree viewport work. Chromium does not notify ready for
    /// activation until these tiles are ready; generic SOON work remains
    /// asynchronous.
    pub required_for_activation: bool,
    /// Semantic record indices; resolve record_begin/end from the input list.
    /// Inaccurate ink bounds always retain the whole record on every visible tile.
    pub record_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerTileStats {
    pub layer_count: usize,
    pub tiling_count: usize,
    pub tile_count: usize,
    pub ready_tile_count: usize,
}
