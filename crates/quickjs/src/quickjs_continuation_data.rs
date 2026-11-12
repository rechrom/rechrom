// Opaque embedder continuation data. This is a VM slot, not scheduler state.
// Matches V8 CPED's promise-reaction registration and job-entry restoration.
pub type JSContinuationData = Option<std::rc::Rc<dyn std::any::Any>>;
pub type JSContinuationSlot = std::rc::Rc<std::cell::RefCell<JSContinuationData>>;
pub unsafe fn JS_SetContinuationPreservedDataSlot(ctx: *mut JSContext, slot: JSContinuationSlot) {
    let target = &mut (*(*ctx).rt).continuation_data_slot;
    if !target.is_null() {
        drop(std::rc::Rc::from_raw(*target));
    }
    *target = std::rc::Rc::into_raw(slot);
}
pub unsafe fn JS_GetContinuationPreservedData(ctx: *mut JSContext) -> JSContinuationData {
    let slot = (*(*ctx).rt).continuation_data_slot;
    if slot.is_null() {
        None
    } else {
        (*slot).borrow().clone()
    }
}
pub unsafe fn JS_SetContinuationPreservedData(ctx: *mut JSContext, value: JSContinuationData) {
    let slot = (*(*ctx).rt).continuation_data_slot;
    if !slot.is_null() {
        *(*slot).borrow_mut() = value;
    }
}
pub unsafe fn JS_IsInContext(ctx: *mut JSContext) -> bool {
    !(*(*ctx).rt).current_stack_frame.is_null()
}
unsafe fn js_capture_continuation_data(ctx: *mut JSContext) -> *mut JSContinuationData {
    let data = JS_GetContinuationPreservedData(ctx);
    if data.is_some() {
        Box::into_raw(Box::new(data))
    } else {
        ptr::null_mut()
    }
}
unsafe fn js_free_continuation_data(data: *mut JSContinuationData) {
    if !data.is_null() {
        drop(Box::from_raw(data));
    }
}
unsafe fn js_continuation_value(data: *const JSContinuationData) -> JSContinuationData {
    if data.is_null() {
        None
    } else {
        (*data).clone()
    }
}
struct JSContinuationScope {
    ctx: *mut JSContext,
    previous: JSContinuationData,
}
impl JSContinuationScope {
    unsafe fn Enter(ctx: *mut JSContext, value: JSContinuationData) -> Self {
        let previous = JS_GetContinuationPreservedData(ctx);
        JS_SetContinuationPreservedData(ctx, value);
        Self { ctx, previous }
    }
}
impl Drop for JSContinuationScope {
    fn drop(&mut self) {
        unsafe {
            JS_SetContinuationPreservedData(self.ctx, self.previous.take());
        }
    }
}
unsafe fn JS_EnqueueJobWithContinuation(
    ctx: *mut JSContext,
    job: Option<JSJobFunc>,
    argc: i32,
    argv: *mut JSValueConst,
    data: *const JSContinuationData,
) -> i32 {
    let _scope = JSContinuationScope::Enter(ctx, js_continuation_value(data));
    JS_EnqueueJob(ctx, job, argc, argv)
}
