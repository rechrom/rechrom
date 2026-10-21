use super::raw_ptr::{RawPtrBackend,BackendKind,ArithmeticPointee,Offset};
// Source: partition_alloc/pointers/raw_ptr_asan_unowned_impl.cc:15-46
unsafe extern "C"{fn __asan_region_is_poisoned(begin:*mut core::ffi::c_void,size:usize)->*mut core::ffi::c_void;}
pub unsafe fn EndOfAliveAllocation(p:*const core::ffi::c_void,adjustable:bool)->bool{
 let address=p.expose_provenance();
 adjustable&&((address&0x0fff)==0||unsafe{!__asan_region_is_poisoned(p.cast_mut(),1).is_null()})
 &&unsafe{__asan_region_is_poisoned(core::ptr::with_exposed_provenance_mut(address.wrapping_sub(1)),1).is_null()}
}
pub fn LikelySmuggledScalar(p:*const core::ffi::c_void)->bool{(p.addr() as isize)<0x4000}
// Source: partition_alloc/pointers/raw_ptr_asan_unowned_impl.h:24-160
pub struct RawPtrAsanUnownedImpl<const ADJUSTABLE:bool,const MAY_DANGLE:bool>;
impl<const A:bool,const D:bool> RawPtrAsanUnownedImpl<A,D>{
 unsafe fn probe<T>(p:*mut T){if !D&&!p.is_null(){
  let probe=p.cast::<core::ffi::c_void>();
  if !LikelySmuggledScalar(probe)&&!unsafe{EndOfAliveAllocation(probe,A)}{
   // Deliberately retain the real instrumented volatile load. It is not a
   // substitute boolean, no-op check, or memory-safety success fallback.
   let _=unsafe{core::ptr::read_volatile(p.cast::<u8>())};
  }
 }}
}
unsafe impl<const A:bool,const D:bool> RawPtrBackend for RawPtrAsanUnownedImpl<A,D>{
 const KIND:BackendKind=BackendKind::AsanUnowned;
 const MUST_ZERO_CONSTRUCT:bool=true;const MUST_ZERO_MOVE:bool=true;const MUST_ZERO_DESTRUCT:bool=true;
 unsafe fn wrap<T>(p:*mut T)->*mut T{p}
 unsafe fn release<T>(p:*mut T){unsafe{Self::probe(p)}}
 unsafe fn dereference<T>(p:*mut T)->*mut T{p}
 unsafe fn extract<T>(p:*mut T)->*mut T{unsafe{Self::probe(p)}p}
 unsafe fn compare<T>(p:*mut T)->*mut T{p}unsafe fn duplicate<T>(p:*mut T)->*mut T{p}
 unsafe fn wrap_dup<T>(p:*mut T)->*mut T{p}unsafe fn unwrap_dup<T>(p:*mut T)->*mut T{p}
 unsafe fn advance<T:ArithmeticPointee,Z:Offset>(p:*mut T,z:Z,_modify:bool)->*mut T{unsafe{z.advance(p)}}
 unsafe fn retreat<T:ArithmeticPointee,Z:Offset>(p:*mut T,z:Z,_modify:bool)->*mut T{unsafe{z.retreat(p)}}
 unsafe fn delta<T:ArithmeticPointee>(a:*mut T,b:*mut T)->isize{unsafe{a.offset_from(b)}}
 unsafe fn trace<T>(_owner:u64,_p:*mut T){}unsafe fn untrace(_owner:u64){}
 // Wrapper ReportIfDangling has an empty body under the actual non-BRP branch.
 unsafe fn report<T>(_p:*mut T){}fn increment_swap(){}fn increment_less(){}
}
