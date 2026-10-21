use crate::src::core::SkTSearch::SkTSearchWith;
use core::ffi::c_void;
// Source: src/core/SkPtrRecorder.h:84-85
// Required virtual hooks retain the original pointer acquisition/release order.
pub trait SkPtrSetHooks {
    unsafe fn incPtr(&mut self, p: *mut c_void);
    unsafe fn decPtr(&mut self, p: *mut c_void);
}
// Native base hooks are intentionally empty; subclasses supply actual owners.
pub struct Unowned;
impl SkPtrSetHooks for Unowned {
    unsafe fn incPtr(&mut self, _: *mut c_void) {}
    unsafe fn decPtr(&mut self, _: *mut c_void) {}
}
// Source: src/core/SkPtrRecorder.h:88-91
#[derive(Clone, Copy)]
struct Pair {
    fPtr: *mut c_void,
    fIndex: u32,
}
// Source: src/core/SkPtrRecorder.h:97-97
// Source: src/core/SkPtrRecorder.h:24-24
pub struct SkPtrSet<H: SkPtrSetHooks = Unowned> {
    fList: Vec<Pair>,
    hooks: H,
}
impl Default for SkPtrSet {
    fn default() -> Self {
        Self::new(Unowned)
    }
}
impl<H: SkPtrSetHooks> SkPtrSet<H> {
    pub fn new(hooks: H) -> Self {
        Self {
            fList: Vec::new(),
            hooks,
        }
    }
    // Source: src/core/SkPtrRecorder.h:45-45
    pub fn count(&self) -> i32 {
        self.fList.len() as i32
    }
    // Source: src/core/SkPtrRecorder.cpp:12-21
    pub fn reset(&mut self) {
        for p in &self.fList {
            unsafe { self.hooks.decPtr(p.fPtr) }
        }
        self.fList.clear();
    }
    // Source: src/core/SkPtrRecorder.cpp:23-25
    fn Less(a: &Pair, b: &Pair) -> bool {
        (a.fPtr as usize) < (b.fPtr as usize)
    }
    // Source: src/core/SkPtrRecorder.cpp:27-42
    pub fn find(&self, ptr: *mut c_void) -> u32 {
        if ptr.is_null() {
            return 0;
        }
        let pair = Pair {
            fPtr: ptr,
            fIndex: 0,
        };
        let index = unsafe {
            SkTSearchWith(
                self.fList.as_ptr(),
                self.count(),
                &pair,
                core::mem::size_of::<Pair>(),
                Self::Less,
            )
        };
        if index < 0 {
            return 0;
        }
        self.fList[index as usize].fIndex
    }
    // Source: src/core/SkPtrRecorder.cpp:44-63
    /// Safety: hooks may retain/release ptr; it must satisfy their concrete owner contract.
    pub unsafe fn add(&mut self, ptr: *mut c_void) -> u32 {
        if ptr.is_null() {
            return 0;
        }
        let count = self.count();
        let mut pair = Pair {
            fPtr: ptr,
            fIndex: 0,
        };
        let mut index = unsafe {
            SkTSearchWith(
                self.fList.as_ptr(),
                count,
                &pair,
                core::mem::size_of::<Pair>(),
                Self::Less,
            )
        };
        if index < 0 {
            index = !index;
            unsafe { self.hooks.incPtr(ptr) }
            pair.fIndex = (count + 1) as u32;
            self.fList.insert(index as usize, pair);
            (count + 1) as u32
        } else {
            self.fList[index as usize].fIndex
        }
    }
    // Source: src/core/SkPtrRecorder.cpp:65-75
    pub fn copyToArray(&self, array: &mut [*mut c_void]) {
        let count = self.count();
        if count > 0 {
            assert!(array.len() >= count as usize);
            for p in &self.fList {
                let index = p.fIndex - 1;
                debug_assert!(index < count as u32);
                array[index as usize] = p.fPtr;
            }
        }
    }
    // Source: src/core/SkPtrRecorder.h:67-69
    pub fn iter(&self) -> SkPtrSetIter<'_, H> {
        SkPtrSetIter {
            fSet: self,
            fIndex: 0,
        }
    }
}
// Source: src/core/SkPtrRecorder.h:79-80
pub struct SkPtrSetIter<'a, H: SkPtrSetHooks> {
    fSet: &'a SkPtrSet<H>,
    fIndex: usize,
}
impl<H: SkPtrSetHooks> SkPtrSetIter<'_, H> {
    // Source: src/core/SkPtrRecorder.h:74-76
    pub fn next(&mut self) -> *mut c_void {
        if self.fIndex < self.fSet.fList.len() {
            let p = self.fSet.fList[self.fIndex].fPtr;
            self.fIndex += 1;
            p
        } else {
            core::ptr::null_mut()
        }
    }
}
// Source: src/core/SkPtrRecorder.h:108-108
// UNVERIFIED generic native pointer casting only; no retained object owner is
// created. The original set owns sorted Pair records and their insertion index.
pub struct SkTPtrSet<T: SkPtrSetPointer> {
    set: SkPtrSet,
    marker: core::marker::PhantomData<T>,
}
// Conversion must round-trip exact thin pointer identity. Raw pointers remain
// borrowed and function pointers are nullable Option values, never fabricated.
pub unsafe trait SkPtrSetPointer: Copy {
    fn into_void(self) -> *mut c_void;
    unsafe fn from_void(pointer: *mut c_void) -> Self;
}
unsafe impl<T> SkPtrSetPointer for *mut T {
    fn into_void(self) -> *mut c_void {
        self.cast()
    }
    unsafe fn from_void(pointer: *mut c_void) -> Self {
        pointer.cast()
    }
}
unsafe impl<T> SkPtrSetPointer for *const T {
    fn into_void(self) -> *mut c_void {
        self.cast_mut().cast()
    }
    unsafe fn from_void(pointer: *mut c_void) -> Self {
        pointer.cast()
    }
}
unsafe impl SkPtrSetPointer for Option<crate::src::core::SkFlattenable::Factory> {
    fn into_void(self) -> *mut c_void {
        self.map_or(core::ptr::null_mut(), |f| f as *mut c_void)
    }
    unsafe fn from_void(pointer: *mut c_void) -> Self {
        if pointer.is_null() {
            None
        } else {
            Some(core::mem::transmute::<
                *mut c_void,
                crate::src::core::SkFlattenable::Factory,
            >(pointer))
        }
    }
}
impl<T: SkPtrSetPointer> Default for SkTPtrSet<T> {
    fn default() -> Self {
        Self {
            set: SkPtrSet::default(),
            marker: core::marker::PhantomData,
        }
    }
}
impl<T: SkPtrSetPointer> SkTPtrSet<T> {
    // Source: src/core/SkPtrRecorder.h:110-112
    pub fn find(&self, pointer: T) -> u32 {
        self.set.find(pointer.into_void())
    }
    // Source: src/core/SkPtrRecorder.h:113-115
    pub unsafe fn add(&mut self, pointer: T) -> u32 {
        self.set.add(pointer.into_void())
    }
    // Source: src/core/SkPtrRecorder.h:117-119
    pub fn copyToArray(&self, array: &mut [T]) {
        assert!(array.len() >= self.set.count() as usize);
        // Explicit conversion replaces void** reinterpretation. Keep original
        // Pair insertion indices, without retaining/releasing pointed-to objects.
        for pair in &self.set.fList {
            array[(pair.fIndex - 1) as usize] = unsafe { T::from_void(pair.fPtr) };
        }
    }
    pub fn count(&self) -> i32 {
        self.set.count()
    }
}

// Native base destructor does not invoke overridden hooks; owning subclass drops
// must explicitly reset while the concrete hooks still exist.
// Source: src/core/SkPtrRecorder.h:140-140
pub struct SkFactorySet {
    set: SkTPtrSet<Option<crate::src::core::SkFlattenable::Factory>>,
}
impl Default for SkFactorySet {
    fn default() -> Self {
        Self {
            set: SkTPtrSet::default(),
        }
    }
}
impl SkFactorySet {
    // Source: src/core/SkPtrRecorder.h:113-115
    pub fn add(&mut self, f: crate::src::core::SkFlattenable::Factory) -> u32 {
        unsafe { self.set.add(Some(f)) }
    }
    // Source: src/core/SkPtrRecorder.h:110-112
    pub fn find(&self, f: crate::src::core::SkFlattenable::Factory) -> u32 {
        self.set.find(Some(f))
    }
    pub fn find_nullable(&self, f: Option<crate::src::core::SkFlattenable::Factory>) -> u32 {
        self.set.find(f)
    }
    pub fn count(&self) -> i32 {
        self.set.count()
    }
    // Source: src/core/SkPtrRecorder.h:117-119
    pub fn copyToArray(&self, out: &mut [Option<crate::src::core::SkFlattenable::Factory>]) {
        self.set.copyToArray(out)
    }
}
// Source: src/core/SkPtrRecorder.h:130-138
// H is the actual retained handle, not the pointed-to object. Default entries
// are native strong Typeface handles; an existing Arc-owned dictionary uses
// SkRefCntSet<Arc<T>> and keeps that same Arc allocation/count protocol.
// Deref projects each retained handle's actual target, never a guessed header.
pub struct SkRefCntSet<H = crate::src::core::SkTypeface::SkTypefaceRef>
where
    H: crate::include::core::SkRefCnt::SkRefCntHandle + core::ops::Deref,
{
    set: SkPtrSet,
    owners: std::collections::HashMap<usize, H>,
}
impl<H> Default for SkRefCntSet<H>
where
    H: crate::include::core::SkRefCnt::SkRefCntHandle + core::ops::Deref,
{
    fn default() -> Self {
        Self {
            set: SkPtrSet::default(),
            owners: std::collections::HashMap::new(),
        }
    }
}
impl<H> SkRefCntSet<H>
where
    H: crate::include::core::SkRefCnt::SkRefCntHandle + core::ops::Deref,
{
    // Source: src/core/SkPtrRecorder.h:113-115
    pub fn add(&mut self, owner: Option<&H>) -> u32 {
        let Some(owner) = owner else {
            return 0;
        };
        if owner.is_null_handle() {
            return 0;
        }
        let p = (&**owner as *const H::Target).cast::<c_void>().cast_mut();
        let old = self.set.find(p);
        if old != 0 {
            return old;
        }
        self.owners.insert(p as usize, owner.ref_handle());
        unsafe { self.set.add(p) }
    }
    // Source: src/core/SkPtrRecorder.h:110-112
    pub fn find(&self, owner: &H) -> u32 {
        if owner.is_null_handle() {
            return 0;
        }
        self.set
            .find((&**owner as *const H::Target).cast::<c_void>().cast_mut())
    }
    // Source: src/core/SkPtrRecorder.h:45-45
    pub fn count(&self) -> i32 {
        self.set.count()
    }
    // Source: src/core/SkPtrRecorder.cpp:12-21
    pub fn reset(&mut self) {
        for pair in &self.set.fList {
            if let Some(owner) = self.owners.remove(&(pair.fPtr as usize)) {
                owner.unref_handle();
            }
        }
        self.set.reset();
    }
    // Source: src/core/SkPtrRecorder.h:117-119
    pub fn copyToArray(&self, out: &mut [*const H::Target]) {
        assert!(out.len() >= self.count() as usize);
        for pair in &self.set.fList {
            // Obtain the original pointer and metadata from the actual live
            // retained owner, rather than reconstructing them from the void key.
            out[pair.fIndex as usize - 1] =
                &*self.owners[&(pair.fPtr as usize)] as *const H::Target;
        }
    }
}
// Source: src/core/SkPtrRecorder.h:132-132
impl<H> Drop for SkRefCntSet<H>
where
    H: crate::include::core::SkRefCnt::SkRefCntHandle + core::ops::Deref,
{
    fn drop(&mut self) {
        self.reset();
    }
}
// Source: src/core/SkPtrRecorder.h:166-168
// Reverse member declaration order preserves native names-before-factory-set
// destruction. The existing retained recorder handle owns this exact payload;
// its one Arc counter supplies native shared retain/release, with no dormant
// second embedded native counter.
// Source: src/core/SkPtrRecorder.h:146-146
pub struct SkNamedFactorySet {
    pub(crate) fNames: Vec<&'static [u8]>,
    pub(crate) fFactorySet: SkFactorySet,
    pub(crate) fNextAddedFactory: usize,
}

// Source: src/core/SkPtrRecorder.h:24-24
// Source: src/core/SkPtrRecorder.h:130-130
// Source: src/core/SkPtrRecorder.h:146-146
// Rust owner representation: the already used Arc owns this exact complete
// recorder allocation. ref/unref are its Clone/Drop, not a second intrusive
// header. UnsafeCell preserves the native externally synchronized mutable payload;
// it is not an independent state or lock. The Arc remains the sole counter.
// No Send/Sync is asserted: each raw operation requires external exclusion and
// nonreentry, and no payload reference may outlive that operation lease.
// This is an explicit language/ownership adaptation, not C++ ABI credit.
pub type SkRefCntSetOwner<H:crate::include::core::SkRefCnt::SkRefCntHandle+core::ops::Deref=crate::src::core::SkTypeface::SkTypefaceRef> = std::sync::Arc<core::cell::UnsafeCell<SkRefCntSet<H>>>;
pub type SkFactorySetOwner = std::sync::Arc<core::cell::UnsafeCell<SkFactorySet>>;
pub type SkNamedFactorySetOwner = std::sync::Arc<core::cell::UnsafeCell<SkNamedFactorySet>>;
impl<H> SkRefCntSet<H>
where
    H: crate::include::core::SkRefCnt::SkRefCntHandle + core::ops::Deref,
{
    pub fn into_owner(self) -> SkRefCntSetOwner<H> {
        std::sync::Arc::new(core::cell::UnsafeCell::new(self))
    }
}
impl SkFactorySet {
    pub fn into_owner(self) -> SkFactorySetOwner {
        std::sync::Arc::new(core::cell::UnsafeCell::new(self))
    }
}
impl SkNamedFactorySet {
    pub fn into_owner(self) -> SkNamedFactorySetOwner {
        std::sync::Arc::new(core::cell::UnsafeCell::new(self))
    }
}
