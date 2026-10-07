// VM support for pure Rust temporary storage and the original operand enums. MIT.
use crate::cutils_header::{get_i16, get_i8, min_int, max_int, max_uint32, clz32};
use crate::quickjs_atom::{JS_ATOM_name, JS_ATOM_value, JS_ATOM_done, JS_ATOM_prototype, JS_ATOM_constructor, JS_ATOM_function, JS_ATOM_undefined, JS_ATOM_length, JS_ATOM_throw, JS_ATOM_Symbol_iterator, JS_ATOM_Symbol_asyncIterator, JS_ATOM_Symbol_unscopables, JS_ATOM_next, JS_ATOM_unknown, JS_ATOM_symbol, JS_ATOM_object, JS_ATOM_string, JS_ATOM_boolean, JS_ATOM_number, JS_ATOM_bigint};
use crate::quickjs_opcode::*;
const OP_SPECIAL_OBJECT_ARGUMENTS: i32 = 0;
const OP_SPECIAL_OBJECT_MAPPED_ARGUMENTS: i32 = 1;
const OP_SPECIAL_OBJECT_THIS_FUNC: i32 = 2;
const OP_SPECIAL_OBJECT_NEW_TARGET: i32 = 3;
const OP_SPECIAL_OBJECT_HOME_OBJECT: i32 = 4;
const OP_SPECIAL_OBJECT_VAR_OBJECT: i32 = 5;
const OP_SPECIAL_OBJECT_IMPORT_META: i32 = 6;
const FUNC_RET_AWAIT: i32 = 0;
const FUNC_RET_YIELD: i32 = 1;
const FUNC_RET_YIELD_STAR: i32 = 2;
const FUNC_RET_INITIAL_YIELD: i32 = 3;
// Rust owns the C alloca equivalent until the call frame returns. It uses
// the Rust allocator, so the JS runtime allocator trace remains unchanged.
// Alignment and contiguous value/var-ref layout match the original C frame.
unsafe fn vm_temp_allocate(
    storage: &mut Vec<u64>,
    size: usize,
    ctx: *mut JSContext,
) -> Option<*mut c_void> {
    let words = size.div_ceil(size_of::<u64>());
    if storage.try_reserve_exact(words).is_err() {
        JS_ThrowOutOfMemory(ctx);
        return None;
    }
    storage.resize(words, 0);
    Some(storage.as_mut_ptr().cast())
}
// Recycle only the raw C-alloca-equivalent call-frame allocation. JSValue
// ownership and cleanup stay in JS_CallInternal; this pool never traces or
// retains a JS object. Each frame exclusively owns its buffer through reentry.
const VM_CALL_FRAME_CACHE_WORDS: usize = 16 * 1024 / size_of::<u64>();
std::thread_local! {
    static VM_CALL_FRAME_CACHE: std::cell::RefCell<[Option<Vec<u64>>; 8]> =
        std::cell::RefCell::new(std::array::from_fn(|_| None));
}
struct VmCallFrameStorage {
    words: Vec<u64>,
}
impl VmCallFrameStorage {
    fn new() -> Self { Self { words: Vec::new() } }
    unsafe fn allocate(&mut self, size: usize, ctx: *mut JSContext) -> Option<*mut c_void> {
        // Native/non-bytecode calls never touch the pool. Large frames retain
        // the original allocation path and cannot displace these small frames.
        let words = size.div_ceil(size_of::<u64>());
        if self.words.capacity() == 0 && words != 0 && words <= VM_CALL_FRAME_CACHE_WORDS {
            let _ = VM_CALL_FRAME_CACHE.try_with(|pool| {
                if let Ok(mut slots) = pool.try_borrow_mut() {
                    let slot = slots.iter().position(|slot|
                        slot.as_ref().is_some_and(|buffer| buffer.capacity() >= words))
                        .or_else(|| slots.iter().position(Option::is_some));
                    if let Some(slot) = slot { self.words = slots[slot].take().unwrap(); }
                }
            });
        }
        // Returned buffers have length zero, so the original resize still
        // initializes every word, including locals, operand stack and varrefs.
        vm_temp_allocate(&mut self.words, size, ctx)
    }
}
impl Drop for VmCallFrameStorage {
    fn drop(&mut self) {
        if self.words.capacity() == 0 || self.words.capacity() > VM_CALL_FRAME_CACHE_WORDS {
            return;
        }
        self.words.clear();
        let _ = VM_CALL_FRAME_CACHE.try_with(|pool| {
            if let Ok(mut slots) = pool.try_borrow_mut() {
                if let Some(slot) = slots.iter_mut().find(|slot| slot.is_none()) {
                    *slot = Some(std::mem::take(&mut self.words));
                }
            }
        });
    }
}
// c: quickjs.c:16999
unsafe fn JS_GetActiveFunction(ctx: *mut JSContext) -> JSValueConst {
    (*(*(*ctx).rt).current_stack_frame).cur_func
}
// c: quickjs.c:50373
unsafe fn JS_ThrowTypeErrorRevokedProxy(ctx: *mut JSContext) -> JSValue {
    JS_ThrowTypeError(ctx, c"revoked proxy".as_ptr())
}
// C macro pointer accessor used by the manually translated call helpers.
#[inline]
unsafe fn JS_VALUE_GET_OBJ(val: JSValueConst) -> *mut JSObject {
    JS_VALUE_GET_PTR(val).cast()
}
// c: quickjs.c:14568
unsafe fn js_pow(a: f64, b: f64) -> f64 {
    if !b.is_finite() && a.abs() == 1.0 { JS_FLOAT64_NAN } else { a.powf(b) }
}
// c: quickjs.c:15602. The equality algorithm groups all numeric tags.
fn tag_is_number(tag: u32) -> JS_BOOL {
    (tag == JS_TAG_INT as u32 || tag == JS_TAG_FLOAT64 as u32
        || tag == JS_TAG_BIG_INT as u32 || tag == JS_TAG_SHORT_BIG_INT as u32) as JS_BOOL
}
