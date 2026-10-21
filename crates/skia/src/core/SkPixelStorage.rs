// Copyright 2026 Google LLC. BSD-style license.
use std::sync::atomic::{AtomicU32, Ordering};
// Source: include/private/SkPixelStorage.h:20-20
// Rust class ownership is supplied by the containing Arc; no native intrusive-refcount ABI claim.
// Source: include/private/SkPixelStorage.h:22-25
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kTextureProxy,
    kPixelRef,
}
// Source: include/private/SkPixelStorage.h:28-28
// The native default virtual destructor is represented by Rust drop of the complete Arc owner.
// Source: include/private/SkPixelStorage.h:30-31
// Source: include/private/SkPixelStorage.h:33-34
impl SkPixelStorage {
    pub fn getPixelStorageId(&self) -> u32 {
        self.fID
    }
    pub fn type_(&self) -> Type {
        self.fType
    }
    pub fn getContentId(&self) -> u32 {
        self.fContentID.load(Ordering::Relaxed)
    }
    pub fn incrementContentId(&self) -> u32 {
        self.fContentID
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }
}
// Source: include/private/SkPixelStorage.h:37-37
// Source: include/private/SkPixelStorage.h:42-42
// Source: include/private/SkPixelStorage.h:47-47
// Refcount ownership is supplied by the containing Arc, preserving shared storage lifetime.
pub struct SkPixelStorage {
    fType: Type,
    fID: u32,
    fContentID: AtomicU32,
}
// Source: src/core/SkPixelStorage.cpp:11-18
impl SkPixelStorage {
    fn NextId() -> u32 {
        static NEXT_ID: AtomicU32 = AtomicU32::new(1);
        loop {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            if id != 0 {
                return id;
            }
        }
    }
}
// Source: src/core/SkPixelStorage.cpp:20-20
impl SkPixelStorage {
    pub fn new(type_: Type) -> Self {
        Self {
            fType: type_,
            fID: Self::NextId(),
            fContentID: AtomicU32::new(0),
        }
    }
}
// Native base-class upcasting retains the complete allocation owner, rather than cloning base fields.
// The accessor is language glue for inherited non-virtual methods; it supplies no pixel algorithm.
pub trait SkPixelStorageObject {
    fn pixel_storage(&self) -> &SkPixelStorage;
}
impl SkPixelStorageObject for SkPixelStorage {
    fn pixel_storage(&self) -> &SkPixelStorage {
        self
    }
}
