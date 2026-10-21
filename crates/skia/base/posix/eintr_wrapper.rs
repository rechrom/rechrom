// Source: base/posix/eintr_wrapper.h: actual expression type, original NDEBUG limit and postincrement.
// Unlike PA WrapEINTR, this does NOT narrow ssize_t/syscall's return type to int.
#[macro_export]
macro_rules! HANDLE_EINTR{($expression:expr)=>{{
    #[cfg(base_is_posix)]{
        #[cfg(not(base_native_ndebug))]let mut eintr_wrapper_counter=0;
        loop{let result=$expression;
            if result!=-1||unsafe{*$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::scoped_clear_last_error::errno_location()}!=libc::EINTR{break result;}
            #[cfg(not(base_native_ndebug))]{let retry=eintr_wrapper_counter<100;eintr_wrapper_counter+=1;if !retry{break result;}}
        }
    }
    #[cfg(not(base_is_posix))]{$expression}
}};}
#[macro_export]
macro_rules! IGNORE_EINTR{($expression:expr)=>{{
    let mut result=$expression;
    #[cfg(base_is_posix)]if result==-1&&unsafe{*$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::scoped_clear_last_error::errno_location()}==libc::EINTR{result=0;}
    result
}};}
