#![allow(non_snake_case)]
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkRect::SkIRect;
// Source: src/core/SkImageFilterCache.h:23-30
// Source: src/core/SkImageFilterCache.h:35-52
// Keys compare canonical fields; the explicit native-byte codec below is a representation adaptation.
// Rust struct layout is not used for hashing.
#[derive(Clone, Debug)]
pub struct SkImageFilterCacheKey {
    pub fUniqueID: u32,
    fMatrix: SkMatrix,
    pub fClipBounds: SkIRect,
    pub fSrcGenID: u32,
    pub fSrcSubset: SkIRect,
}
impl SkImageFilterCacheKey {
    pub fn new(
        uniqueID: u32,
        matrix: &SkMatrix,
        clipBounds: SkIRect,
        srcGenID: u32,
        srcSubset: SkIRect,
    ) -> Self {
        let key = Self {
            fUniqueID: uniqueID,
            fMatrix: matrix.clone(),
            fClipBounds: clipBounds,
            fSrcGenID: srcGenID,
            fSrcSubset: srcSubset,
        };
        key.fMatrix.getType();
        crate::SkASSERT!(@source b"src/core/SkImageFilterCache.h\0".as_ptr(),36; @condition b"fMatrix.isFinite()\0".as_ptr(); key.fMatrix.isFinite());
        key
    }
}
impl PartialEq for SkImageFilterCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.fUniqueID == other.fUniqueID
            && self.fMatrix == other.fMatrix
            && self.fClipBounds == other.fClipBounds
            && self.fSrcGenID == other.fSrcGenID
            && self.fSrcSubset == other.fSrcSubset
    }
}

use super::SkImageFilter::SkImageFilter;
use super::SkImageFilterTypes::FilterResult;
use super::SkTDynamicHash::{DynamicHashTraits, SkTDynamicHash};
use super::SkTHash::{NativeMove, THashMap};
use super::SkTInternalLList::{Linked, Links, SkTInternalLList};
use std::sync::{Arc, Mutex};
// Source: src/core/SkImageFilterCache.h:59-59
pub const kDefaultTransientSize: usize = 32 * 1024 * 1024;
// Source: src/core/SkImageFilterCache.h:65-65
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateIfNecessary {
    kNo,
    kYes,
}
// Source: src/core/SkImageFilterCache.h:32-34
// Packed native key codec is a representation adaptation, not a Rust ABI/layout claim.
// C++ ctor forces a complete type cache; private fMatrix keeps that invariant after construction.
// fMat[9] + int32 type mask are reconstructed through canonical native read-only operations.
impl SkImageFilterCacheKey {
    fn packed_bytes(&self) -> [u8; 80] {
        let mut words = [0u32; 20];
        words[0] = self.fUniqueID;
        let mut matrix = [0.0f32; 9];
        self.fMatrix.get9(&mut matrix);
        for i in 0..9 {
            words[i + 1] = matrix[i].to_bits();
        }
        words[10] =
            (self.fMatrix.getType() | if self.fMatrix.rectStaysRect() { 16 } else { 0 }) as u32;
        for (i, v) in [
            self.fClipBounds.fLeft,
            self.fClipBounds.fTop,
            self.fClipBounds.fRight,
            self.fClipBounds.fBottom,
        ]
        .into_iter()
        .enumerate()
        {
            words[i + 11] = v as u32;
        }
        words[15] = self.fSrcGenID;
        for (i, v) in [
            self.fSrcSubset.fLeft,
            self.fSrcSubset.fTop,
            self.fSrcSubset.fRight,
            self.fSrcSubset.fBottom,
        ]
        .into_iter()
        .enumerate()
        {
            words[i + 16] = v as u32;
        }
        let mut bytes = [0u8; 80];
        for i in 0..20 {
            bytes[4 * i..4 * i + 4].copy_from_slice(&words[i].to_ne_bytes());
        }
        bytes
    }
    // Source: src/core/SkImageFilterCache.cpp:49-51
    fn native_hash(&self) -> u32 {
        super::SkChecksum::Hash32(&self.packed_bytes(), 0)
    }
}
// Pointer identities are stable native base addresses obtained through borrowed Arc allocations;
// cached values retain no image-filter owner, exactly like the native raw filter pointer.
// Source: src/core/SkImageFilterCache.cpp:33-33
// Native CacheImpl private key alias; retains the same canonical key owner.
type Key = SkImageFilterCacheKey;
type FilterId = usize;
// Source: src/core/SkImageFilterCache.cpp:38-53
struct Value {
    // Rust drops image state before key state, matching native member destruction order.
    fImage: FilterResult,
    fKey: Key,
    fFilter: Option<FilterId>,
    links: Links<Value>,
}
impl Value {
    fn new(key: &SkImageFilterCacheKey, image: &FilterResult, filter: FilterId) -> Self {
        let key = key.clone();
        let image = image.clone();
        Self {
            fKey: key,
            fImage: image,
            fFilter: if filter == 0 { None } else { Some(filter) },
            links: Links::default(),
        }
    }
    fn GetKey(&self) -> &SkImageFilterCacheKey {
        &self.fKey
    }
}
// Source: src/core/SkImageFilterCache.cpp:47-52
impl DynamicHashTraits<Value, Key> for Value {
    fn GetKey(value: &Value) -> &Key {
        &value.fKey
    }
    fn Hash(key: &Key) -> u32 {
        key.native_hash()
    }
}
unsafe impl Linked for Value {
    unsafe fn links(node: *mut Self) -> *mut Links<Self> {
        core::ptr::addr_of_mut!((*node).links)
    }
}
// std::vector<Value*> is an actual pointer sequence. Its native move transfers
// the allocation and leaves an initialized empty vector in the source.
struct ValuePointers(Vec<*mut Value>);
impl NativeMove for ValuePointers {
    fn move_construct(source: &mut Self) -> Self {
        Self(core::mem::take(&mut source.0))
    }
    fn move_assign(destination: &mut Self, source: &mut Self) {
        *destination = Self::move_construct(source);
    }
}
// Field order preserves native reverse member destruction after the explicit
// CacheImpl destructor body; every Value is exclusively owned by its raw Box
// allocation indexed in fLookup. No parallel owner table exists.
struct CacheState {
    // Source: src/core/SkImageFilterCache.cpp:149-149
    fCurrentBytes: usize,
    // Source: src/core/SkImageFilterCache.cpp:147-147
    by_filter: THashMap<FilterId, ValuePointers>,
    // A private Box stabilizes this native list identity even when the public
    // cache is moved after Arc::try_unwrap. It owns only the original list
    // header; Value allocations remain owned solely by lookup's raw entries.
    // Source: src/core/SkImageFilterCache.cpp:145-145
    lru: Box<SkTInternalLList<Value>>,
    // Source: src/core/SkImageFilterCache.cpp:144-144
    lookup: SkTDynamicHash<Value, Key>,
}
// All pointers refer solely to this cache's owned Box allocations. Their image
// owner is Send+Sync. The Mutex excludes every access; no pointer escapes cache
// methods and nodes never move. The private boxed list header stays stationary
// even if the surrounding cache state moves.
unsafe impl Send for CacheState
where
    FilterResult: Send,
    Key: Send,
{
}
impl Drop for CacheState {
    // Source: src/core/SkImageFilterCache.cpp:35-37
    fn drop(&mut self) {
        // foreach is native slot order. Deleting a pointee never mutates slots;
        // subsequent table destruction drops pointer values only, without keys.
        self.lookup.foreach(|value| unsafe {
            drop(Box::from_raw(value));
        });
    }
}
// Source: src/core/SkImageFilterCache.cpp:34-34
// Mutex/container declarations adapt the actual native shared cache owner.
// Source: src/core/SkImageFilterCache.h:57-57
// Source: src/core/SkImageFilterCache.h:61-61
pub struct SkImageFilterCache {
    // Source: src/core/SkImageFilterCache.cpp:148-148
    fMaxBytes: usize,
    // Source: src/core/SkImageFilterCache.cpp:150-150
    fMutex: Mutex<CacheState>,
}
impl SkImageFilterCache {
    // Source: src/core/SkImageFilterCache.cpp:155-157
    // Source: src/core/SkImageFilterCache.h:62-62
    pub fn Create(maxBytes: usize) -> Arc<Self> {
        Arc::new(Self {
            fMaxBytes: maxBytes,
            fMutex: Mutex::new(CacheState {
                lookup: SkTDynamicHash::new(),
                lru: Box::new(SkTInternalLList::new()),
                by_filter: THashMap::default(),
                fCurrentBytes: 0,
            }),
        })
    }
    // Source: src/core/SkImageFilterCache.cpp:55-69
    // Source: src/core/SkImageFilterCache.h:70-71
    pub fn get(&self, key: &SkImageFilterCacheKey, result: &mut FilterResult) -> bool {
        let mut state = self.fMutex.lock().unwrap_or_else(|p| p.into_inner());
        let value = state.lookup.find(key);
        if !value.is_null() {
            unsafe {
                if value != state.lru.head() {
                    state.lru.remove(value);
                    state.lru.addToHead(value);
                }
                result.clone_from(&(*value).fImage);
            }
            return true;
        }
        false
    }
    // Source: src/core/SkImageFilterCache.cpp:71-95
    // Source: src/core/SkImageFilterCache.h:74-75
    pub fn set(
        &self,
        key: &SkImageFilterCacheKey,
        filter: Option<&Arc<dyn SkImageFilter>>,
        result: &FilterResult,
    ) {
        let filter = filter.map_or(0, |f| {
            f.base() as *const super::SkImageFilter_Base::SkImageFilter_Base as usize
        });
        self.set_with_filter_identity(key, filter, result);
    }
    // Native cache records carry a nonowning filter identity; no Arc owner is
    // manufactured from the borrowed virtual receiver of filterImage().
    pub(crate) fn set_with_filter_identity(
        &self,
        key: &SkImageFilterCacheKey,
        filter: usize,
        result: &FilterResult,
    ) {
        let mut state = self.fMutex.lock().unwrap_or_else(|p| p.into_inner());
        let previous = state.lookup.find(key);
        if !previous.is_null() {
            unsafe {
                state.removeInternal(previous);
            }
        }
        let value = Box::into_raw(Box::new(Value::new(key, result, filter)));
        unsafe {
            state.lookup.add(value);
            state.lru.addToHead(value);
        }
        state.fCurrentBytes = state
            .fCurrentBytes
            .wrapping_add(result.image().map_or(0, |i| i.getSize()));
        if let Some(values) = state.by_filter.find_mut(&filter) {
            values.0.push(value);
        } else {
            state.by_filter.set(filter, ValuePointers(vec![value]));
        }
        while state.fCurrentBytes > self.fMaxBytes {
            let tail = state.lru.tail();
            crate::SkASSERT!(@source b"src/core/SkImageFilterCache.cpp\0".as_ptr(),89; @condition b"tail\0".as_ptr(); !tail.is_null());
            if tail == value {
                break;
            }
            unsafe {
                state.removeInternal(tail);
            }
        }
    }
    // Source: src/core/SkImageFilterCache.cpp:97-104
    // Source: src/core/SkImageFilterCache.h:76-76
    pub fn purge(&self) {
        let mut state = self.fMutex.lock().unwrap_or_else(|p| p.into_inner());
        while state.fCurrentBytes > 0 {
            let tail = state.lru.tail();
            crate::SkASSERT!(@source b"src/core/SkImageFilterCache.cpp\0".as_ptr(),101; @condition b"tail\0".as_ptr(); !tail.is_null());
            unsafe {
                state.removeInternal(tail);
            }
        }
    }
    // Source: src/core/SkImageFilterCache.cpp:106-119
    // Source: src/core/SkImageFilterCache.h:77-77
    pub fn purgeByImageFilter(&self, filter: Option<&Arc<dyn SkImageFilter>>) {
        let filter = filter.map_or(0, |f| {
            f.base() as *const super::SkImageFilter_Base::SkImageFilter_Base as usize
        });
        self.purge_by_filter_identity(filter);
    }
    // The destructor has only the native base-subobject address, not a live
    // retained Arc. Consume that same nonowning identity without forging one.
    pub(crate) fn purge_by_filter_identity(&self, filter: usize) {
        let mut state = self.fMutex.lock().unwrap_or_else(|p| p.into_inner());
        let Some(values) = state.by_filter.find(&filter) else {
            return;
        };
        // Native loop retains the same pointer-vector while removeInternal sees
        // nullptr fFilter and therefore cannot erase or relocate that vector.
        let ptr = values.0.as_ptr();
        let count = values.0.len();
        for i in 0..count {
            unsafe {
                let value = ptr.add(i).read();
                (*value).fFilter = None;
                state.removeInternal(value);
            }
        }
        state.by_filter.remove(&filter);
    }

    // Source: src/core/SkImageFilterCache.cpp:121-121
    // Rust diagnostic projection locks its interior state; the native debug count assumes external exclusion.
    // Source: src/core/SkImageFilterCache.h:78-78
    pub fn count(&self) -> usize {
        self.fMutex
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .lookup
            .count() as usize
    }
}
impl CacheState {
    // Source: src/core/SkImageFilterCache.cpp:123-142
    unsafe fn removeInternal(&mut self, value: *mut Value) {
        if let Some(filter) = (*value).fFilter {
            if let Some(values) = self.by_filter.find_mut(&filter) {
                if values.0.len() == 1 && values.0[0] == value {
                    self.by_filter.remove(&filter);
                } else if let Some(index) = values.0.iter().position(|&v| v == value) {
                    values.0.remove(index);
                }
            }
        }
        self.fCurrentBytes = self
            .fCurrentBytes
            .wrapping_sub((*value).fImage.image().map_or(0, |image| image.getSize()));
        self.lru.remove(value);
        self.lookup.remove(&(*value).fKey);
        drop(Box::from_raw(value));
    }
}

// Source: src/core/SkImageFilterCache.cpp:23-27
#[cfg(target_os = "ios")]
const kDefaultCacheSize: usize = 2 * 1024 * 1024;
#[cfg(not(target_os = "ios"))]
const kDefaultCacheSize: usize = 128 * 1024 * 1024;
impl SkImageFilterCache {
    // Source: src/core/SkImageFilterCache.cpp:159-168
    // OnceLock retains the native global baseline owner permanently. Unlike a
    // raw-static cast, it requires the real cached image graph to prove sharing;
    // no unsafe Send/Sync assertion is supplied to erase that dependency.
    // Source: src/core/SkImageFilterCache.h:66-66
    pub fn Get(createIfNecessary: CreateIfNecessary) -> Option<Arc<Self>> {
        static CACHE: std::sync::OnceLock<Arc<SkImageFilterCache>> = std::sync::OnceLock::new();
        if createIfNecessary == CreateIfNecessary::kNo {
            return CACHE.get().cloned();
        }
        Some(
            CACHE
                .get_or_init(|| Self::Create(kDefaultCacheSize))
                .clone(),
        )
    }
}
