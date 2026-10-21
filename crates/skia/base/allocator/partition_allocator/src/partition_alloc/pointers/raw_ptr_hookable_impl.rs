use super::raw_ptr::{RawPtrBackend,BackendKind,ArithmeticPointee,Offset};
use core::sync::atomic::{AtomicPtr,Ordering};
// Source: partition_alloc/pointers/raw_ptr_hookable_impl.h:22-52
// Internal Rust function-value protocol, not an assertion of C++ callback ABI.
pub struct RawPtrHooks{
 pub wrap_ptr:unsafe fn(usize,bool),pub release_wrapped_ptr:unsafe fn(usize,bool),
 pub safely_unwrap_for_dereference:unsafe fn(usize,bool),pub safely_unwrap_for_extraction:unsafe fn(usize,bool),
 pub unsafely_unwrap_for_comparison:unsafe fn(usize),pub advance:unsafe fn(usize,usize),
 pub duplicate:unsafe fn(usize,bool),pub wrap_ptr_for_duplication:unsafe fn(usize,bool),
 pub unsafely_unwrap_for_duplication:unsafe fn(usize),
}
// Source: partition_alloc/pointers/raw_ptr_hookable_impl.cc:14-26
unsafe fn default_status(_address:usize,_unprotected:bool){}
unsafe fn default_address(_address:usize){}
unsafe fn default_advance(_old:usize,_new:usize){}
// Source: partition_alloc/pointers/raw_ptr_hookable_impl.cc:28-38
static DEFAULT_HOOKS:RawPtrHooks=RawPtrHooks{wrap_ptr:default_status,release_wrapped_ptr:default_status,
 safely_unwrap_for_dereference:default_status,safely_unwrap_for_extraction:default_status,
 unsafely_unwrap_for_comparison:default_address,advance:default_advance,duplicate:default_status,
 wrap_ptr_for_duplication:default_status,unsafely_unwrap_for_duplication:default_address};
// Source: partition_alloc/pointers/raw_ptr_hookable_impl.cc:42-54
static HOOKS:AtomicPtr<RawPtrHooks>=AtomicPtr::new(core::ptr::addr_of!(DEFAULT_HOOKS).cast_mut());
pub fn GetRawPtrHooks()->*const RawPtrHooks{HOOKS.load(Ordering::Relaxed)}
/// Safety: hooks is a live immutable native-equivalent table for all concurrent
/// callbacks until every user finishes. Relaxed replacement supplies no owner.
pub unsafe fn InstallRawPtrHooks(hooks:*const RawPtrHooks){HOOKS.store(hooks.cast_mut(),Ordering::Relaxed);}
pub fn ResetRawPtrHooks(){unsafe{InstallRawPtrHooks(core::ptr::addr_of!(DEFAULT_HOOKS))}}
// Source: partition_alloc/pointers/raw_ptr_hookable_impl.h:58-271
pub struct RawPtrHookableImpl<const ENABLE:bool,const UNPROTECTED:bool=false>;
unsafe impl<const E:bool,const U:bool> RawPtrBackend for RawPtrHookableImpl<E,U>{
 const KIND:BackendKind=BackendKind::Hookable;
 const MUST_ZERO_CONSTRUCT:bool=true;const MUST_ZERO_MOVE:bool=true;const MUST_ZERO_DESTRUCT:bool=true;
 unsafe fn wrap<T>(p:*mut T)->*mut T{if E{unsafe{((*GetRawPtrHooks()).wrap_ptr)(p.addr(),U)}}p}
 unsafe fn release<T>(p:*mut T){if E{unsafe{((*GetRawPtrHooks()).release_wrapped_ptr)(p.addr(),U)}}}
 unsafe fn dereference<T>(p:*mut T)->*mut T{if E{unsafe{((*GetRawPtrHooks()).safely_unwrap_for_dereference)(p.addr(),U)}}p}
 unsafe fn extract<T>(p:*mut T)->*mut T{if E{unsafe{((*GetRawPtrHooks()).safely_unwrap_for_extraction)(p.addr(),U)}}p}
 unsafe fn compare<T>(p:*mut T)->*mut T{if E{unsafe{((*GetRawPtrHooks()).unsafely_unwrap_for_comparison)(p.addr())}}p}
 unsafe fn duplicate<T>(p:*mut T)->*mut T{if E{unsafe{((*GetRawPtrHooks()).duplicate)(p.addr(),U)}}p}
 unsafe fn wrap_dup<T>(p:*mut T)->*mut T{if E{unsafe{((*GetRawPtrHooks()).wrap_ptr_for_duplication)(p.addr(),U)}}p}
 unsafe fn unwrap_dup<T>(p:*mut T)->*mut T{if E{unsafe{((*GetRawPtrHooks()).unsafely_unwrap_for_duplication)(p.addr())}}p}
 unsafe fn advance<T:ArithmeticPointee,Z:Offset>(p:*mut T,z:Z,_modify:bool)->*mut T{
  if E{let new=unsafe{z.advance(p)};unsafe{((*GetRawPtrHooks()).advance)(p.addr(),new.addr())}}
  unsafe{z.advance(p)}
 }
 unsafe fn retreat<T:ArithmeticPointee,Z:Offset>(p:*mut T,z:Z,_modify:bool)->*mut T{
  if E{let new=unsafe{z.retreat(p)};unsafe{((*GetRawPtrHooks()).advance)(p.addr(),new.addr())}}
  unsafe{z.retreat(p)}
 }
 unsafe fn delta<T:ArithmeticPointee>(a:*mut T,b:*mut T)->isize{unsafe{a.offset_from(b)}}
 unsafe fn trace<T>(_owner:u64,_p:*mut T){}unsafe fn untrace(_owner:u64){}
 // No native ReportIfDangling hookable method exists; raw_ptr only dispatches
 // it under the actual BRP build condition, so this generic surface is unused.
 unsafe fn report<T>(_p:*mut T){}
 fn increment_swap(){}fn increment_less(){}
}
