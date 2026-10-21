// Source: base/debug/stack_trace.h; base/debug/stack_trace.cc
use crate::base::strings::native_ostream::{NativeOStream, NativeStdString, StdStringStorage};
use core::pin::Pin;
use core::{
    ffi::{c_void, CStr},
    mem::MaybeUninit,
};
// Only the assigned native POSIX producer is selected here; other platform
// implementations retain their exact independent source dependency.
#[cfg(all(unix, not(target_os = "fuchsia")))]
use super::stack_trace_posix as platform;
// Source: base/debug/stack_trace.h:78-87
#[cfg(target_os = "android")]
pub const kMaxTraces: usize = 62;
#[cfg(not(target_os = "android"))]
pub const kMaxTraces: usize = 250;
// The suffix of trace_ is genuinely uninitialized, as in std::array<const void*>.
// Only count_ initialized entries can be exposed; Copy/Clone never observes them.
#[derive(Clone, Copy)]
pub struct StackTrace {
    trace_: [MaybeUninit<*const c_void>; kMaxTraces],
    count_: usize,
}
impl StackTrace {
    // Source: base/debug/stack_trace.cc:276
    pub unsafe fn new() -> Self {
        unsafe { Self::with_count(kMaxTraces) }
    }
    // Source: base/debug/stack_trace.cc:278
    pub unsafe fn with_count(count: usize) -> Self {
        let mut trace = Self {
            trace_: [MaybeUninit::uninit(); kMaxTraces],
            count_: 0,
        };
        if !unsafe { Self::ShouldSuppressOutput() } {
            trace.count_ = unsafe {
                platform::CollectStackTrace(trace.trace_.as_mut_ptr().cast(), count.min(kMaxTraces))
            };
        }
        trace
    }
    // Source: base/debug/stack_trace.cc:284
    pub fn from_addresses(addresses: &[*const c_void]) -> Self {
        let count_ = addresses.len().min(kMaxTraces);
        let mut result = Self {
            trace_: [MaybeUninit::uninit(); kMaxTraces],
            count_,
        };
        for i in 0..count_ {
            result.trace_[i].write(addresses[i]);
        }
        result
    }
    // Source: base/debug/stack_trace.h:124-126
    pub fn addresses(&self) -> &[*const c_void] {
        unsafe { core::slice::from_raw_parts(self.trace_.as_ptr().cast(), self.count_) }
    }
    // Source: base/debug/stack_trace.cc:292
    pub fn WillSymbolizeToStreamForTesting() -> bool {
        #[cfg(any(
            not(HAS_SYMBOLS),
            __UCLIBC__,
            target_os = "aix",
            all(OFFICIAL_BUILD, any(unix, target_os = "fuchsia")),
            target_os = "fuchsia",
            target_os = "android",
            PRINT_UNSYMBOLIZED_STACK_TRACES
        ))]
        {
            return false;
        }
        #[cfg(not(any(
            not(HAS_SYMBOLS),
            __UCLIBC__,
            target_os = "aix",
            all(OFFICIAL_BUILD, any(unix, target_os = "fuchsia")),
            target_os = "fuchsia",
            target_os = "android",
            PRINT_UNSYMBOLIZED_STACK_TRACES
        )))]
        {
            true
        }
    }
    // Source: base/debug/stack_trace.cc:338
    pub unsafe fn Print<S: StdStringStorage>(&self) {
        unsafe {
            self.PrintWithPrefix::<S>(c"");
        }
    }
    // Source: base/debug/stack_trace.cc:342
    pub unsafe fn PrintWithPrefix<S: StdStringStorage>(&self, prefix: &CStr) {
        if self.count_ == 0 || unsafe { Self::ShouldSuppressOutput() } {
            let message = unsafe { g_stack_trace_message };
            if !message.is_null() {
                unsafe {
                    platform::PrintMessageWithPrefix(prefix, NativeMessageView(message));
                }
            }
            return;
        }
        unsafe {
            platform::PrintWithPrefixImpl::<S>(self.addresses(), prefix);
        }
    }
    #[cfg(not(any(__UCLIBC__, target_os = "aix")))]
    // Source: base/debug/stack_trace.cc:352
    pub unsafe fn OutputToStream<S: StdStringStorage>(&self, os: &mut NativeOStream<'_>) {
        unsafe {
            self.OutputToStreamWithPrefix::<S>(os, c"");
        }
    }
    #[cfg(not(any(__UCLIBC__, target_os = "aix")))]
    // Source: base/debug/stack_trace.cc:356
    pub unsafe fn OutputToStreamWithPrefix<S: StdStringStorage>(
        &self,
        os: &mut NativeOStream<'_>,
        prefix: &CStr,
    ) {
        if self.count_ == 0 || unsafe { Self::ShouldSuppressOutput() } {
            let message = unsafe { g_stack_trace_message };
            if !message.is_null() {
                unsafe {
                    os.insert_cstr(prefix.as_ptr().cast());
                    InsertNativeMessage(os, message);
                }
            }
            return;
        }
        unsafe {
            platform::OutputToStreamWithPrefixImpl::<S>(self.addresses(), os, prefix);
        }
    }
    // Source: base/debug/stack_trace.cc:387
    pub unsafe fn ShouldSuppressOutput() -> bool {
        let mode = unsafe { g_override_suppression };
        if mode != Mode::kUnset {
            mode == Mode::kSuppressOutput
        } else {
            !unsafe { g_stack_trace_message }.is_null()
        }
    }
}
// Native std::string heap object; only the genuine standard-library primitive
// bridge can construct/move/delete or view it. No byte-vector stream substitute.
static mut g_stack_trace_message: *mut c_void = core::ptr::null_mut();
// Source: base/debug/stack_trace.h:191-195
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    kUnset,
    kForceOutput,
    kSuppressOutput,
}
static mut g_override_suppression: Mode = Mode::kUnset;
pub struct OverrideStackTraceOutputForTesting {
    _pin: core::marker::PhantomPinned,
    _not_copy: core::marker::PhantomData<std::rc::Rc<()>>,
}
// Actual ctor/dtor require the selected CHECK_NE/EQ enum ostream protocol;
// these bodies are kept pending, never replaced by Rust panic or trap-only checks.
pub unsafe fn insert_stack_trace<S: StdStringStorage>(os: &mut NativeOStream<'_>, s: &StackTrace) {
    #[cfg(not(any(__UCLIBC__, target_os = "aix")))]
    unsafe {
        s.OutputToStream::<S>(os);
    }
    #[cfg(any(__UCLIBC__, target_os = "aix"))]
    unsafe {
        os.insert_cstr(
            c"StackTrace::OutputToStream not implemented."
                .as_ptr()
                .cast(),
        );
    }
}
unsafe extern "C" {
    fn rechrom_debug_string_cstr(message: *const c_void) -> *const core::ffi::c_char;
    fn rechrom_ostream_string(stream: *mut c_void, message: *const c_void);
    fn rechrom_stack_message_delete(message: *mut c_void);
    fn rechrom_stack_message_move_new(message: *mut c_void) -> *mut c_void;
}
/// The actual global std::string must remain live and unmodified for this borrow;
/// native suppression configuration has the original external synchronization.
unsafe fn NativeMessageView<'a>(message: *const c_void) -> &'a CStr {
    unsafe { CStr::from_ptr(rechrom_debug_string_cstr(message)) }
}
unsafe fn InsertNativeMessage(os: &mut NativeOStream<'_>, message: *const c_void) {
    unsafe {
        rechrom_ostream_string(os.as_native_mut_ptr(), message);
    }
}

#[cfg(all(
    CAN_UNWIND_WITH_FRAME_POINTERS,
    target_arch = "arm",
    native_gcc,
    not(native_clang)
))]
const kStackFrameAdjustment: usize = core::mem::size_of::<usize>();
#[cfg(all(
    CAN_UNWIND_WITH_FRAME_POINTERS,
    not(all(target_arch = "arm", native_gcc, not(native_clang)))
))]
const kStackFrameAdjustment: usize = 0;
#[cfg(CAN_UNWIND_WITH_FRAME_POINTERS)]
static mut g_stack_scan_max_fp_to_stack_end_gap_bytes: Option<usize> = None;
#[cfg(CAN_UNWIND_WITH_FRAME_POINTERS)]
// Source: base/debug/stack_trace.cc:70
fn StripPointerAuthenticationBits(mut ptr: usize) -> usize {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("xpaclri",inout("x30")ptr,options(nostack,nomem));
    }
    ptr
}
#[cfg(MEMORY_SANITIZER)]
unsafe extern "C" {
    fn __msan_unpoison(p: *const c_void, size: usize);
}
#[cfg(CAN_UNWIND_WITH_FRAME_POINTERS)]
// Source: base/debug/stack_trace.cc:87
unsafe fn GetNextStackFrame(fp: usize) -> usize {
    let fp_addr = fp as *const usize;
    #[cfg(MEMORY_SANITIZER)]
    unsafe {
        __msan_unpoison(fp_addr.cast(), core::mem::size_of::<usize>());
    }
    unsafe { (*fp_addr).wrapping_sub(kStackFrameAdjustment) }
}
#[cfg(CAN_UNWIND_WITH_FRAME_POINTERS)]
// Source: base/debug/stack_trace.cc:97
unsafe fn GetStackFramePC(fp: usize) -> usize {
    let fp_addr = fp as *const usize;
    #[cfg(MEMORY_SANITIZER)]
    unsafe {
        __msan_unpoison(fp_addr.add(1).cast(), core::mem::size_of::<usize>());
    }
    StripPointerAuthenticationBits(unsafe { *fp_addr.add(1) })
}
#[cfg(CAN_UNWIND_WITH_FRAME_POINTERS)]
// Source: base/debug/stack_trace.cc:107
unsafe fn IsStackFrameValid(fp: usize, prev_fp: usize, stack_end: usize) -> bool {
    if fp <= prev_fp {
        return false;
    }
    if fp - prev_fp > 100000 {
        return false;
    }
    if fp & (core::mem::size_of::<usize>() - 1) != 0 {
        return false;
    }
    if stack_end != 0 {
        if fp > stack_end.wrapping_sub(2 * core::mem::size_of::<usize>()) {
            return false;
        }
        if unsafe { GetStackFramePC(fp) } < 32768 {
            return false;
        }
    }
    true
}
#[cfg(CAN_UNWIND_WITH_FRAME_POINTERS)]
// Source: base/debug/stack_trace.cc:156
unsafe fn ScanStackForNextFrame(mut fp: usize, stack_end: usize) -> usize {
    const kMaxStackScanArea: usize = 8192;
    if stack_end == 0 {
        return 0;
    }
    if let Some(gap) = unsafe { g_stack_scan_max_fp_to_stack_end_gap_bytes } {
        if stack_end < fp || stack_end - fp > gap {
            return 0;
        }
    }
    fp = fp.wrapping_add(core::mem::size_of::<usize>());
    // Original ClampedNumeric uintptr_t addition/Min/subtraction, same saturation.
    let last_fp_to_scan = fp
        .saturating_add(kMaxStackScanArea)
        .min(stack_end)
        .saturating_sub(core::mem::size_of::<usize>());
    while fp <= last_fp_to_scan {
        let next_fp = unsafe { GetNextStackFrame(fp) };
        if unsafe { IsStackFrameValid(next_fp, fp, stack_end) } {
            let next2_fp = unsafe { GetNextStackFrame(next_fp) };
            if unsafe { IsStackFrameValid(next2_fp, next_fp, stack_end) } {
                return fp;
            }
        }
        fp = fp.wrapping_add(core::mem::size_of::<usize>());
    }
    0
}
#[cfg(CAN_UNWIND_WITH_FRAME_POINTERS)]
// Source: base/debug/stack_trace.cc:202
unsafe fn LinkStackFrames(fpp: *mut c_void, parent_fp: *mut c_void) -> *mut c_void {
    let fp = (fpp as usize).wrapping_sub(kStackFrameAdjustment) as *mut *mut c_void;
    let prev = unsafe { *fp };
    unsafe {
        *fp = parent_fp;
    }
    prev
}
#[cfg(all(CAN_UNWIND_WITH_FRAME_POINTERS, target_vendor = "apple"))]
// Source: base/debug/stack_trace.cc:226
pub unsafe fn GetStackEnd() -> usize {
    unsafe extern "C" {
        fn pthread_get_stackaddr_np(thread: libc::pthread_t) -> *mut c_void;
    }
    unsafe { pthread_get_stackaddr_np(libc::pthread_self()) as usize }
}
#[cfg(all(
    CAN_UNWIND_WITH_FRAME_POINTERS,
    not(any(
        target_vendor = "apple",
        target_os = "android",
        target_os = "linux",
        chromium_is_chromeos
    ))
))]
// Source: base/debug/stack_trace.cc:226
pub unsafe fn GetStackEnd() -> usize {
    0
}
// Source: base/debug/stack_trace.h:213-217
#[cfg(CAN_UNWIND_WITH_FRAME_POINTERS)]
pub const kEnableScanningByDefault: bool = cfg!(any(target_os = "linux", chromium_is_chromeos));
#[cfg(all(
    CAN_UNWIND_WITH_FRAME_POINTERS,
    any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "x86")
))]
#[inline(never)]
/// Safety: same native readable-stack/frame-chain contract, with the actual
/// CAN_UNWIND config. Root must force frame pointers in this Rust TU: the native
/// __builtin_frame_address(0) establishes a frame, while raw register access
/// alone does not. Without that actual compiler contract this is not closed.
// Source: base/debug/stack_trace.cc:428
pub unsafe fn TraceStackFramePointers(
    out_trace: *mut *const c_void,
    max_depth: usize,
    mut skip_initial: usize,
    enable_scanning: bool,
) -> usize {
    let fp_register: usize;
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("mov {}, x29",out(reg)fp_register,options(nomem,nostack,preserves_flags));
    }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mov {}, rbp",out(reg)fp_register,options(nomem,nostack,preserves_flags));
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("mov {}, ebp",out(reg)fp_register,options(nomem,nostack,preserves_flags));
    }
    let mut fp = fp_register.wrapping_sub(kStackFrameAdjustment);
    let stack_end = unsafe { GetStackEnd() };
    let mut depth = 0;
    let call_size = if cfg!(target_arch = "aarch64") { 4 } else { 0 };
    while depth < max_depth {
        let pc = unsafe { GetStackFramePC(fp) };
        if skip_initial != 0 {
            skip_initial -= 1;
        } else {
            unsafe {
                out_trace
                    .add(depth)
                    .write(pc.wrapping_sub(call_size) as *const c_void);
            }
            depth += 1;
        }
        let next_fp = unsafe { GetNextStackFrame(fp) };
        if unsafe { IsStackFrameValid(next_fp, fp, stack_end) } {
            fp = next_fp;
            continue;
        }
        if !enable_scanning {
            break;
        }
        let next_fp = unsafe { ScanStackForNextFrame(fp, stack_end) };
        if next_fp != 0 {
            fp = next_fp;
        } else {
            break;
        }
    }
    depth
}
struct AddressRange {
    start: usize,
    end: usize,
}
// Source: base/debug/stack_trace.cc:424
fn IsWithinRange(address: usize, range: &AddressRange) -> bool {
    address >= range.start && address <= range.end
}
#[cfg(all(CAN_UNWIND_WITH_FRAME_POINTERS, target_os = "android"))]
// Source: base/debug/stack_trace.cc:226
pub unsafe fn GetStackEnd() -> usize {
    unsafe {
        use crate::base::{
            process::process_handle::GetCurrentProcId, threading::platform_thread::PlatformThread,
        };
        static mut main_stack_end: usize = 0;
        let is_main_thread = GetCurrentProcId() == PlatformThread::CurrentId().raw();
        if is_main_thread && main_stack_end != 0 {
            return main_stack_end;
        }
        let mut stack_begin: *mut c_void = core::ptr::null_mut();
        let mut stack_size = 0usize;
        let mut attributes = MaybeUninit::<libc::pthread_attr_t>::uninit();
        let mut error = libc::pthread_getattr_np(libc::pthread_self(), attributes.as_mut_ptr());
        if error == 0 {
            error =
                libc::pthread_attr_getstack(attributes.as_ptr(), &mut stack_begin, &mut stack_size);
            libc::pthread_attr_destroy(attributes.as_mut_ptr());
        }
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/debug/stack_trace.cc\0".as_ptr().cast(),249);@condition b"!error\0".as_ptr();error==0);
        let stack_end = (stack_begin as usize).wrapping_add(stack_size);
        if is_main_thread {
            main_stack_end = stack_end;
        }
        stack_end
    }
}
#[cfg(all(
    CAN_UNWIND_WITH_FRAME_POINTERS,
    any(target_os = "linux", chromium_is_chromeos)
))]
// Source: base/debug/stack_trace.cc:226
pub unsafe fn GetStackEnd() -> usize {
    #[cfg(target_env = "gnu")]
    unsafe {
        use crate::base::{
            process::process_handle::GetCurrentProcId, threading::platform_thread::PlatformThread,
        };
        unsafe extern "C" {
            static __libc_stack_end: *mut c_void;
        }
        if GetCurrentProcId() == PlatformThread::CurrentId().raw() {
            return __libc_stack_end as usize;
        }
    }
    0
}

impl StackTrace {
    /// message is the actual native by-value argument object; its original owner
    /// destroys the moved-from value after this synchronous call. Native globals
    /// retain their original external synchronization and process lifetime.
    // Source: base/debug/stack_trace.cc:380
    pub unsafe fn SuppressStackTracesWithMessageForTesting<S: StdStringStorage>(
        mut message: Pin<&mut NativeStdString<S>>,
    ) {
        unsafe {
            let replacement = if message.as_ref().view().is_empty() {
                core::ptr::null_mut()
            } else {
                rechrom_stack_message_move_new(message.as_mut().as_native_mut_ptr())
            };
            let old =
                core::ptr::replace(core::ptr::addr_of_mut!(g_stack_trace_message), replacement);
            rechrom_stack_message_delete(old);
        }
    }
}
impl StackTrace {
    /// Both capabilities must be supplied from the real configured SDK ABI. dst
    /// is the final uninitialized std::string return slot; caller destroys it
    /// exactly once after successful synchronous return. No stream heap owner.
    // Source: base/debug/stack_trace.cc:367
    pub unsafe fn ToString_at<
        SS: crate::base::strings::native_ostream::StringStreamStorage,
        DS: StdStringStorage,
    >(
        &self,
        dst: *mut NativeStdString<DS>,
    ) {
        unsafe {
            self.ToStringWithPrefix_at::<SS, DS>(c"", dst);
        }
    }
    // Source: base/debug/stack_trace.cc:371
    pub unsafe fn ToStringWithPrefix_at<
        SS: crate::base::strings::native_ostream::StringStreamStorage,
        DS: StdStringStorage,
    >(
        &self,
        prefix: &CStr,
        dst: *mut NativeStdString<DS>,
    ) {
        use crate::base::strings::native_ostream::NativeStringStream;
        let mut stream = MaybeUninit::<NativeStringStream<SS>>::uninit();
        unsafe {
            NativeStringStream::initialize_at_assuming_native_abi(stream.as_mut_ptr());
        }
        let mut pinned = unsafe { Pin::new_unchecked(stream.assume_init_mut()) };
        #[cfg(not(any(__UCLIBC__, target_os = "aix")))]
        unsafe {
            self.OutputToStreamWithPrefix::<DS>(&mut pinned.as_mut().stream(), prefix);
        }
        unsafe {
            pinned.as_mut().take_str_at(dst);
            core::ptr::drop_in_place(stream.as_mut_ptr());
        }
    }
}
// Source: base/debug/stack_trace.h:26-29 (actual incomplete Win identities)
#[cfg(windows)]
unsafe extern "C" {
    pub type _EXCEPTION_POINTERS;
    pub type _CONTEXT;
}
// Source: base/check_op.h:154-160 (existing scoped-enum overload instantiation).
// No independent formatter: the actual underlying native int overload owns
// the returned malloc-family character buffer, as in the canonical producer.
impl crate::base::check_op::CheckOpValue for Mode {
    unsafe fn CheckOpValueStr(&self) -> *mut core::ffi::c_char {
        crate::base::check_op::CheckOpValueStr_int(*self as i32)
    }
}
// Source: base/debug/stack_trace.h:40-62; 185-186
#[cfg(all(
    unix,
    not(target_os = "fuchsia"),
    not(any(
        ADDRESS_SANITIZER,
        MEMORY_SANITIZER,
        THREAD_SANITIZER,
        LEAK_SANITIZER,
        UNDEFINED_SANITIZER
    ))
))]
pub use platform::SetStackDumpFirstChanceCallback;
#[cfg(all(unix, not(target_os = "fuchsia")))]
pub use platform::{CollectStackTrace, EnableInProcessStackDumping};
impl StackTrace {
    pub const kMaxTraces: usize = kMaxTraces;
}
// Source: base/debug/stack_trace.h:290-302
#[cfg(all(unix, not(target_os = "android"), not(target_os = "fuchsia")))]
pub mod internal {
    pub use super::super::stack_trace_posix::itoa_r;
}
