//! Persistent CPU raster lanes. Each lane owns its raster caches, matching
//! Chromium's worker-local raster state while immutable PaintResources are
//! shared by Arc.
use crate::{
    convert::resources,
    layer_replay::{LayerReplay, PreparedLayer, RasterLayerState},
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

pub(crate) struct RasterJob {
    pub plan_index: usize,
    pub task: RasterTask,
    pub layer: Arc<PreparedLayer>,
    pub reuse: Vec<u8>,
    pub solid_color: Option<u32>,
}
pub(crate) struct RasterProduct {
    pub plan_index: usize,
    pub task: RasterTask,
    pub composition: RasterLayerState,
    pub rgba: Vec<u8>,
    pub bgra: Vec<u32>,
    pub row_support: crate::engine::RasterRowSupport,
}
struct Batch {
    frame: u64,
    trace_context: browser_tracing::Context,
    resources: Option<Arc<PaintResources>>,
    jobs: Arc<Mutex<VecDeque<RasterJob>>>,
    layers: Arc<BTreeMap<layer_tile::LayerId, Arc<PreparedLayer>>>,
    reply: mpsc::Sender<(u64, io::Result<Vec<RasterProduct>>, bool)>,
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
pub(crate) struct RasterWorkerPool {
    workers: Vec<Worker>,
    next_frame: u64,
}
pub(crate) struct PendingTiles {
    frame: u64,
    receiver: mpsc::Receiver<(u64, io::Result<Vec<RasterProduct>>, bool)>,
    outstanding: usize,
    jobs: Arc<Mutex<VecDeque<RasterJob>>>,
    error: Option<io::Error>,
    products: Vec<RasterProduct>,
}
impl PendingTiles {
    fn accept(&mut self, frame: u64, result: io::Result<Vec<RasterProduct>>, complete: bool) {
        if complete { self.outstanding -= 1; }
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
    /// Reprioritize queued SOON tasks that have become NOW, then wait only
    /// for those tile generations. Other accepted work retains its owner and
    /// is collected at subsequent CheckForCompletedTasks opportunities.
    pub fn collect_required(
        &mut self, required: &[(layer_tile::TileId, u64)],
    ) -> io::Result<Vec<RasterProduct>> {
        let mut wanted: std::collections::HashSet<_> = required.iter().copied().collect();
        {
            let mut jobs = self.jobs.lock()
                .map_err(|_| io::Error::other("raster task queue poisoned"))?;
            let queued = std::mem::take(&mut *jobs);
            let (now, soon): (VecDeque<_>, VecDeque<_>) = queued.into_iter()
                .partition(|job| wanted.contains(&(job.task.tile_id, job.task.generation)));
            jobs.extend(now);
            jobs.extend(soon);
        }
        for product in &self.products {
            wanted.remove(&(product.task.tile_id, product.task.generation));
        }
        let _wait = (!wanted.is_empty()).then(|| browser_tracing::span("raster", "RasterRequiredTileWait"));
        while self.outstanding != 0 {
            let response = if wanted.is_empty() {
                match self.receiver.try_recv() {
                    Ok(response) => response,
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.outstanding = 0;
                        self.error.get_or_insert_with(|| io::Error::other("raster worker exited before completing its batch"));
                        break;
                    }
                }
            } else {
                match self.receiver.recv() {
                    Ok(response) => response,
                    Err(_) => {
                        self.outstanding = 0;
                        self.error.get_or_insert_with(|| io::Error::other("raster worker exited before completing its batch"));
                        break;
                    }
                }
            };
            for product in response.1.as_ref().into_iter().flatten() {
                wanted.remove(&(product.task.tile_id, product.task.generation));
            }
            self.accept(response.0, response.1, response.2);
        }
        if let Some(error) = self.error.take() { return Err(error); }
        if !wanted.is_empty() { return Err(io::Error::other("required raster tile did not complete")); }
        Ok(std::mem::take(&mut self.products))
    }
    pub fn is_finished(&self) -> bool { self.outstanding == 0 }
    pub fn finish(mut self) -> io::Result<Vec<RasterProduct>> {
        let _wait = browser_tracing::span("raster", "RasterWorkerWait");
        // Drain every accepted batch even when another lane failed. No reply,
        // allocation or unfinished cache transaction can enter a later frame.
        while self.outstanding != 0 {
            match self.receiver.recv() {
                Ok((frame, result, complete)) => self.accept(frame, result, complete),
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
            match self.receiver.recv() {
                Ok((_, _, complete)) => { if complete { self.outstanding -= 1; } }
                Err(_) => break,
            }
        }
    }
}
impl RasterWorkerPool {
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
        jobs: Vec<RasterJob>,
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
            jobs: jobs.clone(),
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
    let memory_id =
        crate::engine::memory_diagnostics_enabled().then(crate::engine::next_memory_diagnostic_id);
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
            let documents = crate::layer_replay::document_records(&list);
            let mut replay = LayerReplay::for_worker(
                resources,
                documents,
                (*batch.layers).clone(),
                std::mem::take(&mut scratch),
            );
            let result = (|| {
                let products: Vec<RasterProduct> = Vec::new();
                let mut task_count = 0;
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
                    let product = if let Some(premul_rgba) = job.solid_color {
                        crate::solid::raster_product(
                            job.plan_index, &job.task, composition, premul_rgba, job.reuse,
                        )?
                    } else {
                        let rgba = replay.raster_tile_with_pixels(
                            &job.task, &mut images, &mut clips, job.reuse,
                        )?;
                        let row_support = crate::engine::tile_row_support(
                            &rgba, job.task.pixel_size.0 as usize, composition.white_backing,
                        );
                        let bgra = crate::engine::cached_bgra(&rgba);
                        RasterProduct {
                            plan_index: job.plan_index, task: job.task, composition,
                            rgba, bgra, row_support,
                        }
                    };
                    // TileManager publishes each completed task. A NOW tile
                    // must not wait behind the rest of an unrelated SOON batch.
                    let _ = reply.send((frame, Ok(vec![product]), false));
                    task_count += 1;
                }
                trace.set("raster_tasks", task_count as f64);
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
        if let Some(id) = memory_id {
            let product_capacity = result.as_ref().map_or(0, |products| {
                products
                    .iter()
                    .map(|p| p.rgba.capacity() + p.bgra.capacity() * std::mem::size_of::<u32>())
                    .sum::<usize>()
            });
            eprintln!("raster-memory-worker time_ns={} thread={:?} lane={} batch={} image_accounted={} clip_accounted={} halo_capacity={} thread_scratch={} completed_product_capacity={} glyphs={:?}",
                crate::engine::diagnostic_time_ns(), std::thread::current().id(), id, frame, images.resident_pixel_bytes(), clips.accounting().resident_requested_bytes,
                scratch.capacity(), skia::src::core::SkCanvas::RasterThreadScratchBytes(), product_capacity,
                skia::src::core::SkCanvas::RasterThreadGlyphMemoryDiagnostics());
        }
        let _ = reply.send((frame, result, true));
    }
    if let Some(id) = memory_id {
        eprintln!("raster-memory-worker-exit time_ns={} thread={:?} lane={}",
            crate::engine::diagnostic_time_ns(), std::thread::current().id(), id);
    }
}
