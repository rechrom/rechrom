// quickjs.c Atomics synchronization C60565..60773. MIT.
// Host adaptation: Rust owns the global mutex, FIFO waiter registry and per-waiter
// condition variable. Registry membership is the C waiter's linked flag. A wait
// atomically releases/reacquires the same global mutex, including timeout and
// spurious-wakeup behavior. Standard-library timed waits use a relative timeout;
// C uses a CLOCK_REALTIME deadline, so external clock changes are host dependent.
struct JSAtomicsWaiter {
    address: usize,
    cond: std::sync::Condvar,
}
static js_atomics_waiter_list: std::sync::OnceLock<std::sync::Mutex<Vec<std::sync::Arc<JSAtomicsWaiter>>>> = std::sync::OnceLock::new();
fn js_atomics_mutex() -> &'static std::sync::Mutex<Vec<std::sync::Arc<JSAtomicsWaiter>>> {
    js_atomics_waiter_list.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}
unsafe fn cpu_pause() { std::hint::spin_loop(); }
unsafe fn buffer_modf(value: f64, integral: *mut f64) -> f64 {
    *integral = value.trunc();
    if value.is_nan() { return value; }
    if value.is_infinite() || value == *integral { return 0.0_f64.copysign(value); }
    value - *integral
}
unsafe fn js_atomics_wait(ctx: *mut JSContext, _this_obj: JSValueConst, _argc: i32, argv: *mut JSValueConst) -> JSValue {
    let mut idx=0;
    let p=js_atomics_get_buf(ctx,*argv,*argv.add(1),&mut idx,2);
    if p.is_null() { return JS_EXCEPTION; }
    let size_log2=typed_array_size_log2[((*p).class_id as u32-JS_CLASS_UINT8C_ARRAY) as usize];
    let address=(*p).u.array.u.uint8_ptr.add((idx as usize)<<size_log2) as usize;
    let mut value=0_i64;
    if size_log2==3 {
        if JS_ToBigInt64(ctx,&mut value,*argv.add(2))!=0 { return JS_EXCEPTION; }
    } else {
        let mut v32=0;
        if JS_ToInt32(ctx,&mut v32,*argv.add(2))!=0 { return JS_EXCEPTION; }
        value=v32 as i64;
    }
    let mut d=0.0;
    if JS_ToFloat64(ctx,&mut d,*argv.add(3))!=0 { return JS_EXCEPTION; }
    let timeout=if d.is_nan() || d>=9223372036854775808.0 { i64::MAX } else if d<0.0 { 0 } else { d as i64 };
    if (*(*ctx).rt).can_block==0 { return JS_ThrowTypeError(ctx,c"cannot block in this thread".as_ptr()); }
    let mut guard=js_atomics_mutex().lock().unwrap_or_else(|e|e.into_inner());
    let different=if size_log2==3 {
        (&*(address as *const core::sync::atomic::AtomicI64)).load(core::sync::atomic::Ordering::SeqCst)!=value
    } else {
        (&*(address as *const core::sync::atomic::AtomicI32)).load(core::sync::atomic::Ordering::SeqCst) as i64!=value
    };
    if different { drop(guard); return JS_AtomToString(ctx,crate::quickjs_atom::JS_ATOM_not_equal); }
    let waiter=std::sync::Arc::new(JSAtomicsWaiter {address,cond:std::sync::Condvar::new()});
    guard.push(waiter.clone());
    let timed_out;
    if timeout==i64::MAX {
        guard=waiter.cond.wait(guard).unwrap_or_else(|e|e.into_inner());
        timed_out=false;
    } else {
        let (next,result)=waiter.cond.wait_timeout(guard,std::time::Duration::from_millis(timeout as u64)).unwrap_or_else(|e|e.into_inner());
        guard=next;timed_out=result.timed_out();
    }
    if let Some(index)=guard.iter().position(|entry|std::sync::Arc::ptr_eq(entry,&waiter)) { guard.remove(index); }
    drop(guard);
    JS_AtomToString(ctx,if timed_out {crate::quickjs_atom::JS_ATOM_timed_out} else {crate::quickjs_atom::JS_ATOM_ok})
}
unsafe fn js_atomics_notify(ctx: *mut JSContext, _this_obj: JSValueConst, _argc: i32, argv: *mut JSValueConst) -> JSValue {
    let mut idx=0;
    let p=js_atomics_get_buf(ctx,*argv,*argv.add(1),&mut idx,1);
    if p.is_null() { return JS_EXCEPTION; }
    let size_log2=typed_array_size_log2[((*p).class_id as u32-JS_CLASS_UINT8C_ARRAY) as usize];
    let mut count=i32::MAX;
    if JS_IsUndefined(*argv.add(2))==0 && JS_ToInt32Clamp(ctx,&mut count,*argv.add(2),0,i32::MAX,0)!=0 { return JS_EXCEPTION; }
    let abuf=(*(*p).u.typed_array).buffer;
    let mut n=0;
    if (*(*abuf).u.array_buffer).shared!=0 && count>0 {
        let address=(*p).u.array.u.uint8_ptr.add((idx as usize)<<size_log2) as usize;
        let mut guard=js_atomics_mutex().lock().unwrap_or_else(|e|e.into_inner());
        let mut selected=Vec::new();
        let mut index=0;
        while index<guard.len() && n<count {
            if guard[index].address==address { selected.push(guard.remove(index));n+=1; } else {index+=1;}
        }
        for waiter in selected { waiter.cond.notify_one(); }
    }
    JS_NewInt32(ctx,n)
}

// C libc helpers confined to this source's exact byte search/comparison semantics.
unsafe fn buffer_memchr(data: *const c_void, needle: i32, length: usize) -> *mut c_void {
    let bytes=data.cast::<u8>();
    for index in 0..length { if *bytes.add(index)==needle as u8 { return bytes.add(index) as *mut c_void; } }
    ptr::null_mut()
}
unsafe fn buffer_strcmp(a: *const c_char,b: *const c_char)->i32 {
    let mut index=0;
    loop { let x=*a.add(index) as u8;let y=*b.add(index) as u8;
        if x!=y || x==0 { return x as i32-y as i32; } index+=1;
    }
}
