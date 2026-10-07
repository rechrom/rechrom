// Isolate every custom-allocator failure in a child process. The host allocator
// records ownership independently of QuickJS's allocation accounting.
struct IntrinsicOomHost { attempts:u64,fail_at:u64,live:u64,bytes:u64,trace:u64 }
fn oom_trace(h:&mut IntrinsicOomHost,op:u64,size:usize){for x in [op,size as u64]{h.trace=(h.trace^x).wrapping_mul(1099511628211);}}
unsafe fn oom_malloc_cb(s:*mut JSMallocState,n:usize)->*mut c_void{
    let h=&mut *((*s).opaque.cast::<IntrinsicOomHost>());h.attempts+=1;oom_trace(h,1,n);
    if h.attempts==h.fail_at{return ptr::null_mut();}
    let p=std::alloc::alloc(std::alloc::Layout::from_size_align(n+16,16).unwrap());if p.is_null(){return ptr::null_mut();}
    *p.cast::<usize>()=n;h.live+=1;h.bytes+=n as u64;(*s).malloc_count+=1;(*s).malloc_size+=n;p.add(16).cast()
}
unsafe fn oom_free_cb(s:*mut JSMallocState,p:*mut c_void){
    if p.is_null(){return;}let h=&mut *((*s).opaque.cast::<IntrinsicOomHost>());let base=p.cast::<u8>().sub(16);let n=*base.cast::<usize>();oom_trace(h,3,n);
    h.live-=1;h.bytes-=n as u64;(*s).malloc_count-=1;(*s).malloc_size-=n;std::alloc::dealloc(base,std::alloc::Layout::from_size_align(n+16,16).unwrap());
}
unsafe fn oom_realloc_cb(s:*mut JSMallocState,p:*mut c_void,n:usize)->*mut c_void{
    if p.is_null(){return oom_malloc_cb(s,n);}if n==0{oom_free_cb(s,p);return ptr::null_mut();}
    let h=&mut *((*s).opaque.cast::<IntrinsicOomHost>());h.attempts+=1;oom_trace(h,2,n);if h.attempts==h.fail_at{return ptr::null_mut();}
    let base=p.cast::<u8>().sub(16);let old=*base.cast::<usize>();let new=std::alloc::realloc(base,std::alloc::Layout::from_size_align(old+16,16).unwrap(),n+16);if new.is_null(){return ptr::null_mut();}
    *new.cast::<usize>()=n;h.bytes=h.bytes+n as u64-old as u64;(*s).malloc_size=(*s).malloc_size+n-old;new.add(16).cast()
}
unsafe fn oom_usable_cb(p:*const c_void)->usize{if p.is_null(){0}else{*p.cast::<u8>().sub(16).cast::<usize>()}}
pub unsafe fn runtime_intrinsics_oom_fixture(fail_at:u64){
    use std::io::Write;
    let mut h=IntrinsicOomHost{attempts:0,fail_at,live:0,bytes:0,trace:1469598103934665603};
    let mf=JSMallocFunctions{js_malloc:Some(oom_malloc_cb),js_free:Some(oom_free_cb),js_realloc:Some(oom_realloc_cb),js_malloc_usable_size:Some(oom_usable_cb)};
    let rt=JS_NewRuntime2(&mf,ptr::addr_of_mut!(h).cast());let ctx=if rt.is_null(){ptr::null_mut()}else{JS_NewContext(rt)};
    // Flush the return status before destruction: original C has an intrusive
    // GC-list ordering bug on the class_proto allocation failure path. A child
    // crash must remain observable, never silently omitted from the result.
    let mut before=Vec::new();for x in [(!rt.is_null()) as u64,(!ctx.is_null()) as u64,h.attempts,h.live,h.bytes,h.trace]{before.extend_from_slice(&x.to_le_bytes());}
    std::io::stdout().write_all(&before).unwrap();std::io::stdout().flush().unwrap();
    if !ctx.is_null(){JS_FreeContext(ctx);}if !rt.is_null(){JS_FreeRuntime(rt);}
    let mut after=Vec::new();for x in [h.attempts,h.live,h.bytes,h.trace]{after.extend_from_slice(&x.to_le_bytes());}
    std::io::stdout().write_all(&after).unwrap();std::io::stdout().flush().unwrap();assert_eq!(h.live,0,"custom allocator leaked");assert_eq!(h.bytes,0,"custom allocator leaked bytes");
}
