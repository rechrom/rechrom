// Source: base/immediate_crash.h: native compiler/target trap macros and ImmediateCrash.
// The explicit inputs below are native build/compiler inputs, never Rust debug_assertions.
#[inline(always)]
pub fn ImmediateCrash() -> ! {
    #[cfg(all(base_use_fuzzing_engine, base_is_linux))]
    compile_error!(
        "native weak __llvm_profile_write_file lookup requires true weak-function linkage mapping"
    );
    #[cfg(any(base_official_build, base_is_win))]
    unsafe {
        #[cfg(any(
            base_immediate_crash = "gcc_x86",
            base_immediate_crash = "gcc_x86_apple",
            base_immediate_crash = "msvc_clang_x86"
        ))]
        core::arch::asm!("int3", "ud2", options(noreturn, nostack));
        #[cfg(base_immediate_crash = "gcc_armel")]
        core::arch::asm!("bkpt #0", "udf #0", options(noreturn, nostack));
        #[cfg(base_immediate_crash = "gcc_arm64")]
        core::arch::asm!("brk #0", "hlt #0", options(noreturn, nostack));
        #[cfg(not(any(
            base_immediate_crash = "gcc_x86",
            base_immediate_crash = "gcc_x86_apple",
            base_immediate_crash = "msvc_clang_x86",
            base_immediate_crash = "gcc_armel",
            base_immediate_crash = "gcc_arm64"
        )))]
        compile_error!(
            "actual native builtin trap/MSVC debugbreak lowering pending; no substituted trap"
        );
    }
    // This abort is the native source's actual non-official, non-Windows branch.
    #[cfg(not(any(base_official_build, base_is_win)))]
    unsafe {
        libc::abort()
    }
}
