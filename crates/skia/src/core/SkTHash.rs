// Copyright 2015 Google Inc. BSD-style license.
// Native Slot owns initialized T exactly when fHash is nonzero. Generic native
// special-member operations preserve a live moved-from T, including its destructor.
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

// Rust expression of T's actual C++ special members, required only by native
// operations which instantiate them. No Default T, zero bytes or cloning is
// substituted for std::move. A move must leave source a live destructible T.
pub trait NativeMove: Sized {
    fn move_construct(source: &mut Self) -> Self;
    fn move_assign(destination: &mut Self, source: &mut Self);
}
pub trait NativeCopy: Sized {
    fn copy_construct(source: &Self) -> Self;
    fn copy_assign(destination: &mut Self, source: &Self);
}
impl<T: Copy> NativeMove for T {
    fn move_construct(source: &mut Self) -> Self {
        *source
    }
    fn move_assign(destination: &mut Self, source: &mut Self) {
        *destination = *source;
    }
}
impl<T: Copy> NativeCopy for T {
    fn copy_construct(source: &Self) -> Self {
        *source
    }
    fn copy_assign(destination: &mut Self, source: &Self) {
        *destination = *source;
    }
}
// Bind an existing actual non-Copy native owner, not a synthetic demonstration
// value. Its canonical memberwise move transfers Arc/sk_sp while preserving
// source scalars; canonical Clone supplies its existing native copy protocol.
impl NativeMove for crate::src::core::SkPaint::SkPaint {
    fn move_construct(source: &mut Self) -> Self {
        Self::take_from(source)
    }
    fn move_assign(destination: &mut Self, source: &mut Self) {
        destination.assign_move(source);
    }
}
impl NativeCopy for crate::src::core::SkPaint::SkPaint {
    fn copy_construct(source: &Self) -> Self {
        source.clone()
    }
    fn copy_assign(destination: &mut Self, source: &Self) {
        destination.clone_from(source);
    }
}
impl NativeMove for crate::src::core::SkPath::SkPath {
    fn move_construct(source: &mut Self) -> Self {
        let mut moved = Self::default();
        moved.swap(source);
        moved
    }
    fn move_assign(destination: &mut Self, source: &mut Self) {
        *destination = Self::move_construct(source);
    }
}
impl NativeCopy for crate::src::core::SkPath::SkPath {
    fn copy_construct(source: &Self) -> Self {
        source.clone()
    }
    fn copy_assign(destination: &mut Self, source: &Self) {
        destination.clone_from(source);
    }
}

// Explicit binding for native V{} in operator[]. Producers must provide the
// genuine native default constructor; no missing owner/backend is synthesized.
pub trait NativeDefault: Sized {
    fn default_construct() -> Self;
}
macro_rules! native_scalar_default{($($t:ty),* $(,)?)=>{$(impl NativeDefault for $t{fn default_construct()->Self{Self::default()}})*};}
native_scalar_default!(bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);
impl<T> NativeDefault for *const T {
    fn default_construct() -> Self {
        core::ptr::null()
    }
}
impl<T> NativeDefault for *mut T {
    fn default_construct() -> Self {
        core::ptr::null_mut()
    }
}
impl NativeDefault for crate::src::core::SkPaint::SkPaint {
    fn default_construct() -> Self {
        Self::default()
    }
}
// Canonical existing CPU callers: nullable symbols use native nullptr, and
// ProgramUsage::VariableCounts has three explicit native zero initializers.
impl<T> NativeDefault for Option<T> {
    fn default_construct() -> Self {
        None
    }
}
impl NativeDefault for crate::src::sksl::analysis::SkSLProgramUsage::VariableCounts {
    fn default_construct() -> Self {
        Self::default()
    }
}

// Native Traits customization: a pair makes the source's both-or-neither
// ShouldGrow/ShouldShrink requirement explicit without C++ SFINAE.
pub trait HashTraits<T, K: ?Sized> {
    // Native consumers return either const K& or a trivial value key (Digest).
    // Copy excludes invented destructor-bearing key temporaries; neither branch
    // allocates, adds a stored key, nor copies the underlying borrowed owner.
    type Key<'a>: Borrow<K> + Copy
    where
        T: 'a,
        K: 'a;
    fn GetKey<'a>(value: &'a T) -> Self::Key<'a>
    where
        K: 'a;
    fn Hash(key: &K) -> u32;
    fn loadPolicy() -> Option<(fn(i32, i32) -> bool, fn(i32, i32) -> bool)> {
        None
    }
}
// Source: src/core/SkTHash.h:39-40
impl<T, K: ?Sized, Traits> Default for THashTable<T, K, Traits> {
    fn default() -> Self {
        Self {
            fCount: 0,
            fCapacity: 0,
            fSlots: SlotArray::empty(),
            _types: PhantomData,
        }
    }
}
// Source: src/core/SkTHash.h:42-42
impl<T: NativeCopy, K: ?Sized, Traits> Clone for THashTable<T, K, Traits> {
    fn clone(&self) -> Self {
        let mut out = Self::default();
        out.assign_from(self);
        out
    }
}
impl<T, K: ?Sized, Traits> THashTable<T, K, Traits> {
    // Source: src/core/SkTHash.h:43-43
    pub fn move_from(that: &mut Self) -> Self
    where
        T: NativeMove,
    {
        let mut out = Self::default();
        out.assign_move(that);
        out
    }
    // Source: src/core/SkTHash.h:45-55
    pub fn assign_from(&mut self, that: &Self) -> &mut Self
    where
        T: NativeCopy,
    {
        if !std::ptr::eq(self, that) {
            self.fCount = that.fCount;
            self.fCapacity = that.fCapacity;
            self.fSlots = allocate_slots(that.fCapacity);
            for i in 0..self.fCapacity as usize {
                self.fSlots[i].assign_from(&that.fSlots[i]);
            }
        }
        self
    }
    // Source: src/core/SkTHash.h:57-61
    // Source: src/core/SkTHash.h:63-66
    pub fn assign_move(&mut self, that: &mut Self) -> &mut Self {
        if !std::ptr::eq(self, that) {
            self.fCount = that.fCount;
            self.fCapacity = that.fCapacity;
            self.fSlots = std::mem::replace(&mut that.fSlots, SlotArray::empty());
            that.fCount = 0;
            that.fCapacity = 0;
        }
        self
    }
    // Source: src/core/SkTHash.h:69-69
    pub fn reset(&mut self) {
        self.assign_move(&mut Self::default());
    }
    // Source: src/core/SkTHash.h:72-72
    pub fn count(&self) -> i32 {
        self.fCount
    }
    // Source: src/core/SkTHash.h:76-76
    pub fn capacity(&self) -> i32 {
        self.fCapacity
    }
    // Adapted native79: actual Rust Slot can contain a fat trait-object pointer;
    // this reports its actual allocation size, not pinned C++ thin-pointer ABI.
    pub fn approxBytesUsed(&self) -> usize {
        self.fCapacity as usize * std::mem::size_of::<Slot<T>>()
    }
    // Source: src/core/SkTHash.h:82-86
    pub fn swap(&mut self, that: &mut Self) {
        std::mem::swap(&mut self.fCount, &mut that.fCount);
        std::mem::swap(&mut self.fCapacity, &mut that.fCapacity);
        std::mem::swap(&mut self.fSlots, &mut that.fSlots);
    }
    // Source: src/core/SkTHash.h:88-90
    pub fn swap_move(&mut self, that: &mut Self) {
        self.assign_move(that);
    }
}
impl<T, K: ?Sized + PartialEq, Traits: HashTraits<T, K>> THashTable<T, K, Traits> {
    // Source: src/core/SkTHash.h:104-115
    pub fn set(&mut self, mut val: T) -> Option<&mut T>
    where
        T: NativeMove,
    {
        let shouldGrow = if let Some((grow, _)) = Traits::loadPolicy() {
            grow(self.fCount, self.fCapacity)
        } else {
            4 * self.fCount >= 3 * self.fCapacity
        };
        if shouldGrow {
            self.resize(if self.fCapacity > 0 {
                self.fCapacity * 2
            } else {
                4
            });
        }
        let ptr = self.uncheckedSet(&mut val)?;
        // Native callers must supply a growth policy that leaves an empty slot.
        Some(unsafe { &mut *ptr.as_ptr() })
    }
    // Source: src/core/SkTHash.h:118-133
    fn find_index(&self, key: &K) -> Option<usize> {
        let hash = Self::Hash(key);
        let mut index = (hash & (self.fCapacity - 1) as u32) as i32;
        for _ in 0..self.fCapacity {
            let s = &self.fSlots[index as usize];
            if s.empty() {
                return None;
            }
            if hash == s.fHash && key == Traits::GetKey(s.value()).borrow() {
                return Some(index as usize);
            }
            index = self.next(index);
        }
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),131; @condition b"fCapacity == fCount\0".as_ptr(); (self.fCapacity)==(self.fCount));
        None
    }
    // Borrow projection glue for the native pointer-returning find algorithm above.
    pub fn find(&self, key: &K) -> Option<&T> {
        self.find_index(key).map(|i| self.fSlots[i].value())
    }
    // Source: src/core/SkTHash.h:118-133
    // Exclusive Rust projection of native find's mutable T* result.
    pub fn find_mut(&mut self, key: &K) -> Option<&mut T> {
        let i = self.find_index(key)?;
        Some(self.fSlots[i].value_mut())
    }
    // Source: src/core/SkTHash.h:137-142
    // Option<T> is native null-or-pointer for the ResourceCache Rec* domain.
    pub fn findOrNull(&self, key: &K) -> Option<T>
    where
        T: NativeCopy,
    {
        if let Some(p) = self.find(key) {
            return Some(T::copy_construct(p));
        }
        None
    }
    // Source: src/core/SkTHash.h:146-173
    pub fn removeIfExists(&mut self, key: &K) -> bool
    where
        T: NativeMove,
    {
        let hash = Self::Hash(key);
        let mut index = (hash & (self.fCapacity - 1) as u32) as i32;
        for _ in 0..self.fCapacity {
            let s = &self.fSlots[index as usize];
            if s.empty() {
                return false;
            }
            if hash == s.fHash && key == Traits::GetKey(s.value()).borrow() {
                self.removeSlot(index);
                if self.fCapacity > 4 {
                    let shrink = if let Some((_, shrink)) = Traits::loadPolicy() {
                        shrink(self.fCount, self.fCapacity)
                    } else {
                        4 * self.fCount <= self.fCapacity
                    };
                    if shrink {
                        self.resize(self.fCapacity / 2);
                    }
                }
                return true;
            }
            index = self.next(index);
        }
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),171; @condition b"fCapacity == fCount\0".as_ptr(); (self.fCapacity)==(self.fCount));
        false
    }
    // Source: src/core/SkTHash.h:176-178
    pub fn remove(&mut self, key: &K)
    where
        T: NativeMove,
    {
        crate::SkAssertResult!(@source b"src/core/SkTHash.h\0".as_ptr(),177; @condition b"this->removeIfExists(key)\0".as_ptr(); self.removeIfExists(key));
    }
    // Source: src/core/SkTHash.h:182-182
    // Source: src/core/SkTHash.h:184-184
    // Source: src/core/SkTHash.h:187-187
    // Source: src/core/SkTHash.h:189-190
    // Source: src/core/SkTHash.h:192-195
    // Source: src/core/SkTHash.h:197-204
    pub fn resize(&mut self, capacity: i32)
    where
        T: NativeMove,
    {
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),184; @condition b"capacity >= fCount\0".as_ptr(); capacity >= self.fCount);
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),187; @condition b"(capacity & (capacity - 1)) == 0\0".as_ptr(); (capacity & (capacity - 1))==(0));
        let oldCapacity = self.fCapacity;
        #[cfg(sk_debug)]
        let oldCount = self.fCount;
        self.fCount = 0;
        self.fCapacity = capacity;
        let mut oldSlots = std::mem::replace(&mut self.fSlots, allocate_slots(capacity));
        for i in 0..oldCapacity as usize {
            let s = &mut oldSlots[i];
            if s.has_value() {
                self.uncheckedSet(s.moved_value());
            }
        }
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),203; @condition b"fCount == oldCount\0".as_ptr(); (self.fCount)==(oldCount));
    }
    // Source: src/core/SkTHash.h:211-212
    // Source: src/core/SkTHash.h:214-222
    // Source: src/core/SkTHash.h:224-227
    pub fn reserve(&mut self, n: i32)
    where
        T: NativeMove,
    {
        let mut newCapacity = crate::src::core::SkMathPriv::SkNextPow2(n);
        let grow = if let Some((grow, _)) = Traits::loadPolicy() {
            grow(n, newCapacity)
        } else {
            n * 4 > newCapacity * 3
        };
        if grow {
            newCapacity *= 2;
        }
        if newCapacity > self.fCapacity {
            self.resize(newCapacity);
        }
    }
}
impl<T, K: ?Sized, Traits> THashTable<T, K, Traits> {
    // Source: src/core/SkTHash.h:230-237
    /// A callback must preserve every entry's key, as in the native contract.
    pub fn foreach_mut(&mut self, mut f: impl FnMut(&mut T)) {
        for i in 0..self.fCapacity as usize {
            if self.fSlots[i].has_value() {
                f(self.fSlots[i].value_mut());
            }
        }
    }
    // Source: src/core/SkTHash.h:240-247
    pub fn foreach(&self, mut f: impl FnMut(&T)) {
        for i in 0..self.fCapacity as usize {
            if self.fSlots[i].has_value() {
                f(self.fSlots[i].value());
            }
        }
    }
}
// A shared table borrow prevents native-invalidating add/remove during iteration.
// Iterator state contains only the original table borrow and slot index; it never
// copies/moves T, including when the iterator itself is copied/post-incremented.
impl<'a, T, K: ?Sized, Traits> Iter<'a, T, K, Traits> {
    // Source: src/core/SkTHash.h:257-257
    fn new(table: &'a THashTable<T, K, Traits>, slot: i32) -> Self {
        Self { table, slot }
    }
    // Source: src/core/SkTHash.h:259-261
    pub fn MakeBegin(table: &'a THashTable<T, K, Traits>) -> Self {
        Self::new(table, table.firstPopulatedSlot())
    }
    // Source: src/core/SkTHash.h:263-265
    pub fn MakeEnd(table: &'a THashTable<T, K, Traits>) -> Self {
        Self::new(table, table.fCapacity)
    }
    // Source: src/core/SkTHash.h:267-269
    pub fn value(&self) -> &'a T {
        self.table.slot(self.slot)
    }
    // Source: src/core/SkTHash.h:271-273
    pub fn value_ptr(&self) -> *const T {
        self.table.slot(self.slot)
    }
}
// Source: src/core/SkTHash.h:275-275
// Source: src/core/SkTHash.h:277-279
impl<T, K: ?Sized, Traits> PartialEq for Iter<'_, T, K, Traits> {
    fn eq(&self, that: &Self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),277; @condition b"fTable == that.fTable\0".as_ptr(); std::ptr::eq(self.table, that.table));
        self.slot == that.slot
    }
    // Source: src/core/SkTHash.h:281-283
    fn ne(&self, that: &Self) -> bool {
        !self.eq(that)
    }
}
impl<'a, T, K: ?Sized, Traits> Iter<'a, T, K, Traits> {
    // Source: src/core/SkTHash.h:285-288
    pub fn advance(&mut self) -> &mut Self {
        self.slot = self.table.nextPopulatedSlot(self.slot);
        self
    }
    // Source: src/core/SkTHash.h:290-294
    pub fn post_advance(&mut self) -> Self {
        let old = Self::new(self.table, self.slot);
        self.advance();
        old
    }
}
// Source: src/core/SkTHash.h:297-299
// Shared borrow substitutes the native const table pointer; no ABI identity.
pub struct Iter<'a, T, K: ?Sized, Traits> {
    table: &'a THashTable<T, K, Traits>,
    slot: i32,
}
impl<T, K: ?Sized, Traits> THashTable<T, K, Traits> {
    // Source: src/core/SkTHash.h:324-331
    fn firstPopulatedSlot(&self) -> i32 {
        for i in 0..self.fCapacity {
            if self.fSlots[i as usize].has_value() {
                return i;
            }
        }
        self.fCapacity
    }
    // Source: src/core/SkTHash.h:334-341
    fn nextPopulatedSlot(&self, currentSlot: i32) -> i32 {
        for i in currentSlot + 1..self.fCapacity {
            if self.fSlots[i as usize].has_value() {
                return i;
            }
        }
        self.fCapacity
    }
    // Source: src/core/SkTHash.h:344-347
    fn slot(&self, i: i32) -> &T {
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),345; @condition b"fSlots[i].has_value()\0".as_ptr(); self.fSlots[i as usize].has_value());
        self.fSlots[i as usize].value()
    }
}
impl<T, K: ?Sized + PartialEq, Traits: HashTraits<T, K>> THashTable<T, K, Traits> {
    // Source: src/core/SkTHash.h:349-356
    // Source: src/core/SkTHash.h:358-362
    // Source: src/core/SkTHash.h:365-367
    // Source: src/core/SkTHash.h:369-373
    fn uncheckedSet(&mut self, val: &mut T) -> Option<std::ptr::NonNull<T>>
    where
        T: NativeMove,
    {
        let key_view = Traits::GetKey(val);
        let key: &K = key_view.borrow();
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),351; @condition b"key == key\0".as_ptr(); key == key);
        let hash = Self::Hash(key);
        let mut index = (hash & (self.fCapacity - 1) as u32) as i32;
        for _ in 0..self.fCapacity {
            let s = &mut self.fSlots[index as usize];
            if s.empty() {
                s.emplace(val, hash);
                self.fCount += 1;
                return Some(std::ptr::NonNull::from(s.value_mut()));
            }
            if hash == s.fHash && key == Traits::GetKey(s.value()).borrow() {
                s.emplace(val, hash);
                return Some(std::ptr::NonNull::from(s.value_mut()));
            }
            index = self.next(index);
        }
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),371; @condition b"false\0".as_ptr(); false);
        None
    }
    // Source: src/core/SkTHash.h:375-376
    // Source: src/core/SkTHash.h:379-382
    // Source: src/core/SkTHash.h:389-392
    // Source: src/core/SkTHash.h:394-400
    // Source: src/core/SkTHash.h:402-405
    fn removeSlot(&mut self, mut index: i32)
    where
        T: NativeMove,
    {
        self.fCount -= 1;
        loop {
            let emptyIndex = index;
            loop {
                index = self.next(index);
                let s = &self.fSlots[index as usize];
                if s.empty() {
                    self.fSlots[emptyIndex as usize].reset();
                    return;
                }
                let originalIndex = (s.fHash & (self.fCapacity - 1) as u32) as i32;
                if !((index <= originalIndex && originalIndex < emptyIndex)
                    || (originalIndex < emptyIndex && emptyIndex < index)
                    || (emptyIndex < index && index <= originalIndex))
                {
                    break;
                }
            }
            // Native move assignment leaves the source slot engaged and containing
            // a real moved-from T; source hash survives until its later reset.
            if emptyIndex != index {
                let p = self.fSlots.as_mut_ptr();
                unsafe {
                    (*p.add(emptyIndex as usize)).assign_move(&mut *p.add(index as usize));
                }
            }
        }
    }
    // Source: src/core/SkTHash.h:407-411
    fn next(&self, mut index: i32) -> i32 {
        index -= 1;
        if index < 0 {
            index += self.fCapacity;
        }
        index
    }
    // Source: src/core/SkTHash.h:413-416
    fn Hash(key: &K) -> u32 {
        let hash = Traits::Hash(key) & 0xffffffff;
        if hash != 0 {
            hash
        } else {
            1
        }
    }
}
// Source: src/core/SkTHash.h:420-420
impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            fHash: 0,
            fVal: MaybeUninit::uninit(),
        }
    }
}
// Source: src/core/SkTHash.h:421-421
impl<T> Drop for Slot<T> {
    fn drop(&mut self) {
        self.reset();
    }
}
// Source: src/core/SkTHash.h:423-423
impl<T: NativeCopy> Clone for Slot<T> {
    fn clone(&self) -> Self {
        let mut out = Self::default();
        out.assign_from(self);
        out
    }
}
impl<T> Slot<T> {
    // Source: src/core/SkTHash.h:424-439
    // Source: src/core/SkTHash.h:441-444
    fn assign_from(&mut self, that: &Self) -> &mut Self
    where
        T: NativeCopy,
    {
        if std::ptr::eq(self, that) {
            return self;
        }
        if self.fHash != 0 {
            if that.fHash != 0 {
                T::copy_assign(self.value_mut(), that.value());
                self.fHash = that.fHash;
            } else {
                self.reset();
            }
        } else if that.fHash != 0 {
            self.fVal.write(T::copy_construct(that.value()));
            self.fHash = that.fHash;
        }
        self
    }
    // Source: src/core/SkTHash.h:446-446
    fn move_from(that: &mut Self) -> Self
    where
        T: NativeMove,
    {
        let mut out = Self::default();
        out.assign_move(that);
        out
    }
    // Source: src/core/SkTHash.h:447-462
    // Source: src/core/SkTHash.h:464-467
    fn assign_move(&mut self, that: &mut Self) -> &mut Self
    where
        T: NativeMove,
    {
        if std::ptr::eq(self, that) {
            return self;
        }
        if self.fHash != 0 {
            if that.fHash != 0 {
                T::move_assign(self.value_mut(), that.value_mut());
                self.fHash = that.fHash;
            } else {
                self.reset();
            }
        } else if that.fHash != 0 {
            self.fVal.write(T::move_construct(that.value_mut()));
            self.fHash = that.fHash;
        }
        self
    }
    // Source: src/core/SkTHash.h:469-469
    fn value_mut(&mut self) -> &mut T {
        unsafe { self.fVal.assume_init_mut() }
    }
    // Source: src/core/SkTHash.h:470-470
    fn value(&self) -> &T {
        unsafe { self.fVal.assume_init_ref() }
    }
    // Source: src/core/SkTHash.h:471-471
    fn moved_value(&mut self) -> &mut T {
        self.value_mut()
    }
    // Source: src/core/SkTHash.h:472-472
    fn moved_const_value(&self) -> &T {
        self.value()
    }
    // Source: src/core/SkTHash.h:474-479
    fn emplace(&mut self, v: &mut T, h: u32) -> &mut Self
    where
        T: NativeMove,
    {
        self.reset();
        self.fVal.write(T::move_construct(v));
        self.fHash = h;
        self
    }
    // Source: src/core/SkTHash.h:481-481
    fn has_value(&self) -> bool {
        self.fHash != 0
    }
    // Source: src/core/SkTHash.h:482-482
    fn as_bool(&self) -> bool {
        self.has_value()
    }
    // Source: src/core/SkTHash.h:483-483
    fn empty(&self) -> bool {
        !self.has_value()
    }
    // Source: src/core/SkTHash.h:485-490
    fn reset(&mut self) {
        if self.fHash != 0 {
            // Normal native order is destroy T then clear hash. On a Rust
            // unwinding destructor the guard also prevents a second T drop.
            struct ClearHash<'a>(&'a mut u32);
            impl Drop for ClearHash<'_> {
                fn drop(&mut self) {
                    *self.0 = 0;
                }
            }
            let clear = ClearHash(&mut self.fHash);
            unsafe {
                std::ptr::drop_in_place(self.fVal.as_mut_ptr());
            }
            drop(clear);
        }
    }
}
// Source: src/core/SkTHash.h:418-418
// Source: src/core/SkTHash.h:492-492
// Source: src/core/SkTHash.h:495-499
// Dual native mapping: class declaration 418 and member/storage group 492,
// 495-499 form this one Rust type; Rust methods are separate impl items.
// MaybeUninit is the actual inactive union storage: it never initializes T,
// and its own destructor never drops T. Slot::reset is the sole T destructor.
#[repr(C)]
struct Slot<T> {
    fHash: u32,
    fVal: MaybeUninit<T>,
}
// Source: src/core/SkTHash.h:502-505
pub struct THashTable<T, K: ?Sized, Traits> {
    fCount: i32,
    fCapacity: i32,
    fSlots: SlotArray<T>,
    _types: PhantomData<fn(&K) -> Traits>,
}
fn allocate_slots<T>(capacity: i32) -> SlotArray<T> {
    assert!(capacity >= 0);
    SlotArray {
        slots: (0..capacity)
            .map(|_| Slot::default())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}
// unique_ptr<Slot[]> destroys array members in reverse index order. Rust Box
// slice drop is forward, so explicitly finish T destruction first in native
// order; subsequent empty Slot drops have no value/destructor side effects.
struct SlotArray<T> {
    slots: Box<[Slot<T>]>,
}
impl<T> SlotArray<T> {
    fn empty() -> Self {
        Self {
            slots: Box::new([]),
        }
    }
}
impl<T> core::ops::Deref for SlotArray<T> {
    type Target = [Slot<T>];
    fn deref(&self) -> &Self::Target {
        &self.slots
    }
}
impl<T> core::ops::DerefMut for SlotArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slots
    }
}
impl<T> Drop for SlotArray<T> {
    fn drop(&mut self) {
        for slot in self.slots.iter_mut().rev() {
            slot.reset();
        }
    }
}

// Explicit native HashK()/HashT() callable protocol. Default SkGoodHash is pending;
// no guessed hash or standard-library table fallback is supplied. A fresh default
// callable is constructed for every hash, matching the original expression.
pub trait NativeHash<K>: Default {
    fn hash(&self, key: &K) -> u32;
}
// Callable bridge only; canonical checksum owner supplies the actual hash body.
impl<K: crate::src::core::SkChecksum::UniqueObjectRepresentation> NativeHash<K>
    for crate::src::core::SkChecksum::SkGoodHash
{
    fn hash(&self, key: &K) -> u32 {
        crate::src::core::SkChecksum::SkGoodHash::hash(self, key)
    }
}
// Adapted/uncredited selected-ARM64 thin-pointer address bridge. Never hash
// raw provenance-bearing pointer bytes or dereference/name-hash the pointee.
#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
impl<T: Sized> NativeHash<*const T> for crate::src::core::SkChecksum::SkGoodHash {
    fn hash(&self, key: &*const T) -> u32 {
        self.hash_thin_pointer(*key)
    }
}
// Source: src/core/SkTHash.h:509-511
pub type THashMap<K, V, H = crate::src::core::SkChecksum::SkGoodHash> = THashMapStorage<K, V, H>;
// Source: src/core/SkTHash.h:513-513
impl<K, V, H> Default for THashMapStorage<K, V, H> {
    fn default() -> Self {
        Self {
            fTable: THashTable::default(),
        }
    }
}
// Source: src/core/SkTHash.h:515-516
impl<K: NativeCopy, V: NativeCopy, H> Clone for THashMapStorage<K, V, H> {
    fn clone(&self) -> Self {
        Self {
            fTable: self.fTable.clone(),
        }
    }
}
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn move_from(source: &mut Self) -> Self
    where
        K: NativeMove,
        V: NativeMove,
    {
        Self {
            fTable: THashTable::move_from(&mut source.fTable),
        }
    }
    // Source: src/core/SkTHash.h:518-519
    pub fn assign_move(&mut self, source: &mut Self)
    where
        K: NativeMove,
        V: NativeMove,
    {
        self.fTable.assign_move(&mut source.fTable);
    }
    pub fn assign_from(&mut self, source: &Self)
    where
        K: NativeCopy,
        V: NativeCopy,
    {
        self.fTable.assign_from(&source.fTable);
    }
}
// Source: src/core/SkTHash.h:522-526
// The native derived std::pair has ONE actual key/value tuple. ManuallyDrop
// reserves its native reverse member destruction; no extra allocation, proxy
// counter, cloned key or alternate owner is introduced by this layout adapter.
#[repr(transparent)]
pub struct MapPair<K, V> {
    values: core::mem::ManuallyDrop<(K, V)>,
}
impl<K, V> MapPair<K, V> {
    fn tuple(&self) -> &(K, V) {
        &self.values
    }
    fn tuple_mut(&mut self) -> &mut (K, V) {
        &mut self.values
    }
    fn from_move(key: &mut K, value: &mut V) -> Self
    where
        K: NativeMove,
        V: NativeMove,
    {
        Self {
            values: core::mem::ManuallyDrop::new((
                K::move_construct(key),
                V::move_construct(value),
            )),
        }
    }
}
impl<K, V> Drop for MapPair<K, V> {
    fn drop(&mut self) {
        let pair = self.tuple_mut() as *mut (K, V);
        // Native pair destroys second then first, including unwinding out of
        // second's destructor. The guard is only a Rust unwind adaptation.
        struct First<K>(*mut K);
        impl<K> Drop for First<K> {
            fn drop(&mut self) {
                unsafe { core::ptr::drop_in_place(self.0) };
            }
        }
        let first = First(unsafe { core::ptr::addr_of_mut!((*pair).0) });
        unsafe {
            core::ptr::drop_in_place(core::ptr::addr_of_mut!((*pair).1));
        }
        drop(first);
    }
}
impl<K: NativeMove, V: NativeMove> NativeMove for MapPair<K, V> {
    fn move_construct(source: &mut Self) -> Self {
        let (key, value) = source.tuple_mut();
        Self::from_move(key, value)
    }
    fn move_assign(destination: &mut Self, source: &mut Self) {
        let (dk, dv) = destination.tuple_mut();
        let (sk, sv) = source.tuple_mut();
        K::move_assign(dk, sk);
        V::move_assign(dv, sv);
    }
}
impl<K: NativeCopy, V: NativeCopy> NativeCopy for MapPair<K, V> {
    fn copy_construct(source: &Self) -> Self {
        let (key, value) = source.tuple();
        Self {
            values: core::mem::ManuallyDrop::new((
                K::copy_construct(key),
                V::copy_construct(value),
            )),
        }
    }
    fn copy_assign(destination: &mut Self, source: &Self) {
        let (dk, dv) = destination.tuple_mut();
        let (sk, sv) = source.tuple();
        K::copy_assign(dk, sk);
        V::copy_assign(dv, sv);
    }
}
// The real std::pair default/copy constructors exist only when both actual
// member operations exist. Empty outer containers never invoke these.
impl<K: NativeDefault, V: NativeDefault> NativeDefault for MapPair<K, V> {
    fn default_construct() -> Self {
        Self {
            values: core::mem::ManuallyDrop::new((K::default_construct(), V::default_construct())),
        }
    }
}
impl<K: NativeDefault, V: NativeDefault> Default for MapPair<K, V> {
    fn default() -> Self {
        Self::default_construct()
    }
}
impl<K: NativeCopy, V: NativeCopy> Clone for MapPair<K, V> {
    fn clone(&self) -> Self {
        Self::copy_construct(self)
    }
    fn clone_from(&mut self, source: &Self) {
        Self::copy_assign(self, source);
    }
}
pub struct MapPairTraits<H>(PhantomData<H>);
impl<K, V, H: NativeHash<K>> HashTraits<MapPair<K, V>, K> for MapPairTraits<H> {
    type Key<'a>
        = &'a K
    where
        K: 'a,
        V: 'a;
    fn GetKey<'a>(pair: &'a MapPair<K, V>) -> Self::Key<'a>
    where
        K: 'a,
    {
        &pair.tuple().0
    }
    fn Hash(key: &K) -> u32 {
        H::default().hash(key)
    }
}
// Source: src/core/SkTHash.h:528-535
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn from_pairs(pairs: &[(K, V)]) -> Self
    where
        K: NativeCopy + NativeMove + PartialEq,
        V: NativeCopy + NativeMove,
        H: NativeHash<K>,
    {
        let capacity = if pairs.len() >= 4 {
            crate::src::core::SkMathPriv::SkNextPow2((pairs.len() * 4 / 3) as i32)
        } else {
            4
        };
        let mut out = Self::default();
        out.fTable.resize(capacity);
        for (key, value) in pairs {
            let pair = MapPair {
                values: core::mem::ManuallyDrop::new((
                    K::copy_construct(key),
                    V::copy_construct(value),
                )),
            };
            out.fTable.set(pair);
        }
        out
    }
}
// Source: src/core/SkTHash.h:538-538
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn reset(&mut self) {
        self.fTable.reset();
    }
}
// Source: src/core/SkTHash.h:541-541
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn count(&self) -> i32 {
        self.fTable.count()
    }
}
// Source: src/core/SkTHash.h:544-544
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn empty(&self) -> bool {
        self.fTable.count() == 0
    }
}
// Source: src/core/SkTHash.h:547-547
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn approxBytesUsed(&self) -> usize {
        self.fTable.approxBytesUsed()
    }
}
// Source: src/core/SkTHash.h:550-550
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn reserve(&mut self, n: i32)
    where
        K: NativeMove + PartialEq,
        V: NativeMove,
        H: NativeHash<K>,
    {
        self.fTable.reserve(n);
    }
}
// Source: src/core/SkTHash.h:553-553
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn swap(&mut self, other: &mut Self) {
        self.fTable.swap(&mut other.fTable);
    }
}
// Source: src/core/SkTHash.h:554-554
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn swap_move(&mut self, other: &mut Self) {
        self.fTable.swap_move(&mut other.fTable);
    }
}
// Source: src/core/SkTHash.h:560-563
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn set(&mut self, mut key: K, mut value: V) -> &mut V
    where
        K: NativeMove + PartialEq,
        V: NativeMove,
        H: NativeHash<K>,
    {
        let pair = MapPair::from_move(&mut key, &mut value);
        let out = self
            .fTable
            .set(pair)
            .expect("native set requires a populated slot");
        &mut out.tuple_mut().1
    }
}
// Source: src/core/SkTHash.h:567-572
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn find(&self, key: &K) -> Option<&V>
    where
        K: PartialEq,
        H: NativeHash<K>,
    {
        self.fTable.find(key).map(|pair| &pair.tuple().1)
    }
}
// Source: src/core/SkTHash.h:567-572
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn find_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: PartialEq,
        H: NativeHash<K>,
    {
        self.fTable
            .find_index(key)
            .map(|i| &mut self.fTable.fSlots[i].value_mut().tuple_mut().1)
    }
}
// Source: src/core/SkTHash.h:574-579
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn index_or_default(&mut self, key: &K) -> &mut V
    where
        K: NativeCopy + NativeMove + PartialEq,
        V: NativeDefault + NativeMove,
        H: NativeHash<K>,
    {
        if let Some(i) = self.fTable.find_index(key) {
            return &mut self.fTable.fSlots[i].value_mut().tuple_mut().1;
        }
        self.set(K::copy_construct(key), V::default_construct())
    }
}
// Source: src/core/SkTHash.h:582-584
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn remove(&mut self, key: &K)
    where
        K: NativeMove + PartialEq,
        V: NativeMove,
        H: NativeHash<K>,
    {
        self.fTable.remove(key);
    }
}
// Source: src/core/SkTHash.h:587-589
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn removeIfExists(&mut self, key: &K) -> bool
    where
        K: NativeMove + PartialEq,
        V: NativeMove,
        H: NativeHash<K>,
    {
        self.fTable.removeIfExists(key)
    }
}
// Source: src/core/SkTHash.h:592-596
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn foreach_mut(&mut self, mut f: impl FnMut(&K, &mut V)) {
        self.fTable.foreach_mut(|pair| {
            let (key, value) = pair.tuple_mut();
            f(key, value);
        });
    }
}
// Source: src/core/SkTHash.h:599-603
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn foreach(&self, mut f: impl FnMut(&K, &V)) {
        self.fTable.foreach(|pair| {
            let (key, value) = pair.tuple();
            f(key, value);
        });
    }
}
// Source: src/core/SkTHash.h:606-610
impl<K, V, H> THashMapStorage<K, V, H> {
    pub fn foreach_pair(&self, mut f: impl FnMut(&(K, V))) {
        self.fTable.foreach(|pair| f(pair.tuple()));
    }
}
// Source: src/core/SkTHash.h:613-613
// Iterator projects the original tuple subobject for structured binding. Only
// native table pointer/index state is retained; no pair/value copy is made.
pub struct MapIter<'a, K, V, H> {
    inner: Iter<'a, MapPair<K, V>, K, MapPairTraits<H>>,
}
impl<'a, K, V, H> MapIter<'a, K, V, H> {
    pub fn MakeBegin(table: &'a THashTable<MapPair<K, V>, K, MapPairTraits<H>>) -> Self {
        Self {
            inner: Iter::MakeBegin(table),
        }
    }
    pub fn MakeEnd(table: &'a THashTable<MapPair<K, V>, K, MapPairTraits<H>>) -> Self {
        Self {
            inner: Iter::MakeEnd(table),
        }
    }
    pub fn value(&self) -> &'a (K, V) {
        self.inner.value().tuple()
    }
    pub fn value_ptr(&self) -> *const (K, V) {
        self.value()
    }
    pub fn advance(&mut self) -> &mut Self {
        self.inner.advance();
        self
    }
    pub fn post_advance(&mut self) -> Self {
        Self {
            inner: self.inner.post_advance(),
        }
    }
}
impl<K, V, H> PartialEq for MapIter<'_, K, V, H> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl<K, V, H> THashMapStorage<K, V, H> {
    // Source: src/core/SkTHash.h:615-617
    pub fn begin(&self) -> MapIter<'_, K, V, H> {
        MapIter::MakeBegin(&self.fTable)
    }
    // Source: src/core/SkTHash.h:619-621
    pub fn end(&self) -> MapIter<'_, K, V, H> {
        MapIter::MakeEnd(&self.fTable)
    }
}
// Source: src/core/SkTHash.h:624-625
pub struct THashMapStorage<K, V, H> {
    fTable: THashTable<MapPair<K, V>, K, MapPairTraits<H>>,
}
// Source: src/core/SkTHash.h:628-630
pub type THashSet<T, H = crate::src::core::SkChecksum::SkGoodHash> = THashSetStorage<T, H>;
// Source: src/core/SkTHash.h:632-632
impl<T, H> Default for THashSetStorage<T, H> {
    fn default() -> Self {
        Self {
            fTable: THashTable::default(),
        }
    }
}
// Source: src/core/SkTHash.h:634-635
impl<T: NativeCopy, H> Clone for THashSetStorage<T, H> {
    fn clone(&self) -> Self {
        Self {
            fTable: self.fTable.clone(),
        }
    }
}
impl<T, H> THashSetStorage<T, H> {
    pub fn move_from(source: &mut Self) -> Self
    where
        T: NativeMove,
    {
        Self {
            fTable: THashTable::move_from(&mut source.fTable),
        }
    }
    // Source: src/core/SkTHash.h:637-638
    pub fn assign_move(&mut self, source: &mut Self)
    where
        T: NativeMove,
    {
        self.fTable.assign_move(&mut source.fTable);
    }
    pub fn assign_from(&mut self, source: &Self)
    where
        T: NativeCopy,
    {
        self.fTable.assign_from(&source.fTable);
    }
}
// Source: src/core/SkTHash.h:641-648
impl<T, H> THashSetStorage<T, H> {
    pub fn from_values(values: &[T]) -> Self
    where
        T: NativeCopy + NativeMove + PartialEq,
        H: NativeHash<T>,
    {
        let capacity = if values.len() >= 4 {
            crate::src::core::SkMathPriv::SkNextPow2((values.len() * 4 / 3) as i32)
        } else {
            4
        };
        let mut out = Self::default();
        out.fTable.resize(capacity);
        for value in values {
            out.fTable.set(T::copy_construct(value));
        }
        out
    }
}
// Source: src/core/SkTHash.h:651-651
impl<T, H> THashSetStorage<T, H> {
    pub fn reset(&mut self) {
        self.fTable.reset();
    }
}
// Source: src/core/SkTHash.h:654-654
impl<T, H> THashSetStorage<T, H> {
    pub fn count(&self) -> i32 {
        self.fTable.count()
    }
}
// Source: src/core/SkTHash.h:657-657
impl<T, H> THashSetStorage<T, H> {
    pub fn empty(&self) -> bool {
        self.fTable.count() == 0
    }
}
// Source: src/core/SkTHash.h:660-660
impl<T, H> THashSetStorage<T, H> {
    pub fn approxBytesUsed(&self) -> usize {
        self.fTable.approxBytesUsed()
    }
}
// Source: src/core/SkTHash.h:663-663
impl<T, H> THashSetStorage<T, H> {
    pub fn reserve(&mut self, n: i32)
    where
        T: NativeMove + PartialEq,
        H: NativeHash<T>,
    {
        self.fTable.reserve(n);
    }
}
// Source: src/core/SkTHash.h:666-666
impl<T, H> THashSetStorage<T, H> {
    pub fn swap(&mut self, other: &mut Self) {
        self.fTable.swap(&mut other.fTable);
    }
}
// Source: src/core/SkTHash.h:667-667
impl<T, H> THashSetStorage<T, H> {
    pub fn swap_move(&mut self, other: &mut Self) {
        self.fTable.swap_move(&mut other.fTable);
    }
}
// Source: src/core/SkTHash.h:670-670
impl<T, H> THashSetStorage<T, H> {
    pub fn add(&mut self, item: T)
    where
        T: NativeMove + PartialEq,
        H: NativeHash<T>,
    {
        self.fTable.set(item);
    }
}
// Source: src/core/SkTHash.h:673-673
impl<T, H> THashSetStorage<T, H> {
    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
        H: NativeHash<T>,
    {
        self.find(item).is_some()
    }
}
// Source: src/core/SkTHash.h:677-677
impl<T, H> THashSetStorage<T, H> {
    pub fn find(&self, item: &T) -> Option<&T>
    where
        T: PartialEq,
        H: NativeHash<T>,
    {
        self.fTable.find(item)
    }
}
// Source: src/core/SkTHash.h:680-683
impl<T, H> THashSetStorage<T, H> {
    pub fn remove(&mut self, item: &T)
    where
        T: NativeMove + PartialEq,
        H: NativeHash<T>,
    {
        crate::SkASSERT!(@source b"src/core/SkTHash.h\0".as_ptr(),681; @condition b"this->contains(item)\0".as_ptr(); self.contains(item));
        self.fTable.remove(item);
    }
}
// Source: src/core/SkTHash.h:686-689
impl<T, H> THashSetStorage<T, H> {
    pub fn foreach(&self, f: impl FnMut(&T)) {
        self.fTable.foreach(f);
    }
}
// Source: src/core/SkTHash.h:691-695
pub struct SetTraits<H>(PhantomData<H>);
impl<T, H: NativeHash<T>> HashTraits<T, T> for SetTraits<H> {
    type Key<'a>
        = &'a T
    where
        T: 'a;
    fn GetKey<'a>(item: &'a T) -> Self::Key<'a>
    where
        T: 'a,
    {
        item
    }
    fn Hash(item: &T) -> u32 {
        H::default().hash(item)
    }
}
// Source: src/core/SkTHash.h:698-698
pub type SetIter<'a, T, H> = Iter<'a, T, T, SetTraits<H>>;
impl<T, H> THashSetStorage<T, H> {
    // Source: src/core/SkTHash.h:700-702
    pub fn begin(&self) -> SetIter<'_, T, H> {
        Iter::MakeBegin(&self.fTable)
    }
    // Source: src/core/SkTHash.h:704-706
    pub fn end(&self) -> SetIter<'_, T, H> {
        Iter::MakeEnd(&self.fTable)
    }
}
// Source: src/core/SkTHash.h:709-710
pub struct THashSetStorage<T, H> {
    fTable: THashTable<T, T, SetTraits<H>>,
}
