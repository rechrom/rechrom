// Copyright 2017 Google LLC. BSD-style license.
#![cfg(SK_BUILD_FOR_IOS)]
use crate::src::core::SkString::SkString;
use crate::src::ports::SkTypeface_mac_ct::{CFString, CFStringRef};
use crate::src::utils::mac::SkUniqueCFRef::SkUniqueCFRef;
use core::ffi::{c_char, c_int, c_long, c_void};
// SDK opaque identities, not constructed Rust objects or substitute bundle owners.
#[repr(C)]
struct CFURL {
    _opaque: [u8; 0],
}
#[repr(C)]
struct CFBundle {
    _opaque: [u8; 0],
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFBundleGetMainBundle() -> *mut CFBundle;
    fn CFURLCreateFromFileSystemRepresentation(
        allocator: *const c_void,
        buffer: *const u8,
        len: c_long,
        is_directory: u8,
    ) -> *mut CFURL;
    fn CFURLCopyFileSystemPath(url: *mut CFURL, style: c_long) -> CFStringRef;
    fn CFBundleCopyResourceURL(
        bundle: *mut CFBundle,
        name: CFStringRef,
        resource_type: CFStringRef,
        subdir: CFStringRef,
    ) -> *mut CFURL;
    fn CFStringGetSystemEncoding() -> u32;
    fn CFStringGetCStringPtr(string: CFStringRef, encoding: u32) -> *const c_char;
    #[cfg(sk_cfstr_mode = "function")]
    fn __CFStringMakeConstantString(value: *const c_char) -> CFStringRef;
}
// External compiler-ABI mapping, zero Skia source credit. Clang ASTContext
// defines ordinary ObjC CF strings as const int*, int, const char*, long.
// CodeGenModule emits ASCII flags 0x07c8, four characters plus NUL backing,
// class-reference address and mutable __DATA,__cfstring object. The bridge
// selects this only for the actual non-Swift, unsigned-ObjC-isa Mach-O domain.
#[cfg(sk_cfstr_mode = "builtin_objc_unsigned_isa")]
#[repr(C)]
struct ConstantCFString {
    isa: *const c_int,
    flags: c_int,
    characters: *const c_char,
    length: c_long,
}
#[cfg(sk_cfstr_mode = "builtin_objc_unsigned_isa")]
unsafe extern "C" {
    static __CFConstantStringClassReference: [c_int; 0];
}
#[cfg(sk_cfstr_mode = "builtin_objc_unsigned_isa")]
#[unsafe(link_section = "__TEXT,__cstring,cstring_literals")]
static DATA_CHARACTERS: [u8; 5] = *b"data\0";
#[cfg(sk_cfstr_mode = "builtin_objc_unsigned_isa")]
#[unsafe(link_section = "__DATA,__cfstring")]
static mut DATA_CONSTANT: ConstantCFString = ConstantCFString {
    isa: core::ptr::addr_of!(__CFConstantStringClassReference).cast(),
    flags: 0x07c8,
    characters: DATA_CHARACTERS.as_ptr().cast(),
    length: 4,
};
#[cfg(sk_cfstr_mode = "builtin_objc_unsigned_isa")]
unsafe fn data_subdirectory() -> CFStringRef {
    core::ptr::addr_of_mut!(DATA_CONSTANT).cast::<CFString>()
}
// SDK's real alternative CFSTR expansion, only if __CONSTANT_CFSTRINGS__ was
// absent. This is not used as a replacement for the builtin constant branch.
#[cfg(sk_cfstr_mode = "function")]
unsafe fn data_subdirectory() -> CFStringRef {
    unsafe { __CFStringMakeConstantString(b"data\0".as_ptr().cast()) }
}
#[cfg(not(any(
    sk_cfstr_mode = "builtin_objc_unsigned_isa",
    sk_cfstr_mode = "function"
)))]
compile_error!("iOS CFSTR requires actual SDK/compiler CF runtime, constant-string and ObjC-isa signing inputs; unsupported domains are not substituted");
// Source: src/ports/SkOSFile_ios.h:18-54
/// Safety: path is a readable NUL-terminated native filesystem path. The
/// CoreFoundation functions obey their SDK ownership/null/encoding contracts.
/// The same four native +1 temporaries release in reverse local order.
pub(crate) unsafe fn ios_get_path_in_bundle(
    path: *const c_char,
    result: Option<&mut SkString>,
) -> bool {
    // Source: src/ports/SkOSFile_ios.h:20-23
    let main_bundle = unsafe { CFBundleGetMainBundle() };
    if main_bundle.is_null() {
        return false;
    }
    // Source: src/ports/SkOSFile_ios.h:27-30
    let path_url = unsafe {
        SkUniqueCFRef::from_raw(CFURLCreateFromFileSystemRepresentation(
            core::ptr::null(),
            path.cast(),
            libc::strlen(path) as c_long,
            0,
        ))
    };
    // Source: src/ports/SkOSFile_ios.h:32-32
    let path_ref = unsafe { SkUniqueCFRef::from_raw(CFURLCopyFileSystemPath(path_url.get(), 0)) };
    // Source: src/ports/SkOSFile_ios.h:36-40
    let file_url = unsafe {
        SkUniqueCFRef::from_raw(CFBundleCopyResourceURL(
            main_bundle,
            path_ref.get(),
            core::ptr::null_mut(),
            data_subdirectory(),
        ))
    };
    if file_url.is_null() {
        return false;
    }
    // Source: src/ports/SkOSFile_ios.h:41-43
    let Some(result) = result else {
        return true;
    };
    // Source: src/ports/SkOSFile_ios.h:46-46
    let file_path = unsafe { SkUniqueCFRef::from_raw(CFURLCopyFileSystemPath(file_url.get(), 0)) };
    // Source: src/ports/SkOSFile_ios.h:49-49
    let encoding = unsafe { CFStringGetSystemEncoding() };
    // Source: src/ports/SkOSFile_ios.h:52-53
    unsafe {
        result.set_cstr(CFStringGetCStringPtr(file_path.get(), encoding).cast());
    }
    true
}
