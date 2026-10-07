// quickjs.c native call dispatch and stack checks. Bellard/Gordon, MIT.
// c: quickjs.c:2043
#[inline(always)]
unsafe fn js_get_stack_pointer() -> usize {
    #[cfg(target_arch = "aarch64")]
    {
        let sp: usize;
        core::arch::asm!("mov {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
        sp
    }
    #[cfg(target_arch = "x86_64")]
    {
        let sp: usize;
        core::arch::asm!("mov {}, rsp", out(reg) sp, options(nomem, nostack, preserves_flags));
        sp
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        let slot = 0usize;
        core::ptr::addr_of!(slot) as usize
    }
}
// c: quickjs.c:2048
#[inline(always)]
unsafe fn js_check_stack_overflow(rt: *mut JSRuntime, alloca_size: usize) -> JS_BOOL {
    (js_get_stack_pointer().wrapping_sub(alloca_size) < (*rt).stack_limit) as i32
}
