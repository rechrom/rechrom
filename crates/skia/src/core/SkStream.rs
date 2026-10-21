use crate::src::core::SkData::SkData;
use core::{marker::PhantomPinned, mem, pin::Pin, ptr};
use std::sync::Arc;
// Source: include/core/SkStream.h:31-156
// Required native byte I/O and exact native optional defaults. Implementations must
// honor null destination as skip, initialize every returned byte, and return <= size.
// Derived projections below must dispatch to the same concrete virtual operations.
// Source: include/core/SkStream.h:152-155
// Complete built-in native stream objects carry PhantomPinned. Existing unique
// factories pin their original Box allocation; stack constructors must be placed
// with core::pin::pin! before invoking mutable virtuals. Only the owner handle
// moves after pinning. No additional object/allocation or mutable unpinned
// projection is exposed. Unsafe external implementers preserve this same base
// lifetime and must not move/replace their complete object through a receiver.
pub unsafe trait SkStream {
    unsafe fn read(self: Pin<&mut Self>, buffer: *mut u8, size: usize) -> usize;
    fn skip(self: Pin<&mut Self>, size: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read(ptr::null_mut(), size) }
    }
    unsafe fn peek(self: Pin<&mut Self>, _buffer: *mut u8, _size: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        0
    }
    fn isAtEnd(&self) -> bool;
    // Source: src/core/SkStream.cpp:32-34
    fn readS8(self: Pin<&mut Self>, x: &mut i8) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read((x as *mut i8).cast(), 1) == 1 }
    }
    // Source: src/core/SkStream.cpp:36-38
    fn readS16(self: Pin<&mut Self>, x: &mut i16) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read((x as *mut i16).cast(), 2) == 2 }
    }
    // Source: src/core/SkStream.cpp:40-42
    fn readS32(self: Pin<&mut Self>, x: &mut i32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read((x as *mut i32).cast(), 4) == 4 }
    }
    // Source: src/core/SkStream.cpp:44-46
    fn readS64(self: Pin<&mut Self>, x: &mut i64) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read((x as *mut i64).cast(), 8) == 8 }
    }
    fn readU8(self: Pin<&mut Self>, x: &mut u8) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read(x, 1) == 1 }
    }
    fn readU16(self: Pin<&mut Self>, x: &mut u16) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read((x as *mut u16).cast(), 2) == 2 }
    }
    fn readU32(self: Pin<&mut Self>, x: &mut u32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read((x as *mut u32).cast(), 4) == 4 }
    }
    fn readU64(self: Pin<&mut Self>, x: &mut u64) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read((x as *mut u64).cast(), 8) == 8 }
    }
    fn readBool(self: Pin<&mut Self>, b: &mut bool) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        let mut value = 0;
        if !unsafe { Pin::new_unchecked(&mut *this) }.readU8(&mut value) {
            return false;
        }
        *b = value != 0;
        true
    }
    // Source: src/core/SkStream.cpp:48-50
    fn readScalar(self: Pin<&mut Self>, x: &mut f32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read((x as *mut f32).cast(), 4) == 4 }
    }
    fn readPackedUInt(self: Pin<&mut Self>, x: &mut usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        read_packed_uint(unsafe { Pin::new_unchecked(&mut *this) }, x)
    }
    fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        false
    }
    fn duplicate(&self) -> Option<Pin<Box<dyn SkStream>>> {
        SkStream::onDuplicate(self)
    }
    fn fork(&self) -> Option<Pin<Box<dyn SkStream>>> {
        SkStream::onFork(self)
    }
    fn hasPosition(&self) -> bool {
        false
    }
    fn getPosition(&self) -> usize {
        0
    }
    fn seek(self: Pin<&mut Self>, _position: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        false
    }
    fn r#move(self: Pin<&mut Self>, _offset: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        false
    }
    fn hasLength(&self) -> bool {
        false
    }
    fn getLength(&self) -> usize {
        0
    }
    fn getMemoryBase(self: Pin<&mut Self>) -> *const u8 {
        let this = unsafe { self.get_unchecked_mut() };
        ptr::null()
    }
    fn getData(&self) -> Option<Arc<SkData>> {
        None
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStream>>> {
        None
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStream>>> {
        None
    }
}
// Source: include/core/SkStream.h:159-167
pub unsafe trait SkStreamRewindable: SkStream {
    fn rewind(self: Pin<&mut Self>) -> bool;
    fn duplicate(&self) -> Option<Pin<Box<dyn SkStreamRewindable>>> {
        SkStreamRewindable::onDuplicate(self)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamRewindable>>>;
}
// Source: include/core/SkStream.h:170-187
pub unsafe trait SkStreamSeekable: SkStreamRewindable {
    fn duplicate(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>> {
        SkStreamSeekable::onDuplicate(self)
    }
    fn hasPosition(&self) -> bool {
        true
    }
    fn getPosition(&self) -> usize;
    fn seek(self: Pin<&mut Self>, position: usize) -> bool;
    fn r#move(self: Pin<&mut Self>, offset: isize) -> bool;
    fn fork(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>> {
        SkStreamSeekable::onFork(self)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>>;
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>>;
}
// Source: include/core/SkStream.h:190-204
pub unsafe trait SkStreamAsset: SkStreamSeekable {
    fn hasLength(&self) -> bool {
        true
    }
    fn getLength(&self) -> usize;
    fn duplicate(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        SkStreamAsset::onDuplicate(self)
    }
    fn fork(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        SkStreamAsset::onFork(self)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamAsset>>>;
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamAsset>>>;
}
// Source: include/core/SkStream.h:207-219
pub unsafe trait SkStreamMemory: SkStreamAsset {
    fn getMemoryBase(self: Pin<&mut Self>) -> *const u8;
    fn duplicate(&self) -> Option<Pin<Box<dyn SkStreamMemory>>> {
        SkStreamMemory::onDuplicate(self)
    }
    fn fork(&self) -> Option<Pin<Box<dyn SkStreamMemory>>> {
        SkStreamMemory::onFork(self)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamMemory>>>;
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamMemory>>>;
}
// Rust-only ownership proof at native cross-thread font/cache boundaries.
// No additional source algorithm or line credit: the sole stored value remains
// the same native unique stream owner. Ordinary SkStreamAsset is not made Send.
#[repr(transparent)]
pub struct TransferableStreamAsset {
    stream: Pin<Box<dyn SkStreamAsset>>,
}
impl TransferableStreamAsset {
    /// Safety: this stream's entire retained graph may move between threads and
    /// be accessed or destroyed on any receiving thread. The same guarantee must
    /// hold recursively for every duplicate/fork it can produce. Shared backing
    /// must support the independently owned streams' concurrent access. No
    /// thread-bound callbacks, borrowed thread-local data, or conflicting aliases
    /// may remain. This obligation lasts until the last descendant is destroyed.
    pub unsafe fn from_native_transferable(stream: Pin<Box<dyn SkStreamAsset>>) -> Self {
        Self { stream }
    }
    pub fn as_asset(&self) -> &dyn SkStreamAsset {
        &*self.stream
    }
    pub fn as_asset_mut(&mut self) -> Pin<&mut dyn SkStreamAsset> {
        self.stream.as_mut()
    }
    pub fn into_asset(self) -> Pin<Box<dyn SkStreamAsset>> {
        self.stream
    }
    pub fn duplicate(&self) -> Option<Self> {
        SkStreamAsset::duplicate(&*self.stream).map(|stream| Self { stream })
    }
    pub fn fork(&self) -> Option<Self> {
        SkStreamAsset::fork(&*self.stream).map(|stream| Self { stream })
    }
}
// Justified solely by the unsafe constructor's complete, recursive owner
// contract. This does not provide Sync: shared native access still needs a lock.
unsafe impl Send for TransferableStreamAsset {}
// Source: include/core/SkStream.h:436-437
// The actual canonical Data owner retains bytes; each stream has its own offset.
// Private field order maps native destruction; the complete pinned object and
// each constructor's original expression evaluation order remain unchanged.
pub struct SkMemoryStream {
    _pin: PhantomPinned,
    // Source: include/core/SkStream.h:437
    fOffset: usize,
    // Source: include/core/SkStream.h:436
    fData: Arc<SkData>,
}
// Source: src/core/SkStream.cpp:52-54
const MAX_BYTE_U8: usize = 253;
const BYTE_SENTINEL_U16: u8 = 254;
const BYTE_SENTINEL_U32: u8 = 255;
// Source: src/core/SkStream.cpp:56-75
fn read_packed_uint<T: SkStream + ?Sized>(mut stream: Pin<&mut T>, x: &mut usize) -> bool {
    let mut byte = 0;
    if unsafe { stream.as_mut().read(&mut byte, 1) } == 0 {
        return false;
    }
    if byte == BYTE_SENTINEL_U16 {
        let mut value = 0;
        if !stream.as_mut().readU16(&mut value) {
            return false;
        }
        *x = value as usize;
    } else if byte == BYTE_SENTINEL_U32 {
        let mut value = 0;
        if !stream.as_mut().readU32(&mut value) {
            return false;
        }
        *x = value as usize;
    } else {
        *x = byte as usize;
    }
    true
}
// UNVERIFIED: no compilation, tests, syntax or source checks were run.
use crate::src::core::SkOSFile::{self, FILE};
use core::ffi::{c_char, c_long};
// Source: include/core/SkStream.h:358-363
// Source: src/core/SkStream.cpp:180-182
// One shared native FILE handle; only the last shared owner invokes its deleter.
struct StreamFileOwner(*mut FILE);
impl Drop for StreamFileOwner {
    fn drop(&mut self) {
        unsafe {
            SkOSFile::sk_fclose(self.0);
        }
    }
}
// Source: include/core/SkStream.h:298
// Source: include/core/SkStream.h:358-363
pub struct SkFILEStream {
    _pin: PhantomPinned,
    // Source: include/core/SkStream.h:363
    fCurrent: usize,
    // Source: include/core/SkStream.h:362
    fStart: usize,
    // Source: include/core/SkStream.h:361
    fEnd: usize,
    // Source: include/core/SkStream.h:358
    fFILE: Option<Arc<StreamFileOwner>>,
}
impl SkFILEStream {
    // Source: src/core/SkStream.cpp:166-174
    fn from_shared(
        file: Option<Arc<StreamFileOwner>>,
        end: usize,
        start: usize,
        current: usize,
    ) -> Self {
        let fStart = start.min(end);
        let fCurrent = current.max(fStart).min(end);
        crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),172; @condition b"fStart == start\0".as_ptr(); (fStart)==(start));
        crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),173; @condition b"fCurrent == current\0".as_ptr(); (fCurrent)==(current));
        Self {
            fFILE: file,
            fEnd: end,
            fStart,
            fCurrent,
            _pin: PhantomPinned,
        }
    }
    // Source: src/core/SkStream.cpp:176-178
    fn from_shared_start(file: Option<Arc<StreamFileOwner>>, end: usize, start: usize) -> Self {
        Self::from_shared(file, end, start, start)
    }
    // Source: src/core/SkStream.cpp:180-182
    /// Safety: file transfers its unique fclose obligation, including on construction failure.
    pub unsafe fn from_file_range(file: *mut FILE, size: usize, start: usize) -> Self {
        Self::from_shared_start(
            Some(Arc::new(StreamFileOwner(file))),
            crate::src::core::SkSafeMath::SkSafeMath::Add(start, size),
            start,
        )
    }
    // Source: src/core/SkStream.cpp:184-186
    /// Safety: file transfers its unique fclose obligation.
    pub unsafe fn from_file_sized(file: *mut FILE, size: usize) -> Self {
        Self::from_file_range(
            file,
            size,
            if file.is_null() {
                0
            } else {
                SkOSFile::sk_ftell(file)
            },
        )
    }
    // Source: src/core/SkStream.cpp:188-192
    /// Safety: file transfers its unique fclose obligation. Other owners must not close it.
    pub unsafe fn from_file(file: *mut FILE) -> Self {
        Self::from_shared_start(
            Some(Arc::new(StreamFileOwner(file))),
            if file.is_null() {
                0
            } else {
                SkOSFile::sk_fgetsize(file)
            },
            if file.is_null() {
                0
            } else {
                SkOSFile::sk_ftell(file)
            },
        )
    }
    // Source: src/core/SkStream.cpp:194-196
    /// Safety: any nonnull path points to a readable nul-terminated native path.
    pub unsafe fn from_path(path: *const c_char) -> Self {
        Self::from_file(if path.is_null() {
            ptr::null_mut()
        } else {
            SkOSFile::sk_fopen(path, SkOSFile::kRead_SkFILE_Flag)
        })
    }
    // Source: include/core/SkStream.h:303
    pub fn new() -> Self {
        unsafe { Self::from_path(ptr::null()) }
    }
    // Source: include/core/SkStream.h:321-324
    pub unsafe fn Make(path: *const c_char) -> Option<Pin<Box<Self>>> {
        let stream = Box::pin(Self::from_path(path));
        if stream.isValid() {
            Some(stream)
        } else {
            None
        }
    }
    // Source: include/core/SkStream.h:327
    pub fn isValid(&self) -> bool {
        !self.file_ptr().is_null()
    }
    fn file_ptr(&self) -> *mut FILE {
        self.fFILE.as_ref().map_or(ptr::null_mut(), |owner| owner.0)
    }
    // Source: src/core/SkStream.cpp:202-207
    pub fn close(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        this.fFILE = None;
        this.fEnd = 0;
        this.fStart = 0;
        this.fCurrent = 0;
    }
    // Source: src/core/SkStream.cpp:209-222
    /// Safety: nonnull buffer is exclusively writable for the requested size.
    pub unsafe fn read(self: Pin<&mut Self>, buffer: *mut u8, mut size: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        if size > this.fEnd - this.fCurrent {
            size = this.fEnd - this.fCurrent;
        }
        let bytesRead = if buffer.is_null() {
            size
        } else {
            SkOSFile::sk_qread(this.file_ptr(), buffer.cast(), size, this.fCurrent)
        };
        if bytesRead == usize::MAX {
            return 0;
        }
        this.fCurrent += bytesRead;
        bytesRead
    }
    // Source: src/core/SkStream.cpp:224-229
    pub fn isAtEnd(&self) -> bool {
        if self.fCurrent == self.fEnd {
            return true;
        }
        self.fCurrent >= unsafe { SkOSFile::sk_fgetsize(self.file_ptr()) }
    }
    // Source: src/core/SkStream.cpp:231-234
    pub fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        this.fCurrent = this.fStart;
        true
    }
    // Source: src/core/SkStream.cpp:236-238
    pub fn duplicate(&self) -> Pin<Box<Self>> {
        Box::pin(Self::from_shared(
            self.fFILE.clone(),
            self.fEnd,
            self.fStart,
            self.fStart,
        ))
    }
    // Source: src/core/SkStream.cpp:240-243
    pub fn getPosition(&self) -> usize {
        crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),241; @condition b"fCurrent >= fStart\0".as_ptr(); self.fCurrent>=self.fStart);
        self.fCurrent - self.fStart
    }
    // Source: src/core/SkStream.cpp:245-248
    pub fn seek(self: Pin<&mut Self>, position: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        this.fCurrent =
            crate::src::core::SkSafeMath::SkSafeMath::Add(position, this.fStart).min(this.fEnd);
        true
    }
    // Source: src/core/SkStream.cpp:250-268
    pub fn r#move(self: Pin<&mut Self>, offset: c_long) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        if offset < 0 {
            if offset == c_long::MIN {
                this.fCurrent = this.fStart;
            } else if let Ok(amount) = usize::try_from(-offset) {
                if amount >= this.getPosition() {
                    this.fCurrent = this.fStart;
                } else {
                    this.fCurrent -= amount;
                }
            } else {
                this.fCurrent = this.fStart;
            }
        } else if let Ok(amount) = usize::try_from(offset) {
            this.fCurrent =
                crate::src::core::SkSafeMath::SkSafeMath::Add(this.fCurrent, amount).min(this.fEnd);
        } else {
            this.fCurrent = this.fEnd;
        }
        crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),266; @condition b"fCurrent >= fStart && fCurrent <= fEnd\0".as_ptr(); this.fCurrent>=this.fStart&&this.fCurrent<=this.fEnd);
        true
    }
    // Source: src/core/SkStream.cpp:270-272
    pub fn fork(&self) -> Pin<Box<Self>> {
        Box::pin(Self::from_shared(
            self.fFILE.clone(),
            self.fEnd,
            self.fStart,
            self.fCurrent,
        ))
    }
    // Source: src/core/SkStream.cpp:274-276
    pub fn getLength(&self) -> usize {
        self.fEnd - self.fStart
    }
}
// Source: src/core/SkStream.cpp:198-200
impl Drop for SkFILEStream {
    fn drop(&mut self) {
        unsafe { Pin::new_unchecked(self) }.close();
    }
}
// Header inheritance projections preserve the same concrete FILE owner and cursor.
unsafe impl SkStream for SkFILEStream {
    unsafe fn read(self: Pin<&mut Self>, p: *mut u8, n: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        Self::read(unsafe { Pin::new_unchecked(&mut *this) }, p, n)
    }
    fn isAtEnd(&self) -> bool {
        Self::isAtEnd(self)
    }
    fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::rewind(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn hasPosition(&self) -> bool {
        true
    }
    fn getPosition(&self) -> usize {
        Self::getPosition(self)
    }
    fn seek(self: Pin<&mut Self>, n: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::seek(unsafe { Pin::new_unchecked(&mut *this) }, n)
    }
    fn r#move(self: Pin<&mut Self>, n: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::r#move(unsafe { Pin::new_unchecked(&mut *this) }, n as c_long)
    }
    fn hasLength(&self) -> bool {
        true
    }
    fn getLength(&self) -> usize {
        Self::getLength(self)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStream>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStream>>> {
        Some(Self::fork(self))
    }
}
unsafe impl SkStreamRewindable for SkFILEStream {
    fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::rewind(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamRewindable>>> {
        Some(Self::duplicate(self))
    }
}
unsafe impl SkStreamSeekable for SkFILEStream {
    fn getPosition(&self) -> usize {
        Self::getPosition(self)
    }
    fn seek(self: Pin<&mut Self>, n: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::seek(unsafe { Pin::new_unchecked(&mut *this) }, n)
    }
    fn r#move(self: Pin<&mut Self>, n: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::r#move(unsafe { Pin::new_unchecked(&mut *this) }, n as c_long)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>> {
        Some(Self::fork(self))
    }
}
unsafe impl SkStreamAsset for SkFILEStream {
    fn getLength(&self) -> usize {
        Self::getLength(self)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        Some(Self::fork(self))
    }
}

// Source: src/core/SkStream.cpp:280-286
unsafe fn newFromParams(src: *const u8, size: usize, copy_data: bool) -> Arc<SkData> {
    if copy_data {
        unsafe { SkData::MakeWithCopy(src, size) }
    } else {
        unsafe { SkData::MakeWithoutCopy(src, size) }
    }
}
impl SkMemoryStream {
    // Source: include/core/SkStream.h:403-403
    pub fn getData(&self) -> Arc<SkData> {
        self.fData.clone()
    }
    // Source: src/core/SkStream.cpp:288-291
    pub fn new() -> Self {
        Self {
            fData: SkData::MakeEmpty(),
            fOffset: 0,
            _pin: PhantomPinned,
        }
    }
    // Source: src/core/SkStream.cpp:293-296
    /// Safety: initialize the allocated bytes before any read/peek through any derived owner.
    pub unsafe fn with_length(size: usize) -> Self {
        Self {
            fData: unsafe { SkData::MakeUninitialized(size) },
            fOffset: 0,
            _pin: PhantomPinned,
        }
    }
    // Source: src/core/SkStream.cpp:298-301
    /// Safety: src covers size bytes. With copy_data=false, its immutable backing lease
    /// lasts through this stream and every fork/duplicate. Typeface callbacks must not
    /// retain those borrowed streams past callback return, matching native SerialProcs.
    pub unsafe fn from_memory(src: *const u8, size: usize, copy_data: bool) -> Self {
        Self {
            fData: unsafe { newFromParams(src, size, copy_data) },
            fOffset: 0,
            _pin: PhantomPinned,
        }
    }
    // Source: src/core/SkStream.cpp:303-308
    pub fn from_data(data: Option<Arc<SkData>>) -> Self {
        Self {
            fData: data.unwrap_or_else(SkData::MakeEmpty),
            fOffset: 0,
            _pin: PhantomPinned,
        }
    }
    // Source: src/core/SkStream.cpp:310-312
    pub unsafe fn MakeCopy(data: *const u8, length: usize) -> Pin<Box<Self>> {
        Box::pin(unsafe { Self::from_memory(data, length, true) })
    }
    // Source: src/core/SkStream.cpp:314-316
    pub unsafe fn MakeDirect(data: *const u8, length: usize) -> Pin<Box<Self>> {
        Box::pin(unsafe { Self::from_memory(data, length, false) })
    }
    // Source: src/core/SkStream.cpp:318-320
    pub fn Make(data: Option<Arc<SkData>>) -> Pin<Box<Self>> {
        Box::pin(Self::from_data(data))
    }
    // Source: src/core/SkStream.cpp:322-325
    /// Safety: src transfers one sole malloc-compatible allocation of size bytes.
    pub unsafe fn setMemoryOwned(self: Pin<&mut Self>, src: *const u8, size: usize) {
        let this = unsafe { self.get_unchecked_mut() };
        this.fData = unsafe { SkData::MakeFromMalloc(src, size) };
        this.fOffset = 0;
    }
    // Source: src/core/SkStream.cpp:327-330
    pub unsafe fn setMemory(self: Pin<&mut Self>, src: *const u8, size: usize, copy_data: bool) {
        let this = unsafe { self.get_unchecked_mut() };
        this.fData = unsafe { newFromParams(src, size, copy_data) };
        this.fOffset = 0;
    }
    // Source: src/core/SkStream.cpp:332-339
    pub fn setData(self: Pin<&mut Self>, data: Option<Arc<SkData>>) {
        let this = unsafe { self.get_unchecked_mut() };
        this.fData = data.unwrap_or_else(SkData::MakeEmpty);
        this.fOffset = 0;
    }
    // Source: src/core/SkStream.cpp:341-353
    pub unsafe fn read(self: Pin<&mut Self>, buffer: *mut u8, mut size: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        let data_size = this.fData.size();
        crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),344; @condition b"fOffset <= dataSize\0".as_ptr(); this.fOffset<=data_size);
        if size > data_size - this.fOffset {
            size = data_size - this.fOffset;
        }
        if !buffer.is_null() && size != 0 {
            unsafe { ptr::copy_nonoverlapping(this.fData.bytes().add(this.fOffset), buffer, size) }
        }
        this.fOffset += size;
        size
    }
    // Source: src/core/SkStream.cpp:355-363
    // &mut is required because native const peek temporarily mutates this same offset.
    pub unsafe fn peek(self: Pin<&mut Self>, buffer: *mut u8, size: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),356; @condition b"buffer != nullptr\0".as_ptr(); !buffer.is_null());
        let current = this.fOffset;
        let read = unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read(buffer, size) };
        this.fOffset = current;
        read
    }
    // Source: src/core/SkStream.cpp:365-367
    pub fn isAtEnd(&self) -> bool {
        self.fOffset == self.fData.size()
    }
    // Source: src/core/SkStream.cpp:369-372
    pub fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        this.fOffset = 0;
        true
    }
    // Source: src/core/SkStream.cpp:374-376
    pub fn duplicate(&self) -> Pin<Box<Self>> {
        Box::pin(Self::from_data(Some(self.fData.clone())))
    }
    // Source: src/core/SkStream.cpp:378-380
    pub fn getPosition(&self) -> usize {
        self.fOffset
    }
    // Source: src/core/SkStream.cpp:382-387
    pub fn seek(self: Pin<&mut Self>, position: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        this.fOffset = if position > this.fData.size() {
            this.fData.size()
        } else {
            position
        };
        true
    }
    // Source: src/core/SkStream.cpp:389-391
    pub fn r#move(self: Pin<&mut Self>, offset: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        {
            let position = this.fOffset.wrapping_add(offset as usize);
            unsafe { Pin::new_unchecked(&mut *this) }.seek(position)
        }
    }
    // Source: src/core/SkStream.cpp:393-397
    pub fn fork(&self) -> Pin<Box<Self>> {
        let mut that = self.duplicate();
        that.as_mut().seek(self.fOffset);
        that
    }
    // Source: src/core/SkStream.cpp:399-401
    pub fn getLength(&self) -> usize {
        self.fData.size()
    }
    // Source: src/core/SkStream.cpp:403-405
    pub fn getMemoryBase(self: Pin<&mut Self>) -> *const u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.fData.data()
    }
    // Source: src/core/SkStream.cpp:407-409
    pub fn getAtPos(&self) -> *const u8 {
        self.fData.bytes().wrapping_add(self.fOffset)
    }
}
// Required inheritance projections share exactly the same concrete stream/Data owner.
unsafe impl SkStream for SkMemoryStream {
    unsafe fn read(self: Pin<&mut Self>, p: *mut u8, n: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { Self::read(unsafe { Pin::new_unchecked(&mut *this) }, p, n) }
    }
    unsafe fn peek(self: Pin<&mut Self>, p: *mut u8, n: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { Self::peek(unsafe { Pin::new_unchecked(&mut *this) }, p, n) }
    }
    fn isAtEnd(&self) -> bool {
        Self::isAtEnd(self)
    }
    fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::rewind(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn hasPosition(&self) -> bool {
        true
    }
    fn getPosition(&self) -> usize {
        Self::getPosition(self)
    }
    fn seek(self: Pin<&mut Self>, p: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::seek(unsafe { Pin::new_unchecked(&mut *this) }, p)
    }
    fn r#move(self: Pin<&mut Self>, o: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::r#move(unsafe { Pin::new_unchecked(&mut *this) }, o)
    }
    fn hasLength(&self) -> bool {
        true
    }
    fn getLength(&self) -> usize {
        Self::getLength(self)
    }
    fn getMemoryBase(self: Pin<&mut Self>) -> *const u8 {
        let this = unsafe { self.get_unchecked_mut() };
        Self::getMemoryBase(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn getData(&self) -> Option<Arc<SkData>> {
        Some(Self::getData(self))
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStream>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStream>>> {
        Some(Self::fork(self))
    }
}
unsafe impl SkStreamRewindable for SkMemoryStream {
    fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::rewind(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamRewindable>>> {
        Some(Self::duplicate(self))
    }
}
unsafe impl SkStreamSeekable for SkMemoryStream {
    fn getPosition(&self) -> usize {
        Self::getPosition(self)
    }
    fn seek(self: Pin<&mut Self>, p: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::seek(unsafe { Pin::new_unchecked(&mut *this) }, p)
    }
    fn r#move(self: Pin<&mut Self>, o: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::r#move(unsafe { Pin::new_unchecked(&mut *this) }, o)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>> {
        Some(Self::fork(self))
    }
}
unsafe impl SkStreamAsset for SkMemoryStream {
    fn getLength(&self) -> usize {
        Self::getLength(self)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        Some(Self::fork(self))
    }
}
unsafe impl SkStreamMemory for SkMemoryStream {
    fn getMemoryBase(self: Pin<&mut Self>) -> *const u8 {
        let this = unsafe { self.get_unchecked_mut() };
        Self::getMemoryBase(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamMemory>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamMemory>>> {
        Some(Self::fork(self))
    }
}

// Source: include/core/SkStream.h:222-279
// No concrete sink is fabricated: the actual destination owns write and its byte count.
pub unsafe trait SkWStream {
    unsafe fn write(self: Pin<&mut Self>, buffer: *const u8, size: usize) -> bool;
    // Source: src/core/SkStream.cpp:81-83
    fn flush(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
    }
    fn bytesWritten(&self) -> usize;
    fn write8(self: Pin<&mut Self>, value: u32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(value <= u8::MAX as u32);
        let v = value as u8;
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write(&v, 1) }
    }
    fn write16(self: Pin<&mut Self>, value: u32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(value <= u16::MAX as u32);
        let v = value as u16;
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write((&v as *const u16).cast(), 2) }
    }
    fn write32(self: Pin<&mut Self>, value: u32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write((&value as *const u32).cast(), 4) }
    }
    fn write64(self: Pin<&mut Self>, value: u64) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write((&value as *const u64).cast(), 8) }
    }
    unsafe fn writeText(self: Pin<&mut Self>, text: *const core::ffi::c_char) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(!text.is_null());
        let bytes = unsafe { core::ffi::CStr::from_ptr(text) }.to_bytes();
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write(text.cast(), bytes.len()) }
    }
    fn newline(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write(b"\n".as_ptr(), 1) }
    }
    fn writeBool(self: Pin<&mut Self>, value: bool) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { Pin::new_unchecked(&mut *this) }.write8(value as u32)
    }
    // Source: src/core/SkStream.cpp:85-90
    fn writeDecAsText(self: Pin<&mut Self>, dec: i32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        use super::SkString::{kSkStrAppendS32_MaxSize, SkStrAppendS32};
        let mut buffer = [core::mem::MaybeUninit::<u8>::uninit(); kSkStrAppendS32_MaxSize];
        let start = buffer.as_mut_ptr().cast::<u8>();
        unsafe {
            let stop = SkStrAppendS32(start, dec);
            unsafe { Pin::new_unchecked(&mut *this) }.write(start, stop.offset_from(start) as usize)
        }
    }
    // Source: src/core/SkStream.cpp:92-97
    fn writeBigDecAsText(self: Pin<&mut Self>, dec: i64, minDigits: i32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        use super::SkString::{kSkStrAppendU64_MaxSize, SkStrAppendU64};
        let mut buffer = [core::mem::MaybeUninit::<u8>::uninit(); kSkStrAppendU64_MaxSize];
        let start = buffer.as_mut_ptr().cast::<u8>();
        unsafe {
            let stop = SkStrAppendU64(start, dec as u64, minDigits);
            unsafe { Pin::new_unchecked(&mut *this) }.write(start, stop.offset_from(start) as usize)
        }
    }
    // Source: src/core/SkStream.cpp:99-104
    fn writeHexAsText(self: Pin<&mut Self>, hex: u32, digits: i32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        let mut tmp = super::SkString::SkString::new();
        tmp.appendHex(hex, digits);
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write(tmp.c_str(), tmp.size()) }
    }
    // Source: src/core/SkStream.cpp:106-111
    fn writeScalarAsText(self: Pin<&mut Self>, value: f32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        use super::SkString::{kSkStrAppendScalar_MaxSize, SkStrAppendScalar};
        let mut buffer = [core::mem::MaybeUninit::<u8>::uninit(); kSkStrAppendScalar_MaxSize];
        let start = buffer.as_mut_ptr().cast::<u8>();
        unsafe {
            let stop = SkStrAppendScalar(start, value);
            unsafe { Pin::new_unchecked(&mut *this) }.write(start, stop.offset_from(start) as usize)
        }
    }
    // Source: src/core/SkStream.cpp:113-115
    fn writeScalar(self: Pin<&mut Self>, value: f32) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write((&value as *const f32).cast(), 4) }
    }
    // Source: src/core/SkStream.cpp:126-144
    fn writePackedUInt(self: Pin<&mut Self>, value: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        let mut data = [0u8; 5];
        let len;
        if value <= MAX_BYTE_U8 {
            data[0] = value as u8;
            len = 1;
        } else if value <= 65535 {
            let value16 = value as u16;
            data[0] = BYTE_SENTINEL_U16;
            data[1..3].copy_from_slice(&value16.to_ne_bytes());
            len = 3;
        } else {
            debug_assert!(value <= u32::MAX as usize);
            let value32 = value as u32;
            data[0] = BYTE_SENTINEL_U32;
            data[1..5].copy_from_slice(&value32.to_ne_bytes());
            len = 5;
        }
        unsafe { unsafe { Pin::new_unchecked(&mut *this) }.write(data.as_ptr(), len) }
    }
    // Source: src/core/SkStream.cpp:146-162
    /// Safety: input must initialize every requested scratch byte through length,
    /// exactly the native full-read precondition. No zeros replace short input.
    unsafe fn writeStream(
        self: Pin<&mut Self>,
        mut input: Pin<&mut dyn SkStream>,
        mut length: usize,
    ) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        let mut scratch = [core::mem::MaybeUninit::<u8>::uninit(); 1024];
        let max = mem::size_of_val(&scratch);
        while length != 0 {
            let mut n = length;
            if n > max {
                n = max;
            }
            unsafe {
                input.as_mut().read(scratch.as_mut_ptr().cast(), n);
                if !unsafe { Pin::new_unchecked(&mut *this) }.write(scratch.as_ptr().cast(), n) {
                    return false;
                }
            }
            length -= n;
        }
        true
    }
}
// Source: src/core/SkStream.cpp:117-124
pub fn SizeOfPackedUInt(value: usize) -> i32 {
    if value <= MAX_BYTE_U8 {
        1
    } else if value <= 65535 {
        3
    } else {
        5
    }
}

// Source: include/core/SkStream.h:283-293
pub struct SkNullWStream {
    fBytesWritten: usize,
    _pin: PhantomPinned,
}
impl SkNullWStream {
    pub fn new() -> Self {
        Self {
            fBytesWritten: 0,
            _pin: PhantomPinned,
        }
    }
}
unsafe impl SkWStream for SkNullWStream {
    unsafe fn write(self: Pin<&mut Self>, _buffer: *const u8, n: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        this.fBytesWritten = this.fBytesWritten.wrapping_add(n);
        true
    }
    fn flush(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
    }
    fn bytesWritten(&self) -> usize {
        self.fBytesWritten
    }
}

// Source: include/core/SkStream.h:444-460
pub struct SkFILEWStream {
    fFILE: *mut FILE,
    _pin: PhantomPinned,
}
impl SkFILEWStream {
    // Source: src/core/SkStream.cpp:414-417
    pub unsafe fn new(path: *const c_char) -> Self {
        Self {
            fFILE: SkOSFile::sk_fopen(path, SkOSFile::kWrite_SkFILE_Flag),
            _pin: PhantomPinned,
        }
    }
    // Source: include/core/SkStream.h:451
    pub fn isValid(&self) -> bool {
        !self.fFILE.is_null()
    }
    // Source: src/core/SkStream.cpp:426-428
    pub fn bytesWritten(&self) -> usize {
        unsafe { SkOSFile::sk_ftell(self.fFILE) }
    }
    // Source: src/core/SkStream.cpp:430-444
    pub unsafe fn write(self: Pin<&mut Self>, buffer: *const u8, size: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        if this.fFILE.is_null() {
            return false;
        }
        if SkOSFile::sk_fwrite(buffer.cast(), size, this.fFILE) != size {
            // Source: src/core/SkStream.cpp:437-437
            #[cfg(sk_debug)]
            crate::SkDebugf!(@source b"src/core/SkStream.cpp\0".as_ptr(),437; b"SkFILEWStream failed writing %zu bytes\n\0".as_ptr(),size);
            SkOSFile::sk_fclose(this.fFILE);
            this.fFILE = ptr::null_mut();
            return false;
        }
        true
    }
    // Source: src/core/SkStream.cpp:446-451
    pub fn flush(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        if !this.fFILE.is_null() {
            unsafe {
                SkOSFile::sk_fflush(this.fFILE);
            }
        }
    }
    // Source: src/core/SkStream.cpp:453-459
    pub fn fsync(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { Pin::new_unchecked(&mut *this) }.flush();
        if !this.fFILE.is_null() {
            unsafe {
                SkOSFile::sk_fsync(this.fFILE);
            }
        }
    }
}
// Source: src/core/SkStream.cpp:419-424
impl Drop for SkFILEWStream {
    fn drop(&mut self) {
        if !self.fFILE.is_null() {
            unsafe {
                SkOSFile::sk_fclose(self.fFILE);
            }
        }
    }
}
unsafe impl SkWStream for SkFILEWStream {
    unsafe fn write(self: Pin<&mut Self>, p: *const u8, n: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::write(unsafe { Pin::new_unchecked(&mut *this) }, p, n)
    }
    fn flush(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::flush(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn bytesWritten(&self) -> usize {
        Self::bytesWritten(self)
    }
}

// Source: include/core/SkStream.h:464-520
// Three native pointer/count fields retain the exact linked-block owner. The
// explicit native dynamic-sink move remains available; pinned read-stream
// objects have no corresponding move operation. reset/Drop frees each block once.
pub struct SkDynamicMemoryWStream {
    fHead: *mut DynamicBlock,
    fTail: *mut DynamicBlock,
    fBytesWrittenBeforeTail: usize,
    _pin: PhantomPinned,
}
// Source: include/core/SkStream.h:467-468; src/core/SkStream.cpp:497-514
impl Unpin for SkDynamicMemoryWStream {}
impl Default for SkDynamicMemoryWStream {
    fn default() -> Self {
        Self::new()
    }
}
impl SkDynamicMemoryWStream {
    pub fn new() -> Self {
        Self {
            fHead: ptr::null_mut(),
            fTail: ptr::null_mut(),
            fBytesWrittenBeforeTail: 0,
            _pin: PhantomPinned,
        }
    }
}
// Source: src/core/SkStream.cpp:463-469
unsafe fn sk_memcpy_4bytes(dst: *mut u8, src: *const u8, size: usize) {
    // Both native branches perform the same memcpy, including unaligned four-byte copies.
    if size != 0 {
        unsafe { ptr::copy_nonoverlapping(src, dst, size) }
    }
}
// Source: src/core/SkStream.cpp:471-471
const DYNAMIC_MIN_BLOCK_SIZE: usize = 4096;
// Source: src/core/SkStream.cpp:473-495
#[repr(C)]
struct DynamicBlock {
    fNext: *mut DynamicBlock,
    fCurr: *mut u8,
    fStop: *mut u8,
}
impl DynamicBlock {
    unsafe fn start(block: *mut Self) -> *mut u8 {
        unsafe { block.add(1).cast() }
    }
    unsafe fn avail(&self) -> usize {
        unsafe { self.fStop.offset_from(self.fCurr) as usize }
    }
    unsafe fn written(&self) -> usize {
        unsafe {
            self.fCurr
                .offset_from(Self::start(self as *const Self as *mut Self)) as usize
        }
    }
    unsafe fn init(block: *mut Self, size: usize) {
        let start = unsafe { Self::start(block) };
        unsafe {
            ptr::write(
                block,
                Self {
                    fNext: ptr::null_mut(),
                    fCurr: start,
                    fStop: start.add(size),
                },
            )
        }
    }
    unsafe fn append(&mut self, data: *const u8, size: usize) -> *const u8 {
        debug_assert!(unsafe { self.avail() } >= size);
        unsafe {
            sk_memcpy_4bytes(self.fCurr, data, size);
            self.fCurr = self.fCurr.add(size);
            data.add(size)
        }
    }
}
// Actual selected native malloc/free ABI; canfail preserves the partial-tail-write
// behavior on allocation failure. No Rust Vec replaces block allocation policy.
unsafe extern "C" {
    #[link_name = "malloc"]
    fn stream_malloc(size: usize) -> *mut core::ffi::c_void;
}
impl SkDynamicMemoryWStream {
    // Source: src/core/SkStream.cpp:497-514
    // Explicitly emptying a moved-from Rust value is needed only when callers
    // retain that value, as the native writeToAndReset overload does.
    fn take_blocks(self: Pin<&mut Self>) -> Self {
        let this = unsafe { self.get_unchecked_mut() };
        mem::replace(this, Self::new())
    }
    // Source: src/core/SkStream.cpp:519-528
    pub fn reset(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut block = this.fHead;
        while !block.is_null() {
            unsafe {
                let next = (*block).fNext;
                crate::src::core::SkContainers::sk_free(block.cast());
                block = next;
            }
        }
        this.fHead = ptr::null_mut();
        this.fTail = ptr::null_mut();
        this.fBytesWrittenBeforeTail = 0;
    }
    // Source: src/core/SkStream.cpp:530-537
    pub fn bytesWritten(&self) -> usize {
        self.validate();
        if self.fTail.is_null() {
            0
        } else {
            self.fBytesWrittenBeforeTail + unsafe { (*self.fTail).written() }
        }
    }
    // Source: src/core/SkStream.cpp:539-575
    /// Safety: buffer covers count immutable initialized bytes and does not alias
    /// this stream's writable tail. Allocation failure may retain a partial write.
    pub unsafe fn write(self: Pin<&mut Self>, mut buffer: *const u8, mut count: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        if count > 0 {
            crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),541; @condition b"buffer\0".as_ptr(); !buffer.is_null());
            unsafe {
                if !this.fTail.is_null() && (*this.fTail).avail() > 0 {
                    let size = (*this.fTail).avail().min(count);
                    buffer = (*this.fTail).append(buffer, size);
                    crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),547; @condition b"count >= size\0".as_ptr(); count>=size);
                    count -= size;
                    if count == 0 {
                        return true;
                    }
                }
                let size = count
                    .max(DYNAMIC_MIN_BLOCK_SIZE - mem::size_of::<DynamicBlock>())
                    .wrapping_add(3)
                    & !3;
                let block = stream_malloc(mem::size_of::<DynamicBlock>().wrapping_add(size))
                    .cast::<DynamicBlock>();
                if block.is_null() {
                    this.validate();
                    return false;
                }
                DynamicBlock::init(block, size);
                (*block).append(buffer, count);
                if !this.fTail.is_null() {
                    this.fBytesWrittenBeforeTail += (*this.fTail).written();
                    (*this.fTail).fNext = block;
                } else {
                    this.fHead = block;
                    this.fTail = block;
                }
                this.fTail = block;
                this.validate();
            }
        }
        true
    }
    // Source: src/core/SkStream.cpp:577-593
    pub fn writeToDynamicAndReset(self: Pin<&mut Self>, mut dst: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(!ptr::eq(this, &*dst));
        if this.bytesWritten() == 0 {
            return true;
        }
        if dst.bytesWritten() == 0 {
            *dst = unsafe { Pin::new_unchecked(&mut *this) }.take_blocks();
            return true;
        }
        unsafe {
            (*dst.fTail).fNext = this.fHead;
            dst.fBytesWrittenBeforeTail += this.fBytesWrittenBeforeTail + (*dst.fTail).written();
        }
        dst.fTail = this.fTail;
        this.fHead = ptr::null_mut();
        this.fTail = ptr::null_mut();
        this.fBytesWrittenBeforeTail = 0;
        true
    }
    // Source: src/core/SkStream.cpp:595-610
    pub fn prependToAndReset(self: Pin<&mut Self>, mut dst: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(!ptr::eq(this, &*dst));
        if this.bytesWritten() == 0 {
            return;
        }
        if dst.bytesWritten() == 0 {
            *dst = unsafe { Pin::new_unchecked(&mut *this) }.take_blocks();
            return;
        }
        unsafe {
            (*this.fTail).fNext = dst.fHead;
            dst.fHead = this.fHead;
            dst.fBytesWrittenBeforeTail += this.fBytesWrittenBeforeTail + (*this.fTail).written();
        }
        this.fHead = ptr::null_mut();
        this.fTail = ptr::null_mut();
        this.fBytesWrittenBeforeTail = 0;
    }
    // Source: src/core/SkStream.cpp:613-633
    /// Safety: buffer covers count writable bytes and is disjoint from stream blocks.
    pub unsafe fn read(&self, mut buffer: *mut u8, mut offset: usize, mut count: usize) -> bool {
        if offset.wrapping_add(count) > self.bytesWritten() {
            return false;
        }
        let mut block = self.fHead;
        unsafe {
            while !block.is_null() {
                let size = (*block).written();
                if offset < size {
                    let part = if offset.wrapping_add(count) > size {
                        size - offset
                    } else {
                        count
                    };
                    if part != 0 {
                        ptr::copy_nonoverlapping(
                            DynamicBlock::start(block).add(offset),
                            buffer,
                            part,
                        );
                    }
                    if count <= part {
                        return true;
                    }
                    count -= part;
                    buffer = buffer.add(part);
                }
                offset = if offset > size { offset - size } else { 0 };
                block = (*block).fNext;
            }
        }
        false
    }
    // Source: src/core/SkStream.cpp:635-644
    pub unsafe fn copyTo(&self, mut dst: *mut u8) {
        crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),636; @condition b"dst\0".as_ptr(); !dst.is_null());
        let mut block = self.fHead;
        unsafe {
            while !block.is_null() {
                let size = (*block).written();
                if size != 0 {
                    ptr::copy_nonoverlapping(DynamicBlock::start(block), dst, size);
                }
                dst = dst.add(size);
                block = (*block).fNext;
            }
        }
    }
    // Source: src/core/SkStream.cpp:646-654
    pub fn writeToStream(&self, mut dst: Pin<&mut dyn SkWStream>) -> bool {
        let mut block = self.fHead;
        unsafe {
            while !block.is_null() {
                if !dst
                    .as_mut()
                    .write(DynamicBlock::start(block), (*block).written())
                {
                    return false;
                }
                block = (*block).fNext;
            }
        }
        true
    }
    // Source: src/core/SkStream.cpp:656-669
    pub fn padToAlign4(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        if !this.fTail.is_null() {
            unsafe {
                let pad = (*this.fTail).written().wrapping_neg() & 3;
                if pad != 0 {
                    let zero = 0u32;
                    (*this.fTail).append((&zero as *const u32).cast(), pad);
                }
            }
        }
    }
    // Source: src/core/SkStream.cpp:672-691
    /// Safety: nonnull dst covers bytesWritten writable bytes disjoint from blocks.
    pub unsafe fn copyToAndReset(self: Pin<&mut Self>, mut dst: *mut u8) {
        let this = unsafe { self.get_unchecked_mut() };
        if dst.is_null() {
            unsafe { Pin::new_unchecked(&mut *this) }.reset();
            return;
        }
        let mut block = this.fHead;
        unsafe {
            while !block.is_null() {
                let len = (*block).written();
                if len != 0 {
                    ptr::copy_nonoverlapping(DynamicBlock::start(block), dst, len);
                }
                dst = dst.add(len);
                let next = (*block).fNext;
                crate::src::core::SkContainers::sk_free(block.cast());
                block = next;
            }
        }
        this.fHead = ptr::null_mut();
        this.fTail = ptr::null_mut();
        this.fBytesWrittenBeforeTail = 0;
    }
    // Source: src/core/SkStream.cpp:693-709
    pub fn writeToAndReset(self: Pin<&mut Self>, mut dst: Pin<&mut dyn SkWStream>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        let mut good = true;
        let mut block = this.fHead;
        unsafe {
            while !block.is_null() {
                if good
                    && !dst
                        .as_mut()
                        .write(DynamicBlock::start(block), (*block).written())
                {
                    good = false;
                }
                let next = (*block).fNext;
                crate::src::core::SkContainers::sk_free(block.cast());
                block = next;
            }
        }
        this.fHead = ptr::null_mut();
        this.fTail = ptr::null_mut();
        this.fBytesWrittenBeforeTail = 0;
        good
    }
    // Source: src/core/SkStream.cpp:711-719
    pub fn detachAsData(self: Pin<&mut Self>) -> Arc<SkData> {
        let this = unsafe { self.get_unchecked_mut() };
        let size = this.bytesWritten();
        if size == 0 {
            return SkData::MakeEmpty();
        }
        unsafe {
            let data = SkData::MakeUninitialized(size);
            unsafe { Pin::new_unchecked(&mut *this) }.copyToAndReset(data.writable_data());
            data
        }
    }
    // Source: src/core/SkStream.cpp:721-732
    pub fn detachAsVector(self: Pin<&mut Self>) -> Vec<u8> {
        let this = unsafe { self.get_unchecked_mut() };
        let size = this.bytesWritten();
        let mut result = Vec::new();
        if size == 0 {
            return result;
        }
        result.reserve_exact(size);
        unsafe {
            unsafe { Pin::new_unchecked(&mut *this) }.copyToAndReset(result.as_mut_ptr());
            result.set_len(size);
        }
        result
    }
    // Source: src/core/SkStream.cpp:734-753
    fn validate(&self) {
        #[cfg(sk_debug)]
        {
            if self.fHead.is_null() {
                debug_assert!(self.fTail.is_null());
                crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),738; @condition b"fBytesWrittenBeforeTail == 0\0".as_ptr(); (self.fBytesWrittenBeforeTail)==(0));
                return;
            }
            crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),741; @condition b"fTail\0".as_ptr(); !self.fTail.is_null());
            let mut bytes = 0;
            let mut block = self.fHead;
            unsafe {
                while !block.is_null() {
                    if !(*block).fNext.is_null() {
                        bytes += (*block).written();
                    }
                    block = (*block).fNext;
                }
            }
            crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),751; @condition b"bytes == fBytesWrittenBeforeTail\0".as_ptr(); (bytes)==(self.fBytesWrittenBeforeTail));
        }
    }
}
// Source: src/core/SkStream.cpp:515-517
impl Drop for SkDynamicMemoryWStream {
    fn drop(&mut self) {
        unsafe { Pin::new_unchecked(self) }.reset();
    }
}
unsafe impl SkWStream for SkDynamicMemoryWStream {
    unsafe fn write(self: Pin<&mut Self>, buffer: *const u8, size: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { Self::write(unsafe { Pin::new_unchecked(&mut *this) }, buffer, size) }
    }
    fn bytesWritten(&self) -> usize {
        Self::bytesWritten(self)
    }
}

// Source: src/core/SkStream.cpp:757-771
// Arc substitutes for the real native refcount of the immutable transferred
// chain. No stream copy owns/free blocks independently of this shared owner.
struct BlockMemoryRefCnt {
    fHead: *mut DynamicBlock,
}
impl Drop for BlockMemoryRefCnt {
    fn drop(&mut self) {
        let mut block = self.fHead;
        unsafe {
            while !block.is_null() {
                let next = (*block).fNext;
                crate::src::core::SkContainers::sk_free(block.cast());
                block = next;
            }
        }
    }
}
// Source: src/core/SkStream.cpp:773-891
struct SkBlockMemoryStream {
    _pin: PhantomPinned,
    // Source: src/core/SkStream.cpp:890
    fCurrentOffset: usize,
    // Source: src/core/SkStream.cpp:889
    fOffset: usize,
    // Source: src/core/SkStream.cpp:888
    fSize: usize,
    // Source: src/core/SkStream.cpp:887
    fCurrent: *mut DynamicBlock,
    // Source: src/core/SkStream.cpp:886
    fBlockMemory: Arc<BlockMemoryRefCnt>,
}
impl SkBlockMemoryStream {
    fn new(owner: Arc<BlockMemoryRefCnt>, size: usize) -> Self {
        let current = owner.fHead;
        Self {
            fBlockMemory: owner,
            fCurrent: current,
            fSize: size,
            fOffset: 0,
            fCurrentOffset: 0,
            _pin: PhantomPinned,
        }
    }
    unsafe fn read(self: Pin<&mut Self>, mut buffer: *mut u8, raw_count: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        let mut count = raw_count;
        if this.fOffset.wrapping_add(count) > this.fSize {
            count = this.fSize - this.fOffset;
        }
        let mut left = count;
        unsafe {
            while !this.fCurrent.is_null() {
                let available = (*this.fCurrent).written() - this.fCurrentOffset;
                let part = left.min(available);
                if !buffer.is_null() {
                    if part != 0 {
                        ptr::copy_nonoverlapping(
                            DynamicBlock::start(this.fCurrent).add(this.fCurrentOffset),
                            buffer,
                            part,
                        );
                    }
                    buffer = buffer.add(part);
                }
                if left <= part {
                    this.fCurrentOffset += part;
                    this.fOffset += count;
                    return count;
                }
                left -= part;
                this.fCurrent = (*this.fCurrent).fNext;
                this.fCurrentOffset = 0;
            }
        }
        crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),801; @condition b"false\0".as_ptr(); false);
        0
    }
    fn isAtEnd(&self) -> bool {
        self.fOffset == self.fSize
    }
    unsafe fn peek(&self, mut buffer: *mut u8, mut count: usize) -> usize {
        debug_assert!(!buffer.is_null());
        count = count.min(self.fSize - self.fOffset);
        let mut left = count;
        let mut current = self.fCurrent;
        let mut offset = self.fCurrentOffset;
        unsafe {
            while left != 0 {
                crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),819; @condition b"current\0".as_ptr(); !current.is_null());
                let part = ((*current).written() - offset).min(left);
                if part != 0 {
                    ptr::copy_nonoverlapping(
                        DynamicBlock::start(current).add(offset),
                        buffer,
                        part,
                    );
                }
                left -= part;
                buffer = buffer.add(part);
                current = (*current).fNext;
                offset = 0;
            }
        }
        count
    }
    fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        this.fCurrent = this.fBlockMemory.fHead;
        this.fOffset = 0;
        this.fCurrentOffset = 0;
        true
    }
    fn duplicate(&self) -> Pin<Box<Self>> {
        Box::pin(Self::new(self.fBlockMemory.clone(), self.fSize))
    }
    fn getPosition(&self) -> usize {
        self.fOffset
    }
    fn seek(self: Pin<&mut Self>, position: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        if position >= this.fOffset {
            let skip = position - this.fOffset;
            return unsafe {
                unsafe { Pin::new_unchecked(&mut *this) }.read(ptr::null_mut(), skip)
            } == skip;
        }
        let back = this.fOffset - position;
        if back <= this.fCurrentOffset {
            this.fCurrentOffset -= back;
            this.fOffset -= back;
            return true;
        }
        unsafe { Pin::new_unchecked(&mut *this) }.rewind()
            && unsafe { unsafe { Pin::new_unchecked(&mut *this) }.read(ptr::null_mut(), position) }
                == position
    }
    fn r#move(self: Pin<&mut Self>, offset: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        {
            let position = this.fOffset.wrapping_add(offset as usize);
            unsafe { Pin::new_unchecked(&mut *this) }.seek(position)
        }
    }
    fn fork(&self) -> Pin<Box<Self>> {
        let mut that = self.duplicate();
        unsafe {
            let projected = that.as_mut().get_unchecked_mut();
            projected.fCurrent = self.fCurrent;
            projected.fOffset = self.fOffset;
            projected.fCurrentOffset = self.fCurrentOffset;
        }
        that
    }
    fn getLength(&self) -> usize {
        self.fSize
    }
    fn getMemoryBase(&self) -> *const u8 {
        unsafe {
            let head = self.fBlockMemory.fHead;
            if !head.is_null() && (*head).fNext.is_null() {
                DynamicBlock::start(head)
            } else {
                ptr::null()
            }
        }
    }
}
// The four actual inherited virtual projections all retain the same Arc chain.
unsafe impl SkStream for SkBlockMemoryStream {
    unsafe fn read(self: Pin<&mut Self>, p: *mut u8, n: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { Self::read(unsafe { Pin::new_unchecked(&mut *this) }, p, n) }
    }
    unsafe fn peek(self: Pin<&mut Self>, p: *mut u8, n: usize) -> usize {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { Self::peek(this, p, n) }
    }
    fn isAtEnd(&self) -> bool {
        Self::isAtEnd(self)
    }
    fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::rewind(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn hasPosition(&self) -> bool {
        true
    }
    fn getPosition(&self) -> usize {
        Self::getPosition(self)
    }
    fn seek(self: Pin<&mut Self>, n: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::seek(unsafe { Pin::new_unchecked(&mut *this) }, n)
    }
    fn r#move(self: Pin<&mut Self>, n: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::r#move(unsafe { Pin::new_unchecked(&mut *this) }, n)
    }
    fn hasLength(&self) -> bool {
        true
    }
    fn getLength(&self) -> usize {
        Self::getLength(self)
    }
    fn getMemoryBase(self: Pin<&mut Self>) -> *const u8 {
        let this = unsafe { self.get_unchecked_mut() };
        Self::getMemoryBase(this)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStream>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStream>>> {
        Some(Self::fork(self))
    }
}
unsafe impl SkStreamRewindable for SkBlockMemoryStream {
    fn rewind(self: Pin<&mut Self>) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::rewind(unsafe { Pin::new_unchecked(&mut *this) })
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamRewindable>>> {
        Some(Self::duplicate(self))
    }
}
unsafe impl SkStreamSeekable for SkBlockMemoryStream {
    fn getPosition(&self) -> usize {
        Self::getPosition(self)
    }
    fn seek(self: Pin<&mut Self>, n: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::seek(unsafe { Pin::new_unchecked(&mut *this) }, n)
    }
    fn r#move(self: Pin<&mut Self>, n: isize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        Self::r#move(unsafe { Pin::new_unchecked(&mut *this) }, n)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamSeekable>>> {
        Some(Self::fork(self))
    }
}
unsafe impl SkStreamAsset for SkBlockMemoryStream {
    fn getLength(&self) -> usize {
        Self::getLength(self)
    }
    fn onDuplicate(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        Some(Self::duplicate(self))
    }
    fn onFork(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        Some(Self::fork(self))
    }
}
impl SkDynamicMemoryWStream {
    // Source: src/core/SkStream.cpp:893-911
    pub fn detachAsStream(self: Pin<&mut Self>) -> Pin<Box<dyn SkStreamAsset>> {
        let this = unsafe { self.get_unchecked_mut() };
        if this.fHead.is_null() {
            return SkMemoryStream::Make(None);
        }
        unsafe {
            if this.fHead == this.fTail {
                let used = (*this.fTail).fCurr.offset_from(this.fTail.cast::<u8>()) as usize;
                this.fTail =
                    crate::src::core::SkContainers::sk_realloc_throw(this.fTail.cast(), used)
                        .cast();
                this.fHead = this.fTail;
                (*this.fTail).fCurr = this.fTail.cast::<u8>().add(used);
                (*this.fTail).fStop = (*this.fTail).fCurr;
                debug_assert!((*this.fTail).fNext.is_null());
                crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),903; @condition b"0 == fBytesWrittenBeforeTail\0".as_ptr(); (this.fBytesWrittenBeforeTail)==(0));
            }
        }
        let stream = Box::pin(SkBlockMemoryStream::new(
            Arc::new(BlockMemoryRefCnt { fHead: this.fHead }),
            this.bytesWritten(),
        ));
        this.fHead = ptr::null_mut();
        unsafe { Pin::new_unchecked(&mut *this) }.reset();
        stream
    }
}

// Source: src/core/SkStream.cpp:913-922
unsafe fn mmap_filename(path: *const c_char) -> Option<Arc<SkData>> {
    let file = SkOSFile::sk_fopen(path, SkOSFile::kRead_SkFILE_Flag);
    if file.is_null() {
        return None;
    }
    let data = SkData::MakeFromFILE(file);
    SkOSFile::sk_fclose(file);
    data
}
// Source: src/core/SkStream.cpp:924-936
/// Safety: path is a readable nul-terminated native path; returned streams own
/// their actual mapping or file backing, independent of this borrowed path.
pub unsafe fn MakeFromFile(path: *const c_char) -> Option<Pin<Box<dyn SkStreamAsset>>> {
    if let Some(data) = mmap_filename(path) {
        return Some(Box::pin(SkMemoryStream::from_data(Some(data))));
    }
    let stream = Box::pin(SkFILEStream::from_path(path));
    if !stream.isValid() {
        return None;
    }
    Some(stream)
}
// Source: include/core/SkStream.h:38-39
impl dyn SkStream {
    pub unsafe fn MakeFromFile(path: *const c_char) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        MakeFromFile(path)
    }
}

// Source: src/core/SkStream.cpp:938-938
pub mod SkStreamPriv {
    use super::{Pin, SkData, SkDynamicMemoryWStream, SkStream, SkWStream};
    use std::sync::Arc;

    // Source: src/core/SkStream.cpp:940-940
    const kBufferSize: usize = 4096;

    // Source: src/core/SkStream.cpp:942-956
    pub fn CopyStreamToData(mut stream: Pin<&mut dyn SkStream>) -> Option<Arc<SkData>> {
        if stream.hasLength() {
            let length = stream.getLength();
            return SkData::MakeFromStream(stream.as_mut(), length);
        }
        let mut temp_stream = core::pin::pin!(SkDynamicMemoryWStream::new());
        // Only the prefix initialized by read is ever passed to write.
        let mut buffer = [core::mem::MaybeUninit::<u8>::uninit(); kBufferSize];
        loop {
            let bytes_read = unsafe {
                stream
                    .as_mut()
                    .read(buffer.as_mut_ptr().cast(), kBufferSize)
            };
            // Native intentionally ignores the write result on this branch.
            let _ = unsafe {
                temp_stream
                    .as_mut()
                    .write(buffer.as_ptr().cast(), bytes_read)
            };
            if stream.isAtEnd() {
                break;
            }
        }
        Some(temp_stream.as_mut().detachAsData())
    }

    // Source: src/core/SkStream.cpp:958-978
    pub fn Copy(mut out: Pin<&mut dyn SkWStream>, mut input: Pin<&mut dyn SkStream>) -> bool {
        let base = input.as_mut().getMemoryBase();
        if !base.is_null() && input.hasPosition() && input.hasLength() {
            let position = input.getPosition();
            let length = input.getLength();
            crate::SkASSERT!(@source b"src/core/SkStream.cpp\0".as_ptr(),964; @condition b"length >= position\0".as_ptr(); length >= position);
            // This native shortcut does not advance the input stream.
            return unsafe {
                out.as_mut()
                    .write(base.add(position), length.wrapping_sub(position))
            };
        }
        let mut scratch = [core::mem::MaybeUninit::<u8>::uninit(); kBufferSize];
        loop {
            let count = unsafe {
                input
                    .as_mut()
                    .read(scratch.as_mut_ptr().cast(), kBufferSize)
            };
            if count == 0 {
                return true;
            }
            if !unsafe { out.as_mut().write(scratch.as_ptr().cast(), count) } {
                return false;
            }
        }
    }

    // Source: src/core/SkStream.cpp:980-992
    pub fn RemainingLengthIsBelow(stream: &dyn SkStream, len: usize) -> bool {
        if stream.hasLength() {
            if stream.hasPosition() {
                let remaining_bytes = stream.getLength().wrapping_sub(stream.getPosition());
                return remaining_bytes < len;
            }
            return stream.getLength() < len;
        }
        false
    }
}

// Source: src/core/SkStream.cpp:994-1000
// Actual native DebugfStream virtual dispatch uses the same pinned sink object.
unsafe impl SkWStream for super::SkStreamPriv::DebugfStream {
    unsafe fn write(self: Pin<&mut Self>, buffer: *const u8, size: usize) -> bool {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkDebugf!(@source b"src/core/SkStream.cpp\0".as_ptr(),995; b"%.*s\0".as_ptr(),size as core::ffi::c_int,buffer);
        this.fBytesWritten = this.fBytesWritten.wrapping_add(size);
        true
    }
    fn bytesWritten(&self) -> usize {
        self.fBytesWritten
    }
}
