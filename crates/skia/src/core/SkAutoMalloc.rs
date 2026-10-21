// Copyright 2016 Google Inc. BSD-style license.
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ptr::NonNull;
// Rust allocation ownership substitutes for unique_ptr<void,WrapFree>. A released
// block remains an owned value so its allocator/layout cannot be lost at the
// ownership boundary. No C allocator ABI compatibility is claimed.
pub struct Allocation {
    ptr: NonNull<u8>,
    layout: Layout,
}
impl Allocation {
    fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 16).expect("allocation size overflow");
        let ptr =
            NonNull::new(unsafe { alloc(layout) }).unwrap_or_else(|| handle_alloc_error(layout));
        Self { ptr, layout }
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}
impl Drop for Allocation {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}
// Source: src/core/SkAutoMalloc.h:25-28
pub struct SkAutoMalloc {
    fPtr: Option<Allocation>,
    fSize: usize,
}
impl SkAutoMalloc {
    pub fn new(size: usize) -> Self {
        Self {
            fPtr: if size != 0 {
                Some(Allocation::new(size))
            } else {
                None
            },
            fSize: size,
        }
    }
}
// Source: src/core/SkAutoMalloc.h:34-48
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum OnShrink {
    kAlloc_OnShrink,
    kReuse_OnShrink,
}
impl SkAutoMalloc {
    // Source: src/core/SkAutoMalloc.h:53-59
    pub fn reset(&mut self, size: usize, shrink: OnShrink) -> *mut u8 {
        if size != self.fSize && (size > self.fSize || shrink != OnShrink::kReuse_OnShrink) {
            self.fPtr = if size != 0 {
                Some(Allocation::new(size))
            } else {
                None
            };
            self.fSize = size;
        }
        self.get()
    }
    // Source: src/core/SkAutoMalloc.h:64-65
    pub fn get(&self) -> *mut u8 {
        self.fPtr
            .as_ref()
            .map_or(core::ptr::null_mut(), Allocation::as_ptr)
    }
    // Source: src/core/SkAutoMalloc.h:71-74
    pub fn release(&mut self) -> Option<Allocation> {
        self.fSize = 0;
        self.fPtr.take()
    }
}
// Allocation::Drop above performs WrapFree's release of the sole owned block;
// Option<Allocation> is the unique_ptr representation. The wrapper itself has
// no separate copied pointer and therefore cannot double-free after release.
// Source: src/core/SkAutoMalloc.h:90-90
// Rust tracks the inline alternative by None instead of storing a self-pointer.
// Returned inline addresses are invalidated by owner moves/reset/drop, as with
// pointers obtained from a Rust stack array. No pointer is retained internally.
pub struct SkAutoSMalloc<const N: usize> {
    // Source: src/core/SkAutoMalloc.h:172-174
    heap: Option<Allocation>,
    fSize: usize,
    storage: InlineStorage<N>,
}
// Stable const generics cannot express [u32; (N+3)/4]. Three explicit rounding
// bytes retain initialized payload across Rust moves; layout may use four extra
// stack bytes, while the native allocation threshold remains Align4(N).
#[repr(C, align(4))]
struct InlineStorage<const N: usize> {
    bytes: [core::mem::MaybeUninit<u8>; N],
    rounding: [core::mem::MaybeUninit<u8>; 3],
}
impl<const N: usize> SkAutoSMalloc<N> {
    // Source: src/core/SkAutoMalloc.h:96-99
    pub fn new() -> Self {
        Self {
            heap: None,
            fSize: Self::kSize,
            storage: InlineStorage {
                bytes: [core::mem::MaybeUninit::uninit(); N],
                rounding: [core::mem::MaybeUninit::uninit(); 3],
            },
        }
    }
    // Source: src/core/SkAutoMalloc.h:105-109
    pub fn with_size(size: usize) -> Self {
        let mut value = Self::new();
        value.reset(size, OnShrink::kAlloc_OnShrink, None);
        value
    }
    // Source: src/core/SkAutoMalloc.h:126-126
    pub fn get(&mut self) -> *mut u8 {
        match &self.heap {
            Some(block) => block.as_ptr(),
            None => (&mut self.storage as *mut InlineStorage<N>).cast(),
        }
    }
    pub fn get_const(&self) -> *const u8 {
        match &self.heap {
            Some(block) => block.as_ptr(),
            None => (&self.storage as *const InlineStorage<N>).cast(),
        }
    }
    // Source: src/core/SkAutoMalloc.h:133-158
    pub fn reset(
        &mut self,
        mut size: usize,
        shrink: OnShrink,
        didChangeAlloc: Option<&mut bool>,
    ) -> *mut u8 {
        size = if size < Self::kSize {
            Self::kSize
        } else {
            size
        };
        let allocate =
            size != self.fSize && (shrink == OnShrink::kAlloc_OnShrink || size > self.fSize);
        if let Some(changed) = didChangeAlloc {
            *changed = allocate;
        }
        if allocate {
            // The native SMalloc frees before obtaining its replacement.
            drop(self.heap.take());
            if size == Self::kSize {
                crate::SkASSERT!(@source b"src/core/SkAutoMalloc.h\0".as_ptr(), 147; @condition b"fPtr != fStorage\0".as_ptr(); self.fSize != Self::kSize);
            } else {
                self.heap = Some(Allocation::new(size));
            }
            self.fSize = size;
        }
        crate::SkASSERT!(@source b"src/core/SkAutoMalloc.h\0".as_ptr(), 155; @condition b"fSize >= size && fSize >= kSize\0".as_ptr(); self.fSize >= size && self.fSize >= Self::kSize);
        crate::SkASSERT!(@source b"src/core/SkAutoMalloc.h\0".as_ptr(), 156; @condition b"(fPtr == fStorage) || fSize > kSize\0".as_ptr(); self.heap.is_none() || self.fSize > Self::kSize);
        self.get()
    }
    // Source: src/core/SkAutoMalloc.h:162-162
    pub const kSizeAlign4: usize = match N.checked_add(3) {
        Some(n) => n & !3,
        None => panic!("inline size overflow"),
    };
    // Source: src/core/SkAutoMalloc.h:166-167
    #[cfg(SK_BUILD_FOR_GOOGLE3)]
    pub const kMaxBytes: usize = 4 * 1024;
    #[cfg(SK_BUILD_FOR_GOOGLE3)]
    pub const kSize: usize = if N > Self::kMaxBytes {
        Self::kMaxBytes
    } else {
        Self::kSizeAlign4
    };
    // Source: src/core/SkAutoMalloc.h:169-169
    #[cfg(not(SK_BUILD_FOR_GOOGLE3))]
    pub const kSize: usize = Self::kSizeAlign4;
}

// Source: src/core/SkAutoMalloc.h:115-119
impl<const N: usize> Drop for SkAutoSMalloc<N> {
    fn drop(&mut self) {
        drop(self.heap.take());
    }
}
