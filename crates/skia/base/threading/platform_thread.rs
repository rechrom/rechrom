// Source: base/threading/platform_thread.h, actual value types and static class interface.
#![allow(non_snake_case, non_upper_case_globals)]
pub use super::platform_thread_ref::PlatformThreadRef;
pub use crate::base::message_loop::message_pump_type::MessagePumpType;
pub use crate::base::task::thread_type::ThreadType;
use crate::base::time::time::TimeDelta;
#[cfg(base_is_win)]
pub type UnderlyingType = libc::c_ulong;
#[cfg(base_is_fuchsia)]
pub type UnderlyingType = crate::zircon::types::zx_koid_t;
#[cfg(base_is_apple)]
pub type UnderlyingType = u64;
#[cfg(all(base_is_posix, not(base_is_apple)))]
pub type UnderlyingType = libc::pid_t;
// Source: platform_thread.h:76-135. Only the exact underlying type constructor
// exists; no narrowing Into<i32>/Into<u32> conversion is published.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PlatformThreadId {
    value_: UnderlyingType,
}
impl PlatformThreadId {
    pub const fn new(value: UnderlyingType) -> Self {
        Self { value_: value }
    }
    pub const fn null() -> Self {
        Self { value_: 0 }
    }
    pub const fn ForTest(value: libc::c_int) -> Self {
        Self {
            value_: value as UnderlyingType,
        }
    }
    pub const fn raw(self) -> UnderlyingType {
        self.value_
    }
    pub const fn as_u64(self) -> u64 {
        self.value_ as u64
    }
    pub const fn as_i64(self) -> i64 {
        self.value_ as i64
    }
    pub const fn truncate_to_int32_for_display_only(self) -> i32 {
        self.value_ as i32
    }
    // Native inline ostream body consumes the actual native underlying overload.
    pub fn WriteToStream(&self, stream: &mut crate::interop::native_ostream::NativeOStream<'_>) {
        unsafe { rechrom_platform_thread_id_ostream(stream.as_native_mut_ptr(), self.as_u64()) }
    }
}
impl Default for PlatformThreadId {
    fn default() -> Self {
        Self::null()
    }
}
unsafe extern "C" {
    fn rechrom_platform_thread_id_ostream(stream: *mut core::ffi::c_void, bits: u64);
}
const _: () = assert!(core::mem::size_of::<UnderlyingType>() <= core::mem::size_of::<u64>());
// Source: platform_thread.h:145-176. Handle owns no destructor: Join/Detach are explicit.
#[cfg(base_is_win)]
pub type Handle = *mut core::ffi::c_void;
#[cfg(any(base_is_posix, base_is_fuchsia))]
pub type Handle = libc::pthread_t;
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PlatformThreadHandle {
    handle_: Handle,
}
impl PlatformThreadHandle {
    pub const fn new(handle: Handle) -> Self {
        Self { handle_: handle }
    }
    pub const fn null() -> Self {
        Self {
            handle_: 0 as Handle,
        }
    }
    /// Safety: pthread identities retain the native validity/lease preconditions of pthread_equal.
    pub unsafe fn is_equal(&self, other: &Self) -> bool {
        #[cfg(any(base_is_posix, base_is_fuchsia))]
        {
            unsafe { libc::pthread_equal(self.handle_, other.handle_) != 0 }
        }
        #[cfg(base_is_win)]
        {
            self.handle_ == other.handle_
        }
    }
    pub fn is_null(&self) -> bool {
        self.handle_ == 0 as Handle
    }
    pub const fn platform_handle(self) -> Handle {
        self.handle_
    }
}
impl Default for PlatformThreadHandle {
    fn default() -> Self {
        Self::null()
    }
}
pub const kInvalidThreadId: PlatformThreadId = PlatformThreadId::null();
// Source: platform_thread.h:179-198. Implementations retain the native caller's
// cross-thread lifetime/synchronization obligation; no blanket Send or 'static.
pub trait Delegate {
    unsafe fn ThreadMain(&self);
    // Ordinary default virtual definition is outside the three admitted files.
    // Requiring a real implementation preserves that unresolved body, not a zero period.
    #[cfg(base_is_apple)]
    unsafe fn GetRealtimePeriod(&self) -> TimeDelta;
}
// Source: platform_thread.h:235-237. Deleted constructors become an uninhabited
// static namespace; no object or invented service provider can be constructed.
pub enum PlatformThreadBase {}
// Source: platform_thread.h:505-513. Static-only inherited API has one namespace;
// target-specific methods remain source-selected in their actual implementation TUs.
#[cfg(base_is_apple)]
pub type PlatformThreadApple = PlatformThreadBase;
#[cfg(any(base_is_linux, base_is_chromeos))]
pub type PlatformThreadLinux = PlatformThreadBase;
#[cfg(base_is_chromeos)]
pub type PlatformThreadChromeOS = PlatformThreadBase;
pub type PlatformThread = PlatformThreadBase;
impl PlatformThreadBase {
    /// Safety: delegate stays at its original address and alive through ThreadMain;
    /// its shared captures and all thread-local/native dependencies obey original synchronization.
    pub unsafe fn Create<'a>(
        stack_size: usize,
        delegate: core::ptr::NonNull<dyn Delegate + 'a>,
        handle: &mut PlatformThreadHandle,
    ) -> bool {
        unsafe {
            Self::CreateWithType(
                stack_size,
                delegate,
                handle,
                ThreadType::kDefault,
                MessagePumpType::DEFAULT,
            )
        }
    }
}
// Source: base/threading/platform_thread.h:479-515. Native default member initialization.
pub struct RaiseLeasesFields {
    pub(crate) leases: [u32; ThreadType::kMaxValue.0 as usize + 1],
    pub(crate) bitmask: u32,
}
impl RaiseLeasesFields {
    pub const fn new() -> Self {
        Self {
            leases: [0; ThreadType::kMaxValue.0 as usize + 1],
            bitmask: 0,
        }
    }
}
pub struct ThreadTypeManagerFields {
    pub(crate) default_thread_type_: Option<ThreadType>,
    pub(crate) effective_thread_type_: Option<ThreadType>,
    pub(crate) raise_leases_: RaiseLeasesFields,
}
impl ThreadTypeManagerFields {
    pub const fn new() -> Self {
        Self {
            default_thread_type_: None,
            effective_thread_type_: None,
            raise_leases_: RaiseLeasesFields::new(),
        }
    }
}
#[cfg(any(base_is_linux, base_is_chromeos))]
impl PlatformThreadBase {
    pub const kRealTimeAudioPrio: libc::sched_param = libc::sched_param { sched_priority: 8 };
    pub const kRealTimeDisplayPrio: libc::sched_param = libc::sched_param { sched_priority: 6 };
}

// Source: base/threading/platform_thread.h:479-515
// The native virtual slot is a Rust trait; the ordinary concrete manager owns
// exactly one original field set. UnsafeCell expresses native thread-confined
// mutation; it is not a lock or an extra manager. No Send/Sync implementation.
pub struct ThreadTypeManager {
    fields: core::cell::UnsafeCell<ThreadTypeManagerFields>,
}
impl ThreadTypeManager {
    pub const fn new() -> Self {
        Self {
            fields: core::cell::UnsafeCell::new(ThreadTypeManagerFields::new()),
        }
    }
}
// Native virtual default destructor has no body and all members are trivial.
// Rust's ordinary destruction suffices; a derived trait implementation retains
// its own concrete destructor, with no boxed/allocated manager adapter.
pub trait ThreadTypeManagerInterface {
    fn fields(&self) -> &core::cell::UnsafeCell<ThreadTypeManagerFields>;
    /// Safety: original current-thread scheduling and callback prerequisites apply.
    unsafe fn SetCurrentThreadTypeImpl(
        &self,
        thread_type: ThreadType,
        pump_type_hint: MessagePumpType,
    ) {
        unsafe { DefaultSetCurrentThreadTypeImpl(thread_type, pump_type_hint) }
    }
}
impl ThreadTypeManagerInterface for ThreadTypeManager {
    fn fields(&self) -> &core::cell::UnsafeCell<ThreadTypeManagerFields> {
        &self.fields
    }
}
// Source: base/threading/platform_thread.h:195-219
// Borrow is a native lease, not ownership of the manager. Caller must retain its
// address through lease destruction, remain on its original thread, and destroy
// every lease before that thread's TLS teardown. Rust moves transfer ownership;
// move_from additionally expresses the explicit C++ moved-from null state.
pub struct RaiseThreadTypeLease<'a> {
    leased_thread_type_: ThreadType,
    manager_: Option<core::ptr::NonNull<dyn ThreadTypeManagerInterface + 'a>>,
}
impl RaiseThreadTypeLease<'_> {
    pub fn thread_type(&self) -> ThreadType {
        self.leased_thread_type_
    }
}
// Source: base/threading/platform_thread.cc:23-35
fn GetCurrentMessagePumpType() -> MessagePumpType {
    if crate::base::task::current_thread::CurrentIOThread::IsSet() {
        MessagePumpType::IO
    } else if crate::base::task::current_thread::CurrentUIThread::IsSet() {
        MessagePumpType::UI
    } else {
        MessagePumpType::DEFAULT
    }
}
// Source: base/threading/platform_thread.cc:37-40
std::thread_local! {static THREAD_TYPE_MANAGER:ThreadTypeManager=const{ThreadTypeManager::new()};}
fn GetThreadTypeManager() -> *const ThreadTypeManager {
    THREAD_TYPE_MANAGER.with(|manager| manager as *const _)
}
// Source: base/threading/platform_thread.cc:44-46
// Pending: actual perfetto::TracedValue/WriteIntoTracedValue producer and its
// native move-only writer protocol. No fabricated trace/context is constructed.
impl PlatformThreadBase {
    // Source: base/threading/platform_thread.cc:49-51
    pub unsafe fn SetDefaultThreadType(thread_type: ThreadType) {
        unsafe { (&*GetThreadTypeManager()).SetDefault(thread_type) }
    }
    // Source: base/threading/platform_thread.cc:54-56
    pub fn GetCurrentThreadType() -> ThreadType {
        unsafe { (&*GetThreadTypeManager()).GetCurrent() }
    }
    // Source: base/threading/platform_thread.cc:59-70
    pub fn GetThreadLeewayOverride() -> Option<TimeDelta> {
        #[cfg(base_is_fuchsia)]
        if Self::GetCurrentThreadType() == ThreadType::kRealtimeAudio {
            return Some(crate::base::fuchsia::scheduler::kAudioSchedulingPeriod);
        }
        None
    }
    // Source: base/threading/platform_thread.cc:73-75
    pub unsafe fn SetNameCommon<S: crate::interop::native_ostream::StdStringStorage>(
        name: core::pin::Pin<&crate::interop::native_ostream::NativeStdString<S>>,
    ) {
        unsafe {
            crate::base::threading::thread_id_name_manager::ThreadIdNameManager::GetInstance()
                .SetName(name);
        }
    }
    // Source: base/threading/platform_thread.cc:78-80
    pub fn CurrentThreadHasLeases() -> bool {
        unsafe { (&*GetThreadTypeManager()).HasLeases() }
    }
}
impl<'a> RaiseThreadTypeLease<'a> {
    // Source: base/threading/platform_thread.cc:82-84
    /// Safety: lease cannot outlive or escape the current thread; TLS has a stable
    /// address only until thread exit. The caller-selected raw trait bound is not
    /// a safe reference or an extension of the TLS manager lifetime.
    pub unsafe fn new(thread_type: ThreadType) -> Self {
        unsafe {
            Self::with_manager(
                thread_type,
                core::ptr::NonNull::new_unchecked(
                    GetThreadTypeManager().cast_mut() as *mut dyn ThreadTypeManagerInterface
                ),
            )
        }
    }
}
impl<'a> RaiseThreadTypeLease<'a> {
    // Source: base/threading/platform_thread.cc:86-93
    pub unsafe fn with_manager(
        thread_type: ThreadType,
        manager: core::ptr::NonNull<dyn ThreadTypeManagerInterface + 'a>,
    ) -> Self {
        unsafe {
            crate::DCHECK!(@location crate::FROM_HERE!(@function b"RaiseThreadTypeLease\0".as_ptr().cast();@file b"base/threading/platform_thread.cc\0".as_ptr().cast();@line 91);@condition b"!ScopedBoostablePriority::CurrentThreadHasScope()\0".as_ptr().cast();!crate::base::threading::scoped_thread_priority::ScopedBoostablePriority::CurrentThreadHasScope());
            manager.as_ref().AcquireRaiseLease(thread_type);
            Self {
                leased_thread_type_: thread_type,
                manager_: Some(manager),
            }
        }
    }
    // Source: base/threading/platform_thread.cc:95-98
    pub fn move_from(other: &mut Self) -> Self {
        Self {
            leased_thread_type_: other.leased_thread_type_,
            manager_: other.manager_.take(),
        }
    }
}
// Source: base/threading/platform_thread.cc:100-106
impl Drop for RaiseThreadTypeLease<'_> {
    fn drop(&mut self) {
        if let Some(manager) = self.manager_ {
            unsafe {
                crate::DCHECK!(@location crate::FROM_HERE!(@function b"~RaiseThreadTypeLease\0".as_ptr().cast();@file b"base/threading/platform_thread.cc\0".as_ptr().cast();@line 103);@condition b"!ScopedBoostablePriority::CurrentThreadHasScope()\0".as_ptr().cast();!crate::base::threading::scoped_thread_priority::ScopedBoostablePriority::CurrentThreadHasScope());
                manager.as_ref().DropRaiseLease(self.leased_thread_type_);
            }
        }
    }
}
// These default extension methods retain virtual dispatch at the original
// setter call. No field reference survives the native external callout.
pub trait ThreadTypeManagerOperations: ThreadTypeManagerInterface {
    // Source: base/threading/platform_thread.cc:110-114
    unsafe fn SetDefault(&self, value: ThreadType) {
        unsafe {
            check_thread_type(value, 111, b"SetDefault\0");
            (*self.fields().get()).default_thread_type_ = Some(value);
            self.MaybeUpdate();
        }
    }
    // Source: base/threading/platform_thread.cc:116-118
    unsafe fn GetCurrent(&self) -> ThreadType {
        unsafe {
            (*self.fields().get())
                .effective_thread_type_
                .unwrap_or(ThreadType::kDefault)
        }
    }
    // Source: base/threading/platform_thread.cc:120-135
    unsafe fn MaybeUpdate(&self) {
        unsafe {
            let fields = self.fields().get();
            let highest = (*fields).raise_leases_.GetHighestLease();
            let default = (*fields).default_thread_type_;
            let value = if highest.is_none() && default.is_none() {
                ThreadType::kDefault
            } else {
                core::cmp::max(
                    highest.unwrap_or(ThreadType::kBackground),
                    default.unwrap_or(ThreadType::kBackground),
                )
            };
            if Some(value) != (*fields).effective_thread_type_ {
                (*fields).effective_thread_type_ = Some(value);
                self.SetCurrentThreadTypeImpl(value, GetCurrentMessagePumpType());
            }
        }
    }
    // Source: base/threading/platform_thread.cc:137-141
    unsafe fn AcquireRaiseLease(&self, value: ThreadType) {
        unsafe {
            check_thread_type(value, 138, b"AcquireRaiseLease\0");
            (*self.fields().get()).raise_leases_.Acquire(value);
            self.MaybeUpdate();
        }
    }
    // Source: base/threading/platform_thread.cc:143-147
    unsafe fn DropRaiseLease(&self, value: ThreadType) {
        unsafe {
            check_thread_type(value, 144, b"DropRaiseLease\0");
            (*self.fields().get()).raise_leases_.Drop(value);
            self.MaybeUpdate();
        }
    }
    // Source: base/threading/platform_thread.cc:179-181
    unsafe fn HasLeases(&self) -> bool {
        unsafe {
            (*self.fields().get())
                .raise_leases_
                .GetHighestLease()
                .is_some()
        }
    }
}
impl<T: ThreadTypeManagerInterface + ?Sized> ThreadTypeManagerOperations for T {}
// Source: base/threading/platform_thread.cc:111,138,144
fn check_thread_type(value: ThreadType, line: i32, function: &'static [u8]) {
    crate::CHECK_LE!(@location crate::FROM_HERE!(@function function.as_ptr().cast();@file b"base/threading/platform_thread.cc\0".as_ptr().cast();@line line);@condition b"type <= ThreadType::kMaxValue\0".as_ptr().cast();value,ThreadType::kMaxValue);
}
impl RaiseLeasesFields {
    // Source: base/threading/platform_thread.cc:149-154
    /// Safety: the native enum is a nonnegative in-range index; native shifts are defined.
    pub unsafe fn Acquire(&mut self, thread_type: ThreadType) {
        unsafe {
            let index = thread_type.0 as u32;
            let count = self.leases.get_unchecked_mut(index as usize);
            *count = count.wrapping_add(1);
            self.bitmask |= 1u32.unchecked_shl(index);
        }
    }
    // Source: base/threading/platform_thread.cc:156-163
    pub unsafe fn Drop(&mut self, thread_type: ThreadType) {
        unsafe {
            let index = thread_type.0 as u32;
            let count = self.leases.get_unchecked_mut(index as usize);
            *count = count.wrapping_sub(1);
            if *count == 0 {
                self.bitmask &= !(1u32.unchecked_shl(index));
            }
        }
    }
    // Source: base/threading/platform_thread.cc:165-171
    pub fn GetHighestLease(&self) -> Option<ThreadType> {
        if self.bitmask == 0 {
            None
        } else {
            // Source: base/bits.h:108-110. Exact uint32 countl_zero implementation.
            Some(ThreadType(31 - self.bitmask.leading_zeros() as i32))
        }
    }
}
// Source: base/threading/platform_thread.cc:173-177
unsafe fn DefaultSetCurrentThreadTypeImpl(
    thread_type: ThreadType,
    pump_type_hint: MessagePumpType,
) {
    unsafe {
        crate::base::threading::platform_thread_internal::SetCurrentThreadTypeImpl(
            thread_type,
            pump_type_hint,
        );
    }
}
