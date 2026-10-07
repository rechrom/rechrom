// Actual public SAB callbacks and deterministic allocator-failure trials.
struct BufferFaultState { call: usize, fail_at: usize, live: usize }
unsafe fn buffer_fault_malloc(s:*mut JSMallocState,n:usize)->*mut c_void {
    let state=&mut *((*s).opaque as *mut BufferFaultState);let call=state.call;state.call+=1;
    if call==state.fail_at{return ptr::null_mut();}
    let p=js_def_malloc(s,n);if !p.is_null(){state.live+=1;}p
}
unsafe fn buffer_fault_free(s:*mut JSMallocState,p:*mut c_void) {
    if !p.is_null(){(*((*s).opaque as *mut BufferFaultState)).live-=1;}js_def_free(s,p)
}
unsafe fn buffer_fault_realloc(s:*mut JSMallocState,p:*mut c_void,n:usize)->*mut c_void {
    let state=&mut *((*s).opaque as *mut BufferFaultState);let call=state.call;state.call+=1;
    if call==state.fail_at{return ptr::null_mut();}
    let q=js_def_realloc(s,p,n);
    if p.is_null() && !q.is_null(){state.live+=1;}
    if !p.is_null() && n==0{state.live-=1;}q
}
struct BufferHostState {blocks:Vec<Box<[u8]>>,events:Vec<u64>}
unsafe fn buffer_host_alloc(opaque:*mut c_void,n:usize)->*mut c_void {
    let state=&mut *(opaque as *mut BufferHostState);state.events.push(0x100000000+n as u64);
    let mut block=vec![0xa5;n].into_boxed_slice();let p=block.as_mut_ptr();state.blocks.push(block);p.cast()
}
unsafe fn buffer_host_dup(opaque:*mut c_void,_p:*mut c_void) {(*(opaque as *mut BufferHostState)).events.push(0x200000000);}
unsafe fn buffer_host_free(opaque:*mut c_void,_p:*mut c_void) {(*(opaque as *mut BufferHostState)).events.push(0x300000000);}
unsafe fn buffers_dump_api_value(out:&mut Vec<u8>,ctx:*mut JSContext,result:JSValue) {
    out.extend_from_slice(&JS_VALUE_GET_NORM_TAG(result).to_le_bytes());
    if JS_IsException(result)!=0 {
        let exception=JS_GetException(ctx);let mut len=0;let encoded=JS_ToCStringLen2(ctx,&mut len,exception,0);assert!(!encoded.is_null());out.extend_from_slice(&(len as u32).to_le_bytes());out.extend_from_slice(core::slice::from_raw_parts(encoded.cast::<u8>(),len));JS_FreeCString(ctx,encoded);JS_FreeValue(ctx,exception);
    }
}
pub unsafe fn buffers_host_oom_fixture()->Vec<u8> {
    let mut out=Vec::new();
    let mut host=BufferHostState{blocks:Vec::new(),events:Vec::new()};let rt=JS_NewRuntime();assert!(!rt.is_null());let ctx=JS_NewContextRaw(rt);assert!(!ctx.is_null());
    let hooks=JSSharedArrayBufferFunctions{sab_alloc:Some(buffer_host_alloc),sab_free:Some(buffer_host_free),sab_dup:Some(buffer_host_dup),sab_opaque:ptr::addr_of_mut!(host).cast()};JS_SetSharedArrayBufferFunctions(rt,&hooks);
    let mut maximum=31;let buffer=js_array_buffer_constructor2(ctx,JS_UNDEFINED,7,&mut maximum,JS_CLASS_SHARED_ARRAY_BUFFER);
    assert_eq!(JS_IsException(buffer),0);let mut length=0;let data=JS_GetArrayBuffer(ctx,&mut length,buffer);assert!(!data.is_null());out.extend_from_slice(&(length as u32).to_le_bytes());out.extend_from_slice(core::slice::from_raw_parts(data,length));
    let mut args=[JS_NewInt32(ctx,23)];let result=js_array_buffer_resize(ctx,buffer,1,args.as_mut_ptr(),JS_CLASS_SHARED_ARRAY_BUFFER as i32);buffers_dump_api_value(&mut out,ctx,result);JS_FreeValue(ctx,result);let data=JS_GetArrayBuffer(ctx,&mut length,buffer);out.extend_from_slice(&(length as u32).to_le_bytes());out.extend_from_slice(core::slice::from_raw_parts(data,length));
    // SAB detach is deliberately a no-op and must preserve backing bytes.
    JS_DetachArrayBuffer(ctx,buffer);let _=JS_GetArrayBuffer(ctx,&mut length,buffer);out.extend_from_slice(&(length as u32).to_le_bytes());JS_FreeValue(ctx,buffer);
    let mut external=[0x51u8;17];let buffer=JS_NewArrayBuffer(ctx,external.as_mut_ptr(),external.len(),None,ptr::null_mut(),1);assert_eq!(JS_IsException(buffer),0);JS_FreeValue(ctx,buffer);
    // Resizable externally managed ArrayBuffers are rejected through the C API.
    let bad=js_array_buffer_constructor3(ctx,JS_UNDEFINED,7,&mut maximum,JS_CLASS_ARRAY_BUFFER,external.as_mut_ptr(),None,ptr::null_mut(),0);buffers_dump_api_value(&mut out,ctx,bad);JS_FreeValue(ctx,bad);
    JS_FreeContext(ctx);JS_FreeRuntime(rt);out.extend_from_slice(&(host.events.len() as u32).to_le_bytes());for event in host.events {out.extend_from_slice(&event.to_le_bytes());}
    let copy=vec![0x37u8;16384];
    for operation in 0..4 {
        for fail_at in 0..12 {
            let mut state=BufferFaultState{call:0,fail_at:usize::MAX,live:0};let allocator=JSMallocFunctions{js_malloc:Some(buffer_fault_malloc),js_free:Some(buffer_fault_free),js_realloc:Some(buffer_fault_realloc),js_malloc_usable_size:Some(js_def_malloc_usable_size)};
            let rt=JS_NewRuntime2(&allocator,ptr::addr_of_mut!(state).cast());assert!(!rt.is_null());let ctx=JS_NewContextRaw(rt);assert!(!ctx.is_null());
            let mut maximum=65536;let baseline=if operation>=2 {js_array_buffer_constructor2(ctx,JS_UNDEFINED,16384,&mut maximum,JS_CLASS_ARRAY_BUFFER)} else {JS_UNDEFINED};assert_eq!(JS_IsException(baseline),0);
            let mut view=JS_UNDEFINED;
            if operation>=2 {let mut args=[baseline,JS_UNDEFINED,JS_UNDEFINED];view=JS_NewTypedArray(ctx,1,args.as_mut_ptr(),JS_TYPED_ARRAY_UINT8C+2);assert_eq!(JS_IsException(view),0);}
            state.call=0;state.fail_at=fail_at;
            let result=match operation {
                0=>JS_NewArrayBufferCopy(ctx,copy.as_ptr(),copy.len()),
                1=>{let mut args=[JS_NewInt32(ctx,4096),JS_UNDEFINED,JS_UNDEFINED];JS_NewTypedArray(ctx,1,args.as_mut_ptr(),JS_TYPED_ARRAY_UINT8C+6)},
                2=>{let mut args=[JS_NewInt32(ctx,65536)];js_array_buffer_resize(ctx,baseline,1,args.as_mut_ptr(),JS_CLASS_ARRAY_BUFFER as i32)},
                _=>{let mut args=[JS_NewInt32(ctx,65536)];js_array_buffer_transfer(ctx,baseline,1,args.as_mut_ptr(),1)},
            };
            state.fail_at=usize::MAX;buffers_dump_api_value(&mut out,ctx,result);
            if operation!=1 && JS_IsObject(result)!=0 {let mut length=0;let p=JS_GetArrayBuffer(ctx,&mut length,result);if !p.is_null(){out.extend_from_slice(&(length as u32).to_le_bytes());}}
            if operation>=2 {let p=JS_VALUE_GET_PTR(view).cast::<JSObject>();out.extend_from_slice(&(*p).u.array.count.to_le_bytes());}
            JS_FreeValue(ctx,result);JS_FreeValue(ctx,view);JS_FreeValue(ctx,baseline);JS_FreeContext(ctx);JS_FreeRuntime(rt);assert_eq!(state.live,0,"allocator leak operation {operation} fault {fail_at}");
            out.extend_from_slice(&(state.live as u32).to_le_bytes());
        }
    }
    out
}
