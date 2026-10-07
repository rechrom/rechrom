// quickjs.c:43464..43489,43561..43592,43816..43825,44210..44234. MIT.
#[repr(C)]
struct JSIteratorWrapData {
    wrapped_iter: JSValue,
    wrapped_next: JSValue,
}
#[repr(C)]
struct JSIteratorConcatData {
    index: i32,
    count: i32,
    running: i32,
    iter: JSValue,
    next: JSValue,
    values: [JSValue; 0],
}
#[repr(C)]
struct JSIteratorHelperData {
    obj: JSValue,
    next: JSValue,
    func: JSValue,
    inner: JSValue,
    count: i64,
    kind_and_flags: u32,
}
impl JSIteratorHelperData {
    fn kind(&self) -> u32 {
        self.kind_and_flags & 255
    }
    fn set_kind(&mut self, v: u32) {
        self.kind_and_flags = (self.kind_and_flags & !255) | (v & 255);
    }
    fn executing(&self) -> u32 {
        (self.kind_and_flags >> 8) & 1
    }
    fn set_executing(&mut self, v: u32) {
        self.kind_and_flags = (self.kind_and_flags & !(1 << 8)) | ((v & 1) << 8);
    }
    fn done(&self) -> u32 {
        (self.kind_and_flags >> 9) & 1
    }
    fn set_done(&mut self, v: u32) {
        self.kind_and_flags = (self.kind_and_flags & !(1 << 9)) | ((v & 1) << 9);
    }
}
unsafe fn js_iterator_wrap_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .iterator_wrap_data;
    if !it.is_null() {
        JS_FreeValueRT(rt, (*it).wrapped_iter);
        JS_FreeValueRT(rt, (*it).wrapped_next);
        js_free_rt(rt, it.cast());
    }
}
unsafe fn js_iterator_wrap_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .iterator_wrap_data;
    if !it.is_null() {
        JS_MarkValue(rt, (*it).wrapped_iter, mark_func.unwrap());
        JS_MarkValue(rt, (*it).wrapped_next, mark_func.unwrap());
    }
}
unsafe fn js_iterator_concat_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .iterator_concat_data;
    if !it.is_null() {
        JS_FreeValueRT(rt, (*it).iter);
        JS_FreeValueRT(rt, (*it).next);
        for i in (*it).index..(*it).count {
            JS_FreeValueRT(
                rt,
                *ptr::addr_of!((*it).values)
                    .cast::<JSValue>()
                    .add(i as usize),
            );
        }
        js_free_rt(rt, it.cast());
    }
}
unsafe fn js_iterator_concat_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .iterator_concat_data;
    if !it.is_null() {
        JS_MarkValue(rt, (*it).iter, mark_func.unwrap());
        JS_MarkValue(rt, (*it).next, mark_func.unwrap());
        for i in (*it).index..(*it).count {
            JS_MarkValue(
                rt,
                *ptr::addr_of!((*it).values)
                    .cast::<JSValue>()
                    .add(i as usize),
                mark_func.unwrap(),
            );
        }
    }
}
unsafe fn js_iterator_helper_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .iterator_helper_data;
    if !it.is_null() {
        JS_FreeValueRT(rt, (*it).obj);
        JS_FreeValueRT(rt, (*it).func);
        JS_FreeValueRT(rt, (*it).next);
        JS_FreeValueRT(rt, (*it).inner);
        js_free_rt(rt, it.cast());
    }
}
unsafe fn js_iterator_helper_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let it = (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
        .u
        .iterator_helper_data;
    if !it.is_null() {
        JS_MarkValue(rt, (*it).obj, mark_func.unwrap());
        JS_MarkValue(rt, (*it).func, mark_func.unwrap());
        JS_MarkValue(rt, (*it).next, mark_func.unwrap());
        JS_MarkValue(rt, (*it).inner, mark_func.unwrap());
    }
}
