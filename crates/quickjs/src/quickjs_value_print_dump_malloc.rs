// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:1727. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_malloc_dump_arenas(mut s: *mut JSMallocContext) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut el: *mut list_head = core::mem::zeroed();
let mut block_size_idx: i32 = core::mem::zeroed();
let mut block_size: i32 = core::mem::zeroed();
let mut ar: *mut JSMallocArena = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1733
1 => {
vm_block = if ((((block_size_idx) < ((31 as i32))) as i32)) != 0 { 8 } else { 0 }; continue;
}
// C line 1733
2 => {
let _ = { let old = block_size_idx; block_size_idx = (block_size_idx).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 1735
3 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!(*(((*(s)).arena_list).as_mut_ptr()).offset((block_size_idx) as isize)))) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 1735
4 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 3; continue;
}
// C line 1737
5 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%20p %10u %9.1f%%\n".as_ptr(), &[QuickJSPrintArg::Ptr(ar as *const c_void), QuickJSPrintArg::Int(block_size as u64), QuickJSPrintArg::Float((((((((*(ar)).n_used_blocks) as f64)) / ((((((*(ar)).n_blocks) as i32)) as f64)))) * ((((100 as i32)) as f64))) as f64)]));
vm_block = 4; continue;
}
// C line 1736
6 => {
ar = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMallocArena, link) as usize)) as isize))) as *mut JSMallocArena);
vm_block = 5; continue;
}
// C line 1735
7 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!(*(((*(s)).arena_list).as_mut_ptr()).offset((block_size_idx) as isize)))).next; el = assigned; assigned };
vm_block = 3; continue;
}
// C line 1734
8 => {
block_size = ((*((js_malloc_block_sizes).as_ptr()).offset((block_size_idx) as isize)) as i32);
vm_block = 7; continue;
}
// C line 1733
9 => {
let _ = { let assigned = (0 as i32); block_size_idx = assigned; assigned };
vm_block = 1; continue;
}
// C line 1732
10 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%20s %10s %10s\n".as_ptr(), &[QuickJSPrintArg::Str(c"PTR".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"BLK_SIZE".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"ALLOC".as_ptr() as *const c_char)]));
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:1749. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
#[cfg(feature="malloc-iter")]
unsafe fn js_malloc_iter(mut s: *mut JSMallocContext, mut iter_func: Option<JSMallocIterFunc>, mut iter_opaque: *mut c_void) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut el: *mut list_head = core::mem::zeroed();
let mut block_size_idx: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n_words: i32 = core::mem::zeroed();
let mut bmp: u32 = core::mem::zeroed();
let mut block_size: u32 = core::mem::zeroed();
let mut ar: *mut JSMallocArena = core::mem::zeroed();
let mut lb: *mut JSMallocLargeBlockHeader = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1771
1 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(s)).large_block_list))) as i32)) != 0 { 4 } else { 0 }; continue;
}
// C line 1771
2 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 1; continue;
}
// C line 1773
3 => {
let _ = (iter_func).expect("original non-null iteration callback")(iter_opaque, (((((*(lb)).header).user_data).as_mut_ptr()) as *mut c_void).cast::<c_void>());
vm_block = 2; continue;
}
// C line 1772
4 => {
lb = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMallocLargeBlockHeader, link) as usize)) as isize))) as *mut JSMallocLargeBlockHeader);
vm_block = 3; continue;
}
// C line 1771
5 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(s)).large_block_list))).next; el = assigned; assigned };
vm_block = 1; continue;
}
// C line 1756
6 => {
vm_block = if ((((block_size_idx) < ((31 as i32))) as i32)) != 0 { 21 } else { 5 }; continue;
}
// C line 1756
7 => {
let _ = { let old = block_size_idx; block_size_idx = (block_size_idx).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 1758
8 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!(*(((*(s)).arena_list).as_mut_ptr()).offset((block_size_idx) as isize)))) as i32)) != 0 { 19 } else { 7 }; continue;
}
// C line 1758
9 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 8; continue;
}
// C line 1761
10 => {
vm_block = if ((((i) < (n_words)) as i32)) != 0 { 16 } else { 9 }; continue;
}
// C line 1761
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 1763
12 => {
vm_block = if ((((bmp) != ((((0 as i32)) as u32))) as i32)) != 0 { 15 } else { 11 }; continue;
}
// C line 1766
13 => {
let _ = (iter_func).expect("original non-null iteration callback")(iter_opaque, get_arena_block(ar, ((((i).wrapping_mul((32 as i32))).wrapping_add(j)) as u32), block_size).cast::<c_void>());
vm_block = 12; continue;
}
// C line 1765
14 => {
let _ = { bmp = (((((bmp) as u32)) & ((((!(((1 as i32)).wrapping_shl((j) as u32)))) as u32)))) as u32; bmp };
vm_block = 13; continue;
}
// C line 1764
15 => {
let _ = { let assigned = crate::cutils_header::ctz32(bmp); j = assigned; assigned };
vm_block = 14; continue;
}
// C line 1762
16 => {
let _ = { let assigned = *(((*(ar)).bitmap).as_mut_ptr()).offset((i) as isize); bmp = assigned; assigned };
vm_block = 12; continue;
}
// C line 1761
17 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 1760
18 => {
let _ = { let assigned = ((((((*(ar)).n_blocks) as i32)).wrapping_add((31 as i32))) / ((32 as i32))); n_words = assigned; assigned };
vm_block = 17; continue;
}
// C line 1759
19 => {
ar = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMallocArena, link) as usize)) as isize))) as *mut JSMallocArena);
vm_block = 18; continue;
}
// C line 1758
20 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!(*(((*(s)).arena_list).as_mut_ptr()).offset((block_size_idx) as isize)))).next; el = assigned; assigned };
vm_block = 8; continue;
}
// C line 1757
21 => {
block_size = ((*((js_malloc_block_sizes).as_ptr()).offset((block_size_idx) as isize)) as u32);
vm_block = 20; continue;
}
// C line 1756
22 => {
let _ = { let assigned = (0 as i32); block_size_idx = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}
