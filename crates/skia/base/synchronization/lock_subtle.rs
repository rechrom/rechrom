// Source: base/synchronization/lock_subtle.h:39-42
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LockTracking {
    kDisabled,
    kEnabled,
}
// Source: base/synchronization/lock_subtle.h:47-87
#[inline]
pub fn YieldProcessor() {
    unsafe {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        core::arch::asm!("pause", options(nostack, preserves_flags));
        #[cfg(any(target_arch = "aarch64", all(target_arch = "arm", base_arm_arch_ge_6)))]
        core::arch::asm!("yield", options(nostack, preserves_flags));
        #[cfg(target_arch = "mips")]
        core::arch::asm!(".word 0x00000140", options(nostack));
        #[cfg(all(target_arch = "mips64", base_mips_isa_rev_ge_2))]
        core::arch::asm!("pause", options(nostack));
        #[cfg(target_arch = "powerpc64")]
        core::arch::asm!("or 31,31,31", options(nostack));
        #[cfg(target_arch = "riscv64")]
        core::arch::asm!(".insn i 0x0f, 0, x0, x0, 0x010", options(nostack));
        // Native LoongArch NOTREACHED depends on the actual base/notreached producer.
        #[cfg(target_arch = "loongarch64")]
        compile_error!("base lock_subtle LoongArch NOTREACHED producer pending");
    }
}
