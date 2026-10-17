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
    storage: &mut Vec<core::mem::MaybeUninit<u64>>,
    size: usize,
    ctx: *mut JSContext,
) -> Option<*mut c_void> {
    let words = size.div_ceil(size_of::<u64>());
    if storage.try_reserve_exact(words).is_err() {
        JS_ThrowOutOfMemory(ctx);
        return None;
    }
    // The C operand stack is uninitialized alloca storage. JS_CallInternal
    // initializes copied arguments, locals and varrefs before publishing the
    // frame, and writes each operand before making it live. MaybeUninit keeps
    // the raw allocation valid without clearing unused operand-stack capacity.
    storage.set_len(words);
    Some(storage.as_mut_ptr().cast())
}
// C obtains every bytecode call frame from the native stack with `alloca`.
// A Vec-per-call translation makes hot JavaScript pay for TLS lookup, pool
// search and Vec ownership transfer at every nesting level, and starts
// allocating again as soon as the small cache is deeper than eight calls.
//
// Retain the same LIFO lifetime in one stable, thread-local backing store.
// Page owns its JavaScript runtime on one executor, calls nest on that owner
// thread, and the fixed Box never moves while frame pointers are live. A frame
// that exceeds the bounded arena keeps the existing fallible heap path.
const VM_CALL_FRAME_ARENA_WORDS: usize = 1024 * 1024 / size_of::<u64>();
struct VmCallFrameArena {
    words: Box<[core::mem::MaybeUninit<u64>]>,
    used: usize,
}
impl VmCallFrameArena {
    fn new() -> Self {
        Self {
            words: vec![core::mem::MaybeUninit::uninit(); VM_CALL_FRAME_ARENA_WORDS].into_boxed_slice(),
            used: 0,
        }
    }
}
std::thread_local! {
    static VM_CALL_FRAME_ARENA: std::cell::RefCell<VmCallFrameArena> =
        std::cell::RefCell::new(VmCallFrameArena::new());
}
struct VmCallFrameStorage {
    words: Vec<core::mem::MaybeUninit<u64>>,
    arena_range: Option<(usize, usize)>,
}
impl VmCallFrameStorage {
    fn new() -> Self { Self { words: Vec::new(), arena_range: None } }
    unsafe fn allocate(&mut self, size: usize, ctx: *mut JSContext) -> Option<*mut c_void> {
        let words = size.div_ceil(size_of::<u64>());
        if self.arena_range.is_none() && self.words.capacity() == 0 && words != 0 {
            let allocated = VM_CALL_FRAME_ARENA.try_with(|arena| {
                let mut arena = arena.try_borrow_mut().ok()?;
                let start = arena.used;
                let end = start.checked_add(words)?;
                if end > arena.words.len() {
                    return None;
                }
                let pointer = arena.words.as_mut_ptr().add(start).cast();
                arena.used = end;
                Some((pointer, start, end))
            });
            if let Ok(Some((pointer, start, end))) = allocated {
                self.arena_range = Some((start, end));
                return Some(pointer);
            }
        }
        // Arena exhaustion preserves the former fallible allocation behavior.
        vm_temp_allocate(&mut self.words, size, ctx)
    }
}
impl Drop for VmCallFrameStorage {
    fn drop(&mut self) {
        let Some((start, end)) = self.arena_range.take() else {
            return;
        };
        let _ = VM_CALL_FRAME_ARENA.try_with(|arena| {
            if let Ok(mut arena) = arena.try_borrow_mut() {
                debug_assert_eq!(arena.used, end, "QuickJS call frames must unwind LIFO");
                arena.used = start;
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
