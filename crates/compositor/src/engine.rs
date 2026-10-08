//! Retained compositor state, raster coordination, and CompositorFrame production.
//!
//! The engine is independent of its thread, native window, display service, and
//! application metadata. The embedding supplies concrete frame/viewport types
//! and routes typed effects to raster ownership and Viz.

use crate::{BeginFrameAck, CompositorFrame, DeviceRect, FrameBuilder, SolidColorDrawQuad};
use foundation::begin_frame::BeginFrameArgs;
use interaction::input_event::{ScrollGranularity, WheelEvent, WheelPhase};
use layer_tile::{
    CompositorScrollOffset, FrameConfig, FramePlan, LayerTileEngine, UnsupportedReason,
};
use paint::{
    paint_engine::{PaintArtifact, PaintRect},
    paint_property_tree::TransformPaintPropertyNode,
};
use std::{io, sync::Arc, time::Instant};

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

    fn LogicalWidth(self) -> f64 {
        f64::from(self.Width()) / self.Scale()
    }

    fn ToolbarPixels(self, toolbar_height: f64) -> u32 {
        (toolbar_height * self.Scale()).round() as u32
    }

    fn ContentHeight(self, toolbar_height: f64) -> f64 {
        f64::from(
            self.Height()
                .saturating_sub(self.ToolbarPixels(toolbar_height))
                .max(1),
        ) / self.Scale()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockingWheelEventRegions {
    pub covers_viewport: bool,
    pub rects: Vec<PaintRect>,
}

impl BlockingWheelEventRegions {
    pub fn Contains(&self, x: f64, y: f64) -> bool {
        self.covers_viewport
            || self.rects.iter().any(|rect| {
                x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
            })
    }
}

#[derive(Clone)]
pub struct ArtifactSnapshot<V, M> {
    pub document: (u64, u64),
    pub viewport: V,
    pub toolbar_height: f64,
    pub frame_time: Instant,
    pub signature: (u64, u64, V),
    pub toolbar: Arc<PaintArtifact>,
    pub content: Option<Arc<PaintArtifact>>,
    pub metadata: M,
    pub async_root_scroll: bool,
    pub blocking_wheel_regions: BlockingWheelEventRegions,
}

pub enum Command<V, F, M> {
    Snapshot(ArtifactSnapshot<V, M>),
    InputBeginFrame(F),
    BeginFrame(F),
    DisplayBeginFrame(F),
    Wheel {
        event: WheelEvent,
        queued_at: Instant,
    },
    Resize(V),
    Redraw,
    RasterReady(RasterReady<V, M>),
    SpareBundle(RasterBundle),
    Stop,
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
    BeginMainFrame(F),
    RequestBeginFrame,
}

#[derive(Clone)]
pub struct PlannedFrame<V, M> {
    pub snapshot: ArtifactSnapshot<V, M>,
    pub toolbar: FramePlan,
    pub content: Option<FramePlan>,
    root_scroll: Option<RootScroll>,
    visual_scroll: Option<(u64, f64)>,
    raster_scroll: Option<(u64, f64)>,
}

pub struct SubmittedFrame<V, M> {
    pub viewport: V,
    pub toolbar: CompositorFrame,
    pub content: Option<CompositorFrame>,
    pub metadata: M,
}

#[derive(Default)]
pub struct RasterBundle {
    pub toolbar_tiles: LayerTileEngine,
    pub content_tiles: LayerTileEngine,
    pub toolbar_raster: raster::RasterEngine,
    pub content_raster: raster::RasterEngine,
}

pub struct RasterJob<V, M> {
    pub snapshot: ArtifactSnapshot<V, M>,
    pub bundle: RasterBundle,
    pub activation_target: Option<(u64, f64)>,
}

pub struct RasterReady<V, M> {
    pub result: io::Result<PlannedFrame<V, M>>,
    pub bundle: RasterBundle,
}

#[derive(Clone, Copy)]
struct RootScroll {
    id: u64,
    committed: f64,
    maximum: f64,
}

impl<V: CompositorViewport, M> PlannedFrame<V, M> {
    fn RootOverlayScrollbarQuad(&self) -> Option<(SolidColorDrawQuad, DeviceRect)> {
        let root = self.root_scroll?;
        if root.maximum <= 0.0 {
            return None;
        }
        let offset = self
            .visual_scroll
            .filter(|(id, _)| *id == root.id)
            .map_or(root.committed, |(_, value)| value)
            .clamp(0.0, root.maximum);
        let viewport = self
            .snapshot
            .viewport
            .ContentHeight(self.snapshot.toolbar_height)
            .max(1.0);
        let contents = viewport + root.maximum;
        let inner_length = (viewport - 4.0).max(0.0);
        let thumb_length = (inner_length * viewport / contents)
            .max(24.0)
            .min(inner_length);
        if thumb_length <= 0.0 {
            return None;
        }
        let travel = (inner_length - thumb_length).max(0.0);
        let top = 2.0 + travel * (offset / root.maximum).clamp(0.0, 1.0);
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
        let height = self.snapshot.viewport.Height().saturating_sub(
            self.snapshot
                .viewport
                .ToolbarPixels(self.snapshot.toolbar_height),
        );
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
    toolbar_frame_builder: FrameBuilder,
    content_frame_builder: FrameBuilder,
    spare_bundle: Option<RasterBundle>,
    display_has_bundle: bool,
    raster_in_flight: bool,
    pending_snapshot: Option<ArtifactSnapshot<V, M>>,
    pending_activation_target: Option<(u64, f64)>,
    snapshot: Option<ArtifactSnapshot<V, M>>,
    planned: Option<PlannedFrame<V, M>>,
    committed_scroll: Option<(u64, f64)>,
    pending_scroll_delta: f64,
    scroll_active: bool,
    wheel_blocked_on_main: Option<bool>,
    overlay_scrollbar_damage: Option<DeviceRect>,
    current_begin_frame: Option<F>,
    dirty: bool,
}

impl<V, F, M> Default for CompositorEngine<V, F, M> {
    fn default() -> Self {
        Self {
            toolbar_frame_builder: FrameBuilder::default(),
            content_frame_builder: FrameBuilder::default(),
            spare_bundle: Some(RasterBundle::default()),
            display_has_bundle: false,
            raster_in_flight: false,
            pending_snapshot: None,
            pending_activation_target: None,
            snapshot: None,
            planned: None,
            committed_scroll: None,
            pending_scroll_delta: 0.0,
            scroll_active: false,
            wheel_blocked_on_main: None,
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
            Command::Snapshot(snapshot) => {
                let document_changed = self
                    .snapshot
                    .as_ref()
                    .is_some_and(|old| old.document != snapshot.document);
                let raster_changed = self.snapshot.as_ref().is_none_or(|old| {
                    old.document != snapshot.document || old.signature != snapshot.signature
                });
                if document_changed {
                    self.toolbar_frame_builder.Invalidate();
                    self.content_frame_builder.Invalidate();
                    // Paint property node ids are scoped to a document.  A
                    // newly created Page commonly starts its root scroll node
                    // at the same small numeric id as the preceding Page; that
                    // is not identity continuity.  Keep the old active pixels
                    // until the replacement tree is ready, but do not carry
                    // any of its scroll state into the replacement.
                    self.committed_scroll = None;
                    self.pending_scroll_delta = 0.0;
                    self.pending_activation_target = None;
                    self.wheel_blocked_on_main = None;
                    self.scroll_active = false;
                    browser_tracing::instant(
                        "input",
                        "CompositorDocumentScrollReset",
                        &[
                            ("tab", snapshot.document.0 as f64),
                            ("generation", snapshot.document.1 as f64),
                        ],
                    );
                }
                // Input regions are compositor metadata, not raster content.
                // Refresh them even when the PaintArtifact identity is stable.
                if let Some(planned) = &mut self.planned {
                    planned.snapshot.async_root_scroll = snapshot.async_root_scroll;
                    planned.snapshot.blocking_wheel_regions =
                        snapshot.blocking_wheel_regions.clone();
                }
                self.snapshot = Some(snapshot.clone());
                if raster_changed {
                    self.pending_snapshot = Some(snapshot);
                    self.pending_activation_target = if document_changed {
                        None
                    } else {
                        self.current_scroll_position()
                    };
                    self.ScheduleRaster(&mut effects);
                }
            }
            Command::Wheel { event, queued_at } => {
                let _input = browser_tracing::scope(browser_tracing::Context {
                    target_id: 1,
                    input_id: browser_tracing::instant_id(queued_at),
                    ..Default::default()
                });
                let toolbar_height = self
                    .snapshot
                    .as_ref()
                    .map_or(0.0, |snapshot| snapshot.toolbar_height);
                if event.position.y < toolbar_height
                    || event.delta_units != ScrollGranularity::kScrollByPrecisePixel
                {
                    return Ok(effects);
                }
                let started = Instant::now();
                browser_tracing::interval("input", "CompositorInputQueue", queued_at, started, &[]);
                let current_document = self.snapshot.as_ref().map(|snapshot| snapshot.document);
                let compositor_ready = self
                    .planned
                    .as_ref()
                    .is_some_and(|planned| Some(planned.snapshot.document) == current_document);
                let blocks_main = !compositor_ready
                    || self.planned.as_ref().is_some_and(|planned| {
                        planned.snapshot.async_root_scroll
                            && planned.snapshot.blocking_wheel_regions.Contains(
                                event.position.x,
                                event.position.y - planned.snapshot.toolbar_height
                                    + self.pending_scroll_delta,
                            )
                    });
                let blocks_main = match event.phase {
                    WheelPhase::kBegan => {
                        self.wheel_blocked_on_main = Some(blocks_main);
                        blocks_main
                    }
                    WheelPhase::kChanged | WheelPhase::kEnded | WheelPhase::kCancelled => {
                        self.wheel_blocked_on_main.unwrap_or(blocks_main)
                    }
                    WheelPhase::kNone => blocks_main,
                };
                let was_scroll_active = self.scroll_active;
                let mut did_scroll = false;
                if let Some(planned) = &mut self.planned {
                    if planned.snapshot.async_root_scroll && !blocks_main {
                        if let Some(root) = planned.root_scroll {
                            let before = (root.committed + self.pending_scroll_delta)
                                .clamp(0.0, root.maximum);
                            let desired = (before + event.delta.y).clamp(0.0, root.maximum);
                            if desired != before {
                                did_scroll = true;
                                self.pending_scroll_delta += desired - before;
                                let _ = apply_scroll_offset(planned, desired);
                                self.dirty = true;
                                self.pending_snapshot =
                                    self.snapshot.clone().map(|mut snapshot| {
                                        snapshot.frame_time = queued_at;
                                        snapshot
                                    });
                                self.pending_activation_target = Some((root.id, desired));
                                self.ScheduleRaster(&mut effects);
                            }
                        }
                    }
                }
                match event.phase {
                    WheelPhase::kBegan | WheelPhase::kChanged if did_scroll && !blocks_main => {
                        self.scroll_active = true;
                    }
                    WheelPhase::kEnded | WheelPhase::kCancelled => {
                        self.scroll_active = false;
                        // Removing the compositor-owned overlay is itself a
                        // visual change even though the final wheel sample has
                        // no displacement.
                        if was_scroll_active {
                            self.dirty = true;
                        }
                    }
                    _ => {}
                }
                browser_tracing::instant(
                    "input",
                    "WheelCompositorDisposition",
                    &[
                        ("blocked_on_main", blocks_main as u8 as f64),
                        (
                            "region_rects",
                            self.planned.as_ref().map_or(0.0, |planned| {
                                planned.snapshot.blocking_wheel_regions.rects.len() as f64
                            }),
                        ),
                    ],
                );
                if matches!(event.phase, WheelPhase::kEnded | WheelPhase::kCancelled) {
                    self.wheel_blocked_on_main = None;
                }
                if did_scroll || self.dirty && was_scroll_active != self.scroll_active {
                    effects.push(Effect::RequestBeginFrame);
                }
            }
            Command::Resize(viewport) => {
                if let Some(mut snapshot) = self.snapshot.clone() {
                    snapshot.viewport = viewport;
                    snapshot.signature.2 = viewport;
                    self.snapshot = Some(snapshot.clone());
                    self.pending_snapshot = Some(snapshot);
                    self.ScheduleRaster(&mut effects);
                }
            }
            Command::Redraw => {
                self.dirty = true;
                effects.push(Effect::RequestBeginFrame);
            }
            Command::RasterReady(ready) => self.InstallRaster(ready, &mut effects)?,
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
                effects.push(Effect::BeginMainFrame(frame));
            }
            Command::DisplayBeginFrame(_) | Command::Stop => {}
        }
        Ok(effects)
    }

    fn ScheduleRaster(&mut self, effects: &mut Vec<Effect<V, F, M>>) {
        if self.raster_in_flight {
            return;
        }
        // RasterBundle is the retained pending-tree resource owner: its
        // LayerTileEngines and RasterEngines carry tile identities, pixels and
        // raster caches across updates. While Display owns the active
        // bundle and the raster owner owns the pending bundle, wait for
        // Display::Install to return the old active bundle.  Creating a third,
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
        let bundle = self
            .spare_bundle
            .take()
            .expect("spare raster bundle checked above");
        effects.push(Effect::Raster(RasterJob {
            snapshot,
            bundle,
            activation_target,
        }));
        self.raster_in_flight = true;
    }

    fn current_scroll_position(&self) -> Option<(u64, f64)> {
        self.planned.as_ref().and_then(|planned| {
            planned.root_scroll.map(|root| {
                (
                    root.id,
                    (root.committed + self.pending_scroll_delta).clamp(0.0, root.maximum),
                )
            })
        })
    }

    fn InstallRaster(
        &mut self,
        ready: RasterReady<V, M>,
        effects: &mut Vec<Effect<V, F, M>>,
    ) -> io::Result<()> {
        self.raster_in_flight = false;
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
                    browser_tracing::instant(
                        "raster",
                        "PendingTreeRasterRejected",
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
        if self.snapshot.as_ref().is_some_and(|current| {
            current.document != planned.snapshot.document
                || current.signature != planned.snapshot.signature
        }) {
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
        let root_scroll = planned.root_scroll;
        let previous_scroll = self.current_scroll_position();
        if let Some(root) = root_scroll {
            let desired = previous_scroll
                .filter(|(id, _)| *id == root.id)
                .map_or(root.committed, |(_, offset)| offset)
                .clamp(0.0, root.maximum);
            // Pending-tree raster completion is not by itself permission to
            // activate.  The tree must also cover the compositor's current
            // scroll position.  Otherwise replacing the active bundle here
            // snaps to the pending tree's committed position for one frame and
            // then jumps forward when the missing activation tiles arrive.
            // Chromium keeps the old active tree until all
            // required-for-activation tiles are ready; retain the same atomic
            // activation boundary here.
            if !apply_scroll_offset(&mut planned, desired) {
                browser_tracing::instant(
                    "raster",
                    "PendingTreeActivationDeferred",
                    &[
                        ("scroll_node", root.id as f64),
                        ("committed", root.committed),
                        ("desired", desired),
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
            self.committed_scroll = Some((root.id, root.committed));
            self.pending_scroll_delta = desired - root.committed;
        } else {
            self.committed_scroll = None;
            self.pending_scroll_delta = 0.0;
        }
        self.planned = Some(planned);
        effects.push(Effect::InstallRasterBundle(ready.bundle));
        // Bootstrap the second half of the fixed active/pending pair.  The
        // first Display::Install has no preceding active bundle to return;
        // every later install does.  Create exactly this one second owner,
        // then retain and exchange the same two bundles for the document's
        // lifetime.
        if !self.display_has_bundle {
            self.display_has_bundle = true;
            debug_assert!(self.spare_bundle.is_none());
            self.spare_bundle = Some(RasterBundle::default());
        }
        self.dirty = true;
        effects.push(Effect::RequestBeginFrame);
        self.ScheduleRaster(effects);
        Ok(())
    }

    pub fn Draw(&mut self, frame: F) -> Vec<Effect<V, F, M>> {
        vec![Effect::Submit {
            frame,
            scroll_active: self.scroll_active,
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
        if let Some(root) = self
            .planned
            .as_ref()
            .and_then(|planned| planned.root_scroll)
        {
            let desired = (root.committed + self.pending_scroll_delta).clamp(0.0, root.maximum);
            if let Some(planned) = &mut self.planned {
                let _ = apply_scroll_offset(planned, desired);
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
        let toolbar = self
            .toolbar_frame_builder
            .BuildFrame(&planned.toolbar)
            .map_err(io::Error::other)?
            .WithBeginFrameAck(ack);
        let mut content = if let Some(plan) = planned.content.as_ref() {
            Some(
                self.content_frame_builder
                    .BuildFrame(plan)
                    .map_err(io::Error::other)?
                    .WithBeginFrameAck(ack),
            )
        } else {
            self.content_frame_builder.Invalidate();
            None
        };
        let overlay = self
            .scroll_active
            .then(|| planned.RootOverlayScrollbarQuad())
            .flatten();
        let next_overlay_damage = overlay.as_ref().map(|(_, damage)| *damage);
        if let Some(frame) = content.take() {
            content = Some(match overlay {
                Some((quad, damage)) => frame.WithSolidColorQuad(quad, damage),
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
        let submitted = SubmittedFrame {
            viewport: planned.snapshot.viewport,
            toolbar,
            content,
            metadata: planned.snapshot.metadata.clone(),
        };
        self.dirty = false;
        Ok(vec![Effect::FrameProduced {
            frame,
            submitted,
            scroll_active: self.scroll_active,
        }])
    }
}

pub fn PlanAndRaster<V, M>(
    bundle: &mut RasterBundle,
    snapshot: ArtifactSnapshot<V, M>,
    activation_target: Option<(u64, f64)>,
) -> io::Result<PlannedFrame<V, M>>
where
    V: CompositorViewport,
    M: Clone,
{
    let mut trace = browser_tracing::span("raster", "RasterPendingTree");
    let toolbar_config = FrameConfig {
        viewport: PaintRect {
            x: 0.0,
            y: 0.0,
            width: snapshot.viewport.LogicalWidth(),
            height: snapshot.toolbar_height,
        },
        raster_scale: snapshot.viewport.Scale(),
        activation_scroll: None,
        frame_time: Some(snapshot.frame_time),
    };
    let toolbar_update = bundle
        .toolbar_tiles
        .UpdatePending(&snapshot.toolbar, toolbar_config)
        .map_err(|error| io::Error::new(io::ErrorKind::Unsupported, error.reason()))?;
    if toolbar_update.frame_plan.unsupported == Some(UnsupportedReason::TileBudgetExceeded) {
        return Err(io::Error::new(
            io::ErrorKind::OutOfMemory,
            "toolbar tile budget exceeded",
        ));
    }
    let source_root_scroll = snapshot
        .content
        .as_ref()
        .and_then(find_root_scroll_artifact);
    let raster_scroll = source_root_scroll.map(|root| (root.id, root.committed));
    let content = if let Some(artifact) = &snapshot.content {
        let content_update = bundle
            .content_tiles
            .UpdatePending(
                artifact,
                FrameConfig {
                    viewport: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: snapshot.viewport.LogicalWidth(),
                        height: snapshot.viewport.ContentHeight(snapshot.toolbar_height),
                    },
                    raster_scale: snapshot.viewport.Scale(),
                    activation_scroll: source_root_scroll.and_then(|root| {
                        activation_target
                            .filter(|(id, _)| *id == root.id)
                            .map(|(_, desired)| CompositorScrollOffset {
                                scroll_node_id: root.id,
                                translation_y: root.committed - desired,
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
    let toolbar_raster = bundle.toolbar_raster.prepare(
        &toolbar_update.frame_plan,
        &toolbar_update.raster_batch,
        snapshot.viewport.Width(),
        snapshot.viewport.ToolbarPixels(snapshot.toolbar_height),
    )?;
    bundle
        .toolbar_tiles
        .ApplyRasterResults(&toolbar_raster.completions);
    let toolbar_release = bundle
        .toolbar_tiles
        .ActivatePending()
        .ok_or_else(|| io::Error::other("toolbar tiles are not ready for activation"))?;
    let toolbar_released = bundle.toolbar_raster.release_resources(&toolbar_release)?;
    bundle
        .toolbar_tiles
        .AcknowledgeResourceRelease(&toolbar_released);
    let toolbar = bundle
        .toolbar_tiles
        .GetActiveFramePlan()
        .expect("activated toolbar plan")
        .clone();
    let content = if let Some(content) = &content {
        let raster = bundle.content_raster.prepare(
            &content.frame_plan,
            &content.raster_batch,
            snapshot.viewport.Width(),
            snapshot
                .viewport
                .Height()
                .saturating_sub(snapshot.viewport.ToolbarPixels(snapshot.toolbar_height)),
        )?;
        bundle.content_tiles.ApplyRasterResults(&raster.completions);
        let content_release = bundle
            .content_tiles
            .ActivatePending()
            .ok_or_else(|| io::Error::other("content tiles are not ready for activation"))?;
        let released = bundle.content_raster.release_resources(&content_release)?;
        bundle.content_tiles.AcknowledgeResourceRelease(&released);
        Some(
            bundle
                .content_tiles
                .GetActiveFramePlan()
                .expect("activated content plan")
                .clone(),
        )
    } else {
        None
    };
    let root_scroll = content.as_ref().and_then(find_root_scroll);
    trace.set("succeeded", 1.0);
    Ok(PlannedFrame {
        snapshot,
        toolbar,
        content,
        root_scroll,
        visual_scroll: root_scroll.map(|root| (root.id, root.committed)),
        raster_scroll,
    })
}

fn find_root_scroll_artifact(artifact: &Arc<PaintArtifact>) -> Option<RootScroll> {
    let mut best = None;
    for chunk in &artifact.chunks {
        let mut node = Some(&chunk.properties.transform);
        while let Some(transform) = node {
            if let Some(scroll) = &transform.scroll {
                if scroll.user_scrollable_vertical {
                    let maximum =
                        (scroll.contents_rect.height - scroll.container_rect.height).max(0.0);
                    let candidate = RootScroll {
                        id: scroll.id,
                        committed: (-transform.matrix.values[13]).clamp(0.0, maximum),
                        maximum,
                    };
                    if maximum > 0.0
                        && best.is_none_or(|old: RootScroll| candidate.maximum > old.maximum)
                    {
                        best = Some(candidate);
                    }
                }
            }
            node = transform.parent.as_ref();
        }
    }
    best
}

fn find_root_scroll(plan: &FramePlan) -> Option<RootScroll> {
    let mut best = None;
    for layer in &plan.layers {
        let mut node = Some(&layer.properties.transform);
        while let Some(transform) = node {
            if let Some(scroll) = &transform.scroll {
                if scroll.user_scrollable_vertical {
                    let maximum =
                        (scroll.contents_rect.height - scroll.container_rect.height).max(0.0);
                    let candidate = RootScroll {
                        id: scroll.id,
                        committed: (-transform.matrix.values[13]).clamp(0.0, maximum),
                        maximum,
                    };
                    if maximum > 0.0
                        && best.is_none_or(|old: RootScroll| candidate.maximum > old.maximum)
                    {
                        best = Some(candidate);
                    }
                }
            }
            node = transform.parent.as_ref();
        }
    }
    best
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

fn apply_scroll_offset<V, M>(frame: &mut PlannedFrame<V, M>, desired: f64) -> bool {
    let (Some(content), Some(root)) = (&mut frame.content, frame.root_scroll) else {
        return true;
    };
    let raster_base = frame
        .raster_scroll
        .filter(|(id, _)| *id == root.id)
        .map_or(root.committed, |(_, offset)| offset);
    let delta = (raster_base - desired) * content.config.raster_scale;
    // A compositor scroll transform and the tile set which covers it are one
    // activation transaction.  Applying the transform while retaining only
    // the preceding viewport's tiles exposes the transparent backing for a
    // frame (most visible below large carousel layers).  Keep the active
    // property state until every affected layer has drawable coverage.
    let active = content
        .layers
        .iter()
        .map(|layer| (layer.compositor_scroll, layer.tiles.clone()))
        .collect::<Vec<_>>();
    for layer in &mut content.layers {
        layer.compositor_scroll = if layer_has_scroll(&layer.properties.transform, root.id) {
            Some(CompositorScrollOffset {
                scroll_node_id: root.id,
                translation_y: raster_base - desired,
            })
        } else {
            None
        };
    }
    if !select_ready_scroll_tiles(content, root.id) {
        for (layer, (scroll, tiles)) in content.layers.iter_mut().zip(active) {
            layer.compositor_scroll = scroll;
            layer.tiles = tiles;
        }
        browser_tracing::instant(
            "raster",
            "ScrollVisualActivationDeferred",
            &[("scroll_node", root.id as f64), ("desired", desired)],
        );
        return false;
    }
    browser_tracing::instant(
        "input",
        "ScrollApplied",
        &[
            ("scroll_node", root.id as f64),
            ("committed", root.committed),
            ("offset_y", desired),
            ("desired", desired),
            ("delta_device", delta),
            ("compositor", 1.0),
        ],
    );
    frame.visual_scroll = Some((root.id, desired));
    true
}

fn select_ready_scroll_tiles(plan: &mut FramePlan, scroll_id: u64) -> bool {
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
        let selected: Vec<_> = wanted
            .iter()
            .filter(|tile| plan.TileIsReady(tile))
            .cloned()
            .collect();
        if selected.len() == wanted.len() && !wanted.is_empty() {
            // Activate a complete visible tiling in one property-tree state.
            plan.layers[layer_index].tiles = selected;
        } else {
            // Keep the complete preceding property state until every required
            // replacement is ready.  Mixing the new transform with the old
            // tile set leaves uncovered pixels even if some skewport tiles
            // are already available.
            complete = false;
            let layer_id = plan.layers[layer_index].id.0;
            browser_tracing::instant(
                "raster",
                "ScrollTileActivationDeferred",
                &[
                    ("layer", layer_id as f64),
                    ("wanted", wanted.len() as f64),
                    ("ready", selected.len() as f64),
                    ("interest", layer.interest_tiles.len() as f64),
                    ("bounds_complete", layer.bounds_are_complete as u8 as f64),
                    (
                        "has_raster_bounds",
                        layer.raster_content_bounds.is_some() as u8 as f64,
                    ),
                ],
            );
        }
    }
    complete
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
