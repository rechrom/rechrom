// Different real runtimes share external storage through the public buffer API.
// Queue ordering is established under the same global synchronization mutex;
// no sleeps are used as evidence that a waiter has entered its condition wait.
unsafe fn buffer_thread_context(address: usize) -> (*mut JSRuntime,*mut JSContext,JSValue) {
    let rt=JS_NewRuntime();assert!(!rt.is_null());
    let ctx=JS_NewContextRaw(rt);assert!(!ctx.is_null());(*rt).can_block=1;
    let buffer=JS_NewArrayBuffer(ctx,address as *mut u8,16,None,ptr::null_mut(),1);
    assert_eq!(JS_IsException(buffer),0);
    let mut args=[buffer,JS_UNDEFINED,JS_UNDEFINED];
    let array=JS_NewTypedArray(ctx,1,args.as_mut_ptr(),JS_TYPED_ARRAY_UINT8C+5);
    assert_eq!(JS_IsException(array),0);JS_FreeValue(ctx,buffer);
    (rt,ctx,array)
}
unsafe fn buffer_wait_thread(address: usize,index: i32,big: bool) -> Vec<u8> {
    let (rt,ctx,mut array)=buffer_thread_context(address);
    if big {
        let buffer=JS_GetTypedArrayBuffer(ctx,array,ptr::null_mut(),ptr::null_mut(),ptr::null_mut());
        JS_FreeValue(ctx,array);
        let mut args=[buffer,JS_UNDEFINED,JS_UNDEFINED];
        array=JS_NewTypedArray(ctx,1,args.as_mut_ptr(),JS_TYPED_ARRAY_UINT8C+7);JS_FreeValue(ctx,buffer);assert_eq!(JS_IsException(array),0);
    }
    let value=if big {JS_NewBigInt64(ctx,0)} else {JS_NewInt32(ctx,0)};
    let mut args=[array,JS_NewInt32(ctx,index),value,JS_NewInt32(ctx,30000)];
    let result=js_atomics_wait(ctx,JS_UNDEFINED,4,args.as_mut_ptr());assert_eq!(JS_IsException(result),0);
    let mut length=0;let text=JS_ToCStringLen2(ctx,&mut length,result,0);assert!(!text.is_null());
    let bytes=core::slice::from_raw_parts(text.cast::<u8>(),length).to_vec();JS_FreeCString(ctx,text);JS_FreeValue(ctx,result);JS_FreeValue(ctx,value);JS_FreeValue(ctx,array);JS_FreeContext(ctx);JS_FreeRuntime(rt);bytes
}
pub unsafe fn buffers_atomics_threads_fixture() -> Vec<u8> {
    let memory=Box::new([core::sync::atomic::AtomicU64::new(0),core::sync::atomic::AtomicU64::new(0)]);
    let address=memory.as_ptr() as usize;
    let mut handles=Vec::new();
    for (ordinal,(index,big)) in [(0,false),(0,true),(1,false)].into_iter().enumerate() {
        handles.push(std::thread::spawn(move||unsafe{buffer_wait_thread(address,index,big)}));
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        loop {
            let count=js_atomics_mutex().lock().unwrap().len();
            if count==ordinal+1 {break;}
            assert!(std::time::Instant::now()<deadline,"waiter registration timed out");std::thread::yield_now();
        }
    }
    let (rt,ctx,array)=buffer_thread_context(address);let mut out=Vec::new();
    for index in [0,0,1,0] {
        let mut args=[array,JS_NewInt32(ctx,index),JS_NewInt32(ctx,1)];
        let result=js_atomics_notify(ctx,JS_UNDEFINED,3,args.as_mut_ptr());assert_eq!(JS_IsException(result),0);
        out.extend_from_slice(&JS_VALUE_GET_INT(result).to_le_bytes());JS_FreeValue(ctx,result);
    }
    for handle in handles {let bytes=handle.join().unwrap();assert_eq!(bytes,b"ok");out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());out.extend_from_slice(&bytes);}
    assert!(js_atomics_mutex().lock().unwrap().is_empty());
    JS_FreeValue(ctx,array);JS_FreeContext(ctx);JS_FreeRuntime(rt);drop(memory);out
}
