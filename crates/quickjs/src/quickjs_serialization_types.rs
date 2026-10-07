// Official quickjs.c:37263..38301 binary-object layouts. MIT.
#[repr(C)]
struct JSObjectListEntry { obj: *mut JSObject, hash_next: u32 }
#[repr(C)]
struct JSObjectList { object_tab: *mut JSObjectListEntry, object_count: i32, object_size: i32, hash_table: *mut u32, hash_size: u32 }
type BCTagEnum = u32;
const BC_TAG_NULL: BCTagEnum = 1;
const BC_TAG_UNDEFINED: BCTagEnum = 2;
const BC_TAG_BOOL_FALSE: BCTagEnum = 3;
const BC_TAG_BOOL_TRUE: BCTagEnum = 4;
const BC_TAG_INT32: BCTagEnum = 5;
const BC_TAG_FLOAT64: BCTagEnum = 6;
const BC_TAG_STRING: BCTagEnum = 7;
const BC_TAG_OBJECT: BCTagEnum = 8;
const BC_TAG_ARRAY: BCTagEnum = 9;
const BC_TAG_BIG_INT: BCTagEnum = 10;
const BC_TAG_TEMPLATE_OBJECT: BCTagEnum = 11;
const BC_TAG_FUNCTION_BYTECODE: BCTagEnum = 12;
const BC_TAG_MODULE: BCTagEnum = 13;
const BC_TAG_TYPED_ARRAY: BCTagEnum = 14;
const BC_TAG_ARRAY_BUFFER: BCTagEnum = 15;
const BC_TAG_SHARED_ARRAY_BUFFER: BCTagEnum = 16;
const BC_TAG_DATE: BCTagEnum = 17;
const BC_TAG_OBJECT_VALUE: BCTagEnum = 18;
const BC_TAG_OBJECT_REFERENCE: BCTagEnum = 19;
const BC_VERSION: u8 = 5;
#[repr(C)]
struct BCWriterState {
    ctx: *mut JSContext,
    dbuf: crate::cutils_header::DynBuf,
    allow_bytecode: i8,
    allow_sab: i8,
    allow_reference: i8,
    first_atom: u32,
    atom_to_idx: *mut u32,
    atom_to_idx_size: i32,
    idx_to_atom: *mut JSAtom,
    idx_to_atom_count: i32,
    idx_to_atom_size: i32,
    sab_tab: *mut *mut u8,
    sab_tab_len: i32,
    sab_tab_size: i32,
    object_list: JSObjectList,
}
#[repr(C)]
struct BCReaderState {
    ctx: *mut JSContext,
    buf_start: *const u8,
    ptr: *const u8,
    buf_end: *const u8,
    first_atom: u32,
    idx_to_atom_count: u32,
    idx_to_atom: *mut JSAtom,
    error_state: i32,
    allow_sab: i8,
    allow_bytecode: i8,
    is_rom_data: i8,
    allow_reference: i8,
    objects: *mut *mut JSObject,
    objects_count: i32,
    objects_size: i32,
    #[cfg(feature="dump-read-object")]
    ptr_last: *const u8,
    #[cfg(feature="dump-read-object")]
    level: i32,
}
// quickjs.c:37434. The original union probes the native byte order.
unsafe fn is_be() -> i32 { cfg!(target_endian = "big") as i32 }

include!("quickjs_serialization_debug.rs");
