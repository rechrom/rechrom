//! Actual Chromium logging owner fields and management operations. External source;
//! these language mappings do not add to the Skia source denominator.
use core::ffi::CStr;
// Source: base/logging/log_severity.h:18-21
pub const LOGGING_INFO: i32 = 0;
pub const LOGGING_WARNING: i32 = 1;
pub const LOGGING_ERROR: i32 = 2;
pub const LOGGING_FATAL: i32 = 3;
// Source: base/logging/logging_settings.h:30-46
pub const LOG_NONE: u32 = 0;
pub const LOG_TO_FILE: u32 = 1 << 0;
pub const LOG_TO_SYSTEM_DEBUG_LOG: u32 = 1 << 1;
pub const LOG_TO_STDERR: u32 = 1 << 2;
#[cfg(target_os = "fuchsia")]
const LOG_DEFAULT: u32 = LOG_TO_SYSTEM_DEBUG_LOG;
#[cfg(windows)]
const LOG_DEFAULT: u32 = LOG_TO_FILE;
#[cfg(unix)]
const LOG_DEFAULT: u32 = LOG_TO_SYSTEM_DEBUG_LOG | LOG_TO_STDERR;
// Source: base/logging.cc:121-121
static mut g_min_log_level: i32 = 0;
// Source: base/logging.cc:206-206
static mut g_logging_destination: u32 = LOG_DEFAULT;
// Source: base/logging.cc:234-238
static mut g_log_process_id: bool = false;
static mut g_log_thread_id: bool = false;
static mut g_log_timestamp: bool = true;
static mut g_log_tickcount: bool = false;
static mut g_log_prefix: *const u8 = core::ptr::null();
// Source: base/logging.cc:241-241
static mut show_error_dialogs: bool = false;
// Source: base/logging.h:300-304
// std::string is a byte slice for the synchronous callback, not UTF8 conversion.
pub type LogMessageHandlerFunction = unsafe fn(i32, *const u8, i32, usize, &[u8]) -> bool;
// Source: base/logging.cc:252-252
static mut g_log_message_handler: Option<LogMessageHandlerFunction> = None;
// Source: base/logging.cc:577-579
/// Safety: native global configuration requires external synchronization with readers.
pub unsafe fn SetMinLogLevel(level: i32) {
    unsafe {
        g_min_log_level = LOGGING_FATAL.min(level);
    }
}
// Source: base/logging.cc:581-583
pub unsafe fn GetMinLogLevel() -> i32 {
    unsafe { g_min_log_level }
}
// Source: base/logging.cc:585-593
pub unsafe fn ShouldCreateLogMessage(severity: i32) -> bool {
    unsafe {
        if severity < g_min_log_level {
            return false;
        }
        g_logging_destination != LOG_NONE
            || g_log_message_handler.is_some()
            || severity >= LOGGING_ERROR
    }
}
// Source: base/logging.cc:599-614
pub unsafe fn ShouldLogToStderr(severity: i32) -> bool {
    unsafe {
        if g_logging_destination & LOG_TO_STDERR != 0 {
            return true;
        }
        #[cfg(target_os = "fuchsia")]
        {
            return false;
        }
        #[cfg(not(target_os = "fuchsia"))]
        {
            if severity >= LOGGING_ERROR {
                return g_logging_destination & !LOG_TO_FILE == LOG_NONE;
            }
            false
        }
    }
}
// Source: base/logging.cc:616-618
pub unsafe fn GetVlogVerbosity() -> i32 {
    (-1).max(LOGGING_INFO - unsafe { GetMinLogLevel() })
}
// Source: base/logging.cc:630-638
pub unsafe fn SetLogItems(process: bool, thread: bool, timestamp: bool, tickcount: bool) {
    unsafe {
        g_log_process_id = process;
        g_log_thread_id = thread;
        g_log_timestamp = timestamp;
        g_log_tickcount = tickcount;
    }
}
// Source: base/logging.cc:640-644
/// Safety: nonnull prefix must be a live NUL-terminated static string, as in native.
#[derive(Debug)]
pub struct PendingLogMessageFatal;
pub unsafe fn SetLogPrefix(prefix: *const u8) -> Result<(), PendingLogMessageFatal> {
    if crate::base::dcheck_is_on::DCHECK_IS_ON() {
        let valid = prefix.is_null()
            || unsafe { CStr::from_ptr(prefix.cast()) }
                .to_bytes()
                .iter()
                .all(|c| b"abcdefghijklmnopqrstuvwxyz".contains(c));
        // Failure still requires the real CHECK/LogMessageFatal owner; no Rust panic replacement.
        if !valid {
            return Err(PendingLogMessageFatal);
        }
    }
    unsafe {
        g_log_prefix = prefix;
    }
    Ok(())
}
// Source: base/logging.cc:646-648
pub unsafe fn SetShowErrorDialogs(enable: bool) {
    unsafe {
        show_error_dialogs = enable;
    }
}
// Source: base/logging.cc:683-689
pub unsafe fn SetLogMessageHandler(handler: Option<LogMessageHandlerFunction>) {
    unsafe {
        g_log_message_handler = handler;
    }
}
pub unsafe fn GetLogMessageHandler() -> Option<LogMessageHandlerFunction> {
    unsafe { g_log_message_handler }
}
// Source: base/logging.h:527-672; base/logging.cc:719-1155
// Only genuine configured SDK storage admits these complete inline owners.
#[cfg(sk_native_ostream_abi_configured)]
mod owners {
    use super::*;
    use crate::interop::native_ostream::supplied_abi::SuppliedStdStringStorage;
    use crate::interop::native_ostream::{
        AbiMismatch, ConfiguredOStringStream, ConfiguredStdString, NativeOStream,
    };
    use core::{marker::PhantomPinned, mem::MaybeUninit, pin::Pin, ptr};
    // Source: base/logging.h:550-560
    // No repr(C) ABI claim: this is a Rust complete owner of the actual native
    // stream subobject, not a counterfeit C++ LogMessage allocation/vptr.
    pub struct LogMessage {
        severity_: i32,
        stream_: ConfiguredOStringStream,
        message_start_: MaybeUninit<usize>,
        file_: *const u8,
        line_: i32,
        _pin: PhantomPinned,
    }
    impl LogMessage {
        // Source: base/logging.cc:719-722
        /// out is exclusive uninitialized final storage, pinned until destruction.
        /// On Err no LogMessage exists and no destructor may run. On Ok destroy once.
        /// Native global logging and native stream synchronization contracts apply.
        pub unsafe fn construct_at(
            out: *mut Self,
            file: *const u8,
            line: i32,
            severity: i32,
        ) -> Result<(), AbiMismatch> {
            unsafe {
                // Validate both actual configured return-object and stream ABI before Init.
                ConfiguredStdString::validate_configured_abi()?;
                ConfiguredOStringStream::initialize_at(ptr::addr_of_mut!((*out).stream_))?;
                ptr::addr_of_mut!((*out).severity_).write(severity);
                ptr::addr_of_mut!((*out).file_).write(file);
                ptr::addr_of_mut!((*out).line_).write(line);
                ptr::addr_of_mut!((*out)._pin).write(PhantomPinned);
                ptr::addr_of_mut!((*out).message_start_).write(MaybeUninit::uninit());
                Pin::new_unchecked(&mut *out).Init(file, line);
                Ok(())
            }
        }
        // Source: base/logging.h:534-541
        pub fn stream(self: Pin<&mut Self>) -> NativeOStream<'_> {
            unsafe { self.map_unchecked_mut(|x| &mut x.stream_) }.stream()
        }
        pub fn severity(&self) -> i32 {
            self.severity_
        }
        pub fn file(&self) -> *const u8 {
            self.file_
        }
        pub fn line(&self) -> i32 {
            self.line_
        }
        pub fn message_start(&self) -> usize {
            unsafe { self.message_start_.assume_init() }
        }
        pub unsafe fn str_at(self: Pin<&Self>, out: *mut ConfiguredStdString) {
            unsafe {
                self.map_unchecked(|x| &x.stream_).configured_str_at(out);
            }
        }
        // Source: base/logging.cc:921-924
        pub unsafe fn BuildCrashString_at(self: Pin<&Self>, out: *mut ConfiguredStdString) {
            unsafe {
                BuildCrashString_at(
                    self.file_,
                    self.line_,
                    &self.map_unchecked(|x| &x.stream_).view()[self.message_start()..],
                    out,
                );
            }
        }
        // Source: base/logging.cc:927-1001
        unsafe fn Init(mut self: Pin<&mut Self>, file: *const u8, line: i32) {
            unsafe {
                let _last_error = crate::base::strings::stringprintf::ScopedClearLastError::new();
                let file_bytes = CStr::from_ptr(file.cast()).to_bytes();
                let filename = if file_bytes.first() == Some(&b'.') {
                    &file_bytes[6.min(file_bytes.len())..]
                } else {
                    file_bytes
                };
                #[cfg(chromium_is_chromeos)]
                if g_log_format == LogFormat::LOG_FORMAT_SYSLOG {
                    let severity = self.severity_;
                    self.as_mut().InitWithSyslogPrefix(
                        filename,
                        line,
                        TickCount(),
                        log_severity_name(severity),
                        g_log_prefix,
                        g_log_process_id,
                        g_log_thread_id,
                        g_log_timestamp,
                        g_log_tickcount,
                    );
                    let n = self.as_ref().map_unchecked(|x| &x.stream_).view().len();
                    self.get_unchecked_mut().message_start_.write(n);
                    return;
                }
                let severity = self.severity_;
                let mut stream = self.as_mut().stream();
                stream.insert_char(b'[');
                if !g_log_prefix.is_null() {
                    stream.insert_cstr(g_log_prefix);
                    stream.insert_char(b':');
                }
                if g_log_process_id {
                    stream
                        .insert_uint(crate::base::process::process_handle::GetUniqueIdForProcess());
                    stream.insert_char(b':');
                }
                if g_log_thread_id {
                    crate::base::threading::platform_thread::PlatformThread::CurrentId()
                        .WriteToStream(&mut stream);
                    stream.insert_char(b':');
                }
                if g_log_timestamp {
                    stream.setfill(b'0');
                    #[cfg(windows)]
                    {
                        let mut local_time = MaybeUninit::<crate::win::SYSTEMTIME>::uninit();
                        crate::win::GetLocalTime(local_time.as_mut_ptr());
                        let t = local_time.assume_init();
                        for (v, sep) in [
                            (t.wMonth, 0),
                            (t.wDay, b'/'),
                            (t.wHour, 0),
                            (t.wMinute, 0),
                            (t.wSecond, b'.'),
                        ] {
                            stream.setw(2);
                            stream.insert_ushort(v);
                            if sep != 0 {
                                stream.insert_char(sep);
                            }
                        }
                        stream.setw(3);
                        stream.insert_ushort(t.wMilliseconds);
                        stream.insert_char(b':');
                    }
                    #[cfg(any(unix, target_os = "fuchsia"))]
                    {
                        let mut tv = MaybeUninit::<libc::timeval>::uninit();
                        libc::gettimeofday(tv.as_mut_ptr(), ptr::null_mut());
                        let tv = tv.assume_init();
                        let t = tv.tv_sec;
                        let mut local_time = MaybeUninit::<libc::tm>::uninit();
                        libc::localtime_r(&t, local_time.as_mut_ptr());
                        let t = local_time.assume_init();
                        for (v, sep) in [
                            (1 + t.tm_mon, 0),
                            (t.tm_mday, b'/'),
                            (t.tm_hour, 0),
                            (t.tm_min, 0),
                            (t.tm_sec, b'.'),
                        ] {
                            stream.setw(2);
                            stream.insert_int(v);
                            if sep != 0 {
                                stream.insert_char(sep);
                            }
                        }
                        stream.setw(6);
                        stream.insert_long(tv.tv_usec as core::ffi::c_long);
                        stream.insert_char(b':');
                    }
                }
                if g_log_tickcount {
                    stream.insert_ullong(TickCount());
                    stream.insert_char(b':');
                }
                if severity >= 0 {
                    stream.insert_cstr(log_severity_name(severity));
                } else {
                    stream.insert_cstr(b"VERBOSE\0".as_ptr());
                    stream.insert_int(-severity);
                }
                stream.insert_char(b':');
                stream.insert_bytes(filename);
                stream.insert_char(b':');
                stream.insert_int(line);
                stream.insert_bytes(b"] ");
                drop(stream);
                let n = self.as_ref().map_unchecked(|x| &x.stream_).view().len();
                self.get_unchecked_mut().message_start_.write(n);
            }
        }
        // Source: base/logging.cc:728-919
        pub unsafe fn Flush(mut self: Pin<&mut Self>) {
            unsafe {
                let _last_error = crate::base::strings::stringprintf::ScopedClearLastError::new();
                let stack_start = self.as_ref().map_unchecked(|x| &x.stream_).view().len();
                #[cfg(all(
                    not(official_build),
                    not(target_env = "uclibc"),
                    not(target_os = "aix")
                ))]
                if self.severity_ == LOGGING_FATAL && !crate::base::debug::debugger::BeingDebugged()
                {
                    let stack_trace = crate::base::debug::stack_trace::StackTrace::new();
                    self.as_mut().stream().endl();
                    stack_trace
                        .OutputToStream::<SuppliedStdStringStorage>(&mut self.as_mut().stream());
                    #[cfg(target_os = "android")]
                    {
                        let mut java = MaybeUninit::<ConfiguredStdString>::uninit();
                        crate::base::android::jni_android::GetJavaStackTraceIfPresent_at(
                            java.as_mut_ptr(),
                        );
                        let java = Pin::new_unchecked(java.assume_init_mut());
                        if !java.as_ref().view().is_empty() {
                            let mut os = self.as_mut().stream();
                            os.insert_bytes(b"Java stack (may interleave with native stack):\n");
                            os.insert_string(java.as_ref());
                            os.insert_char(b'\n');
                        }
                        ptr::drop_in_place(java.get_unchecked_mut());
                    }
                    let task_trace = crate::base::debug::task_trace::TaskTrace::new();
                    if !task_trace.empty() {
                        task_trace.OutputToStream::<SuppliedStdStringStorage>(
                            &mut self.as_mut().stream(),
                        );
                    }
                    let task=crate::base::task::common::task_annotator::TaskAnnotator::CurrentTaskForThread();
                    if !task.is_null() && (*task).ipc_hash != 0 {
                        let mut text = MaybeUninit::<ConfiguredStdString>::uninit();
                        crate::base::strings::stringprintf::StringPrintfHex8_at(
                            text.as_mut_ptr(),
                            (*task).ipc_hash,
                        );
                        let text = Pin::new_unchecked(text.assume_init_mut());
                        let mut os = self.as_mut().stream();
                        os.insert_bytes(b"IPC message handler context: ");
                        os.insert_string(text.as_ref());
                        os.endl();
                        drop(os);
                        ptr::drop_in_place(text.get_unchecked_mut());
                    }
                    crate::base::debug::crash_logging::OutputCrashKeysToStream(
                        self.as_mut().stream().as_native_mut_ptr().cast(),
                    );
                }
                self.as_mut().stream().endl();
                let mut str_newline = MaybeUninit::<ConfiguredStdString>::uninit();
                self.as_ref().str_at(str_newline.as_mut_ptr());
                let mut str_newline = Pin::new_unchecked(str_newline.assume_init_mut());
                // Cleanup destruction order: fatal callback, then original str_newline, then
                // ScopedClearLastError. Raw identity ends all mutable stream leases first.
                struct StringDrop(*mut ConfiguredStdString);
                impl Drop for StringDrop {
                    fn drop(&mut self) {
                        unsafe {
                            ptr::drop_in_place(self.0);
                        }
                    }
                }
                let string_drop = StringDrop(str_newline.as_mut().get_unchecked_mut());
                TraceLogMessage(
                    self.file_,
                    self.line_,
                    &str_newline.as_ref().view()[self.message_start()..],
                );
                struct FatalCleanup {
                    message: *const LogMessage,
                    stack_start: usize,
                    string: *const ConfiguredStdString,
                }
                impl Drop for FatalCleanup {
                    fn drop(&mut self) {
                        unsafe {
                            let message = &*self.message;
                            if message.severity_ == LOGGING_FATAL {
                                message.HandleFatal(
                                    self.stack_start,
                                    Pin::new_unchecked(&*self.string).view(),
                                );
                            }
                        }
                    }
                }
                let fatal_cleanup = FatalCleanup {
                    message: self.as_ref().get_ref(),
                    stack_start,
                    string: str_newline.as_ref().get_ref(),
                };
                if self.severity_ == LOGGING_FATAL {
                    SetLogFatalCrashKey(self.as_ref());
                }
                let bytes = str_newline.as_ref().view();
                if let Some(handler) = g_log_message_handler {
                    if handler(
                        self.severity_,
                        self.file_,
                        self.line_,
                        self.message_start(),
                        bytes,
                    ) {
                        return;
                    }
                }
                if g_logging_destination & LOG_TO_SYSTEM_DEBUG_LOG != 0 {
                    #[cfg(windows)]
                    crate::win::OutputDebugStringA(str_newline.as_ref().c_str().cast());
                    #[cfg(target_vendor = "apple")]
                    {
                        let bundle = crate::core_foundation::bundle::CFBundleGetMainBundle();
                        let id = if bundle.is_null() {
                            ptr::null()
                        } else {
                            crate::core_foundation::bundle::CFBundleGetIdentifier(bundle)
                        };
                        let mut main_bundle_id = MaybeUninit::<ConfiguredStdString>::uninit();
                        if id.is_null() {
                            ConfiguredStdString::configured_empty_at(main_bundle_id.as_mut_ptr());
                        } else {
                            crate::base::strings::sys_string_conversions::SysCFStringRefToUTF8_at(
                                id,
                                main_bundle_id.as_mut_ptr(),
                            );
                        }
                        let main_bundle_id = Pin::new_unchecked(main_bundle_id.assume_init_mut());
                        let subsystem = if main_bundle_id.as_ref().view().is_empty() {
                            ptr::null()
                        } else {
                            main_bundle_id.as_ref().c_str().cast()
                        };
                        let log = apple_os_log::OSLog::new(subsystem);
                        log.write_public_string(
                            apple_os_log::log_type(self.severity_),
                            str_newline.as_ref().c_str().cast(),
                        );
                        drop(log);
                        ptr::drop_in_place(main_bundle_id.get_unchecked_mut());
                    }
                    #[cfg(target_os = "android")]
                    {
                        let priority = match self.severity_ {
                            LOGGING_INFO => 4,
                            LOGGING_WARNING => 5,
                            LOGGING_ERROR => 6,
                            LOGGING_FATAL => 7,
                            n if n < 0 => 2,
                            _ => 0,
                        };
                        if crate::base::dcheck_is_on::DCHECK_IS_ON() {
                            let mut lines = crate::base::strings::string_split::SplitString(
                                bytes,
                                b"\n",
                                crate::base::strings::string_split::KEEP_WHITESPACE,
                                crate::base::strings::string_split::SPLIT_WANT_ALL,
                            );
                            lines.pop();
                            for line in &lines {
                                crate::android::log::__android_log_write(
                                    priority,
                                    b"chromium\0".as_ptr(),
                                    line.c_str().cast(),
                                );
                            }
                        } else {
                            crate::android::log::__android_log_write(
                                priority,
                                b"chromium\0".as_ptr(),
                                str_newline.as_ref().c_str().cast(),
                            );
                        }
                    }
                    #[cfg(target_os = "fuchsia")]
                    {
                        let message = &bytes[self.message_start()..];
                        GetScopedFxLogger().LogMessage(
                            self.file_,
                            self.line_ as u32,
                            &message[..message.len() - 1],
                            self.severity_,
                        );
                    }
                }
                if ShouldLogToStderr(self.severity_) {
                    WriteToFd(libc::STDERR_FILENO, bytes.as_ptr(), bytes.len());
                }
                if g_logging_destination & LOG_TO_FILE != 0 {
                    #[cfg(any(unix, target_os = "fuchsia"))]
                    let _guard =
                        crate::base::synchronization::lock::AutoLock::new(GetLoggingLock());
                    if InitializeLogFileHandle() {
                        #[cfg(windows)]
                        {
                            let mut written = MaybeUninit::<u32>::uninit();
                            crate::win::WriteFile(
                                g_log_file,
                                bytes.as_ptr().cast(),
                                bytes.len() as u32,
                                written.as_mut_ptr(),
                                ptr::null_mut(),
                            );
                        }
                        #[cfg(any(unix, target_os = "fuchsia"))]
                        {
                            libc::fwrite(bytes.as_ptr().cast(), bytes.len(), 1, g_log_file);
                            libc::fflush(g_log_file);
                        }
                    }
                }
                drop(fatal_cleanup);
                drop(string_drop);
            }
        }
        // Source: base/logging.cc:1003-1041
        unsafe fn HandleFatal(&self, stack_start: usize, str_newline: &[u8]) {
            unsafe {
                let mut str_stack = [0u8; 1024];
                let end = str_newline
                    .iter()
                    .position(|&x| x == 0)
                    .unwrap_or(str_newline.len());
                let count = end.min(1023);
                str_stack[..count].copy_from_slice(&str_newline[..count]);
                crate::base::debug::alias::Alias((&str_stack as *const [u8; 1024]).cast());
                let stack = &*ptr::addr_of!(LOG_ASSERT_HANDLER_STACK);
                if let Some(handler) = stack.last().cloned() {
                    if let Some(handler) = handler {
                        handler(
                            self.file_,
                            self.line_,
                            &str_newline[self.message_start()..stack_start],
                            &str_newline[stack_start..],
                        );
                    }
                } else {
                    #[cfg(not(sk_native_ndebug))]
                    if !crate::base::debug::debugger::BeingDebugged() {
                        DisplayDebugMessageInDialog(Pin::new_unchecked(&self.stream_).view());
                    }
                    crate::base::immediate_crash::ImmediateCrash();
                }
            }
        }
    }
    // Source: base/logging.cc:724-726. Body first, then sole inline stream member.
    impl Drop for LogMessage {
        fn drop(&mut self) {
            unsafe {
                Pin::new_unchecked(self).Flush();
            }
        }
    }
    // Source: base/logging.h:578-582; base/logging.cc:1043-1046
    pub struct LogMessageFatal {
        parent: LogMessage,
    }
    impl LogMessageFatal {
        pub unsafe fn construct_at(
            out: *mut Self,
            file: *const u8,
            line: i32,
            severity: i32,
        ) -> Result<(), AbiMismatch> {
            unsafe {
                LogMessage::construct_at(ptr::addr_of_mut!((*out).parent), file, line, severity)
            }
        }
        pub fn parent(self: Pin<&mut Self>) -> Pin<&mut LogMessage> {
            unsafe { self.map_unchecked_mut(|x| &mut x.parent) }
        }
        pub fn stream(self: Pin<&mut Self>) -> NativeOStream<'_> {
            self.parent().stream()
        }
    }
    impl Drop for LogMessageFatal {
        fn drop(&mut self) {
            unsafe {
                Pin::new_unchecked(&mut self.parent).Flush();
                crate::base::immediate_crash::ImmediateCrash();
            }
        }
    }
    // Source: base/logging.h:587-595 (actual default constructor/void operator body).
    pub struct LogMessageVoidify;
    impl LogMessageVoidify {
        pub const fn new() -> Self {
            Self
        }
        pub fn and(&self, _stream: &mut NativeOStream<'_>) {}
    }
    // Source: base/logging.h:588-595
    #[cfg(windows)]
    pub type SystemErrorCode = core::ffi::c_ulong;
    #[cfg(any(unix, target_os = "fuchsia"))]
    pub type SystemErrorCode = core::ffi::c_int;
    // Source: base/logging.cc:1055-1062
    pub unsafe fn GetLastSystemErrorCode() -> SystemErrorCode {
        unsafe {
            #[cfg(windows)]
            {
                crate::win::GetLastError()
            }
            #[cfg(any(unix, target_os = "fuchsia"))]
            {
                *crate::base::strings::stringprintf::errno_ptr()
            }
        }
    }
    // Source: base/logging.cc:1064-1087
    pub unsafe fn SystemErrorCodeToString_at(
        error_code: SystemErrorCode,
        out: *mut ConfiguredStdString,
    ) {
        unsafe {
            #[cfg(windows)]
            {
                let mut msgbuf: *mut u16 = ptr::null_mut();
                let len = crate::win::FormatMessageW(
                    crate::win::FORMAT_MESSAGE_ALLOCATE_BUFFER
                        | crate::win::FORMAT_MESSAGE_FROM_SYSTEM
                        | crate::win::FORMAT_MESSAGE_IGNORE_INSERTS,
                    ptr::null(),
                    error_code,
                    0,
                    (&mut msgbuf as *mut *mut u16).cast(),
                    0,
                    ptr::null_mut(),
                );
                if len != 0 {
                    let message = crate::base::strings::utf_string_conversions::WideToUTF16(msgbuf);
                    crate::win::LocalFree(msgbuf.cast());
                    let collapsed =
                        crate::base::strings::string_util::CollapseWhitespace(&message, true);
                    let mut utf8 = MaybeUninit::<ConfiguredStdString>::uninit();
                    crate::base::strings::utf_string_conversions::UTF16ToUTF8_at(
                        &collapsed,
                        utf8.as_mut_ptr(),
                    );
                    let mut suffix = MaybeUninit::<ConfiguredStdString>::uninit();
                    crate::base::strings::stringprintf::StringPrintfSystemHex_at(
                        suffix.as_mut_ptr(),
                        error_code,
                    );
                    ConfiguredStdString::configured_concat_at(
                        out,
                        Pin::new_unchecked(utf8.assume_init_ref()),
                        Pin::new_unchecked(suffix.assume_init_ref()),
                    );
                    ptr::drop_in_place(suffix.as_mut_ptr());
                    ptr::drop_in_place(utf8.as_mut_ptr());
                } else {
                    crate::base::strings::stringprintf::StringPrintfSystemError_at(
                        out,
                        crate::win::GetLastError(),
                        error_code,
                    );
                }
            }
            #[cfg(any(unix, target_os = "fuchsia"))]
            {
                let mut error = MaybeUninit::<ConfiguredStdString>::uninit();
                crate::base::posix::safe_strerror::safe_strerror_at(error_code, error.as_mut_ptr());
                let mut suffix = MaybeUninit::<ConfiguredStdString>::uninit();
                crate::base::strings::stringprintf::StringPrintfSystemDecimal_at(
                    suffix.as_mut_ptr(),
                    error_code,
                );
                ConfiguredStdString::configured_concat_at(
                    out,
                    Pin::new_unchecked(error.assume_init_ref()),
                    Pin::new_unchecked(suffix.assume_init_ref()),
                );
                ptr::drop_in_place(suffix.as_mut_ptr());
                ptr::drop_in_place(error.as_mut_ptr());
            }
        }
    }
    // Source: base/logging.h:600-662; base/logging.cc:1089-1155
    macro_rules! error_owner {
        ($name:ident,$fatal:ident) => {
            pub struct $name {
                parent: LogMessage,
                err_: SystemErrorCode,
            }
            impl $name {
                pub unsafe fn construct_at(
                    out: *mut Self,
                    file: *const u8,
                    line: i32,
                    severity: i32,
                    err: SystemErrorCode,
                ) -> Result<(), AbiMismatch> {
                    unsafe {
                        LogMessage::construct_at(
                            ptr::addr_of_mut!((*out).parent),
                            file,
                            line,
                            severity,
                        )?;
                        ptr::addr_of_mut!((*out).err_).write(err);
                        Ok(())
                    }
                }
                pub fn parent(self: Pin<&mut Self>) -> Pin<&mut LogMessage> {
                    unsafe { self.map_unchecked_mut(|x| &mut x.parent) }
                }
                pub fn stream(self: Pin<&mut Self>) -> NativeOStream<'_> {
                    self.parent().stream()
                }
                pub unsafe fn AppendError(mut self: Pin<&mut Self>) {
                    unsafe {
                        let _last_error =
                            crate::base::strings::stringprintf::ScopedClearLastError::new();
                        let err = self.err_;
                        let mut string = MaybeUninit::<ConfiguredStdString>::uninit();
                        SystemErrorCodeToString_at(err, string.as_mut_ptr());
                        let string = Pin::new_unchecked(string.assume_init_mut());
                        let mut os = self.as_mut().stream();
                        os.insert_bytes(b": ");
                        os.insert_string(string.as_ref());
                        drop(os);
                        ptr::drop_in_place(string.get_unchecked_mut());
                        let last_error = err;
                        crate::base::debug::alias::Alias(
                            (&last_error as *const SystemErrorCode).cast(),
                        );
                    }
                }
            }
            impl Drop for $name {
                fn drop(&mut self) {
                    unsafe {
                        Pin::new_unchecked(self).AppendError();
                    }
                }
            }
            pub struct $fatal {
                parent: $name,
            }
            impl $fatal {
                pub unsafe fn construct_at(
                    out: *mut Self,
                    file: *const u8,
                    line: i32,
                    severity: i32,
                    err: SystemErrorCode,
                ) -> Result<(), AbiMismatch> {
                    unsafe {
                        $name::construct_at(
                            ptr::addr_of_mut!((*out).parent),
                            file,
                            line,
                            severity,
                            err,
                        )
                    }
                }
                pub fn parent(self: Pin<&mut Self>) -> Pin<&mut $name> {
                    unsafe { self.map_unchecked_mut(|x| &mut x.parent) }
                }
                pub fn stream(self: Pin<&mut Self>) -> NativeOStream<'_> {
                    self.parent().stream()
                }
            }
            impl Drop for $fatal {
                fn drop(&mut self) {
                    unsafe {
                        Pin::new_unchecked(&mut self.parent).AppendError();
                        Pin::new_unchecked(&mut self.parent.parent).Flush();
                        crate::base::immediate_crash::ImmediateCrash();
                    }
                }
            }
        };
    }
    #[cfg(windows)]
    error_owner!(Win32ErrorLogMessage, Win32ErrorLogMessageFatal);
    #[cfg(any(unix, target_os = "fuchsia"))]
    error_owner!(ErrnoLogMessage, ErrnoLogMessageFatal);
    // Source: base/logging.cc:1270-1291
    pub unsafe fn BuildCrashString_at(
        mut file: *const u8,
        line: i32,
        message: &[u8],
        out: *mut ConfiguredStdString,
    ) {
        unsafe {
            if !file.is_null() {
                let slash = libc::strrchr(
                    file.cast(),
                    if cfg!(windows) {
                        b'\\' as i32
                    } else {
                        b'/' as i32
                    },
                );
                if !slash.is_null() {
                    file = slash.add(1).cast();
                }
            }
            // The native StringPrintf string_view adapter belongs to stringprintf, not a
            // Rust formatter. Its explicit bytes/length preserve the original argument.
            crate::base::strings::stringprintf::BuildCrashStringFormat_at(out, file, line, message);
        }
    }
    // Source: base/logging.cc:431-444
    #[cfg(windows)]
    type SetLogFatalCrashKeyFunc = unsafe extern "C" fn(*const u8, i32, *const u8);
    #[cfg(windows)]
    unsafe extern "C" {
        // Actual Win SDK GetProcAddress/GetModuleHandle and reinterpret_cast primitive;
        // source supplied to the interop steward, no guessed FARPROC Rust ABI.
        fn rechrom_logging_resolve_bootstrap_fatal_key() -> Option<SetLogFatalCrashKeyFunc>;
    }
    #[cfg(windows)]
    unsafe fn SetLogFatalCrashKeyFuncGetter() -> Option<SetLogFatalCrashKeyFunc> {
        use crate::base::no_destructor::NativeLocalStatic;
        static FUNC: NativeLocalStatic<Option<SetLogFatalCrashKeyFunc>> =
            NativeLocalStatic::uninitialized();
        unsafe {
            *FUNC.get_or_initialize(|out| out.write(rechrom_logging_resolve_bootstrap_fatal_key()))
        }
    }
    // Source: base/logging.cc:446-474
    unsafe fn SetLogFatalCrashKey(message: Pin<&LogMessage>) {
        unsafe {
            #[cfg(windows)]
            if let Some(func) = SetLogFatalCrashKeyFuncGetter() {
                let mut text = MaybeUninit::<ConfiguredStdString>::uninit();
                message.str_at(text.as_mut_ptr());
                func(
                    message.file(),
                    message.line(),
                    Pin::new_unchecked(text.assume_init_ref())
                        .c_str()
                        .cast::<u8>()
                        .add(message.message_start()),
                );
                ptr::drop_in_place(text.as_mut_ptr());
                return;
            }
            // Exact native non-thread-safe reentrancy guard, independent of the
            // thread-safe compiler initialization guard on the static key below.
            static mut GUARDED: bool = false;
            if GUARDED {
                return;
            }
            struct AutoReset {
                old: bool,
            }
            impl Drop for AutoReset {
                fn drop(&mut self) {
                    unsafe {
                        GUARDED = self.old;
                    }
                }
            }
            let guard = AutoReset { old: GUARDED };
            GUARDED = true;
            use crate::base::{
                debug::crash_logging::{
                    AllocateCrashKeyString, CrashKeySize, CrashKeyString, SetCrashKeyString,
                },
                no_destructor::NativeLocalStatic,
            };
            static KEY: NativeLocalStatic<*mut CrashKeyString> = NativeLocalStatic::uninitialized();
            let crash_key = *KEY.get_or_initialize(|out| {
                out.write(AllocateCrashKeyString(
                    c"LOG_FATAL".as_ptr(),
                    CrashKeySize::Size1024,
                ))
            });
            let mut text = MaybeUninit::<ConfiguredStdString>::uninit();
            message.BuildCrashString_at(text.as_mut_ptr());
            SetCrashKeyString(crash_key, Pin::new_unchecked(text.assume_init_ref()).view());
            ptr::drop_in_place(text.as_mut_ptr());
            drop(guard);
        }
    }
}
#[cfg(sk_native_ostream_abi_configured)]
pub use owners::*;
// Source: base/logging.cc:196-203
fn log_severity_name(severity: i32) -> *const u8 {
    match severity {
        0 => b"INFO\0".as_ptr(),
        1 => b"WARNING\0".as_ptr(),
        2 => b"ERROR\0".as_ptr(),
        3 => b"FATAL\0".as_ptr(),
        _ => b"UNKNOWN\0".as_ptr(),
    }
}
// Source: base/logging.cc:254-273
unsafe fn TickCount() -> u64 {
    unsafe {
        #[cfg(windows)]
        {
            crate::win::GetTickCount() as u64
        }
        #[cfg(target_os = "fuchsia")]
        {
            (crate::zircon::syscalls::zx_clock_get_monotonic() / 1000) as u64
        }
        #[cfg(target_vendor = "apple")]
        {
            unsafe extern "C" {
                fn mach_absolute_time() -> u64;
            }
            mach_absolute_time()
        }
        #[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "fuchsia")))]
        {
            let mut ts = core::mem::MaybeUninit::<libc::timespec>::uninit();
            libc::clock_gettime(libc::CLOCK_MONOTONIC, ts.as_mut_ptr());
            let ts = ts.assume_init();
            (ts.tv_sec as u64) * 1000000 + (ts.tv_nsec as u64) / 1000
        }
    }
}

// Source: base/logging.h:280-285
// A native RepeatingCallback is copyable shared callable ownership. This Arc is
// that callback owner, not an alternative LogMessage or sink provider.
pub type LogAssertHandlerFunction = Option<std::sync::Arc<dyn Fn(*const u8, i32, &[u8], &[u8])>>;
// Source: base/logging.cc:246-249
// No extra mutex: native caller synchronization/non-reentrant stack discipline
// remains necessary. The static Vec preserves NoDestructor lifetime.
static mut LOG_ASSERT_HANDLER_STACK: Vec<LogAssertHandlerFunction> = Vec::new();
// Source: base/logging.h:287-293
pub struct ScopedLogAssertHandler {
    _not_copy: core::marker::PhantomData<std::rc::Rc<()>>,
}
impl ScopedLogAssertHandler {
    // Source: base/logging.cc:674-677
    /// Safety: caller synchronizes access to the native global stack and destroys
    /// scoped handlers in LIFO order. No concurrent/reentrant mutation is permitted.
    pub unsafe fn new(handler: LogAssertHandlerFunction) -> Self {
        unsafe {
            (&mut *core::ptr::addr_of_mut!(LOG_ASSERT_HANDLER_STACK)).push(handler);
        }
        Self {
            _not_copy: core::marker::PhantomData,
        }
    }
}
// Source: base/logging.cc:679-681
impl Drop for ScopedLogAssertHandler {
    fn drop(&mut self) {
        unsafe {
            (&mut *core::ptr::addr_of_mut!(LOG_ASSERT_HANDLER_STACK)).pop();
        }
    }
}

// Source: base/logging.cc:227-231
#[cfg(unix)]
static mut g_log_file_name: Option<Vec<u8>> = None;
#[cfg(unix)]
static mut g_log_file: *mut libc::FILE = core::ptr::null_mut();
// Source: base/logging.cc:285-315 (POSIX branch)
#[cfg(unix)]
fn GetDefaultLogFile() -> Vec<u8> {
    b"debug.log\0".to_vec()
}
// Source: base/logging.cc:328-390
#[cfg(unix)]
/// Safety: caller holds the actual native logging lock; filename includes NUL.
unsafe fn InitializeLogFileHandle() -> bool {
    unsafe {
        if !g_log_file.is_null() {
            return true;
        }
        let filename = &mut *core::ptr::addr_of_mut!(g_log_file_name);
        if filename.is_none() {
            *filename = Some(GetDefaultLogFile());
        }
        if g_logging_destination & LOG_TO_FILE == 0 {
            return true;
        }
        g_log_file = libc::fopen(
            filename.as_ref().unwrap().as_ptr().cast(),
            b"a\0".as_ptr().cast(),
        );
        if g_log_file.is_null() {
            return false;
        }
        true
    }
}
// Source: base/logging.cc:392-400
#[cfg(unix)]
unsafe fn CloseFile(log: *mut libc::FILE) {
    unsafe {
        libc::fclose(log);
    }
}
// Source: base/logging.cc:402-415
#[cfg(unix)]
unsafe fn CloseLogFileUnlocked() {
    unsafe {
        if g_log_file.is_null() {
            return;
        }
        CloseFile(g_log_file);
        g_log_file = core::ptr::null_mut();
        if (*core::ptr::addr_of!(g_log_file_name)).is_none() {
            g_logging_destination &= !LOG_TO_FILE;
        }
    }
}
// Source: base/logging.cc:417-429
#[cfg(any(target_vendor = "apple", target_os = "linux"))]
unsafe fn WriteToFd(fd: i32, data: *const u8, length: usize) {
    let mut bytes_written = 0;
    while bytes_written < length {
        let rv = loop {
            let rv =
                unsafe { libc::write(fd, data.add(bytes_written).cast(), length - bytes_written) };
            if rv < 0 && unsafe { *crate::base::strings::stringprintf::errno_ptr() } == libc::EINTR
            {
                continue;
            }
            break rv;
        };
        if rv < 0 {
            break;
        }
        bytes_written += rv as usize;
    }
}
// Source: base/logging.cc:691-717 (native deliberately has no non-Windows UI)
#[cfg(all(not(sk_native_ndebug), not(windows)))]
unsafe fn DisplayDebugMessageInDialog(str: &[u8]) {
    if str.is_empty() {
        return;
    }
    if !unsafe { show_error_dialogs } {
        return;
    }
}
// Source: base/logging.cc:1244-1264
#[cfg(any(target_vendor = "apple", target_os = "linux"))]
/// Safety: message is null or a live NUL-terminated native string; logging global
/// configuration has the same external synchronization requirement as Chromium.
pub unsafe fn RawLog(level: i32, message: *const u8) {
    unsafe {
        if level >= g_min_log_level && !message.is_null() {
            let bytes = CStr::from_ptr(message.cast()).to_bytes();
            WriteToFd(libc::STDERR_FILENO, message, bytes.len());
            if !bytes.is_empty() && bytes[bytes.len() - 1] != b'\n' {
                loop {
                    let rv = loop {
                        let rv = libc::write(libc::STDERR_FILENO, b"\n".as_ptr().cast(), 1);
                        if rv < 0 && *crate::base::strings::stringprintf::errno_ptr() == libc::EINTR
                        {
                            continue;
                        }
                        break rv;
                    };
                    if rv < 0 || rv == 1 {
                        break;
                    }
                }
            }
        }
        if level == LOGGING_FATAL {
            crate::base::immediate_crash::ImmediateCrash();
        }
    }
}

// Source: base/logging.cc:800-817
// The native local class owns only the os_log_create retain. The global default
// is borrowed and must never be released. The SDK macro call at 836 remains a
// separately compiled actual SDK bridge below; this owner does not replace it.
#[cfg(target_vendor = "apple")]
mod apple_os_log {
    unsafe extern "C" {
        pub type os_log_s;
        static mut _os_log_default: os_log_s;
        fn os_log_create(
            subsystem: *const core::ffi::c_char,
            category: *const core::ffi::c_char,
        ) -> *mut os_log_s;
        fn os_release(object: *mut core::ffi::c_void);
        // External binding defined in the staged Apple SDK bridge TU.
        // Its type byte is SDK os_log_type_t (enum underlying uint8_t).
        fn skia_log_os_log_public_string(
            log: *mut os_log_s,
            r#type: u8,
            message: *const core::ffi::c_char,
        );
    }
    pub(super) struct OSLog {
        os_log_: *mut os_log_s,
    }
    impl OSLog {
        pub(super) unsafe fn new(subsystem: *const u8) -> Self {
            let os_log_ = if subsystem.is_null() {
                core::ptr::addr_of_mut!(_os_log_default)
            } else {
                unsafe { os_log_create(subsystem.cast(), b"chromium_logging\0".as_ptr().cast()) }
            };
            Self { os_log_ }
        }
        pub(super) fn get(&self) -> *mut os_log_s {
            self.os_log_
        }
        // Source: base/logging.cc:835-836
        /// Safety: message is the original live NUL-terminated c_str() and its
        /// owner remains live through the synchronous SDK call. No retained
        /// pointer or new string owner is manufactured by this boundary.
        pub(super) unsafe fn write_public_string(&self, r#type: u8, message: *const u8) {
            unsafe {
                skia_log_os_log_public_string(self.get(), r#type, message.cast());
            }
        }
    }
    impl Drop for OSLog {
        fn drop(&mut self) {
            if self.os_log_ != core::ptr::addr_of_mut!(_os_log_default) {
                unsafe {
                    os_release(self.os_log_.cast());
                }
            }
        }
    }
    // Source: base/logging.cc:818-834; SDK os/log.h:84-91
    pub(super) fn log_type(severity: i32) -> u8 {
        match severity {
            super::LOGGING_INFO => 0x01,
            super::LOGGING_WARNING => 0,
            super::LOGGING_ERROR => 0x10,
            super::LOGGING_FATAL => 0x11,
            -1 => 0x02,
            _ => 0,
        }
    }
}

// Source: base/logging.cc:492-503
// Genuine synchronous native typed producer. The canonical module is supplied
// by the separate Chromium/Perfetto bridge group; no disabled/fallback version
// of this helper is emitted here. This forwarding boundary has no Skia credit.
/// Safety: file is the original stable native source-name pointer, whose identity
/// and NUL-terminated storage survive tracing interning. message is borrowed for
/// this call; its complete byte length, including embedded NULs, is preserved.
unsafe fn TraceLogMessage(file: *const u8, line: i32, message: &[u8]) {
    unsafe {
        crate::base::trace_event::typed_macros::TraceLogMessage(file, line, message);
    }
}

// Source: base/logging.cc:318-321
#[cfg(any(unix, target_os = "fuchsia"))]
unsafe fn GetLoggingLock() -> core::pin::Pin<&'static crate::base::synchronization::lock::Lock> {
    use crate::base::{
        no_destructor::{NativeLocalStatic, NoDestructor},
        synchronization::lock::Lock,
    };
    // This one guard/storage belongs only to this native declaration.
    static LOCK: NativeLocalStatic<NoDestructor<Lock>> = NativeLocalStatic::uninitialized();
    unsafe {
        let p = LOCK.get_or_initialize(|out| {
            NoDestructor::construct_at(out, |lock| {
                Lock::new(core::pin::Pin::new_unchecked(
                    &mut *lock.cast::<core::mem::MaybeUninit<Lock>>(),
                ));
            })
        });
        core::pin::Pin::new_unchecked(&*p).get()
    }
}
