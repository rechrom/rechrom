use core::{
    ffi::{c_char, c_void},
    ptr,
};
// Source: partition_alloc_base/strings/safe_sprintf.h: actual typed Arg and inline template array.
pub mod internal {
    use super::*;
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Type {
        INT,
        UINT,
        STRING,
        POINTER,
    }
    #[derive(Clone, Copy)]
    pub struct Integer {
        pub i: i64,
        pub width: u8,
    }
    #[derive(Clone, Copy)]
    pub union Value {
        pub integer: Integer,
        pub str_: *const c_char,
        pub ptr_: *const c_void,
    }
    #[derive(Clone, Copy)]
    pub struct Arg {
        pub value: Value,
        pub type_: Type,
    }
    impl Arg {
        const fn int(i: i64, width: usize, type_: Type) -> Self {
            Self {
                value: Value {
                    integer: Integer {
                        i,
                        width: width as u8,
                    },
                },
                type_,
            }
        }
        pub const fn from_schar(v: core::ffi::c_schar) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_char>(),
                Type::INT,
            )
        }
        pub const fn from_uchar(v: core::ffi::c_uchar) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_char>(),
                Type::UINT,
            )
        }
        pub const fn from_short(v: core::ffi::c_short) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_short>(),
                Type::INT,
            )
        }
        pub const fn from_ushort(v: core::ffi::c_ushort) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_short>(),
                Type::UINT,
            )
        }
        pub const fn from_int(v: core::ffi::c_int) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_int>(),
                Type::INT,
            )
        }
        pub const fn from_uint(v: core::ffi::c_uint) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_int>(),
                Type::UINT,
            )
        }
        pub const fn from_long(v: core::ffi::c_long) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_long>(),
                Type::INT,
            )
        }
        pub const fn from_ulong(v: core::ffi::c_ulong) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_long>(),
                Type::UINT,
            )
        }
        pub const fn from_longlong(v: core::ffi::c_longlong) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_longlong>(),
                Type::INT,
            )
        }
        pub const fn from_ulonglong(v: core::ffi::c_ulonglong) -> Self {
            Self::int(
                v as i64,
                core::mem::size_of::<core::ffi::c_longlong>(),
                Type::UINT,
            )
        }
        pub const fn from_nullptr() -> Self {
            Self::int(0, super::CPP_NULL_SIZE, Type::INT)
        }
        pub const fn from_cstr(s: *const c_char) -> Self {
            Self {
                value: Value { str_: s },
                type_: Type::STRING,
            }
        }
        pub fn from_pointer<T>(p: *const T) -> Self {
            Self {
                value: Value { ptr_: p.cast() },
                type_: Type::POINTER,
            }
        }
    }
    pub use super::SafeSNPrintfArgs as SafeSNPrintf;
    #[cfg(pa_is_debug)]
    pub unsafe fn SetSafeSPrintfSSizeMaxForTest(max: usize) {
        super::kSSizeMax = max;
    }
    #[cfg(pa_is_debug)]
    pub unsafe fn GetSafeSPrintfSSizeMaxForTest() -> usize {
        super::kSSizeMax
    }
}
// sizeof(NULL) is a native compiler/header input, distinct from sizeof(nullptr_t).
const CPP_NULL_SIZE: usize = match env!("PA_CPP_NULL_SIZE").as_bytes() {
    b"4" => 4,
    b"8" => 8,
    _ => panic!("actual native sizeof(NULL) required"),
};
const kSSizeMaxConst: usize = usize::MAX >> 1;
#[cfg(pa_is_debug)]
static mut kSSizeMax: usize = kSSizeMaxConst;
#[cfg(not(pa_is_debug))]
const kSSizeMax: usize = kSSizeMaxConst;
macro_rules! debug_check {
    ($condition:expr,$native:literal) => {{
        let condition = $condition;
        #[cfg(pa_is_debug)]
        if !condition {
            unsafe {
                super::super::check::RawCheckFailure(
                    concat!("Check failed: ", $native, "\n\0").as_ptr().cast(),
                );
            }
        }
    }};
}
// Source: safe_sprintf.cc: class Buffer, exact counted/truncated-output state, no allocator/library calls.
struct Buffer {
    buffer_: *mut u8,
    size_: usize,
    count_: usize,
}
impl Buffer {
    unsafe fn new(buffer: *mut c_char, size: usize) -> Self {
        debug_check!(size > 0, "size > 0");
        debug_check!(size <= kSSizeMax, "size <= kSSizeMax");
        Self {
            buffer_: buffer.cast(),
            size_: size - 1,
            count_: 0,
        }
    }
    unsafe fn OutOfAddressableSpace(&self) -> bool {
        self.count_ == kSSizeMax - 1
    }
    unsafe fn GetCount(&self) -> isize {
        debug_check!(self.count_ < kSSizeMax, "count_ < kSSizeMax");
        self.count_ as isize
    }
    unsafe fn Out(&mut self, ch: u8) -> bool {
        if self.size_ >= 1 && self.count_ < self.size_ {
            *self.buffer_.add(self.count_) = ch;
            return self.IncrementCount(1);
        }
        self.IncrementCount(1);
        false
    }
    unsafe fn Pad(&mut self, pad: u8, mut padding: usize, len: usize) -> bool {
        debug_check!(pad != 0, "pad");
        debug_check!(padding <= kSSizeMax, "padding <= kSSizeMax");
        while padding > len {
            if !self.Out(pad) {
                padding -= 1;
                if padding != 0 {
                    self.IncrementCount(padding - len);
                }
                return false;
            }
            padding -= 1;
        }
        true
    }
    unsafe fn IncrementCount(&mut self, inc: usize) -> bool {
        debug_check!(inc <= kSSizeMax - 1, "inc <= kSSizeMax - 1");
        if self.count_ > kSSizeMax - 1 - inc {
            self.count_ = kSSizeMax - 1;
            return false;
        }
        self.count_ += inc;
        true
    }
    unsafe fn GetInsertionPoint(&self) -> *mut u8 {
        self.buffer_.add(self.count_.min(self.size_))
    }
    unsafe fn IToASCII(
        &mut self,
        sign: bool,
        upcase: bool,
        i: i64,
        base: usize,
        pad: u8,
        mut padding: usize,
        mut prefix: *const u8,
    ) -> bool {
        debug_check!(base >= 2, "base >= 2");
        debug_check!(base <= 16, "base <= 16");
        debug_check!(!sign || base == 10, "!sign || base == 10");
        debug_check!(pad == b'0' || pad == b' ', "pad == '0' || pad == ' '");
        debug_check!(padding <= kSSizeMax, "padding <= kSSizeMax");
        debug_check!(
            !(sign && !prefix.is_null() && *prefix != 0),
            "!(sign && prefix && *prefix)"
        );
        let mut minint = 0usize;
        let mut num: u64;
        if sign && i < 0 {
            prefix = b"-\0".as_ptr();
            if i == i64::MIN {
                minint = 1;
                num = (-(i + 1)) as u64;
            } else {
                num = (-i) as u64;
            }
        } else {
            num = i as u64;
        }
        let mut reverse_prefix = ptr::null();
        if !prefix.is_null() && *prefix != 0 {
            if pad == b'0' {
                while *prefix != 0 {
                    if padding != 0 {
                        padding -= 1;
                    }
                    self.Out(*prefix);
                    prefix = prefix.add(1);
                }
                prefix = ptr::null();
            } else {
                reverse_prefix = prefix;
                while *reverse_prefix != 0 {
                    reverse_prefix = reverse_prefix.add(1);
                }
            }
        } else {
            prefix = ptr::null();
        }
        // Native null-null subtraction is not performed on Rust pointers; its intended zero length is explicit.
        let prefix_length = if prefix.is_null() {
            0
        } else {
            reverse_prefix.offset_from(prefix) as usize
        };
        let start = self.count_;
        let mut discarded = 0;
        let mut started = false;
        loop {
            if self.count_ >= self.size_ {
                if start < self.size_ {
                    let mut pos = start;
                    while pos < self.size_ - 1 {
                        *self.buffer_.add(pos) = *self.buffer_.add(pos + 1);
                        pos += 1;
                    }
                    discarded += 1;
                    self.count_ -= 1;
                } else if self.count_ - self.size_ > 1 {
                    self.count_ -= 1;
                    discarded += 1;
                }
            }
            if num == 0 && started {
                if reverse_prefix.addr() > prefix.addr() {
                    reverse_prefix = reverse_prefix.sub(1);
                    self.Out(*reverse_prefix);
                } else {
                    self.Out(pad);
                }
            } else {
                started = true;
                let digits = if upcase {
                    b"0123456789ABCDEF"
                } else {
                    b"0123456789abcdef"
                };
                self.Out(*digits.get_unchecked((num % (base as u64)) as usize + minint));
            }
            minint = 0;
            num /= base as u64;
            if padding > 0 {
                padding -= 1;
                if discarded > 8 * core::mem::size_of::<u64>() + prefix_length {
                    self.IncrementCount(padding);
                    padding = 0;
                }
            }
            if num == 0 && padding == 0 && reverse_prefix.addr() <= prefix.addr() {
                break;
            }
        }
        if start < self.size_ {
            let mut front = self.buffer_.add(start);
            let mut back = self.GetInsertionPoint();
            loop {
                back = back.sub(1);
                if back <= front {
                    break;
                }
                let ch = *back;
                *back = *front;
                *front = ch;
                front = front.add(1);
            }
        }
        self.IncrementCount(discarded);
        discarded == 0
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            *self.GetInsertionPoint() = 0;
        }
    }
}
// Source: safe_sprintf.cc: internal::SafeSNPrintf. The native goto state edges are explicit loop exits.
pub unsafe fn SafeSNPrintfArgs(
    buf: *mut c_char,
    mut sz: usize,
    mut fmt: *const c_char,
    args: *const internal::Arg,
    max_args: usize,
) -> isize {
    use internal::Type;
    if (sz as isize) < 1 {
        return -1;
    }
    sz = sz.min(kSSizeMax);
    let mut buffer = Buffer::new(buf, sz);
    let mut cur_arg = 0u32;
    'format: while *fmt != 0 && !buffer.OutOfAddressableSpace() {
        let current = *fmt as u8;
        fmt = fmt.add(1);
        if current != b'%' {
            buffer.Out(current);
            continue;
        }
        let mut padding = 0usize;
        let mut pad = b' ';
        let mut ch = *fmt as u8;
        fmt = fmt.add(1);
        let mut fail = false;
        if ch >= b'0' && ch <= b'9' {
            pad = if ch == b'0' { b'0' } else { b' ' };
            loop {
                let digit = (ch - b'0') as usize;
                let max_padding = kSSizeMax - 1;
                let overflow =
                    padding > max_padding / 10 || 10 * padding > max_padding.wrapping_sub(digit);
                if overflow {
                    debug_check!(
                        padding <= max_padding / 10
                            && 10 * padding <= max_padding.wrapping_sub(digit),
                        "padding <= max_padding / 10 && 10 * padding <= max_padding - digit"
                    );
                } else {
                    padding = 10 * padding + digit;
                }
                if overflow || padding > max_padding {
                    if !overflow {
                        debug_check!(padding <= max_padding, "padding <= max_padding");
                    }
                    padding = max_padding;
                    loop {
                        ch = *fmt as u8;
                        fmt = fmt.add(1);
                        if ch < b'0' || ch > b'9' {
                            break;
                        }
                    }
                    if (cur_arg as usize) < max_args {
                        cur_arg = cur_arg.wrapping_add(1);
                    }
                    fail = true;
                    break;
                }
                ch = *fmt as u8;
                fmt = fmt.add(1);
                if ch < b'0' || ch > b'9' {
                    break;
                }
            }
        }
        if !fail {
            match ch {
                b'c' | b'd' | b'o' | b'x' | b'X' | b'p' | b's' => {
                    if cur_arg as usize >= max_args {
                        debug_check!((cur_arg as usize) < max_args, "cur_arg < max_args");
                        fail = true;
                    } else {
                        let arg = &*args.add(cur_arg as usize);
                        cur_arg = cur_arg.wrapping_add(1);
                        match ch {
                            b'c' => {
                                if arg.type_ != Type::INT && arg.type_ != Type::UINT {
                                    debug_check!(
                                        arg.type_ == Type::INT || arg.type_ == Type::UINT,
                                        "arg.type == Arg::INT || arg.type == Arg::UINT"
                                    );
                                    fail = true;
                                } else {
                                    buffer.Pad(b' ', padding, 1);
                                    let ch = arg.value.integer.i as u8;
                                    if ch == 0 {
                                        break 'format;
                                    }
                                    buffer.Out(ch);
                                }
                            }
                            b'd' | b'o' | b'x' | b'X' | b'p' => {
                                let mut i = 0i64;
                                let mut prefix = ptr::null();
                                if ch != b'p' {
                                    if arg.type_ != Type::INT && arg.type_ != Type::UINT {
                                        debug_check!(
                                            arg.type_ == Type::INT || arg.type_ == Type::UINT,
                                            "arg.type == Arg::INT || arg.type == Arg::UINT"
                                        );
                                        fail = true;
                                    } else {
                                        i = arg.value.integer.i;
                                        if ch != b'd' && arg.value.integer.width < 8 {
                                            i &= (1i64 << (8 * arg.value.integer.width)) - 1;
                                        }
                                    }
                                } else {
                                    if arg.type_ == Type::POINTER {
                                        i = arg.value.ptr_.addr() as i64;
                                    } else if arg.type_ == Type::STRING {
                                        i = arg.value.str_.addr() as i64;
                                    } else if arg.type_ == Type::INT
                                        && arg.value.integer.width as usize == CPP_NULL_SIZE
                                        && arg.value.integer.i == 0
                                    {
                                        i = 0;
                                    } else {
                                        debug_check!(
                                            arg.type_ == Type::POINTER || arg.type_ == Type::STRING,
                                            "arg.type == Arg::POINTER || arg.type == Arg::STRING"
                                        );
                                        fail = true;
                                    }
                                    prefix = b"0x\0".as_ptr();
                                }
                                if !fail {
                                    buffer.IToASCII(
                                        ch == b'd' && arg.type_ == Type::INT,
                                        ch != b'x',
                                        i,
                                        if ch == b'o' {
                                            8
                                        } else if ch == b'd' {
                                            10
                                        } else {
                                            16
                                        },
                                        pad,
                                        padding,
                                        prefix,
                                    );
                                }
                            }
                            b's' => {
                                let mut s = ptr::null();
                                if arg.type_ == Type::STRING {
                                    s = if arg.value.str_.is_null() {
                                        b"<NULL>\0".as_ptr().cast()
                                    } else {
                                        arg.value.str_
                                    };
                                } else if arg.type_ == Type::INT
                                    && arg.value.integer.width as usize == CPP_NULL_SIZE
                                    && arg.value.integer.i == 0
                                {
                                    s = b"<NULL>\0".as_ptr().cast();
                                } else {
                                    debug_check!(
                                        arg.type_ == Type::STRING,
                                        "arg.type == Arg::STRING"
                                    );
                                    fail = true;
                                }
                                if !fail {
                                    if padding != 0 {
                                        let mut len = 0;
                                        let mut src = s;
                                        while *src != 0 {
                                            src = src.add(1);
                                            len += 1;
                                        }
                                        buffer.Pad(b' ', padding, len);
                                    }
                                    let mut src = s;
                                    while *src != 0 {
                                        buffer.Out(*src as u8);
                                        src = src.add(1);
                                    }
                                }
                            }
                            _ => core::hint::unreachable_unchecked(),
                        }
                    }
                }
                b'%' => {
                    buffer.Out(*(fmt.sub(1)) as u8);
                    continue;
                }
                _ => {
                    fail = true;
                }
            }
        }
        if fail {
            buffer.Out(b'%');
            debug_check!(ch != 0, "ch");
            if ch == 0 {
                break;
            }
            buffer.Out(ch);
        }
    }
    buffer.GetCount()
}
// Source: safe_sprintf.cc: actual no-argument overload, including percent de-duplication.
pub unsafe fn SafeSNPrintf(buf: *mut c_char, mut sz: usize, fmt: *const c_char) -> isize {
    if (sz as isize) < 1 {
        return -1;
    }
    sz = sz.min(kSSizeMax);
    let mut buffer = Buffer::new(buf, sz);
    let mut src = fmt;
    while *src != 0 {
        buffer.Out(*src as u8);
        debug_check!(
            *src != b'%' as c_char || *src.add(1) == b'%' as c_char,
            "src[0] != '%' || src[1] == '%'"
        );
        if *src == b'%' as c_char && *src.add(1) == b'%' as c_char {
            src = src.add(1);
        }
        src = src.add(1);
    }
    buffer.GetCount()
}
pub unsafe fn SafeSPrintf<const N: usize>(
    buf: *mut [c_char; N],
    fmt: *const c_char,
    args: &[internal::Arg],
) -> isize {
    internal::SafeSNPrintf(buf.cast(), N, fmt, args.as_ptr(), args.len())
}
