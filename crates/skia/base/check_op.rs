use core::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void};
// libc ABI and C malloc/free ownership are the ones required by the source.
unsafe extern "C" {
    fn snprintf(buffer: *mut c_char, size: usize, format: *const c_char, ...) -> c_int;
    fn strdup(string: *const c_char) -> *mut c_char;
    fn malloc(size: usize) -> *mut c_void;
    fn free(pointer: *mut c_void);
}
// Source: base/check_op.cc:22-68
macro_rules! number_string {
    ($name:ident,$ty:ty,$format:literal) => {
        pub fn $name(value: $ty) -> *mut c_char {
            let mut buffer = [core::mem::MaybeUninit::<c_char>::uninit(); 50];
            unsafe {
                snprintf(
                    buffer.as_mut_ptr().cast(),
                    50,
                    concat!($format, "\0").as_ptr().cast(),
                    value,
                );
                strdup(buffer.as_ptr().cast())
            }
        }
    };
}
number_string!(CheckOpValueStr_int, c_int, "%d");
number_string!(CheckOpValueStr_unsigned, c_uint, "%u");
number_string!(CheckOpValueStr_long, c_long, "%ld");
number_string!(CheckOpValueStr_ulong, c_ulong, "%lu");
number_string!(CheckOpValueStr_longlong, c_longlong, "%lld");
number_string!(CheckOpValueStr_ulonglong, c_ulonglong, "%llu");
// Source: base/check_op.cc:70-76
pub fn CheckOpValueStr_pointer(value: *const c_void) -> *mut c_char {
    let mut buffer = [core::mem::MaybeUninit::<c_char>::uninit(); 50];
    unsafe {
        snprintf(
            buffer.as_mut_ptr().cast(),
            50,
            b"%p\0".as_ptr().cast(),
            value,
        );
        strdup(buffer.as_ptr().cast())
    }
}
// Source: base/check_op.cc:78-80
pub fn CheckOpValueStr_nullptr() -> *mut c_char {
    unsafe { strdup(b"nullptr\0".as_ptr().cast()) }
}
// Source: base/check_op.cc:82-85
/// Safety: the actual native string remains pinned and supplies its original
/// NUL-terminated c_str view for the synchronous matching strdup call.
pub unsafe fn CheckOpValueStr_native_string<S: StdStringStorage>(
    value: Pin<&NativeStdString<S>>,
) -> *mut c_char {
    unsafe { strdup(value.c_str()) }
}

// Source: base/check_op.cc:87-99
pub unsafe fn CheckOpValueStr_string_view(value: &[u8]) -> *mut c_char {
    unsafe {
        let ret = malloc(value.len() + 1).cast::<u8>();
        core::ptr::copy_nonoverlapping(value.as_ptr(), ret, value.len());
        ret.add(value.len()).write(0);
        ret.cast()
    }
}
// Source: base/check_op.cc:101-104
// CStr is a borrow of the actual c_str() view (including its terminating NUL).
pub fn CheckOpValueStr_cstring(value: &core::ffi::CStr) -> *mut c_char {
    unsafe { strdup(value.as_ptr()) }
}
// Source: base/check_op.cc:106-112
number_string!(CheckOpValueStr_double, f64, "%.6lf");
use crate::interop::native_ostream::{
    NativeOStream, NativeStdString, NativeStringStream, StdStringStorage, StringStreamStorage,
};
use core::{mem::MaybeUninit, pin::Pin};

// Rust cleanup projection for the same original stack native object. The pin
// borrow is not a second stream/string owner or allocation; it executes the
// original automatic native destructor exactly once, including Rust unwinding.
struct NativeStackValue<'a, T> {
    value: Pin<&'a mut T>,
}
impl<T> Drop for NativeStackValue<'_, T> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.value.as_mut().get_unchecked_mut());
        }
    }
}

// Source: base/check_op.cc:114-120
/// Safety: SS and ST have genuine supplied exact SDK ABI metadata, and the
/// actual native primitive append/bridge modules are linked. v is the original
/// live value passed to stream_func, which performs its actual operator<< and
/// cannot retain the borrowed stream. No storage layout or instance is guessed.
pub unsafe fn StreamValToStr<SS: StringStreamStorage, ST: StdStringStorage>(
    v: *const c_void,
    stream_func: unsafe fn(&mut NativeOStream<'_>, *const c_void),
) -> *mut c_char {
    let mut stream_slot = core::pin::pin!(MaybeUninit::<NativeStringStream<SS>>::uninit());
    let stream_ptr = unsafe { stream_slot.as_mut().get_unchecked_mut().as_mut_ptr() };
    unsafe {
        NativeStringStream::<SS>::initialize_at_assuming_native_abi(stream_ptr);
    }
    let mut ss = NativeStackValue {
        value: unsafe {
            stream_slot
                .as_mut()
                .map_unchecked_mut(|slot| slot.assume_init_mut())
        },
    };
    {
        let mut out = ss.value.as_mut().stream();
        unsafe {
            stream_func(&mut out, v);
        }
    }
    let mut string_slot = core::pin::pin!(MaybeUninit::<NativeStdString<ST>>::uninit());
    unsafe {
        ss.value
            .as_mut()
            .take_str_at(string_slot.as_mut().get_unchecked_mut().as_mut_ptr());
    }
    let message = NativeStackValue {
        value: unsafe {
            string_slot
                .as_mut()
                .map_unchecked_mut(|slot| slot.assume_init_mut())
        },
    };
    // Native temporary string dies at this full expression before ss. Rust
    // locals unwind in the same reverse order; neither MaybeUninit double-drops.
    unsafe { strdup(message.value.as_ref().c_str()) }
}

// Source: base/check_op.cc:122-131
/// Safety: ST is genuine exact SDK std::string storage. expr is a native C
/// string; v1/v2 are independent matching-malloc strings consumed here. The
/// original actual base::StrCat producer/return-slot bridge must be linked.
pub unsafe fn CreateCheckOpLogMessageString<ST: StdStringStorage>(
    expr: *const c_char,
    v1: *mut c_char,
    v2: *mut c_char,
) -> *mut c_char {
    let mut slot = core::pin::pin!(MaybeUninit::<NativeStdString<ST>>::uninit());
    unsafe {
        NativeStdString::<ST>::checkop_strcat_at_assuming_native_abi(
            slot.as_mut().get_unchecked_mut().as_mut_ptr(),
            expr,
            v1,
            v2,
        );
    }
    let message = NativeStackValue {
        value: unsafe {
            slot.as_mut()
                .map_unchecked_mut(|slot| slot.assume_init_mut())
        },
    };
    unsafe {
        free(v1.cast());
        free(v2.cast());
        strdup(message.value.as_ref().c_str())
    }
}
// Source: base/check_op.h:53-65
// The trait expresses the actual overload language operation; unsupported
// stream/enum/function-pointer domains are not given a substitute formatter.
pub trait CheckOpValue {
    unsafe fn CheckOpValueStr(&self) -> *mut c_char;
}
macro_rules! value_impl {
    ($ty:ty,$function:ident) => {
        impl CheckOpValue for $ty {
            unsafe fn CheckOpValueStr(&self) -> *mut c_char {
                $function(*self)
            }
        }
    };
}
value_impl!(i32, CheckOpValueStr_int);
value_impl!(u32, CheckOpValueStr_unsigned);
value_impl!(i64, CheckOpValueStr_longlong);
value_impl!(u64, CheckOpValueStr_ulonglong);
value_impl!(f64, CheckOpValueStr_double);
// Native short/char/bool/float select the exact generic ostream overload,
// not a promoted int/double printf overload. Their real CheckOstreamValue
// implementations below require genuine stringstream/string ABI storage through
// CheckOpValueStr_stream; no unconfigured automatic formatter is fabricated.
impl CheckOpValue for core::ffi::CStr {
    unsafe fn CheckOpValueStr(&self) -> *mut c_char {
        CheckOpValueStr_cstring(self)
    }
}
impl CheckOpValue for [u8] {
    unsafe fn CheckOpValueStr(&self) -> *mut c_char {
        unsafe { CheckOpValueStr_string_view(self) }
    }
}
// Source: base/check_op.h:61-61
#[derive(Clone, Copy)]
pub struct NativeNullptr;
impl CheckOpValue for NativeNullptr {
    unsafe fn CheckOpValueStr(&self) -> *mut c_char {
        CheckOpValueStr_nullptr()
    }
}
// Source: base/check_op.h:67-67
impl<S: StdStringStorage> CheckOpValue for NativeStdString<S> {
    /// Safety includes the same original native pinned-address lifetime/ABI
    /// contract; this borrow never takes another string owner.
    unsafe fn CheckOpValueStr(&self) -> *mut c_char {
        unsafe { CheckOpValueStr_native_string(Pin::new_unchecked(self)) }
    }
}

// Source: base/check_op.h:82-101
/// Maps the actual SupportsOstreamOperator<const T&> concept to the concrete
/// source type's original operator<<. No blanket formatter/provider is supplied.
pub trait CheckOstreamValue {
    fn stream_into(&self, stream: &mut NativeOStream<'_>);
}
pub unsafe fn CheckOpValueStr_stream<
    T: CheckOstreamValue,
    SS: StringStreamStorage,
    ST: StdStringStorage,
>(
    value: &T,
) -> *mut c_char {
    unsafe fn stream<T: CheckOstreamValue>(out: &mut NativeOStream<'_>, value: *const c_void) {
        // Rust shared-reference address is never an overloaded C++ operator&.
        // This is the corresponding addressof/const-volatile removal boundary;
        // native Objective-C ARC and actual volatile references are not invented.
        unsafe {
            (&*value.cast::<T>()).stream_into(out);
        }
    }
    unsafe { StreamValToStr::<SS, ST>(core::ptr::from_ref(value).cast(), stream::<T>) }
}
// Source: base/check_op.h:82-101
// Concrete actual ostream operator domains needed by the generic overload.
impl CheckOstreamValue for i8 {
    fn stream_into(&self, out: &mut NativeOStream<'_>) {
        out.insert_char(*self as u8);
    }
}
impl CheckOstreamValue for u8 {
    fn stream_into(&self, out: &mut NativeOStream<'_>) {
        out.insert_char(*self);
    }
}
impl CheckOstreamValue for i16 {
    fn stream_into(&self, out: &mut NativeOStream<'_>) {
        out.insert_short(*self);
    }
}
impl CheckOstreamValue for u16 {
    fn stream_into(&self, out: &mut NativeOStream<'_>) {
        out.insert_ushort(*self);
    }
}
impl CheckOstreamValue for bool {
    fn stream_into(&self, out: &mut NativeOStream<'_>) {
        out.insert_bool(*self);
    }
}
impl CheckOstreamValue for f32 {
    fn stream_into(&self, out: &mut NativeOStream<'_>) {
        out.insert_float(*self);
    }
}

// The selected C++ overload concept is supplied explicitly for a concrete Rust
// source type. No overlapping blanket specialization or false fallback is used.
#[macro_export]
macro_rules! IMPL_CHECK_OP_STREAM_VALUE {
    ($type:ty,$stream_storage:ty,$string_storage:ty) => {
        impl $crate::base::check_op::CheckOpValue for $type {
            unsafe fn CheckOpValueStr(&self) -> *mut core::ffi::c_char {
                unsafe {
                    $crate::base::check_op::CheckOpValueStr_stream::<
                        Self,
                        $stream_storage,
                        $string_storage,
                    >(self)
                }
            }
        }
    };
}
// Source: base/check_op.h:104-124
impl<T> CheckOpValue for *const T {
    unsafe fn CheckOpValueStr(&self) -> *mut c_char {
        CheckOpValueStr_pointer(self.cast())
    }
}
impl<T> CheckOpValue for *mut T {
    unsafe fn CheckOpValueStr(&self) -> *mut c_char {
        CheckOpValueStr_pointer(self.cast())
    }
}
// Source: base/check_op.h:127-134
pub trait CheckToStringValue {
    type Returned: CheckOpValue;
    fn ToString(&self) -> Self::Returned;
}
pub unsafe fn CheckOpValueStr_to_string<T: CheckToStringValue>(value: &T) -> *mut c_char {
    let temporary = value.ToString();
    unsafe { temporary.CheckOpValueStr() }
}
#[macro_export]
macro_rules! IMPL_CHECK_OP_TO_STRING_VALUE {
    ($type:ty) => {
        impl $crate::base::check_op::CheckOpValue for $type {
            unsafe fn CheckOpValueStr(&self) -> *mut core::ffi::c_char {
                unsafe { $crate::base::check_op::CheckOpValueStr_to_string(self) }
            }
        }
    };
}
// Source: base/check_op.h:148-153
pub trait CheckEnumValue {
    type Underlying: CheckOpValue;
    fn underlying(&self) -> Self::Underlying;
}
pub unsafe fn CheckOpValueStr_enum<T: CheckEnumValue>(value: &T) -> *mut c_char {
    unsafe { value.underlying().CheckOpValueStr() }
}
#[macro_export]
macro_rules! IMPL_CHECK_OP_ENUM_VALUE {
    ($type:ty) => {
        impl $crate::base::check_op::CheckOpValue for $type {
            unsafe fn CheckOpValueStr(&self) -> *mut core::ffi::c_char {
                unsafe { $crate::base::check_op::CheckOpValueStr_enum(self) }
            }
        }
    };
}
// Source: base/check_op.h:202-234
// Rust comparisons use the concrete type's actual operator implementation;
// no cross-signed-width C++ arithmetic conversions are silently fabricated.
macro_rules! check_comparison {
 ($name:ident,$bound:ident,$op:tt)=>{
  pub unsafe fn $name<T:CheckOpValue+$bound<U>+?Sized,U:CheckOpValue+?Sized,F>(v1:&T,v2:&U,on_failure:F)->*mut super::check::LogMessage
  where F:FnOnce(*mut c_char,*mut c_char)->*mut super::check::LogMessage {
   if v1 $op v2 {return core::ptr::null_mut();}
   unsafe{on_failure(v1.CheckOpValueStr(),v2.CheckOpValueStr())}
  }
 };
}
check_comparison!(CheckEQImpl,PartialEq,==);
check_comparison!(CheckNEImpl,PartialEq,!=);
check_comparison!(CheckLEImpl,PartialOrd,<=);
check_comparison!(CheckLTImpl,PartialOrd,<);
check_comparison!(CheckGEImpl,PartialOrd,>=);
check_comparison!(CheckGTImpl,PartialOrd,>);

// Source: base/check_op.h:214-224
// The original fundamental overload copies both operands before comparison;
// it must not borrow v1 while evaluating a caller's second argument.
macro_rules! fundamental_comparison {
    ($name:ident,$reference:ident,$bound:ident) => {
        pub unsafe fn $name<T: CheckOpValue + Copy + $bound<U>, U: CheckOpValue + Copy, F>(
            v1: T,
            v2: U,
            on_failure: F,
        ) -> *mut super::check::LogMessage
        where
            F: FnOnce(*mut c_char, *mut c_char) -> *mut super::check::LogMessage,
        {
            unsafe { $reference(&v1, &v2, on_failure) }
        }
    };
}
fundamental_comparison!(CheckEQFundamental, CheckEQImpl, PartialEq);
fundamental_comparison!(CheckNEFundamental, CheckNEImpl, PartialEq);
fundamental_comparison!(CheckLEFundamental, CheckLEImpl, PartialOrd);
fundamental_comparison!(CheckLTFundamental, CheckLTImpl, PartialOrd);
fundamental_comparison!(CheckGEFundamental, CheckGEImpl, PartialOrd);
fundamental_comparison!(CheckGTFundamental, CheckGTImpl, PartialOrd);
