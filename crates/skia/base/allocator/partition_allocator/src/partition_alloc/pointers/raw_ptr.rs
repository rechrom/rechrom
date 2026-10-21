//! Actual raw_ptr template language projection. Backend and tracer are the
//! selected native implementations, never an implicit plain-pointer fallback.
use core::{marker::{PhantomData,PhantomPinned},mem::MaybeUninit,pin::Pin,ptr::NonNull};
// Source: partition_alloc/pointers/raw_ptr.h:106-175
#[repr(transparent)]
#[derive(Clone,Copy,PartialEq,Eq)]
pub struct RawPtrTraits(pub u32);
impl RawPtrTraits{
 pub const kEmpty:Self=Self(0);pub const kMayDangle:Self=Self(1<<0);
 pub const kDisableHooks:Self=Self(1<<2);pub const kAllowPtrArithmetic:Self=Self(1<<3);
 pub const kAllowUninitialized:Self=Self(1<<5);pub const kNoOpImpl:Self=Self(1<<6);
 pub const kIsUnprotectedInRelease:Self=Self(1<<7);
 pub const kUseCountingImplForTest:Self=Self(1<<10);pub const kDummyForTest:Self=Self(1<<11);
 pub const kAllMask:u32=(1<<0)|(1<<2)|(1<<3)|(1<<5)|(1<<6)|(1<<7)|(1<<10)|(1<<11);
 pub const fn contains(self,other:Self)->bool{self.0&other.0==other.0}
}
// Source: partition_alloc/pointers/raw_ptr.h:176-181
super::super::flags::PA_DEFINE_OPERATORS_FOR_FLAGS!(RawPtrTraits,RawPtrTraits::kAllMask);
const impl super::super::flags::ScopedFlags for RawPtrTraits{
 const ALL_MASK:Self=Self(Self::kAllMask);fn bits(self)->u32{self.0}
}
// Source: partition_alloc/pointers/raw_ptr.h:185-259,320-326
/// Actual C++ type support/customization. Implement only for genuine supported
/// nonfunction/non-ObjC/nonblacklisted native pointees, not all Rust types.
pub unsafe trait RawPointee:Sized{const TYPE_TRAITS:u32=0;}
unsafe impl RawPointee for core::ffi::c_void{}
/// Actual non-void complete object arithmetic domain; excludes c_void.
pub unsafe trait ArithmeticPointee:RawPointee{}
macro_rules! arithmetic_pointees{($($t:ty),*)=>{$(unsafe impl RawPointee for $t{} unsafe impl ArithmeticPointee for $t{})*};}
arithmetic_pointees!(u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64,bool);
pub use super::super::partition_alloc_forward::Offset;
#[derive(Clone,Copy,PartialEq,Eq)]
pub enum BackendKind{NoOp,BackupRef,AsanUnowned,Hookable,CountingForTest}
const fn backend_tag(kind:BackendKind)->u8{match kind{BackendKind::NoOp=>0,BackendKind::BackupRef=>1,BackendKind::AsanUnowned=>2,BackendKind::Hookable=>3,BackendKind::CountingForTest=>4}}
// Source: partition_alloc/pointers/raw_ptr.h:261-318
#[derive(Clone,Copy)]
pub struct Build{
 pub underlying:BackendKind,pub enable_brp_for_unprotected:bool,
 pub partition_alloc:bool,pub arithmetic_trait_check:bool,
 pub zero_on_construct:bool,pub zero_on_move:bool,pub zero_on_destruct:bool,
 pub complex_special_members:bool,pub backup_ref:bool,
}
pub const fn resolve_backend(traits:u32,build:Build)->BackendKind{
 if traits&RawPtrTraits::kUseCountingImplForTest.0!=0{BackendKind::CountingForTest}
 else if traits&RawPtrTraits::kNoOpImpl.0!=0 || (traits&RawPtrTraits::kIsUnprotectedInRelease.0!=0&&!build.enable_brp_for_unprotected){BackendKind::NoOp}
 else{build.underlying}
}
// Source: partition_alloc/pointers/raw_ptr.h:340-376
pub unsafe trait RawPtrConfig<T:RawPointee,const POINTER_TRAITS:u32>{
 type Impl:RawPtrBackend;type Tracer:InstanceTracer;
 const BUILD:Build;
}
/// Real InstanceTracer template member. Active instrumentation supplies a
/// genuine new owner ID for every native construction, never a copied ID.
pub unsafe trait InstanceTracer{fn new()->Self;fn owner_id(&self)->u64;}
// Source: partition_alloc/pointers/raw_ptr.h:343-345,994-1023
/// Actual static backend operation contract. Every operation is required;
/// instrumented backends cannot inherit no-op fallback implementations.
pub unsafe trait RawPtrBackend{
 const KIND:BackendKind;const MUST_ZERO_CONSTRUCT:bool;const MUST_ZERO_MOVE:bool;const MUST_ZERO_DESTRUCT:bool;
 unsafe fn wrap<T>(p:*mut T)->*mut T;unsafe fn release<T>(p:*mut T);
 unsafe fn dereference<T>(p:*mut T)->*mut T;unsafe fn extract<T>(p:*mut T)->*mut T;
 unsafe fn compare<T>(p:*mut T)->*mut T;unsafe fn duplicate<T>(p:*mut T)->*mut T;
 unsafe fn wrap_dup<T>(p:*mut T)->*mut T;unsafe fn unwrap_dup<T>(p:*mut T)->*mut T;
 unsafe fn advance<T:ArithmeticPointee,Z:Offset>(p:*mut T,z:Z,modify:bool)->*mut T;
 unsafe fn retreat<T:ArithmeticPointee,Z:Offset>(p:*mut T,z:Z,modify:bool)->*mut T;
 unsafe fn delta<T:ArithmeticPointee>(a:*mut T,b:*mut T)->isize;
 unsafe fn trace<T>(owner:u64,p:*mut T);unsafe fn untrace(owner:u64);
 unsafe fn report<T>(p:*mut T);fn increment_swap();fn increment_less();
}
// Source: partition_alloc/pointers/raw_ptr.h:337-343,1025-1026
// MaybeUninit represents the actual permitted uninitialized default pointer,
// not a new flag. Runtime observations of that state require native validity.
pub struct raw_ptr<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32=0>{
 wrapped_ptr_:MaybeUninit<*mut T>,tracer_:C::Tracer,config:PhantomData<C>,
}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> raw_ptr<T,C,P>{
 pub const TRAITS:u32=P|T::TYPE_TRAITS;
 fn requirements(){const{assert!(Self::TRAITS&!RawPtrTraits::kAllMask==0);assert!(C::BUILD.partition_alloc||backend_tag(C::Impl::KIND)==0);assert!(backend_tag(resolve_backend(Self::TRAITS,C::BUILD))==backend_tag(C::Impl::KIND));}}
 pub const fn zero_on_construct()->bool{C::Impl::MUST_ZERO_CONSTRUCT||(C::BUILD.zero_on_construct&&Self::TRAITS&RawPtrTraits::kAllowUninitialized.0==0)}
 pub const fn zero_on_move()->bool{C::Impl::MUST_ZERO_MOVE||C::BUILD.zero_on_move}
 pub const fn zero_on_destruct()->bool{C::Impl::MUST_ZERO_DESTRUCT||C::BUILD.zero_on_destruct}
 // Source: partition_alloc/pointers/raw_ptr.h:378-397
 /// Safety: if the selected default leaves the pointer uninitialized, initialize
 /// it in place before any observation or a destructor that reads it.
 pub unsafe fn default_native()->Self{
  Self::requirements();let wrapped_ptr_=if Self::zero_on_construct(){MaybeUninit::new(core::ptr::null_mut())}else{MaybeUninit::uninit()};
  Self{wrapped_ptr_,tracer_:C::Tracer::new(),config:PhantomData}
 }
 // Source: partition_alloc/pointers/raw_ptr.h:539-552
 pub fn null()->Self{Self::requirements();Self{wrapped_ptr_:MaybeUninit::new(core::ptr::null_mut()),tracer_:C::Tracer::new(),config:PhantomData}}
 /// Safety: p satisfies the actual selected allocator/backend wrap domain.
 pub unsafe fn from_ptr(p:*mut T)->Self{
  Self::requirements();let wrapped=unsafe{C::Impl::wrap(p)};
  let result=Self{wrapped_ptr_:MaybeUninit::new(wrapped),tracer_:C::Tracer::new(),config:PhantomData};
  unsafe{C::Impl::trace(result.tracer_.owner_id(),wrapped)};result
 }
 // Source: partition_alloc/pointers/raw_ptr.h:398-427
 pub unsafe fn copy_from(source:&Self)->Self{
  let p=unsafe{source.wrapped_ptr_.assume_init()};let wrapped=if C::BUILD.complex_special_members{unsafe{C::Impl::duplicate(p)}}else{p};
  let result=Self{wrapped_ptr_:MaybeUninit::new(wrapped),tracer_:C::Tracer::new(),config:PhantomData};
  if C::BUILD.complex_special_members{unsafe{C::Impl::trace(result.tracer_.owner_id(),p)}}result
 }
 pub unsafe fn copy_assign_raw(dst:NonNull<Self>,source:NonNull<Self>){unsafe{
  let p=(*source.as_ptr()).wrapped_ptr_.assume_init();
  if C::BUILD.complex_special_members{
   let new_ptr=C::Impl::duplicate(p);C::Impl::release((*dst.as_ptr()).wrapped_ptr_.assume_init());
   C::Impl::untrace((*dst.as_ptr()).tracer_.owner_id());(*dst.as_ptr()).wrapped_ptr_.write(new_ptr);
   C::Impl::trace((*dst.as_ptr()).tracer_.owner_id(),new_ptr);
  }else{(*dst.as_ptr()).wrapped_ptr_.write(p);}
 }}
 // Source: partition_alloc/pointers/raw_ptr.h:429-463
 pub unsafe fn move_from(source:&mut Self)->Self{unsafe{
  let wrapped=source.wrapped_ptr_.assume_init();let result=Self{wrapped_ptr_:MaybeUninit::new(wrapped),tracer_:C::Tracer::new(),config:PhantomData};
  if C::BUILD.complex_special_members||C::BUILD.zero_on_move{
   C::Impl::trace(result.tracer_.owner_id(),wrapped);
   if Self::zero_on_move(){source.wrapped_ptr_.write(core::ptr::null_mut());C::Impl::untrace(source.tracer_.owner_id());}
  }result
 }}
 pub unsafe fn move_assign_raw(dst:NonNull<Self>,source:NonNull<Self>){unsafe{
  if C::BUILD.complex_special_members||C::BUILD.zero_on_move{
   if dst!=source{
    C::Impl::release((*dst.as_ptr()).wrapped_ptr_.assume_init());C::Impl::untrace((*dst.as_ptr()).tracer_.owner_id());
    let p=(*source.as_ptr()).wrapped_ptr_.assume_init();(*dst.as_ptr()).wrapped_ptr_.write(p);C::Impl::trace((*dst.as_ptr()).tracer_.owner_id(),p);
    if Self::zero_on_move(){(*source.as_ptr()).wrapped_ptr_.write(core::ptr::null_mut());C::Impl::untrace((*source.as_ptr()).tracer_.owner_id());}
   }
  }else{let p=(*source.as_ptr()).wrapped_ptr_.assume_init();(*dst.as_ptr()).wrapped_ptr_.write(p);}
 }}
 // Source: partition_alloc/pointers/raw_ptr.h:577-601
 pub unsafe fn assign_null(&mut self){unsafe{C::Impl::release(self.wrapped_ptr_.assume_init());C::Impl::untrace(self.tracer_.owner_id());self.wrapped_ptr_.write(core::ptr::null_mut());}}
 pub unsafe fn assign_ptr(&mut self,p:*mut T){unsafe{let new_ptr=C::Impl::wrap(p);C::Impl::release(self.wrapped_ptr_.assume_init());C::Impl::untrace(self.tracer_.owner_id());self.wrapped_ptr_.write(new_ptr);C::Impl::trace(self.tracer_.owner_id(),new_ptr);}}
 // Source: partition_alloc/pointers/raw_ptr.h:655-657,698-722,994-1023
 pub unsafe fn get(&self)->*mut T{unsafe{C::Impl::extract(self.wrapped_ptr_.assume_init())}}
 pub unsafe fn get_for_dereference(&self)->*mut T{unsafe{C::Impl::dereference(self.wrapped_ptr_.assume_init())}}
 pub unsafe fn get_for_comparison(&self)->*mut T{unsafe{C::Impl::compare(self.wrapped_ptr_.assume_init())}}
 pub unsafe fn is_some(&self)->bool{unsafe{!self.wrapped_ptr_.assume_init().is_null()}}
 pub unsafe fn extraction_and_reset(&mut self)->*mut T{let p=unsafe{self.get()};unsafe{self.assign_null()};p}
 // Source: partition_alloc/pointers/raw_ptr.h:492-537
 pub unsafe fn copy_cross<C2:RawPtrConfig<T,Q>,const Q:u32>(source:&raw_ptr<T,C2,Q>)->Self{unsafe{
  const{assert!(Self::TRAITS==(raw_ptr::<T,C2,Q>::TRAITS|RawPtrTraits::kMayDangle.0));assert!(Self::TRAITS!=raw_ptr::<T,C2,Q>::TRAITS);}
  let p=C2::Impl::unwrap_dup(source.wrapped_ptr_.assume_init());let wrapped=C::Impl::wrap_dup(p);
  let result=Self{wrapped_ptr_:MaybeUninit::new(wrapped),tracer_:C::Tracer::new(),config:PhantomData};C::Impl::trace(result.tracer_.owner_id(),wrapped);result
 }}
 pub unsafe fn assign_cross<C2:RawPtrConfig<T,Q>,const Q:u32>(&mut self,source:&raw_ptr<T,C2,Q>){unsafe{
  const{assert!(Self::TRAITS==(raw_ptr::<T,C2,Q>::TRAITS|RawPtrTraits::kMayDangle.0));assert!(Self::TRAITS!=raw_ptr::<T,C2,Q>::TRAITS);}
  C::Impl::release(self.wrapped_ptr_.assume_init());C::Impl::untrace(self.tracer_.owner_id());
  let p=C2::Impl::unwrap_dup(source.wrapped_ptr_.assume_init());let wrapped=C::Impl::wrap_dup(p);self.wrapped_ptr_.write(wrapped);C::Impl::trace(self.tracer_.owner_id(),wrapped);
 }}
 // Source: partition_alloc/pointers/raw_ptr.h:970-978
 pub unsafe fn swap(lhs:&mut Self,rhs:&mut Self){C::Impl::increment_swap();core::mem::swap(&mut lhs.wrapped_ptr_,&mut rhs.wrapped_ptr_);}
 pub unsafe fn report_if_dangling(&self){if C::BUILD.backup_ref{unsafe{C::Impl::report(self.wrapped_ptr_.assume_init())}}}
 pub unsafe fn less(&self,rhs:&Self)->bool{C::Impl::increment_less();unsafe{self.get_for_comparison().addr()<rhs.get_for_comparison().addr()}}
 pub unsafe fn equal<C2:RawPtrConfig<T,Q>,const Q:u32>(&self,rhs:&raw_ptr<T,C2,Q>)->bool{unsafe{self.get_for_comparison()==rhs.get_for_comparison()}}
}
// Source: partition_alloc/pointers/raw_ptr.h:465-490
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> Drop for raw_ptr<T,C,P>{fn drop(&mut self){
 if C::BUILD.complex_special_members||C::BUILD.zero_on_destruct{unsafe{
  C::Impl::release(self.wrapped_ptr_.assume_init());C::Impl::untrace(self.tracer_.owner_id());
  if Self::zero_on_destruct(){self.wrapped_ptr_.write(core::ptr::null_mut());}
 }}
}}
impl<T:ArithmeticPointee,C:RawPtrConfig<T,P>,const P:u32> raw_ptr<T,C,P>{
 fn arithmetic_requirement(){const{assert!(!C::BUILD.arithmetic_trait_check||Self::TRAITS&RawPtrTraits::kAllowPtrArithmetic.0!=0);}}
 // Source: partition_alloc/pointers/raw_ptr.h:724-813
 pub unsafe fn advance_assign<Z:Offset>(&mut self,z:Z){Self::arithmetic_requirement();let p=unsafe{C::Impl::advance(self.wrapped_ptr_.assume_init(),z,true)};self.wrapped_ptr_.write(p);}
 pub unsafe fn retreat_assign<Z:Offset>(&mut self,z:Z){Self::arithmetic_requirement();let p=unsafe{C::Impl::retreat(self.wrapped_ptr_.assume_init(),z,true)};self.wrapped_ptr_.write(p);}
 pub unsafe fn post_increment(&mut self)->Self{let old=unsafe{Self::copy_from(self)};unsafe{self.advance_assign(1i32)};old}
 pub unsafe fn post_decrement(&mut self)->Self{let old=unsafe{Self::copy_from(self)};unsafe{self.retreat_assign(1i32)};old}
 pub unsafe fn index(&self,z:impl Offset)->*mut T{Self::arithmetic_requirement();unsafe{C::Impl::dereference(C::Impl::advance(self.wrapped_ptr_.assume_init(),z,false))}}
 // Source: partition_alloc/pointers/raw_ptr.h:833-878
 pub unsafe fn add<Z:Offset>(&self,z:Z)->Self{Self::arithmetic_requirement();unsafe{Self::from_ptr(C::Impl::advance(self.wrapped_ptr_.assume_init(),z,false))}}
 pub unsafe fn sub<Z:Offset>(&self,z:Z)->Self{Self::arithmetic_requirement();unsafe{Self::from_ptr(C::Impl::retreat(self.wrapped_ptr_.assume_init(),z,false))}}
 pub unsafe fn delta(&self,rhs:&Self)->isize{Self::arithmetic_requirement();unsafe{C::Impl::delta(self.wrapped_ptr_.assume_init(),rhs.wrapped_ptr_.assume_init())}}
}
// Source: partition_alloc/pointers/raw_ptr.h:659-696
pub struct EphemeralRawAddr<'a,T:RawPointee,C:RawPtrConfig<T,P>,const P:u32>{copy:*mut T,original:&'a mut raw_ptr<T,C,P>,pin:PhantomPinned}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> raw_ptr<T,C,P>{
 pub unsafe fn AsEphemeralRawAddr(&mut self)->EphemeralRawAddr<'_,T,C,P>{let copy=unsafe{self.get()};EphemeralRawAddr{copy,original:self,pin:PhantomPinned}}
}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> EphemeralRawAddr<'_,T,C,P>{
 /// Safety: pin this same stack object before taking its copy field address;
 /// the native T**/T*& must not escape its expression or alias original use.
 pub unsafe fn as_mut_ptr(self:Pin<&mut Self>)->*mut *mut T{unsafe{core::ptr::addr_of_mut!(self.get_unchecked_mut().copy)}}
}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> Drop for EphemeralRawAddr<'_,T,C,P>{fn drop(&mut self){unsafe{self.original.assign_ptr(self.copy)}}}
// Source: partition_alloc/pointers/raw_ptr.h:1080-1233
pub const DisableDanglingPtrDetection:u32=RawPtrTraits::kMayDangle.0;
pub const DanglingUntriaged:u32=DisableDanglingPtrDetection;pub const FlakyDanglingUntriaged:u32=DisableDanglingPtrDetection;
pub const AcrossTasksDanglingUntriaged:u32=DisableDanglingPtrDetection;pub const AllowPtrArithmetic:u32=RawPtrTraits::kAllowPtrArithmetic.0;
pub const AllowUninitialized:u32=RawPtrTraits::kAllowUninitialized.0;pub const LeakedDanglingUntriaged:u32=DisableDanglingPtrDetection;
pub const VectorExperimental:u32=DisableDanglingPtrDetection;pub const SetExperimental:u32=DisableDanglingPtrDetection;pub const CtnExperimental:u32=DisableDanglingPtrDetection;
pub const UnprotectedInRelease:u32=RawPtrTraits::kIsUnprotectedInRelease.0;
// No Copy/Clone marker erases native instrumented special members. Rust language
// relocations are not falsely advertised as executing native move hooks; use
// explicit native move_from/move_assign_raw at translated native move sites.
// Source: partition_alloc/pointers/raw_ptr.h:555-575,603-653,712-722
/// Real implicit/static C++ pointer conversion (including adjusted base address).
/// A producer must implement the actual conversion; no universal bitcast exists.
pub unsafe trait PointerConversion<From:RawPointee,To:RawPointee>{unsafe fn convert(p:*mut From)->*mut To;}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> raw_ptr<T,C,P>{
 pub unsafe fn copy_upcast<U:RawPointee,C2:RawPtrConfig<U,P>,A:PointerConversion<U,T>>(source:&raw_ptr<U,C2,P>)->Self{unsafe{
  const{assert!(Self::TRAITS==raw_ptr::<U,C2,P>::TRAITS);}
  let p=C::Impl::duplicate(A::convert(source.wrapped_ptr_.assume_init()));
  let result=Self{wrapped_ptr_:MaybeUninit::new(p),tracer_:C::Tracer::new(),config:PhantomData};C::Impl::trace(result.tracer_.owner_id(),p);result
 }}
 pub unsafe fn move_upcast<U:RawPointee,C2:RawPtrConfig<U,P>,A:PointerConversion<U,T>>(source:&mut raw_ptr<U,C2,P>)->Self{unsafe{
  const{assert!(Self::TRAITS==raw_ptr::<U,C2,P>::TRAITS);}
  let p=A::convert(source.wrapped_ptr_.assume_init());let result=Self{wrapped_ptr_:MaybeUninit::new(p),tracer_:C::Tracer::new(),config:PhantomData};C::Impl::trace(result.tracer_.owner_id(),p);
  if Self::zero_on_move(){source.wrapped_ptr_.write(core::ptr::null_mut());C2::Impl::untrace(source.tracer_.owner_id());}result
 }}
 pub unsafe fn assign_upcast<U:RawPointee,C2:RawPtrConfig<U,P>,A:PointerConversion<U,T>>(&mut self,source:&raw_ptr<U,C2,P>){unsafe{
  const{assert!(Self::TRAITS==raw_ptr::<U,C2,P>::TRAITS);}
  C::Impl::release(self.wrapped_ptr_.assume_init());C::Impl::untrace(self.tracer_.owner_id());
  let p=C::Impl::duplicate(A::convert(source.wrapped_ptr_.assume_init()));self.wrapped_ptr_.write(p);C::Impl::trace(self.tracer_.owner_id(),p);
 }}
 pub unsafe fn move_assign_upcast<U:RawPointee,C2:RawPtrConfig<U,P>,A:PointerConversion<U,T>>(&mut self,source:&mut raw_ptr<U,C2,P>){unsafe{
  const{assert!(Self::TRAITS==raw_ptr::<U,C2,P>::TRAITS);}
  C::Impl::release(self.wrapped_ptr_.assume_init());C::Impl::untrace(self.tracer_.owner_id());
  let p=A::convert(source.wrapped_ptr_.assume_init());self.wrapped_ptr_.write(p);C::Impl::trace(self.tracer_.owner_id(),p);
  if Self::zero_on_move(){source.wrapped_ptr_.write(core::ptr::null_mut());C2::Impl::untrace(source.tracer_.owner_id());}
 }}
 pub unsafe fn explicit_pointer_cast<U:RawPointee,A:PointerConversion<T,U>>(&self)->*mut U{unsafe{A::convert(self.get())}}
 // Source: partition_alloc/pointers/raw_ptr.h:884-892
 pub unsafe fn ExtractAsDangling<C2:RawPtrConfig<T,Q>,const Q:u32>(&mut self)->raw_ptr<T,C2,Q>{unsafe{
  const{assert!(raw_ptr::<T,C2,Q>::TRAITS==(Self::TRAITS|RawPtrTraits::kMayDangle.0));}
  let result=if raw_ptr::<T,C2,Q>::TRAITS==Self::TRAITS{
   let p=self.wrapped_ptr_.assume_init();let result=raw_ptr::<T,C2,Q>{wrapped_ptr_:MaybeUninit::new(p),tracer_:C2::Tracer::new(),config:PhantomData};
   if C::BUILD.complex_special_members||C::BUILD.zero_on_move{
    C2::Impl::trace(result.tracer_.owner_id(),p);
    if Self::zero_on_move(){self.wrapped_ptr_.write(core::ptr::null_mut());C::Impl::untrace(self.tracer_.owner_id());}
   }result
  }else{raw_ptr::<T,C2,Q>::copy_cross(self)};
  self.assign_null();result
 }}
}
// Source: partition_alloc/pointers/raw_ptr.h:861-866
// delete/delete[] must use the actual pointee destructor and matching native
// allocation family, never an assumed Box layout or unknown array cookie.
pub unsafe trait NativeDelete:RawPointee{
 unsafe fn delete(p:*mut Self);unsafe fn delete_array(p:*mut Self);
}
impl<T:NativeDelete,C:RawPtrConfig<T,P>,const P:u32> raw_ptr<T,C,P>{
 pub unsafe fn ClearAndDelete(&mut self){let p=unsafe{self.extraction_and_reset()};unsafe{T::delete(p)}}
 pub unsafe fn ClearAndDeleteArray(&mut self){let p=unsafe{self.extraction_and_reset()};unsafe{T::delete_array(p)}}
}
// Source: partition_alloc/pointers/raw_ptr.h:1036-1062
pub trait PointerMetadata{type Element;const IS_RAW_PTR:bool;const IS_POINTER_OR_RAW:bool;const MAY_DANGLE:bool;}
impl<T> PointerMetadata for T{default type Element=T;default const IS_RAW_PTR:bool=false;default const IS_POINTER_OR_RAW:bool=false;default const MAY_DANGLE:bool=false;}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> PointerMetadata for raw_ptr<T,C,P>{type Element=T;const IS_RAW_PTR:bool=true;const IS_POINTER_OR_RAW:bool=true;const MAY_DANGLE:bool=P&RawPtrTraits::kMayDangle.0!=0;}
impl<T> PointerMetadata for *mut T{type Element=T;const IS_RAW_PTR:bool=false;const IS_POINTER_OR_RAW:bool=true;const MAY_DANGLE:bool=false;}
impl<T> PointerMetadata for *const T{type Element=T;const IS_RAW_PTR:bool=false;const IS_POINTER_OR_RAW:bool=true;const MAY_DANGLE:bool=false;}
// Source: partition_alloc/pointers/raw_ptr.h:1261-1302
pub trait PointerTraits{type Element;type Difference;unsafe fn to_address(self)->*mut Self::Element;}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> PointerTraits for raw_ptr<T,C,P>{type Element=T;type Difference=isize;unsafe fn to_address(self)->*mut T{unsafe{self.get()}}}
// Source: partition_alloc/pointers/raw_ptr.h:923-968,1033-1043
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> raw_ptr<T,C,P>{
 pub unsafe fn equal_raw(&self,rhs:*mut T)->bool{unsafe{self.get_for_comparison()==rhs}}
 pub unsafe fn compare_raw(&self,rhs:*mut T)->core::cmp::Ordering{unsafe{self.get_for_comparison().addr().cmp(&rhs.addr())}}
 pub unsafe fn hash<H:core::hash::Hasher>(&self,state:&mut H){core::hash::Hash::hash(&unsafe{self.get()},state);}
}
// Rust projection of native const pointee qualification. This contains the same
// one raw_ptr state/backend/tracer, adds no allocation or second reference, and
// never exposes a mutable-pointee accessor. It claims no native field ABI.
pub struct raw_ptr_const<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32=0>{inner:raw_ptr<T,C,P>}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> raw_ptr_const<T,C,P>{
 pub fn null()->Self{Self{inner:raw_ptr::null()}}
 pub unsafe fn from_ptr(p:*const T)->Self{Self{inner:unsafe{raw_ptr::from_ptr(p.cast_mut())}}}
 pub unsafe fn get(&self)->*const T{unsafe{self.inner.get().cast_const()}}
 pub unsafe fn comparison(&self)->*const T{unsafe{self.inner.get_for_comparison().cast_const()}}
 pub unsafe fn copy_from(source:&Self)->Self{Self{inner:unsafe{raw_ptr::copy_from(&source.inner)}}}
 pub unsafe fn move_from(source:&mut Self)->Self{Self{inner:unsafe{raw_ptr::move_from(&mut source.inner)}}}
 pub unsafe fn assign_ptr(&mut self,p:*const T){unsafe{self.inner.assign_ptr(p.cast_mut())}}
 pub unsafe fn assign_null(&mut self){unsafe{self.inner.assign_null()}}
 pub unsafe fn copy_assign(&mut self,source:&Self){unsafe{raw_ptr::copy_assign_raw(NonNull::from(&mut self.inner),NonNull::from(&source.inner))}}
 pub unsafe fn move_assign(&mut self,source:&mut Self){unsafe{raw_ptr::move_assign_raw(NonNull::from(&mut self.inner),NonNull::from(&mut source.inner))}}
}
// Source: partition_alloc/pointers/raw_ptr.h:1052-1076
// Genuine template specialization: the nonpointer case remains its original
// type, rather than adding an extra level of indirection or a wrapper object.
pub type RemovePointerT<T>=<T as PointerMetadata>::Element;
pub trait RawPtrIfPtr<C,const P:u32>{type Type;}
impl<T,C,const P:u32> RawPtrIfPtr<C,P> for T{default type Type=T;}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> RawPtrIfPtr<C,P> for *mut T{type Type=raw_ptr<T,C,P>;}
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> RawPtrIfPtr<C,P> for *const T{type Type=raw_ptr_const<T,C,P>;}
impl<T:RawPointee,C:RawPtrConfig<T,P>,S:RawPtrConfig<T,Q>,const P:u32,const Q:u32> RawPtrIfPtr<C,P> for raw_ptr<T,S,Q>{type Type=raw_ptr<T,C,P>;}
pub type RawPtrIfPtrT<T,C,const P:u32>=<T as RawPtrIfPtr<C,P>>::Type;
// Source: partition_alloc/pointers/raw_ptr.h:1261-1302
impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> raw_ptr<T,C,P>{
 pub unsafe fn pointer_to(value:&mut T)->Self{unsafe{Self::from_ptr(core::ptr::from_mut(value))}}
}
// Rebinding is a type-level operation with the genuine selected config for U.
pub type Rebind<U,C,const P:u32>=raw_ptr<U,C,P>;

impl<T:RawPointee,C:RawPtrConfig<T,P>,const P:u32> raw_ptr_const<T,C,P>{
 pub unsafe fn copy_assign_raw(dst:NonNull<Self>,source:NonNull<Self>){unsafe{
  raw_ptr::copy_assign_raw(NonNull::new_unchecked(core::ptr::addr_of_mut!((*dst.as_ptr()).inner)),NonNull::new_unchecked(core::ptr::addr_of_mut!((*source.as_ptr()).inner)))
 }}
 pub unsafe fn move_assign_raw(dst:NonNull<Self>,source:NonNull<Self>){unsafe{
  raw_ptr::move_assign_raw(NonNull::new_unchecked(core::ptr::addr_of_mut!((*dst.as_ptr()).inner)),NonNull::new_unchecked(core::ptr::addr_of_mut!((*source.as_ptr()).inner)))
 }}
}

// Source: partition_alloc/pointers/raw_ptr.h:261-318,340-376
// Actual native preprocessing generates this input. Its absence is an error,
// never a host-derived or debug-derived no-op configuration.
pub(crate) mod native_input{include!(env!("PA_RAW_PTR_NATIVE_CONFIG_RS"));}
pub trait SelectBackend<const KIND:u8,const TRAITS:u32>{type Impl:RawPtrBackend;}
impl<const F:u32> SelectBackend<0,F> for (){type Impl=super::raw_ptr_noop_impl::RawPtrNoOpImpl;}
// Actual enabled Hookable/Asan selections are emitted into native_input only
// when their corresponding native buildflag is enabled. BRP/counting remain
// genuinely unimplemented selections rather than successful no-op aliases.
pub struct NativeConfig;
unsafe impl<T:RawPointee,const P:u32> RawPtrConfig<T,P> for NativeConfig
where ():SelectBackend<{backend_tag(resolve_backend(P|T::TYPE_TRAITS,native_input::BUILD))},{P|T::TYPE_TRAITS}>
{
 type Impl=<() as SelectBackend<{backend_tag(resolve_backend(P|T::TYPE_TRAITS,native_input::BUILD))},{P|T::TYPE_TRAITS}>>::Impl;
 type Tracer=native_input::Tracer;
 const BUILD:Build=native_input::BUILD;
}
pub type LocationPointerConfig=NativeConfig;
