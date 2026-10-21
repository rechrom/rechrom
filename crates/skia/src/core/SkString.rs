use core::{
    ffi::{c_int, c_void},
    mem::{ManuallyDrop, MaybeUninit},
    sync::atomic::{AtomicI32, Ordering},
};
unsafe extern "C" {
    fn strlen(s: *const u8) -> usize;
    fn strncmp(a: *const u8, b: *const u8, n: usize) -> c_int;
    fn strstr(a: *const u8, b: *const u8) -> *const u8;
    fn strchr(a: *const u8, c: c_int) -> *const u8;
    fn strrchr(a: *const u8, c: c_int) -> *const u8;
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn snprintf(out: *mut u8, size: usize, format: *const u8, ...) -> c_int;
}
// Source: include/core/SkString.h:26-30
pub unsafe fn SkStrStartsWith(s: *const u8, prefix: *const u8) -> bool {
    crate::SkASSERT!(@source b"include/core/SkString.h\0".as_ptr(),27; @condition b"string\0".as_ptr(); !s.is_null());
    crate::SkASSERT!(@source b"include/core/SkString.h\0".as_ptr(),28; @condition b"prefixStr\0".as_ptr(); !prefix.is_null());
    strncmp(s, prefix, strlen(prefix)) == 0
}
// Source: include/core/SkString.h:31-34
pub unsafe fn SkStrStartsWithChar(s: *const u8, c: u8) -> bool {
    crate::SkASSERT!(@source b"include/core/SkString.h\0".as_ptr(),32; @condition b"string\0".as_ptr(); !s.is_null());
    c == *s
}
// Source: include/core/SkString.h:41-45
pub unsafe fn SkStrFind(s: *const u8, sub: *const u8) -> i32 {
    let first = strstr(s, sub);
    if first.is_null() {
        -1
    } else {
        first.offset_from(s) as i32
    }
}
// Source: include/core/SkString.h:47-51
pub unsafe fn SkStrFindLastOf(s: *const u8, c: u8) -> i32 {
    let last = strrchr(s, c as c_int);
    if last.is_null() {
        -1
    } else {
        last.offset_from(s) as i32
    }
}
// Source: include/core/SkString.h:53-57
pub unsafe fn SkStrContains(s: *const u8, sub: *const u8) -> bool {
    crate::SkASSERT!(@source b"include/core/SkString.h\0".as_ptr(),54; @condition b"string\0".as_ptr(); !s.is_null());
    crate::SkASSERT!(@source b"include/core/SkString.h\0".as_ptr(),55; @condition b"substring\0".as_ptr(); !sub.is_null());
    SkStrFind(s, sub) != -1
}
// Source: include/core/SkString.h:58-64
pub unsafe fn SkStrContainsChar(s: *const u8, c: u8) -> bool {
    crate::SkASSERT!(@source b"include/core/SkString.h\0".as_ptr(),59; @condition b"string\0".as_ptr(); !s.is_null());
    let tmp = [c, 0];
    SkStrFind(s, tmp.as_ptr()) != -1
}
// Source: include/core/SkString.h:84-84
pub const kSkStrAppendU32_MaxSize: usize = 10;
// Source: include/core/SkString.h:86-86
pub const kSkStrAppendU64_MaxSize: usize = 20;
// Source: include/core/SkString.h:89-89
pub const kSkStrAppendS32_MaxSize: usize = kSkStrAppendU32_MaxSize + 1;
// Source: include/core/SkString.h:91-91
pub const kSkStrAppendS64_MaxSize: usize = kSkStrAppendU64_MaxSize + 1;
// Source: include/core/SkString.h:101-101
pub const kSkStrAppendScalar_MaxSize: usize = 15;
// Source: include/core/SkString.h:118-119
// Source: include/core/SkString.h:272-272
// sk_sp is represented by this intrusive record pointer; Drop and Clone below
// perform native ref/unref. No separate capacity or Rust string owner is used.
pub struct SkString {
    fRec: *mut Rec,
}
impl SkString {
    // Source: include/core/SkString.h:130-136
    pub fn isEmpty(&self) -> bool {
        unsafe { (*self.fRec).fLength == 0 }
    }
    pub fn size(&self) -> usize {
        unsafe { (*self.fRec).fLength as usize }
    }
    pub fn data(&self) -> *const u8 {
        unsafe { Rec::data(self.fRec) }
    }
    pub fn c_str(&self) -> *const u8 {
        unsafe { Rec::data(self.fRec) }
    }
    pub unsafe fn at(&self, n: usize) -> u8 {
        *self.c_str().add(n)
    }
    pub fn begin(&self) -> *const u8 {
        self.data()
    }
    pub fn end(&self) -> *const u8 {
        unsafe { self.data().add(self.size()) }
    }
    // Source: include/core/SkString.h:142-144
    pub unsafe fn startsWith(&self, s: *const u8) -> bool {
        SkStrStartsWith(self.c_str(), s)
    }
    // Source: include/core/SkString.h:145-147
    pub fn startsWithChar(&self, c: u8) -> bool {
        unsafe { SkStrStartsWithChar(self.c_str(), c) }
    }
    // Source: include/core/SkString.h:148-150
    pub unsafe fn endsWith(&self, s: *const u8) -> bool {
        SkStrEndsWith(self.c_str(), s)
    }
    // Source: include/core/SkString.h:151-153
    pub fn endsWithChar(&self, c: u8) -> bool {
        unsafe { SkStrEndsWithChar(self.c_str(), c) }
    }
    // Source: include/core/SkString.h:154-156
    pub unsafe fn contains(&self, s: *const u8) -> bool {
        SkStrContains(self.c_str(), s)
    }
    // Source: include/core/SkString.h:157-159
    pub fn containsChar(&self, c: u8) -> bool {
        unsafe { SkStrContainsChar(self.c_str(), c) }
    }
    // Source: include/core/SkString.h:160-162
    pub unsafe fn find(&self, s: *const u8) -> i32 {
        SkStrFind(self.c_str(), s)
    }
    // Source: include/core/SkString.h:163-165
    pub fn findLastOf(&self, c: u8) -> i32 {
        unsafe { SkStrFindLastOf(self.c_str(), c) }
    }
    // Source: include/core/SkString.h:181-183
    pub unsafe fn at_mut(&mut self, n: usize) -> &mut u8 {
        &mut *self.data_mut().add(n)
    }
    pub fn begin_mut(&mut self) -> *mut u8 {
        self.data_mut()
    }
    pub fn end_mut(&mut self) -> *mut u8 {
        let n = self.size();
        unsafe { self.data_mut().add(n) }
    }
    // Source: include/core/SkString.h:190-190
    pub fn set_string(&mut self, s: &Self) {
        self.assign(s);
    }
    // Source: include/core/SkString.h:193-193
    pub fn set_view(&mut self, s: &[u8]) {
        unsafe { self.set(s.as_ptr(), s.len()) }
    }
    // Source: include/core/SkString.h:197-198
    pub fn insert_string(&mut self, offset: usize, s: &Self) {
        unsafe { self.insert(offset, s.c_str(), s.size()) }
    }
    pub fn insert_view(&mut self, offset: usize, s: &[u8]) {
        unsafe { self.insert(offset, s.as_ptr(), s.len()) }
    }
    // Source: include/core/SkString.h:207-217
    pub unsafe fn append_cstr(&mut self, s: *const u8) {
        self.insert_cstr(usize::MAX, s)
    }
    pub unsafe fn append(&mut self, s: *const u8, n: usize) {
        self.insert(usize::MAX, s, n)
    }
    pub fn append_string(&mut self, s: &Self) {
        unsafe { self.insert(usize::MAX, s.c_str(), s.size()) }
    }
    pub fn append_view(&mut self, s: &[u8]) {
        unsafe { self.insert(usize::MAX, s.as_ptr(), s.len()) }
    }
    pub fn appendUnichar(&mut self, v: i32) {
        self.insertUnichar(usize::MAX, v)
    }
    pub fn appendS32(&mut self, v: i32) {
        self.insertS32(usize::MAX, v)
    }
    pub fn appendS64(&mut self, v: i64, d: i32) {
        self.insertS64(usize::MAX, v, d)
    }
    pub fn appendU32(&mut self, v: u32) {
        self.insertU32(usize::MAX, v)
    }
    pub fn appendU64(&mut self, v: u64, d: i32) {
        self.insertU64(usize::MAX, v, d)
    }
    pub fn appendHex(&mut self, v: u32, d: i32) {
        self.insertHex(usize::MAX, v, d)
    }
    pub fn appendScalar(&mut self, v: f32) {
        self.insertScalar(usize::MAX, v)
    }
    // Source: include/core/SkString.h:219-227
    pub unsafe fn prepend_cstr(&mut self, s: *const u8) {
        self.insert_cstr(0, s)
    }
    pub unsafe fn prepend(&mut self, s: *const u8, n: usize) {
        self.insert(0, s, n)
    }
    pub fn prepend_string(&mut self, s: &Self) {
        unsafe { self.insert(0, s.c_str(), s.size()) }
    }
    pub fn prepend_view(&mut self, s: &[u8]) {
        unsafe { self.insert(0, s.as_ptr(), s.len()) }
    }
    pub fn prependUnichar(&mut self, v: i32) {
        self.insertUnichar(0, v)
    }
    pub fn prependS32(&mut self, v: i32) {
        self.insertS32(0, v)
    }
    pub fn prependS64(&mut self, v: i32, d: i32) {
        self.insertS64(0, v as i64, d)
    }
    pub fn prependHex(&mut self, v: u32, d: i32) {
        self.insertHex(0, v, d)
    }
    pub fn prependScalar(&mut self, v: f32) {
        self.insertScalar(usize::MAX, v)
    }
    // Source: include/core/SkString.h:238-240
    pub fn add_assign_string(&mut self, s: &Self) -> &mut Self {
        self.append_string(s);
        self
    }
    pub unsafe fn add_assign_cstr(&mut self, s: *const u8) -> &mut Self {
        self.append_cstr(s);
        self
    }
    pub fn add_assign_char(&mut self, c: u8) -> &mut Self {
        unsafe { self.append(&c, 1) };
        self
    }
}
// Source: include/core/SkString.h:167-169
// Source: include/core/SkString.h:170-172
impl PartialEq for SkString {
    fn eq(&self, b: &Self) -> bool {
        self.equals(b)
    }
    fn ne(&self, b: &Self) -> bool {
        !self.equals(b)
    }
}
// Source: include/core/SkString.h:251-253
// Source: include/core/SkString.h:263-267
#[repr(C)]
struct Rec {
    fLength: u32,
    fRefCnt: AtomicI32,
    fBeginningOfData: [u8; 1],
}
impl Rec {
    // Source: include/core/SkString.h:253-253
    const fn new(len: u32, refs: i32) -> Self {
        Self {
            fLength: len,
            fRefCnt: AtomicI32::new(refs),
            fBeginningOfData: [0],
        }
    }
    // Source: include/core/SkString.h:255-256
    unsafe fn data(p: *mut Self) -> *mut u8 {
        core::ptr::addr_of_mut!((*p).fBeginningOfData).cast()
    }
}
// Source: include/core/SkString.h:291-291
pub fn SkStringPrintfEmpty() -> SkString {
    SkString::new()
}
// Source: include/core/SkString.h:293-295
pub fn swap(a: &mut SkString, b: &mut SkString) {
    a.swap(b)
}
// Source: src/core/SkString.cpp:26-27
const kBufferSize: usize = 1024;
// Source: src/core/SkString.cpp:29-32
struct StringBuffer {
    fText: *mut u8,
    fLength: i32,
}
// Actual compiler-provided native va_list. Clone performs va_copy and Drop
// performs va_end; no platform layout, tuple storage, or cursor is reconstructed.
pub use core::ffi::VaList;
unsafe extern "C" {
    // ManuallyDrop is the language's transparent ownership suppression for the
    // exact native VaList type. C consumes only this synchronous ABI forwarding
    // copy; the original Rust owner remains alive and executes the unique end.
    fn vsnprintf(out: *mut u8, n: usize, fmt: *const u8, args: ManuallyDrop<VaList<'_>>) -> c_int;
}
// Source: src/core/SkString.cpp:34-36,38-64
unsafe fn apply_format_string<const SIZE: usize>(
    format: *const u8,
    args: &VaList<'_>,
    stack: &mut [MaybeUninit<u8>; SIZE],
    heap: &mut SkString,
) -> StringBuffer {
    let args_copy: VaList<'_> = args.clone(); // native va_copy before consuming the first cursor
    let out_length = unsafe {
        vsnprintf(
            stack.as_mut_ptr().cast(),
            SIZE,
            format,
            ManuallyDrop::new(core::ptr::read(args)),
        )
    };
    if out_length < 0 {
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkString.cpp\0".as_ptr(), 46; b"SkString: vsnprintf reported error.\0".as_ptr());
        }
        drop(args_copy);
        return StringBuffer {
            fText: stack.as_mut_ptr().cast(),
            fLength: 0,
        };
    }
    if out_length < (SIZE as i32) {
        drop(args_copy);
        return StringBuffer {
            fText: stack.as_mut_ptr().cast(),
            fLength: out_length,
        };
    }
    unsafe { heap.set(core::ptr::null(), out_length as usize) };
    let dest = heap.data_mut();
    let check = unsafe {
        vsnprintf(
            dest,
            out_length as usize + 1,
            format,
            ManuallyDrop::new(core::ptr::read(&args_copy)),
        )
    };
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),61; @condition b"checkLength == outLength\0".as_ptr(); check == out_length);
    drop(args_copy);
    StringBuffer {
        fText: dest,
        fLength: out_length,
    }
}
// Source: include/core/SkString.h:36-36
// Source: src/core/SkString.cpp:68-75
pub unsafe fn SkStrEndsWith(s: *const u8, suffix: *const u8) -> bool {
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),69; @condition b"string\0".as_ptr(); !s.is_null());
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),70; @condition b"suffixStr\0".as_ptr(); !suffix.is_null());
    let n = strlen(s);
    let m = strlen(suffix);
    n >= m && strncmp(s.add(n - m), suffix, m) == 0
}
// Source: include/core/SkString.h:37-37
// Source: src/core/SkString.cpp:77-85
pub unsafe fn SkStrEndsWithChar(s: *const u8, c: u8) -> bool {
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),78; @condition b"string\0".as_ptr(); !s.is_null());
    let n = strlen(s);
    if n == 0 {
        false
    } else {
        c == *s.add(n - 1)
    }
}
// Source: include/core/SkString.h:39-39
// Source: src/core/SkString.cpp:87-98
pub unsafe fn SkStrStartsWithOneOf(s: *const u8, mut prefixes: *const u8) -> i32 {
    let mut index = 0;
    loop {
        let limit = strchr(prefixes, 0);
        if strncmp(s, prefixes, limit.offset_from(prefixes) as usize) == 0 {
            return index;
        }
        prefixes = limit.add(1);
        index += 1;
        if *prefixes == 0 {
            break;
        }
    }
    -1
}
// Source: include/core/SkString.h:85-85
// Source: src/core/SkString.cpp:100-118
pub unsafe fn SkStrAppendU32(mut string: *mut u8, mut dec: u32) -> *mut u8 {
    let start = string;
    let mut buffer = [MaybeUninit::<u8>::uninit(); kSkStrAppendU32_MaxSize];
    let base = buffer.as_mut_ptr().cast::<u8>();
    let mut p = base.add(buffer.len());
    loop {
        p = p.sub(1);
        *p = b'0' + (dec % 10) as u8;
        dec /= 10;
        if dec == 0 {
            break;
        }
    }
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),111; @condition b"p >= buffer\0".as_ptr(); p>=base);
    let n = base.add(buffer.len()).offset_from(p) as usize;
    core::ptr::copy_nonoverlapping(p, string, n);
    string = string.add(n);
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),116; @condition b"string - start <= kSkStrAppendU32_MaxSize\0".as_ptr(); string.offset_from(start)<=kSkStrAppendU32_MaxSize as isize);
    string
}
// Source: include/core/SkString.h:90-90
// Source: src/core/SkString.cpp:120-127
pub unsafe fn SkStrAppendS32(mut string: *mut u8, dec: i32) -> *mut u8 {
    let mut udec = dec as u32;
    if dec < 0 {
        *string = b'-';
        string = string.add(1);
        udec = (!udec).wrapping_add(1);
    }
    SkStrAppendU32(string, udec)
}
// Source: include/core/SkString.h:87-87
// Source: src/core/SkString.cpp:129-153
pub unsafe fn SkStrAppendU64(mut string: *mut u8, mut dec: u64, mut digits: i32) -> *mut u8 {
    let start = string;
    let mut buffer = [MaybeUninit::<u8>::uninit(); kSkStrAppendU64_MaxSize];
    let base = buffer.as_mut_ptr().cast::<u8>();
    let mut p = base.add(buffer.len());
    loop {
        p = p.sub(1);
        *p = b'0' + (dec % 10) as u8;
        dec /= 10;
        digits -= 1;
        if dec == 0 {
            break;
        }
    }
    while digits > 0 {
        p = p.sub(1);
        *p = b'0';
        digits -= 1;
    }
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),146; @condition b"p >= buffer\0".as_ptr(); p>=base);
    let n = base.add(buffer.len()).offset_from(p) as usize;
    core::ptr::copy_nonoverlapping(p, string, n);
    string = string.add(n);
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),151; @condition b"string - start <= kSkStrAppendU64_MaxSize\0".as_ptr(); string.offset_from(start)<=kSkStrAppendU64_MaxSize as isize);
    string
}
// Source: include/core/SkString.h:92-92
// Source: src/core/SkString.cpp:155-162
pub unsafe fn SkStrAppendS64(mut string: *mut u8, dec: i64, digits: i32) -> *mut u8 {
    let mut udec = dec as u64;
    if dec < 0 {
        *string = b'-';
        string = string.add(1);
        udec = (!udec).wrapping_add(1);
    }
    SkStrAppendU64(string, udec, digits)
}
// Source: include/core/SkString.h:110-110
// Source: src/core/SkString.cpp:164-189
pub unsafe fn SkStrAppendScalar(string: *mut u8, value: f32) -> *mut u8 {
    if value.is_nan() {
        core::ptr::copy_nonoverlapping(b"nan\0".as_ptr(), string, 4);
        return string.add(3);
    }
    if !value.is_finite() {
        if value > 0. {
            core::ptr::copy_nonoverlapping(b"inf\0".as_ptr(), string, 4);
            return string.add(3);
        } else {
            core::ptr::copy_nonoverlapping(b"-inf\0".as_ptr(), string, 5);
            return string.add(4);
        }
    }
    let mut buffer = [MaybeUninit::<u8>::uninit(); kSkStrAppendScalar_MaxSize + 1];
    let n = snprintf(
        buffer.as_mut_ptr().cast(),
        buffer.len(),
        b"%.8g\0".as_ptr(),
        value as f64,
    );
    core::ptr::copy_nonoverlapping(buffer.as_ptr().cast(), string, n as usize);
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),187; @condition b"len <= kSkStrAppendScalar_MaxSize\0".as_ptr(); n<=kSkStrAppendScalar_MaxSize as i32);
    string.add(n as usize)
}
// Source: include/core/SkString.h:284-284
// Source: src/core/SkString.cpp:193-193
static gEmptyRec: Rec = Rec::new(0, 0);
// Source: src/core/SkString.cpp:195-195
fn size_of_rec() -> usize {
    core::mem::offset_of!(Rec, fBeginningOfData)
}
// Source: src/core/SkString.cpp:197-204
fn trim_size_t_to_u32(mut n: usize) -> u32 {
    if core::mem::size_of::<usize>() > 4 {
        if n > u32::MAX as usize {
            n = u32::MAX as usize;
        }
    }
    n as u32
}
// Source: src/core/SkString.cpp:206-214
fn check_add32(base: usize, mut extra: usize) -> usize {
    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),207; @condition b"base <= UINT32_MAX\0".as_ptr(); base<=u32::MAX as usize);
    if core::mem::size_of::<usize>() > 4 {
        if base.wrapping_add(extra) > u32::MAX as usize {
            extra = u32::MAX as usize - base;
        }
    }
    extra
}
impl Rec {
    // Source: include/core/SkString.h:254-254
    // Source: src/core/SkString.cpp:216-238
    unsafe fn make(text: *const u8, len: usize) -> *mut Self {
        if len == 0 {
            return core::ptr::addr_of!(gEmptyRec).cast_mut();
        }
        let mut safe = crate::src::core::SkSafeMath::SkSafeMath::new();
        let string_len = safe.castTo::<u32, usize>(len);
        let allocation_size = safe.add(len, size_of_rec() + 1);
        let allocation_size = safe.alignUp(allocation_size, 4);
        crate::SkASSERT_RELEASE!(@source b"src/core/SkString.cpp\0".as_ptr(),229; @condition b"safe.ok()\0".as_ptr(); safe.ok());
        // malloc/free provide the selected platform's unsized allocation/deletion
        // primitive. The native 4-byte rounded allocation formula is unchanged.
        let storage = malloc(allocation_size).cast::<Rec>();
        if storage.is_null() {
            std::alloc::handle_alloc_error(std::alloc::Layout::from_size_align_unchecked(
                allocation_size,
                core::mem::align_of::<Rec>(),
            ));
        }
        core::ptr::write(storage, Rec::new(string_len, 1));
        if !text.is_null() {
            core::ptr::copy_nonoverlapping(text, Self::data(storage), len);
        }
        *Self::data(storage).add(len) = 0;
        storage
    }
    // Source: include/core/SkString.h:257-257
    // Source: src/core/SkString.cpp:240-245
    unsafe fn ref_rec(p: *mut Self) {
        if core::ptr::eq(p, core::ptr::addr_of!(gEmptyRec)) {
            return;
        }
        crate::SkAssertResult!(@source b"src/core/SkString.cpp\0".as_ptr(),244; @condition b"this->fRefCnt.fetch_add(+1, std::memory_order_relaxed)\0".as_ptr(); (*p).fRefCnt.fetch_add(1,Ordering::Relaxed)!=0);
    }
    // Source: include/core/SkString.h:258-258
    // Source: include/core/SkString.h:270-270
    // Source: src/core/SkString.cpp:247-256
    unsafe fn unref(p: *mut Self) {
        if core::ptr::eq(p, core::ptr::addr_of!(gEmptyRec)) {
            return;
        }
        let old = (*p).fRefCnt.fetch_add(-1, Ordering::AcqRel);
        crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),252; @condition b"oldRefCnt\0".as_ptr(); old!=0);
        if old == 1 {
            core::ptr::drop_in_place(p);
            free(p.cast());
        }
    }
    // Source: include/core/SkString.h:259-259
    // Source: src/core/SkString.cpp:258-260
    unsafe fn unique(p: *mut Self) -> bool {
        (*p).fRefCnt.load(Ordering::Acquire) == 1
    }
    // Source: include/core/SkString.h:261-261
    // Source: src/core/SkString.cpp:263-265
    #[cfg(sk_debug)]
    unsafe fn getRefCnt(p: *mut Self) -> i32 {
        (*p).fRefCnt.load(Ordering::Relaxed)
    }
}
impl SkString {
    // Source: include/core/SkString.h:278-278
    // Source: src/core/SkString.cpp:267-279
    #[cfg(sk_debug)]
    fn validate(&self) -> &Self {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),269; @condition b"0 == gEmptyRec.fLength\0".as_ptr(); 0 == gEmptyRec.fLength);
            crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),270; @condition b"0 == gEmptyRec.getRefCnt()\0".as_ptr(); 0 == Rec::getRefCnt(core::ptr::addr_of!(gEmptyRec).cast_mut()));
            crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),271; @condition b"0 == gEmptyRec.data()[0]\0".as_ptr(); 0 == gEmptyRec.fBeginningOfData[0]);
            if !core::ptr::eq(self.fRec, core::ptr::addr_of!(gEmptyRec)) {
                crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),274; @condition b"fRec->fLength > 0\0".as_ptr(); (*self.fRec).fLength>0);
                crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),275; @condition b"fRec->getRefCnt() > 0\0".as_ptr(); Rec::getRefCnt(self.fRec)>0);
                crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),276; @condition b"0 == fRec->data()[fRec->fLength]\0".as_ptr(); 0 == *Rec::data(self.fRec).add((*self.fRec).fLength as usize));
            }
        }
        self
    }
    // Source: include/core/SkString.h:277-277
    // Source: src/core/SkString.cpp:281-284
    #[cfg(sk_debug)]
    fn validate_mut(&mut self) -> &mut Self {
        self.validate();
        self
    }
    // Source: include/core/SkString.h:280-281
    #[cfg(not(sk_debug))]
    fn validate_mut(&mut self) -> &mut Self {
        self
    }
    #[cfg(not(sk_debug))]
    fn validate(&self) -> &Self {
        self
    }
    // Source: include/core/SkString.h:120-120
    // Source: src/core/SkString.cpp:289-290
    pub fn new() -> Self {
        Self {
            fRec: core::ptr::addr_of!(gEmptyRec).cast_mut(),
        }
    }
    // Source: include/core/SkString.h:121-121
    // Source: src/core/SkString.cpp:292-294
    pub fn with_len(len: usize) -> Self {
        Self {
            fRec: unsafe { Rec::make(core::ptr::null(), len) },
        }
    }
    // Source: include/core/SkString.h:122-122
    // Source: src/core/SkString.cpp:296-300
    pub unsafe fn from_cstr(text: *const u8) -> Self {
        let len = if text.is_null() { 0 } else { strlen(text) };
        Self {
            fRec: Rec::make(text, len),
        }
    }
    // Source: include/core/SkString.h:123-123
    // Source: src/core/SkString.cpp:302-304
    pub unsafe fn from_raw(text: *const u8, len: usize) -> Self {
        Self {
            fRec: Rec::make(text, len),
        }
    }
}
// Source: include/core/SkString.h:124-124
// Source: src/core/SkString.cpp:306-306
impl Clone for SkString {
    fn clone(&self) -> Self {
        let p = self.validate().fRec;
        unsafe { Rec::ref_rec(p) };
        Self { fRec: p }
    }
}
impl SkString {
    // Source: include/core/SkString.h:125-125
    // Source: src/core/SkString.cpp:308-310
    pub fn move_from(src: &mut Self) -> Self {
        let p = src.validate().fRec;
        src.fRec = core::ptr::addr_of!(gEmptyRec).cast_mut();
        Self { fRec: p }
    }
    // Source: include/core/SkString.h:126-126
    // Source: src/core/SkString.cpp:312-314
    pub fn from_std_string(src: &std::string::String) -> Self {
        unsafe { Self::from_raw(src.as_ptr(), src.len()) }
    }
    // Source: include/core/SkString.h:127-127
    // Source: src/core/SkString.cpp:316-318
    pub fn from_view(src: &[u8]) -> Self {
        unsafe { Self::from_raw(src.as_ptr(), src.len()) }
    }
}
// Source: include/core/SkString.h:128-128
// Source: src/core/SkString.cpp:320-322
impl Drop for SkString {
    fn drop(&mut self) {
        self.validate();
        unsafe { Rec::unref(self.fRec) }
    }
}
impl SkString {
    // Source: include/core/SkString.h:138-138
    // Source: src/core/SkString.cpp:324-326
    pub fn equals(&self, src: &Self) -> bool {
        self.fRec == src.fRec || unsafe { self.equals_raw(src.c_str(), src.size()) }
    }
    // Source: include/core/SkString.h:139-139
    // Source: src/core/SkString.cpp:328-330
    pub unsafe fn equals_cstr(&self, text: *const u8) -> bool {
        self.equals_raw(text, if text.is_null() { 0 } else { strlen(text) })
    }
    // Source: include/core/SkString.h:140-140
    // Source: src/core/SkString.cpp:332-336
    pub unsafe fn equals_raw(&self, text: *const u8, len: usize) -> bool {
        crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),333; @condition b"len == 0 || text != nullptr\0".as_ptr(); len==0||!text.is_null());
        (*self.fRec).fLength as usize == len
            && (len == 0 || memcmp(Rec::data(self.fRec).cast(), text.cast(), len) == 0)
    }
    // Source: include/core/SkString.h:176-176
    // Source: src/core/SkString.cpp:338-342
    pub fn assign(&mut self, src: &Self) -> &mut Self {
        self.validate();
        if !core::ptr::eq(self, src) {
            unsafe {
                Rec::ref_rec(src.fRec);
                Rec::unref(self.fRec);
            }
            self.fRec = src.fRec;
        }
        self
    }
    // Source: include/core/SkString.h:177-177
    // Source: src/core/SkString.cpp:344-351
    pub fn move_assign(&mut self, src: &mut Self) -> &mut Self {
        self.validate();
        if self.fRec != src.fRec {
            self.swap(src);
        }
        self
    }
    // Source: include/core/SkString.h:178-178
    // Source: src/core/SkString.cpp:353-356
    pub unsafe fn assign_cstr(&mut self, text: *const u8) -> &mut Self {
        self.validate();
        let mut temp = Self::from_cstr(text);
        self.move_assign(&mut temp)
    }
    // Source: include/core/SkString.h:185-185
    // Source: src/core/SkString.cpp:358-361
    pub fn reset(&mut self) {
        self.validate();
        let old = self.fRec;
        self.fRec = core::ptr::addr_of!(gEmptyRec).cast_mut();
        unsafe { Rec::unref(old) }
    }
    // Source: include/core/SkString.h:180-180
    // Source: src/core/SkString.cpp:363-372
    pub fn data_mut(&mut self) -> *mut u8 {
        self.validate();
        unsafe {
            if (*self.fRec).fLength != 0 {
                if !Rec::unique(self.fRec) {
                    let p = Rec::make(Rec::data(self.fRec), (*self.fRec).fLength as usize);
                    Rec::unref(self.fRec);
                    self.fRec = p;
                }
            }
            Rec::data(self.fRec)
        }
    }
    // Source: include/core/SkString.h:189-189
    // Source: src/core/SkString.cpp:374-391
    pub fn resize(&mut self, len: usize) {
        let len = trim_size_t_to_u32(len) as usize;
        unsafe {
            if len == 0 {
                self.reset();
            } else if Rec::unique(self.fRec) && (len >> 2) <= ((*self.fRec).fLength as usize >> 2) {
                let p = self.data_mut();
                *p.add(len) = 0;
                (*self.fRec).fLength = len as u32;
            } else {
                let mut new_string = Self::with_len(len);
                let dest = new_string.data_mut();
                let copy_len = (len as u32).min(self.size() as u32) as i32;
                core::ptr::copy_nonoverlapping(self.c_str(), dest, copy_len as usize);
                *dest.add(copy_len as usize) = 0;
                self.swap(&mut new_string);
            }
        }
    }
    // Source: include/core/SkString.h:191-191
    // Source: src/core/SkString.cpp:393-395
    pub unsafe fn set_cstr(&mut self, text: *const u8) {
        self.set(text, if text.is_null() { 0 } else { strlen(text) })
    }
    // Source: include/core/SkString.h:192-192
    // Source: src/core/SkString.cpp:397-413
    pub unsafe fn set(&mut self, text: *const u8, len: usize) {
        let len = trim_size_t_to_u32(len) as usize;
        if len == 0 {
            self.reset();
        } else if Rec::unique(self.fRec) && (len >> 2) <= ((*self.fRec).fLength as usize >> 2) {
            let p = self.data_mut();
            if !text.is_null() {
                core::ptr::copy_nonoverlapping(text, p, len);
            }
            *p.add(len) = 0;
            (*self.fRec).fLength = len as u32;
        } else {
            let mut tmp = Self::from_raw(text, len);
            self.swap(&mut tmp);
        }
    }
    // Source: include/core/SkString.h:195-195
    // Source: src/core/SkString.cpp:415-417
    pub unsafe fn insert_cstr(&mut self, offset: usize, text: *const u8) {
        self.insert(offset, text, if text.is_null() { 0 } else { strlen(text) })
    }
    // Source: include/core/SkString.h:196-196
    // Source: src/core/SkString.cpp:419-471
    pub unsafe fn insert(&mut self, mut offset: usize, text: *const u8, mut len: usize) {
        if len != 0 {
            let length = (*self.fRec).fLength as usize;
            if offset > length {
                offset = length;
            }
            len = check_add32(length, len);
            if len == 0 {
                return;
            }
            if Rec::unique(self.fRec) && (length >> 2) == ((length + len) >> 2) {
                let dst = self.data_mut();
                if offset < length {
                    core::ptr::copy(dst.add(offset), dst.add(offset + len), length - offset);
                }
                core::ptr::copy_nonoverlapping(text, dst.add(offset), len);
                *dst.add(length + len) = 0;
                (*self.fRec).fLength = (length + len) as u32;
            } else {
                let mut tmp = Self::with_len((*self.fRec).fLength as usize + len);
                let dst = tmp.data_mut();
                if offset > 0 {
                    core::ptr::copy_nonoverlapping(Rec::data(self.fRec), dst, offset);
                }
                core::ptr::copy_nonoverlapping(text, dst.add(offset), len);
                if offset < (*self.fRec).fLength as usize {
                    core::ptr::copy_nonoverlapping(
                        Rec::data(self.fRec).add(offset),
                        dst.add(offset + len),
                        (*self.fRec).fLength as usize - offset,
                    );
                }
                self.swap(&mut tmp);
            }
        }
    }
    // Source: include/core/SkString.h:199-199
    // Source: src/core/SkString.cpp:473-480
    pub fn insertUnichar(&mut self, offset: usize, uni: i32) {
        let mut buffer =
            [MaybeUninit::<u8>::uninit(); crate::src::core::SkUTF::kMaxBytesInUTF8Sequence];
        unsafe {
            let len = crate::src::core::SkUTF::ToUTF8(uni, buffer.as_mut_ptr().cast());
            if len != 0 {
                self.insert(offset, buffer.as_ptr().cast(), len);
            }
        }
    }
    // Source: include/core/SkString.h:200-200
    // Source: src/core/SkString.cpp:482-486
    pub fn insertS32(&mut self, offset: usize, dec: i32) {
        let mut buffer = [MaybeUninit::<u8>::uninit(); kSkStrAppendS32_MaxSize];
        unsafe {
            let p = buffer.as_mut_ptr().cast();
            let stop = SkStrAppendS32(p, dec);
            self.insert(offset, p, stop.offset_from(p) as usize);
        }
    }
    // Source: include/core/SkString.h:201-201
    // Source: src/core/SkString.cpp:488-492
    pub fn insertS64(&mut self, offset: usize, dec: i64, digits: i32) {
        let mut buffer = [MaybeUninit::<u8>::uninit(); kSkStrAppendS64_MaxSize];
        unsafe {
            let p = buffer.as_mut_ptr().cast();
            let stop = SkStrAppendS64(p, dec, digits);
            self.insert(offset, p, stop.offset_from(p) as usize);
        }
    }
    // Source: include/core/SkString.h:202-202
    // Source: src/core/SkString.cpp:494-498
    pub fn insertU32(&mut self, offset: usize, dec: u32) {
        let mut buffer = [MaybeUninit::<u8>::uninit(); kSkStrAppendU32_MaxSize];
        unsafe {
            let p = buffer.as_mut_ptr().cast();
            let stop = SkStrAppendU32(p, dec);
            self.insert(offset, p, stop.offset_from(p) as usize);
        }
    }
    // Source: include/core/SkString.h:203-203
    // Source: src/core/SkString.cpp:500-504
    pub fn insertU64(&mut self, offset: usize, dec: u64, digits: i32) {
        let mut buffer = [MaybeUninit::<u8>::uninit(); kSkStrAppendU64_MaxSize];
        unsafe {
            let p = buffer.as_mut_ptr().cast();
            let stop = SkStrAppendU64(p, dec, digits);
            self.insert(offset, p, stop.offset_from(p) as usize);
        }
    }
    // Source: include/core/SkString.h:204-204
    // Source: src/core/SkString.cpp:506-524
    pub fn insertHex(&mut self, offset: usize, mut hex: u32, mut digits: i32) {
        digits = digits.clamp(0, 8);
        let mut buffer = [MaybeUninit::<u8>::uninit(); 8];
        unsafe {
            let base = buffer.as_mut_ptr().cast::<u8>();
            let mut p = base.add(8);
            loop {
                p = p.sub(1);
                *p = b"0123456789ABCDEF"[(hex & 15) as usize];
                hex >>= 4;
                digits -= 1;
                if hex == 0 {
                    break;
                }
            }
            loop {
                digits -= 1;
                if digits < 0 {
                    break;
                }
                p = p.sub(1);
                *p = b'0';
            }
            crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),522; @condition b"p >= buffer\0".as_ptr(); p>=base);
            self.insert(offset, p, base.add(8).offset_from(p) as usize);
        }
    }
    // Source: include/core/SkString.h:205-205
    // Source: src/core/SkString.cpp:526-530
    pub fn insertScalar(&mut self, offset: usize, value: f32) {
        let mut buffer = [MaybeUninit::<u8>::uninit(); kSkStrAppendScalar_MaxSize];
        unsafe {
            let p = buffer.as_mut_ptr().cast();
            let stop = SkStrAppendScalar(p, value);
            self.insert(offset, p, stop.offset_from(p) as usize);
        }
    }
}
impl SkString {
    // Source: include/core/SkString.h:229-229
    // Source: src/core/SkString.cpp:534-539
    /// Safety: format is NUL-terminated and the actual C arguments match it,
    /// including native default promotions and live backing for every pointer.
    pub unsafe extern "C" fn printf(&mut self, format: *const u8, args: ...) {
        unsafe {
            self.printVAList(format, args);
        }
    }
    // Source: include/core/SkString.h:230-230
    // Source: src/core/SkString.cpp:541-548
    pub unsafe fn printVAList(&mut self, format: *const u8, args: VaList<'_>) {
        let mut stack = [MaybeUninit::<u8>::uninit(); kBufferSize];
        let result = unsafe { apply_format_string(format, &args, &mut stack, self) };
        if result.fText == stack.as_mut_ptr().cast() {
            unsafe {
                self.set(result.fText, result.fLength as usize);
            }
        }
        // args owns the original va_start lifetime until all string copying ends.
    }
    // Source: include/core/SkString.h:231-231
    // Source: src/core/SkString.cpp:550-555
    pub unsafe extern "C" fn appendf(&mut self, format: *const u8, args: ...) {
        unsafe {
            self.appendVAList(format, args);
        }
    }
    // Source: include/core/SkString.h:232-232
    // Source: src/core/SkString.cpp:557-568
    pub unsafe fn appendVAList(&mut self, format: *const u8, args: VaList<'_>) {
        if self.isEmpty() {
            unsafe {
                self.printVAList(format, args);
            }
            return;
        }
        let mut overflow = Self::new();
        let mut stack = [MaybeUninit::<u8>::uninit(); kBufferSize];
        let result = unsafe { apply_format_string(format, &args, &mut stack, &mut overflow) };
        unsafe {
            self.append(result.fText, result.fLength as usize);
        }
    }
    // Source: include/core/SkString.h:233-233
    // Source: src/core/SkString.cpp:570-575
    pub unsafe extern "C" fn prependf(&mut self, format: *const u8, args: ...) {
        unsafe {
            self.prependVAList(format, args);
        }
    }
    // Source: include/core/SkString.h:234-234
    // Source: src/core/SkString.cpp:577-588
    pub unsafe fn prependVAList(&mut self, format: *const u8, args: VaList<'_>) {
        if self.isEmpty() {
            unsafe {
                self.printVAList(format, args);
            }
            return;
        }
        let mut overflow = Self::new();
        let mut stack = [MaybeUninit::<u8>::uninit(); kBufferSize];
        let result = unsafe { apply_format_string(format, &args, &mut stack, &mut overflow) };
        unsafe {
            self.prepend(result.fText, result.fLength as usize);
        }
    }
}
impl SkString {
    // Source: include/core/SkString.h:236-236
    // Source: src/core/SkString.cpp:592-617
    pub fn remove(&mut self, offset: usize, mut length: usize) {
        let size = self.size();
        if offset < size {
            if length > size - offset {
                length = size - offset;
            }
            crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),599; @condition b"length <= size\0".as_ptr(); length<=size);
            crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),600; @condition b"offset <= size - length\0".as_ptr(); offset<=size-length);
            if length > 0 {
                let mut tmp = Self::with_len(size - length);
                let dst = tmp.data_mut();
                let src = self.c_str();
                unsafe {
                    if offset != 0 {
                        core::ptr::copy_nonoverlapping(src, dst, offset);
                    }
                    let tail = size - (offset + length);
                    if tail != 0 {
                        core::ptr::copy_nonoverlapping(
                            src.add(offset + length),
                            dst.add(offset),
                            tail,
                        );
                    }
                    crate::SkASSERT!(@source b"src/core/SkString.cpp\0".as_ptr(),613; @condition b"dst[tmp.size()] == 0\0".as_ptr(); *dst.add(tmp.size()) == 0);
                }
                self.swap(&mut tmp);
            }
        }
    }
    // Source: include/core/SkString.h:246-246
    // Source: src/core/SkString.cpp:619-625
    pub fn swap(&mut self, other: &mut Self) {
        self.validate();
        other.validate();
        core::mem::swap(&mut self.fRec, &mut other.fRec);
    }
}
// Source: include/core/SkString.h:288-288
// Source: src/core/SkString.cpp:629-636
/// Safety: native printf argument types/promotions and pointed-to leases match format.
pub unsafe extern "C" fn SkStringPrintf(format: *const u8, args: ...) -> SkString {
    let mut output = SkString::new();
    unsafe {
        output.printVAList(format, args);
    }
    output
}

// Rust moves preserve the trivially relocatable fRec pointer without callbacks.
// Source: include/core/SkString.h:248-248
// Source: include/core/SkString.h:274-274
// No additional owner state is required by this language substitution.
// SkString const operations only touch immutable data and the atomic refcount;
// mutable operations detach through the native unique() branch.
unsafe impl Send for SkString {}
unsafe impl Sync for SkString {}
impl Default for SkString {
    fn default() -> Self {
        Self::new()
    }
}
