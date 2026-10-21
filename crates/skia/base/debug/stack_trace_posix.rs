// Source: base/debug/stack_trace_posix.cc
use super::stack_trace::{kMaxTraces, StackTrace};
use crate::base::strings::native_ostream::{NativeOStream, StdStringStorage};
use core::{
    ffi::{c_int, c_void, CStr},
    marker::PhantomPinned,
    pin::Pin,
};
static mut in_signal_handler: c_int = 0;
pub type FirstChanceCallback =
    unsafe extern "C" fn(c_int, *mut libc::siginfo_t, *mut c_void) -> bool;
static mut try_handle_signal: Option<FirstChanceCallback> = None;
pub trait BacktraceOutputHandler {
    fn HandleOutput(self: Pin<&mut Self>, output: &CStr);
}
// Source: base/debug/stack_trace_posix.cc:171
fn OutputPointer(pointer: *const c_void, mut handler: Pin<&mut dyn BacktraceOutputHandler>) {
    let mut buf = [0u8; 17];
    handler.as_mut().HandleOutput(c"0x");
    itoa_r(pointer as isize, 16, 12, &mut buf);
    handler
        .as_mut()
        .HandleOutput(unsafe { CStr::from_ptr(buf.as_ptr().cast()) });
}
#[cfg(any(USE_SYMBOLIZE, all(target_os = "ios", OFFICIAL_BUILD)))]
// Source: base/debug/stack_trace_posix.cc:181
fn OutputValue(value: usize, mut handler: Pin<&mut dyn BacktraceOutputHandler>) {
    let mut buf = [0u8; 30];
    itoa_r(value as isize, 10, 1, &mut buf);
    handler
        .as_mut()
        .HandleOutput(unsafe { CStr::from_ptr(buf.as_ptr().cast()) });
}
#[cfg(USE_SYMBOLIZE)]
// Source: base/debug/stack_trace_posix.cc:192
fn OutputFrameId(frame_id: usize, mut handler: Pin<&mut dyn BacktraceOutputHandler>) {
    handler.as_mut().HandleOutput(c"#");
    OutputValue(frame_id, handler.as_mut());
}
// Actual iOS official branch uses dladdr, never backtrace_symbols' misleading
// stripped-symbol names. Native libc supplies Dl_info layout and the OS call.
#[cfg(all(target_os = "ios", OFFICIAL_BUILD, not(USE_SYMBOLIZE)))]
// Source: base/debug/stack_trace_posix.cc:198
unsafe fn ProcessBacktrace(
    traces: &[*const c_void],
    prefix: &CStr,
    mut handler: Pin<&mut dyn BacktraceOutputHandler>,
) {
    let traces = &traces[..traces.len().min(kMaxTraces)];
    let mut printed = false;
    if unsafe { core::ptr::read_volatile(core::ptr::addr_of!(in_signal_handler)) } == 0
        && traces.len() <= c_int::MAX as usize
    {
        for (i, trace) in traces.iter().enumerate() {
            if !prefix.to_bytes().is_empty() {
                handler.as_mut().HandleOutput(prefix);
            }
            OutputValue(i, handler.as_mut());
            handler.as_mut().HandleOutput(c" ");
            let mut dl_info = core::mem::MaybeUninit::<libc::Dl_info>::uninit();
            if unsafe { libc::dladdr(*trace, dl_info.as_mut_ptr()) } != 0 {
                let dl_info = unsafe { dl_info.assume_init() };
                let filename = unsafe { CStr::from_ptr(dl_info.dli_fname) };
                let bytes = filename.to_bytes();
                let start = bytes.iter().rposition(|&c| c == b'/').map_or(0, |i| i + 1);
                handler.as_mut().HandleOutput(unsafe {
                    CStr::from_bytes_with_nul_unchecked(&filename.to_bytes_with_nul()[start..])
                });
            } else {
                handler.as_mut().HandleOutput(c"???");
            }
            handler.as_mut().HandleOutput(c" ");
            OutputPointer(*trace, handler.as_mut());
            handler.as_mut().HandleOutput(c"\n");
        }
        printed = true;
    }
    if !printed {
        for trace in traces {
            handler.as_mut().HandleOutput(c" [");
            OutputPointer(*trace, handler.as_mut());
            handler.as_mut().HandleOutput(c"]\n");
        }
    }
}
// Source: base/debug/stack_trace_posix.cc:321
pub(super) unsafe fn PrintToStderr(output: &CStr) {
    loop {
        let rv = unsafe {
            libc::write(
                libc::STDERR_FILENO,
                output.as_ptr().cast(),
                output.to_bytes().len(),
            )
        };
        if rv < 0 && unsafe { *errno_ptr() } == libc::EINTR {
            continue;
        }
        break;
    }
}
unsafe extern "C" {
    fn rechrom_debug_errno_ptr() -> *mut c_int;
}
unsafe fn errno_ptr() -> *mut c_int {
    unsafe { rechrom_debug_errno_ptr() }
}
struct PrintBacktraceOutputHandler {
    _pin: PhantomPinned,
}
impl BacktraceOutputHandler for PrintBacktraceOutputHandler {
    fn HandleOutput(self: Pin<&mut Self>, output: &CStr) {
        unsafe {
            PrintToStderr(output);
        }
    }
}
struct StreamBacktraceOutputHandler<'a, 'b> {
    os_: &'a mut NativeOStream<'b>,
    _pin: PhantomPinned,
}
impl BacktraceOutputHandler for StreamBacktraceOutputHandler<'_, '_> {
    fn HandleOutput(self: Pin<&mut Self>, output: &CStr) {
        unsafe {
            self.get_unchecked_mut()
                .os_
                .insert_cstr(output.as_ptr().cast());
        }
    }
}
// Source: base/debug/stack_trace_posix.cc:630
unsafe fn WarmUpBacktrace() {
    let _stack_trace = unsafe { StackTrace::new() };
}
/// Safety: trace points to count writable pointer slots, which need not be
/// initialized. Only the returned number is subsequently observed by StackTrace.
// Source: base/debug/stack_trace_posix.cc:1039
pub unsafe fn CollectStackTrace(trace: *mut *const c_void, count: usize) -> usize {
    #[cfg(all(EXCLUDE_UNWIND_TABLES, CAN_UNWIND_WITH_FRAME_POINTERS))]
    {
        return unsafe {
            super::stack_trace::TraceStackFramePointers(
                trace,
                count,
                0,
                super::stack_trace::kEnableScanningByDefault,
            )
        };
    }
    #[cfg(all(
        not(all(EXCLUDE_UNWIND_TABLES, CAN_UNWIND_WITH_FRAME_POINTERS)),
        any(
            target_vendor = "apple",
            all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
        )
    ))]
    {
        unsafe extern "C" {
            fn backtrace(trace: *mut *mut c_void, size: c_int) -> c_int;
        }
        let count = count.min(c_int::MAX as usize) as c_int;
        let read = unsafe { backtrace(trace.cast(), count) };
        return read.max(0) as usize;
    }
    #[cfg(all(
        not(all(EXCLUDE_UNWIND_TABLES, CAN_UNWIND_WITH_FRAME_POINTERS)),
        not(any(
            target_vendor = "apple",
            all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
        ))
    ))]
    {
        0
    }
}
// Source: base/debug/stack_trace_posix.cc:1059
pub(super) unsafe fn PrintMessageWithPrefix(prefix: &CStr, message: &CStr) {
    if !prefix.to_bytes().is_empty() {
        unsafe {
            PrintToStderr(prefix);
        }
    }
    unsafe {
        PrintToStderr(message);
    }
}
#[cfg(all(target_os = "ios", OFFICIAL_BUILD, not(USE_SYMBOLIZE)))]
// Source: base/debug/stack_trace_posix.cc:1069
pub(super) unsafe fn PrintWithPrefixImpl<S: StdStringStorage>(
    addresses: &[*const c_void],
    prefix: &CStr,
) {
    let mut handler = core::pin::pin!(PrintBacktraceOutputHandler {
        _pin: PhantomPinned
    });
    unsafe {
        ProcessBacktrace(addresses, prefix, handler.as_mut());
    }
}
#[cfg(all(target_os = "ios", OFFICIAL_BUILD, not(USE_SYMBOLIZE)))]
// Source: base/debug/stack_trace_posix.cc:1079
pub(super) unsafe fn OutputToStreamWithPrefixImpl<S: StdStringStorage>(
    addresses: &[*const c_void],
    os: &mut NativeOStream<'_>,
    prefix: &CStr,
) {
    let mut handler = core::pin::pin!(StreamBacktraceOutputHandler {
        os_: os,
        _pin: PhantomPinned
    });
    unsafe {
        ProcessBacktrace(addresses, prefix, handler.as_mut());
    }
}
// Async-signal-safe integer conversion; no printf, allocation or Rust formatter.
// Source: base/debug/stack_trace_posix.cc:1091
pub fn itoa_r(i: isize, base: c_int, mut padding: usize, buf: &mut [u8]) {
    if buf.is_empty() {
        return;
    }
    if base < 2 || base > 16 {
        buf[0] = 0;
        return;
    }
    let mut written = 0;
    let mut start = 0;
    let mut j = i as usize;
    if i < 0 && base == 10 {
        j = (-(i + 1)) as usize + 1;
        if written == buf.len() {
            buf[0] = 0;
            return;
        }
        buf[written] = b'-';
        written += 1;
        start += 1;
    }
    let digits = b"0123456789abcdef";
    loop {
        if written == buf.len() {
            buf[0] = 0;
            return;
        }
        buf[written] = digits[j % base as usize];
        written += 1;
        j /= base as usize;
        if padding > 0 {
            padding -= 1;
        }
        if j == 0 && padding == 0 {
            break;
        }
    }
    if written == buf.len() {
        buf[0] = 0;
        return;
    }
    buf[written] = 0;
    written += 1;
    buf[start..written - 1].reverse();
}

unsafe extern "C" {
    fn rechrom_debug_install_signal(
        signal: c_int,
        flags: c_int,
        handler: unsafe extern "C" fn(c_int, *mut libc::siginfo_t, *mut c_void),
    ) -> c_int;
    fn rechrom_debug_ignore_signal(signal: c_int) -> c_int;
    fn rechrom_debug_query_signal_matches(
        signal: c_int,
        expected: unsafe extern "C" fn(c_int, *mut libc::siginfo_t, *mut c_void),
        matches: *mut bool,
    ) -> c_int;
    fn rechrom_debug_siginfo_code(info: *const libc::siginfo_t) -> c_int;
    fn rechrom_debug_siginfo_signo(info: *const libc::siginfo_t) -> c_int;
    fn rechrom_debug_siginfo_address(info: *const libc::siginfo_t) -> *const c_void;
    #[cfg(all(target_os = "linux", any(target_arch = "x86", target_arch = "x86_64")))]
    fn rechrom_debug_ucontext_greg(context: *const c_void, index: usize) -> isize;
}
#[cfg(any(target_os = "linux", target_os = "android", chromium_is_chromeos))]
// Source: base/debug/stack_trace_posix.cc:328
unsafe extern "C" fn AlarmSignalHandler(
    _signal: c_int,
    _info: *mut libc::siginfo_t,
    _context: *mut c_void,
) {
    unsafe {
        PrintToStderr(c"Warning: Default signal handler failed to terminate process.\n");
        PrintToStderr(c"Calling exit_group() directly to prevent timeout.\n");
        libc::syscall(libc::SYS_exit_group, libc::EXIT_FAILURE);
    }
}
// Source: base/debug/stack_trace_posix.cc:349
unsafe extern "C" fn StackDumpSignalHandler<S: StdStringStorage>(
    signal: c_int,
    info: *mut libc::siginfo_t,
    void_context: *mut c_void,
) {
    unsafe {
        if let Some(handler) = try_handle_signal {
            if handler(signal, info, void_context) {
                rechrom_debug_install_signal(
                    signal,
                    (libc::SA_RESETHAND | libc::SA_SIGINFO) as c_int,
                    StackDumpSignalHandler::<S>,
                );
                return;
            }
        }
        #[cfg(not(target_vendor = "apple"))]
        core::ptr::write_volatile(core::ptr::addr_of_mut!(in_signal_handler), 1);
        #[cfg(target_vendor = "apple")]
        if !crate::base::dcheck_is_on::DCHECK_IS_ON() {
            core::ptr::write_volatile(core::ptr::addr_of_mut!(in_signal_handler), 1);
        }
        if crate::base::debug::debugger::BeingDebugged() {
            crate::base::debug::debugger::BreakDebugger();
        }
        PrintToStderr(c"Received signal ");
        let mut buf = [0u8; 1024];
        itoa_r(signal as isize, 10, 0, &mut buf);
        PrintToStderr(CStr::from_ptr(buf.as_ptr().cast()));
        let code = rechrom_debug_siginfo_code(info);
        if signal == libc::SIGBUS {
            PrintToStderr(if code == libc::BUS_ADRALN {
                c" BUS_ADRALN "
            } else if code == libc::BUS_ADRERR {
                c" BUS_ADRERR "
            } else if code == libc::BUS_OBJERR {
                c" BUS_OBJERR "
            } else {
                c" <unknown> "
            });
        } else if signal == libc::SIGFPE {
            PrintToStderr(if code == libc::FPE_FLTDIV {
                c" FPE_FLTDIV "
            } else if code == libc::FPE_FLTINV {
                c" FPE_FLTINV "
            } else if code == libc::FPE_FLTOVF {
                c" FPE_FLTOVF "
            } else if code == libc::FPE_FLTRES {
                c" FPE_FLTRES "
            } else if code == libc::FPE_FLTUND {
                c" FPE_FLTUND "
            } else if code == libc::FPE_INTDIV {
                c" FPE_INTDIV "
            } else if code == libc::FPE_INTOVF {
                c" FPE_INTOVF "
            } else {
                c" <unknown> "
            });
        } else if signal == libc::SIGILL {
            PrintToStderr(if code == libc::ILL_BADSTK {
                c" ILL_BADSTK "
            } else if code == libc::ILL_COPROC {
                c" ILL_COPROC "
            } else if code == libc::ILL_ILLOPC {
                c" ILL_ILLOPC "
            } else if code == libc::ILL_ILLOPN {
                c" ILL_ILLOPN "
            } else if code == libc::ILL_ILLADR {
                c" ILL_ILLADR "
            } else if code == libc::ILL_ILLTRP {
                c" ILL_ILLTRP "
            } else if code == libc::ILL_PRVOPC {
                c" ILL_PRVOPC "
            } else if code == libc::ILL_PRVREG {
                c" ILL_PRVREG "
            } else {
                c" <unknown> "
            });
        } else if signal == libc::SIGSEGV {
            if code == libc::SEGV_MAPERR {
                PrintToStderr(c" SEGV_MAPERR ");
            } else if code == libc::SEGV_ACCERR {
                PrintToStderr(c" SEGV_ACCERR ");
            } else {
                #[cfg(all(
                    target_arch = "x86_64",
                    any(target_os = "linux", target_os = "android", chromium_is_chromeos)
                ))]
                if code == libc::SI_KERNEL {
                    PrintToStderr(c" SI_KERNEL");
                } else {
                    PrintToStderr(c" <unknown> ");
                }
                #[cfg(not(all(
                    target_arch = "x86_64",
                    any(target_os = "linux", target_os = "android", chromium_is_chromeos)
                )))]
                PrintToStderr(c" <unknown> ");
            }
        }
        if signal == libc::SIGBUS
            || signal == libc::SIGFPE
            || signal == libc::SIGILL
            || signal == libc::SIGSEGV
        {
            itoa_r(
                rechrom_debug_siginfo_address(info) as isize,
                16,
                12,
                &mut buf,
            );
            PrintToStderr(CStr::from_ptr(buf.as_ptr().cast()));
        }
        PrintToStderr(c"\n");
        #[cfg(CFI_ENFORCEMENT_TRAP)]
        if signal == libc::SIGILL && code == libc::ILL_ILLOPN {
            PrintToStderr(
                c"CFI: Most likely a control flow integrity violation; for more information see:\n",
            );
            PrintToStderr(c"https://www.chromium.org/developers/testing/control-flow-integrity\n");
        }
        #[cfg(all(
            target_arch = "x86_64",
            any(target_os = "linux", target_os = "android", chromium_is_chromeos)
        ))]
        if signal == libc::SIGSEGV && code == libc::SI_KERNEL {
            PrintToStderr(c" Possibly a General Protection Fault, can be due to a non-canonical address dereference. See \"Intel 64 and IA-32 Architectures Software Developer’s Manual\", Volume 1, Section 3.3.7.1.\n");
        }
        StackTrace::new().Print::<S>();
        #[cfg(all(
            any(target_os = "linux", chromium_is_chromeos),
            any(target_arch = "x86", target_arch = "x86_64")
        ))]
        PrintContextRegisters(void_context, &mut buf);
        PrintToStderr(c"[end of stack trace]\n");
        if libc::signal(signal, libc::SIG_DFL) == libc::SIG_ERR {
            libc::_exit(libc::EXIT_FAILURE);
        }
        #[cfg(any(target_os = "linux", target_os = "android", chromium_is_chromeos))]
        {
            rechrom_debug_install_signal(
                libc::SIGALRM,
                libc::SA_RESETHAND as c_int,
                AlarmSignalHandler,
            );
            libc::alarm(5);
            let retval = libc::syscall(
                libc::SYS_rt_tgsigqueueinfo,
                libc::getpid(),
                libc::syscall(libc::SYS_gettid),
                rechrom_debug_siginfo_signo(info),
                info,
            );
            if retval == 0 {
                return;
            }
            if *errno_ptr() != libc::EPERM {
                libc::_exit(libc::EXIT_FAILURE);
            }
        }
        if libc::raise(signal) != 0 {
            libc::_exit(libc::EXIT_FAILURE);
        }
    }
}
// Source: base/debug/stack_trace_posix.cc:982
pub unsafe fn EnableInProcessStackDumping<S: StdStringStorage>() -> bool {
    unsafe {
        #[cfg(USE_SYMBOLIZE)]
        SandboxSymbolizeHelper::GetInstance();
        let mut success = rechrom_debug_ignore_signal(libc::SIGPIPE) == 0;
        WarmUpBacktrace();
        for signal in [
            libc::SIGILL,
            libc::SIGABRT,
            libc::SIGFPE,
            libc::SIGBUS,
            libc::SIGSEGV,
        ] {
            // Non-short-circuit &= preserves registration attempts after earlier failures.
            success &= rechrom_debug_install_signal(
                signal,
                (libc::SA_RESETHAND | libc::SA_SIGINFO) as c_int,
                StackDumpSignalHandler::<S>,
            ) == 0;
        }
        #[cfg(not(any(target_os = "linux", chromium_is_chromeos)))]
        {
            success &= rechrom_debug_install_signal(
                libc::SIGSYS,
                (libc::SA_RESETHAND | libc::SA_SIGINFO) as c_int,
                StackDumpSignalHandler::<S>,
            ) == 0;
        }
        success
    }
}
// SetStackDumpFirstChanceCallback's DCHECK/CHECK/sanitizer-warning branches
// require the actual CheckError/log-owner protocol; not a default success stub.

#[cfg(all(
    any(target_os = "linux", chromium_is_chromeos),
    any(target_arch = "x86", target_arch = "x86_64")
))]
// Source: base/debug/stack_trace_posix.cc:495
unsafe fn PrintContextRegisters(context: *mut c_void, buf: &mut [u8]) {
    #[cfg(target_arch = "x86")]
    let registers = [
        (c"  gs: ", libc::REG_GS as usize),
        (c"  fs: ", libc::REG_FS as usize),
        (c"  es: ", libc::REG_ES as usize),
        (c"  ds: ", libc::REG_DS as usize),
        (c" edi: ", libc::REG_EDI as usize),
        (c" esi: ", libc::REG_ESI as usize),
        (c" ebp: ", libc::REG_EBP as usize),
        (c" esp: ", libc::REG_ESP as usize),
        (c" ebx: ", libc::REG_EBX as usize),
        (c" edx: ", libc::REG_EDX as usize),
        (c" ecx: ", libc::REG_ECX as usize),
        (c" eax: ", libc::REG_EAX as usize),
        (c" trp: ", libc::REG_TRAPNO as usize),
        (c" err: ", libc::REG_ERR as usize),
        (c"  ip: ", libc::REG_EIP as usize),
        (c"  cs: ", libc::REG_CS as usize),
        (c" efl: ", libc::REG_EFL as usize),
        (c" usp: ", libc::REG_UESP as usize),
        (c"  ss: ", libc::REG_SS as usize),
    ];
    #[cfg(target_arch = "x86_64")]
    let registers = [
        (c"  r8: ", libc::REG_R8 as usize),
        (c"  r9: ", libc::REG_R9 as usize),
        (c" r10: ", libc::REG_R10 as usize),
        (c" r11: ", libc::REG_R11 as usize),
        (c" r12: ", libc::REG_R12 as usize),
        (c" r13: ", libc::REG_R13 as usize),
        (c" r14: ", libc::REG_R14 as usize),
        (c" r15: ", libc::REG_R15 as usize),
        (c"  di: ", libc::REG_RDI as usize),
        (c"  si: ", libc::REG_RSI as usize),
        (c"  bp: ", libc::REG_RBP as usize),
        (c"  bx: ", libc::REG_RBX as usize),
        (c"  dx: ", libc::REG_RDX as usize),
        (c"  ax: ", libc::REG_RAX as usize),
        (c"  cx: ", libc::REG_RCX as usize),
        (c"  sp: ", libc::REG_RSP as usize),
        (c"  ip: ", libc::REG_RIP as usize),
        (c" efl: ", libc::REG_EFL as usize),
        (c" cgf: ", libc::REG_CSGSFS as usize),
        (c" erf: ", libc::REG_ERR as usize),
        (c" trp: ", libc::REG_TRAPNO as usize),
        (c" msk: ", libc::REG_OLDMASK as usize),
        (c" cr2: ", libc::REG_CR2 as usize),
    ];
    let padding = if cfg!(target_arch = "x86") { 8 } else { 16 };
    for (i, (label, index)) in registers.iter().enumerate() {
        unsafe {
            PrintToStderr(label);
            itoa_r(
                rechrom_debug_ucontext_greg(context, *index),
                16,
                padding,
                buf,
            );
            PrintToStderr(CStr::from_ptr(buf.as_ptr().cast()));
        }
        if (i + 1) % 4 == 0 {
            unsafe {
                PrintToStderr(c"\n");
            }
        }
    }
    unsafe {
        PrintToStderr(c"\n");
    }
}

#[cfg(all(
    not(USE_SYMBOLIZE),
    not(all(target_os = "ios", OFFICIAL_BUILD)),
    any(
        target_vendor = "apple",
        all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
    )
))]
// Source: base/debug/stack_trace_posix.cc:121
unsafe fn DemangleSymbols<S: crate::base::strings::native_ostream::StdStringStorage>(
    mut text: core::pin::Pin<&mut crate::base::strings::native_ostream::NativeStdString<S>>,
) {
    use crate::base::strings::native_ostream::NativeStdString;
    use core::{mem::MaybeUninit, pin::Pin};
    unsafe extern "C" {
        fn __cxa_demangle(
            mangled: *const core::ffi::c_char,
            out: *mut core::ffi::c_char,
            length: *mut usize,
            status: *mut c_int,
        ) -> *mut core::ffi::c_char;
    }
    struct FreeDeleter(*mut core::ffi::c_char);
    impl Drop for FreeDeleter {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    libc::free(self.0.cast());
                }
            }
        }
    }
    let mut search_from = 0;
    while search_from < text.as_ref().size() {
        let Some(mangled_start) = text.as_ref().find_bytes(b"_Z", search_from) else {
            break;
        };
        let mangled_end = text
            .as_ref()
            .find_first_not_of(
                b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_",
                mangled_start,
            )
            .unwrap_or_else(|| text.as_ref().size());
        let mut mangled_symbol = MaybeUninit::<NativeStdString<S>>::uninit();
        unsafe {
            text.as_ref().substr_count_at_assuming_native_abi(
                mangled_start,
                mangled_end - mangled_start,
                mangled_symbol.as_mut_ptr(),
            );
        }
        let mangled_ref = unsafe { Pin::new_unchecked(mangled_symbol.assume_init_ref()) };
        let mut status = 0;
        let demangled_symbol = FreeDeleter(unsafe {
            __cxa_demangle(
                mangled_ref.c_str(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut status,
            )
        });
        if status == 0 {
            unsafe {
                text.as_mut()
                    .erase(mangled_start, mangled_end - mangled_start);
                text.as_mut().insert_cstr(mangled_start, demangled_symbol.0);
            }
            search_from = mangled_start.wrapping_add(
                unsafe { CStr::from_ptr(demangled_symbol.0) }
                    .to_bytes()
                    .len(),
            );
        } else {
            search_from = mangled_start.wrapping_add(2);
        }
        // Native reverse locals: unique_ptr<char,FreeDeleter>, then mangled std::string.
        drop(demangled_symbol);
        unsafe {
            core::ptr::drop_in_place(mangled_symbol.as_mut_ptr());
        }
    }
}
#[cfg(all(
    not(USE_SYMBOLIZE),
    not(all(target_os = "ios", OFFICIAL_BUILD)),
    any(
        target_vendor = "apple",
        all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
    )
))]
// Source: base/debug/stack_trace_posix.cc:198
unsafe fn ProcessBacktrace<S: crate::base::strings::native_ostream::StdStringStorage>(
    traces: &[*const c_void],
    prefix: &CStr,
    mut handler: Pin<&mut dyn BacktraceOutputHandler>,
) {
    use crate::base::strings::native_ostream::NativeStdString;
    use core::{mem::MaybeUninit, pin::Pin};
    unsafe extern "C" {
        fn backtrace_symbols(
            trace: *const *mut c_void,
            count: c_int,
        ) -> *mut *mut core::ffi::c_char;
    }
    struct FreeArray(*mut *mut core::ffi::c_char);
    impl Drop for FreeArray {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    libc::free(self.0.cast());
                }
            }
        }
    }
    let traces = &traces[..traces.len().min(kMaxTraces)];
    let mut printed = false;
    if unsafe { core::ptr::read_volatile(core::ptr::addr_of!(in_signal_handler)) } == 0
        && traces.len() <= c_int::MAX as usize
    {
        let trace_symbols =
            FreeArray(unsafe { backtrace_symbols(traces.as_ptr().cast(), traces.len() as c_int) });
        // HeapArray::empty is based on size, not pointer nullness; do not add an
        // invented successful fallback on allocation failure. The original owning-
        // pointer factory requires a valid array for this nonzero size.
        if !traces.is_empty() {
            for i in 0..traces.len() {
                let s = unsafe { *trace_symbols.0.add(i) };
                let mut trace_symbol = MaybeUninit::<NativeStdString<S>>::uninit();
                unsafe {
                    NativeStdString::initialize_cstr_at_assuming_native_abi(
                        trace_symbol.as_mut_ptr(),
                        s,
                    );
                }
                let mut symbol = unsafe { Pin::new_unchecked(trace_symbol.assume_init_mut()) };
                unsafe {
                    DemangleSymbols(symbol.as_mut());
                }
                if !prefix.to_bytes().is_empty() {
                    handler.as_mut().HandleOutput(prefix);
                }
                handler
                    .as_mut()
                    .HandleOutput(unsafe { CStr::from_ptr(symbol.as_ref().c_str()) });
                handler.as_mut().HandleOutput(c"\n");
                unsafe {
                    core::ptr::drop_in_place(trace_symbol.as_mut_ptr());
                }
            }
            printed = true;
        }
    }
    if !printed {
        for trace in traces {
            handler.as_mut().HandleOutput(c" [");
            OutputPointer(*trace, handler.as_mut());
            handler.as_mut().HandleOutput(c"]\n");
        }
    }
}
// The generic storage parameter is genuine SDK ABI input, not an alternate
// symbolization provider. It only supplies the inline native std::string extent.
#[cfg(all(
    not(USE_SYMBOLIZE),
    not(all(target_os = "ios", OFFICIAL_BUILD)),
    any(
        target_vendor = "apple",
        all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
    )
))]
// Source: base/debug/stack_trace_posix.cc:1069
pub(super) unsafe fn PrintWithPrefixImpl<
    S: crate::base::strings::native_ostream::StdStringStorage,
>(
    addresses: &[*const c_void],
    prefix: &CStr,
) {
    let mut handler = core::pin::pin!(PrintBacktraceOutputHandler {
        _pin: PhantomPinned
    });
    unsafe {
        ProcessBacktrace::<S>(addresses, prefix, handler.as_mut());
    }
}
#[cfg(all(
    not(USE_SYMBOLIZE),
    not(all(target_os = "ios", OFFICIAL_BUILD)),
    any(
        target_vendor = "apple",
        all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
    )
))]
// Source: base/debug/stack_trace_posix.cc:1079
pub(super) unsafe fn OutputToStreamWithPrefixImpl<
    S: crate::base::strings::native_ostream::StdStringStorage,
>(
    addresses: &[*const c_void],
    os: &mut NativeOStream<'_>,
    prefix: &CStr,
) {
    let mut handler = core::pin::pin!(StreamBacktraceOutputHandler {
        os_: os,
        _pin: PhantomPinned
    });
    unsafe {
        ProcessBacktrace::<S>(addresses, prefix, handler.as_mut());
    }
}
#[cfg(not(any(
    ADDRESS_SANITIZER,
    MEMORY_SANITIZER,
    THREAD_SANITIZER,
    LEAK_SANITIZER,
    UNDEFINED_SANITIZER
)))]
// Source: base/debug/stack_trace_posix.cc:1018
pub unsafe fn SetStackDumpFirstChanceCallback(handler: Option<FirstChanceCallback>) -> bool {
    unsafe {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/debug/stack_trace_posix.cc\0".as_ptr().cast(),1019);@condition b"try_handle_signal == nullptr || handler == nullptr\0".as_ptr();try_handle_signal.is_none()||handler.is_none());
        try_handle_signal = handler;
        true
    }
}
#[cfg(not(any(
    target_vendor = "apple",
    all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
)))]
// Source: base/debug/stack_trace_posix.cc:1069
pub(super) unsafe fn PrintWithPrefixImpl<S: StdStringStorage>(
    _addresses: &[*const c_void],
    _prefix: &CStr,
) {
}
#[cfg(all(
    USE_SYMBOLIZE,
    any(
        target_vendor = "apple",
        all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
    )
))]
// Source: base/debug/stack_trace_posix.cc:198
unsafe fn ProcessBacktrace<S: StdStringStorage>(
    traces: &[*const c_void],
    prefix: &CStr,
    mut handler: Pin<&mut dyn BacktraceOutputHandler>,
) {
    unsafe extern "C" {
        fn rechrom_debug_symbolize(
            pc: *mut c_void,
            out: *mut core::ffi::c_char,
            size: usize,
        ) -> bool;
        #[cfg(ENABLE_STACK_TRACE_LINE_NUMBERS)]
        fn rechrom_debug_dwarf_offsets(
            trace: *const *const c_void,
            offsets: *mut u64,
            count: usize,
        );
        #[cfg(ENABLE_STACK_TRACE_LINE_NUMBERS)]
        fn rechrom_debug_dwarf_line(
            pc: *const c_void,
            offset: u64,
            out: *mut core::ffi::c_char,
            size: usize,
        ) -> bool;
    }
    let traces = &traces[..traces.len().min(kMaxTraces)];
    #[cfg(ENABLE_STACK_TRACE_LINE_NUMBERS)]
    let mut cu_offsets = [0u64; kMaxTraces];
    #[cfg(ENABLE_STACK_TRACE_LINE_NUMBERS)]
    unsafe {
        rechrom_debug_dwarf_offsets(traces.as_ptr(), cu_offsets.as_mut_ptr(), traces.len());
    }
    for (i, trace) in traces.iter().enumerate() {
        if !prefix.to_bytes().is_empty() {
            handler.as_mut().HandleOutput(prefix);
        }
        OutputFrameId(i, handler.as_mut());
        handler.as_mut().HandleOutput(c" ");
        OutputPointer(*trace, handler.as_mut());
        handler.as_mut().HandleOutput(c" ");
        let mut buf = [0u8; 1024];
        let address = (*trace as usize).wrapping_sub(1) as *mut c_void;
        if unsafe { rechrom_debug_symbolize(address, buf.as_mut_ptr().cast(), buf.len()) } {
            handler
                .as_mut()
                .HandleOutput(unsafe { CStr::from_ptr(buf.as_ptr().cast()) });
            #[cfg(ENABLE_STACK_TRACE_LINE_NUMBERS)]
            if unsafe {
                rechrom_debug_dwarf_line(address, cu_offsets[i], buf.as_mut_ptr().cast(), buf.len())
            } {
                handler.as_mut().HandleOutput(c" [");
                handler
                    .as_mut()
                    .HandleOutput(unsafe { CStr::from_ptr(buf.as_ptr().cast()) });
                handler.as_mut().HandleOutput(c"]");
            }
        } else {
            handler.as_mut().HandleOutput(c"<unknown>");
        }
        handler.as_mut().HandleOutput(c"\n");
    }
}
#[cfg(all(
    USE_SYMBOLIZE,
    any(
        target_vendor = "apple",
        all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
    )
))]
// Source: base/debug/stack_trace_posix.cc:1069
pub(super) unsafe fn PrintWithPrefixImpl<S: StdStringStorage>(
    addresses: &[*const c_void],
    prefix: &CStr,
) {
    let mut handler = core::pin::pin!(PrintBacktraceOutputHandler {
        _pin: PhantomPinned
    });
    unsafe {
        ProcessBacktrace::<S>(addresses, prefix, handler.as_mut());
    }
}
#[cfg(all(
    USE_SYMBOLIZE,
    any(
        target_vendor = "apple",
        all(target_env = "gnu", not(__UCLIBC__), not(target_os = "aix"))
    )
))]
// Source: base/debug/stack_trace_posix.cc:1079
pub(super) unsafe fn OutputToStreamWithPrefixImpl<S: StdStringStorage>(
    addresses: &[*const c_void],
    os: &mut NativeOStream<'_>,
    prefix: &CStr,
) {
    let mut handler = core::pin::pin!(StreamBacktraceOutputHandler {
        os_: os,
        _pin: PhantomPinned
    });
    unsafe {
        ProcessBacktrace::<S>(addresses, prefix, handler.as_mut());
    }
}
