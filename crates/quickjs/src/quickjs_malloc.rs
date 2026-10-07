// quickjs.c allocator section, C lines 234..317 and 1419..1725.
// Copyright 2017-2021 Fabrice Bellard and Charlie Gordon; MIT.
const JS_MALLOC_ALIGN: usize = 8;
const JS_MALLOC_ARENA_SIZE: usize = 4096;
const JS_MALLOC_BLOCK_SIZE_COUNT: usize = 31;
const JS_MALLOC_MIN_SMALL_SIZE: usize = 16;
const JS_MALLOC_MAX_SMALL_SIZE: usize = 512;
// quickjs.c:248..253. Select the original sanitizer-friendly host-allocation
// policy explicitly; enabling this feature does not instrument Rust code.
const JS_MALLOC_LARGE_BLOCKS_ONLY: bool = cfg!(feature = "malloc-large-blocks");
const FREE_NIL: u16 = 0xffff;
#[repr(C)]
#[derive(Clone, Copy)]
union JSMallocBlockIndex {
    block_idx: u16,
    free_next: u16,
}
#[repr(C, align(8))]
struct JSMallocBlockHeader {
    u: JSMallocBlockIndex,
    block_size_idx: u8,
    gc_obj_type_and_mark: u8,
    ref_count: i32,
    user_data: [u8; 0],
}
impl JSMallocBlockHeader {
    #[inline]
    fn gc_obj_type(&self) -> u8 {
        self.gc_obj_type_and_mark & 0x7f
    }
}

#[repr(C)]
struct JSMallocLargeBlockHeader {
    #[cfg(feature = "malloc-iter")]
    link: list_head,
    header: JSMallocBlockHeader,
}
#[repr(C, align(8))]
struct JSMallocArena {
    free_link: list_head,
    link: list_head,
    block_size_idx: u8,
    n_used_blocks: u16,
    n_blocks: u16,
    first_free_block: u16,
    #[cfg(feature = "malloc-iter")]
    bitmap: [u32; ((JS_MALLOC_ARENA_SIZE / JS_MALLOC_MIN_SMALL_SIZE) + 31) / 32],
    blocks: [u8; 0],
}
#[repr(C, align(8))]
struct JSMallocContext {
    arena_list: [list_head; JS_MALLOC_BLOCK_SIZE_COUNT],
    free_arena_list: [list_head; JS_MALLOC_BLOCK_SIZE_COUNT],
    #[cfg(feature = "malloc-iter")]
    large_block_list: list_head,
    zero_size_block: [u8; size_of::<JSMallocBlockHeader>()],
    mf: JSMallocFunctions,
    malloc_state: JSMallocState,
}
static js_malloc_block_sizes: [u16; JS_MALLOC_BLOCK_SIZE_COUNT] = [
    16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 144, 160, 176, 192, 208, 224,
    240, 256, 288, 320, 352, 384, 416, 448, 480, 512,
];
fn get_block_size_index(size: usize) -> usize {
    if size <= 16 {
        0
    } else if size <= 128 {
        (size + 7) / 8 - 2
    } else if size <= 256 {
        (size + 15) / 16 + 6
    } else if size <= 512 {
        (size + 31) / 32 + 14
    } else {
        JS_MALLOC_BLOCK_SIZE_COUNT
    }
}
unsafe fn get_zero_size_block(s: *mut JSMallocContext) -> *mut JSMallocBlockHeader {
    ptr::addr_of_mut!((*s).zero_size_block).cast()
}
unsafe fn block_user_data(b: *mut JSMallocBlockHeader) -> *mut c_void {
    ptr::addr_of_mut!((*b).user_data).cast()
}
unsafe fn js_malloc_init(s: *mut JSMallocContext) {
    ptr::write_bytes(s, 0, 1);
    (*get_zero_size_block(s)).u.block_idx = FREE_NIL;
    for i in 0..JS_MALLOC_BLOCK_SIZE_COUNT {
        init_list_head(&mut (*s).arena_list[i]);
        init_list_head(&mut (*s).free_arena_list[i]);
    }
    #[cfg(feature = "malloc-iter")]
    init_list_head(&mut (*s).large_block_list);
}
unsafe fn get_arena_block(
    ar: *mut JSMallocArena,
    idx: u32,
    block_size: u32,
) -> *mut JSMallocBlockHeader {
    ptr::addr_of_mut!((*ar).blocks)
        .cast::<u8>()
        .add((idx * block_size) as usize)
        .cast()
}
unsafe fn js_rc(p: *mut c_void) -> *mut JSMallocBlockHeader {
    p.cast::<u8>()
        .sub(offset_of!(JSMallocBlockHeader, user_data))
        .cast()
}
unsafe fn js_malloc_new_arena(
    s: *mut JSMallocContext,
    block_size_idx: usize,
) -> *mut JSMallocArena {
    let block_size = js_malloc_block_sizes[block_size_idx] as usize;
    let n_blocks = (JS_MALLOC_ARENA_SIZE - size_of::<JSMallocArena>()) / block_size;
    let ar = ((*s).mf.js_malloc.unwrap())(
        &mut (*s).malloc_state,
        size_of::<JSMallocArena>() + n_blocks * block_size,
    )
    .cast::<JSMallocArena>();
    if ar.is_null() {
        return ptr::null_mut();
    }
    (*ar).block_size_idx = block_size_idx as u8;
    (*ar).n_blocks = n_blocks as u16;
    (*ar).n_used_blocks = 0;
    (*ar).first_free_block = 0;
    #[cfg(feature = "malloc-iter")]
    for i in 0..(n_blocks + 31) / 32 {
        (*ar).bitmap[i] = 0;
    }
    for i in 0..n_blocks - 1 {
        let b = get_arena_block(ar, i as u32, block_size as u32);
        (*b).u.free_next = (i + 1) as u16;
        (*b).block_size_idx = block_size_idx as u8;
    }
    let b = get_arena_block(ar, (n_blocks - 1) as u32, block_size as u32);
    (*b).u.free_next = FREE_NIL;
    (*b).block_size_idx = block_size_idx as u8;
    list_add(&mut (*ar).link, &mut (*s).arena_list[block_size_idx]);
    list_add(
        &mut (*ar).free_link,
        &mut (*s).free_arena_list[block_size_idx],
    );
    ar
}
unsafe fn js_malloc_large(s: *mut JSMallocContext, size: usize) -> *mut c_void {
    let b = ((*s).mf.js_malloc.unwrap())(
        &mut (*s).malloc_state,
        size_of::<JSMallocLargeBlockHeader>().wrapping_add(size),
    )
    .cast::<JSMallocLargeBlockHeader>();
    if b.is_null() {
        return ptr::null_mut();
    }
    (*b).header.u.block_idx = FREE_NIL;
    (*b).header.block_size_idx = 0xff;
    #[cfg(feature = "malloc-iter")]
    list_add_tail(&mut (*b).link, &mut (*s).large_block_list);
    block_user_data(&mut (*b).header)
}
unsafe fn __js_malloc(s: *mut JSMallocContext, size: usize) -> *mut c_void {
    if size == 0 {
        return block_user_data(get_zero_size_block(s));
    }
    let total_size = (size.wrapping_add(JS_MALLOC_ALIGN - 1) & !(JS_MALLOC_ALIGN - 1))
        .wrapping_add(size_of::<JSMallocBlockHeader>());
    if !JS_MALLOC_LARGE_BLOCKS_ONLY && total_size <= JS_MALLOC_MAX_SMALL_SIZE {
        let index = get_block_size_index(total_size);
        let block_size = js_malloc_block_sizes[index] as u32;
        let head = ptr::addr_of_mut!((*s).free_arena_list[index]);
        let el = (*head).next;
        let ar = if el == head {
            let ar = js_malloc_new_arena(s, index);
            if ar.is_null() {
                return ptr::null_mut();
            }
            ar
        } else {
            el.cast::<u8>()
                .sub(offset_of!(JSMallocArena, free_link))
                .cast::<JSMallocArena>()
        };
        let block_idx = (*ar).first_free_block;
        let b = get_arena_block(ar, block_idx as u32, block_size);
        (*ar).first_free_block = (*b).u.free_next;
        (*b).u.block_idx = block_idx;
        (*ar).n_used_blocks += 1;
        if (*ar).n_used_blocks == (*ar).n_blocks {
            list_del(&mut (*ar).free_link);
        }
        #[cfg(feature = "malloc-iter")]
        {
            (*ar).bitmap[block_idx as usize / 32] |= 1u32 << (block_idx % 32);
        }
        block_user_data(b)
    } else {
        js_malloc_large(s, size)
    }
}
unsafe fn __js_free(s: *mut JSMallocContext, p: *mut c_void) {
    if p.is_null() {
        return;
    }
    let b = js_rc(p);
    if (*b).u.block_idx == FREE_NIL {
        if b != get_zero_size_block(s) {
            let lb = p.cast::<u8>().sub(
                offset_of!(JSMallocLargeBlockHeader, header)
                    + offset_of!(JSMallocBlockHeader, user_data),
            );
            #[cfg(feature = "malloc-iter")]
            list_del(&mut (*lb.cast::<JSMallocLargeBlockHeader>()).link);
            ((*s).mf.js_free.unwrap())(&mut (*s).malloc_state, lb.cast());
        }
    } else {
        let block_idx = (*b).u.block_idx;
        let index = (*b).block_size_idx as usize;
        let block_size = js_malloc_block_sizes[index] as usize;
        let ar = b
            .cast::<u8>()
            .sub(block_size * block_idx as usize + size_of::<JSMallocArena>())
            .cast::<JSMallocArena>();
        (*b).u.free_next = (*ar).first_free_block;
        (*ar).first_free_block = block_idx;
        #[cfg(feature = "malloc-iter")]
        {
            (*ar).bitmap[block_idx as usize / 32] &= !(1u32 << (block_idx % 32));
        }
        if (*ar).n_used_blocks == (*ar).n_blocks {
            list_add(&mut (*ar).free_link, &mut (*s).free_arena_list[index]);
        }
        (*ar).n_used_blocks -= 1;
        if (*ar).n_used_blocks == 0 {
            list_del(&mut (*ar).link);
            list_del(&mut (*ar).free_link);
            ((*s).mf.js_free.unwrap())(&mut (*s).malloc_state, ar.cast());
        }
    }
}
unsafe fn __js_realloc(s: *mut JSMallocContext, p: *mut c_void, mut size: usize) -> *mut c_void {
    if p.is_null() {
        return __js_malloc(s, size);
    } else if size == 0 {
        __js_free(s, p);
        return ptr::null_mut();
    }
    let b = js_rc(p);
    if (*b).u.block_idx == FREE_NIL {
        if b == get_zero_size_block(s) {
            return __js_malloc(s, size);
        }
        let lb = p.cast::<u8>().sub(
            offset_of!(JSMallocLargeBlockHeader, header)
                + offset_of!(JSMallocBlockHeader, user_data),
        );
        #[cfg(feature = "malloc-iter")]
        list_del(&mut (*lb.cast::<JSMallocLargeBlockHeader>()).link);
        let new_lb = ((*s).mf.js_realloc.unwrap())(
            &mut (*s).malloc_state,
            lb.cast(),
            size_of::<JSMallocLargeBlockHeader>().wrapping_add(size),
        )
        .cast::<JSMallocLargeBlockHeader>();
        if new_lb.is_null() {
            #[cfg(feature = "malloc-iter")]
            list_add_tail(
                &mut (*lb.cast::<JSMallocLargeBlockHeader>()).link,
                &mut (*s).large_block_list,
            );
            return ptr::null_mut();
        }
        (*new_lb).header.u.block_idx = FREE_NIL;
        (*new_lb).header.block_size_idx = 0xff;
        #[cfg(feature = "malloc-iter")]
        list_add_tail(&mut (*new_lb).link, &mut (*s).large_block_list);
        block_user_data(&mut (*new_lb).header)
    } else {
        let block_size = js_malloc_block_sizes[(*b).block_size_idx as usize] as usize;
        let total_size = (size.wrapping_add(JS_MALLOC_ALIGN - 1) & !(JS_MALLOC_ALIGN - 1))
            .wrapping_add(size_of::<JSMallocBlockHeader>());
        if total_size <= block_size {
            return p;
        }
        let new_ptr = __js_malloc(s, size);
        if new_ptr.is_null() {
            return ptr::null_mut();
        }
        let new_b = js_rc(new_ptr);
        (*new_b).gc_obj_type_and_mark = (*b).gc_obj_type_and_mark;
        (*new_b).ref_count = (*b).ref_count;
        let old_size = block_size - size_of::<JSMallocBlockHeader>();
        if size > old_size {
            size = old_size;
        }
        ptr::copy_nonoverlapping(p.cast::<u8>(), new_ptr.cast::<u8>(), size);
        __js_free(s, p);
        new_ptr
    }
}
unsafe fn __js_malloc_usable_size(s: *mut JSMallocContext, p: *const u8) -> usize {
    if p.is_null() {
        return 0;
    }
    let b = js_rc(p.cast_mut().cast());
    if (*b).u.block_idx == FREE_NIL {
        if b == get_zero_size_block(s) {
            return 0;
        }
        let lb = p.sub(
            offset_of!(JSMallocLargeBlockHeader, header)
                + offset_of!(JSMallocBlockHeader, user_data),
        );
        if let Some(usable_size) = (*s).mf.js_malloc_usable_size {
            let size = usable_size(lb.cast());
            if size != 0 {
                size.wrapping_sub(size_of::<JSMallocLargeBlockHeader>())
            } else {
                0
            }
        } else {
            0
        }
    } else {
        js_malloc_block_sizes[(*b).block_size_idx as usize] as usize
            - size_of::<JSMallocBlockHeader>()
    }
}
