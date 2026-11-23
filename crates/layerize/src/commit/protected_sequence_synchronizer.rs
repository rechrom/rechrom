// Translation status: complete
// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/protected_sequence_synchronizer.h:1-221.

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

// Source: cc/base/protected_sequence_synchronizer.h:11-41.
pub trait ProtectedSequenceSynchronizer {
    fn IsOwnerThread(&self) -> bool;
    fn InProtectedSequence(&self) -> bool;
    fn WaitForProtectedSequenceCompletion(&self);
}

// Each wrapper keeps one resident value. Chromium's external protected
// sequence supplies the ordering contract; the local RwLock expresses that
// contract safely when the same Layer is shared across Rust threads. It does
// not create a thread, queue, copy, or document-wide lock.

// Source: cc/base/protected_sequence_synchronizer.h:43-69.
pub struct ProtectedSequenceForbidden<T> {
    value_: RwLock<T>,
}
impl<T> ProtectedSequenceForbidden<T> {
    pub fn new(value: T) -> Self {
        Self {
            value_: RwLock::new(value),
        }
    }
    pub fn Read(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockReadGuard<'_, T> {
        debug_assert!(synchronizer.IsOwnerThread());
        self.value_.read().unwrap()
    }
    pub fn Write(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockWriteGuard<'_, T> {
        self.WriteAccess(synchronizer).Write()
    }
}

// Source: cc/base/protected_sequence_synchronizer.h:71-99.
pub struct ProtectedSequenceReadable<T> {
    value_: RwLock<T>,
}
impl<T> ProtectedSequenceReadable<T> {
    pub fn new(value: T) -> Self {
        Self {
            value_: RwLock::new(value),
        }
    }
    pub fn Read(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockReadGuard<'_, T> {
        debug_assert!(synchronizer.IsOwnerThread() || synchronizer.InProtectedSequence());
        self.value_.read().unwrap()
    }
    pub fn Write(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockWriteGuard<'_, T> {
        self.WriteAccess(synchronizer).Write()
    }
}

// Source: cc/base/protected_sequence_synchronizer.h:101-140.
pub struct ProtectedSequenceWritable<T> {
    value_: RwLock<T>,
}
impl<T> ProtectedSequenceWritable<T> {
    pub fn new(value: T) -> Self {
        Self {
            value_: RwLock::new(value),
        }
    }
    pub fn Read(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockReadGuard<'_, T> {
        debug_assert!(synchronizer.IsOwnerThread() || synchronizer.InProtectedSequence());
        if synchronizer.IsOwnerThread() {
            synchronizer.WaitForProtectedSequenceCompletion();
        }
        self.value_.read().unwrap()
    }
    pub fn Write(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockWriteGuard<'_, T> {
        self.WriteAccess(synchronizer).Write()
    }
}

// Rust spells the three std::unique_ptr<T> specializations as distinct Box
// wrappers. Their read guards expose the same nullable resident owner; callers
// use `as_deref()` without cloning the pointee.

// Source: cc/base/protected_sequence_synchronizer.h:142-168.
pub struct ProtectedSequenceForbiddenBox<T> {
    value_: RwLock<Option<Box<T>>>,
}
impl<T> ProtectedSequenceForbiddenBox<T> {
    pub fn new(value: Option<Box<T>>) -> Self {
        Self {
            value_: RwLock::new(value),
        }
    }
    pub fn Read(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockReadGuard<'_, Option<Box<T>>> {
        debug_assert!(synchronizer.IsOwnerThread());
        self.value_.read().unwrap()
    }
    pub fn Write(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockWriteGuard<'_, Option<Box<T>>> {
        self.WriteAccess(synchronizer).Write()
    }
}

// Source: cc/base/protected_sequence_synchronizer.h:170-192.
pub struct ProtectedSequenceReadableBox<T> {
    value_: RwLock<Option<Box<T>>>,
}
impl<T> ProtectedSequenceReadableBox<T> {
    pub fn new(value: Option<Box<T>>) -> Self {
        Self {
            value_: RwLock::new(value),
        }
    }
    pub fn Read(
        &self,
        _synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockReadGuard<'_, Option<Box<T>>> {
        self.value_.read().unwrap()
    }
    pub fn Write(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockWriteGuard<'_, Option<Box<T>>> {
        self.WriteAccess(synchronizer).Write()
    }
}

// Source: cc/base/protected_sequence_synchronizer.h:194-217.
pub struct ProtectedSequenceWritableBox<T> {
    value_: RwLock<Option<Box<T>>>,
}
impl<T> ProtectedSequenceWritableBox<T> {
    pub fn new(value: Option<Box<T>>) -> Self {
        Self {
            value_: RwLock::new(value),
        }
    }
    pub fn Read(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockReadGuard<'_, Option<Box<T>>> {
        debug_assert!(synchronizer.IsOwnerThread() || synchronizer.InProtectedSequence());
        if synchronizer.IsOwnerThread() {
            synchronizer.WaitForProtectedSequenceCompletion();
        }
        self.value_.read().unwrap()
    }
    pub fn Write(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> RwLockWriteGuard<'_, Option<Box<T>>> {
        self.WriteAccess(synchronizer).Write()
    }
}

// Source: cc/base/protected_sequence_synchronizer.h:219-221.
// Namespace and include-guard closure map to the Rust module boundary.

// Rust scoped expression of the native T& returned by Write(). The source
// synchronization check/wait happens exactly once, before this access exists.
// Short locks permit legal native alias/reentrant calls on the same field.
// This capability borrows its wrapper, is neither Send nor Sync nor Clone,
// and carries no T/host/sequence flags or queued callback.
pub struct WriteAccess<'a, T> {
    value: &'a RwLock<T>,
    no_send_sync: std::marker::PhantomData<dyn FnMut()>,
}
impl<'a, T> WriteAccess<'a, T> {
    fn new(value: &'a RwLock<T>) -> Self {
        Self {
            value,
            no_send_sync: std::marker::PhantomData,
        }
    }
    pub fn Read(&self) -> RwLockReadGuard<'a, T> {
        self.value.read().unwrap()
    }
    pub fn Write(&self) -> RwLockWriteGuard<'a, T> {
        self.value.write().unwrap()
    }
}
impl<T> ProtectedSequenceForbidden<T> {
    // Source: cc/base/protected_sequence_synchronizer.h:61-64.
    pub fn WriteAccess(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> WriteAccess<'_, T> {
        debug_assert!(synchronizer.IsOwnerThread());
        WriteAccess::new(&self.value_)
    }
}
impl<T> ProtectedSequenceReadable<T> {
    // Source: cc/base/protected_sequence_synchronizer.h:91-95.
    pub fn WriteAccess(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> WriteAccess<'_, T> {
        debug_assert!(synchronizer.IsOwnerThread());
        synchronizer.WaitForProtectedSequenceCompletion();
        WriteAccess::new(&self.value_)
    }
}
impl<T> ProtectedSequenceWritable<T> {
    // Source: cc/base/protected_sequence_synchronizer.h:127-132.
    pub fn WriteAccess(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> WriteAccess<'_, T> {
        debug_assert!(synchronizer.IsOwnerThread() || synchronizer.InProtectedSequence());
        if synchronizer.IsOwnerThread() {
            synchronizer.WaitForProtectedSequenceCompletion();
        }
        WriteAccess::new(&self.value_)
    }
}
impl<T> ProtectedSequenceForbiddenBox<T> {
    // Source: cc/base/protected_sequence_synchronizer.h:156-159.
    pub fn WriteAccess(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> WriteAccess<'_, Option<Box<T>>> {
        debug_assert!(synchronizer.IsOwnerThread());
        WriteAccess::new(&self.value_)
    }
}
impl<T> ProtectedSequenceReadableBox<T> {
    // Source: cc/base/protected_sequence_synchronizer.h:180-184.
    pub fn WriteAccess(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> WriteAccess<'_, Option<Box<T>>> {
        debug_assert!(synchronizer.IsOwnerThread());
        synchronizer.WaitForProtectedSequenceCompletion();
        WriteAccess::new(&self.value_)
    }
}
impl<T> ProtectedSequenceWritableBox<T> {
    // Source: cc/base/protected_sequence_synchronizer.h:208-213.
    pub fn WriteAccess(
        &self,
        synchronizer: &(impl ProtectedSequenceSynchronizer + ?Sized),
    ) -> WriteAccess<'_, Option<Box<T>>> {
        debug_assert!(synchronizer.IsOwnerThread() || synchronizer.InProtectedSequence());
        if synchronizer.IsOwnerThread() {
            synchronizer.WaitForProtectedSequenceCompletion();
        }
        WriteAccess::new(&self.value_)
    }
}
