//! Sequence-confined state for Paint property nodes.
//!
//! Chromium: third_party/blink/renderer/platform/graphics/paint/
//! paint_property_node.h:134-145,185-230. A Blink paint property node is one
//! long-lived object whose `State` is replaced in place by `Update` during
//! PrePaint, and whose `changed_`, `changed_sequence_number_`, `cc_node_id_`
//! and GeometryMapper caches are `mutable` members written through `const`
//! references by PaintArtifactCompositor and GeometryMapper. Every reader and
//! writer runs on the document's main thread
//! (`DCHECK_CALLED_ON_VALID_THREAD` in the owners), while retained
//! `PaintArtifact`s on other threads only hold references.
//!
//! Rust expresses that ownership rule with a proof value instead of a
//! debug-only thread check:
//!
//! * [`PropertyAccess`] exists once per thread, inside a thread-local, and is
//!   `!Send + !Sync`; a `&PropertyAccess` is therefore a proof that the caller
//!   runs on the sequence named by [`PropertyAccess::id`].
//! * [`PropertyCell`] holds node state. Reading needs `&'a PropertyAccess` and
//!   returns `&'a T`; replacing needs `&mut PropertyAccess`. The borrow checker
//!   thereby forbids holding a read across an in-place update (the aliasing
//!   rule `RefCell` would check per access) and the owner compare confines all
//!   access to the owning thread.
//! * [`MutableCell`] is the `mutable` member under a shared proof (`Cell`
//!   semantics plus a scoped `&mut` for lazily allocated caches).
//! * `Drop` takes `&mut self`, needs no proof and is sound on any thread: the
//!   last `Arc` owner has exclusive access. Holders on other threads (main cc
//!   layers parked in `FinishCommit`, raster workers holding SVG records) can
//!   name the node and drop it, but cannot obtain a proof for it; an
//!   accidental read panics instead of racing.
//!
//! Soundness rests on three invariants kept by this module only:
//! 1. A [`PaintSequenceId`] is assigned to at most one thread for the life of
//!    the process (monotonic counter, overflow panics).
//! 2. At most one [`PropertyAccess`] exists per thread and it cannot be moved,
//!    cloned, constructed or replaced outside this module.
//! 3. [`with_writer`] hands out the only `&mut PropertyAccess`, through a
//!    `RefCell` that refuses it while any `with_access` borrow is live.
use std::cell::{Cell, RefCell, UnsafeCell};
use std::fmt;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU32, Ordering};

/// One document sequence (thread). Assigned lazily per thread; never reused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PaintSequenceId(NonZeroU32);

impl PaintSequenceId {
    fn allocate() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(1);
        let id = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("paint sequence ids exhausted");
        Self(NonZeroU32::new(id).expect("paint sequence ids start at one"))
    }

    /// The sequence of the calling thread; usable inside any access scope.
    pub fn current() -> Self {
        SEQUENCE.with(|sequence| *sequence)
    }
}

/// Shared proof that the caller runs on [`PropertyAccess::id`]'s sequence;
/// `&mut PropertyAccess` is the exclusive write proof.
pub struct PropertyAccess {
    id: PaintSequenceId,
    _not_send: PhantomData<*const ()>,
}

/// The exclusive form of [`PropertyAccess`].
pub type PropertyWriter = PropertyAccess;

impl PropertyAccess {
    fn new() -> Self {
        Self {
            id: PaintSequenceId::current(),
            _not_send: PhantomData,
        }
    }

    #[inline]
    pub fn id(&self) -> PaintSequenceId {
        self.id
    }

}

/// Chromium: geometry_mapper_transform_cache.cc:15-24 and
/// geometry_mapper_clip_cache.cc:13-21 keep one `s_global_generation` per
/// cache kind for the single main thread. Rechrom runs several Pages per
/// process, so the generation belongs to the sequence that owns the nodes
/// (index 0 transform, index 1 clip). It is a plain sequence-local `Cell`,
/// readable inside any access or writer scope.
fn with_generations<R>(f: impl FnOnce(&Cell<[u32; 2]>) -> R) -> R {
    GENERATIONS.with(f)
}

/// Source: geometry_mapper_transform_cache.h:55-57 `s_global_generation`.
#[inline]
pub fn TransformCacheGeneration() -> u32 {
    with_generations(|generations| generations.get()[0])
}

/// Source: geometry_mapper_clip_cache.h:65-67 `s_global_generation`.
#[inline]
pub fn ClipCacheGeneration() -> u32 {
    with_generations(|generations| generations.get()[1])
}

/// Source: geometry_mapper_transform_cache.cc:15-18 `ClearCache`.
pub fn ClearTransformCache() {
    with_generations(|generations| {
        let [transform, clip] = generations.get();
        generations.set([transform.wrapping_add(1), clip]);
    })
}

/// Source: geometry_mapper_clip_cache.cc:13-16 `ClearCache`.
pub fn ClearClipCache() {
    with_generations(|generations| {
        let [transform, clip] = generations.get();
        generations.set([transform, clip.wrapping_add(1)]);
    })
}

impl fmt::Debug for PropertyAccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PropertyAccess")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

thread_local! {
    static SEQUENCE: PaintSequenceId = PaintSequenceId::allocate();
    static GENERATIONS: Cell<[u32; 2]> = const { Cell::new([1, 1]) };
    static ACCESS: RefCell<PropertyAccess> = RefCell::new(PropertyAccess::new());
}

/// Runs `f` with the calling thread's shared proof. Nested calls are allowed;
/// a call while [`with_writer`] is active on this thread panics.
#[inline]
pub fn with_access<R>(f: impl FnOnce(&PropertyAccess) -> R) -> R {
    ACCESS.with(|access| f(&access.borrow()))
}

/// Runs `f` with the calling thread's exclusive proof. Panics if any
/// [`with_access`] or [`with_writer`] scope is active on this thread.
#[inline]
pub fn with_writer<R>(f: impl FnOnce(&mut PropertyWriter) -> R) -> R {
    ACCESS.with(|access| f(&mut access.borrow_mut()))
}

#[cold]
#[inline(never)]
fn foreign_sequence(owner: PaintSequenceId, access: PaintSequenceId) -> ! {
    panic!("paint property state owned by sequence {owner:?} accessed from {access:?}")
}

#[cold]
#[inline(never)]
fn reentrant_mutable_cell() -> ! {
    panic!("paint property mutable cell accessed while it is mutably borrowed")
}

/// Node state confined to one document sequence.
pub struct PropertyCell<T> {
    owner: PaintSequenceId,
    value: UnsafeCell<T>,
}

impl<T> PropertyCell<T> {
    /// A new cell owned by `access`'s sequence. Creation needs only the
    /// sequence identity: nothing else can reach the new value yet.
    #[inline]
    pub fn new(access: &PropertyAccess, value: T) -> Self {
        Self::with_owner(access.id, value)
    }

    /// A new cell owned by `owner` (normally [`PaintSequenceId::current`]).
    #[inline]
    pub fn with_owner(owner: PaintSequenceId, value: T) -> Self {
        Self {
            owner,
            value: UnsafeCell::new(value),
        }
    }

    #[inline]
    pub fn owner(&self) -> PaintSequenceId {
        self.owner
    }

    #[inline]
    pub fn get<'a>(&'a self, access: &'a PropertyAccess) -> &'a T {
        if self.owner != access.id {
            foreign_sequence(self.owner, access.id);
        }
        // SAFETY: `access` proves this runs on the owning thread (module
        // invariants 1-2), so no other thread touches the value. The only
        // mutation, `set`, needs `&mut PropertyAccess` (invariant 3), which
        // cannot exist while `access` is borrowed for `'a`.
        unsafe { &*self.value.get() }
    }

    #[inline]
    pub fn set(&self, writer: &mut PropertyWriter, value: T) -> T {
        if self.owner != writer.id {
            foreign_sequence(self.owner, writer.id);
        }
        // SAFETY: owning thread as in `get`; `&mut PropertyWriter` proves no
        // `&T` handed out by `get` on this thread is still live.
        unsafe { std::mem::replace(&mut *self.value.get(), value) }
    }

    /// In-place modification under the exclusive proof.
    #[inline]
    pub fn modify<R>(&self, writer: &mut PropertyWriter, f: impl FnOnce(&mut T) -> R) -> R {
        if self.owner != writer.id {
            foreign_sequence(self.owner, writer.id);
        }
        // SAFETY: as in `set`; `f` cannot reach the writer again because it is
        // mutably borrowed by this call.
        f(unsafe { &mut *self.value.get() })
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

// SAFETY: every shared access is confined to the owner thread by the proof
// check; exclusive access (`get_mut`, `into_inner`, `Drop`) may move or drop
// the value on any thread, which needs `T: Send` (the `Mutex<T>` bounds).
unsafe impl<T: Send> Send for PropertyCell<T> {}
unsafe impl<T: Send> Sync for PropertyCell<T> {}

/// Formats the value when the calling thread can read it (owner sequence, no
/// writer scope active); otherwise only the owner.
impl<T: fmt::Debug> fmt::Debug for PropertyCell<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let readable = ACCESS
            .try_with(|access| match access.try_borrow() {
                Ok(access) if access.id == self.owner => Some(self.get(&access).fmt(formatter)),
                _ => None,
            })
            .ok()
            .flatten();
        match readable {
            Some(result) => result,
            None => formatter
                .debug_struct("PropertyCell")
                .field("owner", &self.owner)
                .finish_non_exhaustive(),
        }
    }
}

/// Chromium's `mutable` member: written through a shared proof.
pub struct MutableCell<T> {
    owner: PaintSequenceId,
    borrowed: Cell<bool>,
    value: UnsafeCell<T>,
}

impl<T> MutableCell<T> {
    #[inline]
    pub fn new(access: &PropertyAccess, value: T) -> Self {
        Self::with_owner(access.id, value)
    }

    #[inline]
    pub fn with_owner(owner: PaintSequenceId, value: T) -> Self {
        Self {
            owner,
            borrowed: Cell::new(false),
            value: UnsafeCell::new(value),
        }
    }

    #[inline]
    fn check(&self, access: &PropertyAccess) {
        if self.owner != access.id {
            foreign_sequence(self.owner, access.id);
        }
        if self.borrowed.get() {
            reentrant_mutable_cell();
        }
    }

    #[inline]
    pub fn get(&self, access: &PropertyAccess) -> T
    where
        T: Copy,
    {
        self.check(access);
        // SAFETY: owner thread (proof) and no `&mut T` from `with_mut` is live
        // (flag), so this plain copy cannot race or alias a mutable borrow.
        unsafe { *self.value.get() }
    }

    #[inline]
    pub fn set(&self, access: &PropertyAccess, value: T) {
        drop(self.replace(access, value));
    }

    #[inline]
    pub fn replace(&self, access: &PropertyAccess, value: T) -> T {
        self.check(access);
        // SAFETY: as in `get`; no reference into the value ever escapes a
        // method of this type other than through `with_mut`'s flagged scope.
        unsafe { std::mem::replace(&mut *self.value.get(), value) }
    }

    /// Scoped mutable access; any access to the same cell from inside `f`
    /// panics.
    #[inline]
    pub fn with_mut<R>(&self, access: &PropertyAccess, f: impl FnOnce(&mut T) -> R) -> R {
        self.check(access);
        struct Release<'a>(&'a Cell<bool>);
        impl Drop for Release<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        self.borrowed.set(true);
        let _release = Release(&self.borrowed);
        // SAFETY: owner thread (proof); the flag makes this the only live
        // reference into the value until `_release` runs, also on unwind.
        f(unsafe { &mut *self.value.get() })
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

// SAFETY: as for `PropertyCell`; `borrowed` is only touched after the owner
// check, i.e. on the owner thread.
unsafe impl<T: Send> Send for MutableCell<T> {}
unsafe impl<T: Send> Sync for MutableCell<T> {}

impl<T> fmt::Debug for MutableCell<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MutableCell")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn same_thread_reads_and_writes() {
        let cell = with_access(|a| PropertyCell::new(a, 1));
        with_writer(|w| assert_eq!(cell.set(w, 2), 1));
        with_access(|a| assert_eq!(*cell.get(a), 2));
        let mutable = with_access(|a| MutableCell::new(a, 3));
        with_access(|a| {
            mutable.set(a, 4);
            mutable.with_mut(a, |value| *value += 1);
            assert_eq!(mutable.get(a), 5);
        });
    }

    #[test]
    fn cross_thread_get_panics() {
        let cell = Arc::new(with_access(|a| PropertyCell::new(a, 1)));
        let remote = Arc::clone(&cell);
        let result = std::thread::spawn(move || with_access(|a| *remote.get(a))).join();
        assert!(result.is_err());
        with_access(|a| assert_eq!(*cell.get(a), 1));
    }

    #[test]
    fn cross_thread_mutable_cell_panics() {
        let cell = Arc::new(with_access(|a| MutableCell::new(a, 1)));
        let remote = Arc::clone(&cell);
        let result = std::thread::spawn(move || with_access(|a| remote.set(a, 2))).join();
        assert!(result.is_err());
        with_access(|a| assert_eq!(cell.get(a), 1));
    }

    #[test]
    fn reentrant_writer_panics() {
        let nested = std::panic::catch_unwind(|| with_writer(|_| with_writer(|_| ())));
        assert!(nested.is_err());
        let under_reader = std::panic::catch_unwind(|| with_access(|_| with_writer(|_| ())));
        assert!(under_reader.is_err());
        // The thread-local is usable again after the panics unwound.
        with_writer(|_| ());
    }

    #[test]
    fn reentrant_mutable_cell_panics() {
        let cell = with_access(|a| MutableCell::new(a, 1));
        let nested = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_access(|a| cell.with_mut(a, |_| cell.get(a)))
        }));
        assert!(nested.is_err());
        with_access(|a| assert_eq!(cell.get(a), 1));
    }

    #[test]
    fn foreign_thread_drop_is_allowed() {
        struct Dropped(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Dropped {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }
        let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cell = Arc::new(with_access(|a| PropertyCell::new(a, Dropped(flag.clone()))));
        std::thread::spawn(move || drop(cell)).join().unwrap();
        assert!(flag.load(Ordering::Relaxed));
    }

    #[test]
    fn sequences_are_distinct_per_thread() {
        let here = PaintSequenceId::current();
        let there = std::thread::spawn(PaintSequenceId::current).join().unwrap();
        assert_ne!(here, there);
        assert_eq!(here, PaintSequenceId::current());
    }

    #[test]
    fn cache_generations_follow_chromium_clear_cache() {
        let (transform, clip) = (TransformCacheGeneration(), ClipCacheGeneration());
        with_writer(|_| ClearClipCache());
        assert_eq!(TransformCacheGeneration(), transform);
        assert_ne!(ClipCacheGeneration(), clip);
        ClearTransformCache();
        assert_ne!(TransformCacheGeneration(), transform);
        let other = std::thread::spawn(TransformCacheGeneration).join().unwrap();
        assert_eq!(other, 1);
    }
}
