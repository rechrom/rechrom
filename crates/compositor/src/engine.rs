//! Retained compositor state, raster coordination, and CompositorFrame production.
//!
//! The engine is independent of its thread, native window, display service, and
//! application metadata. The embedding supplies concrete frame/viewport types
//! and routes typed effects to raster ownership and Viz.

use crate::{
    scroll_tree::{CompositorScrollTree, ScrollNode, ScrollTreeSnapshot},
    BeginFrameAck, CompositorFrame, DeviceRect, FrameBuilder, SolidColorDrawQuad,
};
use foundation::begin_frame::BeginFrameArgs;
use interaction::input_event::{ScrollGranularity, WheelEvent};
use layer_tile::{
    CompositorScrollOffset, FrameConfig, FramePlan, LayerTileEngine, UnsupportedReason,
};
use layoutng_assembly::internal::layout_input::Offset;
use paint::{
    paint_engine::{PaintArtifact, PaintRect, RecordedDisplayItemKind},
    paint_property_tree::TransformPaintPropertyNode,
};
use std::{io, sync::Arc, time::Instant};

fn memory_diagnostics_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RECHROM_MEMORY_DIAGNOSTICS").is_some())
}
fn report_pipeline<V, M>(phase: &str, snapshot: &ArtifactSnapshot<V, M>) {
    if memory_diagnostics_enabled() {
        eprintln!("pipeline-memory time_ns={} phase={} tab={} generation={} sequence={} revision={} ack={} artifact_owners={} resource_owners={}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()), phase,
            snapshot.document.0, snapshot.document.1, snapshot.sequence,
            snapshot.content.as_ref().map_or(0, |a| a.recording_revision), snapshot.compositor_scroll_sequence,
            snapshot.content.as_ref().map_or(0, Arc::strong_count),
            snapshot.content.as_ref().and_then(|a| a.resources.as_ref()).map_or(0, Arc::strong_count));
    }
}
pub trait CompositorBeginFrame: Copy {
    fn BeginFrameArgs(&self) -> &BeginFrameArgs;
}

impl CompositorBeginFrame for BeginFrameArgs {
    fn BeginFrameArgs(&self) -> &BeginFrameArgs {
        self
    }
}

pub trait CompositorViewport: Copy + PartialEq {
    fn Width(self) -> u32;
    fn Height(self) -> u32;
    fn Scale(self) -> f64;

    /// Platform policy selected by the application host. LayerTile consumes
    /// the value as pure configuration and never queries an OS or display.
    fn RasterTileSize(self) -> u32 {
        256
    }

    fn LogicalWidth(self) -> f64 {
        f64::from(self.Width()) / self.Scale()
    }

    fn LogicalHeight(self) -> f64 {
        f64::from(self.Height()) / self.Scale()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockingWheelEventRegions {
    pub covers_viewport: bool,
    /// CSS client-space regions attached to the root scroll node.
    pub scrolling_rects: Vec<PaintRect>,
    /// CSS client-space regions attached to the viewport.
    pub fixed_rects: Vec<PaintRect>,
}

impl BlockingWheelEventRegions {
    pub fn Contains(&self, x: f64, y: f64, root_scroll_delta: Offset) -> bool {
        let contains = |rect: &PaintRect, x: f64, y: f64| {
            x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
        };
        self.covers_viewport
            || self
                .scrolling_rects
                .iter()
                .any(|rect| contains(rect, x + root_scroll_delta.x, y + root_scroll_delta.y))
            || self.fixed_rects.iter().any(|rect| contains(rect, x, y))
    }
}

#[derive(Clone)]
pub enum ContentUpdate {
    /// No content commit. Existing content belongs to the compositor owner.
    Unchanged,
    Replace(Arc<PaintArtifact>),
    Clear,
}
impl ContentUpdate {
    pub fn as_ref(&self) -> Option<&Arc<PaintArtifact>> {
        match self {
            Self::Replace(artifact) => Some(artifact),
            _ => None,
        }
    }
}
impl From<Option<Arc<PaintArtifact>>> for ContentUpdate {
    fn from(content: Option<Arc<PaintArtifact>>) -> Self {
        content.map_or(Self::Clear, Self::Replace)
    }
}

#[derive(Clone)]
pub struct ArtifactSnapshot<V, M> {
    pub document: (u64, u64),
    pub viewport: V,
    pub frame_time: Instant,
    pub sequence: u64,
    /// Compositor delta batch reflected by this exact content commit.
    pub compositor_scroll_sequence: u64,
    pub content: ContentUpdate,
    pub transparent_background: bool,
    pub metadata: M,
    pub async_root_scroll: bool,
    pub blocking_wheel_regions: BlockingWheelEventRegions,
}

/// Identity of the document pixels actually selected for a submitted frame.
/// This follows the activated snapshot, which may precede a newer navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameProvenance {
    pub document: (u64, u64),
    /// A content surface can be empty or background-only; presence does not
    /// imply a contentful paint or physical presentation.
    pub content: Option<ContentFrameProvenance>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentFrameProvenance {
    pub sequence: u64,
    pub recording_revision: u64,
}

impl<V, M> ArtifactSnapshot<V, M> {
    fn Provenance(&self) -> FrameProvenance {
        FrameProvenance {
            document: self.document,
            content: self
                .content
                .as_ref()
                .map(|artifact| ContentFrameProvenance {
                    sequence: self.sequence,
                    recording_revision: artifact.recording_revision,
                }),
        }
    }
}

impl<V: PartialEq, M> ArtifactSnapshot<V, M> {
    fn SameRasterInputs(&self, other: &Self) -> bool {
        self.document == other.document
            && self.sequence == other.sequence
            && self.viewport == other.viewport
            && self.transparent_background == other.transparent_background
            && self.compositor_scroll_sequence == other.compositor_scroll_sequence
            && match (self.content.as_ref(), other.content.as_ref()) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

pub enum Command<V, F, M> {
    Snapshot(ArtifactSnapshot<V, M>),
    InputBeginFrame(F),
    BeginFrame(F),
    DisplayBeginFrame(F),
    MainFrameComplete {
        source_id: u64,
        sequence_number: u64,
    },
    Wheel {
        event: WheelEvent,
        queued_at: Instant,
        samples: usize,
    },
    Resize(V),
    Redraw,
    RasterReady(RasterReady<V, M>),
    SpareBundle(RasterBundle),
    ActiveRasterReady(Vec<layer_tile::RasterCompletion>),
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MainThreadScrollDelta {
    pub sequence: u64,
    pub owner_client_id: u64,
    pub delta: Offset,
}

pub enum MainThreadEvent<F> {
    BeginFrame {
        frame: F,
        document: Option<(u64, u64)>,
        scroll: Option<MainThreadScrollDelta>,
    },
    Wheel {
        event: WheelEvent,
        queued_at: Instant,
        samples: usize,
    },
    /// Retained raster input could not be lowered. The Page must publish one
    /// fresh paint artifact while the compositor continues presenting its
    /// active tree.
    RasterFallback,
}

pub enum Effect<V, F, M> {
    Raster(RasterJob<V, M>),
    InstallRasterBundle(RasterBundle),
    FrameProduced {
        frame: F,
        submitted: SubmittedFrame<V, M>,
        scroll_active: bool,
    },
    Submit {
        frame: F,
        scroll_active: bool,
    },
    MainThread(MainThreadEvent<F>),
    RequestBeginFrame,
    /// Active drawable content changed. Unlike generic frame demand, this may
    /// reopen a clean compositor preparation before the current draw deadline.
    VisualDamage,
}

#[derive(Clone)]
pub struct PlannedFrame<V, M> {
    pub snapshot: ArtifactSnapshot<V, M>,
    pub content: Option<FramePlan>,
    scroll_tree: ScrollTreeSnapshot,
    visual_scroll: Option<(u64, Offset)>,
    raster_scroll: Option<(u64, Offset)>,
    root_scrollbars: Vec<layer_tile::SolidScrollbarLayer>,
}

pub struct SubmittedFrame<V, M> {
    pub provenance: FrameProvenance,
    pub viewport: V,
    pub content: Option<CompositorFrame>,
    pub metadata: M,
}

pub struct RasterBundle {
    tile_size: u32,
    pub tiles: LayerTileEngine,
    pub raster: raster::RasterEngine,
}

struct CommittedContent {
    sequence: u64,
    scroll_sequence: u64,
    artifact: Arc<PaintArtifact>,
}

impl Default for RasterBundle {
    fn default() -> Self {
        Self::new(256)
    }
}

impl RasterBundle {
    fn new(tile_size: u32) -> Self {
        Self {
            tile_size,
            tiles: LayerTileEngine::new(tile_size),
            raster: raster::RasterEngine::default(),
        }
    }
}

pub struct RasterJob<V, M> {
    pub snapshot: ArtifactSnapshot<V, M>,
    pub bundle: RasterBundle,
    pub activation_target: Option<(u64, Offset)>,
}

pub struct RasterReady<V, M> {
    pub result: io::Result<PlannedFrame<V, M>>,
    pub bundle: RasterBundle,
}

impl<V: CompositorViewport, M> PlannedFrame<V, M> {
    fn RootScrollbarQuads(&self) -> Option<(Vec<SolidColorDrawQuad>, DeviceRect)> {
        if self.root_scrollbars.is_empty() {
            return None;
        }
        let root = self.scroll_tree.root()?;
        let current = self
            .visual_scroll
            .filter(|(id, _)| *id == root.id)
            .map_or(root.committed, |(_, offset)| offset);
        let zoom = self.content.as_ref()?.config.layout_zoom_factor;
        let mut quads = Vec::new();
        let mut bounds = PaintRect::default();
        for layer in &self.root_scrollbars {
            bounds.union(layer.track);
            let (position, minimum, maximum) = if layer.horizontal {
                (current.x, root.minimum.x, root.maximum.x)
            } else {
                (current.y, root.minimum.y, root.maximum.y)
            };
            let ratio = if maximum > minimum {
                ((position - minimum) / (maximum - minimum)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            for part in &layer.parts {
                let mut rect = part.rect;
                if part.is_thumb {
                    // ScrollbarLayerImplBase::ComputeThumbQuadRect consumes
                    // current impl position, track insets and retained thumb
                    // dimensions independently of main-thread paint.
                    if layer.horizontal {
                        rect.width = (rect.width / zoom).round() * zoom;
                        let travel = ((layer.track.width
                            - layer.start_inset
                            - layer.end_inset
                            - rect.width)
                            / zoom)
                            .max(0.0);
                        rect.x = layer.track.x
                            + layer.start_inset
                            + ((travel as f32 * ratio as f32) as i32 as f64) * zoom;
                    } else {
                        rect.height = (rect.height / zoom).round() * zoom;
                        let travel = ((layer.track.height
                            - layer.start_inset
                            - layer.end_inset
                            - rect.height)
                            / zoom)
                            .max(0.0);
                        rect.y = layer.track.y
                            + layer.start_inset
                            + ((travel as f32 * ratio as f32) as i32 as f64) * zoom;
                    }
                }
                let rect = PaintRect {
                    x: rect.x / zoom,
                    y: rect.y / zoom,
                    width: rect.width / zoom,
                    height: rect.height / zoom,
                };
                quads.push(SolidColorDrawQuad {
                    rect,
                    visible_rect: rect,
                    color: part.color,
                    corner_radius: 0.0,
                });
            }
        }
        let scale = self.snapshot.viewport.Scale() / zoom;
        let x = (bounds.x * scale).floor();
        let y = (bounds.y * scale).floor();
        Some((
            quads,
            DeviceRect::new(
                x as i32,
                y as i32,
                (((bounds.x + bounds.width) * scale).ceil() - x).max(0.0) as u32,
                (((bounds.y + bounds.height) * scale).ceil() - y).max(0.0) as u32,
            ),
        ))
    }
    fn RootOverlayScrollbarQuad(&self) -> Option<(SolidColorDrawQuad, DeviceRect)> {
        let root = self.scroll_tree.root()?;
        // Paint already records the platform/theme scrollbar in border-box
        // space and associates it with the owning scroll translation. Do not
        // cover that output with the compositor's generic black fallback
        // during an active gesture. The fallback remains for embeddings which
        // deliberately publish no painted root scrollbar.
        if self.snapshot.content.as_ref().is_some_and(|artifact| {
            artifact.display_items.iter().any(|record| {
                record.kind == RecordedDisplayItemKind::Scrollbar
                    && record
                        .scroll_translation
                        .as_ref()
                        .and_then(|transform| transform.scroll.as_ref())
                        .is_some_and(|scroll| scroll.id == root.id)
            })
        }) {
            return None;
        }
        let range = root.maximum.y - root.minimum.y;
        if range <= 0.0 {
            return None;
        }
        let offset = self
            .visual_scroll
            .filter(|(id, _)| *id == root.id)
            .map_or(root.committed.y, |(_, value)| value.y)
            .clamp(root.minimum.y, root.maximum.y);
        let viewport = self.snapshot.viewport.LogicalHeight().max(1.0);
        let layout_zoom = self
            .content
            .as_ref()
            .map_or(1.0, |content| content.config.layout_zoom_factor)
            .max(f64::MIN_POSITIVE);
        let contents = viewport + range / layout_zoom;
        let inner_length = (viewport - 4.0).max(0.0);
        let thumb_length = (inner_length * viewport / contents)
            .max(24.0)
            .min(inner_length);
        if thumb_length <= 0.0 {
            return None;
        }
        let travel = (inner_length - thumb_length).max(0.0);
        let top = 2.0 + travel * ((offset - root.minimum.y) / range).clamp(0.0, 1.0);
        let right = self.snapshot.viewport.LogicalWidth() - 3.0;
        let rect = PaintRect {
            x: (right - 10.0).max(0.0),
            y: top,
            width: 10.0,
            height: thumb_length,
        };
        let scale = self.snapshot.viewport.Scale();
        let strip_width = (14.0 * scale).ceil() as u32;
        let width = self.snapshot.viewport.Width();
        let height = self.snapshot.viewport.Height();
        Some((
            SolidColorDrawQuad {
                rect,
                visible_rect: rect,
                color: [0, 0, 0, 128],
                corner_radius: 5.0,
            },
            DeviceRect::new(
                width.saturating_sub(strip_width) as i32,
                0,
                strip_width.min(width),
                height,
            ),
        ))
    }
}

pub struct CompositorEngine<V, F, M> {
    content_frame_builder: FrameBuilder,
    spare_bundle: Option<RasterBundle>,
    active_bundle_installed: bool,
    raster_in_flight: bool,
    raster_in_flight_main_commit: bool,
    raster_fallback_requested: bool,
    main_frame_in_flight: Option<(u64, u64)>,
    pending_snapshot: Option<ArtifactSnapshot<V, M>>,
    pending_activation_target: Option<(u64, Offset)>,
    snapshot: Option<ArtifactSnapshot<V, M>>,
    planned: Option<PlannedFrame<V, M>>,
    scroll_tree: CompositorScrollTree,
    // A compositor instance owns one Page; other Page state never enters it.
    committed_content: Option<CommittedContent>,
    overlay_scrollbar_damage: Option<DeviceRect>,
    current_begin_frame: Option<F>,
    dirty: bool,
}

impl<V, F, M> Default for CompositorEngine<V, F, M> {
    fn default() -> Self {
        Self {
            content_frame_builder: FrameBuilder::default(),
            spare_bundle: Some(RasterBundle::default()),
            active_bundle_installed: false,
            raster_in_flight: false,
            raster_in_flight_main_commit: false,
            raster_fallback_requested: false,
            main_frame_in_flight: None,
            pending_snapshot: None,
            pending_activation_target: None,
            snapshot: None,
            planned: None,
            scroll_tree: CompositorScrollTree::default(),
            committed_content: Default::default(),
            overlay_scrollbar_damage: None,
            current_begin_frame: None,
            dirty: true,
        }
    }
}

impl<V, F, M> CompositorEngine<V, F, M>
where
    V: CompositorViewport,
    F: CompositorBeginFrame,
    M: Clone,
{
    pub fn Handle(&mut self, command: Command<V, F, M>) -> io::Result<Vec<Effect<V, F, M>>> {
        let mut effects = Vec::new();
        match command {
            Command::Snapshot(mut snapshot) => {
                if self
                    .snapshot
                    .as_ref()
                    .is_some_and(|old| old.document != snapshot.document)
                {
                    return Err(io::Error::other(
                        "a Page compositor cannot accept another Page's document",
                    ));
                }
                match &snapshot.content {
                    ContentUpdate::Unchanged => {
                        snapshot.content = if let Some(content) = &self.committed_content {
                            snapshot.sequence = content.sequence;
                            snapshot.compositor_scroll_sequence = content.scroll_sequence;
                            ContentUpdate::Replace(content.artifact.clone())
                        } else {
                            ContentUpdate::Clear
                        };
                    }
                    ContentUpdate::Replace(artifact) => {
                        self.committed_content = Some(CommittedContent {
                            sequence: snapshot.sequence,
                            scroll_sequence: snapshot.compositor_scroll_sequence,
                            artifact: artifact.clone(),
                        });
                    }
                    ContentUpdate::Clear => {
                        self.committed_content = None;
                    }
                }
                report_pipeline("snapshot", &snapshot);
                let raster_changed = self
                    .snapshot
                    .as_ref()
                    .is_none_or(|old| !old.SameRasterInputs(&snapshot))
                    || (!self.snapshot_matches_active(&snapshot)
                        && !self.raster_in_flight
                        && self.pending_snapshot.is_none());
                // SyncedScrollOffset preserves deltas newer than this frame's
                // acknowledgement. A commit need not catch every in-flight
                // input packet before it can provide new paint/tile coverage.
                // Input regions are compositor metadata, not raster content.
                // Refresh them even when the PaintArtifact identity is stable.
                if let Some(planned) = &mut self.planned {
                    planned.snapshot.async_root_scroll = snapshot.async_root_scroll;
                    planned.snapshot.blocking_wheel_regions =
                        snapshot.blocking_wheel_regions.clone();
                }
                self.snapshot = Some(snapshot.clone());
                if raster_changed {
                    self.pending_activation_target = snapshot
                        .content
                        .as_ref()
                        .and_then(|artifact| build_artifact_scroll_tree(artifact).root())
                        .map(|root| {
                            (
                                root.id,
                                self.scroll_tree
                                    .activation_position(root, snapshot.compositor_scroll_sequence),
                            )
                        });
                    self.pending_snapshot = Some(snapshot);
                    self.ScheduleRaster(&mut effects);
                }
            }
            Command::Wheel {
                mut event,
                queued_at,
                samples,
            } => {
                let _input = browser_tracing::scope(browser_tracing::Context {
                    target_id: 1,
                    input_id: browser_tracing::instant_id(queued_at),
                    ..Default::default()
                });
                if event.delta_units != ScrollGranularity::kScrollByPrecisePixel {
                    event.compositor_handled = false;
                    effects.push(Effect::MainThread(MainThreadEvent::Wheel {
                        event,
                        queued_at,
                        samples,
                    }));
                    return Ok(effects);
                }
                let started = Instant::now();
                browser_tracing::interval("input", "CompositorInputQueue", queued_at, started, &[]);
                let current_document = self.snapshot.as_ref().map(|snapshot| snapshot.document);
                let compositor_ready = self
                    .planned
                    .as_ref()
                    .is_some_and(|planned| Some(planned.snapshot.document) == current_document);
                let layout_zoom = self
                    .planned
                    .as_ref()
                    .and_then(|planned| planned.content.as_ref())
                    .map_or(1.0, |content| content.config.layout_zoom_factor)
                    .max(f64::MIN_POSITIVE);
                let blocks_main = !compositor_ready
                    || self.planned.as_ref().is_some_and(|planned| {
                        planned.snapshot.async_root_scroll
                            && planned.snapshot.blocking_wheel_regions.Contains(
                                event.position.x,
                                event.position.y,
                                Offset {
                                    x: self.scroll_tree.pending_delta().x / layout_zoom,
                                    y: self.scroll_tree.pending_delta().y / layout_zoom,
                                },
                            )
                    });
                let root_input_allowed = self.planned.as_ref().is_some_and(|planned| {
                    planned.snapshot.async_root_scroll
                        && planned.scroll_tree.supports_root_input_at(
                            Offset {
                                x: event.position.x * layout_zoom,
                                y: event.position.y * layout_zoom,
                            },
                            self.scroll_tree.pending_delta(),
                        )
                });
                let overlay_before = self.visible_overlay_rect();
                let mut drawable_changed = false;
                let constrained = event.DefaultScrollDelta();
                let update = self.scroll_tree.apply_wheel(
                    event.phase,
                    Offset {
                        x: constrained.x * layout_zoom,
                        y: constrained.y * layout_zoom,
                    },
                    root_input_allowed,
                    blocks_main,
                );
                if update.did_scroll {
                    let desired = update.desired.expect("scrolled wheel has desired offset");
                    let root = self
                        .scroll_tree
                        .root()
                        .expect("scrolled wheel has root node");
                    if let Some(planned) = &mut self.planned {
                        let changed = planned.visual_scroll != Some((root.id, desired));
                        let has_scroll_content = planned.content.as_ref().is_some_and(|content| {
                            content.layers.iter().any(|layer| {
                                !layer.tiles.is_empty()
                                    && layer_has_scroll(&layer.properties.transform, root.id)
                            })
                        });
                        let applied = apply_scroll_offset(
                            planned,
                            root,
                            desired,
                            ScrollTileSelection::ActiveTree,
                        );
                        let has_new_scroll_content =
                            planned.content.as_ref().is_some_and(|content| {
                                content.layers.iter().any(|layer| {
                                    !layer.tiles.is_empty()
                                        && layer_has_scroll(&layer.properties.transform, root.id)
                                })
                            });
                        drawable_changed =
                            applied && changed && (has_scroll_content || has_new_scroll_content);
                    }
                    self.dirty = true;
                    self.pending_snapshot = self.snapshot.clone().map(|mut snapshot| {
                        snapshot.frame_time = queued_at;
                        snapshot
                    });
                    self.pending_activation_target = Some((root.id, desired));
                    self.ScheduleRaster(&mut effects);
                } else if update.active_changed {
                    // Removing the compositor-owned overlay is itself a visual
                    // change even when the final sample has no displacement.
                    self.dirty = true;
                }
                browser_tracing::instant(
                    "input",
                    "WheelCompositorDisposition",
                    &[
                        ("blocked_on_main", update.blocked_on_main as u8 as f64),
                        ("root_input_allowed", root_input_allowed as u8 as f64),
                        ("compositor_ready", compositor_ready as u8 as f64),
                        ("listener_blocks_main", blocks_main as u8 as f64),
                        (
                            "region_rects",
                            self.planned.as_ref().map_or(0.0, |planned| {
                                planned
                                    .snapshot
                                    .blocking_wheel_regions
                                    .scrolling_rects
                                    .len() as f64
                                    + planned.snapshot.blocking_wheel_regions.fixed_rects.len()
                                        as f64
                            }),
                        ),
                    ],
                );
                event.compositor_handled = !update.blocked_on_main;
                effects.push(Effect::MainThread(MainThreadEvent::Wheel {
                    event,
                    queued_at,
                    samples,
                }));
                if drawable_changed || overlay_before != self.visible_overlay_rect() {
                    effects.push(Effect::VisualDamage);
                }
                if update.did_scroll || update.active_changed {
                    effects.push(Effect::RequestBeginFrame);
                }
            }
            Command::Resize(viewport) => {
                if let Some(mut snapshot) = self.snapshot.clone() {
                    snapshot.viewport = viewport;
                    self.snapshot = Some(snapshot.clone());
                    self.pending_snapshot = Some(snapshot);
                    self.ScheduleRaster(&mut effects);
                }
            }
            Command::Redraw => {
                self.dirty = true;
                if self.planned.is_some() {
                    effects.push(Effect::VisualDamage);
                }
                effects.push(Effect::RequestBeginFrame);
            }
            Command::RasterReady(ready) => self.InstallRaster(ready, &mut effects)?,
            Command::ActiveRasterReady(completions) => {
                if let Some(planned) = &mut self.planned {
                    let changed = planned
                        .content
                        .as_mut()
                        .map_or(0, |plan| plan.ApplyRasterResults(&completions));
                    browser_tracing::instant(
                        "raster",
                        "ActiveRasterReady",
                        &[
                            ("completed_tiles", completions.len() as f64),
                            ("updated_placements", changed as f64),
                        ],
                    );
                    if changed != 0 {
                        self.dirty = true;
                        effects.push(Effect::VisualDamage);
                        effects.push(Effect::RequestBeginFrame);
                    }
                }
            }
            Command::SpareBundle(bundle) => {
                self.spare_bundle = Some(bundle);
                self.ScheduleRaster(&mut effects);
            }
            Command::InputBeginFrame(frame) => {
                let args = frame.BeginFrameArgs();
                browser_tracing::instant(
                    "input",
                    "InputBeginFrame",
                    &[
                        ("source_id", args.source_id as f64),
                        ("sequence", args.sequence_number as f64),
                    ],
                );
            }
            Command::BeginFrame(frame) => {
                self.current_begin_frame = Some(frame);
                let args = frame.BeginFrameArgs();
                browser_tracing::instant(
                    "frame",
                    "CompositorBeginFrame",
                    &[
                        ("source_id", args.source_id as f64),
                        ("sequence", args.sequence_number as f64),
                    ],
                );
                // cc's scheduler turns an impl-frame pulse into a main-frame
                // request. The Page never observes the physical source
                // directly; its owner event loop receives this effect.
                // cc Scheduler's non-overlapping main/pending-tree branch.
                // Input and display continue; only a new main commit waits.
                if self.main_frame_in_flight.is_some()
                    || self.raster_in_flight_main_commit
                    || self
                        .pending_snapshot
                        .as_ref()
                        .is_some_and(|snapshot| !self.snapshot_matches_active(snapshot))
                {
                    effects.push(Effect::RequestBeginFrame);
                    return Ok(effects);
                }
                if memory_diagnostics_enabled() {
                    eprintln!(
                        "pipeline-main-request time_ns={} source={} sequence={}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_or(0, |d| d.as_nanos()),
                        args.source_id,
                        args.sequence_number
                    );
                }
                self.main_frame_in_flight = Some((args.source_id, args.sequence_number));
                let scroll = self.scroll_tree.pull_delta_for_main();
                effects.push(Effect::MainThread(MainThreadEvent::BeginFrame {
                    frame,
                    document: self.snapshot.as_ref().map(|snapshot| snapshot.document),
                    scroll,
                }));
            }
            Command::MainFrameComplete {
                source_id,
                sequence_number,
            } => {
                if self.main_frame_in_flight == Some((source_id, sequence_number)) {
                    if memory_diagnostics_enabled() {
                        eprintln!(
                            "pipeline-main-complete time_ns={} source={} sequence={}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map_or(0, |d| d.as_nanos()),
                            source_id,
                            sequence_number
                        );
                    }
                    self.main_frame_in_flight = None;
                }
            }
            Command::DisplayBeginFrame(_) | Command::Stop => {}
        }
        Ok(effects)
    }

    fn snapshot_matches_active(&self, snapshot: &ArtifactSnapshot<V, M>) -> bool {
        self.planned
            .as_ref()
            .is_some_and(|active| active.snapshot.SameRasterInputs(snapshot))
    }

    fn ScheduleRaster(&mut self, effects: &mut Vec<Effect<V, F, M>>) {
        if self.raster_in_flight {
            return;
        }
        // RasterBundle is the retained pending-tree resource owner: its
        // LayerTileEngines and RasterEngines carry tile identities, pixels and
        // raster caches across updates. While the Page's output owner retains
        // the active bundle and its raster executor owns the pending bundle,
        // wait for that owner to return the previous active bundle. A third,
        // empty bundle here loses every retained tile and turns an otherwise
        // local image/layout update into a full visible-viewport raster.  cc
        // likewise bounds active/pending tree resources instead of replacing
        // a busy pending tree with a cache-cold one.
        if self.spare_bundle.is_none() {
            return;
        }
        let Some(snapshot) = self.pending_snapshot.take() else {
            return;
        };
        let activation_target = self.pending_activation_target.take();
        let mut bundle = self
            .spare_bundle
            .take()
            .expect("spare raster bundle checked above");
        let tile_size = snapshot.viewport.RasterTileSize();
        if bundle.tile_size != tile_size {
            bundle = RasterBundle::new(tile_size);
        }
        // Impl-side tile coverage may overlap main-frame synchronization;
        // only a distinct main commit participates in the pending-tree gate.
        self.raster_in_flight_main_commit = !self.snapshot_matches_active(&snapshot);
        report_pipeline("raster-scheduled", &snapshot);
        effects.push(Effect::Raster(RasterJob {
            snapshot,
            bundle,
            activation_target,
        }));
        self.raster_in_flight = true;
    }

    fn InstallRaster(
        &mut self,
        ready: RasterReady<V, M>,
        effects: &mut Vec<Effect<V, F, M>>,
    ) -> io::Result<()> {
        self.raster_in_flight = false;
        self.raster_in_flight_main_commit = false;
        let mut planned = match ready.result {
            Ok(planned) => planned,
            Err(error) => {
                self.spare_bundle = Some(ready.bundle);
                // Resource pressure while building a pending tree must not
                // tear down the active tree. cc keeps presenting the active
                // tree when required pending tiles cannot be prepared; a
                // later scroll/page commit supplies a fresh raster target.
                // Preserve a startup failure, where no active pixels exist,
                // so real initialization errors remain visible.
                if error.kind() == io::ErrorKind::OutOfMemory && self.planned.is_some() {
                    // This failed candidate owns no installable pending tree.
                    // Release its retry gate; a later main opportunity may
                    // retry the latest unactivated commit without spinning on
                    // an immediately repeated allocation failure.
                    self.pending_snapshot = None;
                    self.pending_activation_target = None;
                    browser_tracing::instant(
                        "raster",
                        "PendingTreeRasterRejected",
                        &[("kept_active_tree", 1.0)],
                    );
                    return Ok(());
                }
                if error.kind() == io::ErrorKind::Unsupported && self.planned.is_some() {
                    self.pending_snapshot = None;
                    self.pending_activation_target = None;
                    if !self.raster_fallback_requested {
                        self.raster_fallback_requested = true;
                        effects.push(Effect::MainThread(MainThreadEvent::RasterFallback));
                        effects.push(Effect::RequestBeginFrame);
                    }
                    browser_tracing::instant(
                        "raster",
                        "RetainedRasterFallback",
                        &[("kept_active_tree", 1.0)],
                    );
                    return Ok(());
                }
                return Err(error);
            }
        };
        // A Page commit may overtake raster work. Chromium never activates a
        // pending tree whose source revision is older than the current commit:
        // doing so would move the scroll transform backwards for one frame.
        if self
            .snapshot
            .as_ref()
            .is_some_and(|current| !current.SameRasterInputs(&planned.snapshot))
        {
            browser_tracing::instant(
                "raster",
                "StaleRasterTreeDiscarded",
                &[(
                    "frame_id",
                    planned.content.as_ref().map_or(0, |p| p.frame_id) as f64,
                )],
            );
            self.spare_bundle = Some(ready.bundle);
            self.ScheduleRaster(effects);
            return Ok(());
        }
        if let Some(current) = &self.snapshot {
            planned.snapshot.async_root_scroll = current.async_root_scroll;
            planned.snapshot.blocking_wheel_regions = current.blocking_wheel_regions.clone();
        }
        let root_scroll = planned.scroll_tree.root();
        let acknowledged = planned.snapshot.compositor_scroll_sequence;
        if let Some(root) = root_scroll {
            // SyncedScrollOffset: the main commit supplies the new base.
            // Preserve only impl deltas not reflected by that exact commit,
            // including wheel input which arrived while raster was in flight.
            let desired = self.scroll_tree.activation_position(root, acknowledged);
            // Impl-only raster work updates the active tree's resources;
            // requiring a complete replacement viewport here would discard
            // useful tiles whenever input advances during raster. A distinct
            // main commit still obeys the pending-tree activation boundary.
            let selection = if self.snapshot_matches_active(&planned.snapshot) {
                ScrollTileSelection::ActiveTree
            } else {
                ScrollTileSelection::PendingTree
            };
            if !apply_scroll_offset(&mut planned, root, desired, selection) {
                browser_tracing::instant(
                    "raster",
                    "PendingTreeActivationDeferred",
                    &[
                        ("scroll_node", root.id as f64),
                        ("committed", root.committed.y),
                        ("desired", desired.y),
                        (
                            "frame_id",
                            planned.content.as_ref().map_or(0, |plan| plan.frame_id) as f64,
                        ),
                    ],
                );
                self.spare_bundle = Some(ready.bundle);
                self.pending_snapshot = self.snapshot.clone();
                self.pending_activation_target = Some((root.id, desired));
                self.ScheduleRaster(effects);
                return Ok(());
            }
        }
        self.scroll_tree.activate(root_scroll, acknowledged);
        report_pipeline("activated", &planned.snapshot);
        self.planned = Some(planned);
        self.raster_fallback_requested = false;
        let tile_size = ready.bundle.tile_size;
        effects.push(Effect::InstallRasterBundle(ready.bundle));
        // Bootstrap the second half of the fixed active/pending pair.  The
        // first activation has no preceding active bundle to return;
        // every later install does.  Create exactly this one second owner,
        // then retain and exchange the same two bundles for the document's
        // lifetime.
        if !self.active_bundle_installed {
            self.active_bundle_installed = true;
            debug_assert!(self.spare_bundle.is_none());
            self.spare_bundle = Some(RasterBundle::new(tile_size));
        }
        self.dirty = true;
        effects.push(Effect::VisualDamage);
        effects.push(Effect::RequestBeginFrame);
        self.ScheduleRaster(effects);
        Ok(())
    }

    fn visible_overlay_rect(&self) -> Option<PaintRect> {
        self.scroll_tree
            .is_active()
            .then(|| {
                self.planned.as_ref().and_then(|frame| {
                    frame
                        .RootScrollbarQuads()
                        .map(|(quads, _)| {
                            let mut bounds = PaintRect::default();
                            for (part, quad) in frame
                                .root_scrollbars
                                .iter()
                                .flat_map(|layer| &layer.parts)
                                .zip(quads)
                            {
                                if part.is_thumb {
                                    bounds.union(quad.rect);
                                }
                            }
                            bounds
                        })
                        .or_else(|| frame.RootOverlayScrollbarQuad().map(|(quad, _)| quad.rect))
                })
            })
            .flatten()
    }

    pub fn Draw(&mut self, frame: F) -> Vec<Effect<V, F, M>> {
        vec![Effect::Submit {
            frame,
            scroll_active: self.scroll_tree.is_active(),
        }]
    }

    pub fn Prepare(&mut self, frame: F) -> io::Result<Vec<Effect<V, F, M>>> {
        let args = frame.BeginFrameArgs();
        let _frame = browser_tracing::scope(browser_tracing::Context {
            target_id: 1,
            source_id: args.source_id,
            frame_id: args.sequence_number,
            ..Default::default()
        });
        if !self.dirty {
            return Ok(Vec::new());
        }
        if let Some(root) = self.scroll_tree.root() {
            let desired = self.scroll_tree.visual_offset_for(root);
            if let Some(planned) = &mut self.planned {
                let _ =
                    apply_scroll_offset(planned, root, desired, ScrollTileSelection::ActiveTree);
            }
        }
        let Some(planned) = self.planned.clone() else {
            return Ok(Vec::new());
        };
        let ack = BeginFrameAck {
            source_id: args.source_id,
            sequence_number: args.sequence_number,
            has_damage: true,
        };
        let mut content = if let Some(plan) = planned.content.as_ref() {
            Some(
                self.content_frame_builder
                    .BuildFrame(&{
                        let mut draw = plan.clone();
                        for layer in &mut draw.layers {
                            if planned
                                .root_scrollbars
                                .iter()
                                .any(|scrollbar| scrollbar.layer_id == layer.id)
                            {
                                layer.tiles.clear();
                            }
                        }
                        draw
                    })
                    .map_err(io::Error::other)?
                    .WithBeginFrameAck(ack),
            )
        } else {
            self.content_frame_builder.Invalidate();
            None
        };
        if !planned.root_scrollbars.is_empty() {
            browser_tracing::instant(
                "input",
                "ScrollbarImplPosition",
                &[
                    (
                        "offset_y",
                        planned.visual_scroll.map_or(0.0, |(_, offset)| offset.y),
                    ),
                    ("layers", planned.root_scrollbars.len() as f64),
                ],
            );
        }
        let overlay = planned.RootScrollbarQuads().or_else(|| {
            self.scroll_tree
                .is_active()
                .then(|| {
                    planned
                        .RootOverlayScrollbarQuad()
                        .map(|(quad, damage)| (vec![quad], damage))
                })
                .flatten()
        });
        let next_overlay_damage = overlay.as_ref().map(|(_, damage)| *damage);
        if let Some(frame) = content.take() {
            content = Some(match overlay {
                Some((quads, damage)) => quads
                    .into_iter()
                    .fold(frame, |frame, quad| frame.WithSolidColorQuad(quad, damage)),
                None => match self.overlay_scrollbar_damage {
                    Some(damage) => frame.WithRootDamage(damage),
                    None => frame,
                },
            });
        }
        self.overlay_scrollbar_damage = next_overlay_damage;
        if content.is_none() {
            self.overlay_scrollbar_damage = None;
        }
        if let Some(root) = content
            .as_mut()
            .and_then(|frame| frame.render_pass_list.last_mut())
        {
            root.has_transparent_background = planned.snapshot.transparent_background;
        }
        let submitted = SubmittedFrame {
            provenance: planned.snapshot.Provenance(),
            viewport: planned.snapshot.viewport,
            content,
            metadata: planned.snapshot.metadata.clone(),
        };
        self.dirty = false;
        Ok(vec![Effect::FrameProduced {
            frame,
            submitted,
            scroll_active: self.scroll_tree.is_active(),
        }])
    }
}

pub fn PlanAndRaster<V, M>(
    bundle: &mut RasterBundle,
    snapshot: ArtifactSnapshot<V, M>,
    activation_target: Option<(u64, Offset)>,
) -> io::Result<PlannedFrame<V, M>>
where
    V: CompositorViewport,
    M: Clone,
{
    report_pipeline("raster-start", &snapshot);
    let diagnostic_start = memory_diagnostics_enabled().then(Instant::now);
    let mut trace = browser_tracing::span("raster", "RasterPendingTree");
    let provenance = snapshot.Provenance();
    trace.set("tab_id", provenance.document.0 as f64);
    trace.set("document_generation", provenance.document.1 as f64);
    trace.set("has_content", provenance.content.is_some() as u8 as f64);
    trace.set(
        "content_sequence",
        provenance.content.map_or(0, |c| c.sequence) as f64,
    );
    trace.set(
        "content_recording_revision",
        provenance.content.map_or(0, |c| c.recording_revision) as f64,
    );
    trace.set("succeeded", 0.0);
    let artifact_scale = |artifact: &PaintArtifact| {
        artifact
            .resources
            .as_ref()
            .map_or(1.0, |resources| resources.layout_zoom_factor)
            .max(f64::MIN_POSITIVE)
    };
    let source_root_scroll = snapshot
        .content
        .as_ref()
        .and_then(|artifact| build_artifact_scroll_tree(artifact).root());
    let raster_scroll = source_root_scroll.map(|root| (root.id, root.committed));
    if matches!(snapshot.content, ContentUpdate::Unchanged) {
        return Err(io::Error::other(
            "content update was not resolved by its compositor owner",
        ));
    }
    let content = if let Some(artifact) = snapshot.content.as_ref() {
        let layout_zoom = artifact_scale(artifact);
        let raster_scale = snapshot.viewport.Scale() / layout_zoom;
        let content_update = bundle
            .tiles
            .UpdatePending(
                artifact,
                FrameConfig {
                    viewport: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: snapshot.viewport.LogicalWidth() * layout_zoom,
                        height: snapshot.viewport.LogicalHeight() * layout_zoom,
                    },
                    raster_scale,
                    device_scale_factor: snapshot.viewport.Scale(),
                    layout_zoom_factor: layout_zoom,
                    activation_scroll: source_root_scroll.and_then(|root| {
                        activation_target
                            .filter(|(id, _)| *id == root.id)
                            .map(|(_, desired)| CompositorScrollOffset {
                                scroll_node_id: root.id,
                                translation_x: root.committed.x - desired.x,
                                translation_y: root.committed.y - desired.y,
                            })
                    }),
                    frame_time: Some(snapshot.frame_time),
                },
            )
            .map_err(|error| io::Error::new(io::ErrorKind::Unsupported, error.reason()))?;
        Some(content_update)
    } else {
        None
    };
    if content.as_ref().is_some_and(|update| {
        update.frame_plan.unsupported == Some(UnsupportedReason::TileBudgetExceeded)
    }) {
        return Err(io::Error::new(
            io::ErrorKind::OutOfMemory,
            "content tile budget exceeded",
        ));
    }
    let content = if let Some(content) = &content {
        let raster = bundle.raster.prepare(
            &content.frame_plan,
            &content.raster_batch,
            snapshot.viewport.Width(),
            snapshot.viewport.Height(),
        )?;
        bundle.tiles.ApplyRasterResults(&raster.completions);
        let content_release = bundle
            .tiles
            .ActivatePending()
            .ok_or_else(|| io::Error::other("content tiles are not ready for activation"))?;
        let released = bundle.raster.release_resources(&content_release)?;
        bundle.tiles.AcknowledgeResourceRelease(&released);
        Some(
            bundle
                .tiles
                .GetActiveFramePlan()
                .expect("activated content plan")
                .clone(),
        )
    } else {
        None
    };
    let scroll_tree = content.as_ref().map(build_scroll_tree).unwrap_or_default();
    let root_scroll = scroll_tree.root();
    trace.set("succeeded", 1.0);
    report_pipeline("raster-complete", &snapshot);
    if let Some(start) = diagnostic_start {
        eprintln!(
            "pipeline-raster-time tab={} generation={} sequence={} elapsed_ms={:.3}",
            snapshot.document.0,
            snapshot.document.1,
            snapshot.sequence,
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
    let root_scrollbars = content
        .as_ref()
        .zip(root_scroll)
        .map_or_else(Vec::new, |(plan, root)| plan.SolidScrollbarLayers(root.id));
    browser_tracing::instant(
        "input",
        "ScrollbarLayersCommitted",
        &[("layers", root_scrollbars.len() as f64)],
    );
    Ok(PlannedFrame {
        root_scrollbars,
        snapshot,
        content,
        scroll_tree,
        visual_scroll: root_scroll.map(|root| (root.id, root.committed)),
        raster_scroll,
    })
}

fn build_scroll_tree(plan: &FramePlan) -> ScrollTreeSnapshot {
    let mut nodes = std::collections::BTreeMap::new();
    let mut hit_test_regions = std::collections::BTreeMap::new();
    for layer in &plan.layers {
        let mut node = Some(&layer.properties.transform);
        while let Some(transform) = node {
            if let Some(scroll) = &transform.scroll {
                if let Some(clip) = &scroll.overflow_clip {
                    hit_test_regions.entry(scroll.id).or_insert_with(|| {
                        paint::paint_property_tree::PropertyTreeState {
                            transform: clip.local_transform_space.clone(),
                            clip: clip.clone(),
                            effect: layer.properties.effect.clone(),
                        }
                    });
                }
                nodes.entry(scroll.id).or_insert_with(|| {
                    let minimum = scroll.minimum_scroll_offset;
                    let maximum = Offset {
                        x: scroll.maximum_scroll_offset.x.max(minimum.x),
                        y: scroll.maximum_scroll_offset.y.max(minimum.y),
                    };
                    ScrollNode {
                        id: scroll.id,
                        owner_client_id: scroll.owner_client_id,
                        parent_id: scroll.parent.as_ref().map(|parent| parent.id),
                        container_rect: scroll.container_rect,
                        committed: Offset {
                            x: (-transform.matrix.values[12]).clamp(minimum.x, maximum.x),
                            y: (-transform.matrix.values[13]).clamp(minimum.y, maximum.y),
                        },
                        minimum,
                        maximum,
                        user_scrollable_horizontal: scroll.user_scrollable_horizontal,
                        user_scrollable_vertical: scroll.user_scrollable_vertical,
                    }
                });
            }
            node = transform.parent.as_ref();
        }
    }
    ScrollTreeSnapshot::new(nodes.into_values().collect()).with_hit_test_regions(hit_test_regions)
}

fn build_artifact_scroll_tree(artifact: &PaintArtifact) -> ScrollTreeSnapshot {
    let mut nodes = std::collections::BTreeMap::new();
    let mut hit_test_regions = std::collections::BTreeMap::new();
    for chunk in &artifact.chunks {
        let mut node = Some(&chunk.properties.transform);
        while let Some(transform) = node {
            if let Some(scroll) = &transform.scroll {
                if let Some(clip) = &scroll.overflow_clip {
                    hit_test_regions.entry(scroll.id).or_insert_with(|| {
                        paint::paint_property_tree::PropertyTreeState {
                            transform: clip.local_transform_space.clone(),
                            clip: clip.clone(),
                            effect: chunk.properties.effect.clone(),
                        }
                    });
                }
                nodes.entry(scroll.id).or_insert_with(|| {
                    let minimum = scroll.minimum_scroll_offset;
                    let maximum = Offset {
                        x: scroll.maximum_scroll_offset.x.max(minimum.x),
                        y: scroll.maximum_scroll_offset.y.max(minimum.y),
                    };
                    ScrollNode {
                        id: scroll.id,
                        owner_client_id: scroll.owner_client_id,
                        parent_id: scroll.parent.as_ref().map(|parent| parent.id),
                        container_rect: scroll.container_rect,
                        committed: Offset {
                            x: (-transform.matrix.values[12]).clamp(minimum.x, maximum.x),
                            y: (-transform.matrix.values[13]).clamp(minimum.y, maximum.y),
                        },
                        minimum,
                        maximum,
                        user_scrollable_horizontal: scroll.user_scrollable_horizontal,
                        user_scrollable_vertical: scroll.user_scrollable_vertical,
                    }
                });
            }
            node = transform.parent.as_ref();
        }
    }
    ScrollTreeSnapshot::new(nodes.into_values().collect()).with_hit_test_regions(hit_test_regions)
}

fn layer_has_scroll(transform: &Arc<TransformPaintPropertyNode>, id: u64) -> bool {
    let mut node = Some(transform);
    while let Some(current) = node {
        if current
            .scroll
            .as_ref()
            .is_some_and(|scroll| scroll.id == id)
        {
            return true;
        }
        node = current.parent.as_ref();
    }
    false
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollTileSelection {
    ActiveTree,
    PendingTree,
}

fn apply_scroll_offset<V, M>(
    frame: &mut PlannedFrame<V, M>,
    root: ScrollNode,
    desired: Offset,
    selection: ScrollTileSelection,
) -> bool {
    let Some(content) = &mut frame.content else {
        return true;
    };
    let raster_base = frame
        .raster_scroll
        .filter(|(id, _)| *id == root.id)
        .map_or(root.committed, |(_, offset)| offset);
    let delta = Offset {
        x: (raster_base.x - desired.x) * content.config.raster_scale,
        y: (raster_base.y - desired.y) * content.config.raster_scale,
    };
    // cc requires complete tiles at pending-tree activation. The active
    // tree's impl scroll continues with resident drawable tiles while raster
    // catches up (TileBasedLayerImpl::AppendQuads counts missing tiles).
    let active = (selection == ScrollTileSelection::PendingTree).then(|| {
        content
            .layers
            .iter()
            .map(|layer| (layer.compositor_scroll, layer.tiles.clone()))
            .collect::<Vec<_>>()
    });
    for layer in &mut content.layers {
        layer.compositor_scroll = if layer_has_scroll(&layer.properties.transform, root.id) {
            Some(CompositorScrollOffset {
                scroll_node_id: root.id,
                translation_x: raster_base.x - desired.x,
                translation_y: raster_base.y - desired.y,
            })
        } else {
            None
        };
    }
    if !select_ready_scroll_tiles(content, root.id, selection)
        && selection == ScrollTileSelection::PendingTree
    {
        for (layer, (scroll, tiles)) in content.layers.iter_mut().zip(active.unwrap()) {
            layer.compositor_scroll = scroll;
            layer.tiles = tiles;
        }
        browser_tracing::instant(
            "raster",
            "ScrollVisualActivationDeferred",
            &[("scroll_node", root.id as f64), ("desired", desired.y)],
        );
        return false;
    }
    browser_tracing::instant(
        "input",
        "ScrollApplied",
        &[
            ("scroll_node", root.id as f64),
            ("committed", root.committed.y),
            ("offset_x", desired.x),
            ("offset_y", desired.y),
            ("desired", desired.y),
            ("delta_device", delta.y),
            ("compositor", 1.0),
        ],
    );
    frame.visual_scroll = Some((root.id, desired));
    true
}

fn select_ready_scroll_tiles(
    plan: &mut FramePlan,
    scroll_id: u64,
    selection: ScrollTileSelection,
) -> bool {
    let scale = plan.config.raster_scale;
    let mut complete = true;
    for layer_index in 0..plan.layers.len() {
        if !layer_has_scroll(&plan.layers[layer_index].properties.transform, scroll_id) {
            continue;
        }
        let layer = &plan.layers[layer_index];
        let Ok((device_translation, device_clip)) =
            layer_tile::resolved_compositor_properties_with_scroll(
                &layer.properties,
                scale,
                layer.compositor_scroll,
            )
        else {
            continue;
        };
        let translation = (device_translation.0 / scale, device_translation.1 / scale);
        // The clip and transform are one property-tree snapshot.  A clip below
        // the root scroll moves with compositor scrolling; using the retained
        // committed root_clip here selected the preceding viewport's tiles
        // while final composition used the updated clip.  The newly exposed
        // strip (most visible below large carousel/effect layers) therefore
        // had no source pixels for one activation.
        let compositor_clip = device_clip.map(|clip| PaintRect {
            x: clip.x / scale,
            y: clip.y / scale,
            width: clip.width / scale,
            height: clip.height / scale,
        });
        let visible_root = compositor_clip.map_or(plan.config.viewport, |clip| PaintRect {
            x: plan.config.viewport.x.max(clip.x),
            y: plan.config.viewport.y.max(clip.y),
            width: ((plan.config.viewport.x + plan.config.viewport.width).min(clip.x + clip.width)
                - plan.config.viewport.x.max(clip.x))
            .max(0.0),
            height: ((plan.config.viewport.y + plan.config.viewport.height)
                .min(clip.y + clip.height)
                - plan.config.viewport.y.max(clip.y))
            .max(0.0),
        });
        let mut visible = PaintRect {
            x: visible_root.x - translation.0,
            y: visible_root.y - translation.1,
            width: visible_root.width,
            height: visible_root.height,
        };
        // Bilinear sampling reads the neighboring source pixel at a
        // fractional device placement.  Match LayerTileManager's visible-grid
        // admission so activation never clamps a missing edge tile.
        if visible.width > 0.0
            && visible.height > 0.0
            && ((translation.0 * scale) != (translation.0 * scale).round()
                || (translation.1 * scale) != (translation.1 * scale).round())
        {
            let footprint = 1.0 / scale;
            visible = PaintRect {
                x: visible.x - footprint,
                y: visible.y - footprint,
                width: visible.width + 2.0 * footprint,
                height: visible.height + 2.0 * footprint,
            };
        }
        if !ordinary_effect_is_drawn(&layer.properties) {
            continue;
        }
        // The interest grid is allowed to be empty when this layer has no
        // drawing support in the requested viewport.  Otherwise an empty
        // intersection means the requested scroll escaped the resident
        // tiling and must not activate yet.
        let required_visible = if let Some(bounds) = layer.raster_content_bounds {
            intersect_rect(visible, bounds)
        } else if layer.is_first_layer && !layer.requires_transparent_backing {
            visible
        } else if layer.bounds_are_complete {
            intersect_rect(visible, layer.bounds)
        } else {
            visible
        };
        if required_visible.width <= 0.0 || required_visible.height <= 0.0 {
            continue;
        }
        let wanted: Vec<_> = layer
            .interest_tiles
            .iter()
            .filter(|tile| {
                tile.tile_rect.x < visible.x + visible.width
                    && tile.tile_rect.x + tile.tile_rect.width > visible.x
                    && tile.tile_rect.y < visible.y + visible.height
                    && tile.tile_rect.y + tile.tile_rect.height > visible.y
            })
            .cloned()
            .collect();
        let mut selected: Vec<_> = wanted
            .iter()
            .filter(|tile| plan.TileIsReady(tile))
            .cloned()
            .collect();
        let viewport_covered =
            tiles_cover_rect(required_visible, &wanted, layer.raster_origin, scale);
        if selected.len() == wanted.len() && viewport_covered {
            // Activate a complete visible tiling in one property-tree state.
            plan.layers[layer_index].tiles = selected;
        } else {
            // Pending activation retains the preceding complete tree. Active
            // scrolling uses each ready tile in its actual content coordinates;
            // missing coverage must not freeze the scroll transform.
            complete = false;
            let layer_id = plan.layers[layer_index].id.0;
            browser_tracing::instant(
                "raster",
                if selection == ScrollTileSelection::ActiveTree {
                    "ScrollTileCoverageMissing"
                } else {
                    "ScrollTileActivationDeferred"
                },
                &[
                    ("layer", layer_id as f64),
                    ("wanted", wanted.len() as f64),
                    ("ready", selected.len() as f64),
                    ("viewport_covered", viewport_covered as u8 as f64),
                    ("interest", layer.interest_tiles.len() as f64),
                    ("bounds_complete", layer.bounds_are_complete as u8 as f64),
                    (
                        "has_raster_bounds",
                        layer.raster_content_bounds.is_some() as u8 as f64,
                    ),
                ],
            );
            browser_tracing::instant(
                "raster",
                "ScrollTileCoverageGeometry",
                &[
                    ("layer", layer_id as f64),
                    ("required_x", required_visible.x),
                    ("required_y", required_visible.y),
                    ("required_width", required_visible.width),
                    ("required_height", required_visible.height),
                    ("origin_x", layer.raster_origin.0),
                    ("origin_y", layer.raster_origin.1),
                    ("scale", scale),
                ],
            );
            if selection == ScrollTileSelection::ActiveTree {
                for tile in &layer.tiles {
                    if plan.TileIsReady(tile)
                        && tile.tile_rect.x < visible.x + visible.width
                        && tile.tile_rect.x + tile.tile_rect.width > visible.x
                        && tile.tile_rect.y < visible.y + visible.height
                        && tile.tile_rect.y + tile.tile_rect.height > visible.y
                        && !selected
                            .iter()
                            .any(|candidate| candidate.tile_id == tile.tile_id)
                    {
                        selected.push(tile.clone());
                    }
                }
                plan.layers[layer_index].tiles = selected;
            }
        }
    }
    complete
}

// PictureLayerTiling::CoverageIterator accounts for the entire visible rect,
// including grid slots absent from the interest set. Checking only known tile
// readiness admits a partial viewport when input outruns the pending tiling.
fn tiles_cover_rect(
    required: PaintRect,
    tiles: &[layer_tile::TilePlacement],
    origin: (f64, f64),
    scale: f64,
) -> bool {
    if required.is_empty() {
        return true;
    }
    // CoverageIterator walks integer raster-content pixels. Restore those
    // original tile coordinates before checking adjacency; accumulating CSS
    // widths at fractional scale need not reproduce the shared edge exactly.
    let x = ((required.x - origin.0) * scale).floor();
    let y = ((required.y - origin.1) * scale).floor();
    let required = PaintRect {
        x,
        y,
        width: ((required.x + required.width - origin.0) * scale).ceil() - x,
        height: ((required.y + required.height - origin.1) * scale).ceil() - y,
    };
    let tiles = tiles
        .iter()
        .map(|tile| PaintRect {
            x: ((tile.tile_rect.x - origin.0) * scale).round(),
            y: ((tile.tile_rect.y - origin.1) * scale).round(),
            width: tile.pixel_size.0 as f64,
            height: tile.pixel_size.1 as f64,
        })
        .collect::<Vec<_>>();
    let bottom = required.y + required.height;
    let right = required.x + required.width;
    let mut rows = vec![required.y, bottom];
    for tile in &tiles {
        rows.push(tile.y.clamp(required.y, bottom));
        rows.push((tile.y + tile.height).clamp(required.y, bottom));
    }
    rows.sort_by(f64::total_cmp);
    rows.dedup();
    for row in rows.windows(2) {
        let mut intervals = tiles
            .iter()
            .filter_map(|tile| {
                let rect = *tile;
                (rect.y <= row[0] && rect.y + rect.height >= row[1])
                    .then_some((rect.x, rect.x + rect.width))
            })
            .collect::<Vec<_>>();
        intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut covered = required.x;
        for (left, end) in intervals {
            if left > covered {
                break;
            }
            covered = covered.max(end);
            if covered >= right {
                break;
            }
        }
        if covered < right {
            return false;
        }
    }
    true
}

fn intersect_rect(a: PaintRect, b: PaintRect) -> PaintRect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    PaintRect {
        x,
        y,
        width: ((a.x + a.width).min(b.x + b.width) - x).max(0.0),
        height: ((a.y + a.height).min(b.y + b.height) - y).max(0.0),
    }
}

fn ordinary_effect_is_drawn(properties: &paint::paint_property_tree::PropertyTreeState) -> bool {
    let mut effect = Some(properties.effect.as_ref());
    while let Some(node) = effect {
        if !node.is_mask && node.opacity == 0.0 {
            return false;
        }
        effect = node.parent.as_deref();
    }
    true
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    fn snapshot(generation: u64, content: bool) -> ArtifactSnapshot<(), ()> {
        ArtifactSnapshot {
            document: (7, generation),

            viewport: (),

            frame_time: Instant::now(),
            sequence: 1,
            compositor_scroll_sequence: 0,
            transparent_background: false,

            content: content
                .then(|| {
                    Arc::new(PaintArtifact {
                        recording_revision: 19,
                        ..Default::default()
                    })
                })
                .into(),

            metadata: (),
            async_root_scroll: false,
            blocking_wheel_regions: Default::default(),
        }
    }

    #[test]
    fn empty_page_snapshot_has_no_content_provenance() {
        let provenance = snapshot(4, false).Provenance();
        assert_eq!(provenance.document, (7, 4));
        assert_eq!(provenance.content, None);
    }

    #[test]
    fn retained_snapshot_keeps_its_generation_when_content_sequences_restart() {
        let active = snapshot(4, true);
        let replacement = snapshot(5, true);
        let retained_provenance = active.Provenance();
        assert_eq!(retained_provenance.document, (7, 4));
        assert_eq!(
            retained_provenance.content,
            Some(ContentFrameProvenance {
                sequence: 1,
                recording_revision: 19,
            })
        );
        assert_ne!(retained_provenance, replacement.Provenance());
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use std::time::Duration;

    #[derive(Clone, Copy, PartialEq)]
    struct Viewport;
    impl CompositorViewport for Viewport {
        fn Width(self) -> u32 {
            32
        }
        fn Height(self) -> u32 {
            32
        }
        fn Scale(self) -> f64 {
            1.0
        }
    }
    type Engine = CompositorEngine<Viewport, BeginFrameArgs, ()>;
    fn frame(sequence_number: u64) -> BeginFrameArgs {
        let now = Instant::now();
        BeginFrameArgs {
            source_id: 5,
            sequence_number,
            frame_time: now,
            deadline: now + Duration::from_millis(16),
            interval: Duration::from_millis(16),
        }
    }
    fn snapshot(sequence: u64) -> ArtifactSnapshot<Viewport, ()> {
        ArtifactSnapshot {
            document: (1, 1),

            viewport: Viewport,

            frame_time: Instant::now(),
            sequence,
            compositor_scroll_sequence: 0,
            transparent_background: false,

            content: ContentUpdate::Clear,

            metadata: (),
            async_root_scroll: false,
            blocking_wheel_regions: Default::default(),
        }
    }
    fn requests_main(effects: &[Effect<Viewport, BeginFrameArgs, ()>]) -> bool {
        effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::MainThread(MainThreadEvent::BeginFrame { .. })
            )
        })
    }
    fn complete(engine: &mut Engine, sequence_number: u64) {
        engine
            .Handle(Command::MainFrameComplete {
                source_id: 5,
                sequence_number,
            })
            .unwrap();
    }
    fn activate_empty(engine: &mut Engine) {
        let mut job = engine
            .Handle(Command::Snapshot(snapshot(1)))
            .unwrap()
            .into_iter()
            .find_map(|effect| match effect {
                Effect::Raster(job) => Some(job),
                _ => None,
            })
            .unwrap();
        let result = PlanAndRaster(&mut job.bundle, job.snapshot, job.activation_target);
        engine
            .Handle(Command::RasterReady(RasterReady {
                result,
                bundle: job.bundle,
            }))
            .unwrap();
    }
    #[test]
    fn main_opportunities_are_serial_and_only_matching_completion_releases_them() {
        let mut engine = Engine::default();
        assert!(requests_main(
            &engine.Handle(Command::BeginFrame(frame(1))).unwrap()
        ));
        assert!(!requests_main(
            &engine.Handle(Command::BeginFrame(frame(2))).unwrap()
        ));
        complete(&mut engine, 2);
        assert!(!requests_main(
            &engine.Handle(Command::BeginFrame(frame(3))).unwrap()
        ));
        complete(&mut engine, 1);
        assert!(requests_main(
            &engine.Handle(Command::BeginFrame(frame(4))).unwrap()
        ));
        complete(&mut engine, 4);
        // Main-thread abort/clean frames release the same gate without a snapshot.
        assert!(requests_main(
            &engine.Handle(Command::BeginFrame(frame(5))).unwrap()
        ));
        // Separate instances retain only their own Page state.
        let mut updates = Engine::default();
        let artifact = Arc::new(PaintArtifact::default());
        let mut committed = snapshot(7);
        committed.content = ContentUpdate::Replace(artifact.clone());
        updates.Handle(Command::Snapshot(committed)).unwrap();
        let mut unchanged = snapshot(0);
        unchanged.content = ContentUpdate::Unchanged;
        updates
            .Handle(Command::Snapshot(unchanged.clone()))
            .unwrap();
        let retained = updates.snapshot.as_ref().unwrap();
        assert_eq!(retained.sequence, 7);
        assert!(Arc::ptr_eq(retained.content.as_ref().unwrap(), &artifact));
        let mut other_page = Engine::default();
        let mut other = unchanged.clone();
        other.document = (2, 1);
        other_page.Handle(Command::Snapshot(other.clone())).unwrap();
        assert!(other_page
            .snapshot
            .as_ref()
            .unwrap()
            .content
            .as_ref()
            .is_none());
        assert!(updates.Handle(Command::Snapshot(other)).is_err());
        unchanged.content = ContentUpdate::Clear;
        updates.Handle(Command::Snapshot(unchanged)).unwrap();
        assert!(updates.committed_content.is_none());
    }
    #[test]
    fn impl_coverage_raster_allows_main_but_a_distinct_commit_waits_for_activation() {
        let mut engine = Engine::default();
        activate_empty(&mut engine);
        engine.pending_snapshot = Some(snapshot(1));
        let mut effects = vec![];
        engine.ScheduleRaster(&mut effects);
        assert!(engine.raster_in_flight);
        assert!(!engine.raster_in_flight_main_commit);
        assert!(requests_main(
            &engine.Handle(Command::BeginFrame(frame(1))).unwrap()
        ));
        complete(&mut engine, 1);
        // A newer main tree waits behind the impl coverage job, and gates another commit.
        engine.Handle(Command::Snapshot(snapshot(2))).unwrap();
        assert!(!requests_main(
            &engine.Handle(Command::BeginFrame(frame(2))).unwrap()
        ));
        engine.pending_snapshot = None;
        engine.raster_in_flight_main_commit = true;
        assert!(!requests_main(
            &engine.Handle(Command::BeginFrame(frame(3))).unwrap()
        ));
    }
    #[test]
    fn recoverable_raster_failure_keeps_active_and_releases_gate_for_later_retry() {
        let mut engine = Engine::default();
        activate_empty(&mut engine);
        let mut effects = engine.Handle(Command::Snapshot(snapshot(2))).unwrap();
        let job = effects
            .drain(..)
            .find_map(|effect| match effect {
                Effect::Raster(job) => Some(job),
                _ => None,
            })
            .unwrap();
        // Newer requested work must not leave a permanent pending gate after failure.
        engine.pending_snapshot = Some(snapshot(2));
        let effects = engine
            .Handle(Command::RasterReady(RasterReady {
                result: Err(io::Error::new(
                    io::ErrorKind::OutOfMemory,
                    "test allocation failure",
                )),
                bundle: job.bundle,
            }))
            .unwrap();
        assert!(effects.is_empty());
        assert_eq!(engine.planned.as_ref().unwrap().snapshot.sequence, 1);
        assert!(engine.pending_snapshot.is_none());
        assert!(!engine.raster_in_flight_main_commit);
        assert!(requests_main(
            &engine.Handle(Command::BeginFrame(frame(1))).unwrap()
        ));
        complete(&mut engine, 1);
        assert!(engine
            .Handle(Command::Snapshot(snapshot(2)))
            .unwrap()
            .iter()
            .any(|e| matches!(e, Effect::Raster(_))));
        let mut startup = Engine::default();
        assert!(startup
            .Handle(Command::RasterReady(RasterReady {
                result: Err(io::Error::new(
                    io::ErrorKind::OutOfMemory,
                    "startup failure"
                )),
                bundle: RasterBundle::default()
            }))
            .is_err());
    }
    #[test]
    fn actual_wheel_notifies_only_changed_drawable_state_after_clean_prepare() {
        fn scrolling_engine(ready: bool) -> Engine {
            let mut engine = Engine::default();
            let mut snapshot = snapshot(1);
            snapshot.async_root_scroll = true;
            let mut content = crate::retained_damage_tests::fixture();
            content.config.viewport.width = 32.;
            content.config.viewport.height = 32.;
            for layer in &mut content.layers {
                layer.interest_tiles = if ready { layer.tiles.clone() } else { vec![] };
            }
            let root = ScrollNode {
                id: 2,
                owner_client_id: 2,
                parent_id: None,
                container_rect: content.config.viewport,
                committed: Offset::default(),
                minimum: Offset::default(),
                maximum: Offset { x: 0., y: 32. },
                user_scrollable_horizontal: false,
                user_scrollable_vertical: true,
            };
            engine.scroll_tree.activate(Some(root), 0);
            engine.snapshot = Some(snapshot.clone());
            engine.planned = Some(PlannedFrame {
                snapshot,

                content: Some(content),
                scroll_tree: ScrollTreeSnapshot::new(vec![root]),
                visual_scroll: Some((2, Offset::default())),
                raster_scroll: Some((2, Offset::default())),
                root_scrollbars: Vec::new(),
            });
            engine.dirty = false;
            engine.raster_in_flight = true; // Keep this fixture on the actual impl-only input path.
            engine
        }
        let wheel = |delta| Command::Wheel {
            event: WheelEvent {
                delta_units: ScrollGranularity::kScrollByPrecisePixel,
                delta: Offset { x: 0., y: delta },
                ..Default::default()
            },
            queued_at: Instant::now(),
            samples: 1,
        };
        let mut entering = scrolling_engine(true);
        entering
            .planned
            .as_mut()
            .unwrap()
            .content
            .as_mut()
            .unwrap()
            .layers[1]
            .tiles
            .clear();
        assert!(
            entering
                .Handle(wheel(3.))
                .unwrap()
                .iter()
                .any(|effect| matches!(effect, Effect::VisualDamage)),
            "newly selected resident interest tiles also change drawable content"
        );
        // All known tiles can be ready while their interest grid covers only
        // a prefix of the newly visible viewport. Pending activation must keep
        // the complete active tree rather than publishing that partial strip.
        let mut partial = scrolling_engine(true);
        for tile in &mut partial
            .planned
            .as_mut()
            .unwrap()
            .content
            .as_mut()
            .unwrap()
            .layers[1]
            .interest_tiles
        {
            tile.tile_rect.height = 16.;
            tile.pixel_size.1 = 16;
        }
        let root = partial.scroll_tree.root().unwrap();
        assert!(!apply_scroll_offset(
            partial.planned.as_mut().unwrap(),
            root,
            Offset { x: 0., y: 3. },
            ScrollTileSelection::PendingTree
        ));
        assert_eq!(
            partial.planned.as_ref().unwrap().visual_scroll.unwrap().1.y,
            0.
        );
        for ready in [false, true] {
            let mut engine = scrolling_engine(ready);
            assert!(engine.Prepare(frame(1)).unwrap().is_empty());
            if !ready {
                let root = engine.scroll_tree.root().unwrap();
                assert!(!apply_scroll_offset(
                    engine.planned.as_mut().unwrap(),
                    root,
                    Offset { x: 0., y: 3. },
                    ScrollTileSelection::PendingTree
                ));
                assert_eq!(
                    engine.planned.as_ref().unwrap().visual_scroll.unwrap().1.y,
                    0.
                );
            }
            let effects = engine.Handle(wheel(3.)).unwrap();
            assert_eq!(
                effects
                    .iter()
                    .any(|effect| matches!(effect, Effect::VisualDamage)),
                true
            );
            assert!(effects
                .iter()
                .any(|effect| matches!(effect, Effect::RequestBeginFrame)));
            assert!(effects
                .iter()
                .any(|effect| matches!(effect, Effect::MainThread(MainThreadEvent::Wheel { .. }))));
            let visual = engine.planned.as_ref().unwrap().visual_scroll.unwrap().1.y;
            assert_eq!(visual, 3.);
            if ready {
                assert!(engine
                    .Prepare(frame(1))
                    .unwrap()
                    .iter()
                    .any(|effect| matches!(effect, Effect::FrameProduced { .. })));
                assert!(engine.Prepare(frame(1)).unwrap().is_empty());
                assert!(!engine
                    .Handle(wheel(0.))
                    .unwrap()
                    .iter()
                    .any(|effect| matches!(effect, Effect::VisualDamage)));
            }
            engine.scroll_tree.pull_delta_for_main().unwrap();
            assert!(engine.scroll_tree.has_unacknowledged_batch(0));
            engine.Handle(Command::Snapshot(snapshot(2))).unwrap();
            assert_eq!(
                engine.snapshot.as_ref().unwrap().sequence,
                2,
                "unacknowledged impl input must not reject a new main snapshot"
            );
        }
        // Hiding a Page leaves its compositor instance and SyncedProperty intact.
        let mut returning = scrolling_engine(true);
        let mut other = scrolling_engine(true);
        for expected in 1..=3 {
            returning.Handle(wheel(3.)).unwrap();
            let packet = returning.scroll_tree.pull_delta_for_main().unwrap();
            assert_eq!(packet.sequence, expected);
            let mut root = returning.scroll_tree.root().unwrap();
            root.committed.y += packet.delta.y;
            returning.scroll_tree.activate(Some(root), expected);
            other.Handle(wheel(1.)).unwrap();
            assert_eq!(
                other.scroll_tree.pull_delta_for_main().unwrap().sequence,
                expected
            );
            let mut root = other.scroll_tree.root().unwrap();
            root.committed.y += 1.;
            other.scroll_tree.activate(Some(root), expected);
        }
        let mut navigated = scrolling_engine(true);
        navigated.Handle(wheel(3.)).unwrap();
        assert_eq!(
            navigated
                .scroll_tree
                .pull_delta_for_main()
                .unwrap()
                .sequence,
            1
        );
    }
}
