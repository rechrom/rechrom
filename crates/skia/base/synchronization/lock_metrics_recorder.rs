use super::lock_metrics_recorder_tags::{LockMetricTag, LockMetricTagList};
use crate::base::threading::{
    platform_thread::PlatformThread, platform_thread_ref::PlatformThreadRef,
};
use core::{
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};
// Actual native direct producers; staged TLS/sampler/container implementations retain their native algorithms.
use crate::base::{
    containers::{flat_map::flat_map, ring_buffer::RingBuffer},
    metrics::histogram_base::HistogramBase,
    rand_util::MetricsSubSampler,
    threading::{thread_local::ThreadLocalOwnedPointer, thread_local_storage::ThreadLocalStorage},
    time::{
        time::{TimeDelta, TimeTicks},
        time_override::TimeTicksNowIgnoringOverride,
    },
};
// Source: base/synchronization/lock_metrics_recorder.h:45-48
#[derive(Clone, Copy)]
pub struct LockMetricSample {
    pub wait_time: TimeDelta,
    pub tags: LockMetricTagList,
}
// Source: base/synchronization/lock_metrics_recorder.h:145-168
// Private members ordered for native reverse-member destruction; no duplicate owners.
pub struct LockMetricsRecorder {
    bound_thread_ref_: PlatformThreadRef,
    tagged_lock_histograms_: flat_map<u64, *mut HistogramBase>,
    subsampler_: MetricsSubSampler,
    unified_sample_buffer_: RingBuffer<LockMetricSample, 512>,
    iterating_in_progress_: bool,
    histogram_suffix_: String,
}
// Source: base/synchronization/lock_metrics_recorder.cc:76-77
static g_tls_slot: AtomicPtr<ThreadLocalOwnedPointer<LockMetricsRecorder>> =
    AtomicPtr::new(ptr::null_mut());
impl LockMetricsRecorder {
    pub const kMaxSamples: usize = 512;
    const kSamplingRatio: f64 = 0.001;
    // Source: base/synchronization/lock_metrics_recorder.cc:107-124
    pub unsafe fn GetForCurrentThread() -> *mut Self {
        let slot = g_tls_slot.load(Ordering::Acquire);
        if slot.is_null() {
            return ptr::null_mut();
        }
        if ThreadLocalStorage::HasBeenDestroyed() {
            return ptr::null_mut();
        }
        core::pin::Pin::new_unchecked(&*slot).Get()
    }
    // Source: base/synchronization/lock_metrics_recorder.h:140-142
    fn CalledOnValidThread(&self) -> bool {
        self.bound_thread_ref_ == PlatformThread::CurrentRef()
    }
    // Source: base/synchronization/lock_metrics_recorder.cc:155-158
    // Source diagnostic uses the actual base/check protocol; Location PC/check stream remain separately pending.
    pub unsafe fn ShouldRecordLockAcquisitionTime(&self) -> bool {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),156);@condition b"CalledOnValidThread()\0".as_ptr();self.CalledOnValidThread());
        !self.iterating_in_progress_ && self.subsampler_.ShouldSample(Self::kSamplingRatio)
    }
    // Source: base/synchronization/lock_metrics_recorder.cc:160-164
    pub unsafe fn RecordLockAcquisitionTime(&mut self, sample: LockMetricSample) {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),162);@condition b"CalledOnValidThread()\0".as_ptr();self.CalledOnValidThread());
        self.unified_sample_buffer_.SaveToBuffer(sample);
    }
    // Source: base/synchronization/lock_metrics_recorder.cc:242-248
    pub unsafe fn DisableRecordingOnCurrentThreadForTesting() {
        let slot = g_tls_slot.load(Ordering::Acquire);
        if !slot.is_null() {
            drop(core::pin::Pin::new_unchecked(&*slot).Set(None));
        }
    }
}
// Source: base/synchronization/lock_metrics_recorder.h:90-137
// Unlike a no-op timer, this retains the actual nullable recorder, samples conditionally,
// measures via the original override-ignoring clock, and saves an actual elapsed sample.
// It is thread-scoped, and cannot be sent to a different thread.
pub struct ScopedLockAcquisitionTimer {
    tags_: LockMetricTagList,
    lock_metrics_: *mut LockMetricsRecorder,
    start_time_: Option<TimeTicks>,
    _thread: PhantomData<*mut ()>,
}
impl ScopedLockAcquisitionTimer {
    // Safety: native TLS recorder lifetime extends through this timer, on the same thread.
    pub unsafe fn new(tags: &LockMetricTagList) -> Self {
        Self::with_recorder(LockMetricsRecorder::GetForCurrentThread(), tags)
    }
    pub unsafe fn CreateForTest(
        lock_metrics: *mut LockMetricsRecorder,
        tags: &LockMetricTagList,
    ) -> Self {
        Self::with_recorder(lock_metrics, tags)
    }
    unsafe fn with_recorder(
        lock_metrics: *mut LockMetricsRecorder,
        tags: &LockMetricTagList,
    ) -> Self {
        let mut result = Self {
            tags_: *tags,
            lock_metrics_: lock_metrics,
            start_time_: None,
            _thread: PhantomData,
        };
        if lock_metrics.is_null() || !(*lock_metrics).ShouldRecordLockAcquisitionTime() {
            return result;
        }
        result.start_time_ = Some(TimeTicksNowIgnoringOverride());
        result
    }
}
impl Drop for ScopedLockAcquisitionTimer {
    fn drop(&mut self) {
        let Some(start_time) = self.start_time_.as_ref() else {
            return;
        };
        unsafe {
            (*self.lock_metrics_).RecordLockAcquisitionTime(LockMetricSample {
                wait_time: TimeTicksNowIgnoringOverride() - *start_time,
                tags: self.tags_,
            });
        }
    }
}

// Source: lock_metrics_recorder.h: implicit aggregate default construction within the native RingBuffer.
impl Default for LockMetricSample {
    fn default() -> Self {
        Self {
            wait_time: TimeDelta::new(),
            tags: LockMetricTagList::empty(),
        }
    }
}
// Source: lock_metrics_recorder.cc: CreateLockHistogram, native string concatenation then factory call.
unsafe fn CreateLockHistogram(tag: &LockMetricTag, suffix: &str) -> *mut HistogramBase {
    let name = crate::base::strings::strcat::StrCat(&[
        "Scheduling.ContendedLockAcquisitionTime.",
        tag.name(),
        ".",
        suffix,
    ]);
    crate::base::metrics::histogram::Histogram::FactoryMicrosecondsTimeGet(
        &name,
        crate::base::time::time::Microseconds(1),
        crate::base::time::time::Seconds(1),
        100,
        HistogramBase::kUmaTargetedHistogramFlag,
    )
}
// Native local-static NoDestructor<vector<string>> is inline persistent owner storage, no Box added.
struct AllowedThreads(core::cell::UnsafeCell<Vec<String>>);
unsafe impl Sync for AllowedThreads {}
unsafe impl Send for AllowedThreads {}
unsafe fn GetAllowedThreads() -> *mut Vec<String> {
    static ALLOWED: std::sync::OnceLock<AllowedThreads> = std::sync::OnceLock::new();
    ALLOWED
        .get_or_init(|| {
            AllowedThreads(core::cell::UnsafeCell::new(
                crate::base::strings::string_split::SplitString(
                    crate::base::features::kRecordLockAcquisitionTimeAllowedThreads.Get(),
                    ",",
                    crate::base::strings::string_split::TRIM_WHITESPACE,
                    crate::base::strings::string_split::SPLIT_WANT_NONEMPTY,
                ),
            ))
        })
        .0
        .get()
}
impl LockMetricsRecorder {
    // Safety: native raw receiver stays live on its bound thread through all synchronous callbacks.
    // Raw receivers keep no whole-object mutable borrow alive across reentrant logging/lock acquisition.
    unsafe fn GetOrCreateHistogram(this: *mut Self, tag: &LockMetricTag) -> *mut HistogramBase {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),129);@condition b"CalledOnValidThread()\0".as_ptr();(*this).CalledOnValidThread());
        let hash = tag.hash();
        if let Some(found) = (*this).tagged_lock_histograms_.find(&hash) {
            return found.1;
        }
        let histogram = CreateLockHistogram(tag, &(*this).histogram_suffix_);
        (*this).tagged_lock_histograms_.insert((hash, histogram));
        histogram
    }
    unsafe fn ReportLockHistogram(this: *mut Self, sample: &LockMetricSample) {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),144);@condition b"CalledOnValidThread()\0".as_ptr();(*this).CalledOnValidThread());
        crate::DCHECK_LE!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),145);sample.tags.size(),LockMetricTagList::kMaxTags);
        for i in 0..sample.tags.size() {
            if let Some(tag) = sample.tags.get(i) {
                let histogram = Self::GetOrCreateHistogram(this, tag);
                (*histogram).AddTimeMicrosecondsGranularity(sample.wait_time);
            }
        }
    }
    pub unsafe fn ForEachSample(this: *mut Self, mut f: impl FnMut(&LockMetricSample)) {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),168);@condition b"CalledOnValidThread()\0".as_ptr();(*this).CalledOnValidThread());
        crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),169);@condition b"!iterating_in_progress_\0".as_ptr();!(*this).iterating_in_progress_);
        let _mark = crate::base::auto_reset::AutoReset::new(
            core::ptr::addr_of_mut!((*this).iterating_in_progress_),
            true,
        );
        {
            let mut it = (*core::ptr::addr_of!((*this).unified_sample_buffer_)).Begin();
            while it.is_valid() {
                f(it.get());
                it.increment();
            }
        }
        (*this).unified_sample_buffer_.Clear();
    }
    pub unsafe fn ReportLockAcquisitionTimes(this: *mut Self) {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),183);@condition b"CalledOnValidThread()\0".as_ptr();(*this).CalledOnValidThread());
        if (*this).iterating_in_progress_ {
            return;
        }
        Self::ForEachSample(this, |sample| unsafe {
            Self::ReportLockHistogram(this, sample)
        });
    }
    // Source: lock_metrics_recorder.cc: native ctor/member initialization order at existing final storage.
    unsafe fn init_at(out: *mut Self, suffix: &str) {
        core::ptr::addr_of_mut!((*out).histogram_suffix_).write(suffix.to_owned());
        core::ptr::addr_of_mut!((*out).iterating_in_progress_).write(false);
        core::ptr::addr_of_mut!((*out).unified_sample_buffer_).write(RingBuffer::new());
        core::ptr::addr_of_mut!((*out).subsampler_).write(MetricsSubSampler::new());
        core::ptr::addr_of_mut!((*out).tagged_lock_histograms_).write(flat_map::new());
        core::ptr::addr_of_mut!((*out).bound_thread_ref_).write(PlatformThread::CurrentRef());
    }
    // Source: lock_metrics_recorder.cc: actual feature/allow-list gating then inline TLS publication.
    pub unsafe fn EnableRecordingOnCurrentThread(suffix: &str) {
        crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/synchronization/lock_metrics_recorder.cc\0".as_ptr().cast(),205);@condition b"!histogram_suffix.empty()\0".as_ptr();!suffix.is_empty());
        if !crate::base::feature_list::FeatureList::IsEnabled(
            &crate::base::features::kRecordLockAcquisitionTime,
        ) {
            return;
        }
        if !(*GetAllowedThreads()).iter().any(|thread| thread == suffix) {
            return;
        }
        static INITIALIZED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        static mut TLS_SLOT: core::mem::MaybeUninit<ThreadLocalOwnedPointer<LockMetricsRecorder>> =
            core::mem::MaybeUninit::uninit();
        INITIALIZED.get_or_init(|| unsafe {
            ThreadLocalOwnedPointer::init_at(core::ptr::addr_of_mut!(TLS_SLOT).cast());
        });
        let slot = core::ptr::addr_of_mut!(TLS_SLOT)
            .cast::<ThreadLocalOwnedPointer<LockMetricsRecorder>>();
        g_tls_slot.store(slot, Ordering::Release);
        let slot = core::pin::Pin::new_unchecked(&*slot);
        if slot.Get().is_null() {
            let mut owner = Box::<Self>::new_uninit();
            Self::init_at(owner.as_mut_ptr(), suffix);
            drop(slot.Set(Some(owner.assume_init())));
        }
    }
    pub unsafe fn SetAllowedThreadsForTesting(allowed_threads: Vec<String>) {
        *GetAllowedThreads() = allowed_threads;
    }
}
// Actual FeatureList/Histogram/StrCat/SplitString direct definitions remain pending. No fake registration or successful reporting.
