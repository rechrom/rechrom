use crate::base::location::Location;
use crate::base::time::time::{TimeDelta, TimeTicks};
use core::cell::UnsafeCell;
use std::collections::BTreeMap;
// Source: base/debug/dump_without_crashing.h:16-20
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpWithoutCrashingStatus {
    kThrottled,
    kUploaded,
}
impl DumpWithoutCrashingStatus {
    pub const kMaxValue: Self = Self::kUploaded;
}
// Source: base/debug/dump_without_crashing.cc:22-22
// Internal Rust callback mapping, no C/C++ FFI ABI identity claim.
pub type DumpFunction = unsafe fn();
static mut dump_without_crashing_function_: Option<DumpFunction> = None;
// Source: base/debug/dump_without_crashing.cc:43-47
// Static Rust objects have no process-exit destructor, matching NoDestructor.
// UnsafeCell represents the same one map, protected by the actual native Lock
// below; reset retains the native caller's exclusive testing precondition.
struct LocationMap(UnsafeCell<BTreeMap<Location, TimeTicks>>);
unsafe impl Sync for LocationMap {}
static LOCATION_TO_TIMESTAMP: LocationMap = LocationMap(UnsafeCell::new(BTreeMap::new()));
unsafe fn LocationToTimestampMap() -> *mut BTreeMap<Location, TimeTicks> {
    LOCATION_TO_TIMESTAMP.0.get()
}
// Source: base/debug/dump_without_crashing.cc:24-39
// This is the source's actual Location/TimeTicks map template instantiation.
// No std::Mutex/Instant substitute is supplied for the missing base producers.
unsafe fn ShouldDump(
    map: *mut BTreeMap<Location, TimeTicks>,
    key: &Location,
    time_between_dumps: TimeDelta,
) -> bool {
    struct LockSlot(UnsafeCell<core::mem::MaybeUninit<crate::base::synchronization::lock::Lock>>);
    unsafe impl Sync for LockSlot {}
    static LOCK: LockSlot = LockSlot(UnsafeCell::new(core::mem::MaybeUninit::uninit()));
    static INITIALIZED: std::sync::Once = std::sync::Once::new();
    INITIALIZED.call_once_force(|_| unsafe {
        crate::base::synchronization::lock::Lock::new(core::pin::Pin::new_unchecked(
            &mut *LOCK.0.get(),
        ));
    });
    let lock = unsafe {
        core::pin::Pin::new_unchecked(
            &*LOCK
                .0
                .get()
                .cast::<crate::base::synchronization::lock::Lock>(),
        )
    };
    let _auto_lock = unsafe { crate::base::synchronization::lock::AutoLock::new(lock) };
    let now = TimeTicks::Now();
    unsafe {
        match (*map).entry(*key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(now);
                true
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                if now - *entry.get() >= time_between_dumps {
                    entry.insert(now);
                    true
                } else {
                    false
                }
            }
        }
    }
}
// Source: base/debug/dump_without_crashing.cc:52-55
unsafe fn ShouldDumpWithoutCrashWithLocation(
    location: &Location,
    time_between_dumps: TimeDelta,
) -> bool {
    unsafe { ShouldDump(LocationToTimestampMap(), location, time_between_dumps) }
}
// Source: base/debug/dump_without_crashing.cc:61-82
/// Safety: native registered callback/provider and Location strings remain live;
/// registrations cannot race reads; map access follows the real Lock protocol.
/// Native NOT_TAIL_CALLED language guarantee is not claimed by this Rust body.
#[inline(never)]
pub unsafe fn DumpWithoutCrashing(location: &Location, time_between_dumps: TimeDelta) -> bool {
    crate::TRACE_EVENT0!("base", "DumpWithoutCrashing");
    if unsafe { dump_without_crashing_function_.is_some() }
        && unsafe { ShouldDumpWithoutCrashWithLocation(location, time_between_dumps) }
    {
        let file = unsafe { core::ffi::CStr::from_ptr(location.file_name()).to_bytes() };
        crate::SCOPED_CRASH_KEY_STRING256!(file_key, "DumpWithoutCrashing", "file", file);
        crate::SCOPED_CRASH_KEY_NUMBER!(
            line_key,
            "DumpWithoutCrashing",
            "line",
            location.line_number()
        );
        // Re-read at the original invocation point, after metadata callbacks.
        unsafe {
            dump_without_crashing_function_.unwrap_unchecked()();
        }
        crate::base::metrics::histogram_functions::UmaHistogramEnumeration(
            "Stability.DumpWithoutCrashingStatus",
            DumpWithoutCrashingStatus::kUploaded,
        );
        return true;
    }
    crate::base::metrics::histogram_functions::UmaHistogramEnumeration(
        "Stability.DumpWithoutCrashingStatus",
        DumpWithoutCrashingStatus::kThrottled,
    );
    false
}
// Source: base/debug/dump_without_crashing.cc:84-92
/// Safety: caller excludes concurrent callback reads/registration exactly as
/// required by the native non-atomic global. None really removes the callback.
pub unsafe fn SetDumpWithoutCrashingFunction(function: Option<DumpFunction>) {
    #[cfg(not(COMPONENT_BUILD))]
    crate::DCHECK!(@location unsafe{Location::CurrentWithoutFunctionName(b"base/debug/dump_without_crashing.cc\0".as_ptr().cast(),89)}; @condition b"!dump_without_crashing_function_ || !function\0".as_ptr(); unsafe{dump_without_crashing_function_.is_none()}||function.is_none());
    unsafe {
        dump_without_crashing_function_ = function;
    }
}
// Source: base/debug/dump_without_crashing.cc:94-96
/// Safety: no dump/map operation can overlap this native unlocked testing reset.
pub unsafe fn ResetDumpWithoutCrashingThrottlingForTesting() {
    unsafe {
        (*LocationToTimestampMap()).clear();
    }
}
