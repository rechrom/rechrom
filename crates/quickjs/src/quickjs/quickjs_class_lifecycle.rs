// quickjs.c class payload layouts, destruction and marking. MIT.
use super::*;
pub(crate) unsafe fn js_mapped_arguments_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let var_refs = (*p).u.array.u.var_refs;
    let mut i = 0;
    while i < (*p).u.array.count {
        free_var_ref(rt, *var_refs.add(i as usize));
        i += 1;
    }
    js_free_rt(rt, var_refs.cast());
}
pub(crate) unsafe fn js_mapped_arguments_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let var_refs = (*p).u.array.u.var_refs;
    let mut i = 0;
    while i < (*p).u.array.count {
        mark_func.unwrap()(rt, &mut (**var_refs.add(i as usize)).header);
        i += 1;
    }
}
pub(crate) type JSGeneratorStateEnum = u32;
pub(crate) const JS_GENERATOR_STATE_SUSPENDED_START: JSGeneratorStateEnum = 0;
pub(crate) const JS_GENERATOR_STATE_SUSPENDED_YIELD: JSGeneratorStateEnum = 1;
pub(crate) const JS_GENERATOR_STATE_SUSPENDED_YIELD_STAR: JSGeneratorStateEnum = 2;
pub(crate) const JS_GENERATOR_STATE_EXECUTING: JSGeneratorStateEnum = 3;
pub(crate) const JS_GENERATOR_STATE_COMPLETED: JSGeneratorStateEnum = 4;
#[repr(C)]
pub(crate) struct JSGeneratorData {
    pub(crate) state: JSGeneratorStateEnum,
    pub(crate) func_state: *mut JSAsyncFunctionState,
}
pub(crate) unsafe fn free_generator_stack_rt(rt: *mut JSRuntime, s: *mut JSGeneratorData) {
    if (*s).state == JS_GENERATOR_STATE_COMPLETED {
        return;
    }
    if !(*s).func_state.is_null() {
        async_func_free(rt, (*s).func_state);
        (*s).func_state = ptr::null_mut();
    }
    (*s).state = JS_GENERATOR_STATE_COMPLETED;
}
pub(crate) unsafe fn js_generator_finalizer(rt: *mut JSRuntime, obj: JSValue) {
    let s = JS_GetOpaque(obj, JS_CLASS_GENERATOR).cast::<JSGeneratorData>();
    if !s.is_null() {
        free_generator_stack_rt(rt, s);
        js_free_rt(rt, s.cast());
    }
}
pub(crate) unsafe fn free_generator_stack(ctx: *mut JSContext, s: *mut JSGeneratorData) {
    free_generator_stack_rt((*ctx).rt, s);
}
pub(crate) unsafe fn js_generator_mark(rt: *mut JSRuntime, val: JSValueConst, mark_func: Option<JS_MarkFunc>) {
    let s = (*JS_VALUE_GET_PTR(val).cast::<JSObject>()).u.generator_data;
    if s.is_null() || (*s).func_state.is_null() {
        return;
    }
    mark_func.unwrap()(rt, &mut (*(*s).func_state).header);
}
#[repr(C)]
pub(crate) struct JSArrayIteratorData {
    pub(crate) obj: JSValue,
    pub(crate) kind: JSIteratorKindEnum,
    pub(crate) idx: u32,
}
pub(crate) unsafe fn js_array_iterator_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .array_iterator_data;
    if !it.is_null() {
        JS_FreeValueRT(rt, (*it).obj);
        js_free_rt(rt, it.cast());
    }
}
pub(crate) unsafe fn js_array_iterator_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .array_iterator_data;
    if !it.is_null() {
        JS_MarkValue(rt, (*it).obj, mark_func.unwrap());
    }
}
pub(crate) unsafe fn js_regexp_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let re = ptr::addr_of_mut!((*JS_VALUE_GET_PTR(val).cast::<JSObject>()).u.regexp);
    if !(*re).bytecode.is_null() {
        JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_STRING, (*re).bytecode.cast()));
    }
    if !(*re).pattern.is_null() {
        JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_STRING, (*re).pattern.cast()));
    }
}
#[repr(C)]
pub(crate) struct JSRegExpStringIteratorData {
    pub(crate) iterating_regexp: JSValue,
    pub(crate) iterated_string: JSValue,
    pub(crate) global: JS_BOOL,
    pub(crate) unicode: JS_BOOL,
    pub(crate) done: JS_BOOL,
}
pub(crate) unsafe fn js_regexp_string_iterator_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .regexp_string_iterator_data;
    if !it.is_null() {
        JS_FreeValueRT(rt, (*it).iterating_regexp);
        JS_FreeValueRT(rt, (*it).iterated_string);
        js_free_rt(rt, it.cast());
    }
}
pub(crate) unsafe fn js_regexp_string_iterator_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .regexp_string_iterator_data;
    if !it.is_null() {
        JS_MarkValue(rt, (*it).iterating_regexp, mark_func.unwrap());
        JS_MarkValue(rt, (*it).iterated_string, mark_func.unwrap());
    }
}
pub(crate) unsafe fn js_proxy_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let s = JS_GetOpaque(val, JS_CLASS_PROXY).cast::<JSProxyData>();
    if !s.is_null() {
        JS_FreeValueRT(rt, (*s).target);
        JS_FreeValueRT(rt, (*s).handler);
        js_free_rt(rt, s.cast());
    }
}
pub(crate) unsafe fn js_proxy_mark(rt: *mut JSRuntime, val: JSValueConst, mark_func: Option<JS_MarkFunc>) {
    let s = JS_GetOpaque(val, JS_CLASS_PROXY).cast::<JSProxyData>();
    if !s.is_null() {
        JS_MarkValue(rt, (*s).target, mark_func.unwrap());
        JS_MarkValue(rt, (*s).handler, mark_func.unwrap());
    }
}
#[repr(C)]
pub(crate) struct JSMapIteratorData {
    pub(crate) obj: JSValue,
    pub(crate) kind: JSIteratorKindEnum,
    pub(crate) cur_record: *mut JSMapRecord,
}
pub(crate) unsafe fn js_map_iterator_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .map_iterator_data;
    if !it.is_null() {
        if JS_IsLiveObject(rt, (*it).obj) != 0 && !(*it).cur_record.is_null() {
            map_decref_record(rt, (*it).cur_record);
        }
        JS_FreeValueRT(rt, (*it).obj);
        js_free_rt(rt, it.cast());
    }
}
pub(crate) unsafe fn js_map_iterator_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .map_iterator_data;
    if !it.is_null() {
        JS_MarkValue(rt, (*it).obj, mark_func.unwrap());
    }
}
pub(crate) unsafe fn js_array_buffer_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let abuf = (*JS_VALUE_GET_PTR(val).cast::<JSObject>()).u.array_buffer;
    if !abuf.is_null() {
        for el in ListIter::new(&mut (*abuf).array_list, false, true) {
            let ta = el
                .cast::<u8>()
                .sub(offset_of!(JSTypedArray, link))
                .cast::<JSTypedArray>();
            (*ta).link.prev = ptr::null_mut();
            (*ta).link.next = ptr::null_mut();
            let p1 = (*ta).obj;
            if (*p1).class_id as u32 != JS_CLASS_DATAVIEW {
                (*p1).u.array.count = 0;
                (*p1).u.array.u.ptr = ptr::null_mut();
            }
        }
        if (*abuf).shared != 0 && (*rt).sab_funcs.sab_free.is_some() {
            (*rt).sab_funcs.sab_free.unwrap()((*rt).sab_funcs.sab_opaque, (*abuf).data.cast());
        } else if let Some(free_func) = (*abuf).free_func {
            free_func(rt, (*abuf).opaque, (*abuf).data.cast());
        }
        js_free_rt(rt, abuf.cast());
    }
}
pub(crate) unsafe fn js_typed_array_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let ta = (*JS_VALUE_GET_PTR(val).cast::<JSObject>()).u.typed_array;
    if !ta.is_null() {
        if !(*ta).link.next.is_null() {
            list_del(&mut (*ta).link);
        }
        JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, (*ta).buffer.cast()));
        js_free_rt(rt, ta.cast());
    }
}
pub(crate) unsafe fn js_typed_array_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let ta = (*JS_VALUE_GET_PTR(val).cast::<JSObject>()).u.typed_array;
    if !ta.is_null() {
        JS_MarkValue(
            rt,
            JS_MKPTR(JS_TAG_OBJECT, (*ta).buffer.cast()),
            mark_func.unwrap(),
        );
    }
}
// Promise and asynchronous object storage sections of quickjs.c.
#[repr(C)]
pub(crate) struct JSPromiseData {
    pub(crate) promise_state: JSPromiseStateEnum,
    pub(crate) promise_reactions: [list_head; 2],
    pub(crate) is_handled: JS_BOOL,
    pub(crate) promise_result: JSValue,
}
#[repr(C)]
pub(crate) struct JSPromiseFunctionDataResolved {
    pub(crate) ref_count: i32,
    pub(crate) already_resolved: JS_BOOL,
}
#[repr(C)]
pub(crate) struct JSPromiseFunctionData {
    pub(crate) promise: JSValue,
    pub(crate) presolved: *mut JSPromiseFunctionDataResolved,
}
#[repr(C)]
pub(crate) struct JSPromiseReactionData {
    pub(crate) continuation_data: *mut JSContinuationData,
    pub(crate) link: list_head,
    pub(crate) resolving_funcs: [JSValue; 2],
    pub(crate) handler: JSValue,
}
pub unsafe fn JS_PromiseState(_ctx: *mut JSContext, promise: JSValue) -> JSPromiseStateEnum {
    let s = JS_GetOpaque(promise, JS_CLASS_PROMISE).cast::<JSPromiseData>();
    if s.is_null() {
        -1
    } else {
        (*s).promise_state
    }
}
pub unsafe fn JS_PromiseResult(ctx: *mut JSContext, promise: JSValue) -> JSValue {
    let s = JS_GetOpaque(promise, JS_CLASS_PROMISE).cast::<JSPromiseData>();
    if s.is_null() {
        JS_UNDEFINED
    } else {
        JS_DupValue(ctx, (*s).promise_result)
    }
}
pub(crate) unsafe fn promise_reaction_data_free(rt: *mut JSRuntime, rd: *mut JSPromiseReactionData) {
    js_free_continuation_data((*rd).continuation_data);
    JS_FreeValueRT(rt, (*rd).resolving_funcs[0]);
    JS_FreeValueRT(rt, (*rd).resolving_funcs[1]);
    JS_FreeValueRT(rt, (*rd).handler);
    js_free_rt(rt, rd.cast());
}
pub unsafe fn JS_SetHostPromiseRejectionTracker(
    rt: *mut JSRuntime,
    cb: Option<JSHostPromiseRejectionTracker>,
    opaque: *mut c_void,
) {
    (*rt).host_promise_rejection_tracker = cb;
    (*rt).host_promise_rejection_tracker_opaque = opaque;
}
pub(crate) unsafe fn js_promise_resolve_function_free_resolved(
    rt: *mut JSRuntime,
    sr: *mut JSPromiseFunctionDataResolved,
) {
    (*sr).ref_count -= 1;
    if (*sr).ref_count == 0 {
        js_free_rt(rt, sr.cast());
    }
}
pub(crate) unsafe fn js_promise_resolve_function_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let s = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .promise_function_data;
    if !s.is_null() {
        js_promise_resolve_function_free_resolved(rt, (*s).presolved);
        JS_FreeValueRT(rt, (*s).promise);
        js_free_rt(rt, s.cast());
    }
}
pub(crate) unsafe fn js_promise_resolve_function_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let s = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .promise_function_data;
    if !s.is_null() {
        JS_MarkValue(rt, (*s).promise, mark_func.unwrap());
    }
}
pub(crate) unsafe fn js_promise_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let s = JS_GetOpaque(val, JS_CLASS_PROMISE).cast::<JSPromiseData>();
    if s.is_null() {
        return;
    }
    for i in 0..2 {
        for el in ListIter::new(&mut (*s).promise_reactions[i], false, true) {
            let rd = el
                .cast::<u8>()
                .sub(offset_of!(JSPromiseReactionData, link))
                .cast();
            promise_reaction_data_free(rt, rd);
        }
    }
    JS_FreeValueRT(rt, (*s).promise_result);
    js_free_rt(rt, s.cast());
}
pub(crate) unsafe fn js_promise_mark(rt: *mut JSRuntime, val: JSValueConst, mark_func: Option<JS_MarkFunc>) {
    let s = JS_GetOpaque(val, JS_CLASS_PROMISE).cast::<JSPromiseData>();
    if s.is_null() {
        return;
    }
    for i in 0..2 {
        for el in ListIter::new(&mut (*s).promise_reactions[i], false, false) {
            let rd = el
                .cast::<u8>()
                .sub(offset_of!(JSPromiseReactionData, link))
                .cast::<JSPromiseReactionData>();
            JS_MarkValue(rt, (*rd).resolving_funcs[0], mark_func.unwrap());
            JS_MarkValue(rt, (*rd).resolving_funcs[1], mark_func.unwrap());
            JS_MarkValue(rt, (*rd).handler, mark_func.unwrap());
        }
    }
    JS_MarkValue(rt, (*s).promise_result, mark_func.unwrap());
}
pub(crate) unsafe fn js_async_function_resolve_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let s = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .async_function_data;
    if !s.is_null() {
        async_func_free(rt, s);
    }
}
pub(crate) unsafe fn js_async_function_resolve_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let s = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .async_function_data;
    if !s.is_null() {
        mark_func.unwrap()(rt, &mut (*s).header);
    }
}
pub(crate) type JSAsyncGeneratorStateEnum = u32;
pub(crate) const JS_ASYNC_GENERATOR_STATE_SUSPENDED_START: JSAsyncGeneratorStateEnum = 0;
pub(crate) const JS_ASYNC_GENERATOR_STATE_SUSPENDED_YIELD: JSAsyncGeneratorStateEnum = 1;
pub(crate) const JS_ASYNC_GENERATOR_STATE_SUSPENDED_YIELD_STAR: JSAsyncGeneratorStateEnum = 2;
pub(crate) const JS_ASYNC_GENERATOR_STATE_EXECUTING: JSAsyncGeneratorStateEnum = 3;
pub(crate) const JS_ASYNC_GENERATOR_STATE_AWAITING_RETURN: JSAsyncGeneratorStateEnum = 4;
pub(crate) const JS_ASYNC_GENERATOR_STATE_COMPLETED: JSAsyncGeneratorStateEnum = 5;
#[repr(C)]
pub(crate) struct JSAsyncGeneratorRequest {
    pub(crate) link: list_head,
    pub(crate) completion_type: i32,
    pub(crate) result: JSValue,
    pub(crate) promise: JSValue,
    pub(crate) resolving_funcs: [JSValue; 2],
}
#[repr(C)]
pub(crate) struct JSAsyncGeneratorData {
    pub(crate) generator: *mut JSObject,
    pub(crate) state: JSAsyncGeneratorStateEnum,
    pub(crate) func_state: *mut JSAsyncFunctionState,
    pub(crate) queue: list_head,
}
pub(crate) unsafe fn js_async_generator_free(rt: *mut JSRuntime, s: *mut JSAsyncGeneratorData) {
    for el in ListIter::new(&mut (*s).queue, false, true) {
        let req = el
            .cast::<u8>()
            .sub(offset_of!(JSAsyncGeneratorRequest, link))
            .cast::<JSAsyncGeneratorRequest>();
        JS_FreeValueRT(rt, (*req).result);
        JS_FreeValueRT(rt, (*req).promise);
        JS_FreeValueRT(rt, (*req).resolving_funcs[0]);
        JS_FreeValueRT(rt, (*req).resolving_funcs[1]);
        js_free_rt(rt, req.cast());
    }
    if !(*s).func_state.is_null() {
        async_func_free(rt, (*s).func_state);
    }
    js_free_rt(rt, s.cast());
}
pub(crate) unsafe fn js_async_generator_finalizer(rt: *mut JSRuntime, obj: JSValue) {
    let s = JS_GetOpaque(obj, JS_CLASS_ASYNC_GENERATOR).cast::<JSAsyncGeneratorData>();
    if !s.is_null() {
        js_async_generator_free(rt, s);
    }
}
pub(crate) unsafe fn js_async_generator_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let s = JS_GetOpaque(val, JS_CLASS_ASYNC_GENERATOR).cast::<JSAsyncGeneratorData>();
    if !s.is_null() {
        for el in ListIter::new(&mut (*s).queue, false, false) {
            let req = el
                .cast::<u8>()
                .sub(offset_of!(JSAsyncGeneratorRequest, link))
                .cast::<JSAsyncGeneratorRequest>();
            JS_MarkValue(rt, (*req).result, mark_func.unwrap());
            JS_MarkValue(rt, (*req).promise, mark_func.unwrap());
            JS_MarkValue(rt, (*req).resolving_funcs[0], mark_func.unwrap());
            JS_MarkValue(rt, (*req).resolving_funcs[1], mark_func.unwrap());
        }
        if !(*s).func_state.is_null() {
            mark_func.unwrap()(rt, &mut (*(*s).func_state).header);
        }
    }
}
#[repr(C)]
pub(crate) struct JSAsyncFromSyncIteratorData {
    pub(crate) sync_iter: JSValue,
    pub(crate) next_method: JSValue,
}
pub(crate) unsafe fn js_async_from_sync_iterator_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let s =
        JS_GetOpaque(val, JS_CLASS_ASYNC_FROM_SYNC_ITERATOR).cast::<JSAsyncFromSyncIteratorData>();
    if !s.is_null() {
        JS_FreeValueRT(rt, (*s).sync_iter);
        JS_FreeValueRT(rt, (*s).next_method);
        js_free_rt(rt, s.cast());
    }
}
pub(crate) unsafe fn js_async_from_sync_iterator_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let s =
        JS_GetOpaque(val, JS_CLASS_ASYNC_FROM_SYNC_ITERATOR).cast::<JSAsyncFromSyncIteratorData>();
    if !s.is_null() {
        JS_MarkValue(rt, (*s).sync_iter, mark_func.unwrap());
        JS_MarkValue(rt, (*s).next_method, mark_func.unwrap());
    }
}
