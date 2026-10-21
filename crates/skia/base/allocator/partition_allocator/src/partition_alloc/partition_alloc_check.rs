// Source: partition_alloc/partition_alloc_check.h:31-105
#[macro_export]
macro_rules! PA_CHECK {($($tokens:tt)*)=>{$crate::PA_BASE_CHECK!($($tokens)*)};}
#[macro_export]
macro_rules! PA_DCHECK {
    ($($tokens:tt)*)=>{{
        #[cfg(all(pa_use_partition_alloc_as_malloc,pa_official,not(pa_dchecks_are_on)))] {}
        #[cfg(not(all(pa_use_partition_alloc_as_malloc,pa_official,not(pa_dchecks_are_on))))]
        $crate::PA_BASE_DCHECK!($($tokens)*);
    }};
}
#[macro_export]
macro_rules! PA_SCAN_DCHECK {($($tokens:tt)*)=>{{#[cfg(all(pa_is_debug,pa_dchecks_are_on))]$crate::PA_DCHECK!($($tokens)*);}};}
// Source: partition_alloc/partition_alloc_check.h:124-160
pub const kDebugKeyMaxLength: usize = 8;
#[cfg_attr(
    all(pa_is_android, pa_arch_cpu_x86_family, pa_arch_cpu_32_bits),
    repr(C, align(8))
)]
#[cfg_attr(
    not(all(pa_is_android, pa_arch_cpu_x86_family, pa_arch_cpu_32_bits)),
    repr(C, align(16))
)]
pub struct DebugKv {
    pub k: [u8; 8],
    pub v: u64,
}
impl DebugKv {
    // Safety: key has at least 8 readable bytes, or an earlier readable NUL terminator.
    pub unsafe fn new(key: *const u8, value: u64) -> Self {
        let mut result = Self {
            k: [0; 8],
            v: value,
        };
        for c in &mut result.k {
            *c = b' ';
        }
        for index in 0..8 {
            result.k[index] = *key.add(index);
            if *key.add(index) == 0 {
                break;
            }
        }
        result
    }
}
// Source: partition_alloc/partition_alloc_check.h:189-194
// Binding is introduced in the caller scope so Alias keeps the original stack object live.
#[macro_export]
macro_rules! PA_DEBUG_DATA_ON_STACK{
    ($name:literal,$value:expr)=>{
        const _:()=assert!(concat!($name,"\0").len()<=9);
        let kv=unsafe{$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_check::DebugKv::new(concat!($name,"\0").as_ptr(),$value)};
        unsafe{$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::debug::alias::Alias((&kv as *const _).cast());}
    };
}
// Native PA_PCHECK/PA_DPCHECK and STATIC_ASSERT_OR_PA_CHECK stream forms pending actual logger formatting dependencies.

#[macro_export]
macro_rules! PA_PCHECK{
    (@source $file:expr,$line:expr; @condition $cpp:expr; $condition:expr)=>{{
        #[cfg(all(pa_use_partition_alloc_as_malloc,pa_official,not(pa_dchecks_are_on)))]
        if !$condition{unsafe{let error=$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::log_message::GetLastSystemErrorCode();$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::debug::alias::Alias((&error as *const _).cast());$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::immediate_crash::PA_IMMEDIATE_CRASH!();}}
        #[cfg(not(all(pa_use_partition_alloc_as_malloc,pa_official,not(pa_dchecks_are_on))))]
        $crate::PA_BASE_PCHECK!(@source $file,$line;@condition $cpp;$condition);
    }};
}
#[macro_export]
macro_rules! PA_DPCHECK{
    ($($tokens:tt)*)=>{{#[cfg(pa_dchecks_are_on)]{
        #[cfg(all(pa_use_partition_alloc_as_malloc,pa_official,not(pa_dchecks_are_on)))]$crate::PA_PCHECK!($($tokens)*);
        #[cfg(not(all(pa_use_partition_alloc_as_malloc,pa_official,not(pa_dchecks_are_on))))]$crate::PA_BASE_DPCHECK!($($tokens)*);
    }}};
}
