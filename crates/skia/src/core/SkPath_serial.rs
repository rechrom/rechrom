use crate::include::core::SkPathTypes::{SkPathDirection, SkPathFillType};
use crate::src::core::{
    SkBuffer::{SkRBuffer, SkWBuffer},
    SkData::SkData,
    SkPath::SkPath,
    SkPoint::SkPoint,
    SkRRect::SkRRect,
    SkRRectPriv::SkRRectPriv,
    SkSafeMath::SkSafeMath,
};
use std::sync::Arc;
// Source: src/core/SkPath_serial.cpp:32-38
const TYPE_SHIFT: u32 = 28;
const DIRECTION_SHIFT: u32 = 26;
const FILL_SHIFT: u32 = 8;
const VERSION_MASK: u32 = 255;
// Source: src/core/SkPath_serial.cpp:40-49
const JUST_PUBLIC_DATA: u32 = 4;
const VERBS_FORWARD: u32 = 5;
const MIN_VERSION: u32 = JUST_PUBLIC_DATA;
const CURRENT_VERSION: u32 = VERBS_FORWARD;
// Source: src/core/SkPath_serial.cpp:51-54
const GENERAL: u32 = 0;
const RRECT: u32 = 1;
// Source: src/core/SkPath_serial.cpp:56-58
fn extract_version(packed: u32) -> u32 {
    packed & VERSION_MASK
}
// Source: src/core/SkPath_serial.cpp:60-62
fn extract_filltype(packed: u32) -> SkPathFillType {
    match (packed >> FILL_SHIFT) & 3 {
        0 => SkPathFillType::kWinding,
        1 => SkPathFillType::kEvenOdd,
        2 => SkPathFillType::kInverseWinding,
        _ => SkPathFillType::kInverseEvenOdd,
    }
}
// Source: src/core/SkPath_serial.cpp:64-66
fn extract_serializationtype(packed: u32) -> u32 {
    (packed >> TYPE_SHIFT) & 15
}
impl SkPath {
    // Source: src/core/SkPath_serial.cpp:70-106
    unsafe fn writeToMemoryAsRRect(&self, storage: *mut u8) -> usize {
        let mut rrect = SkRRect::default();
        let first_dir;
        let start;
        if let Some(info) = self.getOvalInfo() {
            rrect.setOval(&info.fBounds);
            first_dir = info.fDirection;
            start = info.fStartIndex as u32 * 2;
        } else if let Some(info) = self.getRRectInfo() {
            rrect = info.fRRect;
            first_dir = info.fDirection;
            start = info.fStartIndex as u32;
        } else {
            return 0;
        }
        let size_needed = 4 + SkRRect::kSizeInMemory + 4;
        if storage.is_null() {
            return size_needed;
        }
        let packed = ((self.getFillType() as u32) << FILL_SHIFT)
            | ((first_dir as u32) << DIRECTION_SHIFT)
            | (RRECT << TYPE_SHIFT)
            | CURRENT_VERSION;
        let mut buffer = unsafe { SkWBuffer::new(storage) };
        unsafe {
            buffer.write32(packed as i32);
            SkRRectPriv::WriteToBuffer(&rrect, &mut buffer);
            buffer.write32(start as i32);
            buffer.padToAlign4();
        }
        crate::SkASSERT!(@source b"src/core/SkPath_serial.cpp\0".as_ptr(),104; @condition b"sizeNeeded == buffer.pos()\0".as_ptr(); (size_needed)==(buffer.pos()));
        buffer.pos()
    }
    // Source: src/core/SkPath_serial.cpp:108-152
    /// Safety: non-null storage covers the returned size exclusively. Null requests the size.
    pub unsafe fn writeToMemory(&self, storage: *mut u8) -> usize {
        // Source: src/core/SkPath_serial.cpp:109; src/core/SkPath.cpp:96-98
        #[cfg(sk_debug)]
        self.validate();
        let bytes = unsafe { self.writeToMemoryAsRRect(storage) };
        if bytes != 0 {
            return bytes;
        }
        let packed =
            ((self.getFillType() as u32) << FILL_SHIFT) | (GENERAL << TYPE_SHIFT) | CURRENT_VERSION;
        let points = self.points();
        let verbs = self.verbs();
        let conics = self.conicWeights();
        let pts = points.len() as i32;
        let cnx = conics.len() as i32;
        let vbs = verbs.len() as i32;
        let mut safe = SkSafeMath::new();
        let mut size = 16;
        let point_bytes = safe.mul(pts as usize, core::mem::size_of::<SkPoint>());
        size = safe.add(size, point_bytes);
        let conic_bytes = safe.mul(cnx as usize, 4);
        size = safe.add(size, conic_bytes);
        let verb_bytes = safe.mul(vbs as usize, 1);
        size = safe.add(size, verb_bytes);
        size = safe.alignUp(size, 4);
        if !safe.ok() {
            return 0;
        }
        if storage.is_null() {
            return size;
        }
        let mut buffer = unsafe { SkWBuffer::new(storage) };
        unsafe {
            buffer.write32(packed as i32);
            buffer.write32(pts);
            buffer.write32(cnx);
            buffer.write32(vbs);
            buffer.write(points.as_ptr().cast(), core::mem::size_of_val(points));
            buffer.write(conics.as_ptr().cast(), core::mem::size_of_val(conics));
            buffer.write(verbs.as_ptr().cast(), core::mem::size_of_val(verbs));
            buffer.padToAlign4();
        }
        crate::SkASSERT!(@source b"src/core/SkPath_serial.cpp\0".as_ptr(),150; @condition b"buffer.pos() == size\0".as_ptr(); (buffer.pos())==(size));
        size
    }
    // Source: src/core/SkPath_serial.cpp:154-159
    pub fn serialize(&self) -> Arc<SkData> {
        let size = unsafe { self.writeToMemory(core::ptr::null_mut()) };
        let data = unsafe { SkData::MakeUninitialized(size) };
        unsafe {
            self.writeToMemory(data.writable_data());
        }
        data
    }
}
// Source: src/core/SkPath_serial.cpp:164-208
unsafe fn read_rrect_path(
    storage: *const u8,
    length: usize,
    bytes_read: &mut usize,
) -> Option<SkPath> {
    let mut buffer = unsafe { SkRBuffer::new(storage, length) };
    let mut packed = 0;
    if !buffer.readU32(&mut packed) {
        *bytes_read = 0;
        return None;
    }
    crate::SkASSERT!(@source b"src/core/SkPath_serial.cpp\0".as_ptr(),173;@condition b"extract_serializationtype(packed) == SerializationType::kRRect\0".as_ptr();extract_serializationtype(packed)==RRECT);
    let dir = (packed >> DIRECTION_SHIFT) & 3;
    let fill = extract_filltype(packed);
    let direction = match dir {
        0 => SkPathDirection::kCW,
        1 => SkPathDirection::kCCW,
        _ => {
            *bytes_read = 0;
            return None;
        }
    };
    let mut rrect = SkRRect::default();
    if !unsafe { SkRRectPriv::ReadFromBuffer(&mut buffer, &mut rrect) } {
        *bytes_read = 0;
        return None;
    }
    let mut start = 0;
    if !buffer.readS32(&mut start) || start != start.clamp(0, 7) {
        *bytes_read = 0;
        return None;
    }
    let mut path = SkPath::RRect(&rrect, direction, start as u32);
    path.setFillType(fill);
    buffer.skipToAlign4();
    *bytes_read = buffer.pos();
    Some(path)
}
impl SkPath {
    // Source: src/core/SkPath_serial.cpp:210-285
    /// Safety: storage covers length readable bytes; output only borrows during this call.
    pub unsafe fn ReadFromMemory(
        storage: *const u8,
        length: usize,
        bytes_read: Option<&mut usize>,
    ) -> Option<Self> {
        let mut bytes_storage = 0;
        let bytes_read = bytes_read.unwrap_or(&mut bytes_storage);
        let mut buffer = unsafe { SkRBuffer::new(storage, length) };
        let mut packed = 0;
        if !buffer.readU32(&mut packed) {
            *bytes_read = 0;
            return None;
        }
        let version = extract_version(packed);
        let forward = version == VERBS_FORWARD;
        if !forward && version != JUST_PUBLIC_DATA {
            *bytes_read = 0;
            return None;
        }
        match extract_serializationtype(packed) {
            RRECT => return unsafe { read_rrect_path(storage, length, bytes_read) },
            GENERAL => {}
            _ => {
                *bytes_read = 0;
                return None;
            }
        }
        let mut counts = [0u32; 3];
        if !unsafe { buffer.read(counts.as_mut_ptr().cast(), 12) } {
            *bytes_read = 0;
            return None;
        }
        let points = buffer.skipCount::<SkPoint>(counts[0] as usize);
        let conics = buffer.skipCount::<f32>(counts[1] as usize);
        let verbs = buffer.skipCount::<u8>(counts[2] as usize);
        buffer.skipToAlign4();
        if !buffer.isValid() {
            *bytes_read = 0;
            return None;
        }
        crate::SkASSERT!(@source b"src/core/SkPath_serial.cpp\0".as_ptr(),257; @condition b"buffer.pos() <= length\0".as_ptr(); buffer.pos()<=length);
        if counts[2] == 0 {
            if counts[0] == 0 && counts[1] == 0 {
                let path = SkPath::new(extract_filltype(packed));
                *bytes_read = buffer.pos();
                return Some(path);
            }
            *bytes_read = 0;
            return None;
        }
        // Native Raw copies these spans. Read potentially unaligned wire scalars into
        // owned POD before forming Rust references; never form invalid enum references.
        let points: Vec<SkPoint> = (0..counts[0] as usize)
            .map(|i| unsafe { points.add(i).read_unaligned() })
            .collect();
        let conics: Vec<f32> = (0..counts[1] as usize)
            .map(|i| unsafe { conics.add(i).read_unaligned() })
            .collect();
        let mut verb_bytes: Vec<u8> = (0..counts[2] as usize)
            .map(|i| unsafe { verbs.add(i).read() })
            .collect();
        if !forward {
            verb_bytes.reverse();
        }
        *bytes_read = buffer.pos();
        Some(SkPath::Make(
            &points,
            &verb_bytes,
            &conics,
            extract_filltype(packed),
            false,
        ))
    }
}
