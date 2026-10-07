use paint::paint_engine::{DisplayItem, PaintArtifact, PaintRect};
use paint::paint_property_tree::PropertyTreeState;
use std::sync::Arc;

/// Backend-proven replay in the original chunk's property coordinates. This
/// adapts the legacy folded-scroll ABI without rewriting native chunk identity.
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameConfig {
    /// Visible root property-tree space; never used as a layer's bounds/origin.
    pub viewport: PaintRect,
    pub raster_scale: f64,
    /// Current pending-tree viewport. Tiles in this target are foreground
    /// work required before activation.
    pub activation_scroll: Option<CompositorScrollOffset>,
    /// Impl-side root-scroll placement while the Page commit trails the
    /// compositor scroll tree. Applied only below `scroll_node_id`.
    pub prepaint_scroll: Option<CompositorScrollOffset>,
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
    pub frame_id: u64,
    pub config: FrameConfig,
    /// Paint order after proven-disjoint layer reordering. Each layer retains
    /// the source order of its merged chunks/records.
    pub layers: Vec<LayerPlan>,
    /// Dirty or unacknowledged tiles in priority order: every visible NOW
    /// tile first, followed by a bounded set of SOON prepaint tiles. The
    /// latter become resident resources but never appear in LayerPlan::tiles
    /// until they are actually visible.
    pub tasks: Vec<RasterTask>,
    pub retired_tiles: Vec<TileId>,
    pub unsupported: Option<UnsupportedReason>,
    pub(crate) source: Arc<PaintArtifact>,
    pub(crate) raster_records: Arc<[RasterRecordContent]>,
    pub(crate) resource_owner: Option<crate::engine::TileResourceOwner>,
}

impl std::fmt::Debug for FramePlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FramePlan")
            .field("frame_id", &self.frame_id)
            .field("config", &self.config)
            .field("layers", &self.layers)
            .field("tasks", &self.tasks)
            .field("retired_tiles", &self.retired_tiles)
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

    /// Tasks required before this plan can draw or activate. Remaining tasks
    /// are bounded SOON prepaint work and may complete asynchronously.
    pub fn RequiredRasterTaskCount(&self) -> usize {
        self.tasks
            .iter()
            .filter(|task| {
                task.required_for_activation
                    || self
                        .layers
                        .iter()
                        .any(|layer| layer.tiles.iter().any(|tile| tile.tile_id == task.tile_id))
            })
            .count()
    }

    #[doc(hidden)]
    pub fn ResourceOwner(&self) -> Option<crate::engine::TileResourceOwner> {
        self.resource_owner.clone()
    }

    pub fn TileIsReady(&self, tile: &TilePlacement) -> bool {
        self.resource_owner
            .as_ref()
            .is_some_and(|owner| owner.raster_is_ready(tile.tile_id, tile.generation))
    }

    #[doc(hidden)]
    pub fn IsCurrentFrame(&self) -> bool {
        self.resource_owner
            .as_ref()
            .and_then(|owner| owner.0.upgrade())
            .is_some_and(|state| {
                let state = state.lock().expect("layer/tile state poisoned");
                state.valid_plan && state.manager.frame_id() == self.frame_id
            })
    }

    /// Backend-only completion protocol; failed, superseded or retired tile
    /// generations never become drawable. This is independent of composition.
    #[doc(hidden)]
    pub fn DidRasterize(&self) -> bool {
        self.DidRasterizeTasks(&self.tasks)
    }

    /// A backend may finish the foreground queue before speculative SOON work.
    /// Acknowledge exactly the completed generations so pending-tree activation
    /// is independent of background prepaint progress.
    #[doc(hidden)]
    pub fn DidRasterizeTasks(&self, tasks: &[RasterTask]) -> bool {
        let Some(state) = self
            .resource_owner
            .as_ref()
            .and_then(|owner| owner.0.upgrade())
        else {
            return false;
        };
        let mut state = state.lock().expect("layer/tile state poisoned");
        if !state.valid_plan || state.manager.frame_id() != self.frame_id {
            return false;
        }
        tasks
            .iter()
            .all(|task| state.manager.mark_rasterized(task.tile_id, task.generation))
    }

    #[doc(hidden)]
    pub fn DidReleaseTileResources(&self) {
        if let Some(state) = self
            .resource_owner
            .as_ref()
            .and_then(|owner| owner.0.upgrade())
        {
            let mut state = state.lock().expect("layer/tile state poisoned");
            for id in &self.retired_tiles {
                state.retired_tiles.remove(id);
            }
        }
    }
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
