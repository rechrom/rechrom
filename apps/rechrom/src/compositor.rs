//! Renderer-side compositor state.
//!
//! The compositor is a message-driven state machine. It owns scroll/layer
//! state and produces effects, but deliberately knows nothing about threads,
//! channels, executors, or the final native display surface.
use crate::{display, engine::Viewport};
use interaction::input_event::{ScrollGranularity, WheelEvent, WheelPhase};
use layer_tile::{CompositorScrollOffset, FrameConfig, FramePlan, LayerTileEngine};
use paint::{
    paint_engine::{PaintArtifact, PaintRect},
    paint_property_tree::TransformPaintPropertyNode,
};
use renderer::layer_tile_renderer::LayerTileRenderer;
use std::{io, sync::Arc, time::Instant};

#[derive(Clone)]
pub(crate) struct ArtifactSnapshot {
    /// Scroll/property node ids are local to one Page document.  Carry the
    /// owning tab generation so equal numeric ids from another document can
    /// never inherit compositor deltas or resident-tree identity.
    pub document: (u64, u64),
    pub viewport: Viewport,
    pub signature: (u64, u64, Viewport),
    pub toolbar: Arc<PaintArtifact>,
    pub content: Option<Arc<PaintArtifact>>,
    pub drag_regions: Vec<crate::chrome::DragRegion>,
    pub async_root_scroll: bool,
    pub blocking_wheel_regions: rechrom::page::BlockingWheelEventRegions,
}

pub(crate) enum Command {
    Snapshot(ArtifactSnapshot),
    BeginFrame(crate::begin_frame_source::NativeBeginFrame),
    Wheel {
        event: WheelEvent,
        queued_at: Instant,
    },
    Resize(Viewport),
    Redraw,
    RasterReady(RasterReady),
    SpareBundle(RasterBundle),
    Stop,
}

pub(crate) enum Effect {
    Raster(RasterJob),
    Display(display::Message),
    RequestBeginFrame,
}

#[derive(Clone)]
pub(crate) struct PlannedFrame {
    pub(crate) snapshot: ArtifactSnapshot,
    pub(crate) toolbar: FramePlan,
    pub(crate) content: Option<FramePlan>,
    root_scroll: Option<RootScroll>,
    /// Current impl-side position carried with the immutable compositor
    /// frame. The root overlay scrollbar reads this directly, so its thumb
    /// advances in the same compositor transaction as page pixels.
    visual_scroll: Option<(u64, f64)>,
    /// Scroll position baked into tile selection/root placement. It may be
    /// newer than the Page property's committed position.
    raster_scroll: Option<(u64, f64)>,
}

#[derive(Default)]
pub(crate) struct RasterBundle {
    pub(crate) toolbar_tiles: LayerTileEngine,
    pub(crate) content_tiles: LayerTileEngine,
    pub(crate) toolbar_renderer: LayerTileRenderer,
    pub(crate) content_renderer: LayerTileRenderer,
}

pub(crate) struct RasterJob {
    pub(crate) snapshot: ArtifactSnapshot,
    pub(crate) bundle: RasterBundle,
    pub(crate) activation_target: Option<(u64, f64)>,
    pub(crate) prepaint_target: Option<(u64, f64)>,
}

pub(crate) struct RasterReady {
    pub(crate) result: io::Result<PlannedFrame>,
    pub(crate) bundle: RasterBundle,
}

#[derive(Clone, Copy)]
struct RootScroll {
    id: u64,
    committed: f64,
    maximum: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct OverlayScrollbar {
    pub(crate) offset: f64,
    pub(crate) maximum: f64,
    pub(crate) viewport_length: f64,
}

impl PlannedFrame {
    pub(crate) fn RootOverlayScrollbar(&self) -> Option<OverlayScrollbar> {
        let root = self.root_scroll?;
        let offset = self
            .visual_scroll
            .filter(|(id, _)| *id == root.id)
            .map_or(root.committed, |(_, value)| value)
            .clamp(0.0, root.maximum);
        Some(OverlayScrollbar {
            offset,
            maximum: root.maximum,
            viewport_length: self.snapshot.viewport.content_height(),
        })
    }
}

pub(crate) struct Compositor {
    spare_bundle: Option<RasterBundle>,
    display_has_bundle: bool,
    raster_in_flight: bool,
    pending_snapshot: Option<ArtifactSnapshot>,
    pending_activation_target: Option<(u64, f64)>,
    pending_prepaint_target: Option<(u64, f64)>,
    snapshot: Option<ArtifactSnapshot>,
    planned: Option<PlannedFrame>,
    committed_scroll: Option<(u64, f64)>,
    pending_scroll_delta: f64,
    scroll_active: bool,
    wheel_blocked_on_main: Option<bool>,
    dirty: bool,
}

impl Default for Compositor {
    fn default() -> Self {
        Self {
            spare_bundle: Some(RasterBundle::default()),
            display_has_bundle: false,
            raster_in_flight: false,
            pending_snapshot: None,
            pending_activation_target: None,
            pending_prepaint_target: None,
            snapshot: None,
            planned: None,
            committed_scroll: None,
            pending_scroll_delta: 0.0,
            scroll_active: false,
            wheel_blocked_on_main: None,
            dirty: true,
        }
    }
}

impl Compositor {
    pub(crate) fn Handle(&mut self, command: Command) -> io::Result<Vec<Effect>> {
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
                    // Paint property node ids are scoped to a document.  A
                    // newly created Page commonly starts its root scroll node
                    // at the same small numeric id as the preceding Page; that
                    // is not identity continuity.  Keep the old active pixels
                    // until the replacement tree is ready, but do not carry
                    // any of its scroll state into the replacement.
                    self.committed_scroll = None;
                    self.pending_scroll_delta = 0.0;
                    self.pending_activation_target = None;
                    self.pending_prepaint_target = None;
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
                    self.pending_prepaint_target = self.pending_activation_target;
                    self.ScheduleRaster(&mut effects);
                }
            }
            Command::Wheel { event, queued_at } => {
                let _input = browser_tracing::scope(browser_tracing::Context {
                    target_id: 1,
                    input_id: browser_tracing::instant_id(queued_at),
                    ..Default::default()
                });
                if event.position.y < crate::engine::TOOLBAR_HEIGHT
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
                                event.position.y - crate::engine::TOOLBAR_HEIGHT
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
                                // PictureLayerTilingSet predicts roughly one
                                // second of scroll motion and caps the skewport
                                // at 2000 screen pixels.  This target expands
                                // raster interest only; it never changes this
                                // frame's draw transform or logical offset.
                                let scale = self
                                    .snapshot
                                    .as_ref()
                                    .map_or(1.0, |snapshot| snapshot.viewport.scale.max(1.0));
                                let limit = 2000.0 / scale;
                                let projected = (event.delta.y * 60.0).clamp(-limit, limit);
                                let raster_target = (desired + projected).clamp(0.0, root.maximum);
                                self.pending_snapshot = self.snapshot.clone();
                                self.pending_activation_target = Some((root.id, desired));
                                self.pending_prepaint_target = Some((root.id, raster_target));
                                browser_tracing::instant(
                                    "raster",
                                    "ScrollRasterPrediction",
                                    &[
                                        ("desired", desired),
                                        ("predicted", raster_target),
                                        ("limit", limit),
                                    ],
                                );
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
            Command::BeginFrame(_) | Command::Stop => {}
        }
        Ok(effects)
    }

    fn ScheduleRaster(&mut self, effects: &mut Vec<Effect>) {
        if self.raster_in_flight {
            return;
        }
        // RasterBundle is the retained pending-tree resource owner: its
        // LayerTileEngines and renderers carry tile identities, pixels and
        // worker-local caches across updates.  While Display owns the active
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
        let prepaint_target = self.pending_prepaint_target.take();
        let bundle = self
            .spare_bundle
            .take()
            .expect("spare raster bundle checked above");
        effects.push(Effect::Raster(RasterJob {
            snapshot,
            bundle,
            activation_target,
            prepaint_target,
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

    fn InstallRaster(&mut self, ready: RasterReady, effects: &mut Vec<Effect>) -> io::Result<()> {
        self.raster_in_flight = false;
        let mut planned = match ready.result {
            Ok(planned) => planned,
            Err(error) => {
                self.spare_bundle = Some(ready.bundle);
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
                self.pending_prepaint_target = Some((root.id, desired));
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
        effects.push(Effect::Display(display::Message::Install(ready.bundle)));
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

    pub(crate) fn Draw(
        &mut self,
        frame: crate::begin_frame_source::NativeBeginFrame,
    ) -> Vec<Effect> {
        vec![Effect::Display(display::Message::Submit {
            frame,
            scroll_active: self.scroll_active,
        })]
    }

    pub(crate) fn Prepare(
        &mut self,
        frame: crate::begin_frame_source::NativeBeginFrame,
    ) -> Vec<Effect> {
        let _frame = browser_tracing::scope(browser_tracing::Context {
            target_id: 1,
            source_id: frame.source_id,
            frame_id: frame.sequence_number,
            ..Default::default()
        });
        if !self.dirty {
            return Vec::new();
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
            return Vec::new();
        };
        self.dirty = false;
        vec![Effect::Display(display::Message::Prepare {
            frame,
            planned,
            scroll_active: self.scroll_active,
        })]
    }
}

pub(crate) fn PlanAndRaster(
    bundle: &mut RasterBundle,
    snapshot: ArtifactSnapshot,
    activation_target: Option<(u64, f64)>,
    prepaint_target: Option<(u64, f64)>,
) -> io::Result<PlannedFrame> {
    let mut trace = browser_tracing::span("raster", "RasterPendingTree");
    let toolbar_config = FrameConfig {
        viewport: PaintRect {
            x: 0.0,
            y: 0.0,
            width: snapshot.viewport.logical_width(),
            height: crate::engine::TOOLBAR_HEIGHT,
        },
        raster_scale: snapshot.viewport.scale,
        activation_scroll: None,
        prepaint_scroll: None,
    };
    bundle.toolbar_tiles.SetFrameConfig(toolbar_config);
    bundle
        .toolbar_tiles
        .Update(&snapshot.toolbar)
        .map_err(|error| io::Error::new(io::ErrorKind::Unsupported, error.reason()))?;
    let toolbar = bundle
        .toolbar_tiles
        .GetFramePlan()
        .expect("toolbar plan")
        .clone();
    let source_root_scroll = snapshot
        .content
        .as_ref()
        .and_then(find_root_scroll_artifact);
    let raster_scroll = source_root_scroll.map(|root| (root.id, root.committed));
    let content = if let Some(artifact) = &snapshot.content {
        bundle.content_tiles.SetFrameConfig(FrameConfig {
            viewport: PaintRect {
                x: 0.0,
                y: 0.0,
                width: snapshot.viewport.logical_width(),
                height: snapshot.viewport.content_height(),
            },
            raster_scale: snapshot.viewport.scale,
            activation_scroll: source_root_scroll.and_then(|root| {
                activation_target
                    .filter(|(id, _)| *id == root.id)
                    .map(|(_, desired)| CompositorScrollOffset {
                        scroll_node_id: root.id,
                        translation_y: root.committed - desired,
                    })
            }),
            prepaint_scroll: source_root_scroll.and_then(|root| {
                prepaint_target
                    .filter(|(id, _)| *id == root.id)
                    .map(|(_, desired)| CompositorScrollOffset {
                        scroll_node_id: root.id,
                        translation_y: root.committed - desired,
                    })
            }),
        });
        bundle
            .content_tiles
            .Update(artifact)
            .map_err(|error| io::Error::new(io::ErrorKind::Unsupported, error.reason()))?;
        Some(
            bundle
                .content_tiles
                .GetFramePlan()
                .expect("content plan")
                .clone(),
        )
    } else {
        None
    };
    bundle.toolbar_renderer.prepare(
        &toolbar,
        snapshot.viewport.width,
        snapshot.viewport.toolbar_pixels(),
    )?;
    if let Some(content) = &content {
        bundle.content_renderer.prepare(
            content,
            snapshot.viewport.width,
            snapshot
                .viewport
                .height
                .saturating_sub(snapshot.viewport.toolbar_pixels()),
        )?;
    }
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

fn apply_scroll_offset(frame: &mut PlannedFrame, desired: f64) -> bool {
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
