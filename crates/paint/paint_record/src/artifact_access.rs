//! Sequence-confined storage for a PaintArtifact's display items and chunks.
//!
//! Chromium: third_party/blink/renderer/platform/graphics/paint/
//! paint_controller.cc:283-286,737-820 and display_item_list.h:197-232. The
//! PaintController moves each cached display item and each cached
//! subsequence's chunks and items out of the *current* PaintArtifact into the
//! new one and leaves tombstones behind, while PaintArtifactCompositor and
//! every RasterInvalidator still reference that same current artifact
//! (raster_invalidator.h:174-187 `old_paint_artifact_`) and read the
//! tombstones (display_item_raster_invalidator.cc:51-89). All of it runs on
//! the document's main thread.
//!
//! Rechrom shares the artifact through `Arc`, so its display-item list and
//! chunk list are interior-mutable, confined to the document sequence that
//! recorded them. This follows `paint_model::property_cell`:
//!
//! * [`ArtifactAccess`] is a `!Send` proof that the caller runs on
//!   [`ArtifactAccess::id`]'s sequence and that no writer is active there.
//!   It is only handed out by [`artifact_access`] (shared, nestable) and
//!   [`artifact_writer`] (exclusive). Each returns a scope guard, so a stage
//!   obtains the proof once and borrows whole slices under it.
//! * [`ArtifactData`] holds a value. Reading needs `&'a ArtifactAccess` and
//!   returns `&'a T`; writing in place ([`ArtifactData::modify`]) needs
//!   `&mut ArtifactWriter`, which exists only while no shared scope is live
//!   on the thread, so no `&T` handed out by a read can observe the write.
//!   Only the PaintController writes in place (moving items and chunks out
//!   and leaving tombstones). An owner holding `&mut` uses
//!   [`ArtifactData::make_mut`] (copy-on-write), as the previous
//!   `SharedArtifactData` did.
//! * Holders on other threads (raster workers holding a document image
//!   record's producer) may hold and drop the `Arc`; a read there panics.
//!
//! Soundness rests on: one [`PaintSequenceId`] per thread
//! (`paint_model::property_cell`), the per-thread scope counter below being
//! touched only by the two guards, and `ArtifactAccess` being constructible
//! only here.
use std::cell::{Cell, UnsafeCell};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use paint_model::property_cell::PaintSequenceId;

thread_local! {
    /// > 0: that many shared scopes; -1: one writer scope; 0: none.
    static SCOPES: Cell<isize> = const { Cell::new(0) };
}

/// Shared proof of the document sequence; `&mut ArtifactAccess` (from an
/// [`ArtifactWriterScope`]) is the exclusive write proof.
pub struct ArtifactAccess {
    id: PaintSequenceId,
    _not_send: PhantomData<*const ()>,
}

/// The exclusive form of [`ArtifactAccess`].
pub type ArtifactWriter = ArtifactAccess;

impl ArtifactAccess {
    #[inline]
    fn current() -> Self {
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

impl fmt::Debug for ArtifactAccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ArtifactAccess")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// A live shared proof. Nested scopes are allowed.
pub struct ArtifactAccessScope {
    access: ArtifactAccess,
}

impl Deref for ArtifactAccessScope {
    type Target = ArtifactAccess;
    #[inline]
    fn deref(&self) -> &ArtifactAccess {
        &self.access
    }
}

impl Drop for ArtifactAccessScope {
    #[inline]
    fn drop(&mut self) {
        SCOPES.with(|scopes| scopes.set(scopes.get() - 1));
    }
}

/// A live exclusive proof.
pub struct ArtifactWriterScope {
    access: ArtifactAccess,
}

impl Deref for ArtifactWriterScope {
    type Target = ArtifactAccess;
    #[inline]
    fn deref(&self) -> &ArtifactAccess {
        &self.access
    }
}

impl DerefMut for ArtifactWriterScope {
    #[inline]
    fn deref_mut(&mut self) -> &mut ArtifactAccess {
        &mut self.access
    }
}

impl Drop for ArtifactWriterScope {
    #[inline]
    fn drop(&mut self) {
        SCOPES.with(|scopes| scopes.set(0));
    }
}

#[cold]
#[inline(never)]
fn writer_active() -> ! {
    panic!("paint artifact read while the PaintController writes it")
}

#[cold]
#[inline(never)]
fn scope_active(scopes: isize) -> ! {
    panic!("paint artifact written while {scopes} access scope(s) are live")
}

#[cold]
#[inline(never)]
fn foreign_sequence(owner: PaintSequenceId, access: PaintSequenceId) -> ! {
    panic!("paint artifact owned by sequence {owner:?} accessed from {access:?}")
}

/// The calling thread's shared proof. Panics while a writer is active.
#[inline]
pub fn artifact_access() -> ArtifactAccessScope {
    SCOPES.with(|scopes| {
        let count = scopes.get();
        if count < 0 {
            writer_active();
        }
        scopes.set(count + 1);
    });
    ArtifactAccessScope {
        access: ArtifactAccess::current(),
    }
}

/// The calling thread's exclusive proof. Panics while any scope is active.
#[inline]
pub fn artifact_writer() -> ArtifactWriterScope {
    SCOPES.with(|scopes| {
        let count = scopes.get();
        if count != 0 {
            scope_active(count);
        }
        scopes.set(-1);
    });
    ArtifactWriterScope {
        access: ArtifactAccess::current(),
    }
}

struct ArtifactCell<T> {
    owner: PaintSequenceId,
    value: UnsafeCell<T>,
}

// SAFETY: every shared access is confined to the owner thread by the proof
// check; exclusive access (`make_mut` on a unique Arc, `Drop`) may run on any
// thread, which needs `T: Send` (the `Mutex<T>` bounds).
unsafe impl<T: Send> Send for ArtifactCell<T> {}
unsafe impl<T: Send> Sync for ArtifactCell<T> {}

/// A recording value shared by every `PaintRecording` header that names the
/// same recording, mutable in place only under [`ArtifactWriter`].
pub struct ArtifactData<T>(Arc<ArtifactCell<T>>);

impl<T> ArtifactData<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self(Arc::new(ArtifactCell {
            owner: PaintSequenceId::current(),
            value: UnsafeCell::new(value),
        }))
    }

    #[inline]
    pub fn get<'a>(&'a self, access: &'a ArtifactAccess) -> &'a T {
        let cell = &*self.0;
        if cell.owner != access.id {
            foreign_sequence(cell.owner, access.id);
        }
        // SAFETY: `access` proves this runs on the owning thread with no
        // writer scope active; the only in-place mutation, `modify`, needs
        // `&mut ArtifactWriter`, which cannot exist while `access` (and so
        // the scope it came from) is live for `'a`.
        unsafe { &*cell.value.get() }
    }

    /// In-place mutation seen by every holder of this recording: the
    /// PaintController's moves out of the current artifact.
    #[inline]
    pub fn modify<R>(&self, writer: &mut ArtifactWriter, f: impl FnOnce(&mut T) -> R) -> R {
        let cell = &*self.0;
        if cell.owner != writer.id {
            foreign_sequence(cell.owner, writer.id);
        }
        // SAFETY: owning thread as in `get`; the writer scope proves no
        // shared scope, and so no `&T` from `get`, is live on this thread.
        f(unsafe { &mut *cell.value.get() })
    }

    /// Mutable access for an owner holding `&mut`: in place when this is the
    /// only holder, otherwise a private copy (copy-on-write).
    #[inline]
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        if Arc::get_mut(&mut self.0).is_none() {
            let copy = {
                let access = artifact_access();
                self.get(&access).clone()
            };
            *self = Self::new(copy);
        }
        let cell = Arc::get_mut(&mut self.0).expect("unique artifact data");
        // Exclusive: the value may adopt the calling sequence (a recording
        // built before its Page moved to its document thread).
        cell.owner = PaintSequenceId::current();
        cell.value.get_mut()
    }

    #[inline]
    pub fn PtrEq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    #[inline]
    pub fn owner(&self) -> PaintSequenceId {
        self.0.owner
    }
}

impl<T> Clone for ArtifactData<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T: Default> Default for ArtifactData<T> {
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for ArtifactData<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl From<Vec<crate::RecordedDisplayItem>> for ArtifactData<crate::DisplayItemList> {
    #[inline]
    fn from(value: Vec<crate::RecordedDisplayItem>) -> Self {
        Self::new(crate::DisplayItemList::from(value))
    }
}

impl<T: PartialEq> PartialEq for ArtifactData<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.PtrEq(other) {
            return true;
        }
        let access = artifact_access();
        self.get(&access) == other.get(&access)
    }
}

impl<T> fmt::Debug for ArtifactData<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ArtifactData")
            .field("owner", &self.0.owner)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_are_seen_by_every_holder() {
        let first = ArtifactData::new(vec![1, 2]);
        let second = first.clone();
        {
            let mut writer = artifact_writer();
            first.modify(&mut writer, |value| value[0] = 7);
        }
        let access = artifact_access();
        assert_eq!(second.get(&access), &vec![7, 2]);
    }

    #[test]
    fn make_mut_copies_shared_values() {
        let first = ArtifactData::new(vec![1]);
        let mut second = first.clone();
        second.make_mut().push(2);
        assert!(!first.PtrEq(&second));
        let access = artifact_access();
        assert_eq!(first.get(&access), &vec![1]);
        assert_eq!(second.get(&access), &vec![1, 2]);
    }

    #[test]
    fn writer_under_reader_panics() {
        let nested = std::panic::catch_unwind(|| {
            let _access = artifact_access();
            let _writer = artifact_writer();
        });
        assert!(nested.is_err());
        let reader_under_writer = std::panic::catch_unwind(|| {
            let _writer = artifact_writer();
            let _access = artifact_access();
        });
        assert!(reader_under_writer.is_err());
        // The counter is restored once the panics unwound.
        drop(artifact_writer());
        drop((artifact_access(), artifact_access()));
    }

    #[test]
    fn cross_thread_read_panics_but_drop_is_allowed() {
        let data = ArtifactData::new(vec![1]);
        let remote = data.clone();
        let read = std::thread::spawn(move || {
            let access = artifact_access();
            remote.get(&access).len()
        })
        .join();
        assert!(read.is_err());
        let remote = data.clone();
        std::thread::spawn(move || drop(remote)).join().unwrap();
        let access = artifact_access();
        assert_eq!(data.get(&access).len(), 1);
    }
}
