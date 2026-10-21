// Source: base/sampling_heap_profiler/lock_free_bloom_filter.h
use core::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_has_atomic = "64"))]
compile_error!("native Bloom requires always-lock-free uint64 atomics");
pub type LockFreeBloomFilterBits = u64;
pub const kMaxLockFreeBloomFilterBits: usize = 64;
pub struct LockFreeBloomFilter<const B: usize, const FAKE: bool = false> {
    bits_: AtomicU64,
}
impl<const B: usize, const FAKE: bool> LockFreeBloomFilter<B, FAKE> {
    pub const fn new() -> Self {
        assert!(B * 6 <= usize::BITS as usize);
        Self {
            bits_: AtomicU64::new(0),
        }
    }
    pub fn GetBitsForKey(&self, key: *mut core::ffi::c_void) -> u64 {
        let int_ptr = key as usize;
        // Native FastHash over the single uintptr_t object's native-order bytes; no replacement hash.
        let mut hash = crate::base::hash::hash::FastHash(&int_ptr.to_ne_bytes());
        let mut bitmask = 0u64;
        for i in 0..B {
            if FAKE {
                hash = int_ptr >> i;
            }
            bitmask |= 1u64 << (hash & 63);
            hash >>= 6;
        }
        bitmask
    }
    pub fn MaybeContains(&self, key: *mut core::ffi::c_void) -> bool {
        let mask = self.GetBitsForKey(key);
        self.bits_.load(Ordering::Relaxed) & mask == mask
    }
    pub fn Add(&self, key: *mut core::ffi::c_void) {
        self.bits_
            .fetch_or(self.GetBitsForKey(key), Ordering::Relaxed);
    }
    pub fn AtomicSetBits(&self, bits: u64) {
        self.bits_.store(bits, Ordering::Relaxed)
    }
    pub fn GetBitsForTesting(&self) -> u64 {
        self.bits_.load(Ordering::Relaxed)
    }
    pub fn CountBits(&self) -> usize {
        self.bits_.load(Ordering::Relaxed).count_ones() as usize
    }
}
