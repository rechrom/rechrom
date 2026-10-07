// Bellard/Gordon MIT. quickjs-libc.c:2131..2159.
// These OS calls belong to the tool host, never to the engine.
#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
pub fn get_time_ms() -> i64 {
    unsafe {
        let mut time: libc::timespec = std::mem::zeroed();
        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time);
        (time.tv_sec as u64)
            .wrapping_mul(1000)
            .wrapping_add((time.tv_nsec / 1000000) as u64) as i64
    }
}
#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
pub fn get_time_ns() -> i64 {
    unsafe {
        let mut time: libc::timespec = std::mem::zeroed();
        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time);
        (time.tv_sec as u64)
            .wrapping_mul(1000000000)
            .wrapping_add(time.tv_nsec as u64) as i64
    }
}

// Separating the arithmetic permits direct testing of the fallback on hosts
// whose normal get_time_* configuration selects CLOCK_MONOTONIC.
#[doc(hidden)]
pub const fn host_timeval_ms(seconds: i64, microseconds: i64) -> i64 {
    seconds.wrapping_mul(1000).wrapping_add(microseconds / 1000)
}
#[doc(hidden)]
pub const fn host_timeval_ns(seconds: i64, microseconds: i64) -> i64 {
    seconds
        .wrapping_mul(1000000000)
        .wrapping_add(microseconds.wrapping_mul(1000))
}

// C2147..2152: deliberately uses wall time, so changing the system date can
// change timeout behavior. The original does not inspect gettimeofday's result.
#[doc(hidden)]
pub fn host_gettimeofday_ms() -> i64 {
    unsafe {
        let mut time: libc::timeval = std::mem::zeroed();
        libc::gettimeofday(&mut time, std::ptr::null_mut());
        host_timeval_ms(time.tv_sec as i64, time.tv_usec as i64)
    }
}
// C2154..2159: microsecond-resolution time expressed in nanoseconds.
#[doc(hidden)]
pub fn host_gettimeofday_ns() -> i64 {
    unsafe {
        let mut time: libc::timeval = std::mem::zeroed();
        libc::gettimeofday(&mut time, std::ptr::null_mut());
        host_timeval_ns(time.tv_sec as i64, time.tv_usec as i64)
    }
}
#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
pub fn get_time_ms() -> i64 {
    host_gettimeofday_ms()
}
#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
pub fn get_time_ns() -> i64 {
    host_gettimeofday_ns()
}
