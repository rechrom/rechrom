use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ffi::{c_char, c_void, CStr};
use std::sync::{Arc, OnceLock};
impl SkData {
    // Source: include/core/SkData.h:38-40
    pub fn equals(&self, other: Option<&Self>) -> bool {
        other.is_some_and(|other| self == other)
    }
}
impl SkData {
    // Source: include/core/SkData.h:46-48
    pub fn Equals(a: Option<&Self>, b: Option<&Self>) -> bool {
        match a {
            None => b.is_none(),
            Some(a) => a.equals(b),
        }
    }
}
impl SkData {
    // Source: include/core/SkData.h:53-53
    pub fn size(&self) -> usize {
        self.fSize
    }
}
impl SkData {
    // Source: include/core/SkData.h:58-58
    // Source: include/core/SkData.h:60-60
    // Source: include/core/SkData.h:62-62
    // Source: include/core/SkData.h:64-64
    pub fn data(&self) -> *const u8 {
        self.fPtr
    }
}
impl SkData {
    pub fn empty(&self) -> bool {
        self.fSize == 0
    }
}
impl SkData {
    pub fn bytes(&self) -> *const u8 {
        self.data()
    }
}
impl SkData {
    pub fn byteSpan(&self) -> (*const u8, usize) {
        (self.bytes(), self.size())
    }
}
impl SkData {
    // Source: include/core/SkData.h:70-72
    pub fn writable_data(&self) -> *mut u8 {
        self.fPtr
    }
}
// Source: include/core/SkData.h:101-101
pub type ReleaseProc = unsafe extern "C" fn(*const u8, *mut c_void);
impl SkData {
    // Source: include/core/SkData.h:139-141
    /// The initialized bytes must remain valid and immutable on every thread retaining the result.
    pub unsafe fn MakeWithoutCopy(data: *const u8, length: usize) -> Arc<Self> {
        Self::MakeWithProc(
            data,
            length,
            Some(Self::NoopReleaseProc),
            std::ptr::null_mut(),
        )
    }
}
impl SkData {
    // Source: include/core/SkData.h:186-191
    pub fn MakeSubset(src: &Arc<Self>, offset: usize, length: usize) -> Arc<Self> {
        if let Some(dst) = src.shareSubset(offset, length) {
            return dst;
        }
        Self::MakeEmpty()
    }
}
impl SkData {
    // Source: include/core/SkData.h:202-202
    pub fn isEmpty(&self) -> bool {
        self.fSize == 0
    }
}
// Source: include/core/SkData.h:206-208
// Arc substitutes native intrusive refs; owned inline bytes become a separate stable raw allocation.
// Raw span remains usable for uninitialized storage without creating an initialized Rust byte slice.
pub struct SkData {
    fReleaseProc: Option<ReleaseProc>,
    fReleaseProcContext: *mut c_void,
    fPtr: *mut u8,
    fSize: usize,
    allocation: Option<Layout>,
}
// Immutable data may be shared across threads, as with native atomic-refcounted SkData.
// Unsafe external-storage constructors require initialized readable bytes to remain valid
// and immutable, and any release callback/context to be valid on the final releasing thread.
unsafe impl Send for SkData {}
unsafe impl Sync for SkData {}
// Source: src/core/SkData.cpp:21-25
impl SkData {
    fn new(ptr: *mut u8, size: usize, proc: Option<ReleaseProc>, ctx: *mut c_void) -> Self {
        Self {
            fReleaseProc: proc,
            fReleaseProcContext: ctx,
            fPtr: ptr,
            fSize: size,
            allocation: None,
        }
    }
}
// Source: src/core/SkData.cpp:36-40
impl Drop for SkData {
    fn drop(&mut self) {
        if let Some(proc) = self.fReleaseProc {
            unsafe {
                proc(self.fPtr, self.fReleaseProcContext);
            }
        }
        if let Some(layout) = self.allocation {
            unsafe {
                dealloc(self.fPtr, layout);
            }
        }
    }
}
// Source: src/core/SkData.cpp:42-47
impl PartialEq for SkData {
    fn eq(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        self.size() == other.size()
            && (self.size() == 0
                || unsafe {
                    std::slice::from_raw_parts(self.data(), self.size())
                        == std::slice::from_raw_parts(other.data(), other.size())
                })
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:49-58
    // Source: src/core/SkData.cpp:60-64
    pub unsafe fn copyRange(&self, offset: usize, mut length: usize, buffer: *mut u8) -> usize {
        let mut available = self.size();
        if offset >= available || length == 0 {
            return 0;
        }
        available -= offset;
        if length > available {
            length = available;
        }
        crate::SkASSERT!(@source b"src/core/SkData.cpp\0".as_ptr(),58; @condition b"length > 0\0".as_ptr(); length>0);
        if !buffer.is_null() {
            std::ptr::copy_nonoverlapping(self.bytes().add(offset), buffer, length);
        }
        length
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:81-82
    // Source: src/core/SkData.cpp:84-86
    // Source: src/core/SkData.cpp:88-88
    // Source: src/core/SkData.cpp:90-94
    // Source: src/core/SkData.cpp:96-98
    // Native const/nonconst overloads share this immutable Arc-owner method.
    pub fn shareSubset(self: &Arc<Self>, offset: usize, length: usize) -> Option<Arc<Self>> {
        if offset > self.size() || length > self.size() - offset {
            return None;
        }
        if offset == 0 && length == self.size() {
            return Some(Arc::clone(self));
        }
        if length == 0 {
            return Some(Self::MakeEmpty());
        }
        unsafe extern "C" fn release(_ptr: *const u8, ctx: *mut c_void) {
            drop(Arc::from_raw(ctx as *const SkData));
        }
        let ctx = Arc::into_raw(Arc::clone(self)) as *mut c_void;
        Some(unsafe { Self::MakeWithProc(self.bytes().add(offset), length, Some(release), ctx) })
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:100-101
    // Source: src/core/SkData.cpp:103-104
    pub fn copySubset(&self, offset: usize, length: usize) -> Option<Arc<Self>> {
        if offset > self.size() || length > self.size() - offset {
            return None;
        }
        Some(unsafe { Self::MakeWithCopy(self.bytes().wrapping_add(offset), length) })
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:110-113
    // Source: src/core/SkData.cpp:115-116
    // Source: src/core/SkData.cpp:118-124
    unsafe fn PrivateNewWithCopy(src: *const u8, length: usize) -> Arc<Self> {
        if length == 0 {
            return Self::MakeEmpty();
        }
        let actual = length.wrapping_add(std::mem::size_of::<Self>());
        crate::SkASSERT_RELEASE!(@source b"src/core/SkData.cpp\0".as_ptr(),116; @condition b"length < actualLength\0".as_ptr(); length<actual);
        let layout = Layout::from_size_align(length, 16).expect("SkData allocation layout");
        let p = alloc(layout);
        if p.is_null() {
            handle_alloc_error(layout);
        }
        let result = Arc::new(Self {
            fReleaseProc: None,
            fReleaseProcContext: std::ptr::null_mut(),
            fPtr: p,
            fSize: length,
            allocation: Some(layout),
        });
        if !src.is_null() {
            std::ptr::copy_nonoverlapping(src, result.writable_data(), length);
        }
        result
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:126-126
    unsafe extern "C" fn NoopReleaseProc(_ptr: *const u8, _ctx: *mut c_void) {}
}
impl SkData {
    // Source: src/core/SkData.cpp:130-133
    pub fn MakeEmpty() -> Arc<Self> {
        static EMPTY: OnceLock<Arc<SkData>> = OnceLock::new();
        Arc::clone(EMPTY.get_or_init(|| {
            Arc::new(Self::new(
                std::ptr::null_mut(),
                0,
                None,
                std::ptr::null_mut(),
            ))
        }))
    }
}
// Source: src/core/SkData.cpp:136-138
unsafe extern "C" fn sk_free_releaseproc(ptr: *const u8, _ctx: *mut c_void) {
    unsafe extern "C" {
        fn free(ptr: *mut c_void);
    }
    free(ptr as *mut c_void);
}
impl SkData {
    // Source: src/core/SkData.cpp:140-143
    /// Takes ownership of a libc malloc-compatible allocation containing `length` initialized bytes.
    pub unsafe fn MakeFromMalloc(data: *const u8, length: usize) -> Arc<Self> {
        Arc::new(Self::new(
            data as *mut u8,
            length,
            Some(sk_free_releaseproc),
            std::ptr::null_mut(),
        ))
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:145-148
    pub unsafe fn MakeWithCopy(src: *const u8, length: usize) -> Arc<Self> {
        crate::SkASSERT!(@source b"src/core/SkData.cpp\0".as_ptr(),146; @condition b"src\0".as_ptr(); !src.is_null());
        Self::PrivateNewWithCopy(src, length)
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:150-152
    /// Caller must initialize every byte before reading or sharing this allocation.
    pub unsafe fn MakeUninitialized(length: usize) -> Arc<Self> {
        unsafe { Self::PrivateNewWithCopy(std::ptr::null(), length) }
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:154-160
    pub fn MakeZeroInitialized(length: usize) -> Arc<Self> {
        let data = unsafe { Self::MakeUninitialized(length) };
        if length != 0 {
            unsafe {
                std::ptr::write_bytes(data.writable_data(), 0, data.size());
            }
        }
        data
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:162-165
    /// Storage must remain initialized, readable and immutable through final release.
    /// The callback/context must own their release obligation and support the final releasing thread.
    pub unsafe fn MakeWithProc(
        data: *const u8,
        length: usize,
        proc: Option<ReleaseProc>,
        ctx: *mut c_void,
    ) -> Arc<Self> {
        Arc::new(Self::new(data as *mut u8, length, proc, ctx))
    }
}
// Source: src/core/SkData.cpp:167-171
#[cfg(all(unix, not(target_os = "ios")))]
unsafe extern "C" fn sk_mmap_releaseproc(addr: *const u8, ctx: *mut c_void) {
    let length = ctx as usize;
    crate::src::core::SkOSFile::sk_fmunmap(addr.cast(), length);
}
#[cfg(all(unix, not(target_os = "ios")))]
impl SkData {
    // Source: src/core/SkData.cpp:173-181
    /// Borrows a live FILE; returned data owns only the mapping, not the FILE.
    /// The mapped file must remain readable and its contents immutable through final release.
    pub unsafe fn MakeFromFILE(f: *mut crate::src::core::SkOSFile::FILE) -> Option<Arc<Self>> {
        let mut size = 0;
        let addr = crate::src::core::SkOSFile::sk_fmmap(f, &mut size);
        if addr.is_null() {
            return None;
        }
        Some(Self::MakeWithProc(
            addr.cast(),
            size,
            Some(sk_mmap_releaseproc),
            size as *mut c_void,
        ))
    }
    // Source: src/core/SkData.cpp:183-191
    /// Accepts a NUL-terminated path or null. The mapped file retains the same
    /// external immutability/readability requirement as MakeFromFILE.
    pub unsafe fn MakeFromFileName(path: *const c_char) -> Option<Arc<Self>> {
        use crate::src::core::SkOSFile::{kRead_SkFILE_Flag, sk_fclose, sk_fopen};
        let f = if path.is_null() {
            std::ptr::null_mut()
        } else {
            sk_fopen(path, kRead_SkFILE_Flag)
        };
        if f.is_null() {
            return None;
        }
        let data = Self::MakeFromFILE(f);
        sk_fclose(f);
        data
    }
    // Source: src/core/SkData.cpp:193-200
    /// Borrows a descriptor; successful data owns its mapping independently.
    /// The caller must retain readable, immutable backing file bytes while data is alive.
    pub unsafe fn MakeFromFD(fd: std::ffi::c_int) -> Option<Arc<Self>> {
        let mut size = 0;
        let addr = crate::src::core::SkOSFile::sk_fdmmap(fd, &mut size);
        if addr.is_null() {
            return None;
        }
        Some(Self::MakeWithProc(
            addr.cast(),
            size,
            Some(sk_mmap_releaseproc),
            size as *mut c_void,
        ))
    }
}
impl SkData {
    // Source: src/core/SkData.cpp:202-211
    pub unsafe fn MakeWithCString(mut cstr: *const c_char) -> Arc<Self> {
        let size;
        if cstr.is_null() {
            cstr = b"\0".as_ptr().cast();
            size = 1;
        } else {
            size = CStr::from_ptr(cstr).to_bytes().len() + 1;
        }
        Self::MakeWithCopy(cstr.cast(), size)
    }
}

impl SkData {
    // Source: src/core/SkData.cpp:215-226
    pub fn MakeFromStream(
        mut stream: core::pin::Pin<&mut dyn super::SkStream::SkStream>,
        size: usize,
    ) -> Option<Arc<Self>> {
        if super::SkStream::SkStreamPriv::RemainingLengthIsBelow(stream.as_ref().get_ref(), size) {
            return None;
        }
        let data = unsafe { Self::MakeUninitialized(size) };
        if unsafe { stream.as_mut().read(data.writable_data(), size) } != size {
            return None;
        }
        Some(data)
    }
}
