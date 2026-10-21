// Copyright 2011 Google Inc. BSD-style license.
use core::{
    marker::{PhantomData, PhantomPinned},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::{self, NonNull},
};
// Source: src/core/SkTLazy.h:16-18
pub fn SkOptAddressOrNull<T>(object: &mut Option<T>) -> *mut T {
    object.as_mut().map_or(ptr::null_mut(), ptr::from_mut)
}
// Source: src/core/SkTLazy.h:20-22
pub fn SkOptAddressOrNull_const<T>(object: &Option<T>) -> *const T {
    object.as_ref().map_or(ptr::null(), ptr::from_ref)
}

// Static constructor/copy/move requirements project the corresponding C++ type
// traits. They do not manufacture an owner or defer a constructor to a service.
// In-place construction must leave exactly one live T at dst on success and no
// complete live T on unwind. Pin-aware operations must preserve source validity.
/// Safety: construct exactly one value in the original stable, uninitialized
/// destination; honor every T pin invariant, including on an unwinding exit.
pub unsafe trait SkTLazyConstruct<Args>: Sized {
    unsafe fn construct(dst: *mut Self, args: Args);
}
/// Safety: copy construction/assignment must implement the type's actual copy
/// semantics. Assignment leaves a live value in the same original destination.
pub unsafe trait SkTLazyCopy: Sized {
    unsafe fn copy_construct(dst: *mut Self, src: &Self);
    unsafe fn copy_assign(dst: Pin<&mut Self>, src: &Self);
}
/// Safety: actual move operations leave the source constructed in its native
/// moved-from state. They must not move a !Unpin value out of its pinned storage.
pub unsafe trait SkTLazyMove: Sized {
    unsafe fn move_construct(dst: *mut Self, src: Pin<&mut Self>);
    unsafe fn move_assign(dst: Pin<&mut Self>, src: Pin<&mut Self>);
}
// Ordinary Rust Default/Clone values may be relocated only when T:Unpin. More
// restrictive native types supply their real in-place operations explicitly.
unsafe impl<T: Default + Unpin> SkTLazyConstruct<()> for T {
    unsafe fn construct(dst: *mut Self, _: ()) {
        unsafe { dst.write(T::default()) };
    }
}
unsafe impl<T: Clone + Unpin> SkTLazyCopy for T {
    unsafe fn copy_construct(dst: *mut Self, src: &Self) {
        unsafe { dst.write(src.clone()) };
    }
    unsafe fn copy_assign(dst: Pin<&mut Self>, src: &Self) {
        dst.get_mut().clone_from(src);
    }
}
// This is reuse of SkBitmap's already translated real native move operations,
// not a default-value producer or a second translation of their algorithms.
unsafe impl SkTLazyMove for super::SkBitmap::SkBitmap {
    unsafe fn move_construct(dst: *mut Self, src: Pin<&mut Self>) {
        unsafe { dst.write(Self::take_from(src.get_mut())) };
    }
    unsafe fn move_assign(dst: Pin<&mut Self>, src: Pin<&mut Self>) {
        dst.get_mut().assign_move(src.get_mut());
    }
}

// Source: src/core/SkTLazy.h:28-28
// std::optional is represented by in-place aligned MaybeUninit<T> plus its
// engagement flag. PhantomPinned keeps returned addresses stable after init.
pub struct SkTLazy<T> {
    // Source: src/core/SkTLazy.h:120-120
    fValue: MaybeUninit<T>,
    initialized: bool,
    _pin: PhantomPinned,
}
impl<T> SkTLazy<T> {
    // Source: src/core/SkTLazy.h:30-30
    pub const fn new() -> Self {
        Self {
            fValue: MaybeUninit::uninit(),
            initialized: false,
            _pin: PhantomPinned,
        }
    }
    // Source: src/core/SkTLazy.h:31-31
    /// Safety: this is uninitialized construction storage (isValid()==false).
    pub unsafe fn construct_copy(self: Pin<&mut Self>, src: Option<&T>)
    where
        T: SkTLazyCopy,
    {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(!this.initialized);
        if let Some(src) = src {
            unsafe { T::copy_construct(this.fValue.as_mut_ptr(), src) };
            this.initialized = true;
        }
    }
    // Source: src/core/SkTLazy.h:32-32
    /// Safety: destination is original uninitialized construction storage.
    pub unsafe fn copy_construct(self: Pin<&mut Self>, that: &Self)
    where
        T: SkTLazyCopy,
    {
        unsafe { self.construct_copy(that.getMaybeNull_ref()) };
    }
    // Source: src/core/SkTLazy.h:33-33
    /// Safety: destination is original uninitialized construction storage; source
    /// retains its own original pinned storage and remains engaged after the move.
    pub unsafe fn move_construct(self: Pin<&mut Self>, that: Pin<&mut Self>)
    where
        T: SkTLazyMove,
    {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(!this.initialized);
        if that.isValid() {
            unsafe { T::move_construct(this.fValue.as_mut_ptr(), that.get()) };
            this.initialized = true;
        }
    }
}
// Source: src/core/SkTLazy.h:35-35
impl<T> Drop for SkTLazy<T> {
    fn drop(&mut self) {
        self.reset_value();
    }
}
impl<T> SkTLazy<T> {
    // Source: src/core/SkTLazy.h:37-40
    pub fn copy_assign<'s>(mut self: Pin<&'s mut Self>, that: &Self) -> Pin<&'s mut Self>
    where
        T: SkTLazyCopy,
    {
        if let Some(src) = that.getMaybeNull_ref() {
            self.as_mut().set(src);
        } else {
            self.as_mut().reset();
        }
        self
    }
    // Source: src/core/SkTLazy.h:42-45
    pub fn move_assign<'s>(mut self: Pin<&'s mut Self>, that: Pin<&mut Self>) -> Pin<&'s mut Self>
    where
        T: SkTLazyMove,
    {
        if that.isValid() {
            self.as_mut().set_move(that.get());
        } else {
            self.as_mut().reset();
        }
        self
    }
    // Source: src/core/SkTLazy.h:53-56
    // Args is the Rust tuple of the native forwarded constructor arguments.
    pub fn init<Args>(self: Pin<&mut Self>, args: Args) -> Pin<&mut T>
    where
        T: SkTLazyConstruct<Args>,
    {
        let this = unsafe { self.get_unchecked_mut() };
        this.reset_value();
        unsafe { T::construct(this.fValue.as_mut_ptr(), args) };
        this.initialized = true;
        unsafe { Pin::new_unchecked(this.fValue.assume_init_mut()) }
    }
    // Naming projection of optional::emplace used by native init; same operation.
    pub fn emplace<Args>(self: Pin<&mut Self>, args: Args) -> Pin<&mut T>
    where
        T: SkTLazyConstruct<Args>,
    {
        self.init(args)
    }
    // Source: src/core/SkTLazy.h:64-67
    pub fn set<'s>(self: Pin<&'s mut Self>, src: &T) -> Pin<&'s mut T>
    where
        T: SkTLazyCopy,
    {
        let this = unsafe { self.get_unchecked_mut() };
        if this.initialized {
            unsafe { T::copy_assign(Pin::new_unchecked(this.fValue.assume_init_mut()), src) };
        } else {
            unsafe { T::copy_construct(this.fValue.as_mut_ptr(), src) };
            this.initialized = true;
        }
        unsafe { Pin::new_unchecked(this.fValue.assume_init_mut()) }
    }
    // Source: src/core/SkTLazy.h:69-72
    pub fn set_move<'s>(self: Pin<&'s mut Self>, src: Pin<&mut T>) -> Pin<&'s mut T>
    where
        T: SkTLazyMove,
    {
        let this = unsafe { self.get_unchecked_mut() };
        if this.initialized {
            unsafe { T::move_assign(Pin::new_unchecked(this.fValue.assume_init_mut()), src) };
        } else {
            unsafe { T::move_construct(this.fValue.as_mut_ptr(), src) };
            this.initialized = true;
        }
        unsafe { Pin::new_unchecked(this.fValue.assume_init_mut()) }
    }
    // Source: src/core/SkTLazy.h:77-79
    pub fn reset(self: Pin<&mut Self>) {
        unsafe { self.get_unchecked_mut() }.reset_value();
    }
    // Source: src/core/SkTLazy.h:85-85
    pub fn isValid(&self) -> bool {
        self.initialized
    }
    // Source: src/core/SkTLazy.h:91-94
    pub fn get(self: Pin<&mut Self>) -> Pin<&mut T> {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(this.initialized);
        assert!(this.initialized, "empty native optional value");
        unsafe { Pin::new_unchecked(this.fValue.assume_init_mut()) }
    }
    // Source: src/core/SkTLazy.h:95-98
    pub fn get_ref(&self) -> &T {
        debug_assert!(self.initialized);
        assert!(self.initialized, "empty native optional value");
        unsafe { self.fValue.assume_init_ref() }
    }
}
// Native operator-> projects the same object through get; Rust performs arrow
// dispatch through these in-place dereference implementations.
// Source: src/core/SkTLazy.h:103-106
impl<T: Unpin> DerefMut for SkTLazy<T> {
    fn deref_mut(&mut self) -> &mut T {
        debug_assert!(self.initialized);
        assert!(self.initialized, "empty native optional value");
        unsafe { self.fValue.assume_init_mut() }
    }
}
// Source: src/core/SkTLazy.h:107-110
impl<T> Deref for SkTLazy<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get_ref()
    }
}

impl<T> SkTLazy<T> {
    // Source: src/core/SkTLazy.h:116-116
    pub fn getMaybeNull_ref(&self) -> Option<&T> {
        if self.initialized {
            Some(self.get_ref())
        } else {
            None
        }
    }
    // Source: src/core/SkTLazy.h:117-117
    pub fn getMaybeNull(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        if self.initialized {
            Some(self.get())
        } else {
            None
        }
    }
    // Language projection of optional destruction. Keep has_value true throughout
    // T's destructor, then clear it; also clear on Rust unwind to avoid a second drop.
    fn reset_value(&mut self) {
        if self.initialized {
            struct Clear(*mut bool);
            impl Drop for Clear {
                fn drop(&mut self) {
                    unsafe { *self.0 = false };
                }
            }
            let clear = Clear(ptr::addr_of_mut!(self.initialized));
            unsafe { ptr::drop_in_place(self.fValue.as_mut_ptr()) };
            drop(clear);
        }
    }
}
impl<T> Default for SkTLazy<T> {
    fn default() -> Self {
        Self::new()
    }
}
// Source: src/core/SkTLazy.h:147-147
pub struct SkTCopyOnFirstWrite<'a, T> {
    // Source: src/core/SkTLazy.h:212-213
    fObj: Option<NonNull<T>>,
    fLazy: SkTLazy<T>,
    _initial: PhantomData<&'a T>,
}
impl<'a, T> SkTCopyOnFirstWrite<'a, T> {
    // Source: src/core/SkTLazy.h:149-149
    pub fn from_ref(initial: &'a T) -> Self {
        Self::from_ptr(Some(initial))
    }
    // Source: src/core/SkTLazy.h:151-151
    pub fn from_ptr(initial: Option<&'a T>) -> Self {
        Self {
            fObj: initial.map(NonNull::from),
            fLazy: SkTLazy::new(),
            _initial: PhantomData,
        }
    }
    // Source: src/core/SkTLazy.h:154-154
    pub fn new() -> Self {
        Self::from_ptr(None)
    }
    // Source: src/core/SkTLazy.h:156-156
    /// Safety: destination is new empty construction storage.
    pub unsafe fn copy_construct(self: Pin<&mut Self>, that: &Self)
    where
        T: SkTLazyCopy,
    {
        let _ = self.copy_assign(that);
    }
    // Source: src/core/SkTLazy.h:157-157
    /// Safety: destination is new empty construction storage.
    pub unsafe fn move_construct(self: Pin<&mut Self>, that: Pin<&mut Self>)
    where
        T: SkTLazyMove,
    {
        let _ = self.move_assign(that);
    }
    // Source: src/core/SkTLazy.h:159-163
    pub fn copy_assign<'s>(self: Pin<&'s mut Self>, that: &Self) -> Pin<&'s mut Self>
    where
        T: SkTLazyCopy,
    {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { Pin::new_unchecked(&mut this.fLazy) }.copy_assign(&that.fLazy);
        this.fObj = if this.fLazy.isValid() {
            this.fLazy.getMaybeNull_ref().map(NonNull::from)
        } else {
            that.fObj
        };
        unsafe { Pin::new_unchecked(this) }
    }
    // Source: src/core/SkTLazy.h:165-169
    pub fn move_assign<'s>(self: Pin<&'s mut Self>, that: Pin<&mut Self>) -> Pin<&'s mut Self>
    where
        T: SkTLazyMove,
    {
        let this = unsafe { self.get_unchecked_mut() };
        let source = unsafe { that.get_unchecked_mut() };
        unsafe { Pin::new_unchecked(&mut this.fLazy) }
            .move_assign(unsafe { Pin::new_unchecked(&mut source.fLazy) });
        this.fObj = if this.fLazy.isValid() {
            this.fLazy.getMaybeNull_ref().map(NonNull::from)
        } else {
            source.fObj
        };
        unsafe { Pin::new_unchecked(this) }
    }
    // Source: src/core/SkTLazy.h:172-176
    pub fn init(self: Pin<&mut Self>, initial: &'a T) {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(this.fObj.is_none());
        debug_assert!(!this.fLazy.isValid());
        this.fObj = Some(NonNull::from(initial));
    }
    // Source: src/core/SkTLazy.h:180-185
    pub fn initIfNeeded<Args>(self: Pin<&mut Self>, args: Args)
    where
        T: SkTLazyConstruct<Args>,
    {
        let this = unsafe { self.get_unchecked_mut() };
        if this.fObj.is_none() {
            debug_assert!(!this.fLazy.isValid());
            let value = unsafe { Pin::new_unchecked(&mut this.fLazy) }.init(args);
            this.fObj = Some(NonNull::from(unsafe { value.get_unchecked_mut() }));
        }
    }
    // Source: src/core/SkTLazy.h:190-197
    pub fn writable(self: Pin<&mut Self>) -> Pin<&mut T>
    where
        T: SkTLazyCopy,
    {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(this.fObj.is_some());
        if !this.fLazy.isValid() {
            let source = this.fObj.expect("null native copy-on-first-write source");
            let value =
                unsafe { Pin::new_unchecked(&mut this.fLazy) }.set(unsafe { source.as_ref() });
            this.fObj = Some(NonNull::from(unsafe { value.get_unchecked_mut() }));
        }
        unsafe { Pin::new_unchecked(&mut this.fLazy) }.get()
    }
    // Source: src/core/SkTLazy.h:199-199
    pub fn get(&self) -> Option<&T> {
        self.fObj.map(|p| unsafe { p.as_ref() })
    }
    // Source: src/core/SkTLazy.h:207-207
    pub fn as_ptr(&self) -> *const T {
        self.fObj.map_or(ptr::null(), |p| p.as_ptr().cast_const())
    }
}
impl<T> Default for SkTCopyOnFirstWrite<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}
// Source: src/core/SkTLazy.h:209-209
impl<T> Deref for SkTCopyOnFirstWrite<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
            .expect("null native copy-on-first-write dereference")
    }
}
// Borrowed initial values cross threads only when shareable; owned inline values
// additionally need Send for transfer. No Arc or shared mutable owner is added.
unsafe impl<T: Send + Sync> Send for SkTCopyOnFirstWrite<'_, T> {}
unsafe impl<T: Sync> Sync for SkTCopyOnFirstWrite<'_, T> {}
