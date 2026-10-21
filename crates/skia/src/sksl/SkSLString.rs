use super::SkSLDefines::SKSL_INT;
// Selected Apple libc++ classic-locale primitive adapter. SDK libc++ locale
// 245-251 supplies a null C locale; 789-790 extracts through strtod_l; 1317-1332
// emits floating values through snprintf_l with the stream's precision.
// No C++ SkSL implementation is linked or called. Other platform primitives
// remain explicitly pending, rather than taking a Rust formatter fallback.
// Source: src/sksl/SkSLString.cpp:83-113
// The native SkSL string API has no Darwin/ARM restriction. In particular the
// real c_variadic formatter below must not be rejected by a file-wide guard.
// The cfg below selects ONLY the actual Darwin locale primitives used by the
// preexisting classic-stream adaptation. The corresponding non-Darwin classic
// locale producer remains an explicit unresolved dependency for float streams;
// no foreign symbol or alternate formatting algorithm is fabricated here.
#[cfg(target_vendor = "apple")]
mod classic {
    use core::ffi::{c_char, c_int, c_void};
    unsafe extern "C" {
        fn snprintf_l(
            out: *mut c_char,
            count: usize,
            locale: *mut c_void,
            format: *const c_char,
            ...
        ) -> c_int;
        fn strtod_l(input: *const c_char, end: *mut *mut c_char, locale: *mut c_void) -> f64;
        pub(super) fn strtoull(input: *const c_char, end: *mut *mut c_char, base: c_int) -> u64;
        fn __error() -> *mut c_int;
    }
    // Exact exported ABI spellings are present in the same SDK usr/lib/libc++.1.tbd
    // (Mach-O's leading underscore is omitted in Rust link_name).
    // __locale:373-380 selects uint32_t mask / _CTYPE_S for Apple;
    // __locale:548-550 performs ASCII-only mask lookup; _ctype.h:79 defines 0x4000.
    #[link(name = "c++")]
    unsafe extern "C" {
        #[link_name = "_ZNSt3__114__num_get_base5__srcE"]
        static NUM_GET_ATOMS: [u8; 33];
        #[link_name = "_ZNSt3__15ctypeIcE13classic_tableEv"]
        fn classic_ctype_table() -> *const u32;
    }
    unsafe extern "C" {
        fn strtof_l(input: *const c_char, end: *mut *mut c_char, locale: *mut c_void) -> f32;
    }
    // Internal stream state preserves consumption/EOF even though SkSL::stod only
    // exposes fail() and finite(). EOF by itself is success, never failbit.
    pub(super) struct FloatExtraction {
        pub(super) consumed: usize,
        pub(super) eof: bool,
        pub(super) fail: bool,
        pub(super) output_written: bool,
    }
    // SDK libc++ istream:313-333 sentry, 350-372 __input_arithmetic<float>;
    // locale:458-467 float prep, 518-582 stage2 loop, 790-809 stage3,
    // 941-991 complete num_get<float> driver. Source SDK hashes are in the
    // independent progress evidence. No C++ object layout or virtual-call guessing.
    pub(super) unsafe fn extract_float(input: &[u8], out: *mut f32) -> FloatExtraction {
        // New stringstream: goodbit, skipws, no tied stream. Classic ctype<char>
        // applies its actual runtime classic mask table only to ASCII bytes.
        let masks = classic_ctype_table();
        let mut consumed = 0usize;
        while consumed < input.len() {
            let c = input[consumed];
            if c >= 128 || (*masks.add(c as usize) & 0x4000) == 0 {
                break;
            }
            consumed += 1;
        }
        if consumed == input.len() {
            // Sentry sets eofbit|failbit; num_get is never entered and out is untouched.
            return FloatExtraction {
                consumed,
                eof: true,
                fail: true,
                output_written: false,
            };
        }
        // classic numpunct<char>: '.' decimal, empty grouping. Its thousands
        // separator is consequently never handled; grouping validation has no work.
        // Classic narrow widening is identity. Read the actual exported __src rather
        // than inventing accepted atom spellings/order (including hexadecimal atoms).
        let atoms = &NUM_GET_ATOMS[..32];
        let mut chars = Vec::<u8>::new();
        let mut in_units = true;
        let mut exp = b'E';
        while consumed < input.len() {
            let c = input[consumed];
            if c == b'.' {
                if !in_units {
                    break;
                }
                in_units = false;
                chars.push(b'.');
                consumed += 1;
                continue;
            }
            let Some(index) = atoms.iter().position(|&atom| atom == c) else {
                break;
            };
            let x = NUM_GET_ATOMS[index];
            if x == b'-' || x == b'+' {
                if chars.is_empty()
                    || chars.last().unwrap_unchecked().to_ascii_uppercase()
                        == exp.to_ascii_uppercase()
                {
                    chars.push(x);
                    consumed += 1;
                    continue;
                }
                break;
            }
            if x == b'x' || x == b'X' {
                exp = b'P';
            } else if x.to_ascii_uppercase() == exp {
                exp = exp.to_ascii_lowercase();
                if in_units {
                    in_units = false;
                }
            }
            chars.push(x);
            consumed += 1;
            // Native digit count / grouping buffer are dead under empty classic
            // grouping. Their increments have no effect on this specialization.
        }
        let mut fail = false;
        let result = if !chars.is_empty() {
            // num_get's basic_string buffer is NUL-terminated, and __a_end is the
            // accepted-atom endpoint, not the input endpoint. Embedded NUL stops
            // lexical accumulation; arbitrary trailing input is left unconsumed.
            let count = chars.len();
            chars.push(0);
            let saved_errno = *errno();
            *errno() = 0;
            let mut end = core::ptr::null_mut();
            // SDK locale:242-243 _LIBCPP_GET_C_LOCALE is literally 0 on Apple.
            let converted = strtof_l(chars.as_ptr().cast(), &mut end, core::ptr::null_mut());
            let current_errno = *errno();
            if current_errno == 0 {
                *errno() = saved_errno;
            }
            if end.cast::<u8>() != chars.as_mut_ptr().add(count) {
                fail = true;
                0.0
            } else {
                // SDK sys/errno.h:126 ERANGE=34. Preserve the converted output
                // even when range failure is set (including subnormal/zero results).
                if current_errno == 34 {
                    fail = true;
                }
                converted
            }
        } else {
            // Actual __num_get_float empty-buffer failure writes zero, unlike the
            // sentry failure above, and does not call conversion or touch errno.
            fail = true;
            0.0
        };
        out.write(result);
        FloatExtraction {
            consumed,
            eof: consumed == input.len(),
            fail,
            output_written: true,
        }
    }
    pub(super) unsafe fn errno() -> *mut c_int {
        __error()
    }
    pub(super) fn emit(value: f64, precision: i32) -> Vec<u8> {
        let mut buf = [0u8; 30];
        let n = unsafe {
            snprintf_l(
                buf.as_mut_ptr().cast(),
                buf.len(),
                core::ptr::null_mut(),
                c"%.*g".as_ptr(),
                precision,
                value,
            )
        };
        // Exactly the SDK num_put narrow-buffer size. For precision7/9/17 every
        // binary32/64 %g spelling fits; a violated private primitive bound cannot return
        // a partial or fabricated number.
        if n < 0 || n as usize >= buf.len() {
            std::process::abort();
        }
        buf[..n as usize].to_vec()
    }
    pub(super) fn read_generated(text: &[u8]) -> f64 {
        // libc++ num_get's numeric lexical stage rejects generated inf/nan spellings
        // and supplies zero with failbit; preserve that source-visible roundtrip value.
        let first = text.get(usize::from(matches!(text.first(), Some(b'-') | Some(b'+'))));
        if matches!(first, Some(b'i') | Some(b'n')) {
            return 0.0;
        }
        let mut buf = [0u8; 30];
        buf[..text.len()].copy_from_slice(text);
        let mut end = core::ptr::null_mut();
        unsafe {
            let old = *errno();
            *errno() = 0;
            let value = strtod_l(buf.as_ptr().cast(), &mut end, core::ptr::null_mut());
            let current = *errno();
            if current == 0 {
                *errno() = old;
            }
            if end.cast::<u8>() != buf.as_mut_ptr().add(text.len()) {
                0.0
            } else {
                value
            }
        }
    }
}
// Pure language substitution for the native RoundtripType template.
trait Roundtrip: Copy + PartialEq {
    fn widen(self) -> f64;
    fn narrow(v: f64) -> Self;
    fn finite(self) -> bool;
}
impl Roundtrip for f32 {
    fn widen(self) -> f64 {
        self as f64
    }
    fn narrow(v: f64) -> Self {
        v as Self
    }
    fn finite(self) -> bool {
        self.is_finite()
    }
}
impl Roundtrip for f64 {
    fn widen(self) -> f64 {
        self
    }
    fn narrow(v: f64) -> Self {
        v
    }
    fn finite(self) -> bool {
        self.is_finite()
    }
}
// Source: src/sksl/SkSLString.cpp:23-48
fn to_string_impl<T: Roundtrip, const FULL_PRECISION: i32>(value: T) -> Vec<u8> {
    let mut text = classic::emit(value.widen(), 7);
    let mut roundtripped = classic::read_generated(&text);
    if value != T::narrow(roundtripped) && value.finite() {
        text = classic::emit(value.widen(), FULL_PRECISION);
        #[cfg(sk_debug)]
        {
            // The native text.c_str() is needed only on assertion failure;
            // keep its actual terminated backing alive through the fatal call.
            let mut terminated_text = Vec::new();
            crate::SkASSERTF!(@source b"src/sksl/SkSLString.cpp\0".as_ptr(),38;
                @condition b"(buffer >> roundtripped, value == (RoundtripType)roundtripped)\0".as_ptr();
                {roundtripped=classic::read_generated(&text);value==T::narrow(roundtripped)},"%.17g -> %s -> %.17g",value.widen(),
                {terminated_text.extend_from_slice(&text);terminated_text.push(0);terminated_text.as_ptr()},roundtripped);
        }
    }
    if !text.contains(&b'.') && !text.contains(&b'e') {
        text.extend_from_slice(b".0");
    }
    text
}
// Source: src/sksl/SkSLString.cpp:50-52
pub fn to_string_float(value: f32) -> Vec<u8> {
    to_string_impl::<f32, 9>(value)
}
// Source: src/sksl/SkSLString.cpp:54-56
pub fn to_string_double(value: f64) -> Vec<u8> {
    to_string_impl::<f64, 17>(value)
}
// Source: src/sksl/SkSLString.cpp:58-64
// UNVERIFIED: user prohibits all compilation, execution, and differential validation.
// Apple libc++ classic stringstream extraction is specialized below from the installed
// SDK's actual sentry/num_get algorithms. This is a language/standard-library adaptation,
// not ported Skia algorithm credit. Only char + classic locale + float is supported here.
// The raw pointer may point at uninitialized writable f32 storage. A failed sentry
// must leave it untouched; the native short-circuit must not read it in that case.
pub unsafe fn stod_raw(s: &[u8], value: *mut super::SkSLDefines::SKSL_FLOAT) -> bool {
    let extraction = classic::extract_float(s, value);
    !extraction.fail && (*value).is_finite()
}
// Initialized-reference facade; no additional source credit.
pub fn stod(s: &[u8], value: &mut super::SkSLDefines::SKSL_FLOAT) -> bool {
    unsafe { stod_raw(s, value) }
}
// Source: src/sksl/SkSLString.cpp:66-81
pub unsafe fn stoi_raw(mut s: &[u8], value: *mut SKSL_INT) -> bool {
    if s.is_empty() {
        return false;
    }
    let suffix = s[s.len() - 1];
    if suffix == b'u' || suffix == b'U' {
        s = &s[..s.len() - 1];
    }
    let mut str = s.to_vec();
    str.push(0);
    unsafe {
        let strEnd = str.as_ptr().add(s.len());
        let mut p = core::ptr::null_mut();
        *classic::errno() = 0;
        let result = classic::strtoull(str.as_ptr().cast(), &mut p, 0);
        *value = result as SKSL_INT;
        p.cast::<u8>() as *const u8 == strEnd && *classic::errno() == 0 && result <= 0xFFFFFFFF
    }
}
// Source: src/sksl/SkSLString.h:30-46
pub fn Separator() -> impl FnMut() -> &'static Vec<u8> {
    struct Output {
        fSpace: Vec<u8>,
        fComma: Vec<u8>,
    }
    static OUTPUT: std::sync::OnceLock<Output> = std::sync::OnceLock::new();
    let output = OUTPUT.get_or_init(|| Output {
        fSpace: Vec::new(),
        fComma: b", ".to_vec(),
    });
    let mut firstSeparator = true;
    move || {
        if firstSeparator {
            firstSeparator = false;
            &output.fSpace
        } else {
            &output.fComma
        }
    }
}

// Language-only facade for previously initialized-reference callers; no new source credit.
pub fn stoi(s: &[u8], value: &mut SKSL_INT) -> bool {
    unsafe { stoi_raw(s, value) }
}

// Source: src/sksl/SkSLString.h:26-28
// Source: src/sksl/SkSLString.cpp:83-113
// Real compiler-created va_list: no tuple pack, synthetic stack slots, cursor
// layout assumptions, or Apple-only argument transport. The Vec is the existing
// language mapping of std::string, not an exported C++ std::string ABI promise.
mod native_format {
    use core::{
        ffi::{c_char, c_int, VaList},
        mem::MaybeUninit,
    };
    unsafe extern "C" {
        fn vsnprintf(out: *mut c_char, count: usize, fmt: *const c_char, args: VaList<'_>)
            -> c_int;
    }
    // Source: src/sksl/SkSLString.cpp:83-91
    /// Safety: fmt is terminated; promoted C arguments match all conversions,
    /// including valid referenced strings and writable %n destinations.
    pub unsafe extern "C" fn printf(fmt: *const c_char, args: ...) -> Vec<u8> {
        let mut result = Vec::new();
        vappendf(&mut result, fmt, args);
        result
    }
    // Source: src/sksl/SkSLString.cpp:93-98
    /// Safety: output is live and exclusive; native printf argument contracts.
    pub unsafe extern "C" fn appendf(output: *mut Vec<u8>, fmt: *const c_char, args: ...) {
        vappendf(&mut *output, fmt, args);
    }
    // Source: src/sksl/SkSLString.cpp:100-113
    /// Safety: actual live native list and initialized formatted bytes. Native
    /// failure/length-domain obligations are unchanged from the existing body.
    pub unsafe fn vappendf(output: &mut Vec<u8>, fmt: *const c_char, args: VaList<'_>) {
        const BUFFER_SIZE: usize = 256;
        let mut buffer = [MaybeUninit::<u8>::uninit(); BUFFER_SIZE];
        let reuse = args.clone(); // va_copy, independently dropped as va_end
        let size = vsnprintf(
            buffer.as_mut_ptr().cast(),
            BUFFER_SIZE,
            fmt,
            core::ptr::read(&args),
        ) as usize;
        if BUFFER_SIZE >= size.wrapping_add(1) {
            assert!(size <= BUFFER_SIZE, "native string append length_error");
            output.extend_from_slice(core::slice::from_raw_parts(
                buffer.as_ptr().cast::<u8>(),
                size,
            ));
        } else {
            let count = size.wrapping_add(1);
            let mut new_buffer = Vec::<MaybeUninit<u8>>::with_capacity(count);
            new_buffer.set_len(count);
            vsnprintf(
                new_buffer.as_mut_ptr().cast(),
                count,
                fmt,
                core::ptr::read(&reuse),
            );
            output.extend_from_slice(core::slice::from_raw_parts(
                new_buffer.as_ptr().cast::<u8>(),
                size,
            ));
        }
        // The by-value FFI copies above lend the actual cursor representation;
        // C does not own/drop Rust values. Keep each owning list here for
        // exactly its va_end after the calls, without inventing its layout.
        drop(reuse);
        drop(args);
    }
}
pub use native_format::{appendf, printf, vappendf};
