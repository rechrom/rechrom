//! Engine-independent, opt-in wall-clock tracing. Collection never calls Page.
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, VecDeque},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        mpsc, Arc, Mutex, OnceLock,
    },
    time::Instant,
};

static ACTIVE: AtomicU64 = AtomicU64::new(0);
static IDS: AtomicU64 = AtomicU64::new(1);
static THREADS: AtomicU64 = AtomicU64::new(1);
static ORIGIN: OnceLock<Instant> = OnceLock::new();
fn origin() -> Instant {
    *ORIGIN.get_or_init(Instant::now)
}
pub fn instant_id(t: Instant) -> u64 {
    let o = origin();
    if let Some(d) = t.checked_duration_since(o) {
        (d.as_nanos().min(((1u64 << 63) - 2) as u128) as u64).saturating_add(1)
    } else {
        (1u64 << 63)
            | (o.duration_since(t)
                .as_nanos()
                .min(((1u64 << 63) - 1) as u128) as u64)
    }
}
pub fn next_id() -> u64 {
    IDS.fetch_add(1, Ordering::Relaxed)
}
pub fn enabled() -> bool {
    ACTIVE.load(Ordering::Relaxed) != 0
}

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Context {
    pub target_id: u64,
    #[serde(default)]
    pub source_id: u64,
    pub frame_id: u64,
    pub input_id: u64,
    pub task_id: u64,
}
struct Local {
    context: Context,
    parent: u64,
    thread_id: u64,
    session: Option<Arc<Recording>>,
    buffer: Option<Arc<ThreadBuffer>>,
}
impl Default for Local {
    fn default() -> Self {
        Self {
            context: Context::default(),
            parent: 0,
            thread_id: THREADS.fetch_add(1, Ordering::Relaxed),
            session: None,
            buffer: None,
        }
    }
}
thread_local! { static LOCAL: RefCell<Local> = RefCell::new(Local::default()); }
pub fn context() -> Context {
    LOCAL.with(|l| l.borrow().context)
}
pub struct ScopeGuard(Context, std::marker::PhantomData<std::rc::Rc<()>>);
pub fn scope(mut c: Context) -> ScopeGuard {
    LOCAL.with(|l| {
        let mut l = l.borrow_mut();
        let old = l.context;
        if c.target_id == 0 {
            c.target_id = old.target_id;
        }
        if c.source_id == 0 {
            c.source_id = old.source_id;
        }
        if c.frame_id == 0 {
            c.frame_id = old.frame_id;
        }
        if c.input_id == 0 {
            c.input_id = old.input_id;
        }
        if c.task_id == 0 {
            c.task_id = old.task_id;
        }
        l.context = c;
        ScopeGuard(old, std::marker::PhantomData)
    })
}
impl Drop for ScopeGuard {
    fn drop(&mut self) {
        LOCAL.with(|l| l.borrow_mut().context = self.0);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StartOptions {
    pub max_events: usize,
    pub target_id: u64,
}
impl Default for StartOptions {
    fn default() -> Self {
        Self {
            max_events: 100_000,
            target_id: 0,
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Target {
    pub id: u64,
    pub title: String,
    pub url: String,
}
#[derive(Default)]
struct Service {
    recordings: VecDeque<Arc<Recording>>,
    targets: BTreeMap<u64, Target>,
}
fn service() -> &'static Mutex<Service> {
    static S: OnceLock<Mutex<Service>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Service::default()))
}
pub fn register_target(id: u64, title: &str, url: &str) {
    origin();
    service().lock().unwrap().targets.insert(
        id,
        Target {
            id,
            title: title.into(),
            url: url.into(),
        },
    );
}
struct Recording {
    id: u64,
    start: Instant,
    end: Mutex<Option<Instant>>,
    options: StartOptions,
    count: AtomicUsize,
    dropped: AtomicU64,
    buffers: Mutex<Vec<Arc<ThreadBuffer>>>,
    archive: Mutex<Vec<Event>>,
}
pub fn start(mut options: StartOptions) -> Result<u64, String> {
    let mut s = service().lock().unwrap();
    if enabled() {
        return Err("A recording is already active".into());
    }
    origin();
    options.max_events = options.max_events.clamp(1024, 500_000);
    let id = IDS.fetch_add(1, Ordering::Relaxed);
    let r = Arc::new(Recording {
        id,
        start: Instant::now(),
        end: Mutex::new(None),
        options,
        count: AtomicUsize::new(0),
        dropped: AtomicU64::new(0),
        buffers: Mutex::new(Vec::new()),
        archive: Mutex::new(Vec::new()),
    });
    while s.recordings.len() >= 3 {
        s.recordings.pop_front();
    }
    s.recordings.push_back(r);
    ACTIVE.store(id, Ordering::Release);
    Ok(id)
}
pub fn stop() -> Result<u64, String> {
    let s = service().lock().unwrap();
    let id = ACTIVE.swap(0, Ordering::AcqRel);
    let r = s
        .recordings
        .iter()
        .find(|r| r.id == id)
        .ok_or("No active recording")?;
    *r.end.lock().unwrap() = Some(Instant::now());
    Ok(id)
}
pub fn status() -> serde_json::Value {
    let s = service().lock().unwrap();
    serde_json::json!({"active_recording":ACTIVE.load(Ordering::Acquire),"targets":s.targets.values().collect::<Vec<_>>(),"recordings":s.recordings.iter().map(|r|serde_json::json!({"id":r.id,"events":r.count.load(Ordering::Relaxed).min(r.options.max_events),"max_events":r.options.max_events,"target_id":r.options.target_id,"dropped_events":r.dropped.load(Ordering::Relaxed)})).collect::<Vec<_>>()})
}

#[derive(Clone, Copy, Default)]
struct Field {
    key: &'static str,
    value: f64,
}
#[derive(Clone, Copy)]
enum Kind {
    Begin,
    End,
    Instant,
    Interval(u64),
}
#[derive(Clone)]
struct Event {
    id: u64,
    parent: u64,
    thread: u64,
    at: u64,
    category: &'static str,
    name: &'static str,
    context: Context,
    fields: [Field; 8],
    len: usize,
    kind: Kind,
}
fn timestamp(r: &Recording, t: Instant) -> u64 {
    t.checked_duration_since(r.start)
        .map(|d| d.as_nanos().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}
fn attach() -> Option<(Arc<Recording>, Arc<ThreadBuffer>, Context, u64, u64)> {
    let id = ACTIVE.load(Ordering::Acquire);
    if id == 0 {
        return None;
    }
    LOCAL.with(|l| {
        let mut l = l.borrow_mut();
        if l.session.as_ref().map(|r| r.id) != Some(id) {
            let r = service()
                .lock()
                .unwrap()
                .recordings
                .iter()
                .find(|r| r.id == id)
                .cloned()?;
            let mut buffers = r.buffers.lock().unwrap();
            if buffers.len() >= 64 {
                r.dropped.fetch_add(1, Ordering::Relaxed);
                return None;
            }
            let (sender, receiver) = mpsc::channel();
            let b = Arc::new(ThreadBuffer {
                sender,
                receiver: Mutex::new(receiver),
            });
            buffers.push(b.clone());
            drop(buffers);
            l.parent = 0;
            l.session = Some(r);
            l.buffer = Some(b);
        }
        let r = l.session.as_ref()?.clone();
        if r.options.target_id != 0 && r.options.target_id != l.context.target_id {
            return None;
        }
        Some((
            r,
            l.buffer.as_ref()?.clone(),
            l.context,
            l.parent,
            l.thread_id,
        ))
    })
}
// The channel is bounded by Recording.count's lifetime event allowance. A
// collector can drain/clone its archive without taking the producer's lock.
struct ThreadBuffer {
    sender: mpsc::Sender<Event>,
    receiver: Mutex<mpsc::Receiver<Event>>,
}
fn push(r: &Recording, b: &ThreadBuffer, e: Event) {
    if r.count
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
            (n < r.options.max_events).then_some(n + 1)
        })
        .is_err()
    {
        r.dropped.fetch_add(1, Ordering::Relaxed);
        return;
    }
    if b.sender.send(e).is_err() {
        r.dropped.fetch_add(1, Ordering::Relaxed);
    }
}
pub struct Span {
    record: Option<(Arc<Recording>, Arc<ThreadBuffer>, Event)>,
    previous: u64,
    _same_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
pub fn span(category: &'static str, name: &'static str) -> Span {
    let mut span = Span {
        record: None,
        previous: 0,
        _same_thread: std::marker::PhantomData,
    };
    if !enabled() {
        return span;
    }
    if let Some((r, b, c, parent, thread)) = attach() {
        let id = IDS.fetch_add(1, Ordering::Relaxed);
        let e = Event {
            id,
            parent,
            thread,
            at: timestamp(&r, Instant::now()),
            category,
            name,
            context: c,
            fields: [Field::default(); 8],
            len: 0,
            kind: Kind::Begin,
        };
        push(&r, &b, e.clone());
        span.previous = parent;
        LOCAL.with(|l| l.borrow_mut().parent = id);
        span.record = Some((r, b, e));
    }
    span
}
impl Span {
    pub fn set(&mut self, key: &'static str, value: f64) {
        if let Some((_, _, e)) = &mut self.record {
            if !value.is_finite() {
                return;
            }
            if let Some(f) = e.fields[..e.len].iter_mut().find(|f| f.key == key) {
                f.value = value;
            } else if e.len < 8 {
                e.fields[e.len] = Field { key, value };
                e.len += 1;
            }
        }
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if let Some((r, b, mut e)) = self.record.take() {
            LOCAL.with(|l| l.borrow_mut().parent = self.previous);
            e.at = timestamp(&r, Instant::now());
            e.kind = Kind::End;
            push(&r, &b, e);
        }
    }
}
pub fn instant(category: &'static str, name: &'static str, fields: &[(&'static str, f64)]) {
    emit(category, name, Instant::now(), None, fields);
}
pub fn instant_at(
    category: &'static str,
    name: &'static str,
    at: Instant,
    fields: &[(&'static str, f64)],
) {
    emit(category, name, at, None, fields);
}
pub fn interval(
    category: &'static str,
    name: &'static str,
    start: Instant,
    end: Instant,
    fields: &[(&'static str, f64)],
) {
    emit(category, name, start, Some(end), fields);
}
fn emit(
    category: &'static str,
    name: &'static str,
    at: Instant,
    end: Option<Instant>,
    fields: &[(&'static str, f64)],
) {
    if !enabled() {
        return;
    }
    if let Some((r, b, c, parent, thread)) = attach() {
        let mut e = Event {
            id: IDS.fetch_add(1, Ordering::Relaxed),
            parent,
            thread,
            at: timestamp(&r, at),
            category,
            name,
            context: c,
            fields: [Field::default(); 8],
            len: 0,
            kind: end
                .map(|t| Kind::Interval(timestamp(&r, t)))
                .unwrap_or(Kind::Instant),
        };
        for &(key, value) in fields.iter().take(8) {
            if value.is_finite() {
                e.fields[e.len] = Field { key, value };
                e.len += 1;
            }
        }
        push(&r, &b, e);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordedSpan {
    pub id: u64,
    pub parent_id: u64,
    pub thread_id: u64,
    pub category: String,
    pub name: String,
    pub start_ns: u64,
    pub duration_ns: u64,
    pub incomplete: bool,
    pub context: Context,
    pub fields: BTreeMap<String, f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    pub recording_id: u64,
    pub active: bool,
    pub duration_ns: u64,
    pub dropped_events: u64,
    pub spans: Vec<RecordedSpan>,
    pub instants: Vec<RecordedSpan>,
}
fn recorded(e: &Event, end: u64, incomplete: bool) -> RecordedSpan {
    RecordedSpan {
        id: e.id,
        parent_id: e.parent,
        thread_id: e.thread,
        category: e.category.into(),
        name: e.name.into(),
        start_ns: e.at,
        duration_ns: end.saturating_sub(e.at),
        incomplete,
        context: e.context,
        fields: e.fields[..e.len]
            .iter()
            .map(|f| (f.key.into(), f.value))
            .collect(),
    }
}
pub fn snapshot(id: Option<u64>) -> Result<Snapshot, String> {
    let r = {
        let s = service().lock().unwrap();
        match id {
            Some(id) => s.recordings.iter().find(|r| r.id == id),
            None => s.recordings.back(),
        }
        .cloned()
        .ok_or("Recording not found")?
    };
    let stop = *r.end.lock().unwrap();
    let end = timestamp(&r, stop.unwrap_or_else(Instant::now));
    let buffers = r.buffers.lock().unwrap().clone();
    let events = {
        // Only collectors serialize here. Producers use independent channels.
        let mut archive = r.archive.lock().unwrap();
        for b in buffers {
            archive.extend(b.receiver.lock().unwrap().try_iter());
        }
        archive
            .iter()
            .filter(|e| e.at <= end)
            .cloned()
            .collect::<Vec<_>>()
    };
    let mut starts = HashMap::new();
    let mut ends = HashMap::new();
    let mut spans = Vec::new();
    let mut instants = Vec::new();
    for e in events {
        match e.kind {
            Kind::Begin => {
                starts.insert(e.id, e);
            }
            Kind::End => {
                ends.insert(e.id, e);
            }
            Kind::Instant => instants.push(recorded(&e, e.at, false)),
            Kind::Interval(t) => spans.push(recorded(&e, t.min(end), false)),
        }
    }
    for (id, e) in starts {
        if let Some(mut finish) = ends.remove(&id) {
            let finished_at = finish.at;
            finish.at = e.at;
            spans.push(recorded(&finish, finished_at.min(end), false));
        } else {
            spans.push(recorded(&e, end, true));
        }
    }
    // End events whose Begin was dropped remain explicitly represented as loss.
    spans.sort_by_key(|e| (e.start_ns, e.id));
    instants.sort_by_key(|e| (e.start_ns, e.id));
    Ok(Snapshot {
        version: 1,
        recording_id: r.id,
        active: stop.is_none(),
        duration_ns: end,
        dropped_events: r.dropped.load(Ordering::Relaxed),
        spans,
        instants,
    })
}
pub fn chrome_trace(s: &Snapshot) -> serde_json::Value {
    let mut events = Vec::new();
    for e in s.spans.iter().chain(s.instants.iter()) {
        let mut args = serde_json::to_value(&e.fields).unwrap();
        args["context"] = serde_json::to_value(e.context).unwrap();
        args["incomplete"] = serde_json::json!(e.incomplete);
        events.push(serde_json::json!({"name":e.name,"cat":e.category,"ph":if e.duration_ns==0 {"i"}else{"X"},"s":"t","pid":e.context.target_id,"tid":e.thread_id,"ts":e.start_ns as f64/1000.,"dur":e.duration_ns as f64/1000.,"args":args}));
    }
    serde_json::json!({"traceEvents":events,"displayTimeUnit":"ms","metadata":{"recording_id":s.recording_id,"dropped_events":s.dropped_events,"clock":"monotonic wall","presentation":"present call return, not display scanout"}})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recording_preserves_nesting_context_open_spans_and_boundaries() {
        assert!(!enabled());
        let id = start(StartOptions::default()).unwrap();
        let outer_context = scope(Context {
            target_id: 7,
            frame_id: 42,
            ..Default::default()
        });
        let mut outer = span("test", "outer");
        outer.set("tiles", 10.);
        let nested = span("test", "nested");
        drop(nested);
        let live = snapshot(Some(id)).unwrap();
        let open = live.spans.iter().find(|s| s.name == "outer").unwrap();
        assert!(open.incomplete);
        let child = live.spans.iter().find(|s| s.name == "nested").unwrap();
        assert_eq!(child.parent_id, open.id);
        assert_eq!(child.context.frame_id, 42);
        drop(outer);
        drop(outer_context);
        assert_eq!(context().target_id, 0);
        let cross_stop = span("test", "cross-stop");
        stop().unwrap();
        drop(cross_stop);
        let done = snapshot(Some(id)).unwrap();
        assert!(!done.active);
        assert_eq!(done.dropped_events, 0);
        assert_eq!(
            done.spans
                .iter()
                .find(|s| s.name == "outer")
                .unwrap()
                .fields["tiles"],
            10.
        );
        assert!(
            done.spans
                .iter()
                .find(|s| s.name == "cross-stop")
                .unwrap()
                .incomplete
        );
        let export = chrome_trace(&done);
        assert!(export["traceEvents"].is_array());
        let _ = start(StartOptions {
            max_events: 1024,
            target_id: 8,
        })
        .unwrap();
        {
            let _scope = scope(Context {
                target_id: 7,
                ..Default::default()
            });
            let _ = span("test", "excluded");
        }
        {
            let _scope = scope(Context {
                target_id: 8,
                ..Default::default()
            });
            for _ in 0..600 {
                let _ = span("test", "bounded");
            }
        }
        stop().unwrap();
        let loss = snapshot(None).unwrap();
        assert!(loss.dropped_events > 0);
        assert!(loss.spans.iter().all(|s| s.name != "excluded"));
        // Live collectors must not lose events while copying earlier history.
        start(StartOptions {
            max_events: 20_000,
            target_id: 0,
        })
        .unwrap();
        let ready = Arc::new(std::sync::Barrier::new(2));
        let worker_ready = ready.clone();
        let worker = std::thread::spawn(move || {
            worker_ready.wait();
            for index in 0..1000 {
                let mut s = span("test", "concurrent");
                s.set("index", index as f64);
            }
        });
        ready.wait();
        for _ in 0..8 {
            let _ = snapshot(None).unwrap();
            std::thread::yield_now();
        }
        worker.join().unwrap();
        stop().unwrap();
        let complete = snapshot(None).unwrap();
        assert_eq!(complete.dropped_events, 0);
        assert_eq!(complete.spans.len(), 1000);
        assert!(complete.spans.iter().all(|s| !s.incomplete));
    }
}
