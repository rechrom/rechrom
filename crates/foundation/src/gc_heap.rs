// C++: foundation/blink_base/heap/{heap.h,heap.cc,visitor.h,persistent.h,garbage_collected.h}.
// This is the single-thread-owned layout heap. Collection is synchronous at the
// end of the outermost scope, so pointers stay stable throughout layout.
use crate::{kMemberDeletedValue, Member, UntracedMember, WeakMember};
use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
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
    fn clear(&self);
    fn weak(&self) -> bool;
}

struct HeapState {
    scopes: usize,
    collecting: bool,
    activity_since_collection: bool,
    heap_bytes: usize,
    allocated_bytes_since_collection: usize,
    collection_requested: bool,
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
    roots: Vec<Weak<dyn RootSlot>>,
    worklist: Vec<TraceWorkItem>,
    weak_slots: Vec<WeakSlot>,
    weak_roots: Vec<Arc<dyn RootSlot>>,
    weak_backings: Vec<WeakBacking>,
    ephemerons: Vec<Ephemeron>,
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
            profile_collection: false,
            profile_mark_edges: 0,
            profile_mark_cache_hits: 0,
            strong_marked_ranges: [(0, 0); 4],
            next_strong_marked_range: 0,
            strong_marked_addresses: [0; 1024],
            objects: BTreeMap::new(),
            roots: Vec::new(),
            worklist: Vec::new(),
            weak_slots: Vec::new(),
            weak_roots: Vec::new(),
            weak_backings: Vec::new(),
            ephemerons: Vec::new(),
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
        // Upgrade and prune the root registry in one pass. Every successful
        // upgrade remains held through dispatch, including cleared root cells.
        // A separate strong_count scan would visit the same registry twice.
        let mut roots = Vec::with_capacity(self.roots.len());
        self.roots.retain(|root| {
            if let Some(root) = root.upgrade() {
                roots.push(root);
                true
            } else {
                false
            }
        });
        for object in self.objects.values_mut() {
            object.marked = false;
            object.strongly_marked = false;
        }
        self.worklist.clear();
        self.weak_slots.clear();
        self.weak_roots.clear();
        self.weak_backings.clear();
        self.ephemerons.clear();
        for root in roots {
            if root.weak() {
                self.weak_roots.push(root);
            } else {
                self.mark(root.get());
            }
        }
        let roots_done = profile.map(|start| start.elapsed());
        let mut visitor = Visitor { heap: self };
        let mut traced_objects = 0;
        loop {
            while let Some(item) = visitor.heap.worklist.pop() {
                if profile.is_some() {
                    traced_objects += 1;
                }
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
        let trace_done = profile.map(|start| start.elapsed());
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
            for root in &visitor.heap.weak_roots {
                if !broker.IsHeapObjectAlive(root.get()) {
                    root.clear();
                }
            }
        }
        let weak_done = profile.map(|start| start.elapsed());
        let dead: Vec<_> = visitor
            .heap
            .objects
            .iter()
            .filter_map(|(&address, object)| (!object.marked).then_some(address))
            .collect();
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

fn collect() {
    let mut trace = browser_tracing::span("gc", "LayoutHeap.Collect");
    if browser_tracing::enabled() {
        HEAP.with(|heap| {
            let heap = heap.borrow();
            trace.set("objects", heap.objects.len() as f64);
            trace.set("roots", heap.roots.len() as f64);
        });
    }
    let dead = HEAP.with(|heap| heap.borrow_mut().collect_marks());
    trace.set("reclaimed_objects", dead.len() as f64);
    SWEEPING.set(true);
    // Keep every allocation mapped while native destructors run. Destructors
    // may query object sizes and release Persistent roots.
    for address in &dead {
        let destroy = HEAP.with(|heap| {
            heap.borrow_mut()
                .objects
                .get_mut(address)
                .unwrap()
                .destroy
                .take()
        });
        if let Some(destroy) = destroy {
            unsafe { destroy(*address as *mut u8) };
        }
    }
    HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        for address in dead {
            heap.objects.remove(&address);
        }
        heap.heap_bytes = heap
            .objects
            .values()
            .map(|object| object.layout.size())
            .sum();
        heap.weak_slots.clear();
        heap.weak_roots.clear();
        heap.weak_backings.clear();
        heap.ephemerons.clear();
        heap.collecting = false;
    });
    SWEEPING.set(false);
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
    let pointer = HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        assert!(
            heap.scopes > 0 && !heap.collecting,
            "layout allocation requires LayoutHeapScope"
        );
        let address = unsafe { alloc_zeroed(layout) };
        let address = NonNull::new(address).unwrap_or_else(|| handle_alloc_error(layout));
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
                trace: None,
                destroy: None,
                marked: false,
                strongly_marked: false,
            },
        );
        heap.heap_bytes = heap.heap_bytes.saturating_add(layout.size());
        address.as_ptr() as *mut T
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| init(pointer)));
    if let Err(error) = result {
        HEAP.with(|heap| {
            let mut heap = heap.borrow_mut();
            if let Some(object) = heap.objects.remove(&(pointer as usize)) {
                heap.heap_bytes = heap.heap_bytes.saturating_sub(object.layout.size());
            }
        });
        std::panic::resume_unwind(error);
    }
    HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        let object = heap.objects.get_mut(&(pointer as usize)).unwrap();
        object.trace = Some(trace_object::<T>);
        object.destroy = Some(destroy_object::<T>);
    });
    pointer
}

struct RootCell<T> {
    pointer: AtomicPtr<T>,
    weak: bool,
    owner: std::thread::ThreadId,
}
impl<T> RootSlot for RootCell<T> {
    fn get(&self) -> *const u8 {
        self.pointer.load(Ordering::Relaxed) as *const u8
    }
    fn clear(&self) {
        self.pointer.store(std::ptr::null_mut(), Ordering::Relaxed);
    }
    fn weak(&self) -> bool {
        self.weak
    }
}

// cpp: foundation/blink_base/heap/persistent.h:12-78
pub struct BasicPersistent<T, const WEAK: bool> {
    slot: Arc<RootCell<T>>,
    registered: bool,
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
            }),
            registered: false,
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
        if !pointer.is_null() {
            assert!(
                IsManagedLayoutAddress(pointer),
                "Persistent requires a managed layout object"
            );
        }
        if !pointer.is_null() {
            // cppgc BasicPersistent::Assign reuses its existing PersistentNode.
            // Our weak-slot registry retains live cells, including cleared
            // cells, until their owner drops; registration is therefore once
            // per cell, without scanning every existing root on each Assign.
            HEAP.with(|heap| {
                let mut heap = heap.borrow_mut();
                assert!(!heap.collecting);
                if !self.registered {
                    let slot: Arc<dyn RootSlot> = self.slot.clone();
                    heap.roots.push(Arc::downgrade(&slot));
                    self.registered = true;
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
        if !self.slot.pointer.load(Ordering::Relaxed).is_null() {
            HEAP.with(|heap| heap.borrow_mut().activity_since_collection = true);
        }
        self.slot
            .pointer
            .store(std::ptr::null_mut(), Ordering::Relaxed);
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
    HEAP.with(|heap| heap.borrow().collection_requested)
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
        !heap.collecting && heap.collection_requested
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
