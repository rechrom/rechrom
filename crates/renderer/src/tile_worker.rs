//! Persistent CPU raster lanes. Each lane owns its raster caches, matching
//! Chromium's worker-local raster state while immutable PaintResources are
//! shared by Arc.
use crate::{
    convert::resources,
    layer_replay::{LayerComposition, LayerReplay, PreparedLayer},
};
use layer_tile::RasterTask;
use layoutng_assembly::fragment_tree::PaintResources;
use paint::paint_engine::PaintArtifact;
use skia::src::core::SkCanvas::RasterImageCache;
use std::{
    any::Any,
    collections::{BTreeMap, VecDeque},
    io,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub(crate) struct TileJob {
    pub plan_index: usize,
    pub task: RasterTask,
    pub layer: Arc<PreparedLayer>,
    pub reuse: Vec<u8>,
    pub solid_color: Option<u32>,
}
pub(crate) struct TileProduct {
    pub plan_index: usize,
    pub task: RasterTask,
    pub composition: LayerComposition,
    pub rgba: Vec<u8>,
    pub bgra: Vec<u32>,
    pub row_support: crate::layer_raster::TileRowSupport,
    pub call_time: std::time::Duration,
    pub support_time: std::time::Duration,
}
struct Batch {
    frame: u64,
    trace_context: browser_tracing::Context,
    resources: Option<Arc<PaintResources>>,
    jobs: Arc<Mutex<VecDeque<TileJob>>>,
    layers: Arc<BTreeMap<layer_tile::LayerId, Arc<PreparedLayer>>>,
    reply: mpsc::Sender<(u64, io::Result<Vec<TileProduct>>)>,
}
struct Worker {
    sender: Option<mpsc::Sender<Batch>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        // Closing the queue finishes any accepted work before shutdown. Join
        // also releases the lane's caches and thread-local raster scratch.
        self.sender.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
pub(crate) struct TileWorkerPool {
    workers: Vec<Worker>,
    next_frame: u64,
}
pub(crate) struct PendingTiles {
    frame: u64,
    receiver: mpsc::Receiver<(u64, io::Result<Vec<TileProduct>>)>,
    outstanding: usize,
    error: Option<io::Error>,
    products: Vec<TileProduct>,
}
impl PendingTiles {
    fn accept(&mut self, frame: u64, result: io::Result<Vec<TileProduct>>) {
        self.outstanding -= 1;
        if frame != self.frame {
            self.error
                .get_or_insert_with(|| io::Error::other("raster worker returned another frame"));
        } else {
            match result {
                Ok(mut ready) => self.products.append(&mut ready),
                Err(error) => {
                    self.error.get_or_insert(error);
                }
            }
        }
    }
    /// Collect a completed background batch without waiting on the frame
    /// thread. None means workers still own at least one accepted batch.
    pub fn try_finish(&mut self) -> Option<io::Result<Vec<TileProduct>>> {
        use std::sync::mpsc::TryRecvError;
        while self.outstanding != 0 {
            match self.receiver.try_recv() {
                Ok((frame, result)) => self.accept(frame, result),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    self.error.get_or_insert_with(|| {
                        io::Error::other("raster worker exited before completing its batch")
                    });
                    self.outstanding = 0;
                }
            }
        }
        Some(if let Some(error) = self.error.take() {
            Err(error)
        } else {
            Ok(std::mem::take(&mut self.products))
        })
    }
    pub fn finish(mut self) -> io::Result<Vec<TileProduct>> {
        let _wait = browser_tracing::span("raster", "RasterWorkerWait");
        // Drain every accepted batch even when another lane failed. No reply,
        // allocation or unfinished cache transaction can enter a later frame.
        while self.outstanding != 0 {
            match self.receiver.recv() {
                Ok((frame, result)) => self.accept(frame, result),
                Err(_) => {
                    self.error.get_or_insert_with(|| {
                        io::Error::other("raster worker exited before completing its batch")
                    });
                    self.outstanding = 0;
                }
            }
        }
        if let Some(error) = self.error.take() {
            Err(error)
        } else {
            Ok(std::mem::take(&mut self.products))
        }
    }
}
impl Drop for PendingTiles {
    fn drop(&mut self) {
        while self.outstanding != 0 {
            if self.receiver.recv().is_err() {
                break;
            }
            self.outstanding -= 1;
        }
    }
}
impl TileWorkerPool {
    pub fn new(clip_limit: usize) -> io::Result<Self> {
        // content/browser/gpu/compositor_util.cc::NumberOfRendererRasterThreads:
        // half the logical processors, clamped to [1, 4].
        let count = thread::available_parallelism()
            .map_or(1, usize::from)
            .saturating_div(2)
            .clamp(1, 4);
        Self::new_with_count(clip_limit, 16 * 1024 * 1024, count)
    }
    /// SOON work uses a distinct single lane so it cannot occupy the NOW
    /// workers which the next frame may need at its display deadline.
    pub fn new_prepaint(clip_limit: usize) -> io::Result<Self> {
        Self::new_with_count(clip_limit, 16 * 1024 * 1024, 1)
    }
    fn new_with_count(clip_limit: usize, image_limit: usize, count: usize) -> io::Result<Self> {
        let mut workers = Vec::with_capacity(count);
        for index in 0..count {
            let (sender, receiver) = mpsc::channel::<Batch>();
            let thread = thread::Builder::new()
                .name(format!("cpu-tile-raster-{index}"))
                .stack_size(8 * 1024 * 1024)
                .spawn(move || worker_loop(receiver, clip_limit, image_limit))?;
            workers.push(Worker {
                sender: Some(sender),
                thread: Some(thread),
            });
        }
        Ok(Self {
            workers,
            next_frame: 0,
        })
    }
    pub fn available(&self) -> bool {
        !self.workers.is_empty()
    }
    pub fn submit(
        &mut self,
        resources: Option<Arc<PaintResources>>,
        jobs: Vec<TileJob>,
    ) -> PendingTiles {
        self.next_frame = self.next_frame.wrapping_add(1);
        let frame = self.next_frame;
        let (reply, receiver) = mpsc::channel();
        // CategorizedWorkerPool workers pull individual ready raster tasks.
        // A shared queue gives the local lanes the same dynamic balancing:
        // expensive image tiles cannot strand one pre-partitioned bucket.
        let layers: Arc<BTreeMap<layer_tile::LayerId, Arc<PreparedLayer>>> = Arc::new(
            jobs.iter()
                .map(|job| (job.task.layer_id, job.layer.clone()))
                .collect(),
        );
        let jobs = Arc::new(Mutex::new(VecDeque::from(jobs)));
        let mut pending = PendingTiles {
            frame,
            receiver,
            outstanding: 0,
            error: None,
            products: Vec::new(),
        };
        for worker in &self.workers {
            let batch = Batch {
                frame,
                trace_context: browser_tracing::context(),
                resources: resources.clone(),
                jobs: jobs.clone(),
                layers: layers.clone(),
                reply: reply.clone(),
            };
            if worker
                .sender
                .as_ref()
                .expect("live worker queue")
                .send(batch)
                .is_err()
            {
                pending
                    .error
                    .get_or_insert_with(|| io::Error::other("raster worker queue closed"));
            } else {
                pending.outstanding += 1;
            }
        }
        // Only accepted batches hold reply senders. A worker panic therefore
        // disconnects recv after the other lanes finish instead of hanging.
        drop(reply);
        pending
    }
}
fn panic_message(value: Box<dyn Any + Send>) -> String {
    if let Some(message) = value.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = value.downcast_ref::<&str>() {
        (*message).to_owned()
    } else {
        "non-string panic payload".to_owned()
    }
}
fn worker_loop(receiver: mpsc::Receiver<Batch>, clip_limit: usize, image_limit: usize) {
    let mut scratch = Vec::new();
    // Decoded sources and mip products remain warm on the same persistent
    // raster lane. Their catalog owners are immutable Arcs.
    let mut images = RasterImageCache::with_pixel_byte_limit(image_limit);
    let mut clips = skia::RasterClipProductCache::with_byte_limit(clip_limit);
    while let Ok(batch) = receiver.recv() {
        let frame = batch.frame;
        let reply = batch.reply.clone();
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let _scope = browser_tracing::scope(batch.trace_context);
            let mut trace = browser_tracing::span("raster", "RasterWorkerBatch");
            let list = PaintArtifact {
                resources: batch.resources,
                ..Default::default()
            };
            // One borrowed immutable resource directory per batch. Image/font
            // owners and all command payloads are shared Arcs, never byte copies.
            let resources = resources(&list);
            let mut replay = LayerReplay::for_worker(
                resources,
                (*batch.layers).clone(),
                std::mem::take(&mut scratch),
            );
            let result = (|| {
                let mut products = Vec::new();
                let profile = std::env::var_os("BROWSER_PROFILE_TILES").is_some();
                loop {
                    let Some(job) = batch
                        .jobs
                        .lock()
                        .map_err(|_| io::Error::other("raster task queue poisoned"))?
                        .pop_front()
                    else {
                        break;
                    };
                    let mut trace = browser_tracing::span("raster", "RasterTile");
                    trace.set("tile_id", job.task.tile_id.0 as f64);
                    trace.set("layer_id", job.task.layer_id.0 as f64);
                    let composition = replay
                        .composition(job.task.layer_id)
                        .ok_or_else(|| io::Error::other("worker job has no prepared layer"))?;
                    if let Some(premul_rgba) = job.solid_color {
                        products.push(crate::layer_raster::layer_solid::raster_product(
                            job.plan_index,
                            &job.task,
                            composition,
                            premul_rgba,
                            job.reuse,
                            profile,
                        )?);
                        continue;
                    }
                    let started = profile.then(std::time::Instant::now);
                    let rgba = replay.raster_tile_with_pixels(
                        &job.task,
                        &mut images,
                        &mut clips,
                        job.reuse,
                    )?;
                    let call_time = started.map_or(std::time::Duration::ZERO, |s| s.elapsed());
                    let started = profile.then(std::time::Instant::now);
                    let row_support = crate::layer_raster::tile_row_support(
                        &rgba,
                        job.task.pixel_size.0 as usize,
                        composition.white_backing,
                    );
                    let bgra = crate::layer_raster::cached_bgra(&rgba);
                    products.push(TileProduct {
                        plan_index: job.plan_index,
                        task: job.task,
                        composition,
                        rgba,
                        bgra,
                        row_support,
                        call_time,
                        support_time: started.map_or(std::time::Duration::ZERO, |s| s.elapsed()),
                    });
                }
                trace.set("raster_tasks", products.len() as f64);
                Ok(products)
            })();
            scratch = replay.take_scratch();
            result
        }));
        let result = match outcome {
            Ok(result) => result,
            Err(panic) => {
                scratch.clear();
                images = RasterImageCache::with_pixel_byte_limit(image_limit);
                clips = skia::RasterClipProductCache::with_byte_limit(clip_limit);
                Err(io::Error::other(format!(
                    "CPU tile raster worker panicked: {}",
                    panic_message(panic)
                )))
            }
        };
        let _ = reply.send((frame, result));
    }
}
