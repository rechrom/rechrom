//! Intrusive native reference counts. The unsafe object contract supplies Rust's concrete
//! destruction dispatch in place of the C++ virtual base destructor. No external owner block.
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicI32, Ordering};
// Source: include/core/SkRefCnt.h:35-35
impl SkRefCntBase {
    pub const fn new() -> Self {
        Self {
            fRefCnt: AtomicI32::new(1),
        }
    }
}
// Source: include/core/SkRefCnt.h:39-45
impl Drop for SkRefCntBase {
    fn drop(&mut self) {
        #[cfg(sk_debug)]
        {
            crate::SkASSERTF!(@source b"include/core/SkRefCnt.h\0".as_ptr(),41; @condition b"this->getRefCnt() == 1\0".as_ptr(); self.getRefCnt()==1,"fRefCnt was %d",self.getRefCnt());
            self.fRefCnt.store(0, Ordering::Relaxed);
        }
    }
}
impl SkRefCntBase {
    // Source: include/core/SkRefCnt.h:50-58
    pub fn unique(&self) -> bool {
        if self.fRefCnt.load(Ordering::Acquire) == 1 {
            return true;
        }
        false
    }
    // Source: include/core/SkRefCnt.h:62-66
    pub fn ref_(&self) {
        crate::SkASSERT!(@source b"include/core/SkRefCnt.h\0".as_ptr(),63; @condition b"this->getRefCnt() > 0\0".as_ptr(); self.getRefCnt()>0);
        self.fRefCnt.fetch_add(1, Ordering::Relaxed);
    }
}
// Source: include/core/SkRefCnt.h:72-80
/// # Safety
/// obj transfers exactly one live strong reference to the complete object. Its
/// concrete raw disposal slot must match the original data pointer. A weak parent
/// may keep the allocation alive after the final strong release; only that slot
/// determines whether disposal constructs a Box or performs weak disposal instead.
pub unsafe fn SkUnref<T: SkRefCntObject + ?Sized>(obj: NonNull<T>) {
    let dispose = {
        let object = unsafe { obj.as_ref() };
        let base = object.ref_cnt_base();
        crate::SkASSERT!(@source b"include/core/SkRefCnt.h\0".as_ptr(),73; @condition b"this->getRefCnt() > 0\0".as_ptr(); base.getRefCnt()>0);
        if base.fRefCnt.fetch_add(-1, Ordering::AcqRel) == 1 {
            Some(object.internal_dispose_raw_proc())
        } else {
            None
        }
    };
    // No object or counter borrow survives this call. The cast removes metadata
    // solely for the paired concrete function, never to reconstruct a trait object.
    if let Some(dispose) = dispose {
        unsafe {
            dispose(obj.cast::<()>());
        }
    }
}
// Native virtual call glue only: no object field, side table, owner or counter.
// Source: include/core/SkRefCnt.h:79-79
pub type SkRawDisposeProc = unsafe fn(NonNull<()>);
// Source: include/core/SkRefCnt.h:94-100
/// # Safety
/// object is the original complete allocation's data pointer of concrete T,
/// selected by that same T's raw slot. Only ordinary strong disposal uses this
/// adapter: a weak parent must supply its actual strong-disposal proc instead.
pub unsafe fn SkRefCntDispose<T: SkRefCntObject>(object: NonNull<()>) {
    let owned = unsafe { Box::from_raw(object.cast::<T>().as_ptr()) };
    unsafe {
        owned.internal_dispose();
    }
}
// Source: include/core/SkRefCnt.h:94-100
/// Qualified native SkRefCntBase::internal_dispose, bypassing weak overrides.
/// # Safety
/// The original complete allocation is ready for its final actual destruction,
/// has strong count zero, and admits Box disposal with this original metadata.
/// No weak reference or other owner may remain that would later use the object.
pub unsafe fn SkRefCntBaseDispose<T: SkRefCntObject + ?Sized>(object: NonNull<T>) {
    #[cfg(sk_debug)]
    {
        let base = unsafe { object.as_ref() }.ref_cnt_base();
        crate::SkASSERT!(@source b"include/core/SkRefCnt.h\0".as_ptr(),96; @condition b"0 == this->getRefCnt()\0".as_ptr(); base.getRefCnt()==0);
        base.fRefCnt.store(1, Ordering::Relaxed);
    }
    // The preceding base borrow ends before complete concrete destruction.
    drop(unsafe { Box::from_raw(object.as_ptr()) });
}
// Source: include/core/SkRefCnt.h:86-100
/// # Safety
/// Implementors return their sole embedded counter, initially exactly one reference. The
/// actual object stays stable before references are shared. Default disposal requires a
/// Box-compatible complete allocation and dynamically correct metadata. Immortal initial
/// owners must never be passed to SkUnref; only their additional retained handles may release.
pub unsafe trait SkRefCntObject {
    fn ref_cnt_base(&self) -> &SkRefCntBase;
    // Source: include/core/SkRefCnt.h:79-79
    // The actual concrete owner supplies its native virtual disposal target.
    // This getter is a pure, non-unwinding slot lookup: its returned proc must
    // accept this same complete object's original data pointer and lifetime.
    // It cannot mutate/dispose the receiver or reconstruct erased trait metadata.
    fn internal_dispose_raw_proc(&self) -> SkRawDisposeProc;
    // Source: include/core/SkRefCnt.h:62-66
    // C++ static inherited ref lookup is supplied by each real derived parent.
    fn ref_(&self) {
        self.ref_cnt_base().ref_();
    }
    /// # Safety
    /// Invoked only at zero references, holding exclusive Box ownership of the complete
    /// allocation. The Box receiver preserves dynamic concrete destruction without a live
    /// borrowed receiver in the releasing call frame.
    unsafe fn internal_dispose(self: Box<Self>) {
        let object = unsafe { NonNull::new_unchecked(Box::into_raw(self)) };
        unsafe {
            SkRefCntBaseDispose(object);
        }
    }
}
impl SkRefCntBase {
    // Source: include/core/SkRefCnt.h:84-90
    #[cfg(sk_debug)]
    fn getRefCnt(&self) -> i32 {
        self.fRefCnt.load(Ordering::Relaxed)
    }
}
// Source: include/core/SkRefCnt.h:106-106
pub struct SkRefCntBase {
    pub(crate) fRefCnt: AtomicI32,
}
// Source: include/core/SkRefCnt.h:114-118
#[cfg(sk_ref_cnt_mixin = "chromium_debug")]
pub use crate::skia::config::sk_ref_cnt_ext_debug::SkRefCnt;
#[cfg(sk_ref_cnt_mixin = "chromium_release")]
pub use crate::skia::config::sk_ref_cnt_ext_release::SkRefCnt;
// Source: include/core/SkRefCnt.h:119-125
#[cfg(sk_ref_cnt_mixin = "none")]
pub struct SkRefCnt {
    base: SkRefCntBase,
}
#[cfg(sk_ref_cnt_mixin = "none")]
impl SkRefCnt {
    pub const fn new() -> Self {
        Self {
            base: SkRefCntBase::new(),
        }
    }
    pub fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base
    }
    pub fn ref_(&self) {
        self.base.ref_();
    }
    // Source: include/core/SkRefCnt.h:121-124
    /// # Safety
    /// The pointer transfers a reference to the complete object containing this parent.
    #[cfg(sk_build_for_google3)]
    pub unsafe fn deref<T: SkRefCntObject + ?Sized>(object: NonNull<T>) {
        unsafe {
            SkUnref(object);
        }
    }
}
#[cfg(sk_ref_cnt_mixin = "none")]
impl Deref for SkRefCnt {
    type Target = SkRefCntBase;
    fn deref(&self) -> &SkRefCntBase {
        &self.base
    }
}
#[cfg(sk_ref_cnt_mixin = "none")]
unsafe impl SkRefCntObject for SkRefCnt {
    fn internal_dispose_raw_proc(&self) -> SkRawDisposeProc {
        SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base
    }
    fn ref_(&self) {
        SkRefCnt::ref_(self);
    }
}
// Source: include/core/SkRefCnt.h:132-136
/// # Safety
/// obj must be a valid live allocation pointer whose object counter may be retained.
pub unsafe fn SkRef<T: SkRefCntObject + ?Sized>(obj: NonNull<T>) -> NonNull<T> {
    unsafe { obj.as_ref() }.ref_();
    obj
}
// Source: include/core/SkRefCnt.h:140-145
/// # Safety
/// Every non-null argument must satisfy SkRef's validity contract.
pub unsafe fn SkSafeRef<T: SkRefCntObject + ?Sized>(obj: Option<NonNull<T>>) -> Option<NonNull<T>> {
    if let Some(obj) = obj {
        unsafe { obj.as_ref() }.ref_();
    }
    obj
}
// Source: include/core/SkRefCnt.h:149-153
/// # Safety
/// Each non-null pointer transfers exactly one owned reference to this operation.
pub unsafe fn SkSafeUnref<T: SkRefCntObject + ?Sized>(obj: Option<NonNull<T>>) {
    if let Some(obj) = obj {
        unsafe {
            SkUnref(obj);
        }
    }
}
// Source: include/core/SkRefCnt.h:159-205
// The CRTP parameter selects concrete Box disposal without virtual dispatch. The marker
// has no storage; no external reference-count block is introduced.
pub struct SkNVRefCnt<Derived: ?Sized> {
    fRefCnt: AtomicI32,
    derived: std::marker::PhantomData<fn() -> Derived>,
}
impl<Derived: ?Sized> SkNVRefCnt<Derived> {
    pub const fn new() -> Self {
        Self {
            fRefCnt: AtomicI32::new(1),
            derived: std::marker::PhantomData,
        }
    }
    pub fn unique(&self) -> bool {
        self.fRefCnt.load(Ordering::Acquire) == 1
    }
    pub fn ref_(&self) {
        // Source: include/core/SkRefCnt.h:177-177
        self.fRefCnt.fetch_add(1, Ordering::Relaxed);
    }
    pub fn refCntGreaterThan(&self, threadIsolatedTestCnt: i32) -> bool {
        let cnt = self.fRefCnt.load(Ordering::Acquire);
        crate::SkASSERT!(@source b"include/core/SkRefCnt.h\0".as_ptr(),194; @condition b"cnt >= threadIsolatedTestCnt\0".as_ptr(); cnt>=threadIsolatedTestCnt);
        cnt > threadIsolatedTestCnt
    }
}
impl<Derived: ?Sized> Drop for SkNVRefCnt<Derived> {
    fn drop(&mut self) {
        #[cfg(sk_debug)]
        {
            let rc = self.fRefCnt.load(Ordering::Relaxed);
            crate::SkASSERTF!(@source b"include/core/SkRefCnt.h\0".as_ptr(),166; @condition b"rc == 1\0".as_ptr(); rc==1,"NVRefCnt was %d",rc);
        }
    }
}
/// Rust projection of the native CRTP object's embedded nonvirtual counter.
/// # Safety
/// The returned counter is the sole counter, initially one, for this complete object.
/// Final release calls dispose_raw on the complete allocation. Initial stack/static
/// ownership must never be released. The object stays stable while additional refs exist.
pub unsafe trait SkNVRefCntObject: Sized {
    fn nv_ref_cnt(&self) -> &SkNVRefCnt<Self>;
    /// # Safety
    /// Called at final release after counter borrows end (debug counter restored to one).
    /// Default requires a complete Box-compatible allocation. Other allocators must override
    /// and destroy/free the original allocation exactly once, including callback unwind.
    unsafe fn dispose_raw(object: NonNull<Self>) {
        drop(unsafe { Box::from_raw(object.as_ptr()) });
    }
}
impl<Derived: SkNVRefCntObject> SkNVRefCnt<Derived> {
    /// # Safety
    /// Transfers one owned reference to the complete object. Final release frees it.
    pub unsafe fn unref(object: NonNull<Derived>) {
        // As for virtual SkUnref, end the shared borrow before concrete deallocation.
        let dispose = {
            let base = unsafe { object.as_ref() }.nv_ref_cnt();
            if base.fRefCnt.fetch_add(-1, Ordering::AcqRel) == 1 {
                #[cfg(sk_debug)]
                {
                    base.fRefCnt.store(1, Ordering::Relaxed);
                }
                true
            } else {
                false
            }
        };
        if dispose {
            unsafe {
                Derived::dispose_raw(object);
            }
        }
    }
    /// # Safety
    /// Same transfer and complete-allocation contract as unref.
    pub unsafe fn deref(object: NonNull<Derived>) {
        unsafe {
            Self::unref(object);
        }
    }
}
// Rust protocol adaptation: no fields, allocation, secondary counter or second handle.
/// # Safety
/// Implementations retain/release the same sole intrusive counter of the complete object.
/// A retained reference must keep payload and metadata valid. release consumes exactly one
/// reference; final release ends all counter borrows before matching raw disposal. It must
/// clean up its allocation if user callbacks unwind; a second destructor panic may abort.
/// These raw methods must never manufacture ownership from an unretained borrow.
pub unsafe trait SkIntrusiveRefCounted {
    unsafe fn intrusive_retain(object: NonNull<Self>);
    unsafe fn intrusive_release(object: NonNull<Self>);
}
// Only virtual objects have a blanket implementation. NV concrete objects implement the
// protocol explicitly, so there are no overlapping virtual/NV blanket implementations.
unsafe impl<T: SkRefCntObject + ?Sized> SkIntrusiveRefCounted for T {
    unsafe fn intrusive_retain(object: NonNull<Self>) {
        unsafe {
            SkRef(object);
        }
    }
    unsafe fn intrusive_release(object: NonNull<Self>) {
        unsafe {
            SkUnref(object);
        }
    }
}
unsafe fn intrusive_safe_ref<T: SkIntrusiveRefCounted + ?Sized>(
    object: Option<NonNull<T>>,
) -> Option<NonNull<T>> {
    if let Some(pointer) = object {
        unsafe {
            T::intrusive_retain(pointer);
        }
    }
    object
}
unsafe fn intrusive_safe_unref<T: SkIntrusiveRefCounted + ?Sized>(object: Option<NonNull<T>>) {
    if let Some(pointer) = object {
        unsafe {
            T::intrusive_release(pointer);
        }
    }
}
// Source: include/core/SkRefCnt.h:224-225
impl<T: SkIntrusiveRefCounted + ?Sized> Default for sk_sp<T> {
    fn default() -> Self {
        Self { fPtr: None }
    }
}
// Source: include/core/SkRefCnt.h:231-231
impl<T: SkIntrusiveRefCounted + ?Sized> Clone for sk_sp<T> {
    fn clone(&self) -> Self {
        Self {
            fPtr: unsafe { intrusive_safe_ref(self.fPtr) },
        }
    }
}
// Source: include/core/SkRefCnt.h:232-234
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    /// Rust representation of the native implicit U* -> T* conversion.
    ///
    /// # Safety
    /// `cast` must perform only the pointer adjustment/metadata projection for
    /// the same complete object. T's retain and release must operate on U's
    /// original sole counter and dispose the original complete allocation.
    /// The projected pointer and metadata remain valid for that object's lifetime.
    /// The projection must not unwind, retain, release, mutate, or invoke callbacks.
    pub unsafe fn clone_from_cast<U: SkIntrusiveRefCounted + ?Sized>(
        that: &sk_sp<U>,
        cast: impl FnOnce(NonNull<U>) -> NonNull<T>,
    ) -> Self {
        let retained = unsafe { intrusive_safe_ref(that.fPtr) };
        Self {
            fPtr: retained.map(cast),
        }
    }
}
// Source: include/core/SkRefCnt.h:241-241
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    // A mutable source retains native's observable null moved-from handle.
    pub fn move_from(that: &mut Self) -> Self {
        Self {
            fPtr: that.release(),
        }
    }
}
// Source: include/core/SkRefCnt.h:242-244
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    /// # Safety
    /// `cast` has the same sole-counter, complete-allocation, lifetime and
    /// non-unwinding pure-projection contract as clone_from_cast.
    pub unsafe fn move_from_cast<U: SkIntrusiveRefCounted + ?Sized>(
        that: &mut sk_sp<U>,
        cast: impl FnOnce(NonNull<U>) -> NonNull<T>,
    ) -> Self {
        Self {
            fPtr: that.release().map(cast),
        }
    }
}
// Source: include/core/SkRefCnt.h:250-250
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    /// # Safety
    /// Adopt exactly one existing owned reference. Non-null must be the full live object
    /// pointer accepted by SkIntrusiveRefCounted's release protocol, with valid dynamic metadata.
    pub unsafe fn from_raw(obj: *mut T) -> Self {
        Self {
            fPtr: NonNull::new(obj),
        }
    }
}
// Source: include/core/SkRefCnt.h:255-258
impl<T: SkIntrusiveRefCounted + ?Sized> Drop for sk_sp<T> {
    fn drop(&mut self) {
        unsafe {
            intrusive_safe_unref(self.fPtr);
        }
        #[cfg(sk_debug)]
        {
            self.fPtr = None;
        }
    }
}
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    // Source: include/core/SkRefCnt.h:260-260
    pub fn reset_to_null(&mut self) {
        unsafe {
            self.reset(None);
        }
    }
    // Source: include/core/SkRefCnt.h:267-272
    pub fn assign(&mut self, that: &Self) -> &mut Self {
        if !std::ptr::eq(self, that) {
            unsafe {
                self.reset(intrusive_safe_ref(that.fPtr));
            }
        }
        self
    }
}
// Source: include/core/SkRefCnt.h:273-278
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    /// # Safety
    /// `cast` obeys clone_from_cast's pure same-object projection contract.
    /// Native ordering retains the source before releasing the destination,
    /// including when both handles designate the same complete allocation.
    pub unsafe fn assign_cast<U: SkIntrusiveRefCounted + ?Sized>(
        &mut self,
        that: &sk_sp<U>,
        cast: impl FnOnce(NonNull<U>) -> NonNull<T>,
    ) -> &mut Self {
        let retained = unsafe { intrusive_safe_ref(that.fPtr) };
        unsafe {
            self.reset(retained.map(cast));
        }
        self
    }
}
// Source: include/core/SkRefCnt.h:285-288
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    pub fn move_assign(&mut self, that: &mut Self) -> &mut Self {
        unsafe {
            self.reset(that.release());
        }
        self
    }
}
// Source: include/core/SkRefCnt.h:289-294
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    /// # Safety
    /// `cast` obeys clone_from_cast's pure same-object projection contract.
    /// The source is emptied before resetting and releasing the old destination.
    pub unsafe fn move_assign_cast<U: SkIntrusiveRefCounted + ?Sized>(
        &mut self,
        that: &mut sk_sp<U>,
        cast: impl FnOnce(NonNull<U>) -> NonNull<T>,
    ) -> &mut Self {
        let incoming = that.release().map(cast);
        unsafe {
            self.reset(incoming);
        }
        self
    }
}
// Source: include/core/SkRefCnt.h:296-299
impl<T: SkIntrusiveRefCounted + ?Sized> Deref for sk_sp<T> {
    type Target = T;
    fn deref(&self) -> &T {
        crate::SkASSERT!(@source b"include/core/SkRefCnt.h\0".as_ptr(),297; @condition b"this->get() != nullptr\0".as_ptr(); self.get().is_some());
        self.get().expect("null sk_sp dereference")
    }
}
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    // Source: include/core/SkRefCnt.h:301-304
    pub fn is_some(&self) -> bool {
        self.fPtr.is_some()
    }
    pub fn get(&self) -> Option<&T> {
        self.fPtr.map(|p| unsafe { p.as_ref() })
    }
    pub fn as_ptr(&self) -> Option<NonNull<T>> {
        self.fPtr
    }
    // Source: include/core/SkRefCnt.h:310-317
    /// # Safety
    /// A non-null ptr transfers exactly one owned reference, including on self-reset.
    /// Raw pointer adoption must not create an owner for an unretained borrowed pointer.
    pub unsafe fn reset(&mut self, ptr: Option<NonNull<T>>) {
        let oldPtr = self.fPtr;
        self.fPtr = ptr;
        unsafe {
            intrusive_safe_unref(oldPtr);
        }
    }
    // Source: include/core/SkRefCnt.h:324-328
    pub fn release(&mut self) -> Option<NonNull<T>> {
        let ptr = self.fPtr;
        self.fPtr = None;
        ptr
    }
    // Source: include/core/SkRefCnt.h:330-333
    pub fn swap(&mut self, that: &mut Self) {
        std::mem::swap(&mut self.fPtr, &mut that.fPtr);
    }
}
// Source: include/core/SkRefCnt.h:338-338
// Nullable wide pointers retain dynamic destruction metadata; representation/ABI is adapted.
pub struct sk_sp<T: SkIntrusiveRefCounted + ?Sized> {
    fPtr: Option<NonNull<T>>,
}
// Language adaptation: Rust moves transfer the sole handle value without retain/release.
unsafe impl<T: SkIntrusiveRefCounted + Send + Sync + ?Sized> Send for sk_sp<T> {}
unsafe impl<T: SkIntrusiveRefCounted + Send + Sync + ?Sized> Sync for sk_sp<T> {}
// Source: include/core/SkRefCnt.h:341-343
pub fn swap<T: SkIntrusiveRefCounted + ?Sized>(a: &mut sk_sp<T>, b: &mut sk_sp<T>) {
    a.swap(b);
}
// Source: include/core/SkRefCnt.h:345-347
impl<T: SkIntrusiveRefCounted + ?Sized, U: SkIntrusiveRefCounted + ?Sized> PartialEq<sk_sp<U>>
    for sk_sp<T>
{
    fn eq(&self, b: &sk_sp<U>) -> bool {
        match (self.fPtr, b.fPtr) {
            (None, None) => true,
            (Some(a), Some(b)) => std::ptr::addr_eq(a.as_ptr(), b.as_ptr()),
            _ => false,
        }
    }
}
// Source: include/core/SkRefCnt.h:348-363
// Rust's nullable Option view replaces both symmetric nullptr overloads. is_some()
// already expresses the native != nullptr forms; this adds the == nullptr check.
impl<T: SkIntrusiveRefCounted + ?Sized> sk_sp<T> {
    pub fn is_none(&self) -> bool {
        self.fPtr.is_none()
    }
}
// Source: include/core/SkRefCnt.h:372-375
impl<T: SkRefCntObject> sk_sp<T> {
    pub fn new(object: T) -> Self {
        unsafe { Self::from_raw(Box::into_raw(Box::new(object))) }
    }
}
pub fn sk_make_sp<T: SkRefCntObject>(object: T) -> sk_sp<T> {
    sk_sp::new(object)
}
// Source: include/core/SkRefCnt.h:383-389
/// # Safety
/// A non-null pointer must be a live full object pointer. A fresh additional owner is retained.
pub unsafe fn sk_ref_sp<T: SkIntrusiveRefCounted + ?Sized>(obj: Option<NonNull<T>>) -> sk_sp<T> {
    sk_sp {
        fPtr: unsafe { intrusive_safe_ref(obj) },
    }
}

// Rust ownership adaptation for native classes whose existing complete owners
// are Arc allocations. This protocol operates ONLY on that original Arc count;
// it must not be paired with an unused embedded SkRefCntBase. A handle clone is
// ref(), consuming a handle is unref(), and Rust moves transfer it unchanged.
// Source: include/core/SkRefCnt.h:50-58
// Source: include/core/SkRefCnt.h:62-80
// A retained handle refers to the same original complete allocation and target
// identity across ref_handle; its target remains live until matching unref_handle.
// This protocol never supplies an arbitrary raw pointer or a new ownership block.
pub trait SkRefCntHandle: Sized {
    // Native pointer-set null check precedes any nonnull object projection.
    // Arc owners are always nonnull; native sk_sp below supplies its real state.
    fn is_null_handle(&self) -> bool {
        false
    }
    fn ref_handle(&self) -> Self;
    fn unref_handle(self);
    fn unique_handle(&self) -> bool;
}
impl<T: ?Sized> SkRefCntHandle for std::sync::Arc<T> {
    fn ref_handle(&self) -> Self {
        std::sync::Arc::clone(self)
    }
    fn unref_handle(self) {
        drop(self);
    }
    fn unique_handle(&self) -> bool {
        if std::sync::Arc::strong_count(self) == 1 {
            // Pair the actual owner count observation with native acquire ordering.
            std::sync::atomic::fence(Ordering::Acquire);
            true
        } else {
            false
        }
    }
}

// Source: include/core/SkRefCnt.h:296-299
// Language integration only: use the same native nonnull borrowed object view
// as existing Deref, without retaining or manufacturing a second owner.
impl<T: SkIntrusiveRefCounted + ?Sized> AsRef<T> for sk_sp<T> {
    fn as_ref(&self) -> &T {
        core::ops::Deref::deref(self)
    }
}

// Source: include/core/SkRefCnt.h:50-80
// Source: include/core/SkRefCnt.h:231-231
// Source: include/core/SkRefCnt.h:255-258
// Actual virtual intrusive owners reuse exactly the same dictionary protocol;
// no Arc counter, mirrored count or retained borrow is introduced.
impl<T: SkRefCntObject + ?Sized> SkRefCntHandle for sk_sp<T> {
    fn is_null_handle(&self) -> bool {
        self.is_none()
    }
    fn ref_handle(&self) -> Self {
        self.clone()
    }
    fn unref_handle(self) {
        drop(self);
    }
    fn unique_handle(&self) -> bool {
        self.ref_cnt_base().unique()
    }
}
