// quickjs.c Array/Iterator local records and enum values. MIT.
#[repr(C)]
struct ValueSlot { val: JSValue, str: *mut JSString, pos: i64 }
#[repr(C)]
struct array_sort_context { ctx: *mut JSContext, exception: i32, has_method: i32, method: JSValueConst }
type JSIteratorHelperKindEnum = u32;
const JS_ITERATOR_HELPER_KIND_DROP: u32 = 0;
const JS_ITERATOR_HELPER_KIND_EVERY: u32 = 1;
const JS_ITERATOR_HELPER_KIND_FILTER: u32 = 2;
const JS_ITERATOR_HELPER_KIND_FIND: u32 = 3;
const JS_ITERATOR_HELPER_KIND_FLAT_MAP: u32 = 4;
const JS_ITERATOR_HELPER_KIND_FOR_EACH: u32 = 5;
const JS_ITERATOR_HELPER_KIND_MAP: u32 = 6;
const JS_ITERATOR_HELPER_KIND_SOME: u32 = 7;
const JS_ITERATOR_HELPER_KIND_TAKE: u32 = 8;
const ArrayFind: i32 = 0;
const ArrayFindIndex: i32 = 1;
const ArrayFindLast: i32 = 2;
const ArrayFindLastIndex: i32 = 3;
const special_every: i32 = 0;
const special_some: i32 = 1;
const special_forEach: i32 = 2;
const special_map: i32 = 3;
const special_filter: i32 = 4;
const special_TA: i32 = 8;
const special_reduce: i32 = 0;
const special_reduceRight: i32 = 1;
const special_indexOf: i32 = 0;
const special_lastIndexOf: i32 = 1;
