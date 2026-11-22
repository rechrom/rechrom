// Official quickjs.c:37263..38301 binary-object layouts. MIT.
use super::*;
#[repr(C)]
pub(crate) struct JSObjectListEntry { pub(crate) obj: *mut JSObject, pub(crate) hash_next: u32 }
#[repr(C)]
pub(crate) struct JSObjectList { pub(crate) object_tab: *mut JSObjectListEntry, pub(crate) object_count: i32, pub(crate) object_size: i32, pub(crate) hash_table: *mut u32, pub(crate) hash_size: u32 }
pub(crate) type BCTagEnum = u32;
pub(crate) const BC_TAG_NULL: BCTagEnum = 1;
pub(crate) const BC_TAG_UNDEFINED: BCTagEnum = 2;
pub(crate) const BC_TAG_BOOL_FALSE: BCTagEnum = 3;
pub(crate) const BC_TAG_BOOL_TRUE: BCTagEnum = 4;
pub(crate) const BC_TAG_INT32: BCTagEnum = 5;
pub(crate) const BC_TAG_FLOAT64: BCTagEnum = 6;
pub(crate) const BC_TAG_STRING: BCTagEnum = 7;
pub(crate) const BC_TAG_OBJECT: BCTagEnum = 8;
pub(crate) const BC_TAG_ARRAY: BCTagEnum = 9;
pub(crate) const BC_TAG_BIG_INT: BCTagEnum = 10;
pub(crate) const BC_TAG_TEMPLATE_OBJECT: BCTagEnum = 11;
pub(crate) const BC_TAG_FUNCTION_BYTECODE: BCTagEnum = 12;
pub(crate) const BC_TAG_MODULE: BCTagEnum = 13;
pub(crate) const BC_TAG_TYPED_ARRAY: BCTagEnum = 14;
pub(crate) const BC_TAG_ARRAY_BUFFER: BCTagEnum = 15;
pub(crate) const BC_TAG_SHARED_ARRAY_BUFFER: BCTagEnum = 16;
pub(crate) const BC_TAG_DATE: BCTagEnum = 17;
pub(crate) const BC_TAG_OBJECT_VALUE: BCTagEnum = 18;
pub(crate) const BC_TAG_OBJECT_REFERENCE: BCTagEnum = 19;
pub(crate) const BC_VERSION: u8 = 5;
#[repr(C)]
pub(crate) struct BCWriterState {
    pub(crate) ctx: *mut JSContext,
    pub(crate) dbuf: crate::cutils_header::DynBuf,
    pub(crate) allow_bytecode: i8,
    pub(crate) allow_sab: i8,
    pub(crate) allow_reference: i8,
    pub(crate) first_atom: u32,
    pub(crate) atom_to_idx: *mut u32,
    pub(crate) atom_to_idx_size: i32,
    pub(crate) idx_to_atom: *mut JSAtom,
    pub(crate) idx_to_atom_count: i32,
    pub(crate) idx_to_atom_size: i32,
    pub(crate) sab_tab: *mut *mut u8,
    pub(crate) sab_tab_len: i32,
    pub(crate) sab_tab_size: i32,
    pub(crate) object_list: JSObjectList,
}
#[repr(C)]
pub(crate) struct BCReaderState {
    pub(crate) ctx: *mut JSContext,
    pub(crate) buf_start: *const u8,
    pub(crate) ptr: *const u8,
    pub(crate) buf_end: *const u8,
    pub(crate) first_atom: u32,
    pub(crate) idx_to_atom_count: u32,
    pub(crate) idx_to_atom: *mut JSAtom,
    pub(crate) error_state: i32,
    pub(crate) allow_sab: i8,
    pub(crate) allow_bytecode: i8,
    pub(crate) is_rom_data: i8,
    pub(crate) allow_reference: i8,
    pub(crate) objects: *mut *mut JSObject,
    pub(crate) objects_count: i32,
    pub(crate) objects_size: i32,
    #[cfg(feature="dump-read-object")]
    pub(crate) ptr_last: *const u8,
    #[cfg(feature="dump-read-object")]
    pub(crate) level: i32,
}
// quickjs.c:37434. The original union probes the native byte order.
pub(crate) unsafe fn is_be() -> i32 { cfg!(target_endian = "big") as i32 }

mod quickjs_serialization_debug;
pub use quickjs_serialization_debug::*;
