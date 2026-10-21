// Source: base/threading/platform_thread_posix.cc. Actual pthread/syscall operations.
#![cfg(any(base_is_posix, base_is_fuchsia))]
#![allow(non_snake_case, non_upper_case_globals)]
use super::platform_thread::{
    Delegate, MessagePumpType, PlatformThreadBase, PlatformThreadHandle, PlatformThreadId,
    ThreadType,
};
use super::platform_thread_ref::PlatformThreadRef;
use crate::base::time::time::{Seconds, TimeDelta};
use core::{
    ffi::{c_int, c_void},
    mem::MaybeUninit,
    ptr::NonNull,
};
unsafe extern "C" {
    fn rechrom_platform_thread_errno_location() -> *mut c_int;
}
// Source: platform_thread_posix.cc:57-59 declares these genuine base functions.
// Their actual OS implementation TU is not among the three named inputs:
// InitThreading(); TerminateOnThread(); GetDefaultThreadStackSize(const pthread_attr_t&).
// Calls below retain those exact unresolved native definitions rather than empty helpers.
struct ThreadParams<'a> {
    delegate: Option<NonNull<dyn Delegate + 'a>>,
    joinable: bool,
    thread_type: ThreadType,
    message_pump_type: MessagePumpType,
}
impl ThreadParams<'_> {
    fn new() -> Self {
        Self {
            delegate: None,
            joinable: false,
            thread_type: ThreadType::kDefault,
            message_pump_type: MessagePumpType::DEFAULT,
        }
    }
}
// Only the native pthread publication path calls this private C ABI entry.
// The caller's unsafe lease, not a fabricated 'static or Send/Sync bound,
// keeps the original delegate allocation/captures alive on the created thread.
extern "C" fn ThreadFunc<'a>(params: *mut c_void) -> *mut c_void {
    unsafe {
        let delegate;
        {
            let thread_params = Box::from_raw(params.cast::<ThreadParams<'a>>());
            delegate = thread_params.delegate.unwrap_unchecked();
            if !thread_params.joinable {
                crate::base::threading::thread_restrictions::DisallowSingleton();
            }
            #[cfg(base_pa_use_partition_alloc_as_malloc)]
            crate::partition_alloc::stack::StackTopRegistry::Get().NotifyThreadCreated();
            #[cfg(base_is_apple)]
            PlatformThreadBase::SetCurrentThreadRealtimePeriodValue(
                delegate.as_ref().GetRealtimePeriod(),
            );
            PlatformThreadBase::SetDefaultThreadType(thread_params.thread_type);
        } // Original ThreadParams is deleted before registration/ThreadMain.
        let manager =
            crate::base::threading::thread_id_name_manager::ThreadIdNameManager::GetInstance();
        (*manager).RegisterThread(
            PlatformThreadBase::CurrentHandle().platform_handle(),
            PlatformThreadBase::CurrentId(),
        );
        delegate.as_ref().ThreadMain();
        let manager =
            crate::base::threading::thread_id_name_manager::ThreadIdNameManager::GetInstance();
        (*manager).RemoveName(
            PlatformThreadBase::CurrentHandle().platform_handle(),
            PlatformThreadBase::CurrentId(),
        );
        #[cfg(base_is_android)]
        crate::base::threading::platform_thread_metrics::PlatformThreadPriorityMonitor::Get()
            .UnregisterCurrentThread();
        #[cfg(base_pa_use_partition_alloc_as_malloc)]
        crate::partition_alloc::stack::StackTopRegistry::Get().NotifyThreadDestroyed();
        TerminateOnThread();
        core::ptr::null_mut()
    }
}
/// Safety: original delegate address/lifetime and all cross-thread accesses obey
/// native caller preconditions; unchecked SDK initializer failures retain their
/// original initialized-output precondition, with no manufactured success value.
unsafe fn CreateThread<'a>(
    mut stack_size: usize,
    joinable: bool,
    delegate: NonNull<dyn Delegate + 'a>,
    thread_handle: &mut PlatformThreadHandle,
    thread_type: ThreadType,
    message_pump_type: MessagePumpType,
) -> bool {
    unsafe {
        // Native DCHECK(thread_handle) is guaranteed by this nonnull Rust output reference.
        InitThreading();
        let mut attributes = MaybeUninit::<libc::pthread_attr_t>::uninit();
        libc::pthread_attr_init(attributes.as_mut_ptr());
        if !joinable {
            libc::pthread_attr_setdetachstate(
                attributes.as_mut_ptr(),
                libc::PTHREAD_CREATE_DETACHED,
            );
        }
        if stack_size == 0 {
            stack_size = GetDefaultThreadStackSize(attributes.as_ptr());
        }
        if stack_size > 0 {
            libc::pthread_attr_setstacksize(attributes.as_mut_ptr(), stack_size);
        }
        let mut params = Box::new(ThreadParams::new());
        params.delegate = Some(delegate);
        params.joinable = joinable;
        params.thread_type = thread_type;
        params.message_pump_type = message_pump_type;
        // Transfer to raw publication before pthread_create: the new thread may delete
        // this payload before pthread_create returns. Failure reclaims exactly the same
        // allocation and defers its destruction until after pthread_attr_destroy.
        let params = Box::into_raw(params);
        let mut handle = MaybeUninit::<libc::pthread_t>::uninit();
        let err = libc::pthread_create(
            handle.as_mut_ptr(),
            attributes.as_ptr(),
            ThreadFunc,
            params.cast(),
        );
        let success = err == 0;
        let failed_params;
        if success {
            failed_params = None;
        } else {
            failed_params = Some(Box::from_raw(params));
            handle.as_mut_ptr().write(0 as libc::pthread_t);
            *rechrom_platform_thread_errno_location() = err;
            if crate::base::logging::ShouldCreateLogMessage(crate::base::logging::LOGGING_ERROR) {
                // Actual ErrnoLogMessage ctor/stream/dtor are required; no plain RawLog or
                // successful no-op substitutes the pending owned PLOG temporary.
                let mut message = crate::base::logging::ErrnoLogMessage::new(
                    b"base/threading/platform_thread_posix.cc\0".as_ptr(),
                    162,
                    crate::base::logging::LOGGING_ERROR,
                    crate::base::logging::GetLastSystemErrorCode(),
                );
                let mut stream = message.stream();
                stream.insert_cstr(b"pthread_create\0".as_ptr());
                drop(stream);
                drop(message);
            }
        }
        *thread_handle = PlatformThreadHandle::new(handle.assume_init());
        libc::pthread_attr_destroy(attributes.as_mut_ptr());
        drop(failed_params);
        success
    }
}
#[cfg(any(base_is_linux, base_is_chromeos))]
mod tid_cache {
    use core::{
        cell::UnsafeCell,
        sync::atomic::{AtomicBool, Ordering},
    };
    // Native trivial TLS has no destructor; no std::thread ID or hash is involved.
    std::thread_local! {pub(super) static g_thread_id:UnsafeCell<libc::pid_t>=const{UnsafeCell::new(-1)};pub(super) static g_is_main_thread:UnsafeCell<bool>=const{UnsafeCell::new(true)};}
    pub(super) static g_main_thread_tid_cache_valid: AtomicBool = AtomicBool::new(false);
    pub(super) static INIT: std::sync::Once = std::sync::Once::new();
    pub(super) extern "C" fn InvalidateTidCache() {
        g_main_thread_tid_cache_valid.store(false, Ordering::Relaxed);
    }
}
#[cfg(any(base_is_linux, base_is_chromeos))]
pub fn InvalidateTidCache() {
    tid_cache::InvalidateTidCache()
}
#[cfg(MEMORY_SANITIZER)]
unsafe extern "C" {
    fn __msan_unpoison(p: *const c_void, size: usize);
}
impl PlatformThreadBase {
    // Source: native CurrentId branches use their actual SDK value types.
    #[cfg(base_is_apple)]
    pub fn CurrentId() -> PlatformThreadId {
        let mut tid = MaybeUninit::<u64>::uninit();
        crate::CHECK_EQ!(@location crate::FROM_HERE!(@function b"CurrentId\0".as_ptr().cast();@file b"base/threading/platform_thread_posix.cc\0".as_ptr().cast();@line 224);@condition b"pthread_threadid_np(nullptr, &tid) == 0\0".as_ptr().cast();unsafe{libc::pthread_threadid_np(core::ptr::null_mut(),tid.as_mut_ptr())},0);
        PlatformThreadId::new(unsafe { tid.assume_init() })
    }
    #[cfg(any(base_is_linux, base_is_chromeos))]
    pub fn CurrentId() -> PlatformThreadId {
        use core::sync::atomic::Ordering;
        tid_cache::g_thread_id.with(|tid|tid_cache::g_is_main_thread.with(|main|unsafe{
  #[cfg(MEMORY_SANITIZER)]{__msan_unpoison(tid.get().cast(),core::mem::size_of::<libc::pid_t>());__msan_unpoison(main.get().cast(),core::mem::size_of::<bool>());}
  tid_cache::INIT.call_once_force(|_|{libc::pthread_atfork(None,None,Some(tid_cache::InvalidateTidCache));});
  if *tid.get()==-1||(*main.get()&&!tid_cache::g_main_thread_tid_cache_valid.load(Ordering::Relaxed)){
   *tid.get()=libc::syscall(libc::SYS_gettid) as libc::pid_t;
   if *tid.get()==libc::getpid(){tid_cache::g_main_thread_tid_cache_valid.store(true,Ordering::Relaxed);}else{*main.get()=false;}
  }else{
   #[cfg(any(not(sk_native_ndebug),DCHECK_ALWAYS_ON))]
   if *tid.get() as libc::c_long!=libc::syscall(libc::SYS_gettid){crate::base::logging::RawLog(crate::base::logging::LOGGING_FATAL,b"Thread id stored in TLS is different from thread id returned by the system. It is likely that the process was forked without going through fork().\0".as_ptr());}
  }
  PlatformThreadId::new(*tid.get())
 }))
    }
    #[cfg(base_is_android)]
    pub fn CurrentId() -> PlatformThreadId {
        PlatformThreadId::new(unsafe { libc::gettid() })
    }
    // Fuchsia koid and other POSIX SDK constructors retain exact pending source
    // branches; no cast to a guessed pid_t or Windows/Fuchsia identity provider.
    pub fn CurrentRef() -> PlatformThreadRef {
        PlatformThreadRef::from_id(unsafe { libc::pthread_self() })
    }
    pub fn CurrentHandle() -> PlatformThreadHandle {
        PlatformThreadHandle::new(unsafe { libc::pthread_self() })
    }
    #[cfg(not(base_is_apple))]
    pub fn YieldCurrentThread() {
        unsafe {
            libc::sched_yield();
        }
    }
    /// Safety: native duration decomposition/static casts and ordinary signed
    /// arithmetic are representable; invalid nanosleep input is not repaired.
    pub unsafe fn Sleep(mut duration: TimeDelta) {
        unsafe {
            let mut sleep_time = MaybeUninit::<libc::timespec>::uninit();
            let mut remaining = MaybeUninit::<libc::timespec>::uninit();
            let sec = duration.InSeconds() as libc::time_t;
            core::ptr::addr_of_mut!((*sleep_time.as_mut_ptr()).tv_sec).write(sec);
            duration -= Seconds(sec);
            core::ptr::addr_of_mut!((*sleep_time.as_mut_ptr()).tv_nsec)
                .write(duration.InMicroseconds().unchecked_mul(1000) as libc::c_long);
            while libc::nanosleep(sleep_time.as_ptr(), remaining.as_mut_ptr()) == -1
                && *rechrom_platform_thread_errno_location() == libc::EINTR
            {
                core::ptr::copy_nonoverlapping(remaining.as_ptr(), sleep_time.as_mut_ptr(), 1);
            }
        }
    }
    pub unsafe fn GetName() -> *const libc::c_char {
        unsafe {
            let manager =
                crate::base::threading::thread_id_name_manager::ThreadIdNameManager::GetInstance();
            (*manager).GetName(Self::CurrentId())
        }
    }
    pub unsafe fn CreateWithType<'a>(
        stack: usize,
        delegate: NonNull<dyn Delegate + 'a>,
        handle: &mut PlatformThreadHandle,
        thread_type: ThreadType,
        pump: MessagePumpType,
    ) -> bool {
        unsafe { CreateThread(stack, true, delegate, handle, thread_type, pump) }
    }
    pub unsafe fn CreateNonJoinable<'a>(
        stack: usize,
        delegate: NonNull<dyn Delegate + 'a>,
    ) -> bool {
        unsafe {
            Self::CreateNonJoinableWithType(
                stack,
                delegate,
                ThreadType::kDefault,
                MessagePumpType::DEFAULT,
            )
        }
    }
    pub unsafe fn CreateNonJoinableWithType<'a>(
        stack: usize,
        delegate: NonNull<dyn Delegate + 'a>,
        thread_type: ThreadType,
        pump: MessagePumpType,
    ) -> bool {
        let mut unused = PlatformThreadHandle::null();
        unsafe { CreateThread(stack, false, delegate, &mut unused, thread_type, pump) }
    }
    pub unsafe fn Join(handle: PlatformThreadHandle) {
        unsafe {
            let _scoped=crate::base::threading::scoped_blocking_call::ScopedBlockingCallWithBaseSyncPrimitives::new(crate::FROM_HERE!(@function b"Join\0".as_ptr().cast();@file b"base/threading/platform_thread_posix.cc\0".as_ptr().cast();@line 351),crate::base::threading::scoped_blocking_call::BlockingType::MAY_BLOCK);
            crate::CHECK_EQ!(@location crate::FROM_HERE!(@function b"Join\0".as_ptr().cast();@file b"base/threading/platform_thread_posix.cc\0".as_ptr().cast();@line 352);@condition b"0 == pthread_join(thread_handle.platform_handle(), nullptr)\0".as_ptr().cast();0,libc::pthread_join(handle.platform_handle(),core::ptr::null_mut()));
        }
    }
    pub unsafe fn Detach(handle: PlatformThreadHandle) {
        crate::CHECK_EQ!(@location crate::FROM_HERE!(@function b"Detach\0".as_ptr().cast();@file b"base/threading/platform_thread_posix.cc\0".as_ptr().cast();@line 357);@condition b"0 == pthread_detach(thread_handle.platform_handle())\0".as_ptr().cast();0,unsafe{libc::pthread_detach(handle.platform_handle())});
    }
    #[cfg(not(any(base_is_apple, base_is_fuchsia)))]
    pub unsafe fn CanChangeThreadType(from: ThreadType, to: ThreadType) -> bool {
        if from >= to {
            return true;
        }
        unsafe {
            if to == ThreadType::kRealtimeAudio {
                return super::platform_thread_internal_posix::CanSetThreadTypeToRealtimeAudio();
            }
            crate::base::posix::can_lower_nice_to::CanLowerNiceTo(
                super::platform_thread_internal_posix::ThreadTypeToNiceValue(to),
            )
        }
    }
    #[cfg(not(any(base_is_apple, base_is_fuchsia)))]
    pub unsafe fn GetCurrentEffectiveThreadTypeForTest() -> ThreadType {
        unsafe {
            if let Some(priority)=super::platform_thread_internal_posix::GetCurrentEffectiveThreadTypeForPlatformForTest(){return priority}
            let nice = super::platform_thread_internal_posix::GetCurrentThreadNiceValue();
            super::platform_thread_internal_posix::NiceValueToThreadTypeForTest(nice)
        }
    }
    pub unsafe fn GetDefaultThreadStackSize() -> usize {
        unsafe {
            let mut attributes = MaybeUninit::<libc::pthread_attr_t>::uninit();
            libc::pthread_attr_init(attributes.as_mut_ptr());
            GetDefaultThreadStackSize(attributes.as_ptr())
        }
    }
}
