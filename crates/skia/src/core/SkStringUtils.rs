use crate::src::core::SkString::SkString;
// Source: src/core/SkStringUtils.h:18-21
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SkScalarAsStringType {
    kDec_SkScalarAsStringType,
    kHex_SkScalarAsStringType,
}
// Source: src/core/SkStringUtils.h:23-23
// Source: src/core/SkStringUtils.cpp:20-35
pub fn SkAppendScalar(string: &mut SkString, value: f32, as_type: SkScalarAsStringType) {
    match as_type {
        SkScalarAsStringType::kHex_SkScalarAsStringType => unsafe {
            string.appendf(b"SkBits2Float(0x%08x)\0".as_ptr(), value.to_bits());
        },
        SkScalarAsStringType::kDec_SkScalarAsStringType => {
            let mut tmp = SkString::new();
            unsafe {
                tmp.printf(b"%.9g\0".as_ptr(), value as f64);
            }
            if tmp.containsChar(b'.') {
                tmp.appendUnichar('f' as i32);
            }
            string.append_string(&tmp);
        }
    }
}
// Source: src/core/SkStringUtils.h:25-27
pub fn SkAppendScalarDec(string: &mut SkString, value: f32) {
    SkAppendScalar(
        string,
        value,
        SkScalarAsStringType::kDec_SkScalarAsStringType,
    );
}
// Source: src/core/SkStringUtils.h:29-31
pub fn SkAppendScalarHex(string: &mut SkString, value: f32) {
    SkAppendScalar(
        string,
        value,
        SkScalarAsStringType::kHex_SkScalarAsStringType,
    );
}
// Source: src/core/SkStringUtils.h:34-34
// Source: src/core/SkStringUtils.cpp:37-62
pub fn SkTabString(string: &SkString, tab_count: i32) -> SkString {
    if tab_count <= 0 {
        return string.clone();
    }
    let mut tabs = SkString::new();
    for _ in 0..tab_count {
        tabs.append_view(b"\t");
    }
    let mut result = SkString::new();
    let mut input = string.c_str();
    let mut next_nl = unsafe { crate::src::core::SkString::SkStrFind(input, b"\n\0".as_ptr()) };
    while next_nl >= 0 {
        if next_nl > 0 {
            result.append_string(&tabs);
        }
        unsafe {
            result.append(input, (next_nl + 1) as usize);
            input = input.add((next_nl + 1) as usize);
        }
        next_nl = unsafe { crate::src::core::SkString::SkStrFind(input, b"\n\0".as_ptr()) };
    }
    if unsafe { *input } != 0 {
        result.append_string(&tabs);
        unsafe {
            result.append_cstr(input);
        }
    }
    result
}
// Source: src/core/SkStringUtils.h:36-36
// Source: src/core/SkStringUtils.cpp:64-89
/// Safety: source addresses count readable, aligned UTF16 code units; an empty
/// span permits null exactly as the native pointer/count entry does.
pub unsafe fn SkStringFromUTF16(source: *const u16, count: usize) -> SkString {
    use crate::src::core::SkUTF::{NextUTF16, ToUTF8};
    let mut result = SkString::new();
    let stop = source.wrapping_add(count);
    if count > 0 {
        crate::SkASSERT!(@source b"src/core/SkStringUtils.cpp\0".as_ptr(),68; @condition b"src\0".as_ptr(); !source.is_null());
        let mut n = 0usize;
        let mut end = stop;
        let mut ptr = source;
        while ptr < end {
            let last = ptr;
            let u = unsafe { NextUTF16(&mut ptr, stop) };
            let size = unsafe { ToUTF8(u, core::ptr::null_mut()) };
            if n > u32::MAX as usize - size {
                end = last;
                break;
            }
            n += size;
        }
        result = SkString::with_len(n);
        let mut out = result.data_mut();
        let mut ptr = source;
        while ptr < end {
            let u = unsafe { NextUTF16(&mut ptr, stop) };
            out = unsafe { out.add(ToUTF8(u, out)) };
        }
        crate::SkASSERT!(@source b"src/core/SkStringUtils.cpp\0".as_ptr(),86; @condition b"out == ret.data() + n\0".as_ptr(); out == unsafe{result.data_mut().add(n)});
    }
    result
}
// Source: src/core/SkStringUtils.h:38-42
#[cfg(target_os = "windows")]
unsafe extern "C" {
    #[link_name = "_stricmp"]
    pub fn SK_strcasecmp(a: *const u8, b: *const u8) -> core::ffi::c_int;
}
#[cfg(not(target_os = "windows"))]
unsafe extern "C" {
    #[link_name = "strcasecmp"]
    pub fn SK_strcasecmp(a: *const u8, b: *const u8) -> core::ffi::c_int;
}
// Source: src/core/SkStringUtils.h:44-53
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SkStrSplitMode {
    kStrict_SkStrSplitMode,
    kCoalesce_SkStrSplitMode,
}
unsafe extern "C" {
    fn strspn(string: *const u8, accept: *const u8) -> usize;
    fn strcspn(string: *const u8, reject: *const u8) -> usize;
}
// Source: src/core/SkStringUtils.h:56-59
// Source: src/core/SkStringUtils.cpp:91-122
/// Safety: input and delimiter are readable NUL-terminated native strings.
pub unsafe fn SkStrSplit<const MEM_MOVE: bool>(
    mut string: *const u8,
    delimiters: *const u8,
    split_mode: SkStrSplitMode,
    out: &mut crate::include::private::SkTArray::TArray<SkString, MEM_MOVE>,
) {
    if split_mode == SkStrSplitMode::kCoalesce_SkStrSplitMode {
        string = unsafe { string.add(strspn(string, delimiters)) };
    }
    if unsafe { *string } == 0 {
        return;
    }
    loop {
        let len = unsafe { strcspn(string, delimiters) };
        if split_mode == SkStrSplitMode::kStrict_SkStrSplitMode || len > 0 {
            let item = out.push_back_with(SkString::new);
            unsafe {
                item.set(string, len);
                string = string.add(len);
            }
        }
        if unsafe { *string } == 0 {
            break;
        }
        if split_mode == SkStrSplitMode::kCoalesce_SkStrSplitMode {
            string = unsafe { string.add(strspn(string, delimiters)) };
        } else {
            string = unsafe { string.add(1) };
        }
    }
}
// Source: src/core/SkStringUtils.h:61-64
pub unsafe fn SkStrSplit_default<const MEM_MOVE: bool>(
    string: *const u8,
    delimiters: *const u8,
    out: &mut crate::include::private::SkTArray::TArray<SkString, MEM_MOVE>,
) {
    unsafe {
        SkStrSplit(
            string,
            delimiters,
            SkStrSplitMode::kCoalesce_SkStrSplitMode,
            out,
        );
    }
}
