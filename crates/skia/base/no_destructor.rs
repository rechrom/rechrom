//! Native inline lifetime suppression, not a heap owner or another reference count.
use core::{mem::MaybeUninit,pin::Pin,marker::PhantomPinned,ptr};
// Source: base/no_destructor.h:83-155
// MaybeUninit<T> has precisely T's size/alignment and suppresses T's Drop.
// No Copy/Clone/DerefMut is implemented for the complete self-referential owner.
#[repr(C)]
pub struct NoDestructor<T>{
 storage_:MaybeUninit<T>,
 #[cfg(LEAK_SANITIZER)]storage_ptr_:*mut T,
 _pin:PhantomPinned,
}
impl<T>NoDestructor<T>{
 // Source: base/no_destructor.h:85-93
 const fn require_nontrivial_destructor(){assert!(core::mem::needs_drop::<T>(),"T is trivially destructible; please use a function-local static of type T directly instead");}
 // Source: base/no_destructor.h:97-101
 /// Final uninitialized exclusive storage must remain stationary indefinitely.
 /// construct is the monomorphized original T constructor/forwarded arguments,
 /// and must initialize exactly the supplied slot, without publishing it early.
 /// If it unwinds, the complete NoDestructor does not exist; original T's
 /// constructor must clean its successfully constructed members itself.
 pub unsafe fn construct_at<F:FnOnce(*mut T)>(out:*mut Self,construct:F){
  const{Self::require_nontrivial_destructor();}
  unsafe{
   ptr::addr_of_mut!((*out).storage_).write(MaybeUninit::uninit());
   ptr::addr_of_mut!((*out)._pin).write(PhantomPinned);
   construct(ptr::addr_of_mut!((*out).storage_).cast());
   #[cfg(LEAK_SANITIZER)]ptr::addr_of_mut!((*out).storage_ptr_).write(ptr::addr_of_mut!((*out).storage_).cast());
  }
 }
 // Source: base/no_destructor.h:105-106
 // Rust move/Clone construction is permitted only when the actual value's
 // movement is valid; pinned native types use construct_at instead.
 pub unsafe fn move_construct_at(out:*mut Self,value:T)where T:Unpin{unsafe{Self::construct_at(out,|slot|slot.write(value));}}
 pub unsafe fn copy_construct_at(out:*mut Self,value:&T)where T:Clone+Unpin{unsafe{Self::construct_at(out,|slot|slot.write(value.clone()));}}
 // Source: base/no_destructor.h:113-122
 pub fn get(self:Pin<&Self>)->Pin<&T>{unsafe{Pin::new_unchecked(self.get_ref().storage_.assume_init_ref())}}
 pub fn get_mut(self:Pin<&mut Self>)->Pin<&mut T>{unsafe{Pin::new_unchecked(self.get_unchecked_mut().storage_.assume_init_mut())}}
 pub fn as_ptr(&self)->*const T{self.storage_.as_ptr()}
 pub fn as_mut_ptr(&mut self)->*mut T{self.storage_.as_mut_ptr()}
}
// Source: base/no_destructor.h:111. Default destructor never destroys storage T.
// Rust field drop is exactly empty for MaybeUninit/raw pointer/PhantomPinned.

/// Language mapping of a *particular declaration's* C++ local-static storage and
/// hidden initialization guard. Declare a distinct static for every source local;
/// a static declared inside a generic Rust function must never be used to pretend
/// that C++ template specializations have distinct statics.
/// No destructor runs for the stored T, as for the particular native locals used
/// here (NoDestructor<Lock>, pointer and function pointer).
pub struct NativeLocalStatic<T>{guard:std::sync::Once,storage:core::cell::UnsafeCell<MaybeUninit<T>>}
// Shared access only exposes an unsafe raw initialization/access operation.
// This is not a Send/Sync promise about T or its pointee; callers must establish
// the original static object's cross-thread usage/protected-data requirements.
unsafe impl<T>Sync for NativeLocalStatic<T>{}
impl<T>NativeLocalStatic<T>{
 pub const fn uninitialized()->Self{Self{guard:std::sync::Once::new(),storage:core::cell::UnsafeCell::new(MaybeUninit::uninit())}}
 /// initialize must construct the exact final slot once, clean partial members
 /// on unwind, and obey native recursive-initialization restrictions. Publishing
 /// waits for completion; failed construction can retry, as C++ local statics.
 /// All use of the returned pointer obeys T's native alias/thread/Pin contract.
 pub unsafe fn get_or_initialize<F:FnOnce(*mut T)>(&'static self,initialize:F)->*mut T{
  let slot=self.storage.get().cast::<T>();
  self.guard.call_once_force(|_|initialize(slot));slot
 }
}
