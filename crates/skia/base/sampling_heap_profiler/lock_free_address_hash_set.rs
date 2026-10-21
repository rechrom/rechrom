// Source: base/sampling_heap_profiler/lock_free_address_hash_set.h/.cc
use super::lock_free_bloom_filter::LockFreeBloomFilter;
use crate::base::synchronization::lock::Lock;
use core::{
    cell::UnsafeCell,
    ffi::c_void,
    pin::Pin,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};
#[cfg(target_pointer_width = "64")]
type Filter = LockFreeBloomFilter<8>;
#[cfg(target_pointer_width = "32")]
type Filter = LockFreeBloomFilter<5>;
struct Node {
    key: AtomicPtr<c_void>,
    next: *mut Node,
}
struct Bucket {
    head: AtomicPtr<Node>,
    filter: Filter,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContainsResult {
    kFound,
    kNotFound,
    kNotFoundButMatchedInBloomFilter,
}
pub struct LockFreeAddressHashSet {
    lock_: NonNull<Lock>,
    buckets_: Vec<Bucket>,
    size_: UnsafeCell<usize>,
    bucket_mask_: usize,
}
// Native publication allows concurrent readers; mutation requires the same external lock.
unsafe impl Sync for LockFreeAddressHashSet {}
unsafe impl Send for LockFreeAddressHashSet {}
impl LockFreeAddressHashSet {
    /// Lock must remain pinned and alive through all readers and destruction. No concurrent writers.
    pub unsafe fn init_at<'a>(
        out: Pin<&'a mut core::mem::MaybeUninit<Self>>,
        count: usize,
        lock: Pin<&Lock>,
    ) -> Pin<&'a mut Self> {
        let buckets_ = (0..count)
            .map(|_| Bucket {
                head: AtomicPtr::new(core::ptr::null_mut()),
                filter: Filter::new(),
            })
            .collect();
        let value = Self {
            lock_: NonNull::from(lock.get_ref()),
            buckets_,
            size_: UnsafeCell::new(0),
            bucket_mask_: count.wrapping_sub(1),
        };
        // CHECKs occur after actual member construction, as in the native constructor.
        crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"std::has_single_bit(buckets_count)\0".as_ptr().cast(); count.is_power_of_two());
        crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"bucket_mask_ <= std::numeric_limits<uint32_t>::max()\0".as_ptr().cast(); value.bucket_mask_<=u32::MAX as usize);
        let p = out.get_unchecked_mut().write(value);
        Pin::new_unchecked(p)
    }
    fn Hash(key: *mut c_void) -> u32 {
        let random_bits = 0x4bfdb9df5a6f243bu64 as usize;
        ((key as usize as u64).wrapping_mul(random_bits as u64) >> 32) as u32
    }
    pub fn Contains(&self, key: *mut c_void) -> ContainsResult {
        let b = &self.buckets_[Self::Hash(key) as usize & self.bucket_mask_];
        if b.filter.MaybeContains(key) {
            if !unsafe { Self::FindNode(b, key) }.is_null() {
                ContainsResult::kFound
            } else {
                ContainsResult::kNotFoundButMatchedInBloomFilter
            }
        } else {
            ContainsResult::kNotFound
        }
    }
    unsafe fn FindNode(b: &Bucket, key: *mut c_void) -> *mut Node {
        crate::DCHECK!(@location crate::base::location::Location::Current(); @condition b"key != nullptr\0".as_ptr().cast();!key.is_null());
        let mut n = b.head.load(Ordering::Acquire);
        while !n.is_null() {
            if (*n).key.load(Ordering::Relaxed) == key {
                return n;
            }
            n = (*n).next;
        }
        core::ptr::null_mut()
    }
    pub fn buckets_count(&self) -> usize {
        crate::DCHECK!(@location crate::base::location::Location::Current(); @condition b"buckets_.size() == bucket_mask_ + 1\0".as_ptr().cast();self.buckets_.len()==self.bucket_mask_+1);
        self.buckets_.len()
    }
    pub unsafe fn size(&self) -> usize {
        Pin::new_unchecked(self.lock_.as_ref()).AssertAcquired();
        *self.size_.get()
    }
    pub unsafe fn load_factor(&self) -> f32 {
        Pin::new_unchecked(self.lock_.as_ref()).AssertAcquired();
        1.0f32 * self.size() as f32 / self.buckets_.len() as f32
    }
    pub unsafe fn Insert(&self, key: *mut c_void) {
        Pin::new_unchecked(self.lock_.as_ref()).AssertAcquired();
        crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"key != nullptr\0".as_ptr().cast();!key.is_null());
        crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"Contains(key) != ContainsResult::kFound\0".as_ptr().cast();self.Contains(key)!=ContainsResult::kFound);
        let b = &self.buckets_[Self::Hash(key) as usize & self.bucket_mask_];
        b.filter.Add(key);
        *self.size_.get() += 1;
        let mut n = b.head.load(Ordering::Relaxed);
        while !n.is_null() {
            if (*n).key.load(Ordering::Relaxed).is_null() {
                (*n).key.store(key, Ordering::Relaxed);
                return;
            }
            n = (*n).next;
        }
        let mut allocated = Box::<Node>::new_uninit();
        allocated.write(Node {
            key: AtomicPtr::new(key),
            next: b.head.load(Ordering::Relaxed),
        });
        let n = Box::into_raw(allocated.assume_init());
        b.head.store(n, Ordering::Release);
    }
    pub unsafe fn Remove(&self, key: *mut c_void) {
        Pin::new_unchecked(self.lock_.as_ref()).AssertAcquired();
        let b = &self.buckets_[Self::Hash(key) as usize & self.bucket_mask_];
        let mut bits = 0;
        let mut n = b.head.load(Ordering::Relaxed);
        while !n.is_null() {
            let k = (*n).key.load(Ordering::Relaxed);
            if k == key {
                break;
            }
            if !k.is_null() {
                bits |= b.filter.GetBitsForKey(k)
            }
            n = (*n).next;
        }
        crate::DCHECK!(@location crate::base::location::Location::Current(); @condition b"node != nullptr\0".as_ptr().cast();!n.is_null());
        // Native missing-key release domain dereferences null; checked Rust boundary, never fake success.
        let n = NonNull::new(n)
            .expect("Remove requires an existing key")
            .as_ptr();
        (*n).key.store(core::ptr::null_mut(), Ordering::Relaxed);
        *self.size_.get() -= 1;
        let mut n = (*n).next;
        while !n.is_null() {
            let k = (*n).key.load(Ordering::Relaxed);
            crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"node_key != key\0".as_ptr().cast();k!=key);
            if !k.is_null() {
                bits |= b.filter.GetBitsForKey(k)
            }
            n = (*n).next;
        }
        b.filter.AtomicSetBits(bits);
    }
    pub unsafe fn Copy(&self, other: &Self) {
        Pin::new_unchecked(self.lock_.as_ref()).AssertAcquired();
        crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"0u == size()\0".as_ptr().cast();self.size()==0);
        for b in &other.buckets_ {
            let mut n = b.head.load(Ordering::Relaxed);
            while !n.is_null() {
                let k = (*n).key.load(Ordering::Relaxed);
                if !k.is_null() {
                    self.Insert(k)
                }
                n = (*n).next;
            }
        }
    }
}
impl Drop for LockFreeAddressHashSet {
    fn drop(&mut self) {
        for b in &self.buckets_ {
            let mut n = b.head.load(Ordering::Relaxed);
            while !n.is_null() {
                unsafe {
                    let next = (*n).next;
                    drop(Box::from_raw(n));
                    n = next;
                }
            }
        }
    }
}
