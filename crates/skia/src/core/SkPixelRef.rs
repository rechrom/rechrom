//! Restricted SkPixelRef pixel ownership for tightly packed 8888 storage.
//!
//! Official SkBitmap::installPixels attaches a SkPixelRef and release callback;
//! SkSurfaces::WrapPixels and SkCanvas::MakeRasterDirect retain those pixels.
//! This Rust adapter makes writable ownership exclusive instead of sharing a
//! mutable C++ ref-counted pointer. The owner replaces releaseProc/context.
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

enum Storage {
    Owned(Vec<u8>),
    External {
        pixels: NonNull<u8>,
        len: usize,
        owner: Box<dyn Any + Send>,
    },
}

/// Exclusively owned writable pixel storage. External mappings never become a
/// Vec, cannot be reallocated, and are released exactly when their owner drops.
pub struct PixelStorage(Storage);

// SAFETY: from_external requires the entire mapping and its exclusive access
// to be transferable between threads; Owned(Vec) and the retained owner are Send.
unsafe impl Send for PixelStorage {}

impl PixelStorage {
    pub fn owned(data: Vec<u8>) -> Self {
        Self(Storage::Owned(data))
    }

    /// Takes exclusive ownership of an existing initialized mapping.
    ///
    /// # Safety
    /// `pixels` must identify `len` initialized, readable and writable bytes in
    /// a single allocation. That address must remain stable and valid until
    /// `owner` drops, and owner drop must release/unmap it as appropriate. No
    /// other CPU or GPU user may access these bytes while this storage or any
    /// slice borrowed from it exists. The mapping's exclusive ownership and
    /// access must be transferable between threads. Accessing the retained
    /// owner must not resize, unmap or otherwise invalidate this mapping.
    pub unsafe fn from_external(
        pixels: NonNull<u8>,
        len: usize,
        owner: Box<dyn Any + Send>,
    ) -> Option<Self> {
        if len == 0 || len > isize::MAX as usize {
            return None;
        }
        Some(Self(Storage::External { pixels, len, owner }))
    }

    pub fn as_slice(&self) -> &[u8] {
        match &self.0 {
            Storage::Owned(data) => data,
            Storage::External { pixels, len, .. } => {
                // SAFETY: the constructor's exclusive, stable mapping contract
                // is retained by owner; this slice borrows storage immutably.
                unsafe { core::slice::from_raw_parts(pixels.as_ptr(), *len) }
            }
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        match &mut self.0 {
            Storage::Owned(data) => data,
            Storage::External { pixels, len, .. } => {
                // SAFETY: construction guarantees exclusive access and the
                // returned slice exclusively borrows this storage.
                unsafe { core::slice::from_raw_parts_mut(pixels.as_ptr(), *len) }
            }
        }
    }

    /// Inspect the native target identity while storage retains its mapping.
    pub fn external_owner(&self) -> Option<&(dyn Any + Send)> {
        match &self.0 {
            Storage::External { owner, .. } => Some(owner.as_ref()),
            Storage::Owned(_) => None,
        }
    }

    /// Consume storage before handing the owner back to native presentation.
    /// No pixel reference survives this operation.
    pub fn into_external_owner(self) -> Option<Box<dyn Any + Send>> {
        match self.0 {
            Storage::External { owner, .. } => Some(owner),
            Storage::Owned(_) => None,
        }
    }

    /// Legacy readback: retain an owned Vec, or explicitly copy an external
    /// mapping. Direct presentation must consume storage itself instead.
    pub fn into_vec(self) -> Vec<u8> {
        match self.0 {
            Storage::Owned(data) => data,
            Storage::External { pixels, len, owner } => {
                // SAFETY: owner is live through the completed copy.
                let result = unsafe { core::slice::from_raw_parts(pixels.as_ptr(), len) }.to_vec();
                drop(owner);
                result
            }
        }
    }
}

impl Clone for PixelStorage {
    fn clone(&self) -> Self {
        // Never duplicate a mutable native mapping or its release obligation.
        Self::owned(self.as_slice().to_vec())
    }
}
impl PartialEq for PixelStorage {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Deref for PixelStorage {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl DerefMut for PixelStorage {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}
