#![allow(non_snake_case)]

// ICU exports a version suffix in its C ABI. build.rs discovers that suffix
// from icu-uc.pc, while these declarations preserve the vendor header's
// function signatures used by the translated font and bidi modules.
// C++ headers: icu_bidi/unicode/ubidi.h, uchar.h, uscript.h.
pub enum UBidi {}

unsafe extern "C" {
    #[link_name = concat!("ubidi_open_", env!("ICU_MAJOR"))]
    pub fn ubidi_open() -> *mut UBidi;
    #[link_name = concat!("ubidi_close_", env!("ICU_MAJOR"))]
    pub fn ubidi_close(bidi: *mut UBidi);
    #[link_name = concat!("ubidi_setPara_", env!("ICU_MAJOR"))]
    pub fn ubidi_setPara(
        bidi: *mut UBidi,
        text: *const u16,
        length: i32,
        para_level: u8,
        embedding_levels: *mut u8,
        error: *mut i32,
    );
    #[link_name = concat!("ubidi_getDirection_", env!("ICU_MAJOR"))]
    pub fn ubidi_getDirection(bidi: *const UBidi) -> i32;
    #[link_name = concat!("ubidi_getParaLevel_", env!("ICU_MAJOR"))]
    pub fn ubidi_getParaLevel(bidi: *const UBidi) -> u8;
    #[link_name = concat!("ubidi_getLogicalRun_", env!("ICU_MAJOR"))]
    pub fn ubidi_getLogicalRun(bidi: *const UBidi, start: i32, end: *mut i32, level: *mut u8);
    #[link_name = concat!("ubidi_reorderVisual_", env!("ICU_MAJOR"))]
    pub fn ubidi_reorderVisual(levels: *const u8, length: i32, indices: *mut i32);
    #[link_name = concat!("uscript_getScript_", env!("ICU_MAJOR"))]
    pub fn uscript_getScript(ch: i32, status: *mut i32) -> i32;
    #[link_name = concat!("uscript_getScriptExtensions_", env!("ICU_MAJOR"))]
    pub fn uscript_getScriptExtensions(
        ch: i32,
        scripts: *mut i32,
        capacity: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name = concat!("u_getBidiPairedBracket_", env!("ICU_MAJOR"))]
    pub fn u_getBidiPairedBracket(ch: i32) -> i32;
    #[link_name = concat!("u_getIntPropertyValue_", env!("ICU_MAJOR"))]
    pub fn u_getIntPropertyValue(ch: i32, property: i32) -> i32;
    #[link_name = concat!("u_charType_", env!("ICU_MAJOR"))]
    pub fn u_charType(ch: i32) -> i8;
    // cpp: icu_bidi/unicode/uchar.h:3573-3573
    #[link_name = concat!("u_charDirection_", env!("ICU_MAJOR"))]
    pub fn u_charDirection(ch: i32) -> i32;
}
