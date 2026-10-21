#![cfg(target_vendor = "apple")]
use super::SkUniqueCFRef::SkUniqueCFRef;
use crate::src::ports::SkTypeface_mac_ct::{
    CFDictionary, CFNumber, CFString, CTFont, CTFontDescriptor, CTFontRef, OpszVariation,
};
use core::ffi::{c_char, c_void};
use core::ptr;

// SDK ABI declarations from CoreFoundation CFDictionary/CFNumber/CFString and
// CoreText CTFont/CTFontDescriptor headers. These import native framework
// objects and callbacks; no Rust dictionary or font producer replaces them.
type Retain = unsafe extern "C" fn(*const c_void, *const c_void) -> *const c_void;
type Release = unsafe extern "C" fn(*const c_void, *const c_void);
type Description = unsafe extern "C" fn(*const c_void) -> *mut CFString;
type Equal = unsafe extern "C" fn(*const c_void, *const c_void) -> u8;
type Hash = unsafe extern "C" fn(*const c_void) -> usize;
#[repr(C)]
struct DictionaryKeys {
    version: isize,
    retain: Option<Retain>,
    release: Option<Release>,
    copy_description: Option<Description>,
    equal: Option<Equal>,
    hash: Option<Hash>,
}
#[repr(C)]
struct DictionaryValues {
    version: isize,
    retain: Option<Retain>,
    release: Option<Release>,
    copy_description: Option<Description>,
    equal: Option<Equal>,
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFTypeDictionaryKeyCallBacks: DictionaryKeys;
    static kCFTypeDictionaryValueCallBacks: DictionaryValues;
    fn CFDictionaryCreateMutable(
        allocator: *const c_void,
        capacity: isize,
        keys: *const DictionaryKeys,
        values: *const DictionaryValues,
    ) -> *mut CFDictionary;
    fn CFDictionarySetValue(
        dictionary: *mut CFDictionary,
        key: *const c_void,
        value: *const c_void,
    );
    fn CFNumberCreate(allocator: *const c_void, kind: isize, value: *const c_void)
        -> *mut CFNumber;
    fn CFNumberGetTypeID() -> usize;
    fn CFGetTypeID(value: *const c_void) -> usize;
    fn CFNumberGetValue(number: *mut CFNumber, kind: isize, output: *mut c_void) -> u8;
    fn CFStringCreateWithCString(
        allocator: *const c_void,
        text: *const c_char,
        encoding: u32,
    ) -> *mut CFString;
}
#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTFontCopyAttribute(font: CTFontRef, attribute: *mut CFString) -> *mut c_void;
    fn CTFontGetSize(font: CTFontRef) -> f64;
    fn CTFontDescriptorCreateWithAttributes(attributes: *mut CFDictionary)
        -> *mut CTFontDescriptor;
    fn CTFontCreateCopyWithAttributes(
        font: CTFontRef,
        size: f64,
        matrix: *const c_void,
        attributes: *mut CTFontDescriptor,
    ) -> CTFontRef;
}
const DOUBLE_TYPE: isize = 13;
const INT_TYPE: isize = 9;
const UTF8: u32 = 0x08000100;

// CFSTR is a compiler intrinsic in native source. The Rust adaptation obtains
// an owned native CFString with identical contents; dictionary type callbacks
// retain the key. The extra temporary CF allocation is explicitly adapted.
unsafe fn attribute_name(name: &'static [u8]) -> SkUniqueCFRef<CFString> {
    unsafe {
        SkUniqueCFRef::from_raw(CFStringCreateWithCString(
            ptr::null(),
            name.as_ptr().cast(),
            UTF8,
        ))
    }
}

// Source: src/utils/mac/SkCTFontCreateExactCopy.cpp:25-31
unsafe fn add_opsz_attr(attr: *mut CFDictionary, opsz: f64) {
    let number = unsafe {
        SkUniqueCFRef::from_raw(CFNumberCreate(
            ptr::null(),
            DOUBLE_TYPE,
            (&opsz as *const f64).cast(),
        ))
    };
    let name = unsafe { attribute_name(b"NSCTFontOpticalSizeAttribute\0") };
    unsafe { CFDictionarySetValue(attr, name.get().cast(), number.get().cast()) };
}

// Source: src/utils/mac/SkCTFontCreateExactCopy.cpp:34-40
unsafe fn add_notrak_attr(attr: *mut CFDictionary) {
    let zero = 0i32;
    let number = unsafe {
        SkUniqueCFRef::from_raw(CFNumberCreate(
            ptr::null(),
            INT_TYPE,
            (&zero as *const i32).cast(),
        ))
    };
    let name = unsafe { attribute_name(b"NSCTFontUnscaledTrackingAttribute\0") };
    unsafe { CFDictionarySetValue(attr, name.get().cast(), number.get().cast()) };
}

// Source: src/utils/mac/SkCTFontCreateExactCopy.h:32-33
// Source: src/utils/mac/SkCTFontCreateExactCopy.cpp:42-80
/// Safety: base_font is a live native CTFont through the synchronous framework calls.
pub unsafe fn SkCTFontCreateExactCopy(
    base_font: CTFontRef,
    text_size: f64,
    opsz_variation: OpszVariation,
) -> SkUniqueCFRef<CTFont> {
    let attr = unsafe {
        SkUniqueCFRef::from_raw(CFDictionaryCreateMutable(
            ptr::null(),
            0,
            ptr::addr_of!(kCFTypeDictionaryKeyCallBacks),
            ptr::addr_of!(kCFTypeDictionaryValueCallBacks),
        ))
    };
    if opsz_variation.isSet {
        unsafe { add_opsz_attr(attr.get(), opsz_variation.value) };
    } else {
        let name = unsafe { attribute_name(b"NSCTFontOpticalSizeAttribute\0") };
        let opsz = unsafe { SkUniqueCFRef::from_raw(CTFontCopyAttribute(base_font, name.get())) };
        let mut opsz_value = 0.0f64;
        if opsz.is_null()
            || unsafe { CFGetTypeID(opsz.get()) != CFNumberGetTypeID() }
            || unsafe {
                CFNumberGetValue(
                    opsz.get().cast(),
                    DOUBLE_TYPE,
                    (&mut opsz_value as *mut f64).cast(),
                )
            } == 0
            || opsz_value <= 0.0
        {
            opsz_value = unsafe { CTFontGetSize(base_font) };
        }
        unsafe { add_opsz_attr(attr.get(), opsz_value) };
    }
    unsafe { add_notrak_attr(attr.get()) };
    let desc = unsafe { SkUniqueCFRef::from_raw(CTFontDescriptorCreateWithAttributes(attr.get())) };
    unsafe {
        SkUniqueCFRef::from_raw(CTFontCreateCopyWithAttributes(
            base_font,
            text_size,
            ptr::null(),
            desc.get(),
        ))
    }
}
