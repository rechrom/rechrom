// Source: base/debug/task_trace.h; base/debug/task_trace.cc
use super::stack_trace::StackTrace;
use crate::base::strings::native_ostream::{NativeOStream, StdStringStorage};
use core::ffi::c_void;
// Native owning member order reversed for parent/member destruction; no extra
// holder or synthetic task context. StackTrace itself is the actual pointer array.
#[derive(Clone, Copy)]
pub struct TaskTrace {
    trace_overflow_: bool,
    stack_trace_: Option<StackTrace>,
}
impl TaskTrace {
    /// Requires the actual TaskAnnotator TLS and PendingTask definitions/producer.
    /// This does not create another TLS task pointer or accept a substitute provider.
    // Source: base/debug/task_trace.cc:48
    pub unsafe fn new() -> Self {
        use crate::base::{
            pending_task::TaskMetadata, task::common::task_annotator::TaskAnnotator,
        };
        let mut result = Self {
            stack_trace_: None,
            trace_overflow_: false,
        };
        let current_task = unsafe { TaskAnnotator::CurrentTaskForThread() };
        if current_task.is_null() {
            return result;
        }
        let current_task = unsafe { &*current_task };
        let mut task_trace = [core::ptr::null::<c_void>(); TaskMetadata::kTaskBacktraceLength + 1];
        task_trace[0] = current_task.posted_from.program_counter();
        task_trace[1..].copy_from_slice(&current_task.task_backtrace);
        let mut length = 0;
        while length < task_trace.len() && !task_trace[length].is_null() {
            length += 1;
        }
        if length == 0 {
            return result;
        }
        result.stack_trace_ = Some(StackTrace::from_addresses(&task_trace[..length]));
        result.trace_overflow_ = current_task.task_backtrace_overflow;
        result
    }
    // Source: base/debug/task_trace.cc:67
    pub fn empty(&self) -> bool {
        self.stack_trace_.is_none()
    }
    // Source: base/debug/task_trace.cc:75
    pub unsafe fn OutputToStream<S: StdStringStorage>(&self, os: &mut NativeOStream<'_>) {
        unsafe {
            os.insert_cstr(c"Task trace:".as_ptr().cast());
        }
        os.endl();
        let Some(stack) = &self.stack_trace_ else {
            unsafe {
                os.insert_cstr(c"No active task.".as_ptr().cast());
            }
            return;
        };
        unsafe {
            super::stack_trace::insert_stack_trace::<S>(os, stack);
        }
        if self.trace_overflow_ {
            unsafe {
                os.insert_cstr(c"Task trace buffer limit hit, update PendingTask::kTaskBacktraceLength to increase.".as_ptr().cast());
            }
            os.endl();
        }
    }
    // Source: base/debug/task_trace.cc:95
    pub fn GetAddresses(&self, addresses: &mut [*const c_void]) -> usize {
        if self.empty() {
            return 0;
        }
        let current_addresses = self.stack_trace_.as_ref().unwrap().addresses();
        let copy_size = current_addresses.len().min(addresses.len());
        addresses[..copy_size].copy_from_slice(&current_addresses[..copy_size]);
        current_addresses.len()
    }
}
pub unsafe fn insert_task_trace<S: StdStringStorage>(
    os: &mut NativeOStream<'_>,
    task_trace: &TaskTrace,
) {
    unsafe {
        task_trace.OutputToStream::<S>(os);
    }
}
// Print/DefaultOutputStream and ToString require actual native cerr/Android
// stringbuf and stringstream ABI storage; no host stderr/byte-vector replacement.
unsafe extern "C" {
    fn rechrom_debug_task_default_ostream() -> *mut c_void;
}
impl TaskTrace {
    /// The real native default stream is synchronously borrowed under its actual
    /// native synchronization/alias protocol. No Rust stderr substitute is used.
    // Source: base/debug/task_trace.cc:71
    pub unsafe fn Print<S: StdStringStorage>(&self) {
        unsafe {
            let raw = core::ptr::NonNull::new_unchecked(rechrom_debug_task_default_ostream());
            let mut os = NativeOStream::from_native(raw);
            self.OutputToStream::<S>(&mut os);
        }
    }
    // Source: base/debug/task_trace.cc:89
    pub unsafe fn ToString_at<
        SS: crate::base::strings::native_ostream::StringStreamStorage,
        DS: StdStringStorage,
    >(
        &self,
        dst: *mut crate::base::strings::native_ostream::NativeStdString<DS>,
    ) {
        use crate::base::strings::native_ostream::NativeStringStream;
        let mut stream = core::mem::MaybeUninit::<NativeStringStream<SS>>::uninit();
        unsafe {
            NativeStringStream::initialize_at_assuming_native_abi(stream.as_mut_ptr());
        }
        let mut pinned = unsafe { core::pin::Pin::new_unchecked(stream.assume_init_mut()) };
        unsafe {
            self.OutputToStream::<DS>(&mut pinned.as_mut().stream());
            pinned.as_mut().take_str_at(dst);
            core::ptr::drop_in_place(stream.as_mut_ptr());
        }
    }
}
