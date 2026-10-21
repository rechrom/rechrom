use core::{
    ffi::{c_int, c_void},
    sync::atomic::{AtomicBool, Ordering},
};
// Source: base/rand_util_posix.cc: true BoringSSL experiment state, initially false.
static g_use_boringssl: AtomicBool = AtomicBool::new(false);
pub fn UseBoringSSLForRandBytes() -> bool {
    g_use_boringssl.load(Ordering::Relaxed)
}
// BASE_FEATURE(kUseBoringSSLForRandBytes, FEATURE_ENABLED_BY_DEFAULT) actual Feature representation pending.
pub unsafe fn ConfigureBoringSSLBackedRandBytesFieldTrial() {
    g_use_boringssl.store(
        crate::base::feature_list::FeatureList::IsEnabled(&kUseBoringSSLForRandBytes),
        Ordering::Relaxed,
    );
}
unsafe extern "C" {
    fn RAND_bytes(buf: *mut u8, len: usize) -> c_int;
}
// SDK CommonRandom.h and CommonCryptoError.h supply CCRNGStatus=int32_t and kCCSuccess=0.
#[cfg(all(base_is_apple, not(base_is_mac)))]
unsafe extern "C" {
    fn CCRandomGenerateBytes(bytes: *mut c_void, count: usize) -> i32;
}
#[cfg(not(base_is_apple))]
struct URandomFd {
    fd_: c_int,
}
#[cfg(not(base_is_apple))]
impl URandomFd {
    unsafe fn new() -> Self {
        #[cfg(base_is_aix)]
        let flags = libc::O_RDONLY;
        #[cfg(not(base_is_aix))]
        let flags = libc::O_RDONLY | libc::O_CLOEXEC;
        let fd_ = crate::HANDLE_EINTR!(libc::open(b"/dev/urandom\0".as_ptr().cast(), flags));
        crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util_posix.cc\0".as_ptr().cast(),58);@condition b"fd_ >= 0\0".as_ptr();fd_>=0;@stream b"Cannot open /dev/urandom\0".as_ptr());
        Self { fd_ }
    }
}
#[cfg(not(base_is_apple))]
impl Drop for URandomFd {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd_);
        }
    }
}
#[cfg(not(base_is_apple))]
pub unsafe fn GetUrandomFD() -> c_int {
    static FD: std::sync::OnceLock<URandomFd> = std::sync::OnceLock::new();
    FD.get_or_init(|| URandomFd::new()).fd_
}
#[cfg(any(base_is_linux, base_is_chromeos, base_is_android))]
unsafe fn KernelSupportsGetRandom() -> bool {
    crate::base::system::sys_info::KernelVersionNumber::Current()
        >= crate::base::system::sys_info::KernelVersionNumber::new(3, 17, 0)
}
#[cfg(any(base_is_linux, base_is_chromeos, base_is_android))]
unsafe fn GetRandomSyscall(output: *mut u8, length: usize) -> bool {
    // Actual __NR_getrandom compiler/kernel-header constant, not a guessed syscall number.
    let r = crate::HANDLE_EINTR!(libc::syscall(
        crate::base::system::linux_syscall_numbers::__NR_getrandom,
        output,
        length,
        0
    ));
    if length == r as usize {
        #[cfg(base_memory_sanitizer)]
        crate::base::compiler_specific::MSAN_UNPOISON(output.cast(), length);
        return true;
    }
    false
}
// Raw writable bytes preserve native uninitialized caller storage until the true entropy call succeeds.
pub unsafe fn RandBytesInternal(output: *mut u8, length: usize, avoid_allocation: bool) {
    if !avoid_allocation && UseBoringSSLForRandBytes() {
        let _ = RAND_bytes(output, length);
        return;
    }
    #[cfg(base_is_mac)]
    crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util_posix.cc\0".as_ptr().cast(),131);@condition b"getentropy(output.data(), output.size()) == 0\0".as_ptr();libc::getentropy(output.cast(),length)==0);
    #[cfg(all(base_is_apple, not(base_is_mac)))]
    crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util_posix.cc\0".as_ptr().cast(),133);@condition b"CCRandomGenerateBytes(output.data(), output.size()) == kCCSuccess\0".as_ptr();CCRandomGenerateBytes(output.cast(),length)==0);
    #[cfg(not(base_is_apple))]
    {
        #[cfg(any(base_is_linux, base_is_chromeos, base_is_android))]
        {
            static SUPPORT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
            if *SUPPORT.get_or_init(|| KernelSupportsGetRandom())
                && GetRandomSyscall(output, length)
            {
                return;
            }
        }
        let fd = GetUrandomFD();
        let success = crate::base::files::file_util::ReadFromFD(
            fd,
            core::slice::from_raw_parts_mut(output.cast::<core::mem::MaybeUninit<u8>>(), length),
        );
        crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util_posix.cc\0".as_ptr().cast(),152);@condition b"success\0".as_ptr();success);
    }
}
pub unsafe fn RandBytes(output: *mut u8, length: usize) {
    RandBytesInternal(output, length, false);
}
pub unsafe fn RandDoubleAvoidAllocation() -> f64 {
    let mut number = core::mem::MaybeUninit::<u64>::uninit();
    RandBytesInternal(number.as_mut_ptr().cast(), 8, true);
    ((number.assume_init() >> 11) as f64) * (1.0 / 9007199254740992.0)
}
// Actual FeatureList, SysInfo, syscall-number/MSAN and non-Apple ReadFromFD interfaces remain exact pending.
