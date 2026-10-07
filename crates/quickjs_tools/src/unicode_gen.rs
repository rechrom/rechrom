//! unicode_gen.c, official Unicode 17 table generator. Bellard/Gordon MIT.
//! All operating-system IO is confined to this separate tool crate.
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_imports,
    static_mut_refs
)]
use super::unicode_gen_def::*;
use core::{
    ffi::{c_char, c_void},
    mem::size_of,
    ptr,
};
use quickjs::cutils::{dbuf_free, dbuf_init, dbuf_put_u32, dbuf_putc, DynBuf};
use quickjs::cutils_header::{max_int, min_int, FALSE, TRUE};
include!("unicode_gen_support.rs");
#[repr(C)]
#[derive(Clone, Copy)]
struct REString {
    next: *mut REString,
    hash: u32,
    len: u32,
    flags: u32,
    buf: [u32; 0],
}
#[repr(C)]
#[derive(Clone, Copy)]
struct REStringList {
    n_strings: u32,
    hash_size: u32,
    hash_bits: i32,
    hash_table: *mut *mut REString,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct CCInfo {
    u_len: u8,
    l_len: u8,
    f_len: u8,
    u_data: [i32; 3],
    l_data: [i32; 3],
    f_data: [i32; 3],
    combining_class: u8,
    flags: u8,
    general_category: u8,
    script: u8,
    script_ext_len: u8,
    script_ext: *mut u8,
    prop_bitmap_tab: [u32; 3],
    decomp_len: i32,
    decomp_data: *mut i32,
}
impl CCInfo {
    fn is_compat(&self) -> u8 {
        self.flags & 1
    }
    fn set_is_compat(&mut self, v: u8) {
        self.flags = (self.flags & !1) | (v & 1);
    }
    fn is_excluded(&self) -> u8 {
        (self.flags >> 1) & 1
    }
    fn set_is_excluded(&mut self, v: u8) {
        self.flags = (self.flags & !2) | ((v & 1) << 1);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
struct UnicodeSequenceProperties {
    count: i32,
    size: i32,
    tab: *mut i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct TableEntry {
    code: i32,
    len: i32,
    v_type: i32,
    data: i32,
    ext_len: i32,
    ext_data: [i32; 3],
    data_index: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct DecompEntry {
    code: i32,
    len: u8,
    v_type: u8,
    c_len: u8,
    c_min: u16,
    data_index: u16,
    cost: i32,
}
const CHARCODE_MAX: i32 = 0x10ffff;
const CC_LEN_MAX: usize = 3;
const CC_BLOCK_LEN: i32 = 32;
static mut unicode_db: *mut CCInfo = ptr::null_mut();
static mut rgi_emoji_zwj_sequence: REStringList = REStringList {
    n_strings: 0,
    hash_size: 0,
    hash_bits: 0,
    hash_table: ptr::null_mut(),
};
static mut rgi_emoji_tag_sequence: DynBuf = DynBuf {
    buf: ptr::null_mut(),
    size: 0,
    allocated_size: 0,
    error: 0,
    realloc_func: None,
    opaque: ptr::null_mut(),
};
static mut total_tables: u32 = 0;
static mut total_table_bytes: u32 = 0;
static mut total_index: u32 = 0;
static mut total_index_bytes: u32 = 0;

include!("unicode_gen_constants.rs");
include!("unicode_gen_bodies.rs");
include!("unicode_gen_entry.rs");
include!("unicode_gen_dormant.rs");
