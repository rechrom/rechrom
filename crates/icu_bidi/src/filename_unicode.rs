//! ICU primitives for net filename generation. Source net/base/net_string_util_icu.cc:23-67;
//! base/i18n/icu_string_conversions.cc:153-234; base/i18n/file_util_icu.cc:122-165.
use std::ffi::{c_char, c_void, CString};
use std::ptr::NonNull;
pub enum UConverter {}
pub enum UNormalizer2 {}
pub enum UConverterToUnicodeArgs {}
type ToUnicodeCallback = unsafe extern "C" fn(
    *const c_void,
    *mut UConverterToUnicodeArgs,
    *const c_char,
    i32,
    i32,
    *mut i32,
);
unsafe extern "C" {
    #[link_name = concat!("ucnv_open_", env!("ICU_MAJOR"))]
    fn ucnv_open(name: *const c_char, status: *mut i32) -> *mut UConverter;
    #[link_name = concat!("ucnv_close_", env!("ICU_MAJOR"))]
    fn ucnv_close(converter: *mut UConverter);
    #[link_name = concat!("ucnv_setToUCallBack_", env!("ICU_MAJOR"))]
    fn ucnv_setToUCallBack(
        converter: *mut UConverter,
        callback: Option<ToUnicodeCallback>,
        context: *const c_void,
        old_callback: *mut Option<ToUnicodeCallback>,
        old_context: *mut *const c_void,
        status: *mut i32,
    );
    #[link_name = concat!("UCNV_TO_U_CALLBACK_STOP_", env!("ICU_MAJOR"))]
    fn stop(
        context: *const c_void,
        args: *mut UConverterToUnicodeArgs,
        code_units: *const c_char,
        length: i32,
        reason: i32,
        status: *mut i32,
    );
    #[link_name = concat!("ucnv_toUChars_", env!("ICU_MAJOR"))]
    fn ucnv_toUChars(
        converter: *mut UConverter,
        output: *mut u16,
        capacity: i32,
        input: *const c_char,
        length: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name = concat!("ucnv_toAlgorithmic_", env!("ICU_MAJOR"))]
    fn ucnv_toAlgorithmic(
        kind: i32,
        converter: *mut UConverter,
        output: *mut c_char,
        capacity: i32,
        input: *const c_char,
        length: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name = concat!("unorm2_getNFCInstance_", env!("ICU_MAJOR"))]
    fn unorm2_getNFCInstance(status: *mut i32) -> *const UNormalizer2;
    #[link_name = concat!("unorm2_normalize_", env!("ICU_MAJOR"))]
    fn unorm2_normalize(
        normalizer: *const UNormalizer2,
        input: *const u16,
        length: i32,
        output: *mut u16,
        capacity: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name = concat!("u_isUWhiteSpace_", env!("ICU_MAJOR"))]
    fn u_isUWhiteSpace(ch: i32) -> i8;
}
struct Converter(NonNull<UConverter>);
impl Converter {
    fn open(charset: &[u8]) -> Option<Self> {
        // Chromium passes a C string: an embedded NUL terminates the name.
        let charset = charset.split(|&b| b == 0).next()?;
        let name = CString::new(charset).ok()?;
        let mut status = 0;
        let ptr = unsafe { ucnv_open(name.as_ptr(), &mut status) };
        if status > 0 {
            return None;
        }
        NonNull::new(ptr).map(Self)
    }
}
impl Drop for Converter {
    fn drop(&mut self) {
        unsafe { ucnv_close(self.0.as_ptr()) }
    }
}
/// ConvertToUtf8 uses ICU's default substitution callback (RFC2047 branch).
pub fn ConvertToUtf8(input: &[u8], charset: &[u8]) -> Option<String> {
    let converter = Converter::open(charset)?;
    let length = i32::try_from(input.len()).ok()?;
    let capacity = i32::try_from(input.len().checked_mul(3)?).ok()?;
    let mut bytes = vec![0; capacity as usize];
    let mut status = 0;
    let written = unsafe {
        ucnv_toAlgorithmic(
            4,
            converter.0.as_ptr(),
            bytes.as_mut_ptr().cast(),
            capacity,
            input.as_ptr().cast(),
            length,
            &mut status,
        )
    };
    if status > 0 || written < 0 {
        return None;
    }
    bytes.truncate(written as usize);
    String::from_utf8(bytes).ok()
}
/// CodepageToUTF16(...FAIL), with the same input.len()+1 ICU capacity.
pub fn ConvertToUTF16(input: &[u8], charset: &[u8]) -> Option<Vec<u16>> {
    let converter = Converter::open(charset)?;
    let length = i32::try_from(input.len()).ok()?;
    let capacity = length.checked_add(1)?;
    let mut status = 0;
    unsafe {
        ucnv_setToUCallBack(
            converter.0.as_ptr(),
            Some(stop),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut status,
        )
    };
    let mut output = vec![0; capacity as usize];
    let written = unsafe {
        ucnv_toUChars(
            converter.0.as_ptr(),
            output.as_mut_ptr(),
            capacity,
            input.as_ptr().cast(),
            length,
            &mut status,
        )
    };
    if status > 0 || written < 0 {
        return None;
    }
    output.truncate(written as usize);
    Some(output)
}
pub fn ConvertToUtf8AndNormalize(input: &[u8], charset: &[u8]) -> Option<String> {
    let input = ConvertToUTF16(input, charset)?;
    let length = i32::try_from(input.len()).ok()?;
    let mut status = 0;
    let normalizer = unsafe { unorm2_getNFCInstance(&mut status) };
    if status > 0 || normalizer.is_null() {
        return None;
    }
    let mut output = vec![0; input.len().max(1)];
    let mut written = unsafe {
        unorm2_normalize(
            normalizer,
            input.as_ptr(),
            length,
            output.as_mut_ptr(),
            output.len() as i32,
            &mut status,
        )
    };
    if status == 15 && written >= 0 {
        output.resize(written as usize, 0);
        status = 0;
        written = unsafe {
            unorm2_normalize(
                normalizer,
                input.as_ptr(),
                length,
                output.as_mut_ptr(),
                written,
                &mut status,
            )
        };
    }
    if status > 0 || written < 0 {
        return None;
    }
    output.truncate(written as usize);
    String::from_utf16(&output).ok()
}
pub fn IsIllegalAnywhere(ch: char) -> bool {
    let cp = ch as u32;
    matches!(ch, '"' | '*' | '/' | ':' | '<' | '>' | '?' | '\\' | '|')
        || matches!(unsafe { crate::ubidi_ffi::u_charType(cp as i32) }, 15 | 16)
        || (0xfdd0..=0xfdef).contains(&cp)
        || (cp & 0xfffe) == 0xfffe
}
pub fn IsIllegalAtEnds(ch: char) -> bool {
    matches!(ch, '.' | '~') || unsafe { u_isUWhiteSpace(ch as i32) } != 0
}
