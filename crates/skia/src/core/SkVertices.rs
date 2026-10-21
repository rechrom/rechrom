use crate::include::core::SkRefCnt::{sk_sp, SkIntrusiveRefCounted, SkNVRefCnt, SkNVRefCntObject};
use crate::src::core::{
    SkColor::SkColor, SkPoint::SkPoint, SkRect::SkRect, SkSafeMath::SkSafeMath,
};
use core::{
    mem::{align_of, size_of, MaybeUninit},
    ptr::NonNull,
};

// Source: include/core/SkVertices.h:30-36
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexMode {
    kTriangles_VertexMode,
    kTriangleStrip_VertexMode,
    kTriangleFan_VertexMode,
}
pub const kLast_VertexMode: VertexMode = VertexMode::kTriangleFan_VertexMode;
// Source: include/core/SkVertices.h:49-60
impl SkVertices {
    pub unsafe fn MakeCopy_no_indices(
        mode: VertexMode,
        vertex_count: i32,
        positions: *const SkPoint,
        texs: *const SkPoint,
        colors: *const SkColor,
    ) -> sk_sp<Self> {
        unsafe {
            Self::MakeCopy(
                mode,
                vertex_count,
                positions,
                texs,
                colors,
                0,
                core::ptr::null(),
            )
        }
    }
}
// Source: include/core/SkVertices.h:62-65
#[repr(i32)]
pub enum BuilderFlags {
    kHasTexCoords_BuilderFlag = 1 << 0,
    kHasColors_BuilderFlag = 1 << 1,
}
pub const kHasTexCoords_BuilderFlag: u32 = BuilderFlags::kHasTexCoords_BuilderFlag as u32;
pub const kHasColors_BuilderFlag: u32 = BuilderFlags::kHasColors_BuilderFlag as u32;
// Source: include/core/SkVertices.h:66-66
// The partial object has exactly one intrusive owner. A separately aligned u16
// buffer substitutes unique_ptr<uint8_t[]> for the native temporary fan bytes.
pub struct Builder {
    // Source: include/core/SkVertices.h:88-91
    fVertices: sk_sp<SkVertices>,
    fIntermediateFanIndices: Vec<MaybeUninit<u16>>,
}
// Source: include/core/SkVertices.h:70-70
impl Builder {
    pub fn isValid(&self) -> bool {
        self.fVertices.is_some()
    }
}
// Source: include/core/SkVertices.h:97-98
impl SkVertices {
    pub fn uniqueID(&self) -> u32 {
        unsafe { self.fUniqueID.assume_init() }
    }
    pub fn bounds(&self) -> &SkRect {
        unsafe { self.fBounds.assume_init_ref() }
    }
}
// Source: include/core/SkVertices.h:26-26
// Native contiguous object+arrays storage is retained, using Rust's actual
// object layout rather than claiming C++ ABI identity. MaybeUninit preserves
// bounds/ID staging until detach; the four array pointers are interior borrows.
#[repr(C)]
pub struct SkVertices {
    base: SkNVRefCnt<SkVertices>,
    // Source: include/core/SkVertices.h:120-132
    pub(crate) fUniqueID: MaybeUninit<u32>,
    pub(crate) fPositions: *mut SkPoint,
    pub(crate) fIndices: *mut u16,
    pub(crate) fTexs: *mut SkPoint,
    pub(crate) fColors: *mut SkColor,
    pub(crate) fBounds: MaybeUninit<SkRect>,
    pub(crate) fVertexCount: i32,
    pub(crate) fIndexCount: i32,
    pub(crate) fMode: VertexMode,
}

// Source: src/core/SkVertices.cpp:27-35
fn next_id() -> u32 {
    static NEXT_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    loop {
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if id != 0 {
            return id;
        }
    }
}
// Source: src/core/SkVertices.cpp:37-43
// Source: include/core/SkVertices.h:27-27
pub(crate) struct Desc {
    pub(crate) fMode: VertexMode,
    pub(crate) fVertexCount: i32,
    pub(crate) fIndexCount: i32,
    pub(crate) fHasTexs: bool,
    pub(crate) fHasColors: bool,
}
// Source: src/core/SkVertices.cpp:94-104
#[derive(Default)]
// Source: include/core/SkVertices.h:28-28
pub(crate) struct Sizes {
    pub(crate) fTotal: usize,
    pub(crate) fArrays: usize,
    pub(crate) fVSize: usize,
    pub(crate) fTSize: usize,
    pub(crate) fCSize: usize,
    pub(crate) fISize: usize,
    pub(crate) fBuilderTriFanISize: usize,
}
// Source: src/core/SkVertices.cpp:45-87
impl Sizes {
    pub(crate) fn new(desc: &Desc) -> Self {
        let mut safe = SkSafeMath::new();
        let mut sizes = Self::default();
        sizes.fVSize = safe.mul(desc.fVertexCount as usize, size_of::<SkPoint>());
        sizes.fTSize = if desc.fHasTexs {
            safe.mul(desc.fVertexCount as usize, size_of::<SkPoint>())
        } else {
            0
        };
        sizes.fCSize = if desc.fHasColors {
            safe.mul(desc.fVertexCount as usize, size_of::<SkColor>())
        } else {
            0
        };
        sizes.fBuilderTriFanISize = 0;
        sizes.fISize = safe.mul(desc.fIndexCount as usize, size_of::<u16>());
        if desc.fMode == VertexMode::kTriangleFan_VertexMode {
            let num_fan_tris;
            if desc.fIndexCount != 0 {
                sizes.fBuilderTriFanISize = sizes.fISize;
                num_fan_tris = desc.fIndexCount - 2;
            } else {
                num_fan_tris = desc.fVertexCount - 2;
                if desc.fVertexCount > u16::MAX as i32 + 1 {
                    return Self::default();
                }
            }
            if num_fan_tris <= 0 {
                return Self::default();
            }
            sizes.fISize = safe.mul(num_fan_tris as usize, 3 * size_of::<u16>());
        }
        // Sequential temporaries preserve native right-nested SafeMath additions.
        let ci = safe.add(sizes.fCSize, sizes.fISize);
        let tci = safe.add(sizes.fTSize, ci);
        let vtci = safe.add(sizes.fVSize, tci);
        sizes.fTotal = safe.add(size_of::<SkVertices>(), vtci);
        if safe.ok() {
            sizes.fArrays = sizes.fVSize + sizes.fTSize + sizes.fCSize + sizes.fISize;
        } else {
            sizes = Self::default();
        }
        sizes
    }
    // Source: src/core/SkVertices.cpp:89-92
    pub(crate) fn isValid(&self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 90; @condition b"fTotal >= fVSize\0".as_ptr(); self.fTotal>=self.fVSize);
        self.fVSize > 0
    }
}
// Source: include/core/SkVertices.h:68-68
// Source: src/core/SkVertices.cpp:106-111
impl Builder {
    pub fn new(mode: VertexMode, vertex_count: i32, index_count: i32, flags: u32) -> Self {
        Self::from_desc(&Desc {
            fMode: mode,
            fVertexCount: vertex_count,
            fIndexCount: index_count,
            fHasTexs: flags & kHasTexCoords_BuilderFlag != 0,
            fHasColors: flags & kHasColors_BuilderFlag != 0,
        })
    }
    // Source: include/core/SkVertices.h:83-83
    // Source: src/core/SkVertices.cpp:113-115
    pub(crate) fn from_desc(desc: &Desc) -> Self {
        let mut builder = Self {
            fVertices: sk_sp::default(),
            fIntermediateFanIndices: Vec::new(),
        };
        builder.init(desc);
        builder
    }
    // Source: include/core/SkVertices.h:85-85
    // Source: src/core/SkVertices.cpp:117-151
    fn init(&mut self, desc: &Desc) {
        let sizes = Sizes::new(desc);
        if !sizes.isValid() {
            crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 120; @condition b"!this->isValid()\0".as_ptr(); !self.isValid());
            return;
        }
        let layout =
            std::alloc::Layout::from_size_align(sizes.fTotal, align_of::<SkVertices>()).unwrap();
        let storage = unsafe { std::alloc::alloc(layout) };
        if storage.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        if sizes.fBuilderTriFanISize != 0 {
            self.fIntermediateFanIndices.resize_with(
                sizes.fBuilderTriFanISize / size_of::<u16>(),
                MaybeUninit::uninit,
            );
        }
        let mut ptr = unsafe { storage.add(size_of::<SkVertices>()) };
        let mut advance = |size: usize| {
            let out = if size != 0 {
                ptr
            } else {
                core::ptr::null_mut()
            };
            ptr = unsafe { ptr.add(size) };
            out
        };
        let object = storage.cast::<SkVertices>();
        unsafe {
            object.write(SkVertices {
                base: SkNVRefCnt::new(),
                fUniqueID: MaybeUninit::uninit(),
                fPositions: advance(sizes.fVSize).cast(),
                fTexs: advance(sizes.fTSize).cast(),
                fColors: advance(sizes.fCSize).cast(),
                fIndices: advance(sizes.fISize).cast(),
                fBounds: MaybeUninit::uninit(),
                fVertexCount: desc.fVertexCount,
                fIndexCount: desc.fIndexCount,
                fMode: desc.fMode,
            });
        }
        self.fVertices = unsafe { sk_sp::from_raw(object) };
    }
    // Source: include/core/SkVertices.h:80-80
    // Source: src/core/SkVertices.cpp:153-182
    /// Safety: initialize all requested arrays first. Indices must reference live
    /// positions; raw builder pointers may not be used after this ownership transfer.
    pub unsafe fn detach(&mut self) -> sk_sp<SkVertices> {
        if let Some(mut vertices) = self.fVertices.as_ptr() {
            let v = unsafe { vertices.as_mut() };
            v.fBounds.write(SkRect::BoundsOrEmpty(unsafe {
                core::slice::from_raw_parts(v.fPositions, v.fVertexCount as usize)
            }));
            if v.fMode == VertexMode::kTriangleFan_VertexMode {
                if !self.fIntermediateFanIndices.is_empty() {
                    crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 159; @condition b"fVertices->fIndexCount\0".as_ptr(); v.fIndexCount!=0);
                    let temp = self.fIntermediateFanIndices.as_ptr().cast::<u16>();
                    for t in 0..v.fIndexCount - 2 {
                        unsafe {
                            v.fIndices.add(3 * t as usize).write(*temp);
                            v.fIndices
                                .add(3 * t as usize + 1)
                                .write(*temp.add(t as usize + 1));
                            v.fIndices
                                .add(3 * t as usize + 2)
                                .write(*temp.add(t as usize + 2));
                        }
                    }
                    v.fIndexCount = 3 * (v.fIndexCount - 2);
                } else {
                    crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 168; @condition b"!fVertices->fIndexCount\0".as_ptr(); (v.fIndexCount) == (0));
                    for t in 0..v.fVertexCount - 2 {
                        unsafe {
                            v.fIndices.add(3 * t as usize).write(0);
                            v.fIndices.add(3 * t as usize + 1).write((t + 1) as u16);
                            v.fIndices.add(3 * t as usize + 2).write((t + 2) as u16);
                        }
                    }
                    v.fIndexCount = 3 * (v.fVertexCount - 2);
                }
                v.fMode = VertexMode::kTriangles_VertexMode;
            }
            v.fUniqueID.write(next_id());
            return sk_sp::move_from(&mut self.fVertices);
        }
        sk_sp::default()
    }
    // Source: include/core/SkVertices.h:72-72
    // Source: src/core/SkVertices.cpp:184-186
    pub fn positions(&mut self) -> *mut SkPoint {
        self.fVertices
            .get()
            .map_or(core::ptr::null_mut(), |v| v.fPositions)
    }
    // Source: include/core/SkVertices.h:76-76
    // Source: src/core/SkVertices.cpp:188-190
    pub fn texCoords(&mut self) -> *mut SkPoint {
        self.fVertices
            .get()
            .map_or(core::ptr::null_mut(), |v| v.fTexs)
    }
    // Source: include/core/SkVertices.h:77-77
    // Source: src/core/SkVertices.cpp:192-194
    pub fn colors(&mut self) -> *mut SkColor {
        self.fVertices
            .get()
            .map_or(core::ptr::null_mut(), |v| v.fColors)
    }
    // Source: include/core/SkVertices.h:73-73
    // Source: src/core/SkVertices.cpp:196-204
    pub fn indices(&mut self) -> *mut u16 {
        if !self.fVertices.is_some() {
            return core::ptr::null_mut();
        }
        if !self.fIntermediateFanIndices.is_empty() {
            return self.fIntermediateFanIndices.as_mut_ptr().cast();
        }
        self.fVertices.fIndices
    }
}
// Source: include/core/SkVertices.h:42-47
// Source: src/core/SkVertices.cpp:208-260
impl SkVertices {
    /// Safety: supplied arrays cover the native counts; optional arrays may be
    /// null only when their requested byte size is zero. Native SkToU16 domain
    /// restrictions remain unchanged. This factory copies, then clamps indices.
    pub unsafe fn MakeCopy(
        mode: VertexMode,
        vertex_count: i32,
        positions: *const SkPoint,
        texs: *const SkPoint,
        colors: *const SkColor,
        index_count: i32,
        indices: *const u16,
    ) -> sk_sp<Self> {
        let desc = Desc {
            fMode: mode,
            fVertexCount: vertex_count,
            fIndexCount: index_count,
            fHasTexs: !texs.is_null(),
            fHasColors: !colors.is_null(),
        };
        let mut builder = Builder::from_desc(&desc);
        if !builder.isValid() {
            return sk_sp::default();
        }
        let sizes = Sizes::new(&desc);
        crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 219; @condition b"sizes.isValid()\0".as_ptr(); sizes.isValid());
        unsafe fn careful_copy<T>(dst: *mut T, src: *const T, bytes: usize) {
            if bytes != 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(src.cast::<u8>(), dst.cast::<u8>(), bytes);
                }
            }
        }
        unsafe {
            careful_copy(builder.positions(), positions, sizes.fVSize);
            careful_copy(builder.texCoords(), texs, sizes.fTSize);
            careful_copy(builder.colors(), colors, sizes.fCSize);
        }
        let isize = if mode == VertexMode::kTriangleFan_VertexMode {
            sizes.fBuilderTriFanISize
        } else {
            sizes.fISize
        };
        let icount = isize / size_of::<u16>();
        crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 230; @condition b"vertexCount > 0\0".as_ptr(); vertex_count>0);
        debug_assert!(vertex_count - 1 <= u16::MAX as i32);
        let max_index = (vertex_count - 1) as u16;
        let out = builder.indices();
        let mut i = 0;
        // Scalar lane operations replace only skvx loads/min/store, retaining
        // native 8/4/2/1 chunking and exact unsigned clamp arithmetic.
        while i + 8 <= icount {
            for lane in 0..8 {
                unsafe {
                    out.add(i + lane)
                        .write((*indices.add(i + lane)).min(max_index));
                }
            }
            i += 8;
        }
        if i + 4 <= icount {
            for lane in 0..4 {
                unsafe {
                    out.add(i + lane)
                        .write((*indices.add(i + lane)).min(max_index));
                }
            }
            i += 4;
        }
        if i + 2 <= icount {
            for lane in 0..2 {
                unsafe {
                    out.add(i + lane)
                        .write((*indices.add(i + lane)).min(max_index));
                }
            }
            i += 2;
        }
        if i < icount {
            unsafe {
                out.add(i).write((*indices.add(i)).min(max_index));
            }
            i += 1;
        }
        crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 257; @condition b"i == icount\0".as_ptr(); (i) == (icount));
        unsafe { builder.detach() }
    }
    // Source: include/core/SkVertices.h:101-101
    // Source: src/core/SkVertices.cpp:262-264
    pub fn approximateSize(&self) -> usize {
        self.getSizes().fTotal
    }
    // Source: include/core/SkVertices.h:116-116
    // Source: src/core/SkVertices.cpp:266-270
    pub(crate) fn getSizes(&self) -> Sizes {
        let sizes = Sizes::new(&Desc {
            fMode: self.fMode,
            fVertexCount: self.fVertexCount,
            fIndexCount: self.fIndexCount,
            fHasTexs: !self.fTexs.is_null(),
            fHasColors: !self.fColors.is_null(),
        });
        crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 268; @condition b"sizes.isValid()\0".as_ptr(); sizes.isValid());
        sizes
    }
}
// Source: src/core/SkVertices.cpp:277-279
pub(crate) const kMode_Mask: u32 = 0x0FF;
pub(crate) const kHasTexs_Mask: u32 = 0x100;
pub(crate) const kHasColors_Mask: u32 = 0x200;

// Source: src/core/SkVertices.cpp:281-306
impl crate::src::core::SkVerticesPriv::SkVerticesPriv<'_> {
    pub fn encode(&self, buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer) {
        let v = self.fVertices;
        let mut packed = v.fMode as u32;
        crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 284; @condition b"(packed & ~kMode_Mask) == 0\0".as_ptr(); (packed&!kMode_Mask) == (0));
        if !v.fTexs.is_null() {
            packed |= kHasTexs_Mask;
        }
        if !v.fColors.is_null() {
            packed |= kHasColors_Mask;
        }
        let sizes = v.getSizes();
        crate::SkASSERT!(@source b"src/core/SkVertices.cpp\0".as_ptr(), 293; @condition b"!sizes.fBuilderTriFanISize\0".as_ptr(); (sizes.fBuilderTriFanISize) == (0));
        buffer.writeUInt(packed);
        buffer.writeInt(v.fVertexCount);
        buffer.writeInt(v.fIndexCount);
        unsafe {
            buffer.writeByteArray(v.fPositions.cast(), sizes.fVSize);
            buffer.writeByteArray(v.fTexs.cast(), sizes.fTSize);
            buffer.writeByteArray(v.fColors.cast(), sizes.fCSize);
            buffer.writeByteArray(v.fIndices.cast(), sizes.fISize);
        }
    }
}
// Source: src/core/SkVertices.cpp:308-376
impl crate::src::core::SkVerticesPriv::SkVerticesPriv<'_> {
    pub fn Decode(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> sk_sp<SkVertices> {
        let decode =
            |buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>| -> sk_sp<SkVertices> {
                let mut safe = crate::src::core::SkSafeRange::SkSafeRange::new();
                // Native SkPicturePriv::kVerticesRemoveCustomData_Version = 86;
                // retaining the wire version value does not synthesize a Picture owner.
                let has_custom_data = buffer.isVersionLT(86);
                let packed = buffer.readUInt();
                let vertex_count = safe.checkGE(buffer.readInt(), 0);
                let index_count = safe.checkGE(buffer.readInt(), 0);
                let attr_count = if has_custom_data {
                    safe.checkGE(buffer.readInt(), 0)
                } else {
                    0
                };
                let mode_value =
                    safe.checkLE((packed & kMode_Mask) as u64, kLast_VertexMode as u32);
                let mode = match mode_value {
                    0 => VertexMode::kTriangles_VertexMode,
                    1 => VertexMode::kTriangleStrip_VertexMode,
                    2 => VertexMode::kTriangleFan_VertexMode,
                    _ => unreachable!("SkSafeRange bounds enum"),
                };
                let has_texs = packed & kHasTexs_Mask != 0;
                let has_colors = packed & kHasColors_Mask != 0;
                if !safe.ok()
                    || !buffer.isValid()
                    || attr_count != 0
                    || mode == VertexMode::kTriangleFan_VertexMode
                {
                    return sk_sp::default();
                }
                let desc = Desc {
                    fMode: mode,
                    fVertexCount: vertex_count,
                    fIndexCount: index_count,
                    fHasTexs: has_texs,
                    fHasColors: has_colors,
                };
                let sizes = Sizes::new(&desc);
                if !sizes.isValid() || sizes.fArrays > buffer.available() {
                    return sk_sp::default();
                }
                let mut builder = Builder::from_desc(&desc);
                if !builder.isValid() {
                    return sk_sp::default();
                }
                unsafe {
                    buffer.readByteArray_raw(builder.positions().cast(), sizes.fVSize);
                }
                if has_custom_data {
                    let (_, custom_data_size) = buffer.skipByteArray();
                    if custom_data_size != 0 {
                        return sk_sp::default();
                    }
                }
                unsafe {
                    buffer.readByteArray_raw(builder.texCoords().cast(), sizes.fTSize);
                    buffer.readByteArray_raw(builder.colors().cast(), sizes.fCSize);
                    buffer.readByteArray_raw(builder.indices().cast(), sizes.fISize);
                }
                if !buffer.isValid() {
                    return sk_sp::default();
                }
                if index_count > 0 {
                    let indices = builder.indices();
                    for i in 0..index_count as usize {
                        if unsafe { *indices.add(i) } as u32 >= vertex_count as u32 {
                            return sk_sp::default();
                        }
                    }
                }
                unsafe { builder.detach() }
            };
        let verts = decode(buffer);
        if verts.is_some() {
            return verts;
        }
        buffer.validate(false);
        sk_sp::default()
    }
}

// Rust concrete destruction preserves the sole native NV counter and matching
// object+array allocation; it never reconstructs a size-of-object Box for this tail.
unsafe impl SkNVRefCntObject for SkVertices {
    fn nv_ref_cnt(&self) -> &SkNVRefCnt<Self> {
        &self.base
    }
    // Source: src/core/SkVertices.cpp:378-380
    // Source: include/core/SkVertices.h:114-114
    unsafe fn dispose_raw(object: NonNull<Self>) {
        let total = unsafe { object.as_ref() }.getSizes().fTotal;
        unsafe {
            core::ptr::drop_in_place(object.as_ptr());
            std::alloc::dealloc(
                object.as_ptr().cast(),
                std::alloc::Layout::from_size_align(total, align_of::<Self>()).unwrap(),
            );
        }
    }
}
unsafe impl SkIntrusiveRefCounted for SkVertices {
    unsafe fn intrusive_retain(object: NonNull<Self>) {
        unsafe { object.as_ref() }.base.ref_();
    }
    unsafe fn intrusive_release(object: NonNull<Self>) {
        unsafe {
            SkNVRefCnt::unref(object);
        }
    }
}

// Rust implicit-drop ordering bridge; no additional native algorithm credit.
impl Drop for Builder {
    fn drop(&mut self) {
        drop(core::mem::take(&mut self.fIntermediateFanIndices));
    }
}
