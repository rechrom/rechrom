use core::ffi::{c_char, c_int, c_void};
// Actual generated native-build selection, not a fallback backend provider.
use crate::base::allocator::partition_allocator::src::partition_alloc::pointers::raw_ptr::LocationPointerConfig;
use crate::base::allocator::partition_allocator::src::partition_alloc::pointers::raw_ptr::{
    raw_ptr_const, UnprotectedInRelease,
};
type LocationPc = raw_ptr_const<c_void, LocationPointerConfig, { UnprotectedInRelease }>;
// Source: base/location.h:20-25
// Source: base/location.h:106-112
// Non-owning source/PC fields retain the original long-lived native string
// contract. raw_ptr instrumentation is not claimed ABI-identical to this state.
pub struct Location {
    function_name_: *const c_char,
    file_name_: *const c_char,
    line_number_: i32,
    program_counter_: LocationPc,
}
impl Location {
    // Source: base/location.cc:59-62
    pub fn new() -> Self {
        Self {
            function_name_: core::ptr::null(),
            file_name_: core::ptr::null(),
            line_number_: -1,
            program_counter_: LocationPc::null(),
        }
    }
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        self.function_name_ = other.function_name_;
        self.file_name_ = other.file_name_;
        self.line_number_ = other.line_number_;
        unsafe { self.program_counter_.copy_assign(&other.program_counter_) };
        self
    }
    pub fn move_from(source: &mut Self) -> Self {
        Self {
            function_name_: source.function_name_,
            file_name_: source.file_name_,
            line_number_: source.line_number_,
            program_counter_: unsafe { LocationPc::move_from(&mut source.program_counter_) },
        }
    }
    // Source: base/location.cc:64-65
    pub unsafe fn from_file_pc(file: *const c_char, pc: *const c_void) -> Self {
        Self {
            function_name_: core::ptr::null(),
            file_name_: file,
            line_number_: -1,
            program_counter_: unsafe { LocationPc::from_ptr(pc) },
        }
    }
    // Source: base/location.cc:67-79
    /// Safety: nonnull source strings remain valid for every use of the result.
    pub unsafe fn from_source_pc(
        function: *const c_char,
        file: *const c_char,
        line: i32,
        pc: *const c_void,
    ) -> Self {
        let result = Self {
            function_name_: function,
            file_name_: file,
            line_number_: line,
            program_counter_: unsafe { LocationPc::from_ptr(pc) },
        };
        crate::DCHECK!(@location unsafe{Location::CurrentWithoutFunctionName(concat!(env!("BASE_LOCATION_NATIVE_FILE"),"\0").as_ptr().cast(),78)}; @condition b"program_counter\0".as_ptr(); !pc.is_null());
        result
    }
    // Source: base/location.h:27-32
    pub unsafe fn CreateForTesting(
        function: *const c_char,
        file: *const c_char,
        line: i32,
        pc: *const c_void,
    ) -> Self {
        unsafe { Self::from_source_pc(function, file, line, pc) }
    }
    // Source: base/location.h:58-77
    pub fn has_source_info(&self) -> bool {
        !self.function_name_.is_null() && !self.file_name_.is_null()
    }
    pub fn function_name(&self) -> *const c_char {
        self.function_name_
    }
    pub fn file_name(&self) -> *const c_char {
        self.file_name_
    }
    pub fn line_number(&self) -> i32 {
        self.line_number_
    }
    pub fn program_counter(&self) -> *const c_void {
        unsafe { self.program_counter_.get() }
    }
    // Source: base/location.cc:81-87
    /// Safety: all nonnull original source strings retain their native lifetime.
    pub unsafe fn ToString(&self) -> Vec<u8> {
        unsafe extern "C" {
            fn snprintf(dst: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
        }
        let mut buffer = [core::mem::MaybeUninit::<c_char>::uninit(); 50];
        unsafe {
            if self.has_source_info() {
                let mut output = core::ffi::CStr::from_ptr(self.function_name_)
                    .to_bytes()
                    .to_vec();
                output.push(b'@');
                output.extend_from_slice(core::ffi::CStr::from_ptr(self.file_name_).to_bytes());
                output.push(b':');
                snprintf(
                    buffer.as_mut_ptr().cast(),
                    50,
                    b"%d\0".as_ptr().cast(),
                    self.line_number_ as c_int,
                );
                output.extend_from_slice(
                    core::ffi::CStr::from_ptr(buffer.as_ptr().cast()).to_bytes(),
                );
                output
            } else {
                snprintf(
                    buffer.as_mut_ptr().cast(),
                    50,
                    b"pc:%p\0".as_ptr().cast(),
                    self.program_counter(),
                );
                core::ffi::CStr::from_ptr(buffer.as_ptr().cast())
                    .to_bytes()
                    .to_vec()
            }
        }
    }
}
// Source: base/location.h:36-46
impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.program_counter_.comparison() == other.program_counter_.comparison() }
    }
}
impl Eq for Location {}
impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Location {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        unsafe {
            self.program_counter_
                .comparison()
                .addr()
                .cmp(&other.program_counter_.comparison().addr())
        }
    }
}
// Source: base/location.h:50-52
impl core::hash::Hash for Location {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(&self.program_counter(), state);
    }
}
// Source: base/location.cc:22-55
// These are the original C++ compilation inputs, supplied by the build bridge;
// Rust file!() or a guessed zero prefix would change the stored source names.
const NATIVE_FILE: &str = env!("BASE_LOCATION_NATIVE_FILE");
const THIS_FILE: &str = env!("BASE_LOCATION_NATIVE_THIS_FILE_PATH");
pub const kStrippedPrefixLength: usize = {
    let file = NATIVE_FILE.as_bytes();
    let suffix = THIS_FILE.as_bytes();
    assert!(
        file.len() >= suffix.len(),
        "Invalid file path for base/location.cc."
    );
    let prefix = file.len() - suffix.len();
    let mut i = 0;
    while i < suffix.len() {
        assert!(
            file[prefix + i] == suffix[i],
            "The file name does not match the expected prefix format."
        );
        i += 1;
    }
    prefix
};
// Native Current/CurrentWithoutFunctionName/GetProgramCounter require the
// actual compiler RETURN_ADDRESS operation, including extraction/PAC semantics.
// No fabricated PC/hash, call-site allocation or null fallback is published.

// Source: base/location.cc:59-59
impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}
// Source: base/location.h:117-117
// Native source defaults are supplied explicitly; no Rust file!()/line!()
// identity is substituted for the source location under translation.
#[macro_export]
macro_rules! FROM_HERE {
    (@function $function:expr; @file $file:expr; @line $line:expr) => {
        unsafe { $crate::base::location::Location::Current($function, $file, $line) }
    };
}

// Source: base/location.cc:96-103
// Capture in the actual noinline Rust Location boundary, not in an FFI wrapper
// frame. GCC extraction is a separate primitive taking the already-captured
// address, so it cannot substitute its own helper's caller PC.
#[cfg(native_compiler_gcc)]
unsafe extern "C" {
    fn chromium_location_extract_return_address(address: *const c_void) -> *const c_void;
}
impl Location {
    // Source: base/location.h:84-86
    // Source: base/location.cc:106-111
    /// Safety: native builtin source strings have their original global
    /// lifetime and file carries the exact native build prefix above.
    #[inline(never)]
    pub unsafe fn Current(function: *const c_char, file: *const c_char, line: c_int) -> Self {
        #[cfg(native_compiler_gcc)]
        let pc = unsafe {
            chromium_location_extract_return_address(core::intrinsics::return_address().cast())
        };
        #[cfg(native_compiler_msvc)]
        let pc = core::intrinsics::return_address().cast();
        #[cfg(native_compiler_other)]
        let pc = core::ptr::null(); // Exact original RETURN_ADDRESS fallback only.
        unsafe { Self::from_source_pc(function, file.add(kStrippedPrefixLength), line, pc) }
    }
    // Source: base/location.h:88-90
    // Source: base/location.cc:114-118
    /// Safety: file is the original global native builtin file string, with
    /// exact native prefix; no temporary or Rust host source metadata.
    #[inline(never)]
    pub unsafe fn CurrentWithoutFunctionName(file: *const c_char, line: c_int) -> Self {
        #[cfg(native_compiler_gcc)]
        let pc = unsafe {
            chromium_location_extract_return_address(core::intrinsics::return_address().cast())
        };
        #[cfg(native_compiler_msvc)]
        let pc = core::intrinsics::return_address().cast();
        #[cfg(native_compiler_other)]
        let pc = core::ptr::null(); // Explicit native compiler fallback only.
        unsafe {
            Self::from_source_pc(core::ptr::null(), file.add(kStrippedPrefixLength), line, pc)
        }
    }
}
// Source: base/location.h:115
// Source: base/location.cc:121-123
#[inline(never)]
pub fn GetProgramCounter() -> *const c_void {
    #[cfg(native_compiler_gcc)]
    {
        unsafe {
            chromium_location_extract_return_address(core::intrinsics::return_address().cast())
        }
    }
    #[cfg(native_compiler_msvc)]
    {
        core::intrinsics::return_address().cast()
    }
    #[cfg(native_compiler_other)]
    {
        core::ptr::null()
    }
}
#[cfg(not(any(native_compiler_gcc, native_compiler_msvc, native_compiler_other)))]
compile_error!("Location requires the actual native COMPILER_GCC/MSVC/other selection");
// Source: base/location.h:50-52
// Genuine template operation protocol for an actual external hash-state type;
// this is not a new Location owner, hash algorithm or invented producer.
pub trait LocationHashState: Sized {
    fn combine(self, program_counter: *const c_void) -> Self;
}
pub fn AbslHashValue<H: LocationHashState>(state: H, location: &Location) -> H {
    state.combine(location.program_counter())
}
// Source: base/location.cc:89-94
// WriteIntoTrace requires actual perfetto::TracedValue/WriteDictionary/Add
// producer bindings. No no-op tracing method or fake dictionary is supplied.

// Source: base/location.cc:60
impl Clone for Location {
    fn clone(&self) -> Self {
        Self {
            function_name_: self.function_name_,
            file_name_: self.file_name_,
            line_number_: self.line_number_,
            program_counter_: unsafe { LocationPc::copy_from(&self.program_counter_) },
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}
// Native selected raw_ptr field owns its instrumentation lifetime and releases
// before the plain source fields. No Location Copy erases copy/move hooks.

// Source: base/location.cc:62
impl Location {
    /// Safety: live exclusive objects; equal pointers are the genuine native
    /// copy-self-assignment. No aliased mutable Rust references are formed.
    pub unsafe fn copy_assign_raw(dst: core::ptr::NonNull<Self>, source: core::ptr::NonNull<Self>) {
        unsafe {
            let function = (*source.as_ptr()).function_name_;
            let file = (*source.as_ptr()).file_name_;
            let line = (*source.as_ptr()).line_number_;
            (*dst.as_ptr()).function_name_ = function;
            (*dst.as_ptr()).file_name_ = file;
            (*dst.as_ptr()).line_number_ = line;
            LocationPc::copy_assign_raw(
                core::ptr::NonNull::new_unchecked(core::ptr::addr_of_mut!(
                    (*dst.as_ptr()).program_counter_
                )),
                core::ptr::NonNull::new_unchecked(core::ptr::addr_of_mut!(
                    (*source.as_ptr()).program_counter_
                )),
            );
        }
    }
}
