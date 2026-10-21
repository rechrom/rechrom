use crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
impl<T: DataChange> SkCachedData<T> {
    // Source: src/core/SkCachedData.h:25-26
    // Source: src/core/SkCachedData.h:28-28
    pub fn size(&self) -> usize {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).fSize
    }
}
impl<T: DataChange> SkCachedData<T> {
    pub fn data(&self) -> *const u8 {
        self.writable_data()
    }
}
impl<T: DataChange> SkCachedData<T> {
    pub fn writable_data(&self) -> *mut u8 {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).fData
    }
}
// Source: src/core/SkCachedData.h:30-30
impl<T: DataChange> Clone for SkCachedData<T> {
    fn clone(&self) -> Self {
        self.internalRef(false)
    }
}
// Source: src/core/SkCachedData.h:31-31
// Native unref is Rust Drop of each actual client/cache handle; see cpp67-72 below.
impl<T: DataChange> SkCachedData<T> {
    // Source: src/core/SkCachedData.h:33-35
    pub fn testing_only_getRefCnt(&self) -> i32 {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).fRefCnt
    }
}
impl<T: DataChange> SkCachedData<T> {
    pub fn testing_only_isLocked(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .fIsLocked
    }
}
impl<T: DataChange> SkCachedData<T> {
    pub fn testing_only_isInCache(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .fInCache
    }
}
// Source: src/core/SkCachedData.h:37-39
impl<T: DataChange> SkCachedData<T> {
    /// Non-owning diagnostic pointer. Retain this cached-data owner while using it,
    /// and synchronize any allocation operations with all other users.
    pub fn diagnostic_only_getDiscardable(
        &self,
    ) -> Option<std::ptr::NonNull<dyn SkDiscardableMemory>> {
        let mut c = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        match &mut c.storage {
            Storage::Discardable(dm) => Some(std::ptr::NonNull::from(dm.as_mut())),
            Storage::Malloc(_) => None,
        }
    }
}
// Source: src/core/SkCachedData.h:43-43
// Native virtual subclass hook becomes a statically owned Rust state object.
pub trait DataChange {
    fn onDataChange(&mut self, _old: *mut u8, _new: *mut u8) {}
}
// Unit state is the actual non-derived base class using its native empty hook.
impl DataChange for () {}
// Source: src/core/SkCachedData.h:46-46
// Source: src/core/SkCachedData.h:48-51
// Source: src/core/SkCachedData.h:53-62
struct Contents<T: DataChange> {
    state: T,
    storage: Storage,
    fData: *mut u8,
    fSize: usize,
    fRefCnt: i32,
    fInCache: bool,
    fIsLocked: bool,
}
enum Storage {
    Malloc(*mut u8),
    Discardable(Box<dyn SkDiscardableMemory>),
}
// Native contents are protected by the same ownership/mutex rules as the original shared cache object.
unsafe impl<T: DataChange + Send> Send for Contents<T> {}
// Each client handle supplies one native ref; a cache handle supplies the unique cache ref.
// Arc controls final allocation destruction, while native counts independently control backing locking.
pub struct SkCachedData<T: DataChange> {
    inner: Arc<Mutex<Contents<T>>>,
    fromCache: bool,
}
impl<T: DataChange> Contents<T> {
    // Source: src/core/SkCachedData.h:73-74
    // Source: src/core/SkCachedData.h:76-79
    fn setData(&mut self, newData: *mut u8) {
        if newData != self.fData {
            self.state.onDataChange(self.fData, newData);
            self.fData = newData;
        }
    }
}
impl<T: DataChange> SkCachedData<T> {
    // Source: src/core/SkCachedData.h:106-106
    pub fn attachToCacheAndRef(&self) -> Self {
        self.internalRef(true)
    }
}
impl<T: DataChange> SkCachedData<T> {
    // Source: src/core/SkCachedData.h:112-112
    pub fn detachFromCacheAndUnref(self) {
        debug_assert!(self.fromCache);
        drop(self);
    }
}
impl<T: DataChange> SkCachedData<T> {
    // Owned-state access substitutes native protected subclass fields; no cache algorithms are delegated.
    pub(crate) fn with_state<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let c = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        f(&c.state)
    }
}
impl<T: DataChange> SkCachedData<T> {
    pub(crate) fn with_state_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut c = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut c.state)
    }
}
// Source: src/core/SkCachedData.cpp:12-21
impl<T: DataChange> SkCachedData<T> {
    /// Takes ownership of malloc-compatible storage; caller must transfer exclusive release ownership.
    pub unsafe fn from_malloc(data: *mut u8, size: usize, state: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Contents {
                state,
                storage: Storage::Malloc(data),
                fData: data,
                fSize: size,
                fRefCnt: 1,
                fInCache: false,
                fIsLocked: true,
            })),
            fromCache: false,
        }
    }
}
impl<T: DataChange> SkCachedData<T> {
    // Source: src/core/SkCachedData.cpp:23-32
    pub fn from_discardable(size: usize, mut dm: Box<dyn SkDiscardableMemory>, state: T) -> Self {
        let data = dm.data();
        Self {
            inner: Arc::new(Mutex::new(Contents {
                state,
                storage: Storage::Discardable(dm),
                fData: data,
                fSize: size,
                fRefCnt: 1,
                fInCache: false,
                fIsLocked: true,
            })),
            fromCache: false,
        }
    }
}
// Source: src/core/SkCachedData.cpp:34-43
impl Drop for Storage {
    fn drop(&mut self) {
        if let Self::Malloc(p) = self {
            unsafe extern "C" {
                fn free(p: *mut c_void);
            }
            unsafe {
                free((*p).cast());
            }
        } /* Box releases the actual native abstract discardable allocation. */
    }
}
impl<T: DataChange> SkCachedData<T> {
    // Source: src/core/SkCachedData.cpp:45-54
    // Source: src/core/SkCachedData.cpp:56-57
    // Source: src/core/SkCachedData.cpp:59-61
    // Source: src/core/SkCachedData.cpp:63-65
    fn internalRef(&self, fromCache: bool) -> Self {
        let mut c = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        c.validate();
        c.inMutexRef(fromCache);
        c.validate();
        Self {
            inner: Arc::clone(&self.inner),
            fromCache,
        }
    }
}
// Source: src/core/SkCachedData.cpp:67-68
// Source: src/core/SkCachedData.cpp:70-72
impl<T: DataChange> Drop for SkCachedData<T> {
    fn drop(&mut self) {
        let mut c = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        c.validate();
        let _delete = c.inMutexUnref(self.fromCache);
        c.validate(); /* Arc destroys only after the guard is released. */
    }
}
impl<T: DataChange> Contents<T> {
    // Source: src/core/SkCachedData.cpp:76-79
    // Source: src/core/SkCachedData.cpp:81-86
    fn inMutexRef(&mut self, fromCache: bool) {
        if self.fRefCnt == 1 && self.fInCache {
            self.inMutexLock();
        }
        self.fRefCnt += 1;
        if fromCache {
            crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),83; @condition b"!fInCache\0".as_ptr(); !self.fInCache);
            self.fInCache = true;
        }
    }
}
impl<T: DataChange> Contents<T> {
    // Source: src/core/SkCachedData.cpp:88-90
    // Source: src/core/SkCachedData.cpp:92-97
    // Source: src/core/SkCachedData.cpp:101-106
    // Source: src/core/SkCachedData.cpp:108-111
    // Source: src/core/SkCachedData.cpp:114-115
    fn inMutexUnref(&mut self, fromCache: bool) -> bool {
        self.fRefCnt -= 1;
        match self.fRefCnt {
            0 => {
                if self.fIsLocked {
                    self.inMutexUnlock();
                }
            }
            1 => {
                if self.fInCache && !fromCache {
                    self.inMutexUnlock();
                }
            }
            _ => {}
        }
        if fromCache {
            crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),109; @condition b"fInCache\0".as_ptr(); self.fInCache);
            self.fInCache = false;
        }
        self.fRefCnt == 0
    }
}
impl<T: DataChange> Contents<T> {
    // Source: src/core/SkCachedData.cpp:117-118
    // Source: src/core/SkCachedData.cpp:120-121
    // Source: src/core/SkCachedData.cpp:123-137
    fn inMutexLock(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),120; @condition b"!fIsLocked\0".as_ptr(); !self.fIsLocked);
        self.fIsLocked = true;
        let data = match &mut self.storage {
            Storage::Malloc(p) => *p,
            Storage::Discardable(dm) => {
                if dm.lock() {
                    let p = dm.data();
                    crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),130; @condition b"ptr\0".as_ptr(); !p.is_null());
                    p
                } else {
                    std::ptr::null_mut()
                }
            }
        };
        self.setData(data);
    }
}
impl<T: DataChange> Contents<T> {
    // Source: src/core/SkCachedData.cpp:139-140
    // Source: src/core/SkCachedData.cpp:142-143
    // Source: src/core/SkCachedData.cpp:145-146
    // Source: src/core/SkCachedData.cpp:148-156
    fn inMutexUnlock(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),142; @condition b"fIsLocked\0".as_ptr(); self.fIsLocked);
        self.fIsLocked = false;
        if let Storage::Discardable(dm) = &mut self.storage {
            if !self.fData.is_null() {
                dm.unlock();
            }
        }
        self.setData(std::ptr::null_mut());
    }
}
impl<T: DataChange> Contents<T> {
    // Source: src/core/SkCachedData.cpp:161-168
    // Source: src/core/SkCachedData.cpp:170-176
    fn validate(&self) {
        #[cfg(sk_debug)]
        if self.fIsLocked {
            crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),163; @condition b"(fInCache && fRefCnt > 1) || !fInCache\0".as_ptr(); (self.fInCache&&self.fRefCnt>1)||!self.fInCache);
            if let Storage::Malloc(p) = self.storage {
                crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),166; @condition b"fData == fStorage.fMalloc\0".as_ptr(); self.fData==p);
            }
        } else {
            crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),173; @condition b"(fInCache && 1 == fRefCnt) || (0 == fRefCnt)\0".as_ptr(); (self.fInCache&&1==self.fRefCnt)||(0==self.fRefCnt));
            crate::SkASSERT!(@source b"src/core/SkCachedData.cpp\0".as_ptr(),174; @condition b"nullptr == fData\0".as_ptr(); self.fData.is_null());
        }
    }
}

impl<T: DataChange> SkCachedData<T> {
    // Source: src/core/SkCachedData.h:85-85
    // Source: src/core/SkCachedData.h:87-87
    // Native debug checks inspect this same contents object. The actual mutex
    // supplies the safe public borrow; internal already-locked callers retain
    // Contents::validate directly and do not recursively acquire it.
    pub fn validate(&self) {
        #[cfg(sk_debug)]
        {
            let contents = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            contents.validate();
        }
    }
}
