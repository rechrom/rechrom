// Matching injected allocator capacity policy for the two diagnostic runtimes.
unsafe fn diagnostic_capacity(p:*const c_void)->usize {if p.is_null(){0}else{*p.cast::<u8>().sub(16).cast::<usize>()}}
unsafe fn diagnostic_malloc(s:*mut JSMallocState,n:usize)->*mut c_void {
    if (*s).malloc_size.saturating_add(n)>(*s).malloc_limit{return core::ptr::null_mut();}
    let p=std::alloc::alloc(std::alloc::Layout::from_size_align(n+16,16).unwrap());if p.is_null(){return p.cast();}*p.cast::<usize>()=n;
    (*s).malloc_count+=1;(*s).malloc_size+=n;p.add(16).cast()
}
unsafe fn diagnostic_free(s:*mut JSMallocState,p:*mut c_void) {if !p.is_null(){let n=diagnostic_capacity(p);(*s).malloc_count-=1;(*s).malloc_size-=n;std::alloc::dealloc(p.cast::<u8>().sub(16),std::alloc::Layout::from_size_align(n+16,16).unwrap());}}
unsafe fn diagnostic_realloc(s:*mut JSMallocState,p:*mut c_void,n:usize)->*mut c_void {
    if p.is_null(){return diagnostic_malloc(s,n);}if n==0 {diagnostic_free(s,p);return core::ptr::null_mut();}let old=diagnostic_capacity(p);
    if (*s).malloc_size.saturating_sub(old).saturating_add(n)>(*s).malloc_limit{return core::ptr::null_mut();}
    let p=std::alloc::realloc(p.cast::<u8>().sub(16),std::alloc::Layout::from_size_align(old+16,16).unwrap(),n+16);if p.is_null(){return p.cast();}*p.cast::<usize>()=n;
    (*s).malloc_size=(*s).malloc_size-old+n;p.add(16).cast()
}
static DIAGNOSTIC_MALLOC:JSMallocFunctions=JSMallocFunctions {js_malloc:Some(diagnostic_malloc),js_free:Some(diagnostic_free),js_realloc:Some(diagnostic_realloc),js_malloc_usable_size:Some(diagnostic_capacity)};
