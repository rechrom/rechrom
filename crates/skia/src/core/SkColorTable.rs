// Copyright 2023 Google LLC. BSD-style license.
use crate::src::core::{SkBitmap::SkBitmap, SkImageInfo::SkImageInfo};
use std::sync::Arc;

// Source: include/core/SkColorTable.h:25-25
// Source: include/core/SkColorTable.h:59-59
// Preserve the immutable bitmap owner and shared pixel reference, rather than copying its bytes.
pub struct SkColorTable {
    fTable: SkBitmap,
}
// Adapted: the native immutable sharing guarantee is expressed on this restricted owner,
// not arbitrary SkBitmap. The only constructor is private and reached only after MakeARGB
// allocates fresh A8 pixels, finishes all writes and sets immutable. No safe API exposes
// mutable byte references; bitmap() only borrows the owner and raw-pointer writes are unsafe.
// The retained native PixelRef is itself Send+Sync and keeps allocation/destruction alive.
unsafe impl Send for SkColorTable {}
unsafe impl Sync for SkColorTable {}
impl SkColorTable {
    // Source: include/core/SkColorTable.h:29-31
    pub fn Make(table: Option<&[u8; 256]>) -> Option<Arc<Self>> {
        Self::MakeARGB(table, table, table, table)
    }
    // Source: include/core/SkColorTable.h:35-38
    // Source: src/core/SkColorTable.cpp:14-39
    pub fn MakeARGB(
        tableA: Option<&[u8; 256]>,
        tableR: Option<&[u8; 256]>,
        tableG: Option<&[u8; 256]>,
        tableB: Option<&[u8; 256]>,
    ) -> Option<Arc<Self>> {
        if tableA.is_none() && tableR.is_none() && tableG.is_none() && tableB.is_none() {
            return None;
        }
        let mut table = SkBitmap::default();
        if !table.tryAllocPixels(&SkImageInfo::MakeA8(256, 4), 0) {
            return None;
        }
        let a = table.getAddr8(0, 0);
        let r = table.getAddr8(0, 1);
        let g = table.getAddr8(0, 2);
        let b = table.getAddr8(0, 3);
        for i in 0..256 {
            // The fresh bitmap owns all four 256-byte A8 rows exclusively until publication.
            unsafe {
                a.add(i).write(tableA.map_or(i as u8, |t| t[i]));
                r.add(i).write(tableR.map_or(i as u8, |t| t[i]));
                g.add(i).write(tableG.map_or(i as u8, |t| t[i]));
                b.add(i).write(tableB.map_or(i as u8, |t| t[i]));
            }
        }
        table.setImmutable();
        Some(Arc::new(Self::new(&table)))
    }
    // Source: include/core/SkColorTable.h:41-44
    pub fn alphaTable(&self) -> *const u8 {
        self.fTable.getAddr8(0, 0)
    }
    pub fn redTable(&self) -> *const u8 {
        self.fTable.getAddr8(0, 1)
    }
    pub fn greenTable(&self) -> *const u8 {
        self.fTable.getAddr8(0, 2)
    }
    pub fn blueTable(&self) -> *const u8 {
        self.fTable.getAddr8(0, 3)
    }
    // Source: include/core/SkColorTable.h:53-53
    fn new(table: &SkBitmap) -> Self {
        Self {
            fTable: table.clone(),
        }
    }
    // Source: include/core/SkColorTable.h:57-57
    pub(crate) fn bitmap(&self) -> &SkBitmap {
        &self.fTable
    }
}
// Source: src/core/SkColorTable.cpp:41-43
impl SkColorTable {
    pub fn flatten(&self, buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer) {
        unsafe {
            buffer.writeByteArray(self.fTable.getAddr8(0, 0), 4 * 256);
        }
    }
    // Source: src/core/SkColorTable.cpp:45-51
    pub fn Deserialize(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> Option<Arc<Self>> {
        let mut argb = [0u8; 4 * 256];
        if buffer.readByteArray(&mut argb) {
            return Self::MakeARGB(
                Some(argb[0..256].try_into().unwrap()),
                Some(argb[256..512].try_into().unwrap()),
                Some(argb[512..768].try_into().unwrap()),
                Some(argb[768..1024].try_into().unwrap()),
            );
        }
        None
    }
}
