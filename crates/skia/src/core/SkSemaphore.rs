use crate::include::private::SkOnce::SkOnce;
use core::{
    cell::UnsafeCell,
    ptr,
    sync::atomic::{AtomicI32, Ordering},
};

// Source: include/private/SkSemaphore.h:18-54
// Adapted: UnsafeCell holds the once-published native pointer; no owner allocation
// or mutex is introduced. Native lifetime rules remain caller obligations.
pub struct SkSemaphore {
    fCount: AtomicI32,
    fOSSemaphoreOnce: SkOnce,
    fOSSemaphore: UnsafeCell<*mut OSSemaphore>,
}
unsafe impl Send for SkSemaphore {}
unsafe impl Sync for SkSemaphore {}
impl SkSemaphore {
    // Source: include/private/SkSemaphore.h:20
    pub const fn new(count: i32) -> Self {
        Self {
            fCount: AtomicI32::new(count),
            fOSSemaphoreOnce: SkOnce::new(),
            fOSSemaphore: UnsafeCell::new(ptr::null_mut()),
        }
    }
    // Source: include/private/SkSemaphore.h:56-72
    /// Safety: object is live through each atomic operation and any once access.
    /// For contention, both object and OS handle must remain live until the last
    /// OS signal completes. Raw receiver alone does not establish that lifetime.
    pub unsafe fn signal(this: *const Self, n: i32) {
        let prev = unsafe { &(*this).fCount }.fetch_add(n, Ordering::Release);
        let to_signal = core::cmp::min(-prev, n);
        if to_signal > 0 {
            unsafe { Self::osSignal(this, to_signal) };
        }
        // No object access after the release on the uncontended branch, or
        // after the final OS signal on the contended branch.
    }
    // Source: include/private/SkSemaphore.h:74-82
    /// Safety: object and OS semaphore remain live throughout this wait.
    pub unsafe fn wait(this: *const Self) {
        if unsafe { &(*this).fCount }.fetch_sub(1, Ordering::Acquire) <= 0 {
            unsafe { Self::osWait(this) };
        }
    }
    // Source: src/core/SkSemaphore.cpp:67-70
    unsafe fn osSignal(this: *const Self, n: i32) {
        unsafe {
            SkOnce::call_once(ptr::addr_of!((*this).fOSSemaphoreOnce), || {
                *(*this).fOSSemaphore.get() = allocate_os_semaphore();
            })
        };
        let semaphore = unsafe { *(*this).fOSSemaphore.get() };
        unsafe { OSSemaphore::signal(semaphore, n) };
    }
    // Source: src/core/SkSemaphore.cpp:72-75
    unsafe fn osWait(this: *const Self) {
        unsafe {
            SkOnce::call_once(ptr::addr_of!((*this).fOSSemaphoreOnce), || {
                *(*this).fOSSemaphore.get() = allocate_os_semaphore();
            })
        };
        let semaphore = unsafe { *(*this).fOSSemaphore.get() };
        unsafe { OSSemaphore::wait(semaphore) };
    }
    // Source: src/core/SkSemaphore.cpp:77-83
    /// Safety: object remains live through the load and optional CAS.
    pub unsafe fn try_wait(this: *const Self) -> bool {
        let count = unsafe { &(*this).fCount }.load(Ordering::Relaxed);
        if count > 0 {
            return unsafe { &(*this).fCount }
                .compare_exchange_weak(count, count - 1, Ordering::Acquire, Ordering::Acquire)
                .is_ok();
        }
        false
    }
}
// Source: src/core/SkSemaphore.cpp:63-65
impl Drop for SkSemaphore {
    fn drop(&mut self) {
        let semaphore = *self.fOSSemaphore.get_mut();
        if !semaphore.is_null() {
            unsafe { drop(Box::from_raw(semaphore)) };
        }
    }
}

// Source: src/core/SkSemaphore.cpp:12-23
#[cfg(any(sk_features_build_for_mac, sk_features_build_for_ios))]
mod platform {
    use core::ffi::{c_long, c_void};
    unsafe extern "C" {
        fn dispatch_semaphore_create(value: c_long) -> *mut c_void;
        fn dispatch_semaphore_signal(semaphore: *mut c_void) -> c_long;
        fn dispatch_semaphore_wait(semaphore: *mut c_void, timeout: u64) -> c_long;
        fn dispatch_release(object: *mut c_void);
    }
    pub(super) struct OSSemaphore {
        fSemaphore: *mut c_void,
    }
    impl OSSemaphore {
        pub(super) fn new() -> Self {
            Self {
                fSemaphore: unsafe { dispatch_semaphore_create(0) },
            }
        }
        pub(super) unsafe fn signal(this: *const Self, mut n: i32) {
            let handle = unsafe { (*this).fSemaphore };
            loop {
                let previous = n;
                n = n.wrapping_sub(1);
                if previous <= 0 {
                    break;
                }
                unsafe { dispatch_semaphore_signal(handle) };
            }
        }
        pub(super) unsafe fn wait(this: *const Self) {
            let handle = unsafe { (*this).fSemaphore };
            unsafe { dispatch_semaphore_wait(handle, u64::MAX) };
        }
    }
    impl Drop for OSSemaphore {
        fn drop(&mut self) {
            unsafe { dispatch_release(self.fSemaphore) };
        }
    }
}

// Source: src/core/SkSemaphore.cpp:24-42
#[cfg(all(
    not(any(sk_features_build_for_mac, sk_features_build_for_ios)),
    sk_features_build_for_win
))]
mod platform {
    use core::ffi::c_void;
    type HANDLE = *mut c_void;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateSemaphoreW(
            attributes: *const c_void,
            initial: i32,
            maximum: i32,
            name: *const u16,
        ) -> HANDLE;
        fn CloseHandle(handle: HANDLE) -> i32;
        fn ReleaseSemaphore(handle: HANDLE, count: i32, previous: *mut i32) -> i32;
        fn WaitForSingleObject(handle: HANDLE, timeout: u32) -> u32;
    }
    pub(super) struct OSSemaphore {
        fSemaphore: HANDLE,
    }
    impl OSSemaphore {
        pub(super) fn new() -> Self {
            Self {
                fSemaphore: unsafe {
                    CreateSemaphoreW(core::ptr::null(), 0, i32::MAX, core::ptr::null())
                },
            }
        }
        pub(super) unsafe fn signal(this: *const Self, n: i32) {
            let handle = unsafe { (*this).fSemaphore };
            unsafe { ReleaseSemaphore(handle, n, core::ptr::null_mut()) };
        }
        pub(super) unsafe fn wait(this: *const Self) {
            let handle = unsafe { (*this).fSemaphore };
            unsafe { WaitForSingleObject(handle, u32::MAX) };
        }
    }
    impl Drop for OSSemaphore {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.fSemaphore) };
        }
    }
}
// Source: src/core/SkSemaphore.cpp:43-59
#[cfg(not(any(
    sk_features_build_for_mac,
    sk_features_build_for_ios,
    sk_features_build_for_win
)))]
mod platform {
    use core::{cell::UnsafeCell, mem::MaybeUninit};
    // Source: src/core/SkSemaphore.cpp:47-48
    // Canonical libc owns sem_t layout. MaybeUninit retains the native unchecked
    // initialization contract; UnsafeCell permits the OS's synchronized mutation.
    pub(super) struct OSSemaphore {
        fSemaphore: UnsafeCell<MaybeUninit<libc::sem_t>>,
    }
    impl OSSemaphore {
        // Source: src/core/SkSemaphore.cpp:50-50
        /// Safety: the native unchecked sem_init must establish a usable handle.
        /// This Box stays at its original address through sem_destroy; no active
        /// operation may outlive it. A raw receiver does not establish that lease.
        pub(super) unsafe fn new_boxed() -> Box<Self> {
            let semaphore = Box::new(Self {
                fSemaphore: UnsafeCell::new(MaybeUninit::uninit()),
            });
            libc::sem_init(semaphore.fSemaphore.get().cast::<libc::sem_t>(), 0, 0);
            semaphore
        }
    }
    // Source: src/core/SkSemaphore.cpp:51-51
    impl Drop for OSSemaphore {
        fn drop(&mut self) {
            unsafe {
                libc::sem_destroy(self.fSemaphore.get().cast::<libc::sem_t>());
            }
        }
    }
    impl OSSemaphore {
        // Source: src/core/SkSemaphore.cpp:53-53
        /// Safety: the initialized OS allocation remains live through the final
        /// post and no sem_destroy races any operation on its interior storage.
        pub(super) unsafe fn signal(this: *const Self, mut n: i32) {
            let semaphore = (*this).fSemaphore.get().cast::<libc::sem_t>();
            loop {
                let previous = n;
                n = n.wrapping_sub(1);
                if previous <= 0 {
                    break;
                }
                libc::sem_post(semaphore);
            }
        }
        // Source: src/core/SkSemaphore.cpp:54-57
        /// Safety: same initialized allocation/lifetime lease as native sem_wait.
        pub(super) unsafe fn wait(this: *const Self) {
            let semaphore = (*this).fSemaphore.get().cast::<libc::sem_t>();
            while libc::sem_wait(semaphore) == -1 && current_errno() == libc::EINTR {}
        }
    }
    // errno is the actual target libc thread-local entry, not a cached error or
    // a second synchronization implementation. All symbols below are libc APIs.
    #[cfg(any(
        target_os = "linux",
        target_os = "l4re",
        target_os = "emscripten",
        target_os = "dragonfly",
        target_os = "hurd",
        target_os = "fuchsia",
        target_os = "redox"
    ))]
    unsafe fn current_errno() -> libc::c_int {
        *libc::__errno_location()
    }
    #[cfg(any(target_os = "android", target_os = "netbsd", target_os = "openbsd"))]
    unsafe fn current_errno() -> libc::c_int {
        *libc::__errno()
    }
    #[cfg(target_os = "freebsd")]
    unsafe fn current_errno() -> libc::c_int {
        *libc::__error()
    }
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    unsafe fn current_errno() -> libc::c_int {
        *libc::___errno()
    }
    #[cfg(target_os = "haiku")]
    unsafe fn current_errno() -> libc::c_int {
        *libc::_errnop()
    }
    #[cfg(target_os = "aix")]
    unsafe fn current_errno() -> libc::c_int {
        *libc::_Errno()
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "l4re",
        target_os = "emscripten",
        target_os = "dragonfly",
        target_os = "hurd",
        target_os = "fuchsia",
        target_os = "redox",
        target_os = "android",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "solaris",
        target_os = "illumos",
        target_os = "haiku",
        target_os = "aix"
    )))]
    compile_error!("SkSemaphore POSIX errno entry requires its actual target libc binding");
}
use platform::OSSemaphore;

// Allocation bridge only: POSIX sem_init operates at the final heap-member
// address, matching native new OSSemaphore construction. Never move an already
// initialized sem_t from a temporary stack object into Box storage.
unsafe fn allocate_os_semaphore() -> *mut OSSemaphore {
    #[cfg(any(
        sk_features_build_for_mac,
        sk_features_build_for_ios,
        sk_features_build_for_win
    ))]
    let semaphore = Box::new(OSSemaphore::new());
    #[cfg(not(any(
        sk_features_build_for_mac,
        sk_features_build_for_ios,
        sk_features_build_for_win
    )))]
    let semaphore = OSSemaphore::new_boxed();
    Box::into_raw(semaphore)
}
