use core::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering},
};
// Source: base/rand_util.cc: anonymous namespace original relaxed testing switches.
static g_subsampling_always_sample: AtomicBool = AtomicBool::new(false);
static g_subsampling_never_sample: AtomicBool = AtomicBool::new(false);
// Source: base/rand_util.h/.cc: InsecureRandomGenerator. Cell is the actual mutable const state.
pub struct InsecureRandomGenerator {
    a_: Cell<u64>,
    b_: Cell<u64>,
}
impl InsecureRandomGenerator {
    pub(crate) const fn new() -> Self {
        Self {
            a_: Cell::new(0),
            b_: Cell::new(0),
        }
    }
    pub unsafe fn ReseedForTesting(&self, seed: u64) {
        crate::CHECK_NE!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),137);seed,0u64);
        self.a_.set(seed);
        self.b_.set(seed);
    }
    pub unsafe fn RandUint64(&self) -> u64 {
        if self.a_.get() == 0 && self.b_.get() == 0 {
            loop {
                self.a_.set(RandUint64());
                self.b_.set(RandUint64());
                if self.a_.get() != 0 || self.b_.get() != 0 {
                    break;
                }
            }
        }
        let mut t = self.a_.get();
        let s = self.b_.get();
        self.a_.set(s);
        t ^= t << 23;
        t ^= t >> 17;
        t ^= s ^ (s >> 26);
        self.b_.set(t);
        t.wrapping_add(s)
    }
    pub unsafe fn RandUint32(&self) -> u32 {
        (self.RandUint64() >> 32) as u32
    }
    pub unsafe fn RandDouble(&self) -> f64 {
        ((self.RandUint64() >> 11) as f64) * (1.0 / 9007199254740992.0)
    }
}
pub struct MetricsSubSampler {
    generator_: InsecureRandomGenerator,
}
impl MetricsSubSampler {
    pub(crate) const fn new() -> Self {
        Self {
            generator_: InsecureRandomGenerator::new(),
        }
    }
    pub(crate) unsafe fn ShouldSample(&self, probability: f64) -> bool {
        if g_subsampling_always_sample.load(Ordering::Relaxed) {
            return true;
        }
        if g_subsampling_never_sample.load(Ordering::Relaxed) {
            return false;
        }
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),194);@condition b"probability >= 0 && probability <= 1\0".as_ptr();probability>=0.0&&probability<=1.0);
        self.generator_.RandDouble() < probability
    }
    pub(crate) fn Reseed(&mut self) {
        self.generator_ = InsecureRandomGenerator::new();
    }
    unsafe fn GetSharedMetricsSubsampler() -> *mut Self {
        core::ptr::addr_of_mut!(g_shared_subsampler)
    }
}
// Original thread_local is trivially destructible. Cell state is initialized without allocator work.
#[thread_local]
static mut g_shared_subsampler: MetricsSubSampler = MetricsSubSampler::new();
pub unsafe fn ShouldRecordSubsampledMetric(probability: f64) -> bool {
    (*MetricsSubSampler::GetSharedMetricsSubsampler()).ShouldSample(probability)
}
pub unsafe fn ReseedSharedMetricsSubsampler() {
    (*MetricsSubSampler::GetSharedMetricsSubsampler()).Reseed();
}
pub struct ScopedAlwaysSampleForTesting;
impl ScopedAlwaysSampleForTesting {
    pub unsafe fn new() -> Self {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),210);@condition b"!g_subsampling_always_sample.load(std::memory_order_relaxed)\0".as_ptr();!g_subsampling_always_sample.load(Ordering::Relaxed));
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),211);@condition b"!g_subsampling_never_sample.load(std::memory_order_relaxed)\0".as_ptr();!g_subsampling_never_sample.load(Ordering::Relaxed));
        g_subsampling_always_sample.store(true, Ordering::Relaxed);
        Self
    }
}
impl Drop for ScopedAlwaysSampleForTesting {
    fn drop(&mut self) {
        unsafe {
            crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),217);@condition b"g_subsampling_always_sample.load(std::memory_order_relaxed)\0".as_ptr();g_subsampling_always_sample.load(Ordering::Relaxed));
            crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),211);@condition b"!g_subsampling_never_sample.load(std::memory_order_relaxed)\0".as_ptr();!g_subsampling_never_sample.load(Ordering::Relaxed));
            g_subsampling_always_sample.store(false, Ordering::Relaxed);
        }
    }
}
pub struct ScopedNeverSampleForTesting;
impl ScopedNeverSampleForTesting {
    pub unsafe fn new() -> Self {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),210);@condition b"!g_subsampling_always_sample.load(std::memory_order_relaxed)\0".as_ptr();!g_subsampling_always_sample.load(Ordering::Relaxed));
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),218);@condition b"!g_subsampling_never_sample.load(std::memory_order_relaxed)\0".as_ptr();!g_subsampling_never_sample.load(Ordering::Relaxed));
        g_subsampling_never_sample.store(true, Ordering::Relaxed);
        Self
    }
}
impl Drop for ScopedNeverSampleForTesting {
    fn drop(&mut self) {
        unsafe {
            crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),229);@condition b"!g_subsampling_always_sample\0".as_ptr();!g_subsampling_always_sample.load(Ordering::SeqCst));
            crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/rand_util.cc\0".as_ptr().cast(),230);@condition b"g_subsampling_never_sample\0".as_ptr();g_subsampling_never_sample.load(Ordering::SeqCst));
            g_subsampling_never_sample.store(false, Ordering::Relaxed);
        }
    }
}
// The real entropy producer and RandBytes OS/BoringSSL paths are still pending; no fixed seed/RNG replacement.

// Source: base/rand_util.cc:33-37, actual entropy producer called only on the original lazy-seed edge.
pub unsafe fn RandUint64() -> u64 {
    let mut number = core::mem::MaybeUninit::<u64>::uninit();
    super::rand_util_posix::RandBytes(number.as_mut_ptr().cast(), 8);
    number.assume_init()
}
