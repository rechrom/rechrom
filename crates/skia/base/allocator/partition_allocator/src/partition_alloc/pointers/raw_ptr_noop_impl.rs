use super::raw_ptr::{RawPtrBackend,BackendKind,ArithmeticPointee,Offset};
// Source: partition_alloc/pointers/raw_ptr_noop_impl.h:15-155
pub struct RawPtrNoOpImpl;
unsafe impl RawPtrBackend for RawPtrNoOpImpl{
 const KIND:BackendKind=BackendKind::NoOp;
 const MUST_ZERO_CONSTRUCT:bool=false;const MUST_ZERO_MOVE:bool=false;const MUST_ZERO_DESTRUCT:bool=false;
 unsafe fn wrap<T>(p:*mut T)->*mut T{p}
 unsafe fn release<T>(p:*mut T){
  // Actual native PA_MSAN_CHECK_MEM_IS_INITIALIZED, not an empty sanitizer stub.
  unsafe{super::raw_ptr::native_input::msan_check_initialized(core::ptr::from_ref(&p).cast(),core::mem::size_of_val(&p));}
 }
 unsafe fn dereference<T>(p:*mut T)->*mut T{p}unsafe fn extract<T>(p:*mut T)->*mut T{p}
 unsafe fn compare<T>(p:*mut T)->*mut T{p}unsafe fn duplicate<T>(p:*mut T)->*mut T{p}
 unsafe fn wrap_dup<T>(p:*mut T)->*mut T{p}unsafe fn unwrap_dup<T>(p:*mut T)->*mut T{p}
 unsafe fn advance<T:ArithmeticPointee,Z:Offset>(p:*mut T,z:Z,_modify:bool)->*mut T{unsafe{z.advance(p)}}
 unsafe fn retreat<T:ArithmeticPointee,Z:Offset>(p:*mut T,z:Z,_modify:bool)->*mut T{unsafe{z.retreat(p)}}
 unsafe fn delta<T:ArithmeticPointee>(a:*mut T,b:*mut T)->isize{unsafe{a.offset_from(b)}}
 // These are genuine native empty bodies, not substitutes for other backends.
 unsafe fn trace<T>(_owner:u64,_p:*mut T){}unsafe fn untrace(_owner:u64){}
 unsafe fn report<T>(_p:*mut T){}fn increment_swap(){}fn increment_less(){}
}
