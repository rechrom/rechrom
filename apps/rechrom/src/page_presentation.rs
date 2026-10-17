//! Host assembly of independent Page compositors. No Page sees another Page's
//! recording, scroll state, tile owner, raster engine, or resource completions.
//! Correspondence: cc::LayerTreeHost/LayerTreeHostImpl own each producer's
//! active/pending trees and TileManager. LayerTreeFrameSink submits completed
//! frames; ClientResourceProvider exports read leases independently of engines.
use crate::{begin_frame_source::NativeBeginFrame, chrome::DragRegion, engine::Viewport};
use compositor::{MainThreadEvent, RasterBundle};
use std::{collections::VecDeque, io, sync::Arc, time::Instant};

#[cfg(test)]
use std::{collections::BTreeMap, sync::mpsc};

pub(crate) type Document = (u64, u64);
pub(crate) type PageSnapshot = compositor::ArtifactSnapshot<Viewport, ()>;
pub(crate) type PageFrame = compositor::SubmittedFrame<Viewport, ()>;
pub(crate) type PageCommand = compositor::Command<Viewport, NativeBeginFrame, ()>;
type PageEffect = compositor::Effect<Viewport, NativeBeginFrame, ()>;
pub(crate) type RasterJob = compositor::RasterJob<Viewport, ()>;
type RasterReady = compositor::RasterReady<Viewport, ()>;
type PageCompositor = compositor::CompositorEngine<Viewport, NativeBeginFrame, ()>;
pub(crate) enum RasterWork {
    Raster {
        job: RasterJob,
        completion_owner: RasterSender,
    },
    Release(RasterReady),
    Retire {
        compositor: PageCompositor,
        active: Option<RasterBundle>,
    },
}

/// A queued completion keeps its producer executor alive. If the Page closes
/// before this message is consumed, destruction returns to that executor too.
/// The window coordinator never joins or destroys another Page's worker pool.
pub(crate) struct PageRasterReady {
    ready: Option<RasterReady>,
    owner: RasterSender,
}
impl PageRasterReady {
    pub(crate) fn take(mut self) -> RasterReady {
        self.ready
            .take()
            .expect("unconsumed Page raster completion")
    }
}
impl Drop for PageRasterReady {
    fn drop(&mut self) {
        if let Some(ready) = self.ready.take() {
            let _ = self.owner.send(RasterWork::Release(ready));
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct Placement {
    pub document: Document,
    pub destination: compositor::DeviceRect,
}
#[derive(Clone, PartialEq)]
pub(crate) struct Scene {
    pub viewport: Viewport,
    pub placements: Vec<Placement>,
    pub live_documents: Vec<Document>,
    pub metadata: Vec<DragRegion>,
}
#[cfg(test)]
pub(crate) struct Snapshot {
    pub scene: Scene,
    pub pages: Vec<PageSnapshot>,
}

#[cfg(test)]
pub(crate) enum Command {
    Snapshot(Snapshot),
    InputBeginFrame(NativeBeginFrame),
    BeginFrame(NativeBeginFrame),
    DisplayBeginFrame(NativeBeginFrame),
    MainFrameComplete {
        source_id: u64,
        sequence_number: u64,
    },
    Wheel {
        event: interaction::input_event::WheelEvent,
        queued_at: Instant,
        samples: usize,
    },
    Resize(Viewport),
    Redraw,
    RasterReady(Document, PageRasterReady),
    Stop,
}
pub(crate) enum Effect {
    #[cfg(test)]
    Raster {
        sender: RasterSender,
        job: RasterJob,
    },
    FrameProduced {
        frame: NativeBeginFrame,
        submitted: PageFrame,
        resources: raster::RasterResourceSnapshot,
        scroll_active: bool,
    },
    Submit {
        frame: NativeBeginFrame,
        scroll_active: bool,
    },
    #[cfg(test)]
    Scene(Scene),
    MainThread(MainThreadEvent<NativeBeginFrame>),
    RequestBeginFrame,
    VisualDamage,
}
pub(crate) struct PagePresentation {
    compositor: PageCompositor,
    raster: RasterSender,
    active: Option<RasterBundle>,
}
#[derive(Clone)]
pub(crate) struct RasterSender(Arc<RasterDispatch>);
struct RasterDispatch {
    document: Document,
    executor: Arc<dyn executor::Executor>,
    reply: Arc<dyn Fn(PageRasterReady) + Send + Sync>,
}
impl RasterSender {
    pub(crate) fn send(&self, work: RasterWork) -> io::Result<()> {
        let dispatch = self.0.clone();
        self.0.executor.execute(Box::new(move || {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: if dispatch.document.0 == 0 { 2 } else { 1 },
                ..Default::default()
            });
            match work {
                RasterWork::Release(ready) => drop(ready),
                RasterWork::Retire { compositor, active } => drop((compositor, active)),
                RasterWork::Raster {
                    mut job,
                    completion_owner,
                } => {
                    job.bundle.raster.SetExecutor(dispatch.executor.clone());
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        compositor::PlanAndRaster(
                            &mut job.bundle,
                            job.snapshot,
                            job.activation_target,
                        )
                    }))
                    .unwrap_or_else(|payload| Err(crate::page_error::panic_error(payload)));
                    (dispatch.reply)(PageRasterReady {
                        ready: Some(RasterReady {
                            result,
                            bundle: job.bundle,
                        }),
                        owner: completion_owner,
                    });
                }
            }
        }))
    }
}
impl PagePresentation {
    #[cfg(test)]
    pub(crate) fn new(
        document: Document,
        reply: Arc<dyn Fn(PageRasterReady) + Send + Sync>,
    ) -> io::Result<Self> {
        Self::with_threads(document, reply, &browser::page::PageThreads::default())
    }
    pub(crate) fn with_threads(
        document: Document,
        reply: Arc<dyn Fn(PageRasterReady) + Send + Sync>,
        threads: &browser::page::PageThreads,
    ) -> io::Result<Self> {
        let executor = if threads.raster {
            threads.executor.clone()
        } else {
            executor::inline()
        };
        let raster = RasterSender(Arc::new(RasterDispatch {
            document,
            executor,
            reply,
        }));
        browser_tracing::instant(
            "frame",
            "PageCompositorCreated",
            &[
                ("view_id", document.0 as f64),
                ("document_generation", document.1 as f64),
            ],
        );
        Ok(Self {
            compositor: Default::default(),
            raster,
            active: None,
        })
    }
}

impl Drop for PagePresentation {
    fn drop(&mut self) {
        let compositor = std::mem::take(&mut self.compositor);
        let active = self.active.take();
        let _ = self.raster.send(RasterWork::Retire { compositor, active });
    }
}

#[cfg(test)]
#[path = "page_presentation_legacy.rs"]
mod legacy;
#[cfg(test)]
pub(crate) use legacy::WindowCompositor;

impl PagePresentation {
    pub(crate) fn Handle(&mut self, command: PageCommand) -> io::Result<Vec<Effect>> {
        let effects = self.compositor.Handle(command)?;
        self.effects(effects)
    }
    pub(crate) fn Prepare(&mut self, frame: NativeBeginFrame) -> io::Result<Vec<Effect>> {
        let mut result = Vec::new();
        if let Some(active) = &mut self.active {
            let ready = active
                .raster
                .PollCompletedRaster()
                .map_err(crate::page_error::raster_error)?;
            active.tiles.ApplyRasterResults(&ready);
            if !ready.is_empty() {
                result.extend(self.Handle(PageCommand::ActiveRasterReady(ready))?);
            }
        }
        let effects = self.compositor.Prepare(frame)?;
        result.extend(self.effects(effects)?);
        Ok(result)
    }
    fn effects(&mut self, effects: Vec<PageEffect>) -> io::Result<Vec<Effect>> {
        let mut pending: VecDeque<_> = effects.into();
        let mut output = Vec::new();
        while let Some(effect) = pending.pop_front() {
            match effect {
                PageEffect::Raster(job) => {
                    self.raster
                        .send(RasterWork::Raster {
                            job,
                            completion_owner: self.raster.clone(),
                        })
                        .map_err(|_| io::Error::other("Page raster executor stopped"))?;
                }
                PageEffect::InstallRasterBundle(bundle) => {
                    if let Some(previous) = self.active.replace(bundle) {
                        pending.extend(self.compositor.Handle(PageCommand::SpareBundle(previous))?);
                    }
                }
                PageEffect::FrameProduced {
                    frame,
                    submitted,
                    scroll_active,
                } => {
                    let active = self
                        .active
                        .as_ref()
                        .ok_or_else(|| io::Error::other("Page has no raster owner"))?;
                    let resources = active.raster.ExportResources(
                        submitted
                            .content
                            .iter()
                            .flat_map(|frame| frame.resource_list.iter().map(|r| r.tile_id)),
                    );
                    browser_tracing::instant(
                        "frame",
                        "PageCompositorFrameProduced",
                        &[
                            ("view_id", submitted.provenance.document.0 as f64),
                            (
                                "document_generation",
                                submitted.provenance.document.1 as f64,
                            ),
                            (
                                "sequence",
                                submitted.provenance.content.map_or(0, |p| p.sequence) as f64,
                            ),
                        ],
                    );
                    output.push(Effect::FrameProduced {
                        frame,
                        submitted,
                        resources,
                        scroll_active,
                    });
                }
                PageEffect::MainThread(event) => output.push(Effect::MainThread(event)),
                PageEffect::RequestBeginFrame => output.push(Effect::RequestBeginFrame),
                PageEffect::VisualDamage => output.push(Effect::VisualDamage),
                PageEffect::Submit {
                    frame,
                    scroll_active,
                } => output.push(Effect::Submit {
                    frame,
                    scroll_active,
                }),
            }
        }
        Ok(output)
    }
}
