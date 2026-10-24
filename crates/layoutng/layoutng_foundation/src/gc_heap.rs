// C++: foundation/blink_base/heap/{heap.h,heap.cc,visitor.h,persistent.h,garbage_collected.h}.
// This is the single-thread-owned layout heap. Marking is atomic outside engine
// scopes; the Page host schedules incremental sweeping on the same owner.
use crate::{kMemberDeletedValue, Member, UntracedMember, WeakMember};
use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, Weak};

pub trait Traceable {
    fn Trace(&self, visitor: &mut Visitor<'_>);
}

// cpp: foundation/blink_base/heap/trace_traits.h:20-28
// Rust requires the trace policy to be explicit on each value type. Types
// without heap edges implement Traceable as a no-op; member-bearing types
// visit their edges.
pub fn TraceIfNeeded<T: Traceable + ?Sized>(visitor: &mut Visitor<'_>, value: &T) {
    visitor.Trace(value);
}

// C++: foundation/blink_base/heap/trace_traits.h:20-28. These scalar and
// independently owned values contain no layout-heap pointer edges.
macro_rules! impl_untraced_value {
    ($($type:ty),* $(,)?) => {
        $(impl Traceable for $type {
            fn Trace(&self, _visitor: &mut Visitor<'_>) {}
        })*
    };
}
impl_untraced_value!(
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize,
    f32,
    f64,
    std::string::String,
    crate::BlinkString,
    crate::Length,
    crate::LengthBox,
    crate::LengthPoint,
    crate::LengthSize,
    crate::Color,
    crate::AtomicString,
    crate::LayoutUnit,
    crate::TextDecorationThickness,
    crate::StyleAspectRatio,
    crate::StyleInitialLetter,
    crate::TabSize,
    crate::PhysicalOffset,
    crate::gfx::Size,
    crate::gfx::SizeF,
    crate::gfx::Vector2dF,
    crate::DynamicRangeLimit,
    crate::HangingPunctuation,
    crate::ImageAnimationEnum,
    crate::TouchAction,
    crate::TextJustify,
    crate::UnicodeBidi,
    crate::LineCap,
    crate::LineJoin,
    crate::WindRule
);

impl<T: ?Sized> Traceable for Member<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        if let Some(pointer) = self.GetNonNull() {
            visitor.heap.mark(pointer.as_ptr() as *const u8);
        }
    }
}

impl<T: ?Sized> Traceable for WeakMember<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        let slot = self as *const Self as *const ();
        visitor.heap.weak_slots.push(WeakSlot {
            slot,
            get: |slot| {
                unsafe { &*(slot as *const Self) }
                    .GetNonNull()
                    .map_or(std::ptr::null(), |p| p.as_ptr() as *const u8)
            },
            clear: |slot| unsafe { &*(slot as *const Self) }.ClearForCollection(),
        });
    }
}

// A borrowed traceable object has the same GC edges as its referent.
impl<T: Traceable + ?Sized> Traceable for &T {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        <T as Traceable>::Trace(*self, visitor);
    }
}

impl<T: ?Sized> Traceable for UntracedMember<T> {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

impl<T: Traceable> Traceable for Vec<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        for item in self {
            item.Trace(visitor);
        }
    }
}

impl<T: Traceable> Traceable for Option<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        if let Some(item) = self {
            item.Trace(visitor);
        }
    }
}

impl<T: ?Sized> Traceable for *const T {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.heap.mark(*self as *const u8);
    }
}

impl<T: ?Sized> Traceable for *mut T {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.heap.mark(*self as *const u8);
    }
}

type TraceFn = unsafe fn(*const u8, &mut Visitor<'_>);
type DestroyFn = unsafe fn(*mut u8);

// Collection-local dispatch data, copied from validated immutable metadata.
// It retains no pointer/reference to BTree-owned Allocation storage.
struct TraceWorkItem {
    address: *const u8,
    trace: TraceFn,
}

struct Allocation {
    address: NonNull<u8>,
    layout: Layout,
    trace: Option<TraceFn>,
    destroy: Option<DestroyFn>,
    marked: bool,
    strongly_marked: bool,
}

impl Drop for Allocation {
    fn drop(&mut self) {
        unsafe { dealloc(self.address.as_ptr(), self.layout) };
    }
}

struct WeakSlot {
    slot: *const (),
    get: fn(*const ()) -> *const u8,
    clear: fn(*const ()),
}

struct Ephemeron {
    key: *const u8,
    value: *const u8,
    trace: TraceFn,
    processed: bool,
}

struct WeakBacking {
    address: usize,
    owner: *const (),
    callback: fn(&LivenessBroker<'_>, *const ()),
}

trait RootSlot: Send + Sync {
    fn get(&self) -> *const u8;
    fn clear(&self) -> Option<usize>;
    fn weak(&self) -> bool;
}

enum PersistentNode {
    Used(Weak<dyn RootSlot>),
    Free(Option<usize>),
}

// cppgc/internal/persistent-node.h: PersistentRegionBase::{AllocateNode,FreeNode}.
// Slots have stable indices. Retired handles return their slot immediately;
// transient style/attachment handles must not grow the registry until GC.
#[derive(Default)]
struct PersistentRegion {
    nodes: Vec<PersistentNode>,
    free: Option<usize>,
    used: usize,
}
impl PersistentRegion {
    fn allocate(&mut self, root: Weak<dyn RootSlot>) -> usize {
        self.used += 1;
        if let Some(index) = self.free {
            let PersistentNode::Free(next) = self.nodes[index] else {
                unreachable!()
            };
            self.free = next;
            self.nodes[index] = PersistentNode::Used(root);
            index
        } else {
            let index = self.nodes.len();
            self.nodes.push(PersistentNode::Used(root));
            index
        }
    }
    fn release(&mut self, index: usize) {
        assert!(matches!(self.nodes[index], PersistentNode::Used(_)));
        self.nodes[index] = PersistentNode::Free(self.free);
        self.free = Some(index);
        self.used -= 1;
    }
    fn live_roots(&mut self) -> Vec<Arc<dyn RootSlot>> {
        let mut roots = Vec::with_capacity(self.used);
        for index in 0..self.nodes.len() {
            if let PersistentNode::Used(root) = &self.nodes[index] {
                if let Some(root) = root.upgrade() {
                    roots.push(root);
                } else {
                    // A handle may retire during thread/heap teardown when
                    // its owner cannot borrow the heap. Normal Clear/Drop
                    // already returned their slots and never reach this path.
                    self.release(index);
                }
            }
        }
        roots
    }
    fn iter(&self) -> impl Iterator<Item = &Weak<dyn RootSlot>> {
        self.nodes.iter().filter_map(|node| match node {
            PersistentNode::Used(root) => Some(root),
            PersistentNode::Free(_) => None,
        })
    }
    fn len(&self) -> usize {
        self.nodes.len()
    }
    fn capacity(&self) -> usize {
        self.nodes.capacity()
    }
}

struct HeapState {
    scopes: usize,
    collecting: bool,
    activity_since_collection: bool,
    heap_bytes: usize,
    allocated_bytes_since_collection: usize,
    collection_requested: bool,
    collections_completed: u64,
    last_before_bytes: usize,
    last_after_bytes: usize,
    last_reclaimed_objects: usize,
    profile_collection: bool,
    profile_mark_edges: usize,
    profile_mark_cache_hits: usize,
    // Valid only during one collection, when allocation intervals are immutable
    // and strong marks are monotonic. Repeated edges need no tree lookup.
    strong_marked_ranges: [(usize, usize); 4],
    next_strong_marked_range: usize,
    // Revisited shared edges need not be among the four most recent intervals.
    // Exact-address hits are valid only within this collection; collisions miss.
    strong_marked_addresses: [usize; 1024],
    objects: BTreeMap<usize, Allocation>,
    roots: PersistentRegion,
    worklist: Vec<TraceWorkItem>,
    weak_slots: Vec<WeakSlot>,
    weak_roots: Vec<Arc<dyn RootSlot>>,
    weak_backings: Vec<WeakBacking>,
    ephemerons: Vec<Ephemeron>,
    sweep: Option<SweepState>,
}

impl HeapState {
    fn new() -> Self {
        Self {
            scopes: 0,
            collecting: false,
            activity_since_collection: false,
            heap_bytes: 0,
            allocated_bytes_since_collection: 0,
            collection_requested: false,
            collections_completed: 0,
            last_before_bytes: 0,
            last_after_bytes: 0,
            last_reclaimed_objects: 0,
            profile_collection: false,
            profile_mark_edges: 0,
            profile_mark_cache_hits: 0,
            strong_marked_ranges: [(0, 0); 4],
            next_strong_marked_range: 0,
            strong_marked_addresses: [0; 1024],
            objects: BTreeMap::new(),
            roots: PersistentRegion::default(),
            worklist: Vec::new(),
            weak_slots: Vec::new(),
            weak_roots: Vec::new(),
            weak_backings: Vec::new(),
            ephemerons: Vec::new(),
            sweep: None,
        }
    }

    fn find(&self, pointer: *const u8) -> Option<usize> {
        let address = pointer as usize;
        let (&start, object) = self.objects.range(..=address).next_back()?;
        (address.wrapping_sub(start) < object.layout.size()).then_some(start)
    }

    fn alive(&self, pointer: *const u8) -> bool {
        if pointer.is_null() || pointer as usize == kMemberDeletedValue {
            return true;
        }
        if self.collecting && self.is_cached_strong_mark(pointer as usize) {
            return true;
        }
        self.objects
            .range(..=pointer as usize)
            .next_back()
            .filter(|(&start, object)| {
                (pointer as usize).wrapping_sub(start) < object.layout.size()
            })
            .expect("liveness query reached another heap")
            .1
            .marked
    }

    fn strong_mark_address_slot(address: usize) -> usize {
        // Fold allocation alignment and higher address bits. The full address,
        // rather than this slot or an unchecked interval, establishes a hit.
        ((address >> 4) ^ (address >> 16)) & 1023
    }

    fn is_cached_strong_mark(&self, address: usize) -> bool {
        (address != 0
            && self.strong_marked_addresses[Self::strong_mark_address_slot(address)] == address)
            || self
                .strong_marked_ranges
                .iter()
                .any(|&(start, size)| address.wrapping_sub(start) < size)
    }

    fn mark(&mut self, pointer: *const u8) {
        if pointer.is_null() || pointer as usize == kMemberDeletedValue {
            return;
        }
        if self.profile_collection {
            self.profile_mark_edges += 1;
        }
        if self.collecting && self.is_cached_strong_mark(pointer as usize) {
            if self.profile_collection {
                self.profile_mark_cache_hits += 1;
            }
            // An interval hit also establishes this exact interior address.
            self.strong_marked_addresses[Self::strong_mark_address_slot(pointer as usize)] =
                pointer as usize;
            return;
        }
        // Keep the interval lookup's Allocation instead of searching the same
        // BTree again by its base address. Interior pointers still mark the
        // whole allocation, and pointers beyond its end remain invalid.
        let (_, object) = self
            .objects
            .range_mut(..=pointer as usize)
            .next_back()
            .filter(|(&start, object)| {
                (pointer as usize).wrapping_sub(start) < object.layout.size()
            })
            .expect("Trace reached an object outside the layout heap");
        assert!(
            object.trace.is_some() && object.destroy.is_some(),
            "Trace reached an unfinished allocation"
        );
        if !object.strongly_marked {
            object.marked = true;
            object.strongly_marked = true;
            self.worklist.push(TraceWorkItem {
                address: object.address.as_ptr(),
                trace: object.trace.unwrap(),
            });
        }
        if self.collecting {
            // Store only after the original interval/finished-object checks and
            // a strong mark. Weak backing marks must never enter this cache.
            self.strong_marked_addresses[Self::strong_mark_address_slot(pointer as usize)] =
                pointer as usize;
            self.strong_marked_ranges[self.next_strong_marked_range] =
                (object.address.as_ptr() as usize, object.layout.size());
            self.next_strong_marked_range =
                (self.next_strong_marked_range + 1) % self.strong_marked_ranges.len();
        }
    }

    fn collect_marks(&mut self) -> Vec<usize> {
        assert_eq!(self.scopes, 0);
        assert!(!self.collecting);
        self.collecting = true;
        self.profile_collection = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        self.profile_mark_edges = 0;
        self.profile_mark_cache_hits = 0;
        let profile = self.profile_collection.then(std::time::Instant::now);
        self.strong_marked_ranges = [(0, 0); 4];
        self.next_strong_marked_range = 0;
        self.strong_marked_addresses.fill(0);
        self.activity_since_collection = false;
        self.allocated_bytes_since_collection = 0;
        self.collection_requested = false;
        let mut root_trace = browser_tracing::span("gc", "LayoutHeap.RootRegistry");
        root_trace.set("slots", self.roots.len() as f64);
        let roots = self.roots.live_roots();
        root_trace.set("live_cells", roots.len() as f64);
        drop(root_trace);
        let reset_trace = browser_tracing::span("gc", "LayoutHeap.ResetMarks");
        for object in self.objects.values_mut() {
            object.marked = false;
            object.strongly_marked = false;
        }
        self.worklist.clear();
        self.weak_slots.clear();
        self.weak_roots.clear();
        self.weak_backings.clear();
        self.ephemerons.clear();
        drop(reset_trace);
        let mut roots_trace = browser_tracing::span("gc", "LayoutHeap.MarkRoots");
        roots_trace.set("live_cells", roots.len() as f64);
        for root in roots {
            if root.weak() {
                self.weak_roots.push(root);
            } else {
                self.mark(root.get());
            }
        }
        drop(roots_trace);
        let roots_done = profile.map(|start| start.elapsed());
        let mut mark_trace = browser_tracing::span("gc", "LayoutHeap.MarkGraph");
        let mut visitor = Visitor { heap: self };
        let mut traced_objects = 0;
        loop {
            while let Some(item) = visitor.heap.worklist.pop() {
                traced_objects += 1;
                // The payload stays allocated throughout mark/weak processing;
                // collection cannot allocate/free/cancel an object while tracing.
                // The callback was validated when this exact object was marked.
                unsafe { (item.trace)(item.address, &mut visitor) };
            }
            let mut processed = false;
            for index in 0..visitor.heap.ephemerons.len() {
                let edge = &visitor.heap.ephemerons[index];
                if !edge.processed && visitor.heap.alive(edge.key) {
                    let (value, trace) = (edge.value, edge.trace);
                    visitor.heap.ephemerons[index].processed = true;
                    unsafe { trace(value, &mut visitor) };
                    processed = true;
                }
            }
            if !processed && visitor.heap.worklist.is_empty() {
                break;
            }
        }
        mark_trace.set("traced_objects", traced_objects as f64);
        drop(mark_trace);
        let trace_done = profile.map(|start| start.elapsed());
        let weak_trace = browser_tracing::span("gc", "LayoutHeap.ProcessWeak");
        {
            let broker = LivenessBroker {
                heap: &*visitor.heap,
            };
            for backing in &visitor.heap.weak_backings {
                if !broker.heap.objects[&backing.address].strongly_marked {
                    (backing.callback)(&broker, backing.owner);
                }
            }
            for slot in &visitor.heap.weak_slots {
                if !broker.IsHeapObjectAlive((slot.get)(slot.slot)) {
                    (slot.clear)(slot.slot);
                }
            }
        }
        for index in 0..visitor.heap.weak_roots.len() {
            let root = &visitor.heap.weak_roots[index];
            if !visitor.heap.alive(root.get()) {
                if let Some(slot) = root.clear() {
                    visitor.heap.roots.release(slot);
                }
            }
        }
        drop(weak_trace);
        let weak_done = profile.map(|start| start.elapsed());
        let dead_trace = browser_tracing::span("gc", "LayoutHeap.FindDead");
        let dead: Vec<_> = visitor
            .heap
            .objects
            .iter()
            .filter_map(|(&address, object)| (!object.marked).then_some(address))
            .collect();
        drop(dead_trace);
        if let Some(start) = profile {
            let elapsed = start.elapsed();
            eprintln!(
                "layout-gc-mark-profile objects={} roots={} traced={} dead={} edges={} cache_hits={} roots_ms={:.3} trace_ms={:.3} weak_ms={:.3} dead_scan_ms={:.3} total_ms={:.3}",
                visitor.heap.objects.len(), visitor.heap.roots.len(), traced_objects, dead.len(),
                visitor.heap.profile_mark_edges, visitor.heap.profile_mark_cache_hits,
                roots_done.unwrap().as_secs_f64() * 1000.0,
                (trace_done.unwrap() - roots_done.unwrap()).as_secs_f64() * 1000.0,
                (weak_done.unwrap() - trace_done.unwrap()).as_secs_f64() * 1000.0,
                (elapsed - weak_done.unwrap()).as_secs_f64() * 1000.0,
                elapsed.as_secs_f64() * 1000.0,
            );
        }
        visitor.heap.profile_collection = false;
        dead
    }
}

thread_local! {
    static HEAP: RefCell<HeapState> = RefCell::new(HeapState::new());
    static SWEEPING: Cell<bool> = const { Cell::new(false) };
}

// cppgc supports atomic marking with incremental sweeping. No mutation may
// interleave with mark/weak processing. After marking, only the fixed dead set
// is swept; new allocations and changes to surviving objects belong to the next
// collection. All finalizers finish before metadata/storage is released, since
// native destructors can inspect other dead allocations.
struct SweepState {
    dead: Vec<usize>,
    finalized: usize,
    released: usize,
    before_bytes: usize,
}

fn start_collection() {
    let mut trace = browser_tracing::span("gc", "LayoutHeap.StartCollection");
    HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        assert!(
            heap.sweep.is_none(),
            "finish sweeping before starting another collection"
        );
        trace.set("objects", heap.objects.len() as f64);
        trace.set("roots", heap.roots.used as f64);
        trace.set("root_slots", heap.roots.len() as f64);
        let before_bytes = heap.heap_bytes;
        let dead = heap.collect_marks();
        trace.set("reclaimed_objects", dead.len() as f64);
        heap.sweep = Some(SweepState {
            dead,
            finalized: 0,
            released: 0,
            before_bytes,
        });
        heap.collecting = false;
    });
}

fn sweep_until(deadline: Option<std::time::Instant>) {
    let mut state = HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        assert_eq!(heap.scopes, 0);
        assert!(!heap.collecting);
        let state = heap.sweep.take().expect("pending sweep");
        heap.collecting = true;
        state
    });
    let has_time = || deadline.is_none_or(|limit| std::time::Instant::now() < limit);
    SWEEPING.set(true);
    {
        let mut trace = browser_tracing::span("gc", "LayoutHeap.Finalize");
        let started = state.finalized;
        while state.finalized < state.dead.len() && has_time() {
            let address = state.dead[state.finalized];
            let destroy = HEAP.with(|heap| {
                heap.borrow_mut()
                    .objects
                    .get_mut(&address)
                    .unwrap()
                    .destroy
                    .take()
            });
            if let Some(destroy) = destroy {
                unsafe { destroy(address as *mut u8) };
            }
            state.finalized += 1;
        }
        trace.set("objects", (state.finalized - started) as f64);
    }
    {
        let mut trace = browser_tracing::span("gc", "LayoutHeap.Sweep");
        let started = state.released;
        if state.finalized == state.dead.len() {
            HEAP.with(|heap| {
                let mut heap = heap.borrow_mut();
                while state.released < state.dead.len() && has_time() {
                    let address = state.dead[state.released];
                    let object = heap.objects.remove(&address).unwrap();
                    heap.heap_bytes = heap.heap_bytes.saturating_sub(object.layout.size());
                    drop(object);
                    state.released += 1;
                }
            });
        }
        trace.set("objects", (state.released - started) as f64);
    }
    HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        if state.released == state.dead.len() {
            heap.weak_slots.clear();
            heap.weak_roots.clear();
            heap.weak_backings.clear();
            heap.ephemerons.clear();
            heap.collections_completed = heap.collections_completed.saturating_add(1);
            heap.last_before_bytes = state.before_bytes;
            heap.last_after_bytes = heap.heap_bytes;
            heap.last_reclaimed_objects = state.dead.len();
        } else {
            heap.sweep = Some(state);
        }
        heap.collecting = false;
    });
    SWEEPING.set(false);
}

fn collect() {
    let _trace = browser_tracing::span("gc", "LayoutHeap.Collect");
    if HEAP.with(|heap| heap.borrow().sweep.is_some()) {
        sweep_until(None);
    }
    start_collection();
    sweep_until(None);
}

/// Run one non-nestable foreground GC task on the heap owner. This corresponds
/// to cppgc's atomic marking / incremental sweeping mode. The deadline covers
/// both mark and sweep; marking and an individual destructor remain atomic.
/// The owner reposts while IsLayoutHeapCollectionRequested() remains true.
pub fn CollectLayoutHeapIncrementally(budget: std::time::Duration) {
    let deadline = std::time::Instant::now() + budget;
    let mut trace = browser_tracing::span("gc", "LayoutHeap.CollectionTask");
    let (requested, sweeping) = HEAP.with(|heap| {
        let heap = heap.borrow();
        assert_eq!(heap.scopes, 0);
        assert!(!heap.collecting);
        (heap.collection_requested, heap.sweep.is_some())
    });
    trace.set("starts_collection", (requested && !sweeping) as u8 as f64);
    if !sweeping {
        if !requested {
            return;
        }
        start_collection();
    }
    sweep_until(Some(deadline));
    trace.set("pending", IsLayoutHeapCollectionRequested() as u8 as f64);
}

// cpp: foundation/blink_base/heap/heap.h:61-76
pub struct LayoutHeapScope {
    reuse_unchanged_heap: bool,
    defer_collection: bool,
}

impl LayoutHeapScope {
    pub fn new() -> Self {
        HEAP.with(|heap| {
            let mut heap = heap.borrow_mut();
            assert!(!heap.collecting, "cannot enter a scope during collection");
            heap.scopes += 1;
        });
        Self {
            reuse_unchanged_heap: false,
            defer_collection: false,
        }
    }

    /// An unchanged resident layout may reuse the last completed collection.
    /// Allocation or Persistent root changes still force collection at this
    /// scope exit. Other scopes keep their existing eager collection behavior.
    /// This is a conservative local adapter to cppgc allocation-driven GC;
    /// no incremental collector, growing thresholds or write barrier is added.
    pub fn AllowUnchangedReuse(&mut self) {
        self.reuse_unchanged_heap = true;
    }

    /// Keep a browser lifecycle operation free of stop-the-world collection.
    /// Allocations still raise the heap's collection request; the owner event
    /// loop services that request from a low-priority turn. This is the small
    /// synchronous collector's equivalent of cppgc scheduling collection from
    /// allocation pressure instead of collecting at every Blink API boundary.
    pub fn DeferCollection(&mut self) {
        self.defer_collection = true;
    }
}

impl Default for LayoutHeapScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LayoutHeapScope {
    fn drop(&mut self) {
        let collect_now = HEAP.with(|heap| {
            let mut heap = heap.borrow_mut();
            assert!(heap.scopes > 0);
            heap.scopes -= 1;
            heap.scopes == 0
                && !self.defer_collection
                && (!self.reuse_unchanged_heap || heap.activity_since_collection)
        });
        if collect_now {
            collect();
        }
    }
}

pub struct Visitor<'a> {
    heap: &'a mut HeapState,
}

impl Visitor<'_> {
    pub fn IsConcurrent(&self) -> bool {
        false
    }
    pub fn Trace<T: Traceable + ?Sized>(&mut self, value: &T) {
        value.Trace(self);
    }
    pub fn TraceStrongly<T: ?Sized>(&mut self, value: &WeakMember<T>) {
        if let Some(pointer) = value.GetNonNull() {
            self.heap.mark(pointer.as_ptr() as *const u8);
        }
    }
    pub fn TraceEphemeron<K: ?Sized, V: Traceable>(&mut self, key: &WeakMember<K>, value: &V) {
        unsafe fn trace_value<V: Traceable>(pointer: *const u8, visitor: &mut Visitor<'_>) {
            (&*(pointer as *const V)).Trace(visitor);
        }
        let key = key
            .GetNonNull()
            .map_or(std::ptr::null(), |p| p.as_ptr() as *const u8);
        self.heap.ephemerons.push(Ephemeron {
            key,
            value: value as *const V as *const u8,
            trace: trace_value::<V>,
            processed: false,
        });
    }
    pub fn TraceWeakBacking<T: Traceable>(
        &mut self,
        backing: *const T,
        callback: fn(&LivenessBroker<'_>, *const ()),
        owner: *const (),
    ) {
        if backing.is_null() {
            return;
        }
        let address = self
            .heap
            .find(backing as *const u8)
            .expect("weak backing belongs to another heap");
        assert_eq!(address, backing as usize);
        self.heap.objects.get_mut(&address).unwrap().marked = true;
        self.heap.weak_backings.push(WeakBacking {
            address,
            owner,
            callback,
        });
        if !self.heap.objects[&address].strongly_marked {
            backing.Trace(self);
        }
    }
}

pub struct LivenessBroker<'a> {
    heap: &'a HeapState,
}

impl LivenessBroker<'_> {
    pub fn IsHeapObjectAlive<T: ?Sized>(&self, pointer: *const T) -> bool {
        self.heap.alive(pointer as *const u8)
    }
}

// cpp: foundation/blink_base/heap/garbage_collected.h:34-70
pub fn MakeGarbageCollected<T: Traceable + 'static>(value: T) -> *mut T {
    unsafe {
        MakeGarbageCollectedWithAdditionalBytes::<T>(0, |pointer| {
            pointer.write(value);
        })
    }
}

/// Allocates a managed object with trailing storage. `init` must initialize
/// exactly one `T` at `pointer` before returning, as C++ placement-new does.
///
/// # Safety
/// The caller must fully initialize `T` even if it uses the trailing bytes.
/// Those bytes must not be accessed after the allocation is collected.
pub unsafe fn MakeGarbageCollectedWithAdditionalBytes<T: Traceable + 'static>(
    additional: usize,
    init: impl FnOnce(*mut T),
) -> *mut T {
    unsafe fn trace_object<T: Traceable>(pointer: *const u8, visitor: &mut Visitor<'_>) {
        (&*(pointer as *const T)).Trace(visitor);
    }
    unsafe fn destroy_object<T>(pointer: *mut u8) {
        std::ptr::drop_in_place(pointer as *mut T);
    }
    let size = std::mem::size_of::<T>()
        .checked_add(additional)
        .expect("layout allocation size overflow")
        .max(1);
    let layout = Layout::from_size_align(size, std::mem::align_of::<T>())
        .expect("invalid layout allocation");
    HEAP.with(|heap| {
        let heap = heap.borrow();
        assert!(
            heap.scopes > 0 && !heap.collecting,
            "layout allocation requires LayoutHeapScope"
        );
    });
    let address = unsafe { alloc_zeroed(layout) };
    let address = NonNull::new(address).unwrap_or_else(|| handle_alloc_error(layout));
    let pointer = address.as_ptr() as *mut T;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| init(pointer)));
    if let Err(error) = result {
        unsafe { dealloc(address.as_ptr(), layout) };
        std::panic::resume_unwind(error);
    }
    HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        // Registration follows complete initialization. Collection cannot run
        // inside LayoutHeapScope, so no trace can observe the object between
        // placement construction and this insertion.
        heap.activity_since_collection = true;
        heap.allocated_bytes_since_collection = heap
            .allocated_bytes_since_collection
            .saturating_add(layout.size());
        // cppgc grows the heap between collections and schedules GC from
        // allocation pressure. Our collector is currently synchronous, so a
        // moderately sized budget keeps it out of every layout while bounding
        // transient garbage until the owner services the request while idle.
        const MIN_ALLOCATION_BUDGET: usize = 4 * 1024 * 1024;
        let budget = MIN_ALLOCATION_BUDGET.max(heap.heap_bytes / 2);
        if heap.allocated_bytes_since_collection >= budget {
            heap.collection_requested = true;
        }
        heap.objects.insert(
            address.as_ptr() as usize,
            Allocation {
                address,
                layout,
                trace: Some(trace_object::<T>),
                destroy: Some(destroy_object::<T>),
                marked: false,
                strongly_marked: false,
            },
        );
        heap.heap_bytes = heap.heap_bytes.saturating_add(layout.size());
    });
    pointer
}

struct RootCell<T> {
    pointer: AtomicPtr<T>,
    weak: bool,
    owner: std::thread::ThreadId,
    // Index + 1; zero means that this cleared cell owns no region slot.
    registration: AtomicUsize,
}
impl<T> RootSlot for RootCell<T> {
    fn get(&self) -> *const u8 {
        self.pointer.load(Ordering::Relaxed) as *const u8
    }
    fn clear(&self) -> Option<usize> {
        self.pointer.store(std::ptr::null_mut(), Ordering::Relaxed);
        self.registration.swap(0, Ordering::Relaxed).checked_sub(1)
    }
    fn weak(&self) -> bool {
        self.weak
    }
}

// cpp: foundation/blink_base/heap/persistent.h:12-78
pub struct BasicPersistent<T, const WEAK: bool> {
    slot: Arc<RootCell<T>>,
}
pub type Persistent<T> = BasicPersistent<T, false>;
pub type WeakPersistent<T> = BasicPersistent<T, true>;

impl<T: 'static, const WEAK: bool> BasicPersistent<T, WEAK> {
    pub fn new() -> Self {
        Self {
            slot: Arc::new(RootCell {
                pointer: AtomicPtr::new(std::ptr::null_mut()),
                weak: WEAK,
                owner: std::thread::current().id(),
                registration: AtomicUsize::new(0),
            }),
        }
    }
    pub fn from_ptr(pointer: *mut T) -> Self {
        let mut result = Self::new();
        result.Assign(pointer);
        result
    }
    fn check_thread(&self) {
        assert_eq!(
            self.slot.owner,
            std::thread::current().id(),
            "Persistent belongs to another thread"
        );
    }
    pub fn Get(&self) -> *mut T {
        self.check_thread();
        let pointer = self.slot.pointer.load(Ordering::Relaxed);
        if !pointer.is_null() {
            assert!(
                IsManagedLayoutAddress(pointer),
                "Persistent refers to a reclaimed object"
            );
        }
        pointer
    }
    pub fn Assign(&mut self, pointer: *mut T) {
        self.check_thread();
        if pointer.is_null() {
            self.Clear();
            return;
        }
        if !pointer.is_null() {
            assert!(
                IsManagedLayoutAddress(pointer),
                "Persistent requires a managed layout object"
            );
        }
        if !pointer.is_null() {
            // cppgc BasicPersistent::Assign reuses a non-null handle's node;
            // assigning null/Clear/Drop returns it to the region's free list.
            HEAP.with(|heap| {
                let mut heap = heap.borrow_mut();
                assert!(!heap.collecting);
                if self.slot.registration.load(Ordering::Relaxed) == 0 {
                    let slot: Arc<dyn RootSlot> = self.slot.clone();
                    let index = heap.roots.allocate(Arc::downgrade(&slot));
                    self.slot.registration.store(index + 1, Ordering::Relaxed);
                }
            });
        }
        if self.slot.pointer.load(Ordering::Relaxed) != pointer {
            HEAP.with(|heap| heap.borrow_mut().activity_since_collection = true);
        }
        self.slot.pointer.store(pointer, Ordering::Relaxed);
    }
    pub fn Clear(&mut self) {
        self.check_thread();
        if self.slot.get().is_null() && self.slot.registration.load(Ordering::Relaxed) == 0 {
            return;
        }
        HEAP.with(|heap| {
            let mut heap = heap.borrow_mut();
            if !self.slot.get().is_null() {
                heap.activity_since_collection = true;
            }
            if let Some(index) = self.slot.clear() {
                heap.roots.release(index);
            }
        });
    }
    pub fn Release(&mut self) -> *mut T {
        let pointer = self.Get();
        self.Clear();
        pointer
    }
    pub fn Swap(&mut self, other: &mut Self) {
        let pointer = self.Get();
        self.Assign(other.Get());
        other.Assign(pointer);
    }
}
impl<T, const WEAK: bool> Drop for BasicPersistent<T, WEAK> {
    fn drop(&mut self) {
        if self.slot.owner == std::thread::current().id()
            && !self.slot.pointer.load(Ordering::Relaxed).is_null()
        {
            let _ = HEAP.try_with(|heap| {
                if let Ok(mut heap) = heap.try_borrow_mut() {
                    heap.activity_since_collection = true;
                    if let Some(index) = self.slot.clear() {
                        heap.roots.release(index);
                    }
                }
            });
        }
    }
}
impl<T: 'static, const WEAK: bool> Default for BasicPersistent<T, WEAK> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: 'static, const WEAK: bool> Clone for BasicPersistent<T, WEAK> {
    fn clone(&self) -> Self {
        Self::from_ptr(self.Get())
    }
}

pub fn IsLayoutHeapSweepingOnOwningThread() -> bool {
    SWEEPING.get()
}
pub fn IsManagedLayoutAddress<T: ?Sized>(pointer: *const T) -> bool {
    HEAP.with(|heap| heap.borrow().find(pointer as *const u8).is_some())
}
pub fn LayoutObjectSize<T: ?Sized>(pointer: *const T) -> usize {
    HEAP.with(|heap| {
        let heap = heap.borrow();
        let address = pointer as *const u8 as usize;
        assert_eq!(heap.find(pointer as *const u8), Some(address));
        heap.objects[&address].layout.size()
    })
}
pub fn FreeLayoutBacking<T: ?Sized>(pointer: *mut T) {
    if pointer.is_null() || SWEEPING.get() {
        return;
    }
    let destroy = HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        assert!(heap.scopes > 0 && !heap.collecting);
        let address = pointer as *mut u8 as usize;
        heap.objects
            .get_mut(&address)
            .expect("backing belongs to another heap")
            .destroy
            .take()
    });
    if let Some(destroy) = destroy {
        unsafe { destroy(pointer as *mut u8) };
    }
    HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        if let Some(object) = heap.objects.remove(&(pointer as *mut u8 as usize)) {
            heap.heap_bytes = heap.heap_bytes.saturating_sub(object.layout.size());
        }
    });
}
pub fn LayoutHeapAllocationCountForTesting() -> usize {
    HEAP.with(|heap| heap.borrow().objects.len())
}
pub fn CollectLayoutHeapForTesting() {
    collect();
}

/// True once allocation pressure asks the owner event loop for a collection.
/// The query is side-effect free and is valid only on the layout heap's owner
/// thread, like all other heap operations.
pub fn IsLayoutHeapCollectionRequested() -> bool {
    HEAP.with(|heap| {
        let heap = heap.borrow();
        heap.collection_requested || heap.sweep.is_some()
    })
}

/// Ask the next low-priority owner turn to collect, for example after an
/// entire document has been retired.
pub fn RequestLayoutHeapCollection() {
    HEAP.with(|heap| heap.borrow_mut().collection_requested = true);
}

/// Service an allocation-driven request outside input/layout/frame work.
/// Returns whether a collection ran.
pub fn CollectLayoutHeapIfRequested() -> bool {
    let requested = HEAP.with(|heap| {
        let heap = heap.borrow();
        assert_eq!(heap.scopes, 0, "cannot collect inside a layout heap scope");
        !heap.collecting && (heap.collection_requested || heap.sweep.is_some())
    });
    if requested {
        collect();
    }
    requested
}

#[cfg(test)]
mod interval_lookup_tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn allocate(heap: &mut HeapState, size: usize) -> usize {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let address = NonNull::new(unsafe { alloc_zeroed(layout) }).unwrap();
        let start = address.as_ptr() as usize;
        heap.objects.insert(
            start,
            Allocation {
                address,
                layout,
                trace: Some(|_, _| {}),
                destroy: Some(|_| {}),
                marked: false,
                strongly_marked: false,
            },
        );
        start
    }

    // Exact previous lookup/mark policy, retained only as a test reference.
    fn reference_mark(heap: &mut HeapState, pointer: *const u8) {
        if pointer.is_null() || pointer as usize == kMemberDeletedValue {
            return;
        }
        let start = heap
            .find(pointer)
            .expect("Trace reached an object outside the layout heap");
        let object = heap.objects.get_mut(&start).unwrap();
        assert!(
            object.trace.is_some() && object.destroy.is_some(),
            "Trace reached an unfinished allocation"
        );
        if !object.strongly_marked {
            object.marked = true;
            object.strongly_marked = true;
            heap.worklist.push(TraceWorkItem {
                address: object.address.as_ptr(),
                trace: object.trace.unwrap(),
            });
        }
    }

    fn reference_alive(heap: &HeapState, pointer: *const u8) -> bool {
        if pointer.is_null() || pointer as usize == kMemberDeletedValue {
            return true;
        }
        heap.objects
            .get(
                &heap
                    .find(pointer)
                    .expect("liveness query reached another heap"),
            )
            .unwrap()
            .marked
    }

    #[test]
    fn interval_lookup_matches_previous_mark_and_liveness_at_every_byte() {
        let mut actual = HeapState::new();
        let mut reference = HeapState::new();
        let starts: Vec<_> = (1..=32)
            .map(|size| {
                (
                    allocate(&mut actual, size),
                    allocate(&mut reference, size),
                    size,
                )
            })
            .collect();
        for &(a, b, size) in starts.iter().rev() {
            for offset in (0..size).rev() {
                let a_ptr = (a + offset) as *const u8;
                let b_ptr = (b + offset) as *const u8;
                assert_eq!(actual.alive(a_ptr), reference_alive(&reference, b_ptr));
                actual.mark(a_ptr);
                reference_mark(&mut reference, b_ptr);
                assert_eq!(actual.alive(a_ptr), reference_alive(&reference, b_ptr));
                assert_eq!(actual.objects[&a].marked, reference.objects[&b].marked);
                assert_eq!(
                    actual.objects[&a].strongly_marked,
                    reference.objects[&b].strongly_marked
                );
                assert_eq!(actual.worklist.len(), reference.worklist.len());
            }
        }
        assert_eq!(actual.worklist.len(), starts.len());
        for pointer in [std::ptr::null(), kMemberDeletedValue as *const u8] {
            let length = actual.worklist.len();
            actual.mark(pointer);
            reference_mark(&mut reference, pointer);
            assert_eq!(actual.worklist.len(), length);
            assert!(actual.alive(pointer));
            assert_eq!(actual.alive(pointer), reference_alive(&reference, pointer));
        }
        assert_eq!(
            actual
                .worklist
                .iter()
                .map(|item| item.address as usize)
                .collect::<Vec<_>>(),
            starts.iter().rev().map(|&(a, _, _)| a).collect::<Vec<_>>()
        );
    }

    #[test]
    fn interval_lookup_preserves_outside_and_unfinished_assertions() {
        for (has_trace, has_destroy) in [(true, true), (false, true), (true, false)] {
            let mut heap = HeapState::new();
            let start = allocate(&mut heap, 16);
            if !has_trace {
                heap.objects.get_mut(&start).unwrap().trace = None;
            }
            if !has_destroy {
                heap.objects.get_mut(&start).unwrap().destroy = None;
            }
            for address in [start - 1, start + 16, start + 17] {
                let pointer = address as *const u8;
                assert!(catch_unwind(AssertUnwindSafe(|| heap.mark(pointer))).is_err());
                assert!(
                    catch_unwind(AssertUnwindSafe(|| reference_mark(&mut heap, pointer))).is_err()
                );
                assert!(catch_unwind(AssertUnwindSafe(|| heap.alive(pointer))).is_err());
                assert!(
                    catch_unwind(AssertUnwindSafe(|| reference_alive(&heap, pointer))).is_err()
                );
            }
            if !has_trace || !has_destroy {
                for offset in [0, 1, 15] {
                    let pointer = (start + offset) as *const u8;
                    assert!(catch_unwind(AssertUnwindSafe(|| heap.mark(pointer))).is_err());
                    assert!(
                        catch_unwind(AssertUnwindSafe(|| reference_mark(&mut heap, pointer)))
                            .is_err()
                    );
                    assert!(!heap.alive(pointer));
                }
                assert!(heap.worklist.is_empty());
            }
        }
    }

    #[test]
    fn full_collection_preserves_interior_roots_weak_ephemerons_and_destruction_on_workers() {
        use std::rc::Rc;
        struct Counted {
            id: usize,
            drops: Rc<RefCell<Vec<usize>>>,
        }
        impl Traceable for Counted {
            fn Trace(&self, _: &mut Visitor<'_>) {}
        }
        impl Drop for Counted {
            fn drop(&mut self) {
                // Sweep must keep all allocation metadata until destructors finish.
                assert!(IsManagedLayoutAddress(self as *const Self));
                assert!(LayoutObjectSize(self as *const Self) >= std::mem::size_of::<Self>());
                self.drops.borrow_mut().push(self.id);
            }
        }
        struct Edges {
            strong_interior: *const u8,
            weak_interior: WeakMember<u8>,
            key: WeakMember<Counted>,
            value_key: WeakMember<Counted>,
            value: Member<Counted>,
            leaf: Member<Counted>,
        }
        impl Traceable for Edges {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                self.strong_interior.Trace(visitor);
                self.weak_interior.Trace(visitor);
                self.key.Trace(visitor);
                self.value_key.Trace(visitor);
                // Reverse dependency order requires an ephemeron fixed-point revisit.
                if self.value_key.GetNonNull().is_some() {
                    visitor.TraceEphemeron(&self.value_key, &self.leaf);
                }
                if self.key.GetNonNull().is_some() {
                    visitor.TraceEphemeron(&self.key, &self.value);
                }
            }
        }
        let workers: Vec<_> = (0..2)
            .map(|_| {
                std::thread::spawn(|| {
                    let drops = Rc::new(RefCell::new(Vec::new()));
                    let (root, mut key_root, weak);
                    {
                        let _scope = LayoutHeapScope::new();
                        let node = |id| {
                            MakeGarbageCollected(Counted {
                                id,
                                drops: drops.clone(),
                            })
                        };
                        let strong = node(1);
                        let dead = node(2);
                        let key = node(3);
                        let value = node(4);
                        let leaf = node(5);
                        weak = WeakPersistent::from_ptr(unsafe { dead.cast::<u8>().add(1) });
                        key_root = Persistent::from_ptr(key);
                        root = Persistent::from_ptr(MakeGarbageCollected(Edges {
                            strong_interior: unsafe { strong.cast::<u8>().add(1) },
                            weak_interior: WeakMember::from_ptr(unsafe {
                                dead.cast::<u8>().add(1)
                            }),
                            key: WeakMember::from_ptr(key),
                            value_key: WeakMember::from_ptr(value),
                            value: Member::from_ptr(value),
                            leaf: Member::from_ptr(leaf),
                        }));
                    }
                    assert_eq!(&*drops.borrow(), &[2]);
                    assert!(weak.Get().is_null());
                    assert!(unsafe { &*root.Get() }.weak_interior.Get().is_null());
                    key_root.Clear();
                    {
                        let _scope = LayoutHeapScope::new();
                    }
                    let mut dead = drops.borrow().clone();
                    dead.sort_unstable();
                    assert_eq!(dead, vec![2, 3, 4, 5]);
                    drop(root);
                    {
                        let _scope = LayoutHeapScope::new();
                    }
                    let mut dead = drops.borrow().clone();
                    dead.sort_unstable();
                    assert_eq!(dead, vec![1, 2, 3, 4, 5]);
                    assert_eq!(LayoutHeapAllocationCountForTesting(), 0);
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
    }
}

#[cfg(test)]
mod reuse_tests {
    use super::*;
    use std::rc::Rc;
    struct Counted {
        traces: Rc<Cell<usize>>,
        drops: Rc<Cell<usize>>,
    }
    impl Traceable for Counted {
        fn Trace(&self, _: &mut Visitor<'_>) {
            self.traces.set(self.traces.get() + 1);
        }
    }
    impl Drop for Counted {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }
    #[test]
    fn root_registration_survives_clear_collection_reassign_clone_and_weak_clear() {
        let _scope = LayoutHeapScope::new();
        let a = MakeGarbageCollected(11u32);
        let b = MakeGarbageCollected(22u32);
        let keeper_a = Persistent::from_ptr(a);
        let keeper_b = Persistent::from_ptr(b);
        let mut root = Persistent::from_ptr(a);
        let mut weak = WeakPersistent::from_ptr(a);
        let count = HEAP.with(|heap| heap.borrow().roots.len());
        for _ in 0..100 {
            root.Assign(b);
            root.Clear();
            root.Assign(a);
            weak.Assign(b);
            weak.Clear();
            weak.Assign(a);
        }
        assert_eq!(HEAP.with(|heap| heap.borrow().roots.len()), count);
        let clone = root.clone();
        assert_eq!(HEAP.with(|heap| heap.borrow().roots.len()), count + 1);
        root.Clear();
        weak.Clear();
        // Nested scope cannot collect; finish the outer scope explicitly.
        drop(_scope);
        assert_eq!(*unsafe { &*clone.Get() }, 11);
        {
            let _scope = LayoutHeapScope::new();
            root.Assign(b);
            weak.Assign(b);
        }
        assert_eq!(HEAP.with(|heap| heap.borrow().roots.len()), count + 1);
        assert_eq!(root.Get(), b);
        drop(keeper_a);
        drop(keeper_b);
        drop(clone);
        root.Clear();
        {
            let _scope = LayoutHeapScope::new();
        }
        assert!(weak.Get().is_null());
        let _scope = LayoutHeapScope::new();
        let c = MakeGarbageCollected(33u32);
        weak.Assign(c);
        let keeper_c = Persistent::from_ptr(c);
        assert_eq!(weak.Get(), keeper_c.Get());
        let slots = HEAP.with(|heap| heap.borrow().roots.len());
        let registered = HEAP.with(|heap| heap.borrow().roots.used);
        for _ in 0..100_000 {
            drop(keeper_c.clone());
        }
        HEAP.with(|heap| {
            let heap = heap.borrow();
            assert!(heap.roots.len() <= slots + 1);
            assert_eq!(heap.roots.used, registered);
            assert!(heap.roots.iter().all(|root| root.upgrade().is_some()));
        });
    }
    #[test]
    fn incremental_sweep_preserves_new_allocations_and_next_collection_root_changes() {
        std::thread::spawn(|| {
            let traces = Rc::new(Cell::new(0));
            let drops = Rc::new(Cell::new(0));
            let make = || Counted {
                traces: traces.clone(),
                drops: drops.clone(),
            };
            let mut keeper;
            let weak;
            {
                let mut scope = LayoutHeapScope::new();
                scope.DeferCollection();
                keeper = Persistent::from_ptr(MakeGarbageCollected(make()));
                weak = WeakPersistent::from_ptr(keeper.Get());
                for _ in 0..32 {
                    MakeGarbageCollected(make());
                }
            }
            RequestLayoutHeapCollection();
            // Mark atomically, then yield before any finalizer. Mutator work
            // is allowed here; it must not be swept using this cycle's marks.
            CollectLayoutHeapIncrementally(std::time::Duration::ZERO);
            assert!(IsLayoutHeapCollectionRequested());
            assert!(!IsLayoutHeapSweepingOnOwningThread());
            assert_eq!(drops.get(), 0);
            let next;
            {
                let mut scope = LayoutHeapScope::new();
                scope.DeferCollection();
                keeper.Clear();
                next = Persistent::from_ptr(MakeGarbageCollected(make()));
            }
            CollectLayoutHeapIncrementally(std::time::Duration::from_secs(1));
            assert!(!IsLayoutHeapCollectionRequested());
            assert_eq!(drops.get(), 32);
            assert!(!weak.Get().is_null());
            assert!(!next.Get().is_null());
            CollectLayoutHeapForTesting();
            assert!(weak.Get().is_null());
            assert_eq!(drops.get(), 33);
            assert_eq!(LayoutHeapAllocationCountForTesting(), 1);
            drop(next);
            CollectLayoutHeapForTesting();
            assert_eq!(drops.get(), 34);
        })
        .join()
        .unwrap();
    }
    #[test]
    fn unchanged_reuse_skips_tracing_but_allocation_and_root_retirement_collect() {
        std::thread::spawn(|| {
            let traces = Rc::new(Cell::new(0));
            let drops = Rc::new(Cell::new(0));
            let mut root;
            {
                let _scope = LayoutHeapScope::new();
                root = Persistent::from_ptr(MakeGarbageCollected(Counted {
                    traces: traces.clone(),
                    drops: drops.clone(),
                }));
            }
            assert_eq!(traces.get(), 1);
            for _ in 0..12 {
                let mut scope = LayoutHeapScope::new();
                scope.AllowUnchangedReuse();
            }
            assert_eq!(
                traces.get(),
                1,
                "cached entry must not trace resident objects again"
            );
            {
                let mut scope = LayoutHeapScope::new();
                scope.AllowUnchangedReuse();
                MakeGarbageCollected(Counted {
                    traces: traces.clone(),
                    drops: drops.clone(),
                });
            }
            assert_eq!(traces.get(), 2);
            assert_eq!(drops.get(), 1, "new unreachable allocations are reclaimed");
            let weak = WeakPersistent::from_ptr(root.Get());
            {
                let mut scope = LayoutHeapScope::new();
                scope.AllowUnchangedReuse();
                root.Clear();
            }
            assert_eq!(drops.get(), 2);
            assert!(weak.Get().is_null());
            assert_eq!(LayoutHeapAllocationCountForTesting(), 0);
        })
        .join()
        .unwrap();
    }
}

#[cfg(test)]
mod worklist_dispatch_lifecycle_tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn failed_initializer_and_trace_reentry_cannot_cancel_pending_payloads() {
        struct Attempts;
        impl Traceable for Attempts {
            fn Trace(&self, _: &mut Visitor<'_>) {
                // collect_marks owns the heap RefMut through every callback.
                assert!(catch_unwind(|| {
                    let _scope = LayoutHeapScope::new();
                })
                .is_err());
                assert!(catch_unwind(|| {
                    MakeGarbageCollected(17u64);
                })
                .is_err());
                assert!(catch_unwind(CollectLayoutHeapForTesting).is_err());
                assert!(
                    catch_unwind(|| FreeLayoutBacking(self as *const Self as *mut Self)).is_err()
                );
                assert!(
                    catch_unwind(|| Persistent::from_ptr(self as *const Self as *mut Self))
                        .is_err()
                );
            }
        }
        std::thread::spawn(|| {
            let root;
            {
                let _scope = LayoutHeapScope::new();
                let before = LayoutHeapAllocationCountForTesting();
                let failed = catch_unwind(|| {
                    // SAFETY: Initialization always panics before writing a T;
                    // the allocator's cancellation path owns all cleanup and no
                    // payload address or descriptor escapes this call.
                    unsafe {
                        MakeGarbageCollectedWithAdditionalBytes::<u64>(8, |_| {
                            panic!("cancel initialization")
                        })
                    };
                });
                assert!(failed.is_err());
                assert_eq!(LayoutHeapAllocationCountForTesting(), before);
                root = Persistent::from_ptr(MakeGarbageCollected(Attempts));
            }
            assert!(!root.Get().is_null());
            assert_eq!(LayoutHeapAllocationCountForTesting(), 1);
            // Repeat a complete collection to verify caught reentry attempts
            // did not poison metadata, heap state or copied dispatch entries.
            {
                let _scope = LayoutHeapScope::new();
            }
            drop(root);
            {
                let _scope = LayoutHeapScope::new();
            }
            assert_eq!(LayoutHeapAllocationCountForTesting(), 0);
        })
        .join()
        .unwrap();
    }

    #[test]
    fn uncaught_trace_panic_retains_pending_payloads_and_existing_failure_state() {
        // This heap is intentionally isolated: the existing collector does not
        // recover its collecting flag after an uncaught Trace panic. Both these
        // types have no owned fields/resources, so thread teardown can deallocate
        // their payloads without introducing leaked native resources.
        struct Exploding {
            next: *const u8,
        }
        impl Traceable for Exploding {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                self.next.Trace(visitor);
                panic!("intentional trace failure after scheduling another object");
            }
        }
        std::thread::spawn(|| {
            let scope = LayoutHeapScope::new();
            let pending = MakeGarbageCollected(19u64);
            let root = Persistent::from_ptr(MakeGarbageCollected(Exploding {
                next: pending.cast(),
            }));
            assert!(catch_unwind(AssertUnwindSafe(|| drop(scope))).is_err());
            HEAP.with(|heap| {
                let heap = heap.borrow();
                assert!(heap.collecting);
                assert_eq!(heap.scopes, 0);
                assert_eq!(heap.objects.len(), 2);
                assert_eq!(heap.worklist.len(), 1);
                assert!(heap.objects[&(pending as usize)].strongly_marked);
                assert!(heap.objects[&(pending as usize)].destroy.is_some());
            });
            assert!(!SWEEPING.get());
            assert!(!root.Get().is_null());
            // No retry/recovery is added; the thread's private heap dies here.
        })
        .join()
        .unwrap();
    }

    #[test]
    fn sweep_reentry_preserves_metadata_until_all_native_destructors_finish() {
        struct DestructorAttempts {
            peer: *mut u64,
            drops: Arc<AtomicUsize>,
        }
        impl Traceable for DestructorAttempts {
            fn Trace(&self, _: &mut Visitor<'_>) {}
        }
        impl Drop for DestructorAttempts {
            fn drop(&mut self) {
                assert!(IsLayoutHeapSweepingOnOwningThread());
                let before = LayoutHeapAllocationCountForTesting();
                assert!(IsManagedLayoutAddress(self.peer));
                assert_eq!(LayoutObjectSize(self.peer), std::mem::size_of::<u64>());
                FreeLayoutBacking(self.peer); // Existing sweeping no-op.
                assert_eq!(LayoutHeapAllocationCountForTesting(), before);
                assert!(catch_unwind(|| {
                    MakeGarbageCollected(23u64);
                })
                .is_err());
                assert!(catch_unwind(|| {
                    let _scope = LayoutHeapScope::new();
                })
                .is_err());
                assert!(catch_unwind(CollectLayoutHeapForTesting).is_err());
                self.drops.fetch_add(1, Ordering::SeqCst);
            }
        }
        std::thread::spawn(|| {
            let drops = Arc::new(AtomicUsize::new(0));
            {
                let _scope = LayoutHeapScope::new();
                let peer = MakeGarbageCollected(29u64);
                MakeGarbageCollected(DestructorAttempts {
                    peer,
                    drops: drops.clone(),
                });
            }
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            assert_eq!(LayoutHeapAllocationCountForTesting(), 0);
            assert!(!IsLayoutHeapSweepingOnOwningThread());
        })
        .join()
        .unwrap();
    }
    #[test]
    fn queued_trace_reads_current_fields_once_for_repeated_interior_edges() {
        use std::rc::Rc;
        struct Leaf {
            name: &'static str,
            traces: Rc<RefCell<Vec<&'static str>>>,
            drops: Rc<RefCell<Vec<&'static str>>>,
        }
        impl Traceable for Leaf {
            fn Trace(&self, _: &mut Visitor<'_>) {
                self.traces.borrow_mut().push(self.name);
            }
        }
        impl Drop for Leaf {
            fn drop(&mut self) {
                self.drops.borrow_mut().push(self.name);
            }
        }
        struct Target {
            edge: Cell<*mut Leaf>,
            traces: Rc<RefCell<Vec<&'static str>>>,
        }
        impl Traceable for Target {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                self.traces.borrow_mut().push("target");
                self.edge.get().Trace(visitor);
            }
        }
        struct Enqueuer {
            target: *mut Target,
            replacement: *mut Leaf,
            traces: Rc<RefCell<Vec<&'static str>>>,
        }
        impl Traceable for Enqueuer {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                self.traces.borrow_mut().push("enqueue");
                self.target.Trace(visitor);
                // All pointers refer to the same allocation; its callback must
                // receive the allocation base once, after this callback returns.
                unsafe { self.target.cast::<u8>().add(1) }.Trace(visitor);
                self.target.Trace(visitor);
                // Trace descriptors may be copied, but traced edges must be read
                // from the actual payload when its queued callback runs.
                unsafe { &*self.target }.edge.set(self.replacement);
            }
        }
        std::thread::spawn(|| {
            let traces = Rc::new(RefCell::new(Vec::new()));
            let drops = Rc::new(RefCell::new(Vec::new()));
            let root;
            {
                let _scope = LayoutHeapScope::new();
                let leaf = |name| {
                    MakeGarbageCollected(Leaf {
                        name,
                        traces: traces.clone(),
                        drops: drops.clone(),
                    })
                };
                let old = leaf("old");
                let replacement = leaf("replacement");
                let target = MakeGarbageCollected(Target {
                    edge: Cell::new(old),
                    traces: traces.clone(),
                });
                root = Persistent::from_ptr(MakeGarbageCollected(Enqueuer {
                    target,
                    replacement,
                    traces: traces.clone(),
                }));
            }
            assert_eq!(&*traces.borrow(), &["enqueue", "target", "replacement"]);
            assert_eq!(&*drops.borrow(), &["old"]);
            assert_eq!(LayoutHeapAllocationCountForTesting(), 3);
            drop(root);
            {
                let _scope = LayoutHeapScope::new();
            }
            assert_eq!(&*drops.borrow(), &["old", "replacement"]);
            assert_eq!(LayoutHeapAllocationCountForTesting(), 0);
        })
        .join()
        .unwrap();
    }
}

/// Requested allocation storage, including uncollected garbage. Excludes
/// separately owned Vec/String payloads and allocator metadata/resident pages.
/// Root counts classify registered cells, not unique managed objects. Container
/// bytes are requested capacities, excluding Arc cells and allocation-map nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutHeapMemoryDiagnostics {
    pub allocated_requested_bytes: usize,
    pub allocated_since_collection_bytes: usize,
    pub allocations: usize,
    pub root_registry_slots: usize,
    pub root_registry_capacity_bytes: usize,
    pub roots_expired: usize,
    pub roots_strong_nonnull: usize,
    pub roots_weak_nonnull: usize,
    pub roots_cleared: usize,
    pub worklist_capacity_bytes: usize,
    pub weak_slots_capacity_bytes: usize,
    pub weak_roots_capacity_bytes: usize,
    pub weak_backings_capacity_bytes: usize,
    pub ephemerons_capacity_bytes: usize,
    pub collection_requested: bool,
    pub active_scopes: usize,
    pub collections_completed: u64,
    pub last_before_bytes: usize,
    pub last_after_bytes: usize,
    pub last_reclaimed_objects: usize,
}
/// Read-only, opt-in O(registered roots) snapshot. Temporary Arc upgrades do not
/// register, clear or retain roots, mark targets, request GC or prune capacity.
pub fn LayoutHeapMemoryDiagnostics() -> LayoutHeapMemoryDiagnostics {
    HEAP.with(|heap| {
        let h = heap.borrow();
        let mut result = LayoutHeapMemoryDiagnostics {
            allocated_requested_bytes: h.heap_bytes,
            allocated_since_collection_bytes: h.allocated_bytes_since_collection,
            allocations: h.objects.len(),
            root_registry_slots: h.roots.len(),
            root_registry_capacity_bytes: h.roots.capacity()
                * std::mem::size_of::<PersistentNode>(),
            worklist_capacity_bytes: h.worklist.capacity() * std::mem::size_of::<TraceWorkItem>(),
            weak_slots_capacity_bytes: h.weak_slots.capacity() * std::mem::size_of::<WeakSlot>(),
            weak_roots_capacity_bytes: h.weak_roots.capacity()
                * std::mem::size_of::<Arc<dyn RootSlot>>(),
            weak_backings_capacity_bytes: h.weak_backings.capacity()
                * std::mem::size_of::<WeakBacking>(),
            ephemerons_capacity_bytes: h.ephemerons.capacity() * std::mem::size_of::<Ephemeron>(),
            collection_requested: h.collection_requested || h.sweep.is_some(),
            active_scopes: h.scopes,
            collections_completed: h.collections_completed,
            last_before_bytes: h.last_before_bytes,
            last_after_bytes: h.last_after_bytes,
            last_reclaimed_objects: h.last_reclaimed_objects,
            ..Default::default()
        };
        for slot in h.roots.iter() {
            match slot.upgrade() {
                None => result.roots_expired += 1,
                Some(root) if root.get().is_null() => result.roots_cleared += 1,
                Some(root) if root.weak() => result.roots_weak_nonnull += 1,
                Some(_) => result.roots_strong_nonnull += 1,
            }
        }
        result
    })
}

#[cfg(test)]
mod memory_diagnostic_tests {
    use super::*;
    #[test]
    fn registry_snapshot_classifies_cells_without_mutating_heap_or_roots() {
        std::thread::spawn(|| {
            let mut scope = LayoutHeapScope::new();
            scope.DeferCollection();
            let object = MakeGarbageCollected(17u32);
            let root = Persistent::from_ptr(object);
            let weak = WeakPersistent::from_ptr(object);
            let mut cleared = root.clone();
            cleared.Clear();
            drop(root.clone());
            drop(scope);
            let before = LayoutHeapMemoryDiagnostics();
            assert_eq!(
                (
                    before.roots_strong_nonnull,
                    before.roots_weak_nonnull,
                    before.roots_cleared,
                    before.roots_expired
                ),
                (1, 1, 0, 0)
            );
            assert!(
                before.root_registry_capacity_bytes
                    >= 4 * std::mem::size_of::<Weak<dyn RootSlot>>()
            );
            for _ in 0..4 {
                assert_eq!(before, LayoutHeapMemoryDiagnostics());
            }
            assert_eq!(unsafe { *root.Get() }, 17);
            assert_eq!(weak.Get(), root.Get());
            drop((root, weak, cleared));
            RequestLayoutHeapCollection();
            assert!(CollectLayoutHeapIfRequested());
            assert_eq!(LayoutHeapMemoryDiagnostics().allocations, 0);
        })
        .join()
        .unwrap();
    }
}
