// quickjs.c class registration, C lines 3810..3925, GC helpers 6540..6570. MIT.
include!("quickjs_classes_constants.rs");
static mut js_class_id_alloc: JSClassID = JS_CLASS_INIT_COUNT;
#[cfg(feature = "atomics")]
static js_class_id_mutex: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub unsafe fn JS_NewClassID(pclass_id: *mut JSClassID) -> JSClassID {
    #[cfg(feature = "atomics")]
    let _lock = js_class_id_mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut class_id = *pclass_id;
    if class_id == 0 {
        class_id = js_class_id_alloc;
        js_class_id_alloc = js_class_id_alloc.wrapping_add(1);
        *pclass_id = class_id;
    }
    class_id
}
pub unsafe fn JS_IsRegisteredClass(rt: *mut JSRuntime, class_id: JSClassID) -> JS_BOOL {
    (class_id < (*rt).class_count as u32
        && (*(*rt).class_array.add(class_id as usize)).class_id != 0) as i32
}
unsafe fn JS_NewClass1(
    rt: *mut JSRuntime,
    class_id: JSClassID,
    class_def: *const JSClassDef,
    name: JSAtom,
) -> i32 {
    if class_id >= 1 << 16 {
        return -1;
    }
    if class_id < (*rt).class_count as u32
        && (*(*rt).class_array.add(class_id as usize)).class_id != 0
    {
        return -1;
    }
    if class_id >= (*rt).class_count as u32 {
        let new_size =
            (JS_CLASS_INIT_COUNT as i32).max((class_id as i32 + 1).max((*rt).class_count * 3 / 2));
        for el in ListIter::new(&mut (*rt).context_list, false, false) {
            let ctx = el
                .cast::<u8>()
                .sub(offset_of!(JSContext, link))
                .cast::<JSContext>();
            let new_tab = js_realloc_rt(
                rt,
                (*ctx).class_proto.cast(),
                size_of::<JSValue>() * new_size as usize,
            )
            .cast::<JSValue>();
            if new_tab.is_null() {
                return -1;
            }
            for i in (*rt).class_count..new_size {
                *new_tab.add(i as usize) = JS_NULL;
            }
            (*ctx).class_proto = new_tab;
        }
        let new_class_array = js_realloc_rt(
            rt,
            (*rt).class_array.cast(),
            size_of::<JSClass>() * new_size as usize,
        )
        .cast::<JSClass>();
        if new_class_array.is_null() {
            return -1;
        }
        ptr::write_bytes(
            new_class_array.add((*rt).class_count as usize),
            0,
            (new_size - (*rt).class_count) as usize,
        );
        (*rt).class_array = new_class_array;
        (*rt).class_count = new_size;
    }
    let cl = (*rt).class_array.add(class_id as usize);
    (*cl).class_id = class_id;
    (*cl).class_name = JS_DupAtomRT(rt, name);
    (*cl).finalizer = (*class_def).finalizer;
    (*cl).gc_mark = (*class_def).gc_mark;
    (*cl).call = (*class_def).call;
    (*cl).exotic = (*class_def).exotic;
    0
}
pub unsafe fn JS_NewClass(
    rt: *mut JSRuntime,
    class_id: JSClassID,
    class_def: *const JSClassDef,
) -> i32 {
    let len = core::ffi::CStr::from_ptr((*class_def).class_name)
        .to_bytes()
        .len() as i32;
    let mut name = __JS_FindAtom(
        rt,
        (*class_def).class_name,
        len as usize,
        JS_ATOM_TYPE_STRING,
    );
    if name == JS_ATOM_NULL as u32 {
        name = __JS_NewAtomInit(rt, (*class_def).class_name, len, JS_ATOM_TYPE_STRING);
        if name == JS_ATOM_NULL as u32 {
            return -1;
        }
    }
    let ret = JS_NewClass1(rt, class_id, class_def, name);
    JS_FreeAtomRT(rt, name);
    ret
}
unsafe fn JS_IsEmptyString(v: JSValueConst) -> JS_BOOL {
    (JS_VALUE_GET_TAG(v) == JS_TAG_STRING && (*JS_VALUE_GET_PTR(v).cast::<JSString>()).len() == 0)
        as i32
}
unsafe fn add_gc_object(
    rt: *mut JSRuntime,
    h: *mut JSGCObjectHeader,
    obj_type: JSGCObjectTypeEnum,
) {
    (*js_rc(h.cast())).gc_obj_type_and_mark = obj_type as u8 & 0x7f;
    list_add_tail(&mut (*h).link, &mut (*rt).gc_obj_list);
}
unsafe fn remove_gc_object(h: *mut JSGCObjectHeader) {
    list_del(&mut (*h).link);
}
pub unsafe fn JS_MarkValue(rt: *mut JSRuntime, val: JSValueConst, mark_func: JS_MarkFunc) {
    if JS_VALUE_HAS_REF_COUNT(val) != 0 {
        match JS_VALUE_GET_TAG(val) {
            JS_TAG_OBJECT | JS_TAG_FUNCTION_BYTECODE | JS_TAG_MODULE => {
                mark_func(rt, JS_VALUE_GET_PTR(val).cast())
            }
            _ => {}
        }
    }
}
