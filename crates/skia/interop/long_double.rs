//! External compiler primitive binding. No Rust long-double layout or value ABI
//! is asserted. The companion native TU must use the actual selected target and
//! floating-point compiler options; it must not be built for the Cargo host.
use core::{
    ffi::{c_char, c_int, c_void},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Format {
    pub size: usize,
    pub alignment: usize,
    pub radix: i32,
    pub mantissa_digits: i32,
    pub minimum_exponent: i32,
    pub maximum_exponent: i32,
}
unsafe extern "C" {
    fn sk_native_ld_format(out: *mut Format);
    fn sk_native_ld_with_slots(
        context: *mut c_void,
        callback: unsafe extern "C" fn(
            *mut c_void,
            *mut c_void,
            *mut c_void,
            *mut c_void,
            *mut c_void,
        ),
    );
    fn sk_native_ld_copy(out: *mut c_void, x: *const c_void);
    fn sk_native_ld_from_f32(out: *mut c_void, x: f32);
    fn sk_native_ld_from_f64(out: *mut c_void, x: f64);
    fn sk_native_ld_from_i64(out: *mut c_void, x: i64);
    fn sk_native_ld_from_u64(out: *mut c_void, x: u64);
    fn sk_native_ld_to_f32(x: *const c_void) -> f32;
    fn sk_native_ld_to_f64(x: *const c_void) -> f64;
    fn sk_native_ld_to_i32(x: *const c_void) -> i32;
    fn sk_native_ld_to_i64(x: *const c_void) -> i64;
    fn sk_native_ld_to_u64(x: *const c_void) -> u64;
    fn sk_native_ld_add(out: *mut c_void, x: *const c_void, y: *const c_void);
    fn sk_native_ld_sub(out: *mut c_void, x: *const c_void, y: *const c_void);
    fn sk_native_ld_mul(out: *mut c_void, x: *const c_void, y: *const c_void);
    fn sk_native_ld_div(out: *mut c_void, x: *const c_void, y: *const c_void);
    fn sk_native_ld_eq(x: *const c_void, y: *const c_void) -> u8;
    fn sk_native_ld_ne(x: *const c_void, y: *const c_void) -> u8;
    fn sk_native_ld_lt(x: *const c_void, y: *const c_void) -> u8;
    fn sk_native_ld_le(x: *const c_void, y: *const c_void) -> u8;
    fn sk_native_ld_gt(x: *const c_void, y: *const c_void) -> u8;
    fn sk_native_ld_ge(x: *const c_void, y: *const c_void) -> u8;
    fn sk_native_ld_floor(out: *mut c_void, x: *const c_void);
    fn sk_native_ld_ceil(out: *mut c_void, x: *const c_void);
    fn sk_native_ld_round(out: *mut c_void, x: *const c_void);
    fn sk_native_ld_trunc(out: *mut c_void, x: *const c_void);
    fn sk_native_ld_snprintf(
        out: *mut c_char,
        size: usize,
        format: *const c_char,
        x: *const c_void,
    ) -> c_int;
}
pub fn format() -> Format {
    let mut value = MaybeUninit::uninit();
    unsafe {
        sk_native_ld_format(value.as_mut_ptr());
        value.assume_init()
    }
}
#[derive(Clone, Copy)]
pub struct ValueRef<'a> {
    ptr: NonNull<c_void>,
    lease: PhantomData<&'a c_void>,
}
pub struct Slot<'a> {
    ptr: NonNull<c_void>,
    lease: PhantomData<&'a mut c_void>,
}
impl<'a> Slot<'a> {
    pub fn as_ref(&self) -> ValueRef<'_> {
        ValueRef {
            ptr: self.ptr,
            lease: PhantomData,
        }
    }
    pub fn copy_from(&mut self, x: ValueRef<'_>) {
        unsafe { sk_native_ld_copy(self.ptr.as_ptr(), x.ptr.as_ptr()) }
    }
    pub fn from_f32(&mut self, x: f32) {
        unsafe { sk_native_ld_from_f32(self.ptr.as_ptr(), x) }
    }
    pub fn from_f64(&mut self, x: f64) {
        unsafe { sk_native_ld_from_f64(self.ptr.as_ptr(), x) }
    }
    pub fn from_i64(&mut self, x: i64) {
        unsafe { sk_native_ld_from_i64(self.ptr.as_ptr(), x) }
    }
    pub fn from_u64(&mut self, x: u64) {
        unsafe { sk_native_ld_from_u64(self.ptr.as_ptr(), x) }
    }
}
macro_rules! binary {($($name:ident=>$ffi:ident),*)=>{$(impl Slot<'_>{pub fn $name(&mut self,x:ValueRef<'_>,y:ValueRef<'_>){unsafe{$ffi(self.ptr.as_ptr(),x.ptr.as_ptr(),y.ptr.as_ptr())}}})*};}
binary!(add=>sk_native_ld_add,sub=>sk_native_ld_sub,mul=>sk_native_ld_mul,div=>sk_native_ld_div);
macro_rules! unary {($($name:ident=>$ffi:ident),*)=>{$(impl Slot<'_>{pub fn $name(&mut self,x:ValueRef<'_>){unsafe{$ffi(self.ptr.as_ptr(),x.ptr.as_ptr())}}})*};}
unary!(floor=>sk_native_ld_floor,ceil=>sk_native_ld_ceil,round=>sk_native_ld_round,trunc=>sk_native_ld_trunc);
macro_rules! compare {($($name:ident=>$ffi:ident),*)=>{$(impl ValueRef<'_>{pub fn $name(self,y:ValueRef<'_>)->bool{unsafe{$ffi(self.ptr.as_ptr(),y.ptr.as_ptr())!=0}}})*};}
compare!(eq=>sk_native_ld_eq,ne=>sk_native_ld_ne,lt=>sk_native_ld_lt,le=>sk_native_ld_le,gt=>sk_native_ld_gt,ge=>sk_native_ld_ge);
impl ValueRef<'_> {
    pub fn to_f32(self) -> f32 {
        unsafe { sk_native_ld_to_f32(self.ptr.as_ptr()) }
    }
    pub fn to_f64(self) -> f64 {
        unsafe { sk_native_ld_to_f64(self.ptr.as_ptr()) }
    }
    /// Safety: original C++ integral conversion is defined for this operand.
    pub unsafe fn to_i32(self) -> i32 {
        unsafe { sk_native_ld_to_i32(self.ptr.as_ptr()) }
    }
    /// Safety: original C++ integral conversion is defined for this operand.
    pub unsafe fn to_i64(self) -> i64 {
        unsafe { sk_native_ld_to_i64(self.ptr.as_ptr()) }
    }
    /// Safety: original C++ integral conversion is defined for this operand.
    pub unsafe fn to_u64(self) -> u64 {
        unsafe { sk_native_ld_to_u64(self.ptr.as_ptr()) }
    }
    /// Safety: output is writable for size bytes; format is a terminated printf
    /// format whose sole variadic operand is this actual long double (%L...).
    pub unsafe fn snprintf(self, out: *mut c_char, size: usize, format: *const c_char) -> c_int {
        unsafe { sk_native_ld_snprintf(out, size, format, self.ptr.as_ptr()) }
    }
    /// Safety: ptr identifies an actual initialized native long double object
    /// borrowed immutably for 'a. No guessed byte buffer or narrower float allowed.
    pub unsafe fn from_native<'a>(ptr: NonNull<c_void>) -> ValueRef<'a> {
        ValueRef {
            ptr,
            lease: PhantomData,
        }
    }
}
/// Four actual native stack objects, no heap or guessed Rust inline payload.
/// The closure must not unwind across the native callback frame.
pub unsafe fn with_slots<R, F>(f: F) -> R
where
    F: for<'a> FnOnce([Slot<'a>; 4]) -> R,
{
    struct Context<F, R> {
        f: Option<F>,
        result: MaybeUninit<R>,
    }
    unsafe extern "C" fn invoke<F, R>(
        ctx: *mut c_void,
        a: *mut c_void,
        b: *mut c_void,
        c: *mut c_void,
        d: *mut c_void,
    ) where
        F: for<'a> FnOnce([Slot<'a>; 4]) -> R,
    {
        let context = unsafe { &mut *ctx.cast::<Context<F, R>>() };
        let slots = [a, b, c, d].map(|ptr| Slot {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            lease: PhantomData,
        });
        let f = context.f.take().unwrap();
        context.result.write(f(slots));
    }
    let mut context = Context {
        f: Some(f),
        result: MaybeUninit::uninit(),
    };
    unsafe {
        sk_native_ld_with_slots((&mut context as *mut Context<F, R>).cast(), invoke::<F, R>);
        context.result.assume_init()
    }
}
