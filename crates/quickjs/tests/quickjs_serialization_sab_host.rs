// Genuine injected shared-buffer ownership for the C/Rust oracle; the engine
// retains no platform allocator or reference-count policy.
struct SerializationSABHost {entries:Vec<(*mut c_void,usize,usize)>}
unsafe fn serialization_sab_alloc(opaque:*mut c_void,size:usize)->*mut c_void {
    let host=&mut*opaque.cast::<SerializationSABHost>();let size=size.max(1);
    let ptr=std::alloc::alloc(std::alloc::Layout::from_size_align(size,8).unwrap()).cast::<c_void>();
    if !ptr.is_null(){host.entries.push((ptr,size,1));}ptr
}
unsafe fn serialization_sab_dup(opaque:*mut c_void,ptr:*mut c_void){let host=&mut*opaque.cast::<SerializationSABHost>();let entry=host.entries.iter_mut().find(|e|e.0==ptr).expect("valid shared pointer");entry.2+=1;}
unsafe fn serialization_sab_free(opaque:*mut c_void,ptr:*mut c_void){let host=&mut*opaque.cast::<SerializationSABHost>();let index=host.entries.iter().position(|e|e.0==ptr).expect("valid shared pointer");host.entries[index].2-=1;if host.entries[index].2==0 {let entry=host.entries.swap_remove(index);std::alloc::dealloc(ptr.cast(),std::alloc::Layout::from_size_align(entry.1,8).unwrap());}}
unsafe fn serialization_emit_bytes(out:&mut Vec<u8>,data:*const u8,len:usize,sab_tab:*mut *mut u8,sab_len:usize) {
    let mut bytes=core::slice::from_raw_parts(data,len).to_vec();
    for i in 0..sab_len {
        let pointer=(*sab_tab.add(i) as usize as u64).to_le_bytes();
        let canonical=(0x7f00_0000_0000_0000u64+i as u64).to_le_bytes();
        for j in 0..=len.saturating_sub(8) {if len>=8&&bytes[j..j+8]==pointer {bytes[j..j+8].copy_from_slice(&canonical);}}
    }
    out.extend_from_slice(&bytes);
}
