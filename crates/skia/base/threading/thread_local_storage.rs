use crate::base::synchronization::lock::{AutoLock, Lock};
use core::{ffi::c_void, ptr, sync::atomic::Ordering};
// Source: base/threading/thread_local_storage.h: PlatformThreadLocalStorage/Slot actual value state.
pub type TLSKey = libc::pthread_key_t;
pub type TLSDestructorFunc = Option<unsafe extern "C-unwind" fn(*mut c_void)>;
pub const TLS_KEY_OUT_OF_INDEXES: TLSKey = 0x7fff_ffff;
#[cfg(base_pthread_key_abi = "u32")]
use core::sync::atomic::AtomicU32 as AtomicTLSKey;
#[cfg(base_pthread_key_abi = "u64")]
use core::sync::atomic::AtomicU64 as AtomicTLSKey;
#[cfg(not(any(base_pthread_key_abi = "u32", base_pthread_key_abi = "u64")))]
compile_error!("actual pthread_key_t atomic ABI input required");
static g_native_tls_key: AtomicTLSKey = AtomicTLSKey::new(TLS_KEY_OUT_OF_INDEXES);
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
enum TlsVectorState {
    kUninitialized = 0,
    kDestroying = 1,
    kDestroyed = 2,
    kInUse = 3,
}
const kVectorStateBitMask: usize = 3;
const kThreadLocalStorageSize: usize = 256;
#[derive(Clone, Copy, PartialEq, Eq)]
enum TlsStatus {
    FREE,
    IN_USE,
}
#[derive(Clone, Copy)]
struct TlsMetadata {
    status: TlsStatus,
    destructor: TLSDestructorFunc,
    version: u32,
    sequence_num: u32,
}
#[derive(Clone, Copy)]
struct TlsVectorEntry {
    data: *mut c_void,
    version: u32,
}
static mut g_tls_metadata: [TlsMetadata; 256] = [TlsMetadata {
    status: TlsStatus::FREE,
    destructor: None,
    version: 0,
    sequence_num: 0,
}; 256];
static mut g_last_assigned_slot: usize = 0;
static mut g_sequence_num: u32 = 0;
const kMaxDestructorIterations: usize = 256;
// Native local-static pointer uses one allocation, leaked for process lifetime. OnceLock is the
// language-level initialization guard, not a replacement for the actual base::Lock algorithm.
unsafe fn GetTLSMetadataLock() -> core::pin::Pin<&'static Lock> {
    static LOCK: std::sync::OnceLock<core::pin::Pin<Box<Lock>>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| {
        let mut storage = Box::<Lock>::new_uninit();
        unsafe {
            Lock::new(core::pin::Pin::new_unchecked(&mut *storage));
            core::pin::Pin::new_unchecked(storage.assume_init())
        }
    })
    .as_ref()
}
unsafe fn SetTlsVectorValue(key: TLSKey, tls_data: *mut TlsVectorEntry, state: TlsVectorState) {
    crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),194);@condition b"tls_data || (state == TlsVectorState::kUninitialized) || (state == TlsVectorState::kDestroyed)\0".as_ptr();!tls_data.is_null()||state==TlsVectorState::kUninitialized||state==TlsVectorState::kDestroyed);
    let tagged = tls_data.map_addr(|p| p | state as usize).cast();
    #[cfg(all(base_is_mac, target_arch = "x86_64"))]
    let _ = tagged.expose_provenance();
    PlatformThreadLocalStorage::SetTLSValue(key, tagged);
}
unsafe fn GetTlsVectorStateAndValueRaw(
    value: *mut c_void,
    entry: *mut *mut TlsVectorEntry,
) -> TlsVectorState {
    if !entry.is_null() {
        *entry = value.map_addr(|p| p & !kVectorStateBitMask).cast();
    }
    match value.addr() & kVectorStateBitMask {
        0 => TlsVectorState::kUninitialized,
        1 => TlsVectorState::kDestroying,
        2 => TlsVectorState::kDestroyed,
        3 => TlsVectorState::kInUse,
        _ => core::hint::unreachable_unchecked(),
    }
}
unsafe fn GetTlsVectorStateAndValue(
    key: TLSKey,
    entry: *mut *mut TlsVectorEntry,
) -> TlsVectorState {
    #[cfg(all(base_is_mac, target_arch = "x86_64"))]
    {
        let value: usize;
        core::arch::asm!("mov {value}, gs:[{key}*8]",value=out(reg)value,key=in(reg)key as usize,options(nostack,readonly,preserves_flags));
        return GetTlsVectorStateAndValueRaw(ptr::with_exposed_provenance_mut(value), entry);
    }
    #[cfg(not(all(base_is_mac, target_arch = "x86_64")))]
    {
        GetTlsVectorStateAndValueRaw(PlatformThreadLocalStorage::GetTLSValue(key), entry)
    }
}
unsafe fn ConstructTlsVector() -> *mut TlsVectorEntry {
    let mut key = g_native_tls_key.load(Ordering::Relaxed);
    if key == TLS_KEY_OUT_OF_INDEXES {
        crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),267);@condition b"PlatformThreadLocalStorage::AllocTLS(&key)\0".as_ptr();PlatformThreadLocalStorage::AllocTLS(&mut key));
        if key == TLS_KEY_OUT_OF_INDEXES {
            let tmp = key;
            crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),276);@condition b"PlatformThreadLocalStorage::AllocTLS(&key) && key != PlatformThreadLocalStorage::TLS_KEY_OUT_OF_INDEXES\0".as_ptr();PlatformThreadLocalStorage::AllocTLS(&mut key)&&key!=TLS_KEY_OUT_OF_INDEXES);
            PlatformThreadLocalStorage::FreeTLS(tmp);
        }
        if g_native_tls_key
            .compare_exchange(
                TLS_KEY_OUT_OF_INDEXES,
                key,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_err()
        {
            PlatformThreadLocalStorage::FreeTLS(key);
            key = g_native_tls_key.load(Ordering::Relaxed);
        }
    }
    crate::CHECK_EQ!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),295);GetTlsVectorStateAndValue(key,ptr::null_mut()),TlsVectorState::kUninitialized);
    let mut stack_data = [TlsVectorEntry {
        data: ptr::null_mut(),
        version: 0,
    }; 256];
    SetTlsVectorValue(key, stack_data.as_mut_ptr(), TlsVectorState::kInUse);
    let storage = Box::<[TlsVectorEntry; 256]>::new_uninit();
    let data = Box::into_raw(storage).cast::<TlsVectorEntry>();
    ptr::copy_nonoverlapping(stack_data.as_ptr(), data, 256);
    SetTlsVectorValue(key, data, TlsVectorState::kInUse);
    data
}
unsafe fn OnThreadExitInternal(tls_data: *mut TlsVectorEntry) {
    crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),324);@condition b"tls_data\0".as_ptr();!tls_data.is_null());
    let mut stack_data = core::mem::MaybeUninit::<[TlsVectorEntry; 256]>::uninit();
    ptr::copy_nonoverlapping(tls_data, stack_data.as_mut_ptr().cast(), 256);
    let mut stack_data = stack_data.assume_init();
    let key = g_native_tls_key.load(Ordering::Relaxed);
    SetTlsVectorValue(key, stack_data.as_mut_ptr(), TlsVectorState::kDestroying);
    drop(Box::from_raw(tls_data.cast::<[TlsVectorEntry; 256]>()));
    let mut remaining_attempts = kMaxDestructorIterations + 1;
    let mut need_to_scan = true;
    while need_to_scan {
        need_to_scan = false;
        let metadata = {
            let _lock = AutoLock::new(GetTLSMetadataLock());
            g_tls_metadata
        };
        #[derive(Clone, Copy)]
        struct OrderedSlot {
            sequence_num: u32,
            slot: u16,
        }
        let mut order = core::array::from_fn::<_, 256, _>(|i| OrderedSlot {
            sequence_num: metadata[i].sequence_num,
            slot: i as u16,
        });
        // Native std::ranges::sort has no stability requirement for equal sequence numbers.
        order.sort_unstable_by(|a, b| b.sequence_num.cmp(&a.sequence_num));
        for ordered in order {
            let slot = ordered.slot as usize;
            let value = stack_data[slot].data;
            if value.is_null()
                || metadata[slot].status == TlsStatus::FREE
                || stack_data[slot].version != metadata[slot].version
            {
                continue;
            }
            let Some(destructor) = metadata[slot].destructor else {
                continue;
            };
            stack_data[slot].data = ptr::null_mut();
            destructor(value);
            need_to_scan = true;
        }
        remaining_attempts -= 1;
        if remaining_attempts == 0 {
            crate::NOTREACHED!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),406));
        }
    }
    SetTlsVectorValue(key, ptr::null_mut(), TlsVectorState::kDestroyed);
}
pub struct PlatformThreadLocalStorage;
impl PlatformThreadLocalStorage {
    pub unsafe fn GetTLSValue(key: TLSKey) -> *mut c_void {
        libc::pthread_getspecific(key)
    }
    // Actual POSIX two-pass destructor, not a Rust thread_local allocation substitute.
    pub unsafe extern "C" fn OnThreadExit(value: *mut c_void) {
        let mut vector = ptr::null_mut();
        let state = GetTlsVectorStateAndValueRaw(value, &mut vector);
        if state == TlsVectorState::kDestroyed {
            let key = g_native_tls_key.load(Ordering::Relaxed);
            SetTlsVectorValue(key, ptr::null_mut(), TlsVectorState::kUninitialized);
            return;
        }
        OnThreadExitInternal(vector);
    }
    // Source: base/threading/thread_local_storage_posix.cc: original pthread key operations.
    pub unsafe fn AllocTLS(key: *mut TLSKey) -> bool {
        libc::pthread_key_create(key, Some(Self::OnThreadExit)) == 0
    }
    pub unsafe fn FreeTLS(key: TLSKey) {
        let ret = libc::pthread_key_delete(key);
        crate::DCHECK_EQ!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage_posix.cc\0".as_ptr().cast(),18);ret,0);
    }
    pub unsafe fn SetTLSValue(key: TLSKey, value: *mut c_void) {
        let ret = libc::pthread_setspecific(key, value);
        crate::DCHECK_EQ!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage_posix.cc\0".as_ptr().cast(),23);ret,0);
    }
}
pub struct ThreadLocalStorage;
impl ThreadLocalStorage {
    pub unsafe fn HasBeenDestroyed() -> bool {
        let key = g_native_tls_key.load(Ordering::Relaxed);
        if key == TLS_KEY_OUT_OF_INDEXES {
            return false;
        }
        let state = GetTlsVectorStateAndValue(key, ptr::null_mut());
        state == TlsVectorState::kDestroying || state == TlsVectorState::kDestroyed
    }
}
pub struct Slot {
    slot_: usize,
    version_: u32,
}
impl Slot {
    const kInvalidSlotValue: usize = usize::MAX;
    pub unsafe fn new(destructor: TLSDestructorFunc) -> Self {
        let mut out = Self {
            slot_: Self::kInvalidSlotValue,
            version_: 0,
        };
        out.Initialize(destructor);
        out
    }
    unsafe fn Initialize(&mut self, destructor: TLSDestructorFunc) {
        #[cfg(not(all(base_is_ios,not(pa_use_allocator_shim))))]
        let _mute=crate::base::sampling_heap_profiler::poisson_allocation_sampler::ScopedMuteThreadSamples::new();
        let key = g_native_tls_key.load(Ordering::Relaxed);
        if key == TLS_KEY_OUT_OF_INDEXES
            || GetTlsVectorStateAndValue(key, ptr::null_mut()) == TlsVectorState::kUninitialized
        {
            ConstructTlsVector();
        }
        {
            let _lock = AutoLock::new(GetTLSMetadataLock());
            for i in 0..256 {
                let candidate = g_last_assigned_slot.wrapping_add(1).wrapping_add(i) % 256;
                if g_tls_metadata[candidate].status == TlsStatus::FREE {
                    g_tls_metadata[candidate].status = TlsStatus::IN_USE;
                    g_tls_metadata[candidate].destructor = destructor;
                    g_sequence_num = g_sequence_num.wrapping_add(1);
                    g_tls_metadata[candidate].sequence_num = g_sequence_num;
                    g_last_assigned_slot = candidate;
                    crate::DCHECK_EQ!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),504);Self::kInvalidSlotValue,self.slot_);
                    self.slot_ = candidate;
                    self.version_ = g_tls_metadata[candidate].version;
                    break;
                }
            }
        }
        crate::CHECK_LT!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),511);self.slot_,256usize);
    }
    unsafe fn Free(&mut self) {
        crate::DCHECK_LT!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),515);self.slot_,256usize);
        {
            let _lock = AutoLock::new(GetTLSMetadataLock());
            g_tls_metadata[self.slot_].status = TlsStatus::FREE;
            g_tls_metadata[self.slot_].destructor = None;
            g_tls_metadata[self.slot_].version = g_tls_metadata[self.slot_].version.wrapping_add(1);
        }
        self.slot_ = Self::kInvalidSlotValue;
    }
    pub unsafe fn Get(&self) -> *mut c_void {
        let mut vector = ptr::null_mut();
        let state =
            GetTlsVectorStateAndValue(g_native_tls_key.load(Ordering::Relaxed), &mut vector);
        crate::DCHECK_NE!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),432);state,TlsVectorState::kDestroyed);
        if vector.is_null() {
            return ptr::null_mut();
        }
        crate::DCHECK_LT!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),515);self.slot_,256usize);
        if (*vector.add(self.slot_)).version != self.version_ {
            return ptr::null_mut();
        }
        (*vector.add(self.slot_)).data
    }
    pub unsafe fn Set(&self, value: *mut c_void) {
        let mut vector = ptr::null_mut();
        let state =
            GetTlsVectorStateAndValue(g_native_tls_key.load(Ordering::Relaxed), &mut vector);
        crate::DCHECK_NE!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),432);state,TlsVectorState::kDestroyed);
        if vector.is_null() {
            if value.is_null() {
                return;
            }
            vector = ConstructTlsVector();
        }
        crate::DCHECK_LT!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_storage.cc\0".as_ptr().cast(),515);self.slot_,256usize);
        (*vector.add(self.slot_)).data = value;
        (*vector.add(self.slot_)).version = self.version_;
    }
}
impl Drop for Slot {
    fn drop(&mut self) {
        unsafe {
            self.Free();
        }
    }
}
// Windows OS callbacks are a separate actual platform producer, not aliased to pthread.
