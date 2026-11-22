// quickjs.c Array/Iterator local records and enum values. MIT.
use super::*;
#[repr(C)]
pub(crate) struct ValueSlot { pub(crate) val: JSValue, pub(crate) str: *mut JSString, pub(crate) pos: i64 }
#[repr(C)]
pub(crate) struct array_sort_context { pub(crate) ctx: *mut JSContext, pub(crate) exception: i32, pub(crate) has_method: i32, pub(crate) method: JSValueConst }
pub(crate) type JSIteratorHelperKindEnum = u32;
pub(crate) const JS_ITERATOR_HELPER_KIND_DROP: u32 = 0;
pub(crate) const JS_ITERATOR_HELPER_KIND_EVERY: u32 = 1;
pub(crate) const JS_ITERATOR_HELPER_KIND_FILTER: u32 = 2;
pub(crate) const JS_ITERATOR_HELPER_KIND_FIND: u32 = 3;
pub(crate) const JS_ITERATOR_HELPER_KIND_FLAT_MAP: u32 = 4;
pub(crate) const JS_ITERATOR_HELPER_KIND_FOR_EACH: u32 = 5;
pub(crate) const JS_ITERATOR_HELPER_KIND_MAP: u32 = 6;
pub(crate) const JS_ITERATOR_HELPER_KIND_SOME: u32 = 7;
pub(crate) const JS_ITERATOR_HELPER_KIND_TAKE: u32 = 8;
pub(crate) const ArrayFind: i32 = 0;
pub(crate) const ArrayFindIndex: i32 = 1;
pub(crate) const ArrayFindLast: i32 = 2;
pub(crate) const ArrayFindLastIndex: i32 = 3;
pub(crate) const special_every: i32 = 0;
pub(crate) const special_some: i32 = 1;
pub(crate) const special_forEach: i32 = 2;
pub(crate) const special_map: i32 = 3;
pub(crate) const special_filter: i32 = 4;
pub(crate) const special_TA: i32 = 8;
pub(crate) const special_reduce: i32 = 0;
pub(crate) const special_reduceRight: i32 = 1;
pub(crate) const special_indexOf: i32 = 0;
pub(crate) const special_lastIndexOf: i32 = 1;
