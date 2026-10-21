use super::SkCachedData::SkCachedData;
use super::SkMessageBus::Inbox;
use super::SkTHash::THashTable;
use crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::NonNull;
// Copyright 2013 Google Inc. BSD-style license.
// A boxed word buffer replaces the C++ tightly-packed Key header plus derived-class contents.
// Header words retain the native byte representation and ordering used by hashing and comparison.
// Namespace is an identity tag; it is never dereferenced. Contents are owned immutable bytes.
impl Key {
    // Rust allocation/subclass substitution: copy the completed subclass contents into the trailing words.
    pub fn new(nameSpace: *const (), sharedID: u64, contents: &[u32]) -> Self {
        let local_words = 4 + std::mem::size_of::<usize>() / 4;
        let mut words = vec![0; local_words + contents.len()].into_boxed_slice();
        words[local_words..].copy_from_slice(contents);
        let mut key = Self { words };
        key.init(nameSpace, sharedID, contents.len() * 4);
        key
    }
    // Source: src/core/SkResourceCache.h:43-45
    // Source: src/core/SkResourceCache.h:47-48
    pub fn size(&self) -> usize {
        ((self.words[0] as i32) << 2) as usize
    }
    pub fn getNamespace(&self) -> *const () {
        let mut bytes = [0u8; std::mem::size_of::<usize>()];
        for i in 0..bytes.len() / 4 {
            bytes[4 * i..4 * i + 4].copy_from_slice(&self.words[4 + i].to_ne_bytes());
        }
        usize::from_ne_bytes(bytes) as *const ()
    }
    pub fn getSharedID(&self) -> u64 {
        ((self.words[3] as u64) << 32) | self.words[2] as u64
    }
    // Source: src/core/SkResourceCache.h:51-51
    pub fn hash(&self) -> u32 {
        self.words[1]
    }
}
// Source: src/core/SkResourceCache.h:53-62
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        let a = self.as32();
        let b = other.as32();
        for i in 0..self.words[0] as usize {
            if a[i] != b[i] {
                return false;
            }
        }
        true
    }
}

#[derive(Clone)]
pub struct Key {
    words: Box<[u32]>,
}
// Source: src/core/SkResourceCache.h:73-73
impl Key {
    // Native const pointer projects the original immutable packed key storage;
    // a borrowed slice retains its precise allocation and initialized words.
    fn as32(&self) -> &[u32] {
        &self.words
    }
}
// Source: src/core/SkResourceCache.h:76-77
// Source: src/core/SkResourceCache.h:79-80
// Source: src/core/SkResourceCache.h:82-82
// Source: src/core/SkResourceCache.h:84-85
// Source: src/core/SkResourceCache.h:90-90
// Source: src/core/SkResourceCache.h:101-101
// Source: src/core/SkResourceCache.h:104-105
// Source: src/core/SkResourceCache.h:108-109
// Source: src/core/SkResourceCache.h:112-112
// Native abstract record uses trait dispatch; actual per-object list storage stays in RecLinks.
// Links are required rather than fabricated by the cache. Box-owned records must remain at stable addresses.
pub type RecKey = Key;
/// Safety: links must name the same unique embedded base storage for this record.
/// Once attached, the record address, key, and links must remain stable until cache removal;
/// only the owning cache may mutate the links while attached. Key and byte cost
/// remain stable. Shared canBePurged/postAddInstall and external native release
/// callbacks may overlap: implementations must synchronize every mutable field,
/// discardable-memory transition and counter, and must never manufacture a
/// whole-&mut Rec lease while external aliases exist. Send alone is not proof
/// of that callback protocol. Final purgeability publication/unlock must permit
/// immediate record deletion without a surviving receiver/field borrow or any
/// later access; ordinary shared receivers do not establish that lifetime proof.
/// No callback may unwind or reenter the owning cache. Local-cache owners must
/// end all external raw record callbacks before destroying the cache.
pub unsafe trait Rec: std::any::Any + Send {
    fn links(&self) -> &RecLinks;
    fn links_mut(&mut self) -> &mut RecLinks;
    // Native empty base constructor/destructor add no resource lifecycle beyond Rust field destruction.
    fn getHash(&self) -> u32 {
        self.getKey().hash()
    }

    fn getKey(&self) -> &Key;
    fn bytesUsed(&self) -> usize;

    fn canBePurged(&self) -> bool {
        true
    }

    unsafe fn postAddInstall(&self, _payload: *mut std::ffi::c_void) {}
    fn getCategory(&self) -> *const std::ffi::c_char;
    fn diagnostic_only_getDiscardable(
        &self,
    ) -> Option<
        std::ptr::NonNull<
            dyn crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory,
        >,
    > {
        None
    }
}
// Native constructor leaves both pointers uninitialized until addToHead writes them.
// Option<NonNull<dyn Rec>> is the nullable polymorphic pointer language substitution, no ABI claim.
pub struct RecLinks {
    fNext: UnsafeCell<MaybeUninit<Option<NonNull<dyn Rec>>>>,
    fPrev: UnsafeCell<MaybeUninit<Option<NonNull<dyn Rec>>>>,
}
impl RecLinks {
    pub fn new() -> Self {
        Self {
            fNext: UnsafeCell::new(MaybeUninit::uninit()),
            fPrev: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}
// Links are used only while the externally synchronized cache exclusively owns stable boxed records.
// No thread may follow or mutate these pointers without the same cache synchronization.
unsafe impl Send for RecLinks {}
// Only the owning cache's exclusive operation may inspect/mutate these cells;
// shared Rec callbacks never gain safe access to their contents.
unsafe impl Sync for RecLinks {}
impl RecLinks {
    unsafe fn next(&self) -> Option<NonNull<dyn Rec>> {
        (*self.fNext.get()).assume_init()
    }
    unsafe fn prev(&self) -> Option<NonNull<dyn Rec>> {
        (*self.fPrev.get()).assume_init()
    }
    unsafe fn set_next(&self, p: Option<NonNull<dyn Rec>>) {
        (*self.fNext.get()).write(p);
    }
    unsafe fn set_prev(&self, p: Option<NonNull<dyn Rec>>) {
        (*self.fPrev.get()).write(p);
    }
}
impl Drop for RecLinks {
    fn drop(&mut self) {}
}
// Source: src/core/SkResourceCache.h:115-118
#[derive(Clone, Copy)]
pub struct PurgeSharedIDMessage {
    pub fSharedID: u64,
}
// Source: src/core/SkResourceCache.h:120-120
pub type ID = NonNull<dyn Rec>;
// Source: src/core/SkResourceCache.h:132-132
pub type FindVisitor = unsafe fn(&dyn Rec, *mut c_void) -> bool;
// Source: src/core/SkResourceCache.h:138-138
// Factory transfers a real locked discardable allocation or native null.
pub type DiscardableFactory = unsafe fn(usize) -> Option<Box<dyn SkDiscardableMemory>>;
// Source: src/core/SkResourceCache.h:157-157
pub type Visitor = unsafe fn(&dyn Rec, *mut c_void);
impl SkResourceCache {
    // Source: src/core/SkResourceCache.h:226-226
    pub fn getTotalBytesUsed(&self) -> usize {
        self.fTotalBytesUsed
    }
    // Source: src/core/SkResourceCache.h:227-227
    pub fn getTotalByteLimit(&self) -> usize {
        self.fTotalByteLimit
    }
    // Source: src/core/SkResourceCache.h:249-251
    pub fn purgeAll(&mut self) {
        self.purgeAsNeeded(true);
    }
    // Source: src/core/SkResourceCache.h:253-253
    pub fn discardableFactory(&self) -> Option<DiscardableFactory> {
        self.fDiscardableFactory
    }
}
// Source: src/core/SkResourceCache.h:263-264
// Source: src/core/SkResourceCache.h:266-267
// Source: src/core/SkResourceCache.h:269-269
// Source: src/core/SkResourceCache.h:271-274
// Source: src/core/SkResourceCache.h:276-276
// Raw intrusive pointers are nonowning views of stable Box records. The cache
// is the sole Box owner and reclaims each through remove or its destructor.
// Hash is manually destroyed inside the native destructor body, before inbox.
pub struct SkResourceCache {
    fPurgeSharedIDInbox: Inbox<PurgeSharedIDMessage, u32>,
    fHead: Option<NonNull<dyn Rec>>,
    fTail: Option<NonNull<dyn Rec>>,
    fHash: ManuallyDrop<Box<CacheHash>>,
    fDiscardableFactory: Option<DiscardableFactory>,
    fTotalBytesUsed: usize,
    fTotalByteLimit: usize,
    fSingleAllocationByteLimit: usize,
    fCount: i32,
}
// Records require Send; exclusive cache ownership transfers with this object.
// The synchronized wrapper protects every operation. Raw record callbacks must
// obey each concrete record's synchronization and cannot outlive local cache.
unsafe impl Send for SkResourceCache {}
// Source: src/core/SkResourceCache.cpp:35-36
// Source: src/core/SkResourceCache.cpp:39-40
impl crate::src::core::SkMessageBus::RoutedMessage<u32> for PurgeSharedIDMessage {
    fn should_post(&self, _id: u32) -> bool {
        true
    }
}
// Source: src/core/SkResourceCache.cpp:53-54
// Source: src/core/SkResourceCache.cpp:57-60

// Source: src/core/SkResourceCache.cpp:66-69
// Source: src/core/SkResourceCache.cpp:71-73
impl Key {
    fn init(&mut self, nameSpace: *const (), sharedID: u64, dataSize: usize) {
        crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),54; @condition b"SkAlign4(dataSize) == dataSize\0".as_ptr(); dataSize.wrapping_add(3)&!3==dataSize);
        let kUnhashedLocal32s = 2usize;
        let kSharedIDLocal32s = 2usize;
        let kHashedLocal32s = kSharedIDLocal32s + (std::mem::size_of::<usize>() >> 2);
        let kLocal32s = kUnhashedLocal32s + kHashedLocal32s;
        debug_assert_eq!(self.words.len(), kLocal32s + (dataSize >> 2));
        debug_assert!(kLocal32s + (dataSize >> 2) <= i32::MAX as usize);
        self.words[0] = (kLocal32s + (dataSize >> 2)) as u32;
        self.words[2] = (sharedID & 0xffffffff) as u32;
        self.words[3] = (sharedID >> 32) as u32;
        let namespace = (nameSpace as usize).to_ne_bytes();
        for i in 0..namespace.len() / 4 {
            self.words[4 + i] = u32::from_ne_bytes(namespace[4 * i..4 * i + 4].try_into().unwrap());
        }
        let mut hashed = Vec::with_capacity((self.words[0] as usize - kUnhashedLocal32s) << 2);
        for &word in &self.words[kUnhashedLocal32s..] {
            hashed.extend_from_slice(&word.to_ne_bytes());
        }
        self.words[1] = crate::src::core::SkChecksum::Hash32(&hashed, 0);
    }
}
// Source: src/core/SkResourceCache.cpp:76-82
struct CacheHashTraits;
impl super::SkTHash::HashTraits<NonNull<dyn Rec>, Key> for CacheHashTraits {
    fn Hash(key: &Key) -> u32 {
        key.hash()
    }
    type Key<'a> = &'a Key;
    fn GetKey<'a>(rec: &'a NonNull<dyn Rec>) -> Self::Key<'a>
    where
        Key: 'a,
    {
        unsafe { rec.as_ref().getKey() }
    }
}
// Source: src/core/SkResourceCache.cpp:84-85
type CacheHash = THashTable<NonNull<dyn Rec>, Key, CacheHashTraits>;
impl SkResourceCache {
    // Source: src/core/SkResourceCache.cpp:90-96
    // Source: src/core/SkResourceCache.cpp:99-101
    fn init(inbox: Inbox<PurgeSharedIDMessage, u32>) -> Self {
        Self {
            fPurgeSharedIDInbox: inbox,
            fHead: None,
            fTail: None,
            fHash: ManuallyDrop::new(Box::new(CacheHash::default())),
            fTotalBytesUsed: 0,
            fCount: 0,
            fSingleAllocationByteLimit: 0,
            fTotalByteLimit: 0,
            fDiscardableFactory: None,
        }
    }
    // Source: src/core/SkResourceCache.cpp:103-107
    pub unsafe fn with_discardable_factory(factory: Option<DiscardableFactory>) -> Self {
        let mut out = Self::init(Inbox::uninit(0));
        out.fDiscardableFactory = factory;
        out
    }
    // Source: src/core/SkResourceCache.cpp:109-113
    pub fn with_byte_limit(byteLimit: usize) -> Self {
        let mut out = Self::init(Inbox::uninit(0));
        out.fTotalByteLimit = byteLimit;
        out
    }
}
// Source: src/core/SkResourceCache.cpp:115-123
impl Drop for SkResourceCache {
    fn drop(&mut self) {
        unsafe {
            let mut rec = self.fHead;
            while let Some(p) = rec {
                let next = p.as_ref().links().next();
                drop(Box::from_raw(p.as_ptr()));
                rec = next;
            }
            ManuallyDrop::drop(&mut self.fHash);
        }
    }
}
impl SkResourceCache {
    // Source: src/core/SkResourceCache.cpp:127-128
    // Source: src/core/SkResourceCache.cpp:130-141
    /// Safety: context must be valid for visitor, which may not retain the
    /// borrowed record or reenter this cache. Native false marks it stale.
    pub unsafe fn find(&mut self, key: &Key, visitor: FindVisitor, context: *mut c_void) -> bool {
        self.checkMessages();
        if let Some(rec) = self.fHash.findOrNull(key) {
            if visitor(rec.as_ref(), context) {
                self.moveToHead(rec);
                true
            } else {
                self.remove(rec);
                false
            }
        } else {
            false
        }
    }
    // Source: src/core/SkResourceCache.cpp:155-156
    // Source: src/core/SkResourceCache.cpp:158-158
    // Source: src/core/SkResourceCache.cpp:160-162
    // Source: src/core/SkResourceCache.cpp:164-165
    // Source: src/core/SkResourceCache.cpp:167-171
    // Source: src/core/SkResourceCache.cpp:173-175
    // Source: src/core/SkResourceCache.cpp:186-187
    /// Safety: transfer unique Box ownership of a stable initialized native
    /// record. Payload must satisfy postAddInstall's concrete contract. All
    /// callbacks must not reenter this cache; external callback lifetimes must
    /// end before local-cache destruction (global cache is intentionally leaked).
    pub unsafe fn add(&mut self, rec: Box<dyn Rec>, payload: *mut c_void) {
        self.checkMessages();
        debug_assert!(!std::ptr::from_ref(rec.as_ref()).is_null());
        if let Some(prev) = self.fHash.findOrNull(rec.getKey()) {
            if prev.as_ref().canBePurged() {
                self.remove(prev);
            } else {
                prev.as_ref().postAddInstall(payload);
                drop(rec);
                return;
            }
        }
        let rec = NonNull::new_unchecked(Box::into_raw(rec));
        self.addToHead(rec);
        self.fHash.set(rec);
        rec.as_ref().postAddInstall(payload);
        // Native transaction logging is separately excluded; mutation/order
        // and purge are the actual cache algorithm.
        self.purgeAsNeeded(false);
    }
    // Source: src/core/SkResourceCache.cpp:189-192
    // Source: src/core/SkResourceCache.cpp:194-195
    // Source: src/core/SkResourceCache.cpp:197-198
    // Source: src/core/SkResourceCache.cpp:210-211
    unsafe fn remove(&mut self, rec: NonNull<dyn Rec>) {
        crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),190; @condition b"rec->canBePurged()\0".as_ptr(); rec.as_ref().canBePurged());
        let used = rec.as_ref().bytesUsed();
        crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),192; @condition b"used <= fTotalBytesUsed\0".as_ptr(); used<=self.fTotalBytesUsed);
        self.release(rec);
        self.fHash.remove(rec.as_ref().getKey());
        self.fTotalBytesUsed -= used;
        self.fCount -= 1;
        drop(Box::from_raw(rec.as_ptr()));
    }
    // Source: src/core/SkResourceCache.cpp:213-215
    // Source: src/core/SkResourceCache.cpp:217-223
    // Source: src/core/SkResourceCache.cpp:225-229
    // Source: src/core/SkResourceCache.cpp:231-237
    fn purgeAsNeeded(&mut self, forcePurge: bool) {
        unsafe {
            let (countLimit, byteLimit) = if self.fDiscardableFactory.is_some() {
                (1024, u32::MAX as usize)
            } else {
                (i32::MAX, self.fTotalByteLimit)
            };
            let mut rec = self.fTail;
            while let Some(p) = rec {
                if !forcePurge && self.fTotalBytesUsed < byteLimit && self.fCount < countLimit {
                    break;
                }
                let prev = p.as_ref().links().prev();
                if p.as_ref().canBePurged() {
                    self.remove(p);
                }
                rec = prev;
            }
        }
    }
    // Source: src/core/SkResourceCache.cpp:246-249
    // Source: src/core/SkResourceCache.cpp:257-260
    // Source: src/core/SkResourceCache.cpp:263-265
    // Source: src/core/SkResourceCache.cpp:269-271
    // Source: src/core/SkResourceCache.cpp:281-281
    pub fn purgeSharedID(&mut self, sharedID: u64) {
        unsafe {
            if sharedID == 0 {
                return;
            }
            let mut rec = self.fTail;
            while let Some(p) = rec {
                let prev = p.as_ref().links().prev();
                if p.as_ref().getKey().getSharedID() == sharedID && p.as_ref().canBePurged() {
                    self.remove(p);
                }
                rec = prev;
            }
        }
    }
    // Source: src/core/SkResourceCache.cpp:283-283
    // Source: src/core/SkResourceCache.cpp:286-291
    /// Safety: visitor's context is valid, the record borrow cannot escape,
    /// and callbacks must not reenter this same cache.
    pub unsafe fn visitAll(&self, visitor: Visitor, context: *mut c_void) {
        let mut rec = self.fTail;
        while let Some(p) = rec {
            visitor(p.as_ref(), context);
            rec = p.as_ref().links().prev();
        }
    }
    // Source: src/core/SkResourceCache.cpp:295-302
    pub fn setTotalByteLimit(&mut self, newLimit: usize) -> usize {
        let prevLimit = self.fTotalByteLimit;
        self.fTotalByteLimit = newLimit;
        if newLimit < prevLimit {
            self.purgeAsNeeded(false);
        }
        prevLimit
    }
    // Source: src/core/SkResourceCache.cpp:304-305
    // Source: src/core/SkResourceCache.cpp:307-313
    pub fn newCachedData(&mut self, bytes: usize) -> Option<SkCachedData<()>> {
        self.checkMessages();
        if let Some(factory) = self.fDiscardableFactory {
            let dm = unsafe { factory(bytes) };
            dm.map(|dm| SkCachedData::from_discardable(bytes, dm, ()))
        } else {
            unsafe extern "C" {
                fn malloc(bytes: usize) -> *mut c_void;
            }
            let p = unsafe { malloc(bytes) };
            // Native sk_malloc_throw is fail-fast for nonzero allocation failure.
            if p.is_null() && bytes != 0 {
                std::process::abort();
            }
            Some(unsafe { SkCachedData::from_malloc(p.cast(), bytes, ()) })
        }
    }
    // Source: src/core/SkResourceCache.cpp:317-319
    // Source: src/core/SkResourceCache.cpp:321-326
    // Source: src/core/SkResourceCache.cpp:328-332
    // Source: src/core/SkResourceCache.cpp:334-335
    unsafe fn release(&mut self, rec: NonNull<dyn Rec>) {
        let prev = rec.as_ref().links().prev();
        let next = rec.as_ref().links().next();
        if let Some(prev) = prev {
            prev.as_ref().links().set_next(next);
        } else {
            crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),322; @condition b"fHead == rec\0".as_ptr(); self.fHead==Some(rec));
            self.fHead = next;
        }
        if let Some(next) = next {
            next.as_ref().links().set_prev(prev);
        } else {
            self.fTail = prev;
        }
        rec.as_ref().links().set_next(None);
        rec.as_ref().links().set_prev(None);
    }
    // Source: src/core/SkResourceCache.cpp:337-340
    // Source: src/core/SkResourceCache.cpp:342-343
    // Source: src/core/SkResourceCache.cpp:345-345
    // Source: src/core/SkResourceCache.cpp:347-347
    // Source: src/core/SkResourceCache.cpp:349-351
    // Source: src/core/SkResourceCache.cpp:353-354
    unsafe fn moveToHead(&mut self, rec: NonNull<dyn Rec>) {
        if self.fHead == Some(rec) {
            return;
        }
        crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),342; @condition b"fHead\0".as_ptr(); self.fHead.is_some());
        crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),343; @condition b"fTail\0".as_ptr(); self.fTail.is_some());
        self.validate();
        self.release(rec);
        self.fHead.unwrap().as_ref().links().set_prev(Some(rec));
        rec.as_ref().links().set_next(self.fHead);
        self.fHead = Some(rec);
        self.validate();
    }
    // Source: src/core/SkResourceCache.cpp:356-357
    // Source: src/core/SkResourceCache.cpp:359-369
    // Source: src/core/SkResourceCache.cpp:371-372
    unsafe fn addToHead(&mut self, rec: NonNull<dyn Rec>) {
        self.validate();
        rec.as_ref().links().set_prev(None);
        rec.as_ref().links().set_next(self.fHead);
        if let Some(head) = self.fHead {
            head.as_ref().links().set_prev(Some(rec));
        }
        self.fHead = Some(rec);
        if self.fTail.is_none() {
            self.fTail = Some(rec);
        }
        self.fTotalBytesUsed += rec.as_ref().bytesUsed();
        self.fCount += 1;
        self.validate();
    }
    // Source: src/core/SkResourceCache.cpp:377-382
    // Source: src/core/SkResourceCache.cpp:384-389
    // Source: src/core/SkResourceCache.cpp:391-394
    // Source: src/core/SkResourceCache.cpp:396-405
    // Source: src/core/SkResourceCache.cpp:407-414
    // Source: src/core/SkResourceCache.cpp:416-418
    #[cfg(sk_debug)]
    fn validate(&self) {
        unsafe {
            if self.fHead.is_none() {
                crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),379; @condition b"nullptr == fTail\0".as_ptr(); self.fTail.is_none());
                crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),380; @condition b"0 == fTotalBytesUsed\0".as_ptr(); 0==self.fTotalBytesUsed);
                return;
            }
            let head = self.fHead.unwrap();
            let tail = self.fTail.unwrap();
            if head == tail {
                crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),385; @condition b"nullptr == fHead->fPrev\0".as_ptr(); head.as_ref().links().prev().is_none());
                crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),386; @condition b"nullptr == fHead->fNext\0".as_ptr(); head.as_ref().links().next().is_none());
                crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),387; @condition b"fHead->bytesUsed() == fTotalBytesUsed\0".as_ptr(); head.as_ref().bytesUsed()==self.fTotalBytesUsed);
                return;
            }
            crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),391; @condition b"nullptr == fHead->fPrev\0".as_ptr(); head.as_ref().links().prev().is_none());
            crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),392; @condition b"fHead->fNext\0".as_ptr(); head.as_ref().links().next().is_some());
            crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),393; @condition b"nullptr == fTail->fNext\0".as_ptr(); tail.as_ref().links().next().is_none());
            crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),394; @condition b"fTail->fPrev\0".as_ptr(); tail.as_ref().links().prev().is_some());
            let mut used = 0;
            let mut count = 0;
            let mut rec = self.fHead;
            while let Some(p) = rec {
                count += 1;
                used += p.as_ref().bytesUsed();
                crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),402; @condition b"used <= fTotalBytesUsed\0".as_ptr(); used<=self.fTotalBytesUsed);
                rec = p.as_ref().links().next();
            }
            crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),405; @condition b"fCount == count\0".as_ptr(); self.fCount==count);
            rec = self.fTail;
            while let Some(p) = rec {
                crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),409; @condition b"count > 0\0".as_ptr(); count>0);
                count -= 1;
                crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),411; @condition b"used >= rec->bytesUsed()\0".as_ptr(); used>=p.as_ref().bytesUsed());
                used -= p.as_ref().bytesUsed();
                rec = p.as_ref().links().prev();
            }
            crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),416; @condition b"0 == count\0".as_ptr(); 0==count);
            crate::SkASSERT!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),417; @condition b"0 == used\0".as_ptr(); 0==used);
        }
    }
    // Source: src/core/SkResourceCache.h:292-292
    #[cfg(not(sk_debug))]
    fn validate(&self) {}
    // Source: src/core/SkResourceCache.cpp:421-422
    // Source: src/core/SkResourceCache.cpp:424-426
    pub fn dump(&self) {
        self.validate();
        let kind: &[u8] = if self.fDiscardableFactory.is_some() {
            b"discardable\0"
        } else {
            b"malloc\0"
        };
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),424;
            b"SkResourceCache: count=%d bytes=%zu %s\n\0".as_ptr(),
            self.fCount as core::ffi::c_int,self.fTotalBytesUsed,kind.as_ptr());
        }
    }
    // Source: src/core/SkResourceCache.cpp:428-432
    pub fn setSingleAllocationByteLimit(&mut self, newLimit: usize) -> usize {
        let oldLimit = self.fSingleAllocationByteLimit;
        self.fSingleAllocationByteLimit = newLimit;
        oldLimit
    }
    // Source: src/core/SkResourceCache.cpp:434-436
    pub fn getSingleAllocationByteLimit(&self) -> usize {
        self.fSingleAllocationByteLimit
    }
    // Source: src/core/SkResourceCache.cpp:438-438
    // Source: src/core/SkResourceCache.cpp:440-440
    // Source: src/core/SkResourceCache.cpp:444-452
    pub fn getEffectiveSingleAllocationByteLimit(&self) -> usize {
        let mut limit = self.fSingleAllocationByteLimit;
        if self.fDiscardableFactory.is_none() {
            if limit == 0 {
                limit = self.fTotalByteLimit;
            } else {
                limit = limit.min(self.fTotalByteLimit);
            }
        }
        limit
    }
    // Source: src/core/SkResourceCache.cpp:454-460
    pub(crate) fn checkMessages(&mut self) {
        // Existing message-bus API uses an owned Vec for the native TArray.
        let mut msgs = Vec::new();
        unsafe {
            Inbox::poll(NonNull::from(&mut self.fPurgeSharedIDInbox), &mut msgs);
        }
        for i in 0..msgs.len() {
            self.purgeSharedID(msgs[i].fSharedID);
        }
    }
}
// Source: src/core/SkResourceCache.cpp:464-473
// Native local-static initialization is guarded once; the single actual cache
// allocation remains intentionally immortal exactly as the native new pointer.
struct GlobalResourceCache {
    once: crate::include::private::SkOnce::SkOnce,
    cache: UnsafeCell<*mut super::SkSynchronizedResourceCache::SkSynchronizedResourceCache>,
}
unsafe impl Sync for GlobalResourceCache {}
static GLOBAL_RESOURCE_CACHE: GlobalResourceCache = GlobalResourceCache {
    once: crate::include::private::SkOnce::SkOnce::new(),
    cache: UnsafeCell::new(core::ptr::null_mut()),
};
fn get_cache(
) -> core::pin::Pin<&'static super::SkSynchronizedResourceCache::SkSynchronizedResourceCache> {
    unsafe {
        crate::include::private::SkOnce::SkOnce::call_once(
            core::ptr::addr_of!(GLOBAL_RESOURCE_CACHE.once),
            || {
                #[cfg(SK_USE_DISCARDABLE_SCALEDIMAGECACHE)]
            let cache=super::SkSynchronizedResourceCache::SkSynchronizedResourceCache::with_discardable_factory(
                Some(crate::include::private::chromium::SkDiscardableMemory::Create));
                #[cfg(not(SK_USE_DISCARDABLE_SCALEDIMAGECACHE))]
            let cache=super::SkSynchronizedResourceCache::SkSynchronizedResourceCache::with_byte_limit(32*1024*1024);
                let cache = Box::into_raw(Box::new(cache));
                core::pin::Pin::new_unchecked(&mut *cache).init_owner();
                GLOBAL_RESOURCE_CACHE.cache.get().write(cache);
            },
        );
        core::pin::Pin::new_unchecked(&**GLOBAL_RESOURCE_CACHE.cache.get())
    }
}
// Source: src/core/SkResourceCache.cpp:475-477
pub fn GetTotalBytesUsed() -> usize {
    get_cache().getTotalBytesUsed()
}
// Source: src/core/SkResourceCache.cpp:479-481
pub fn GetTotalByteLimit() -> usize {
    get_cache().getTotalByteLimit()
}
// Source: src/core/SkResourceCache.cpp:483-485
pub fn SetTotalByteLimit(newLimit: usize) -> usize {
    get_cache().setTotalByteLimit(newLimit)
}
// Source: src/core/SkResourceCache.cpp:487-489
pub fn GetDiscardableFactory() -> Option<DiscardableFactory> {
    get_cache().discardableFactory()
}
// Source: src/core/SkResourceCache.cpp:491-493
pub fn NewCachedData(bytes: usize) -> Option<SkCachedData<()>> {
    get_cache().newCachedData(bytes)
}
// Source: src/core/SkResourceCache.cpp:495-497
pub fn Dump() {
    get_cache().dump();
}
// Source: src/core/SkResourceCache.cpp:499-501
pub fn SetSingleAllocationByteLimit(size: usize) -> usize {
    get_cache().setSingleAllocationByteLimit(size)
}
// Source: src/core/SkResourceCache.cpp:503-505
pub fn GetSingleAllocationByteLimit() -> usize {
    get_cache().getSingleAllocationByteLimit()
}
// Source: src/core/SkResourceCache.cpp:507-509
pub fn GetEffectiveSingleAllocationByteLimit() -> usize {
    get_cache().getEffectiveSingleAllocationByteLimit()
}
// Source: src/core/SkResourceCache.cpp:511-513
pub fn PurgeAll() {
    get_cache().purgeAll();
}
// Source: src/core/SkResourceCache.cpp:515-517
/// Safety: native CheckMessages is an unguarded nonvirtual base operation.
/// Caller must exclude concurrent cache operations and record callbacks.
pub unsafe fn CheckMessages() {
    unsafe {
        get_cache().checkMessages_unlocked();
    }
}
// Source: src/core/SkResourceCache.cpp:519-521
/// Safety: context/callback obey native find contract; callback cannot reenter global cache.
pub unsafe fn Find(key: &Key, visitor: FindVisitor, context: *mut c_void) -> bool {
    get_cache().find(key, visitor, context)
}
// Source: src/core/SkResourceCache.cpp:523-525
/// Safety: transfer stable unique record ownership; payload/record callbacks
/// obey actual native contracts and cannot unwind/reenter the global cache.
pub unsafe fn Add(rec: Box<dyn Rec>, payload: *mut c_void) {
    get_cache().add(rec, payload);
}
// Source: src/core/SkResourceCache.cpp:527-529
/// Safety: visitor/context valid for every callback, no escaped record borrow
/// or global-cache reentry; native operation mutex remains held throughout.
pub unsafe fn VisitAll(visitor: Visitor, context: *mut c_void) {
    get_cache().visitAll(visitor, context);
}
// Source: src/core/SkResourceCache.cpp:531-535
pub fn PostPurgeSharedID(sharedID: u64) {
    if sharedID == 0 {
        return;
    }
    unsafe {
        crate::src::core::SkMessageBus::SkMessageBus::<PurgeSharedIDMessage, u32>::Post(
            PurgeSharedIDMessage {
                fSharedID: sharedID,
            },
        );
    }
}

// Source: src/core/SkResourceCache.cpp:539-542
unsafe fn dump_visitor(rec: &dyn Rec, _context: *mut c_void) {
    let discardable = rec
        .diagnostic_only_getDiscardable()
        .map_or(core::ptr::null(), |p| p.as_ptr().cast::<()>());
    unsafe {
        crate::SkDebugf!(@source b"src/core/SkResourceCache.cpp\0".as_ptr(),540;
        b"RC: %12s bytes %9zu  discardable %p\n\0".as_ptr(),
        rec.getCategory(),rec.bytesUsed(),discardable);
    }
}
// Source: src/core/SkResourceCache.cpp:544-546
pub fn TestDumpMemoryStatistics() {
    unsafe {
        VisitAll(dump_visitor, core::ptr::null_mut());
    }
}
// Source: src/core/SkResourceCache.cpp:548-562
unsafe fn sk_trace_dump_visitor(rec: &dyn Rec, context: *mut c_void) {
    use crate::include::core::SkTraceMemoryDump::SkTraceMemoryDump;
    let dump = unsafe { &mut **context.cast::<&mut dyn SkTraceMemoryDump>() };
    let name_owner = unsafe {
        super::SkString::SkStringPrintf(
            b"skia/sk_resource_cache/%s_%p\0".as_ptr(),
            rec.getCategory(),
            core::ptr::from_ref(rec).cast::<()>(),
        )
    };
    let name = unsafe { core::ffi::CStr::from_ptr(name_owner.c_str().cast()) };
    if let Some(discardable) = rec.diagnostic_only_getDiscardable() {
        dump.setDiscardableMemoryBacking(name, unsafe { discardable.as_ref() });
        dump.dumpNumericValue(name, c"discardable_size", c"bytes", rec.bytesUsed() as u64);
    } else {
        dump.dumpNumericValue(name, c"size", c"bytes", rec.bytesUsed() as u64);
        dump.setMemoryBacking(name, c"malloc", None);
    }
}
// Source: src/core/SkResourceCache.cpp:564-568
pub fn DumpMemoryStatistics(
    mut dump: &mut dyn crate::include::core::SkTraceMemoryDump::SkTraceMemoryDump,
) {
    // Visitor context points to the real fat reference for this synchronous walk;
    // no metadata is discarded and neither record nor dumper borrow escapes.
    unsafe {
        VisitAll(sk_trace_dump_visitor, core::ptr::from_mut(&mut dump).cast());
    }
}

// Source: src/core/SkResourceCache.cpp:34-34
crate::src::core::SkMessageBus::DECLARE_SKMESSAGEBUS_MESSAGE!(PurgeSharedIDMessage, u32, true);
impl SkResourceCache {
    /// Safety: final pinned cache allocation, not previously initialized. All
    /// constructors returned staging storage; no cache operation may precede
    /// this actual native embedded Inbox construction/registration.
    pub unsafe fn init_owner(self: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            Inbox::construct(core::pin::Pin::new_unchecked(&mut this.fPurgeSharedIDInbox));
        }
    }
}

impl SkResourceCache {
    // Source: src/core/SkResourceCache.h:172-172
    // Source: src/core/SkResourceCache.cpp:544-546
    pub fn TestDumpMemoryStatistics() {
        TestDumpMemoryStatistics();
    }
    // Source: src/core/SkResourceCache.h:177-177
    // Source: src/core/SkResourceCache.cpp:564-568
    pub fn DumpMemoryStatistics(
        dump: &mut dyn crate::include::core::SkTraceMemoryDump::SkTraceMemoryDump,
    ) {
        DumpMemoryStatistics(dump);
    }
}
