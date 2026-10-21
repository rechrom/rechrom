//! Genuine C++ standard-library object binding; no byte-vector stream model.
//! No storage implementation or SDK ABI numbers are guessed by this module.
use core::{
    ffi::{
        c_char, c_int, c_long, c_longlong, c_short, c_uint, c_ulong, c_ulonglong, c_ushort, c_void,
    },
    marker::{PhantomData, PhantomPinned},
    mem::{align_of, size_of, MaybeUninit},
    pin::Pin,
    ptr::NonNull,
};

/// Implement only for raw storage whose size/alignment come from the exact
/// configured native SDK ABI input. No owning fields/destructors are permitted.
/// Never use a larger approximate buffer, another target's metadata or defaults.
pub unsafe trait OStringStreamStorage {
    const NATIVE_SIZE: usize;
    const NATIVE_ALIGN: usize;
}
/// Same contract for the real std::string returned by ostringstream::str().
pub unsafe trait StdStringStorage {
    const NATIVE_SIZE: usize;
    const NATIVE_ALIGN: usize;
}

#[derive(Debug)]
pub struct AbiMismatch {
    pub native_size: usize,
    pub native_align: usize,
    pub input_size: usize,
    pub input_align: usize,
    pub rust_size: usize,
    pub rust_align: usize,
}
fn check<T>(
    size: usize,
    align: usize,
    input_size: usize,
    input_align: usize,
) -> Result<(), AbiMismatch> {
    if size == input_size
        && align == input_align
        && size_of::<T>() == size
        && align_of::<T>() == align
    {
        Ok(())
    } else {
        Err(AbiMismatch {
            native_size: size,
            native_align: align,
            input_size,
            input_align,
            rust_size: size_of::<T>(),
            rust_align: align_of::<T>(),
        })
    }
}

// Source: base/logging.h:553-554
// Actual std::ostringstream exists inline at storage's address. Pin prevents a
// Rust bytewise move of native internal self-pointers. Phantom fields have no
// counter, heap owner, initialization flag or external layout claim.
#[repr(C)]
pub struct NativeOStringStream<S: OStringStreamStorage> {
    storage: MaybeUninit<S>,
    _pin: PhantomPinned,
    _thread_domain: PhantomData<*mut ()>,
}
impl<S: OStringStreamStorage> NativeOStringStream<S> {
    /// Construct at the final aligned address of the containing native owner.
    ///
    /// # Safety
    /// dst is valid uninitialized storage, exclusively borrowed, and remains
    /// stationary until its eventual Drop. Caller marks it initialized only on
    /// Ok and destroys it exactly once. This never pins a copied temporary.
    pub unsafe fn initialize_at(dst: *mut Self) -> Result<(), AbiMismatch> {
        check::<S>(
            unsafe { rechrom_ostringstream_size() },
            unsafe { rechrom_ostringstream_align() },
            S::NATIVE_SIZE,
            S::NATIVE_ALIGN,
        )?;
        unsafe {
            dst.write(Self {
                storage: MaybeUninit::uninit(),
                _pin: PhantomPinned,
                _thread_domain: PhantomData,
            });
            rechrom_ostringstream_construct(core::ptr::addr_of_mut!((*dst).storage).cast());
        }
        Ok(())
    }
    // Source: base/logging.h:534
    pub fn stream<'a>(self: Pin<&'a mut Self>) -> NativeOStream<'a> {
        let ptr = unsafe { self.get_unchecked_mut().storage.as_mut_ptr().cast() };
        let stream = unsafe { rechrom_ostringstream_ostream(ptr) };
        NativeOStream {
            ptr: unsafe { NonNull::new_unchecked(stream) },
            _borrow: PhantomData,
        }
    }
    // Source: base/logging.cc:732; base/logging.cc:1000
    pub fn view(self: Pin<&Self>) -> &[u8] {
        let mut data = core::ptr::null();
        let mut len = 0;
        unsafe {
            rechrom_ostringstream_view(self.get_ref().storage.as_ptr().cast(), &mut data, &mut len)
        };
        // Empty native string_view is allowed a null data pointer. Rust's empty
        // slice represents the same length without manufacturing a null slice.
        if len == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(data, len) }
        }
    }
    // Source: base/logging.h:537; base/logging.cc:764
    /// Construct the original native str() result directly in final dst storage.
    /// dst has the same uninitialized/pinned/exclusive/exactly-once contract as
    /// initialize_at, and cannot overlap this live stream or its buffer.
    pub unsafe fn str_at<T: StdStringStorage>(
        self: Pin<&Self>,
        dst: *mut NativeStdString<T>,
    ) -> Result<(), AbiMismatch> {
        NativeStdString::<T>::check_abi()?;
        unsafe {
            dst.write(NativeStdString::uninitialized_storage());
            rechrom_ostringstream_str_at(
                self.get_ref().storage.as_ptr().cast(),
                core::ptr::addr_of_mut!((*dst).storage).cast(),
            );
        }
        Ok(())
    }
}
impl<S: OStringStreamStorage> Drop for NativeOStringStream<S> {
    fn drop(&mut self) {
        unsafe { rechrom_ostringstream_destroy(self.storage.as_mut_ptr().cast()) }
    }
}

// Source: base/logging.h:537; base/logging.cc:764-765
#[repr(C)]
pub struct NativeStdString<S: StdStringStorage> {
    storage: MaybeUninit<S>,
    _pin: PhantomPinned,
    _thread_domain: PhantomData<*mut ()>,
}
impl<S: StdStringStorage> NativeStdString<S> {
    fn uninitialized_storage() -> Self {
        Self {
            storage: MaybeUninit::uninit(),
            _pin: PhantomPinned,
            _thread_domain: PhantomData,
        }
    }
    fn check_abi() -> Result<(), AbiMismatch> {
        check::<S>(
            unsafe { rechrom_std_string_size() },
            unsafe { rechrom_std_string_align() },
            S::NATIVE_SIZE,
            S::NATIVE_ALIGN,
        )
    }
    pub fn view(self: Pin<&Self>) -> &[u8] {
        let mut data = core::ptr::null();
        let mut len = 0;
        unsafe {
            rechrom_std_string_view(self.get_ref().storage.as_ptr().cast(), &mut data, &mut len)
        };
        if len == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(data, len) }
        }
    }
    /// dst is exclusively borrowed uninitialized final pinned storage and must
    /// not overlap self. Native substr's range precondition/exception behavior
    /// is retained; no clamping or invented successful result is performed.
    pub unsafe fn substr_at<T: StdStringStorage>(
        self: Pin<&Self>,
        start: usize,
        dst: *mut NativeStdString<T>,
    ) -> Result<(), AbiMismatch> {
        NativeStdString::<T>::check_abi()?;
        unsafe {
            dst.write(NativeStdString::uninitialized_storage());
            rechrom_std_string_substr_at(
                self.get_ref().storage.as_ptr().cast(),
                start,
                core::ptr::addr_of_mut!((*dst).storage).cast(),
            );
        }
        Ok(())
    }
}
impl<S: StdStringStorage> Drop for NativeStdString<S> {
    fn drop(&mut self) {
        unsafe { rechrom_std_string_destroy(self.storage.as_mut_ptr().cast()) }
    }
}

// Source: base/logging.h:534; base/check.h:133; base/check.cc:335-337
/// Borrow of the actual native std::ostream base subobject, not another stream.
/// Arbitrary native operator<< domains require their own actual typed bridge;
/// Rust Display/format!/byte formatting is deliberately not a substitute.
pub struct NativeOStream<'a> {
    ptr: NonNull<c_void>,
    _borrow: PhantomData<&'a mut c_void>,
}
impl<'a> NativeOStream<'a> {
    /// ptr is the actual adjusted native ostream pointer, exclusively borrowed
    /// for 'a. All prior native aliases must obey that borrow; no owner is taken.
    pub unsafe fn from_native(ptr: NonNull<c_void>) -> Self {
        Self {
            ptr,
            _borrow: PhantomData,
        }
    }
    /// Pass only synchronously to genuine native ostream-consuming functions;
    /// do not retain, destroy, move, alias mutably or reconstruct an owner.
    pub unsafe fn as_native_mut_ptr(&mut self) -> *mut c_void {
        self.ptr.as_ptr()
    }
    pub fn insert_char(&mut self, x: u8) {
        unsafe { rechrom_ostream_char(self.ptr.as_ptr(), x as c_char) }
    }
    pub unsafe fn insert_cstr(&mut self, x: *const u8) {
        unsafe { rechrom_ostream_cstr(self.ptr.as_ptr(), x.cast()) }
    }
    pub fn insert_bytes(&mut self, x: &[u8]) {
        unsafe { rechrom_ostream_bytes(self.ptr.as_ptr(), x.as_ptr(), x.len()) }
    }
    pub fn insert_string<S: StdStringStorage>(&mut self, x: Pin<&NativeStdString<S>>) {
        unsafe { rechrom_ostream_string(self.ptr.as_ptr(), x.get_ref().storage.as_ptr().cast()) }
    }
    pub fn insert_bool(&mut self, x: bool) {
        unsafe { rechrom_ostream_bool(self.ptr.as_ptr(), x) }
    }
    pub fn insert_short(&mut self, x: c_short) {
        unsafe { rechrom_ostream_short(self.ptr.as_ptr(), x) }
    }
    pub fn insert_ushort(&mut self, x: c_ushort) {
        unsafe { rechrom_ostream_ushort(self.ptr.as_ptr(), x) }
    }
    pub fn insert_int(&mut self, x: c_int) {
        unsafe { rechrom_ostream_int(self.ptr.as_ptr(), x) }
    }
    pub fn insert_uint(&mut self, x: c_uint) {
        unsafe { rechrom_ostream_uint(self.ptr.as_ptr(), x) }
    }
    pub fn insert_long(&mut self, x: c_long) {
        unsafe { rechrom_ostream_long(self.ptr.as_ptr(), x) }
    }
    pub fn insert_ulong(&mut self, x: c_ulong) {
        unsafe { rechrom_ostream_ulong(self.ptr.as_ptr(), x) }
    }
    pub fn insert_llong(&mut self, x: c_longlong) {
        unsafe { rechrom_ostream_llong(self.ptr.as_ptr(), x) }
    }
    pub fn insert_ullong(&mut self, x: c_ulonglong) {
        unsafe { rechrom_ostream_ullong(self.ptr.as_ptr(), x) }
    }
    pub fn insert_float(&mut self, x: f32) {
        unsafe { rechrom_ostream_float(self.ptr.as_ptr(), x) }
    }
    pub fn insert_double(&mut self, x: f64) {
        unsafe { rechrom_ostream_double(self.ptr.as_ptr(), x) }
    }
    pub fn insert_pointer(&mut self, x: *const c_void) {
        unsafe { rechrom_ostream_pointer(self.ptr.as_ptr(), x) }
    }
    pub fn insert_size(&mut self, x: usize) {
        unsafe { rechrom_ostream_size(self.ptr.as_ptr(), x) }
    }
    // Source: base/logging.cc:969-987
    pub fn setfill(&mut self, x: u8) {
        unsafe { rechrom_ostream_setfill(self.ptr.as_ptr(), x as c_char) }
    }
    pub fn setw(&mut self, x: c_int) {
        unsafe { rechrom_ostream_setw(self.ptr.as_ptr(), x) }
    }
    // Source: base/logging.cc:737; base/logging.cc:763
    pub fn endl(&mut self) {
        unsafe { rechrom_ostream_endl(self.ptr.as_ptr()) }
    }
}

unsafe extern "C" {
    fn rechrom_ostringstream_size() -> usize;
    fn rechrom_ostringstream_align() -> usize;
    fn rechrom_ostringstream_construct(p: *mut c_void);
    fn rechrom_ostringstream_destroy(p: *mut c_void);
    fn rechrom_ostringstream_ostream(p: *mut c_void) -> *mut c_void;
    fn rechrom_ostringstream_view(p: *const c_void, data: *mut *const u8, len: *mut usize);
    fn rechrom_std_string_size() -> usize;
    fn rechrom_std_string_align() -> usize;
    fn rechrom_ostringstream_str_at(p: *const c_void, dst: *mut c_void);
    fn rechrom_std_string_destroy(p: *mut c_void);
    fn rechrom_std_string_view(p: *const c_void, data: *mut *const u8, len: *mut usize);
    fn rechrom_std_string_substr_at(p: *const c_void, start: usize, dst: *mut c_void);
    fn rechrom_ostream_char(p: *mut c_void, x: c_char);
    fn rechrom_ostream_cstr(p: *mut c_void, x: *const c_char);
    fn rechrom_ostream_bytes(p: *mut c_void, x: *const u8, n: usize);
    fn rechrom_ostream_string(p: *mut c_void, x: *const c_void);
    fn rechrom_ostream_bool(p: *mut c_void, x: bool);
    fn rechrom_ostream_short(p: *mut c_void, x: c_short);
    fn rechrom_ostream_ushort(p: *mut c_void, x: c_ushort);
    fn rechrom_ostream_int(p: *mut c_void, x: c_int);
    fn rechrom_ostream_uint(p: *mut c_void, x: c_uint);
    fn rechrom_ostream_long(p: *mut c_void, x: c_long);
    fn rechrom_ostream_ulong(p: *mut c_void, x: c_ulong);
    fn rechrom_ostream_llong(p: *mut c_void, x: c_longlong);
    fn rechrom_ostream_ullong(p: *mut c_void, x: c_ulonglong);
    fn rechrom_ostream_float(p: *mut c_void, x: f32);
    fn rechrom_ostream_double(p: *mut c_void, x: f64);
    fn rechrom_ostream_pointer(p: *mut c_void, x: *const c_void);
    fn rechrom_ostream_size(p: *mut c_void, x: usize);
    fn rechrom_ostream_setfill(p: *mut c_void, x: c_char);
    fn rechrom_ostream_setw(p: *mut c_void, x: c_int);
    fn rechrom_ostream_endl(p: *mut c_void);
}

// SDK binding integration only. Root's build fragment generates this file from
// supplied genuine native metadata; it never runs a layout probe or uses HOST.
// No concrete stream/string alias exists when that native domain is unconfigured.
#[cfg(sk_native_ostream_abi_configured)]
pub mod supplied_abi {
    include!(concat!(
        env!("OUT_DIR"),
        "/rechrom_native_ostream_abi_storage.rs"
    ));
}
#[cfg(sk_native_ostream_abi_configured)]
pub type ConfiguredOStringStream = NativeOStringStream<supplied_abi::SuppliedOStringStreamStorage>;
#[cfg(sk_native_ostream_abi_configured)]
pub type ConfiguredStdString = NativeStdString<supplied_abi::SuppliedStdStringStorage>;

// Append inside the genuine native_ostream module (not a separate owner/module).
// External standard-library primitive bindings, no Rust/Skia algorithm credit.
// No storage implementation or ABI sizes are supplied by this extension.
impl<S: StdStringStorage> NativeStdString<S> {
    /// Safety: all inherited StdStringStorage actual-ABI requirements hold,
    /// including exact Rust storage size/alignment, and dst is final stationary
    /// exclusive uninitialized storage. source is the original live c_str().
    /// This is the primitive unchecked construction path used only under that
    /// explicit unsafe native-ABI contract; it is not an approximate-size fallback.
    pub unsafe fn initialize_cstr_at_assuming_native_abi(dst: *mut Self, source: *const c_char) {
        unsafe {
            dst.write(Self::uninitialized_storage());
            rechrom_debug_string_cstr_construct(
                core::ptr::addr_of_mut!((*dst).storage).cast(),
                source,
            );
        }
    }
    /// Same exact native-ABI/final-storage contract; native std::string is move
    /// constructed at dst from the actual live native string object source.
    pub unsafe fn move_construct_at_assuming_native_abi(dst: *mut Self, source: *mut c_void) {
        unsafe {
            dst.write(Self::uninitialized_storage());
            rechrom_debug_string_move_construct(
                core::ptr::addr_of_mut!((*dst).storage).cast(),
                source,
            );
        }
    }
    pub fn as_native_ptr(self: Pin<&Self>) -> *const c_void {
        self.get_ref().storage.as_ptr().cast()
    }
    pub fn as_native_mut_ptr(self: Pin<&mut Self>) -> *mut c_void {
        unsafe { self.get_unchecked_mut().storage.as_mut_ptr().cast() }
    }
    pub fn c_str(self: Pin<&Self>) -> *const c_char {
        unsafe { rechrom_debug_string_cstr(self.as_native_ptr()) }
    }
    pub fn size(self: Pin<&Self>) -> usize {
        self.view().len()
    }
    pub fn find_bytes(self: Pin<&Self>, needle: &[u8], start: usize) -> Option<usize> {
        let mut result = 0;
        let present = unsafe {
            rechrom_debug_string_find(
                self.as_native_ptr(),
                needle.as_ptr(),
                needle.len(),
                start,
                &mut result,
            )
        };
        present.then_some(result)
    }
    pub fn find_first_not_of(self: Pin<&Self>, set: &[u8], start: usize) -> Option<usize> {
        let mut result = 0;
        let present = unsafe {
            rechrom_debug_string_find_first_not_of(
                self.as_native_ptr(),
                set.as_ptr(),
                set.len(),
                start,
                &mut result,
            )
        };
        present.then_some(result)
    }
    pub unsafe fn erase(self: Pin<&mut Self>, start: usize, count: usize) {
        unsafe {
            rechrom_debug_string_erase(self.as_native_mut_ptr(), start, count);
        }
    }
    pub unsafe fn insert_cstr(self: Pin<&mut Self>, start: usize, value: *const c_char) {
        unsafe {
            rechrom_debug_string_insert(self.as_native_mut_ptr(), start, value);
        }
    }
    pub unsafe fn substr_count_at_assuming_native_abi<T: StdStringStorage>(
        self: Pin<&Self>,
        start: usize,
        count: usize,
        dst: *mut NativeStdString<T>,
    ) {
        unsafe {
            dst.write(NativeStdString::uninitialized_storage());
            rechrom_debug_string_substr_construct(
                self.as_native_ptr(),
                start,
                count,
                core::ptr::addr_of_mut!((*dst).storage).cast(),
            );
        }
    }
}
unsafe extern "C" {
    fn rechrom_debug_string_cstr_construct(dst: *mut c_void, s: *const c_char);
    fn rechrom_debug_string_move_construct(dst: *mut c_void, s: *mut c_void);
    fn rechrom_debug_string_cstr(s: *const c_void) -> *const c_char;
    fn rechrom_debug_string_find(
        s: *const c_void,
        n: *const u8,
        length: usize,
        start: usize,
        out: *mut usize,
    ) -> bool;
    fn rechrom_debug_string_find_first_not_of(
        s: *const c_void,
        n: *const u8,
        length: usize,
        start: usize,
        out: *mut usize,
    ) -> bool;
    fn rechrom_debug_string_erase(s: *mut c_void, start: usize, count: usize);
    fn rechrom_debug_string_insert(s: *mut c_void, start: usize, value: *const c_char);
    fn rechrom_debug_string_substr_construct(
        s: *const c_void,
        start: usize,
        count: usize,
        dst: *mut c_void,
    );
}

// Genuine std::stringstream capability; it is not ostringstream storage.
// Exact target native SDK layout is mandatory. S is raw non-owning storage
// with no destructors/owned values; size_of<S>/align_of<S> must exactly match the
// actual native std::stringstream. No implementation/ABI number is supplied.
pub unsafe trait StringStreamStorage {
    const NATIVE_SIZE: usize;
    const NATIVE_ALIGN: usize;
}
#[repr(C)]
pub struct NativeStringStream<S: StringStreamStorage> {
    storage: MaybeUninit<S>,
    _pin: PhantomPinned,
    _thread_domain: PhantomData<*mut ()>,
}
impl<S: StringStreamStorage> NativeStringStream<S> {
    /// Safety: exact target native std::stringstream size/alignment for S and the
    /// same final-address/exclusive/uninitialized construction contract as the
    /// existing stream owner. No approximate storage or bytewise move is valid.
    pub unsafe fn initialize_at_assuming_native_abi(dst: *mut Self) {
        unsafe {
            dst.write(Self {
                storage: MaybeUninit::uninit(),
                _pin: PhantomPinned,
                _thread_domain: PhantomData,
            });
            rechrom_debug_stringstream_construct(core::ptr::addr_of_mut!((*dst).storage).cast());
        }
    }
    pub fn stream<'a>(self: Pin<&'a mut Self>) -> NativeOStream<'a> {
        let p = unsafe {
            rechrom_debug_stringstream_ostream(self.get_unchecked_mut().storage.as_mut_ptr().cast())
        };
        unsafe { NativeOStream::from_native(NonNull::new_unchecked(p)) }
    }
    /// Native std::move(stream).str(), directly constructed at the actual original
    /// return-value slot. dst has genuine native std::string storage, stationary,
    /// uninitialized and disjoint from the stream through its later Drop.
    pub unsafe fn take_str_at<T: StdStringStorage>(
        self: Pin<&mut Self>,
        dst: *mut NativeStdString<T>,
    ) {
        unsafe {
            dst.write(NativeStdString::uninitialized_storage());
            rechrom_debug_stringstream_take_str(
                self.get_unchecked_mut().storage.as_mut_ptr().cast(),
                core::ptr::addr_of_mut!((*dst).storage).cast(),
            );
        }
    }
}
impl<S: StringStreamStorage> Drop for NativeStringStream<S> {
    fn drop(&mut self) {
        unsafe {
            rechrom_debug_stringstream_destroy(self.storage.as_mut_ptr().cast());
        }
    }
}
unsafe extern "C" {
    fn rechrom_debug_stringstream_construct(dst: *mut c_void);
    fn rechrom_debug_stringstream_ostream(p: *mut c_void) -> *mut c_void;
    fn rechrom_debug_stringstream_take_str(p: *mut c_void, dst: *mut c_void);
    fn rechrom_debug_stringstream_destroy(p: *mut c_void);
}

// Real standard-library constructor/append primitives required by StringAppendV.
impl<S: StdStringStorage> NativeStdString<S> {
    /// Safety: exact supplied SDK ABI, exclusive uninitialized final address, one
    /// later native destructor. No native std::string bytewise move is performed.
    pub unsafe fn initialize_empty_at_assuming_native_abi(dst: *mut Self) {
        unsafe {
            dst.write(Self::uninitialized_storage());
            rechrom_std_string_default_construct(core::ptr::addr_of_mut!((*dst).storage).cast());
        }
    }
    pub fn append(self: Pin<&mut Self>, value: &[u8]) {
        unsafe {
            rechrom_std_string_append(self.as_native_mut_ptr(), value.as_ptr(), value.len());
        }
    }
    pub fn clear(self: Pin<&mut Self>) {
        unsafe {
            rechrom_std_string_clear(self.as_native_mut_ptr());
        }
    }
}
impl<S: OStringStreamStorage> NativeOStringStream<S> {
    /// Same actual ABI/final-address contract as initialize_at, with prior real
    /// configuration assertion supplied by root. No repeated runtime ABI check.
    pub unsafe fn initialize_at_assuming_native_abi(dst: *mut Self) {
        unsafe {
            dst.write(Self {
                storage: MaybeUninit::uninit(),
                _pin: PhantomPinned,
                _thread_domain: PhantomData,
            });
            rechrom_ostringstream_construct(core::ptr::addr_of_mut!((*dst).storage).cast());
        }
    }
    pub unsafe fn str_at_assuming_native_abi<T: StdStringStorage>(
        self: Pin<&Self>,
        dst: *mut NativeStdString<T>,
    ) {
        unsafe {
            dst.write(NativeStdString::uninitialized_storage());
            rechrom_ostringstream_str_at(
                self.get_ref().storage.as_ptr().cast(),
                core::ptr::addr_of_mut!((*dst).storage).cast(),
            );
        }
    }
    pub unsafe fn take_str_at<T: StdStringStorage>(
        self: Pin<&mut Self>,
        dst: *mut NativeStdString<T>,
    ) {
        unsafe {
            dst.write(NativeStdString::uninitialized_storage());
            rechrom_ostringstream_take_str(
                self.get_unchecked_mut().storage.as_mut_ptr().cast(),
                core::ptr::addr_of_mut!((*dst).storage).cast(),
            );
        }
    }
}
unsafe extern "C" {
    fn rechrom_std_string_default_construct(dst: *mut c_void);
    fn rechrom_std_string_append(dst: *mut c_void, data: *const u8, len: usize);
    fn rechrom_std_string_clear(dst: *mut c_void);
    fn rechrom_ostringstream_take_str(src: *mut c_void, dst: *mut c_void);
    // Actual compiler va_list cursor is borrowed, not a second Rust va_end owner.
    pub fn rechrom_native_vspanprintf(
        dst: *mut u8,
        len: usize,
        format: *const u8,
        args: core::mem::ManuallyDrop<core::ffi::VaList<'_>>,
    ) -> c_int;
    pub fn rechrom_native_va_end_slot(slot: *mut c_void);
    pub fn rechrom_native_errno_address() -> *mut c_int;
    #[cfg(windows)]
    pub fn rechrom_native_get_last_error() -> c_ulong;
    #[cfg(windows)]
    pub fn rechrom_native_set_last_error(value: c_ulong);
}
#[cfg(sk_native_ostream_abi_configured)]
pub type ConfiguredStringStream = NativeStringStream<supplied_abi::SuppliedStringStreamStorage>;

// APPEND within interop/native_ostream.rs; exact configured SDK inputs only.
#[cfg(sk_native_ostream_abi_configured)]
impl ConfiguredStdString {
    pub fn validate_configured_abi() -> Result<(), AbiMismatch> {
        Self::check_abi()
    }
    /// Caller already validated configured ABI and supplies exclusive uninitialized
    /// final pinned storage. No initialization flag or second owner is created.
    pub unsafe fn configured_empty_at(dst: *mut Self) {
        unsafe {
            dst.write(Self::uninitialized_storage());
            rechrom_logging_std_string_empty_at(core::ptr::addr_of_mut!((*dst).storage).cast());
        }
    }
    pub unsafe fn configured_concat_at(dst: *mut Self, a: Pin<&Self>, b: Pin<&Self>) {
        unsafe {
            dst.write(Self::uninitialized_storage());
            rechrom_logging_std_string_concat_at(
                core::ptr::addr_of_mut!((*dst).storage).cast(),
                a.get_ref().storage.as_ptr().cast(),
                b.get_ref().storage.as_ptr().cast(),
            );
        }
    }
}
#[cfg(sk_native_ostream_abi_configured)]
impl ConfiguredOStringStream {
    /// Same final-storage contract; stream and return ABI were both validated by
    /// the containing LogMessage constructor before any initialized owner exists.
    pub unsafe fn configured_str_at(self: Pin<&Self>, dst: *mut ConfiguredStdString) {
        unsafe {
            dst.write(ConfiguredStdString::uninitialized_storage());
            rechrom_ostringstream_str_at(
                self.get_ref().storage.as_ptr().cast(),
                core::ptr::addr_of_mut!((*dst).storage).cast(),
            );
        }
    }
}
unsafe extern "C" {
    fn rechrom_logging_std_string_empty_at(dst: *mut c_void);
    fn rechrom_logging_std_string_concat_at(dst: *mut c_void, a: *const c_void, b: *const c_void);
}

// Append in the genuine interop/native_ostream module. This only constructs the
// original called base::StrCat return value in its original final return slot.
// No layout/ABI/storage instance or extra native object is supplied.
impl<S: StdStringStorage> NativeStdString<S> {
    /// Safety: S is the supplied exact native std::string ABI storage; dst is
    /// exclusive, uninitialized, final stationary storage. Inputs are original
    /// nonnull live NUL-terminated strings through this synchronous StrCat call.
    pub unsafe fn checkop_strcat_at_assuming_native_abi(
        dst: *mut Self,
        expr: *const c_char,
        v1: *const c_char,
        v2: *const c_char,
    ) {
        unsafe {
            dst.write(Self::uninitialized_storage());
            rechrom_checkop_strcat_at(core::ptr::addr_of_mut!((*dst).storage).cast(), expr, v1, v2);
        }
    }
}
unsafe extern "C" {
    fn rechrom_checkop_strcat_at(
        dst: *mut c_void,
        expr: *const c_char,
        v1: *const c_char,
        v2: *const c_char,
    );
}

impl<S: StdStringStorage> NativeStdString<S> {
    /// Safety: out is genuine exact-ABI final uninitialized return storage. Caller
    /// must immediately execute its actual native constructor at the returned slot
    /// before exposing/dropping the Rust wrapper. No initialized value is returned.
    pub(crate) unsafe fn native_return_slot(out: *mut Self) -> *mut c_void {
        unsafe {
            out.write(Self::uninitialized_storage());
            core::ptr::addr_of_mut!((*out).storage).cast()
        }
    }
}

impl<S: StringStreamStorage> NativeStringStream<S> {
    pub fn validate_abi() -> Result<(), AbiMismatch> {
        check::<S>(
            unsafe { rechrom_debug_stringstream_size() },
            unsafe { rechrom_debug_stringstream_align() },
            S::NATIVE_SIZE,
            S::NATIVE_ALIGN,
        )
    }
}
unsafe extern "C" {
    fn rechrom_debug_stringstream_size() -> usize;
    fn rechrom_debug_stringstream_align() -> usize;
}
