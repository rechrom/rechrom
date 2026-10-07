// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37276. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_object_list_init(mut s: *mut JSObjectList) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37278
1 => {
let _ = { let dst = (((s) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<JSObjectList>() as usize)) as usize); dst as *mut c_void };
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37281. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_object_list_get_hash(mut p: *mut JSObject, mut hash_size: u32) -> u32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37283
1 => {
return (((((((p) as usize)).wrapping_mul((((3163 as i32)) as usize))) & ((((hash_size).wrapping_sub((((1 as i32)) as u32))) as usize)))) as u32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37286. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_object_list_resize_hash(mut ctx: *mut JSContext, mut s: *mut JSObjectList, mut new_hash_size: u32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut e: *mut JSObjectListEntry = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut new_hash_table: *mut u32 = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37308
1 => {
return (0 as i32);
}
// C line 37302
2 => {
vm_block = if ((((i) < ((((*(s)).object_count) as u32))) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line 37302
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 37306
4 => {
let _ = { let assigned = i; *((*(s)).hash_table).offset((h) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 37305
5 => {
let _ = { let assigned = *((*(s)).hash_table).offset((h) as isize); (*(e)).hash_next = assigned; assigned };
vm_block = 4; continue;
}
// C line 37304
6 => {
let _ = { let assigned = js_object_list_get_hash((*(e)).obj, (*(s)).hash_size); h = assigned; assigned };
vm_block = 5; continue;
}
// C line 37303
7 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).object_tab).offset((i) as isize)); e = assigned; assigned };
vm_block = 6; continue;
}
// C line 37302
8 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 37299
9 => {
vm_block = if ((((i) < ((*(s)).hash_size)) as i32)) != 0 { 11 } else { 8 }; continue;
}
// C line 37299
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 37300
11 => {
let _ = { let assigned = ((((1 as i32)).wrapping_neg()) as u32); *((*(s)).hash_table).offset((i) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 37299
12 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 37297
13 => {
let _ = { let assigned = new_hash_size; (*(s)).hash_size = assigned; assigned };
vm_block = 12; continue;
}
// C line 37296
14 => {
let _ = { let assigned = new_hash_table; (*(s)).hash_table = assigned; assigned };
vm_block = 13; continue;
}
// C line 37295
15 => {
let _ = js_free(ctx, (((*(s)).hash_table) as *mut c_void));
vm_block = 14; continue;
}
// C line 37294
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37293
17 => {
vm_block = if ((!(!(new_hash_table).is_null()) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 37292
18 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<u32>() as usize)).wrapping_mul(((new_hash_size) as usize)))) as *mut u32); new_hash_table = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37313. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_object_list_add(mut ctx: *mut JSContext, mut s: *mut JSObjectList, mut obj: *mut JSObject) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut e: *mut JSObjectListEntry = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut new_hash_size: u32 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37334
1 => {
return (0 as i32);
}
// C line 37333
2 => {
let _ = { let assigned = ((((*(s)).object_count).wrapping_sub((1 as i32))) as u32); *((*(s)).hash_table).offset((h) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 37332
3 => {
let _ = { let assigned = *((*(s)).hash_table).offset((h) as isize); (*(e)).hash_next = assigned; assigned };
vm_block = 2; continue;
}
// C line 37331
4 => {
let _ = { let assigned = obj; (*(e)).obj = assigned; assigned };
vm_block = 3; continue;
}
// C line 37330
5 => {
let _ = { let assigned = js_object_list_get_hash(obj, (*(s)).hash_size); h = assigned; assigned };
vm_block = 4; continue;
}
// C line 37329
6 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).object_tab).offset(({ let old = (*(s)).object_count; (*(s)).object_count = ((*(s)).object_count).wrapping_add(1); old }) as isize)); e = assigned; assigned };
vm_block = 5; continue;
}
// C line 37327
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37326
8 => {
vm_block = if (js_object_list_resize_hash(ctx, s, new_hash_size)) != 0 { 7 } else { 6 }; continue;
}
// C line 37324
9 => {
vm_block = if ((((new_hash_size) <= ((((*(s)).object_count) as u32))) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 37325
10 => {
let _ = { new_hash_size = ((((new_hash_size) as u32)).wrapping_mul((((2 as i32)) as u32))) as u32; new_hash_size };
vm_block = 9; continue;
}
// C line 37323
11 => {
let _ = { let assigned = crate::cutils_header::max_uint32((*(s)).hash_size, (((4 as i32)) as u32)); new_hash_size = assigned; assigned };
vm_block = 9; continue;
}
// C line 37322
12 => {
vm_block = if ((((!(((!(((((((((*(s)).object_count).wrapping_add((1 as i32))) as u32)) >= ((*(s)).hash_size)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 11 } else { 6 }; continue;
}
// C line 37321
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37318
14 => {
vm_block = if (js_resize_array(ctx, ((((core::ptr::addr_of_mut!((*(s)).object_tab)) as *mut c_void)) as *mut *mut c_void), (((size_of::<JSObjectListEntry>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).object_size), ((*(s)).object_count).wrapping_add((1 as i32)))) != 0 { 13 } else { 12 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37338. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_object_list_find(mut ctx: *mut JSContext, mut s: *mut JSObjectList, mut obj: *mut JSObject) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut e: *mut JSObjectListEntry = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut p: u32 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37354
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37348
2 => {
vm_block = if ((((p) != (((((1 as i32)).wrapping_neg()) as u32))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 37352
3 => {
let _ = { let assigned = (*(e)).hash_next; p = assigned; assigned };
vm_block = 2; continue;
}
// C line 37351
4 => {
return ((p) as i32);
}
// C line 37350
5 => {
vm_block = if (((((*(e)).obj) == (obj)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 37349
6 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).object_tab).offset((p) as isize)); e = assigned; assigned };
vm_block = 5; continue;
}
// C line 37347
7 => {
let _ = { let assigned = *((*(s)).hash_table).offset((h) as isize); p = assigned; assigned };
vm_block = 2; continue;
}
// C line 37346
8 => {
let _ = { let assigned = js_object_list_get_hash(obj, (*(s)).hash_size); h = assigned; assigned };
vm_block = 7; continue;
}
// C line 37345
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37344
10 => {
vm_block = if (((((*(s)).object_count) == ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37357. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_object_list_end(mut ctx: *mut JSContext, mut s: *mut JSObjectList) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37360
1 => {
let _ = js_free(ctx, (((*(s)).hash_table) as *mut c_void));
vm_block = 0; continue;
}
// C line 37359
2 => {
let _ = js_free(ctx, (((*(s)).object_tab) as *mut c_void));
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37443. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_put_u8(mut s: *mut BCWriterState, mut v: u8) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37445
1 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!((*(s)).dbuf), v);
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37448. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_put_u16(mut s: *mut BCWriterState, mut v: u16) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37452
1 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!((*(s)).dbuf), v);
vm_block = 0; continue;
}
// C line 37451
2 => {
let _ = { let assigned = crate::cutils_header::bswap16(v); v = assigned; assigned };
vm_block = 1; continue;
}
// C line 37450
3 => {
vm_block = if (is_be()) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37455. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_put_u32(mut s: *mut BCWriterState, mut v: u32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37459
1 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!((*(s)).dbuf), v);
vm_block = 0; continue;
}
// C line 37458
2 => {
let _ = { let assigned = crate::cutils_header::bswap32(v); v = assigned; assigned };
vm_block = 1; continue;
}
// C line 37457
3 => {
vm_block = if (is_be()) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37462. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_put_u64(mut s: *mut BCWriterState, mut v: u64) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37466
1 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!((*(s)).dbuf), ((core::ptr::addr_of_mut!(v)) as *mut u8), (size_of::<u64>() as usize));
vm_block = 0; continue;
}
// C line 37465
2 => {
let _ = { let assigned = crate::cutils_header::bswap64(v); v = assigned; assigned };
vm_block = 1; continue;
}
// C line 37464
3 => {
vm_block = if (is_be()) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37469. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_put_leb128(mut s: *mut BCWriterState, mut v: u32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37471
1 => {
let _ = dbuf_put_leb128(core::ptr::addr_of_mut!((*(s)).dbuf), v);
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37474. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_put_sleb128(mut s: *mut BCWriterState, mut v: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37476
1 => {
let _ = dbuf_put_sleb128(core::ptr::addr_of_mut!((*(s)).dbuf), v);
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37479. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_set_flags(mut pflags: *mut u32, mut pidx: *mut i32, mut val: u32, mut n: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37482
1 => {
let _ = { *(pidx) = ((((*(pidx)) as i32)).wrapping_add(n)) as i32; *(pidx) };
vm_block = 0; continue;
}
// C line 37481
2 => {
let _ = { let assigned = ((*(pflags)) | ((val).wrapping_shl((*(pidx)) as u32))); *(pflags) = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37485. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_atom_to_idx(mut s: *mut BCWriterState, mut pres: *mut u32, mut atom: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: u32 = core::mem::zeroed();
let mut old_size: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37522
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37521 labels: fail
2 => {
let _ = { let assigned = (((0 as i32)) as u32); *(pres) = assigned; assigned };
vm_block = 1; continue;
}
// C line 37519
3 => {
return (0 as i32);
}
// C line 37518
4 => {
let _ = { let assigned = v; *(pres) = assigned; assigned };
vm_block = 3; continue;
}
// C line 37517
5 => {
let _ = { let assigned = v; *((*(s)).atom_to_idx).offset((atom) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 37516
6 => {
let _ = { v = ((((v) as u32)).wrapping_add((*(s)).first_atom)) as u32; v };
vm_block = 5; continue;
}
// C line 37515
7 => {
let _ = { let assigned = (atom).wrapping_add((*(s)).first_atom); *((*(s)).idx_to_atom).offset((v) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 37514
8 => {
let _ = { let assigned = (({ let old = (*(s)).idx_to_atom_count; (*(s)).idx_to_atom_count = ((*(s)).idx_to_atom_count).wrapping_add(1); old }) as u32); v = assigned; assigned };
vm_block = 7; continue;
}
// C line 37512
9 => {
vm_block = 2; continue;
}
// C line 37509
10 => {
vm_block = if (js_resize_array((*(s)).ctx, ((core::ptr::addr_of_mut!((*(s)).idx_to_atom)) as *mut *mut c_void), (((size_of::<JSAtom>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).idx_to_atom_size), ((*(s)).idx_to_atom_count).wrapping_add((1 as i32)))) != 0 { 9 } else { 8 }; continue;
}
// C line 37506
11 => {
vm_block = if ((((i) < ((*(s)).atom_to_idx_size)) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line 37506
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 37507
13 => {
let _ = { let assigned = (((0 as i32)) as u32); *((*(s)).atom_to_idx).offset((i) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 37506
14 => {
let _ = { let assigned = old_size; i = assigned; assigned };
vm_block = 11; continue;
}
// C line 37504
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37501
16 => {
vm_block = if (js_resize_array((*(s)).ctx, ((core::ptr::addr_of_mut!((*(s)).atom_to_idx)) as *mut *mut c_void), (((size_of::<u32>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).atom_to_idx_size), (((atom).wrapping_add((((1 as i32)) as JSAtom))) as i32))) != 0 { 15 } else { 14 }; continue;
}
// C line 37500
17 => {
let _ = { let assigned = (*(s)).atom_to_idx_size; old_size = assigned; assigned };
vm_block = 16; continue;
}
// C line 37498
18 => {
vm_block = if ((((atom) >= ((((*(s)).atom_to_idx_size) as JSAtom))) as i32)) != 0 { 17 } else { 10 }; continue;
}
// C line 37496
19 => {
return (0 as i32);
}
// C line 37495
20 => {
let _ = { let assigned = *((*(s)).atom_to_idx).offset((atom) as isize); *(pres) = assigned; assigned };
vm_block = 19; continue;
}
// C line 37494
21 => {
vm_block = if ((((((((atom) < ((((*(s)).atom_to_idx_size) as JSAtom))) as i32)) != 0) && (((((*((*(s)).atom_to_idx).offset((atom) as isize)) != ((((0 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 37493
22 => {
let _ = { atom = ((((atom) as u32)).wrapping_sub((*(s)).first_atom)) as JSAtom; atom };
vm_block = 21; continue;
}
// C line 37491
23 => {
return (0 as i32);
}
// C line 37490
24 => {
let _ = { let assigned = atom; *(pres) = assigned; assigned };
vm_block = 23; continue;
}
// C line 37489
25 => {
vm_block = if ((((((((atom) < ((*(s)).first_atom)) as i32)) != 0) || ((__JS_AtomIsTaggedInt(atom)) != 0)) as i32)) != 0 { 24 } else { 22 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37525. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_put_atom(mut s: *mut BCWriterState, mut atom: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37537
1 => {
return (0 as i32);
}
// C line 37536
2 => {
let _ = bc_put_leb128(s, v);
vm_block = 1; continue;
}
// C line 37530
3 => {
let _ = { let assigned = (((__JS_AtomToUInt32(atom)).wrapping_shl(((1 as i32)) as u32)) | ((((1 as i32)) as u32))); v = assigned; assigned };
vm_block = 2; continue;
}
// C line 37534
4 => {
let _ = { v = ((((v) as u32)).wrapping_shl(((1 as i32)) as u32)) as u32; v };
vm_block = 2; continue;
}
// C line 37533
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37532
6 => {
vm_block = if (bc_atom_to_idx(s, core::ptr::addr_of_mut!(v), atom)) != 0 { 5 } else { 4 }; continue;
}
// C line 37529
7 => {
vm_block = if (__JS_AtomIsTaggedInt(atom)) != 0 { 3 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37540. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_byte_swap(mut bc_buf: *mut u8, mut bc_len: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut pos: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut op: i32 = core::mem::zeroed();
let mut fmt: i32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37545
1 => {
vm_block = if ((((pos) < (bc_len)) as i32)) != 0 { 22 } else { 0 }; continue;
}
// C line 37596
2 => {
let _ = { pos = ((((pos) as i32)).wrapping_add(len)) as i32; pos };
vm_block = 1; continue;
}
// C line 37594
3 => {
vm_block = 2; continue;
}
// C line 37592
4 => {
vm_block = 2; continue;
}
// C line 37590
5 => {
let _ = crate::cutils_header::put_u16((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((2 as i32)) as isize)), crate::cutils_header::bswap16(((crate::cutils_header::get_u16((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((2 as i32)) as isize)))) as u16)));
vm_block = 4; continue;
}
// C line 37588
6 => {
let _ = crate::cutils_header::put_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), crate::cutils_header::bswap16(((crate::cutils_header::get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as u16)));
vm_block = 5; continue;
}
// C line 37586
7 => {
vm_block = 2; continue;
}
// C line 37583
8 => {
let _ = crate::cutils_header::put_u16(((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((4 as i32)) as isize))).offset((((4 as i32)) as isize)), crate::cutils_header::bswap16(((crate::cutils_header::get_u16(((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((4 as i32)) as isize))).offset((((4 as i32)) as isize)))) as u16)));
vm_block = 7; continue;
}
// C line 37582
9 => {
vm_block = if ((((fmt) == ((OP_FMT_atom_label_u16 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 37580
10 => {
let _ = crate::cutils_header::put_u32((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((4 as i32)) as isize)), crate::cutils_header::bswap32(crate::cutils_header::get_u32((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((4 as i32)) as isize)))));
vm_block = 9; continue;
}
// C line 37578
11 => {
let _ = crate::cutils_header::put_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), crate::cutils_header::bswap32(crate::cutils_header::get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))));
vm_block = 10; continue;
}
// C line 37575
12 => {
vm_block = 2; continue;
}
// C line 37573
13 => {
let _ = crate::cutils_header::put_u16((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((4 as i32)) as isize)), crate::cutils_header::bswap16(((crate::cutils_header::get_u16((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((4 as i32)) as isize)))) as u16)));
vm_block = 12; continue;
}
// C line 37571
14 => {
let _ = crate::cutils_header::put_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), crate::cutils_header::bswap32(crate::cutils_header::get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))));
vm_block = 13; continue;
}
// C line 37568
15 => {
vm_block = 2; continue;
}
// C line 37566
16 => {
let _ = crate::cutils_header::put_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), crate::cutils_header::bswap32(crate::cutils_header::get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))));
vm_block = 15; continue;
}
// C line 37559
17 => {
vm_block = 2; continue;
}
// C line 37557
18 => {
let _ = crate::cutils_header::put_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), crate::cutils_header::bswap16(((crate::cutils_header::get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as u16)));
vm_block = 17; continue;
}
// C line 37549
19 => {
vm_block = match fmt { x if x == (OP_FMT_npop_u16 as i32) => 6, x if x == (OP_FMT_atom_label_u16 as i32) => 11, x if x == (OP_FMT_atom_label_u8 as i32) => 11, x if x == (OP_FMT_label_u16 as i32) => 14, x if x == (OP_FMT_atom_u16 as i32) => 14, x if x == (OP_FMT_atom_u8 as i32) => 16, x if x == (OP_FMT_atom as i32) => 16, x if x == (OP_FMT_label as i32) => 16, x if x == (OP_FMT_const as i32) => 16, x if x == (OP_FMT_u32 as i32) => 16, x if x == (OP_FMT_i32 as i32) => 16, x if x == (OP_FMT_var_ref as i32) => 18, x if x == (OP_FMT_arg as i32) => 18, x if x == (OP_FMT_loc as i32) => 18, x if x == (OP_FMT_npop as i32) => 18, x if x == (OP_FMT_label16 as i32) => 18, x if x == (OP_FMT_i16 as i32) => 18, x if x == (OP_FMT_u16 as i32) => 18, _ => 3, }; continue;
}
// C line 37548
20 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)).fmt) as i32); fmt = assigned; assigned };
vm_block = 19; continue;
}
// C line 37547
21 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)).size) as i32); len = assigned; assigned };
vm_block = 20; continue;
}
// C line 37546
22 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 21; continue;
}
// C line 37544
23 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37600. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteFunctionBytecode(mut s: *mut BCWriterState, mut bc_buf1: *const u8, mut bc_len: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut pos: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut op: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut bc_buf: *mut u8 = core::mem::zeroed();
let mut val: u32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37643
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37642 labels: fail
2 => {
let _ = js_free((*(s)).ctx, ((bc_buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 37640
3 => {
return (0 as i32);
}
// C line 37639
4 => {
let _ = js_free((*(s)).ctx, ((bc_buf) as *mut c_void));
vm_block = 3; continue;
}
// C line 37637
5 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!((*(s)).dbuf), bc_buf, ((bc_len) as usize));
vm_block = 4; continue;
}
// C line 37635
6 => {
let _ = bc_byte_swap(bc_buf, bc_len);
vm_block = 5; continue;
}
// C line 37634
7 => {
vm_block = if (is_be()) != 0 { 6 } else { 5 }; continue;
}
// C line 37614
8 => {
vm_block = if ((((pos) < (bc_len)) as i32)) != 0 { 18 } else { 7 }; continue;
}
// C line 37631
9 => {
let _ = { pos = ((((pos) as i32)).wrapping_add(len)) as i32; pos };
vm_block = 8; continue;
}
// C line 37629
10 => {
vm_block = 9; continue;
}
// C line 37627
11 => {
vm_block = 9; continue;
}
// C line 37626
12 => {
let _ = crate::cutils_header::put_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), val);
vm_block = 11; continue;
}
// C line 37625
13 => {
vm_block = 2; continue;
}
// C line 37624
14 => {
vm_block = if (bc_atom_to_idx(s, core::ptr::addr_of_mut!(val), atom)) != 0 { 13 } else { 12 }; continue;
}
// C line 37623
15 => {
let _ = { let assigned = crate::cutils_header::get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); atom = assigned; assigned };
vm_block = 14; continue;
}
// C line 37617
16 => {
vm_block = match (((*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)).fmt) as i32) { x if x == (OP_FMT_atom_label_u16 as i32) => 15, x if x == (OP_FMT_atom_label_u8 as i32) => 15, x if x == (OP_FMT_atom_u16 as i32) => 15, x if x == (OP_FMT_atom_u8 as i32) => 15, x if x == (OP_FMT_atom as i32) => 15, _ => 10, }; continue;
}
// C line 37616
17 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)).size) as i32); len = assigned; assigned };
vm_block = 16; continue;
}
// C line 37615
18 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 17; continue;
}
// C line 37613
19 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 8; continue;
}
// C line 37611
20 => {
let _ = { let dst = (((bc_buf) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((bc_buf1) as *const c_void)) as *const u8, dst, (((bc_len) as usize)) as usize); dst as *mut c_void };
vm_block = 19; continue;
}
// C line 37610
21 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37609
22 => {
vm_block = if ((!(!(bc_buf).is_null()) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 37608
23 => {
let _ = { let assigned = ((js_malloc((*(s)).ctx, ((bc_len) as usize))) as *mut u8); bc_buf = assigned; assigned };
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37646. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteString(mut s: *mut BCWriterState, mut p: *mut JSString) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 37651
1 => {
vm_block = if ((((i) < ((((*(p)).len()) as i32))) as i32)) != 0 { 3 } else { 0 }; continue;
}
// C line 37651
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 37652
3 => {
let _ = bc_put_u16(s, *((((*(p)).u).str16).as_mut_ptr()).offset((i) as isize));
vm_block = 2; continue;
}
// C line 37651
4 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
// C line 37654
5 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!((*(s)).dbuf), (((*(p)).u).str8).as_mut_ptr(), (((*(p)).len()) as usize));
vm_block = 0; continue;
}
// C line 37650
6 => {
vm_block = if ((*(p)).is_wide_char()) != 0 { 4 } else { 5 }; continue;
}
// C line 37649
7 => {
let _ = bc_put_leb128(s, ((((*(p)).len()).wrapping_shl(((1 as i32)) as u32)) | ((((*(p)).is_wide_char()) as u32))));
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37658. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteBigInt(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut buf: JSBigIntBuf = core::mem::zeroed();
let mut p: *mut JSBigInt = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut v: js_limb_t = core::mem::zeroed();
let mut b: js_limb_t = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37704
1 => {
return (0 as i32);
}
// C line 37700
2 => {
vm_block = if ((((i) < (((len) % ((((((64 as i32)) / ((8 as i32)))) as u32))))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 37700
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 37701
4 => {
let _ = bc_put_u8(s, (((((*(((*(p)).tab).as_mut_ptr()).offset((((*(p)).len).wrapping_sub((((1 as i32)) as u32))) as isize)).wrapping_shr(((i).wrapping_mul((((8 as i32)) as u32))) as u32)) & ((((255 as i32)) as js_limb_t)))) as u8));
vm_block = 3; continue;
}
// C line 37700
5 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 37693
6 => {
vm_block = if ((((i) < (((len) / ((((((64 as i32)) / ((8 as i32)))) as u32))))) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line 37693
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 37697
8 => {
let _ = bc_put_u64(s, *(((*(p)).tab).as_mut_ptr()).offset((i) as isize));
vm_block = 7; continue;
}
// C line 37693
9 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 37692
10 => {
vm_block = if ((((len) > ((((0 as i32)) as u32))) as i32)) != 0 { 9 } else { 1 }; continue;
}
// C line 37691
11 => {
let _ = bc_put_leb128(s, len);
vm_block = 10; continue;
}
// C line 37674
12 => {
let _ = { let assigned = (((0 as i32)) as u32); len = assigned; assigned };
vm_block = 11; continue;
}
// C line 37681
13 => {
vm_block = if ((((shift) > ((0 as i32))) as i32)) != 0 { 20 } else { 11 }; continue;
}
// C line 37688
14 => {
let _ = { let old = len; len = (len).wrapping_sub(1); old };
vm_block = 13; continue;
}
// C line 37687
15 => {
let _ = { shift = ((((shift) as i32)).wrapping_sub((8 as i32))) as i32; shift };
vm_block = 14; continue;
}
// C line 37686
16 => {
vm_block = 11; continue;
}
// C line 37685
17 => {
vm_block = if ((((((b) & ((((1 as i32)) as js_limb_t)))) != ((((v).wrapping_shr(((shift).wrapping_sub((1 as i32))) as u32)) & ((((1 as i32)) as js_limb_t))))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 37684
18 => {
vm_block = 11; continue;
}
// C line 37683
19 => {
vm_block = if ((((((((b) != ((((0 as i32)) as js_limb_t))) as i32)) != 0) && (((((b) != ((((255 as i32)) as js_limb_t))) as i32)) != 0)) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 37682
20 => {
let _ = { let assigned = (((v).wrapping_shr((shift) as u32)) & ((((255 as i32)) as js_limb_t))); b = assigned; assigned };
vm_block = 19; continue;
}
// C line 37680
21 => {
let _ = { let assigned = ((64 as i32)).wrapping_sub((8 as i32)); shift = assigned; assigned };
vm_block = 13; continue;
}
// C line 37679
22 => {
let _ = { let assigned = *(((*(p)).tab).as_mut_ptr()).offset((((*(p)).len).wrapping_sub((((1 as i32)) as u32))) as isize); v = assigned; assigned };
vm_block = 21; continue;
}
// C line 37678
23 => {
let _ = { let assigned = ((*(p)).len).wrapping_mul((((((64 as i32)) / ((8 as i32)))) as u32)); len = assigned; assigned };
vm_block = 22; continue;
}
// C line 37672
24 => {
vm_block = if (((((((((*(p)).len) == ((((1 as i32)) as u32))) as i32)) != 0) && (((((*(((*(p)).tab).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((((0 as i32)) as js_limb_t))) as i32)) != 0)) as i32)) != 0 { 12 } else { 23 }; continue;
}
// C line 37669
25 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf), obj); p = assigned; assigned };
vm_block = 24; continue;
}
// C line 37671
26 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSBigInt); p = assigned; assigned };
vm_block = 24; continue;
}
// C line 37668
27 => {
vm_block = if (((((((obj).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 25 } else { 26 }; continue;
}
// C line 37666
28 => {
let _ = bc_put_u8(s, (((BC_TAG_BIG_INT as i32)) as u8));
vm_block = 27; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37709. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteFunctionTag(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut b: *mut JSFunctionBytecode = core::mem::zeroed();
let mut flags: u32 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vd: *mut JSBytecodeVarDef = core::mem::zeroed();
let mut cv: *mut JSClosureVar = core::mem::zeroed();
let mut vm_block: usize = 74;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37795 labels: fail
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37793
2 => {
return (0 as i32);
}
// C line 37789
3 => {
vm_block = if ((((i) < ((*(b)).cpool_count)) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 37789
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 37791
5 => {
vm_block = 1; continue;
}
// C line 37790
6 => {
vm_block = if (JS_WriteObjectRec(s, *((*(b)).cpool).offset((i) as isize))) != 0 { 5 } else { 4 }; continue;
}
// C line 37789
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 37783
8 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!((*(s)).dbuf), ((((*(b)).debug).source) as *mut u8), ((((*(b)).debug).source_len) as usize));
vm_block = 7; continue;
}
// C line 37782
9 => {
let _ = bc_put_leb128(s, ((((*(b)).debug).source_len) as u32));
vm_block = 8; continue;
}
// C line 37785
10 => {
let _ = bc_put_leb128(s, (((0 as i32)) as u32));
vm_block = 7; continue;
}
// C line 37781
11 => {
vm_block = if !(((*(b)).debug).source).is_null() { 9 } else { 10 }; continue;
}
// C line 37780
12 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!((*(s)).dbuf), ((*(b)).debug).pc2line_buf, ((((*(b)).debug).pc2line_len) as usize));
vm_block = 11; continue;
}
// C line 37779
13 => {
let _ = bc_put_leb128(s, ((((*(b)).debug).pc2line_len) as u32));
vm_block = 12; continue;
}
// C line 37778
14 => {
let _ = bc_put_atom(s, ((*(b)).debug).filename);
vm_block = 13; continue;
}
// C line 37777
15 => {
vm_block = if ((*(b)).has_debug()) != 0 { 14 } else { 7 }; continue;
}
// C line 37775
16 => {
vm_block = 1; continue;
}
// C line 37774
17 => {
vm_block = if (JS_WriteFunctionBytecode(s, (*(b)).byte_code_buf, (*(b)).byte_code_len)) != 0 { 16 } else { 15 }; continue;
}
// C line 37761
18 => {
vm_block = if ((((i) < ((*(b)).closure_var_count)) as i32)) != 0 { 29 } else { 17 }; continue;
}
// C line 37761
19 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 18; continue;
}
// C line 37771
20 => {
let _ = bc_put_u16(s, ((flags) as u16));
vm_block = 19; continue;
}
// C line 37770
21 => {
let _ = if ((((!(((((idx) <= ((16 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 20; continue;
}
// C line 37769
22 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(cv)).var_kind()) as u32), (4 as i32));
vm_block = 21; continue;
}
// C line 37768
23 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(cv)).is_lexical()) as u32), (1 as i32));
vm_block = 22; continue;
}
// C line 37767
24 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(cv)).is_const()) as u32), (1 as i32));
vm_block = 23; continue;
}
// C line 37766
25 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(cv)).closure_type()) as u32), (3 as i32));
vm_block = 24; continue;
}
// C line 37765
26 => {
let _ = { let assigned = (({ let assigned = (0 as i32); idx = assigned; assigned }) as u32); flags = assigned; assigned };
vm_block = 25; continue;
}
// C line 37764
27 => {
let _ = bc_put_leb128(s, (((*(cv)).var_idx) as u32));
vm_block = 26; continue;
}
// C line 37763
28 => {
let _ = bc_put_atom(s, (*(cv)).var_name);
vm_block = 27; continue;
}
// C line 37762
29 => {
cv = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((i) as isize));
vm_block = 28; continue;
}
// C line 37761
30 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 18; continue;
}
// C line 37743
31 => {
vm_block = if ((((i) < (((((*(b)).arg_count) as i32)).wrapping_add((((*(b)).var_count) as i32)))) as i32)) != 0 { 44 } else { 30 }; continue;
}
// C line 37743
32 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 37755
33 => {
let _ = bc_put_u8(s, ((flags) as u8));
vm_block = 32; continue;
}
// C line 37754
34 => {
let _ = if ((((!(((((idx) <= ((8 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 33; continue;
}
// C line 37753
35 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(vd)).has_scope()) as u32), (1 as i32));
vm_block = 34; continue;
}
// C line 37752
36 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(vd)).is_captured()) as u32), (1 as i32));
vm_block = 35; continue;
}
// C line 37751
37 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(vd)).is_lexical()) as u32), (1 as i32));
vm_block = 36; continue;
}
// C line 37750
38 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(vd)).is_const()) as u32), (1 as i32));
vm_block = 37; continue;
}
// C line 37749
39 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(vd)).var_kind()) as u32), (4 as i32));
vm_block = 38; continue;
}
// C line 37748
40 => {
let _ = { let assigned = (({ let assigned = (0 as i32); idx = assigned; assigned }) as u32); flags = assigned; assigned };
vm_block = 39; continue;
}
// C line 37747
41 => {
let _ = bc_put_leb128(s, (((*(vd)).var_ref_idx) as u32));
vm_block = 40; continue;
}
// C line 37746
42 => {
let _ = bc_put_leb128(s, ((((*(vd)).scope_next).wrapping_add((1 as i32))) as u32));
vm_block = 41; continue;
}
// C line 37745
43 => {
let _ = bc_put_atom(s, (*(vd)).var_name);
vm_block = 42; continue;
}
// C line 37744
44 => {
vd = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((i) as isize));
vm_block = 43; continue;
}
// C line 37743
45 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 31; continue;
}
// C line 37742
46 => {
let _ = bc_put_leb128(s, ((((((*(b)).arg_count) as i32)).wrapping_add((((*(b)).var_count) as i32))) as u32));
vm_block = 45; continue;
}
// C line 37758
47 => {
let _ = bc_put_leb128(s, (((0 as i32)) as u32));
vm_block = 30; continue;
}
// C line 37741
48 => {
vm_block = if !((*(b)).vardefs).is_null() { 46 } else { 47 }; continue;
}
// C line 37740
49 => {
let _ = bc_put_leb128(s, (((*(b)).byte_code_len) as u32));
vm_block = 48; continue;
}
// C line 37739
50 => {
let _ = bc_put_leb128(s, (((*(b)).cpool_count) as u32));
vm_block = 49; continue;
}
// C line 37738
51 => {
let _ = bc_put_leb128(s, (((*(b)).closure_var_count) as u32));
vm_block = 50; continue;
}
// C line 37737
52 => {
let _ = bc_put_leb128(s, (((*(b)).var_ref_count) as u32));
vm_block = 51; continue;
}
// C line 37736
53 => {
let _ = bc_put_leb128(s, (((*(b)).stack_size) as u32));
vm_block = 52; continue;
}
// C line 37735
54 => {
let _ = bc_put_leb128(s, (((*(b)).defined_arg_count) as u32));
vm_block = 53; continue;
}
// C line 37734
55 => {
let _ = bc_put_leb128(s, (((*(b)).var_count) as u32));
vm_block = 54; continue;
}
// C line 37733
56 => {
let _ = bc_put_leb128(s, (((*(b)).arg_count) as u32));
vm_block = 55; continue;
}
// C line 37731
57 => {
let _ = bc_put_atom(s, (*(b)).func_name);
vm_block = 56; continue;
}
// C line 37730
58 => {
let _ = bc_put_u8(s, (*(b)).js_mode);
vm_block = 57; continue;
}
// C line 37729
59 => {
let _ = bc_put_u16(s, ((flags) as u16));
vm_block = 58; continue;
}
// C line 37728
60 => {
let _ = if ((((!(((((idx) <= ((16 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 59; continue;
}
// C line 37727
61 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).is_direct_or_indirect_eval()) as u32), (1 as i32));
vm_block = 60; continue;
}
// C line 37726
62 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).has_debug()) as u32), (1 as i32));
vm_block = 61; continue;
}
// C line 37725
63 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).arguments_allowed()) as u32), (1 as i32));
vm_block = 62; continue;
}
// C line 37724
64 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).super_allowed()) as u32), (1 as i32));
vm_block = 63; continue;
}
// C line 37723
65 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).super_call_allowed()) as u32), (1 as i32));
vm_block = 64; continue;
}
// C line 37722
66 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).new_target_allowed()) as u32), (1 as i32));
vm_block = 65; continue;
}
// C line 37721
67 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).func_kind()) as u32), (2 as i32));
vm_block = 66; continue;
}
// C line 37720
68 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).need_home_object()) as u32), (1 as i32));
vm_block = 67; continue;
}
// C line 37719
69 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).is_derived_class_constructor()) as u32), (1 as i32));
vm_block = 68; continue;
}
// C line 37718
70 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).has_simple_parameter_list()) as u32), (1 as i32));
vm_block = 69; continue;
}
// C line 37717
71 => {
let _ = bc_set_flags(core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(idx), (((*(b)).has_prototype()) as u32), (1 as i32));
vm_block = 70; continue;
}
// C line 37716
72 => {
let _ = { let assigned = (({ let assigned = (0 as i32); idx = assigned; assigned }) as u32); flags = assigned; assigned };
vm_block = 71; continue;
}
// C line 37715
73 => {
let _ = bc_put_u8(s, (((BC_TAG_FUNCTION_BYTECODE as i32)) as u8));
vm_block = 72; continue;
}
// C line 37711
74 => {
b = ((((obj).u).ptr) as *mut JSFunctionBytecode);
vm_block = 73; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37798. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteModule(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut m: *mut JSModuleDef = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut rme: *mut JSReqModuleEntry = core::mem::zeroed();
let mut me: *mut JSExportEntry = core::mem::zeroed();
let mut se: *mut JSStarExportEntry = core::mem::zeroed();
let mut mi: *mut JSImportEntry = core::mem::zeroed();
let mut vm_block: usize = 42;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37848 labels: fail
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37846
2 => {
return (0 as i32);
}
// C line 37845
3 => {
vm_block = 1; continue;
}
// C line 37844
4 => {
vm_block = if (JS_WriteObjectRec(s, (*(m)).func_obj)) != 0 { 3 } else { 2 }; continue;
}
// C line 37842
5 => {
let _ = bc_put_u8(s, ((((*(m)).has_tla as i32)) as u8));
vm_block = 4; continue;
}
// C line 37834
6 => {
vm_block = if ((((i) < ((*(m)).import_entries_count)) as i32)) != 0 { 12 } else { 5 }; continue;
}
// C line 37834
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 37839
8 => {
let _ = bc_put_leb128(s, (((*(mi)).req_module_idx) as u32));
vm_block = 7; continue;
}
// C line 37838
9 => {
let _ = bc_put_atom(s, (*(mi)).import_name);
vm_block = 8; continue;
}
// C line 37837
10 => {
let _ = bc_put_u8(s, (((*(mi)).is_star) as u8));
vm_block = 9; continue;
}
// C line 37836
11 => {
let _ = bc_put_leb128(s, (((*(mi)).var_idx) as u32));
vm_block = 10; continue;
}
// C line 37835
12 => {
mi = core::ptr::addr_of_mut!(*((*(m)).import_entries).offset((i) as isize));
vm_block = 11; continue;
}
// C line 37834
13 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 37833
14 => {
let _ = bc_put_leb128(s, (((*(m)).import_entries_count) as u32));
vm_block = 13; continue;
}
// C line 37828
15 => {
vm_block = if ((((i) < ((*(m)).star_export_entries_count)) as i32)) != 0 { 18 } else { 14 }; continue;
}
// C line 37828
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 37830
17 => {
let _ = bc_put_leb128(s, (((*(se)).req_module_idx) as u32));
vm_block = 16; continue;
}
// C line 37829
18 => {
se = core::ptr::addr_of_mut!(*((*(m)).star_export_entries).offset((i) as isize));
vm_block = 17; continue;
}
// C line 37828
19 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 37827
20 => {
let _ = bc_put_leb128(s, (((*(m)).star_export_entries_count) as u32));
vm_block = 19; continue;
}
// C line 37815
21 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 29 } else { 20 }; continue;
}
// C line 37815
22 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 21; continue;
}
// C line 37824
23 => {
let _ = bc_put_atom(s, (*(me)).export_name);
vm_block = 22; continue;
}
// C line 37819
24 => {
let _ = bc_put_leb128(s, (((((*(me)).u).local).var_idx) as u32));
vm_block = 23; continue;
}
// C line 37822
25 => {
let _ = bc_put_atom(s, (*(me)).local_name);
vm_block = 23; continue;
}
// C line 37821
26 => {
let _ = bc_put_leb128(s, ((((*(me)).u).req_module_idx) as u32));
vm_block = 25; continue;
}
// C line 37818
27 => {
vm_block = if (((((((*(me)).export_type) as u32)) == ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0 { 24 } else { 26 }; continue;
}
// C line 37817
28 => {
let _ = bc_put_u8(s, (((*(me)).export_type) as u8));
vm_block = 27; continue;
}
// C line 37816
29 => {
me = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize));
vm_block = 28; continue;
}
// C line 37815
30 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 21; continue;
}
// C line 37814
31 => {
let _ = bc_put_leb128(s, (((*(m)).export_entries_count) as u32));
vm_block = 30; continue;
}
// C line 37807
32 => {
vm_block = if ((((i) < ((*(m)).req_module_entries_count)) as i32)) != 0 { 37 } else { 31 }; continue;
}
// C line 37807
33 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 32; continue;
}
// C line 37811
34 => {
vm_block = 1; continue;
}
// C line 37810
35 => {
vm_block = if (JS_WriteObjectRec(s, (*(rme)).attributes)) != 0 { 34 } else { 33 }; continue;
}
// C line 37809
36 => {
let _ = bc_put_atom(s, (*(rme)).module_name);
vm_block = 35; continue;
}
// C line 37808
37 => {
rme = core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((i) as isize));
vm_block = 36; continue;
}
// C line 37807
38 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 32; continue;
}
// C line 37806
39 => {
let _ = bc_put_leb128(s, (((*(m)).req_module_entries_count) as u32));
vm_block = 38; continue;
}
// C line 37804
40 => {
let _ = bc_put_atom(s, (*(m)).module_name);
vm_block = 39; continue;
}
// C line 37803
41 => {
let _ = bc_put_u8(s, (((BC_TAG_MODULE as i32)) as u8));
vm_block = 40; continue;
}
// C line 37800
42 => {
m = ((((obj).u).ptr) as *mut JSModuleDef);
vm_block = 41; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37852. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteArray(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut is_template: i32 = core::mem::zeroed();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut pr: *mut JSProperty = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 55;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37927 labels: fail
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37925
2 => {
return (0 as i32);
}
// C line 37918
3 => {
vm_block = 1; continue;
}
// C line 37917
4 => {
vm_block = if (ret) != 0 { 3 } else { 2 }; continue;
}
// C line 37916
5 => {
let _ = { let assigned = JS_WriteObjectRec(s, ((*(pr)).u).value); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 37914
6 => {
vm_block = 1; continue;
}
// C line 37913
7 => {
let _ = JS_ThrowTypeError(ctx, c"only value properties are supported".as_ptr());
vm_block = 6; continue;
}
// C line 37912
8 => {
vm_block = if ((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0 { 7 } else { 5 }; continue;
}
// C line 37922
9 => {
vm_block = 1; continue;
}
// C line 37921
10 => {
vm_block = if (ret) != 0 { 9 } else { 2 }; continue;
}
// C line 37920
11 => {
let _ = { let assigned = JS_WriteObjectRec(s, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }); ret = assigned; assigned };
vm_block = 10; continue;
}
// C line 37911
12 => {
vm_block = if !(prs).is_null() { 8 } else { 11 }; continue;
}
// C line 37910
13 => {
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr), p, (((crate::quickjs_atom::JS_ATOM_raw as i32)) as JSAtom)); prs = assigned; assigned };
vm_block = 12; continue;
}
// C line 37908
14 => {
vm_block = if (is_template) != 0 { 13 } else { 2 }; continue;
}
// C line 37880
15 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 19 } else { 14 }; continue;
}
// C line 37880
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 37883
17 => {
vm_block = 1; continue;
}
// C line 37882
18 => {
vm_block = if (ret) != 0 { 17 } else { 16 }; continue;
}
// C line 37881
19 => {
let _ = { let assigned = JS_WriteObjectRec(s, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }); ret = assigned; assigned };
vm_block = 18; continue;
}
// C line 37880
20 => {
let _ = { let assigned = (((*(p)).u).array).count; i = assigned; assigned };
vm_block = 15; continue;
}
// C line 37875
21 => {
vm_block = if ((((i) < ((((*(p)).u).array).count)) as i32)) != 0 { 25 } else { 20 }; continue;
}
// C line 37875
22 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 21; continue;
}
// C line 37878
23 => {
vm_block = 1; continue;
}
// C line 37877
24 => {
vm_block = if (ret) != 0 { 23 } else { 22 }; continue;
}
// C line 37876
25 => {
let _ = { let assigned = JS_WriteObjectRec(s, *(((((*(p)).u).array).u).values).offset((i) as isize)); ret = assigned; assigned };
vm_block = 24; continue;
}
// C line 37875
26 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 21; continue;
}
// C line 37886
27 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 43 } else { 14 }; continue;
}
// C line 37886
28 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 27; continue;
}
// C line 37900
29 => {
vm_block = 1; continue;
}
// C line 37899
30 => {
vm_block = if (ret) != 0 { 29 } else { 28 }; continue;
}
// C line 37898
31 => {
let _ = { let assigned = JS_WriteObjectRec(s, ((*(pr)).u).value); ret = assigned; assigned };
vm_block = 30; continue;
}
// C line 37896
32 => {
vm_block = 1; continue;
}
// C line 37895
33 => {
let _ = JS_ThrowTypeError(ctx, c"only value properties are supported".as_ptr());
vm_block = 32; continue;
}
// C line 37894
34 => {
vm_block = if ((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0 { 33 } else { 31 }; continue;
}
// C line 37904
35 => {
vm_block = 1; continue;
}
// C line 37903
36 => {
vm_block = if (ret) != 0 { 35 } else { 28 }; continue;
}
// C line 37902
37 => {
let _ = { let assigned = JS_WriteObjectRec(s, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }); ret = assigned; assigned };
vm_block = 36; continue;
}
// C line 37893
38 => {
vm_block = if ((((!(prs).is_null()) && (((((((*(prs)).flags()) as i32)) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0)) as i32)) != 0 { 34 } else { 37 }; continue;
}
// C line 37892
39 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 38; continue;
}
// C line 37891
40 => {
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr), p, atom); prs = assigned; assigned };
vm_block = 39; continue;
}
// C line 37890
41 => {
vm_block = 1; continue;
}
// C line 37889
42 => {
vm_block = if ((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 37888
43 => {
let _ = { let assigned = JS_NewAtomUInt32(ctx, i); atom = assigned; assigned };
vm_block = 42; continue;
}
// C line 37886
44 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 27; continue;
}
// C line 37874
45 => {
vm_block = if ((*(p)).fast_array()) != 0 { 26 } else { 44 }; continue;
}
// C line 37873
46 => {
let _ = bc_put_leb128(s, len);
vm_block = 45; continue;
}
// C line 37872
47 => {
vm_block = 1; continue;
}
// C line 37871
48 => {
vm_block = if (js_get_length32(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 47 } else { 46 }; continue;
}
// C line 37866
49 => {
let _ = { let assigned = (1 as i32); is_template = assigned; assigned };
vm_block = 48; continue;
}
// C line 37865
50 => {
let _ = bc_put_u8(s, (((BC_TAG_TEMPLATE_OBJECT as i32)) as u8));
vm_block = 49; continue;
}
// C line 37869
51 => {
let _ = { let assigned = (0 as i32); is_template = assigned; assigned };
vm_block = 48; continue;
}
// C line 37868
52 => {
let _ = bc_put_u8(s, (((BC_TAG_ARRAY as i32)) as u8));
vm_block = 51; continue;
}
// C line 37862
53 => {
vm_block = if (((((((*(s)).allow_bytecode as i32)) != 0) && (((!(((*(p)).extensible()) != 0) as i32)) != 0)) as i32)) != 0 { 50 } else { 52 }; continue;
}
// C line 37855
54 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 53; continue;
}
// C line 37854
55 => {
ctx = (*(s)).ctx;
vm_block = 54; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37930. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteObjectTag(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut prop_count: u32 = core::mem::zeroed();
let mut sh: *mut JSShape = core::mem::zeroed();
let mut pr: *mut JSShapeProperty = core::mem::zeroed();
let mut pass: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 24;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37966 labels: fail
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37964
2 => {
return (0 as i32);
}
// C line 37942
3 => {
vm_block = if ((((pass) < ((2 as i32))) as i32)) != 0 { 19 } else { 2 }; continue;
}
// C line 37942
4 => {
let _ = { let old = pass; pass = (pass).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 37945
5 => {
vm_block = if ((((i) < ((((*(sh)).prop_count) as u32))) as i32)) != 0 { 16 } else { 4 }; continue;
}
// C line 37945
6 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pr; pr = (pr).offset(1); old } };
vm_block = 5; continue;
}
// C line 37955
7 => {
let _ = { let old = prop_count; prop_count = (prop_count).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 37959
8 => {
vm_block = 1; continue;
}
// C line 37958
9 => {
vm_block = if (JS_WriteObjectRec(s, ((*((*(p)).prop).offset((i) as isize)).u).value)) != 0 { 8 } else { 6 }; continue;
}
// C line 37957
10 => {
let _ = bc_put_atom(s, atom);
vm_block = 9; continue;
}
// C line 37954
11 => {
vm_block = if ((((pass) == ((0 as i32))) as i32)) != 0 { 7 } else { 10 }; continue;
}
// C line 37952
12 => {
vm_block = 1; continue;
}
// C line 37951
13 => {
let _ = JS_ThrowTypeError((*(s)).ctx, c"only value properties are supported".as_ptr());
vm_block = 12; continue;
}
// C line 37950
14 => {
vm_block = if ((((((*(pr)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0 { 13 } else { 11 }; continue;
}
// C line 37947
15 => {
vm_block = if ((((((((((((atom) != ((((0 as i32)) as JSAtom))) as i32)) != 0) && ((JS_AtomIsString((*(s)).ctx, atom)) != 0)) as i32)) != 0) && (((((((*(pr)).flags()) as i32)) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0)) as i32)) != 0 { 14 } else { 6 }; continue;
}
// C line 37946
16 => {
let _ = { let assigned = (*(pr)).atom; atom = assigned; assigned };
vm_block = 15; continue;
}
// C line 37945
17 => {
let _ = { let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned }; { let assigned = get_shape_prop(sh); pr = assigned; assigned } };
vm_block = 5; continue;
}
// C line 37944
18 => {
let _ = bc_put_leb128(s, prop_count);
vm_block = 17; continue;
}
// C line 37943
19 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 37942
20 => {
let _ = { let assigned = (0 as i32); pass = assigned; assigned };
vm_block = 3; continue;
}
// C line 37941
21 => {
let _ = { let assigned = (*(p)).shape; sh = assigned; assigned };
vm_block = 20; continue;
}
// C line 37940
22 => {
let _ = { let assigned = (((0 as i32)) as u32); prop_count = assigned; assigned };
vm_block = 21; continue;
}
// C line 37939
23 => {
let _ = bc_put_u8(s, (((BC_TAG_OBJECT as i32)) as u8));
vm_block = 22; continue;
}
// C line 37932
24 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 23; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37969. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteTypedArray(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37980
1 => {
return (0 as i32);
}
// C line 37979
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37978
3 => {
vm_block = if (JS_WriteObjectRec(s, JSValue { u: JSValueUnion { ptr: (((*(ta)).buffer) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) })) != 0 { 2 } else { 1 }; continue;
}
// C line 37977
4 => {
let _ = bc_put_leb128(s, (*(ta)).offset);
vm_block = 3; continue;
}
// C line 37976
5 => {
let _ = bc_put_leb128(s, (((*(p)).u).array).count);
vm_block = 4; continue;
}
// C line 37975
6 => {
let _ = bc_put_u8(s, ((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as u8));
vm_block = 5; continue;
}
// C line 37974
7 => {
let _ = bc_put_u8(s, (((BC_TAG_TYPED_ARRAY as i32)) as u8));
vm_block = 6; continue;
}
// C line 37972
8 => {
ta = ((*(p)).u).typed_array;
vm_block = 7; continue;
}
// C line 37971
9 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37983. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteArrayBuffer(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37995
1 => {
return (0 as i32);
}
// C line 37994
2 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!((*(s)).dbuf), (*(abuf)).data, (((*(abuf)).byte_length) as usize));
vm_block = 1; continue;
}
// C line 37993
3 => {
let _ = bc_put_leb128(s, (((*(abuf)).max_byte_length) as u32));
vm_block = 2; continue;
}
// C line 37992
4 => {
let _ = bc_put_leb128(s, (((*(abuf)).byte_length) as u32));
vm_block = 3; continue;
}
// C line 37991
5 => {
let _ = bc_put_u8(s, (((BC_TAG_ARRAY_BUFFER as i32)) as u8));
vm_block = 4; continue;
}
// C line 37989
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37988
7 => {
let _ = JS_ThrowTypeErrorDetachedArrayBuffer((*(s)).ctx);
vm_block = 6; continue;
}
// C line 37987
8 => {
vm_block = if ((*(abuf)).detached) != 0 { 7 } else { 5 }; continue;
}
// C line 37986
9 => {
abuf = ((*(p)).u).array_buffer;
vm_block = 8; continue;
}
// C line 37985
10 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37998. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteSharedArrayBuffer(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38012
1 => {
return (0 as i32);
}
// C line 38011
2 => {
let _ = { let assigned = (*(abuf)).data; *((*(s)).sab_tab).offset(({ let old = (*(s)).sab_tab_len; (*(s)).sab_tab_len = ((*(s)).sab_tab_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 38009
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38007
4 => {
vm_block = if (js_resize_array((*(s)).ctx, ((core::ptr::addr_of_mut!((*(s)).sab_tab)) as *mut *mut c_void), (((size_of::<*mut u8>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).sab_tab_size), ((*(s)).sab_tab_len).wrapping_add((1 as i32)))) != 0 { 3 } else { 2 }; continue;
}
// C line 38006
5 => {
let _ = bc_put_u64(s, (((((*(abuf)).data) as usize)) as u64));
vm_block = 4; continue;
}
// C line 38005
6 => {
let _ = bc_put_leb128(s, (((*(abuf)).max_byte_length) as u32));
vm_block = 5; continue;
}
// C line 38004
7 => {
let _ = bc_put_leb128(s, (((*(abuf)).byte_length) as u32));
vm_block = 6; continue;
}
// C line 38003
8 => {
let _ = bc_put_u8(s, (((BC_TAG_SHARED_ARRAY_BUFFER as i32)) as u8));
vm_block = 7; continue;
}
// C line 38002
9 => {
let _ = if ((((!(((!(((*(abuf)).detached) != 0) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 8; continue;
}
// C line 38001
10 => {
abuf = ((*(p)).u).array_buffer;
vm_block = 9; continue;
}
// C line 38000
11 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38015. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteObjectRec(mut s: *mut BCWriterState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut tag: u32 = core::mem::zeroed();
let mut u: JSFloat64Union = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut p_1: *mut JSObject = core::mem::zeroed();
let mut ret_1: i32 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut vm_block: usize = 86;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38155 labels: fail
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38152
2 => {
return (0 as i32);
}
// C line 38150
3 => {
vm_block = 1; continue;
}
// C line 38149 labels: invalid_tag
4 => {
let _ = JS_ThrowInternalError_cargs((*(s)).ctx, c"unsupported tag (%d)".as_ptr(), &[ParserFormatArg::Signed((tag) as i32)]);
vm_block = 3; continue;
}
// C line 38146
5 => {
vm_block = 2; continue;
}
// C line 38145
6 => {
vm_block = 1; continue;
}
// C line 38144
7 => {
vm_block = if (JS_WriteBigInt(s, obj)) != 0 { 6 } else { 5 }; continue;
}
// C line 38141
8 => {
vm_block = 2; continue;
}
// C line 38139
9 => {
vm_block = 1; continue;
}
// C line 38138
10 => {
vm_block = if (ret_1) != 0 { 9 } else { 8 }; continue;
}
// C line 38137
11 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(p_1)).set_tmp_mark((assigned) as _); assigned };
vm_block = 10; continue;
}
// C line 38135
12 => {
vm_block = 11; continue;
}
// C line 38130
13 => {
let _ = { let assigned = JS_WriteTypedArray(s, obj); ret_1 = assigned; assigned };
vm_block = 12; continue;
}
// C line 38133
14 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); ret_1 = assigned; assigned };
vm_block = 12; continue;
}
// C line 38132
15 => {
let _ = JS_ThrowTypeError((*(s)).ctx, c"unsupported object class".as_ptr());
vm_block = 14; continue;
}
// C line 38128
16 => {
vm_block = if (((((((((((*(p_1)).class_id) as i32)) >= ((JS_CLASS_UINT8C_ARRAY as i32))) as i32)) != 0) && ((((((((*(p_1)).class_id) as i32)) <= ((JS_CLASS_FLOAT64_ARRAY as i32))) as i32)) != 0)) as i32)) != 0 { 13 } else { 15 }; continue;
}
// C line 38126
17 => {
vm_block = 11; continue;
}
// C line 38125
18 => {
let _ = { let assigned = JS_WriteObjectRec(s, ((*(p_1)).u).object_data); ret_1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 38124
19 => {
let _ = bc_put_u8(s, (((BC_TAG_OBJECT_VALUE as i32)) as u8));
vm_block = 18; continue;
}
// C line 38119
20 => {
vm_block = 11; continue;
}
// C line 38118
21 => {
let _ = { let assigned = JS_WriteObjectRec(s, ((*(p_1)).u).object_data); ret_1 = assigned; assigned };
vm_block = 20; continue;
}
// C line 38117
22 => {
let _ = bc_put_u8(s, (((BC_TAG_DATE as i32)) as u8));
vm_block = 21; continue;
}
// C line 38115
23 => {
vm_block = 11; continue;
}
// C line 38114
24 => {
let _ = { let assigned = JS_WriteSharedArrayBuffer(s, obj); ret_1 = assigned; assigned };
vm_block = 23; continue;
}
// C line 38113
25 => {
vm_block = 4; continue;
}
// C line 38112
26 => {
vm_block = if ((!((((*(s)).allow_sab as i32)) != 0) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 38110
27 => {
vm_block = 11; continue;
}
// C line 38109
28 => {
let _ = { let assigned = JS_WriteArrayBuffer(s, obj); ret_1 = assigned; assigned };
vm_block = 27; continue;
}
// C line 38107
29 => {
vm_block = 11; continue;
}
// C line 38106
30 => {
let _ = { let assigned = JS_WriteObjectTag(s, obj); ret_1 = assigned; assigned };
vm_block = 29; continue;
}
// C line 38104
31 => {
vm_block = 11; continue;
}
// C line 38103
32 => {
let _ = { let assigned = JS_WriteArray(s, obj); ret_1 = assigned; assigned };
vm_block = 31; continue;
}
// C line 38101
33 => {
vm_block = match (((*(p_1)).class_id) as i32) { x if x == (JS_CLASS_BIG_INT as i32) => 19, x if x == (JS_CLASS_BOOLEAN as i32) => 19, x if x == (JS_CLASS_STRING as i32) => 19, x if x == (JS_CLASS_NUMBER as i32) => 19, x if x == (JS_CLASS_DATE as i32) => 22, x if x == (JS_CLASS_SHARED_ARRAY_BUFFER as i32) => 26, x if x == (JS_CLASS_ARRAY_BUFFER as i32) => 28, x if x == (JS_CLASS_OBJECT as i32) => 30, x if x == (JS_CLASS_ARRAY as i32) => 32, _ => 16, }; continue;
}
// C line 38089
34 => {
vm_block = 2; continue;
}
// C line 38088
35 => {
let _ = bc_put_leb128(s, ((idx) as u32));
vm_block = 34; continue;
}
// C line 38087
36 => {
let _ = bc_put_u8(s, (((BC_TAG_OBJECT_REFERENCE as i32)) as u8));
vm_block = 35; continue;
}
// C line 38092
37 => {
vm_block = 1; continue;
}
// C line 38091
38 => {
vm_block = if (js_object_list_add((*(s)).ctx, core::ptr::addr_of_mut!((*(s)).object_list), p_1)) != 0 { 37 } else { 33 }; continue;
}
// C line 38086
39 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 36 } else { 38 }; continue;
}
// C line 38085
40 => {
let _ = { let assigned = js_object_list_find((*(s)).ctx, core::ptr::addr_of_mut!((*(s)).object_list), p_1); idx = assigned; assigned };
vm_block = 39; continue;
}
// C line 38099
41 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(p_1)).set_tmp_mark((assigned) as _); assigned };
vm_block = 33; continue;
}
// C line 38097
42 => {
vm_block = 1; continue;
}
// C line 38096
43 => {
let _ = JS_ThrowTypeError((*(s)).ctx, c"circular reference".as_ptr());
vm_block = 42; continue;
}
// C line 38095
44 => {
vm_block = if ((*(p_1)).tmp_mark()) != 0 { 43 } else { 41 }; continue;
}
// C line 38084
45 => {
vm_block = if (((*(s)).allow_reference as i32)) != 0 { 40 } else { 44 }; continue;
}
// C line 38081
46 => {
p_1 = ((((obj).u).ptr) as *mut JSObject);
vm_block = 45; continue;
}
// C line 38078
47 => {
vm_block = 2; continue;
}
// C line 38077
48 => {
vm_block = 1; continue;
}
// C line 38076
49 => {
vm_block = if (JS_WriteModule(s, obj)) != 0 { 48 } else { 47 }; continue;
}
// C line 38075
50 => {
vm_block = 4; continue;
}
// C line 38074
51 => {
vm_block = if ((!((((*(s)).allow_bytecode as i32)) != 0) as i32)) != 0 { 50 } else { 49 }; continue;
}
// C line 38072
52 => {
vm_block = 2; continue;
}
// C line 38071
53 => {
vm_block = 1; continue;
}
// C line 38070
54 => {
vm_block = if (JS_WriteFunctionTag(s, obj)) != 0 { 53 } else { 52 }; continue;
}
// C line 38069
55 => {
vm_block = 4; continue;
}
// C line 38068
56 => {
vm_block = if ((!((((*(s)).allow_bytecode as i32)) != 0) as i32)) != 0 { 55 } else { 54 }; continue;
}
// C line 38066
57 => {
vm_block = 2; continue;
}
// C line 38064
58 => {
vm_block = 1; continue;
}
// C line 38063
59 => {
vm_block = if (ret) != 0 { 58 } else { 57 }; continue;
}
// C line 38062
60 => {
let _ = JS_FreeValue((*(s)).ctx, str);
vm_block = 59; continue;
}
// C line 38061
61 => {
let _ = { let assigned = JS_WriteObjectRec(s, str); ret = assigned; assigned };
vm_block = 60; continue;
}
// C line 38060
62 => {
vm_block = 1; continue;
}
// C line 38059
63 => {
vm_block = if (JS_IsException(str)) != 0 { 62 } else { 61 }; continue;
}
// C line 38058
64 => {
let _ = { let assigned = JS_ToString((*(s)).ctx, obj); str = assigned; assigned };
vm_block = 63; continue;
}
// C line 38053
65 => {
vm_block = 2; continue;
}
// C line 38051
66 => {
let _ = JS_WriteString(s, p);
vm_block = 65; continue;
}
// C line 38050
67 => {
let _ = bc_put_u8(s, (((BC_TAG_STRING as i32)) as u8));
vm_block = 66; continue;
}
// C line 38049
68 => {
p = ((((obj).u).ptr) as *mut JSString);
vm_block = 67; continue;
}
// C line 38046
69 => {
vm_block = 2; continue;
}
// C line 38044
70 => {
let _ = bc_put_u64(s, (u).u64);
vm_block = 69; continue;
}
// C line 38043
71 => {
let _ = { let assigned = ((obj).u).float64; (u).d = assigned; assigned };
vm_block = 70; continue;
}
// C line 38042
72 => {
let _ = bc_put_u8(s, (((BC_TAG_FLOAT64 as i32)) as u8));
vm_block = 71; continue;
}
// C line 38038
73 => {
vm_block = 2; continue;
}
// C line 38037
74 => {
let _ = bc_put_sleb128(s, ((((obj).u).uint64) as i32));
vm_block = 73; continue;
}
// C line 38036
75 => {
let _ = bc_put_u8(s, (((BC_TAG_INT32 as i32)) as u8));
vm_block = 74; continue;
}
// C line 38034
76 => {
vm_block = 2; continue;
}
// C line 38033
77 => {
let _ = bc_put_u8(s, ((((BC_TAG_BOOL_FALSE as i32)).wrapping_add(((((obj).u).uint64) as i32))) as u8));
vm_block = 76; continue;
}
// C line 38031
78 => {
vm_block = 2; continue;
}
// C line 38030
79 => {
let _ = bc_put_u8(s, (((BC_TAG_UNDEFINED as i32)) as u8));
vm_block = 78; continue;
}
// C line 38028
80 => {
vm_block = 2; continue;
}
// C line 38027
81 => {
let _ = bc_put_u8(s, (((BC_TAG_NULL as i32)) as u8));
vm_block = 80; continue;
}
// C line 38025
82 => {
vm_block = match tag { x if x == (((JS_TAG_BIG_INT as i32)) as u32) => 7, x if x == (((JS_TAG_SHORT_BIG_INT as i32)) as u32) => 7, x if x == (((JS_TAG_OBJECT as i32)) as u32) => 46, x if x == (((JS_TAG_MODULE as i32)) as u32) => 51, x if x == (((JS_TAG_FUNCTION_BYTECODE as i32)) as u32) => 56, x if x == (((JS_TAG_STRING_ROPE as i32)) as u32) => 64, x if x == (((JS_TAG_STRING as i32)) as u32) => 68, x if x == (((JS_TAG_FLOAT64 as i32)) as u32) => 72, x if x == (((JS_TAG_INT as i32)) as u32) => 75, x if x == (((JS_TAG_BOOL as i32)) as u32) => 77, x if x == (((JS_TAG_UNDEFINED as i32)) as u32) => 79, x if x == (((JS_TAG_NULL as i32)) as u32) => 81, _ => 4, }; continue;
}
// C line 38024
83 => {
let _ = { let assigned = (((((obj).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 82; continue;
}
// C line 38021
84 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38020
85 => {
let _ = JS_ThrowStackOverflow((*(s)).ctx);
vm_block = 84; continue;
}
// C line 38019
86 => {
vm_block = if (js_check_stack_overflow((*((*(s)).ctx)).rt, (((0 as i32)) as usize))) != 0 { 85 } else { 83 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38159. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_WriteObjectAtoms(mut s: *mut BCWriterState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut dbuf1: DynBuf = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut atoms_size: i32 = core::mem::zeroed();
let mut p: *mut JSAtomStruct = core::mem::zeroed();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38190
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38189 labels: fail
2 => {
let _ = crate::cutils::dbuf_free(core::ptr::addr_of_mut!(dbuf1));
vm_block = 1; continue;
}
// C line 38187
3 => {
return (0 as i32);
}
// C line 38186
4 => {
let _ = { (*(s)).dbuf = core::ptr::read(core::ptr::addr_of!(dbuf1)); core::ptr::read(core::ptr::addr_of!((*(s)).dbuf)) };
vm_block = 3; continue;
}
// C line 38185
5 => {
let _ = crate::cutils::dbuf_free(core::ptr::addr_of_mut!((*(s)).dbuf));
vm_block = 4; continue;
}
// C line 38184
6 => {
let _ = { (dbuf1).size = (((((dbuf1).size) as usize)).wrapping_add(((atoms_size) as usize))) as usize; (dbuf1).size };
vm_block = 5; continue;
}
// C line 38183
7 => {
let _ = { let dst = ((((dbuf1).buf) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(s)).dbuf).buf) as *const c_void)) as *const u8, dst, (((atoms_size) as usize)) as usize); dst as *mut c_void };
vm_block = 6; continue;
}
// C line 38182
8 => {
let _ = { let dst = (((((dbuf1).buf).offset(((atoms_size) as isize))) as *mut c_void)) as *mut u8; core::ptr::copy(((((dbuf1).buf) as *const c_void)) as *const u8, dst, ((dbuf1).size) as usize); dst as *mut c_void };
vm_block = 7; continue;
}
// C line 38181
9 => {
vm_block = 2; continue;
}
// C line 38180
10 => {
vm_block = if (crate::cutils::dbuf_claim(core::ptr::addr_of_mut!(dbuf1), ((atoms_size) as usize))) != 0 { 9 } else { 8 }; continue;
}
// C line 38179
11 => {
let _ = { let assigned = ((((*(s)).dbuf).size) as i32); atoms_size = assigned; assigned };
vm_block = 10; continue;
}
// C line 38170
12 => {
vm_block = if ((((i) < ((*(s)).idx_to_atom_count)) as i32)) != 0 { 15 } else { 11 }; continue;
}
// C line 38170
13 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 38172
14 => {
let _ = JS_WriteString(s, p);
vm_block = 13; continue;
}
// C line 38171
15 => {
p = *((*(rt)).atom_array).offset((*((*(s)).idx_to_atom).offset((i) as isize)) as isize);
vm_block = 14; continue;
}
// C line 38170
16 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 12; continue;
}
// C line 38169
17 => {
let _ = bc_put_leb128(s, (((*(s)).idx_to_atom_count) as u32));
vm_block = 16; continue;
}
// C line 38167
18 => {
let _ = bc_put_u8(s, (((5 as i32)) as u8));
vm_block = 17; continue;
}
// C line 38166
19 => {
let _ = js_dbuf_init((*(s)).ctx, core::ptr::addr_of_mut!((*(s)).dbuf));
vm_block = 18; continue;
}
// C line 38165
20 => {
let _ = { dbuf1 = core::ptr::read(core::ptr::addr_of!((*(s)).dbuf)); core::ptr::read(core::ptr::addr_of!(dbuf1)) };
vm_block = 19; continue;
}
// C line 38161
21 => {
rt = (*((*(s)).ctx)).rt;
vm_block = 20; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38193. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_WriteObject2(mut ctx: *mut JSContext, mut psize: *mut usize, mut obj: JSValue, mut flags: i32, mut psab_tab: *mut *mut *mut u8, mut psab_tab_len: *mut usize) -> *mut u8 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ss: BCWriterState = core::mem::zeroed();
let mut s: *mut BCWriterState = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38234
1 => {
return core::ptr::null_mut::<u8>();
}
// C line 38233
2 => {
let _ = { let assigned = (((0 as i32)) as usize); *(psab_tab_len) = assigned; assigned };
vm_block = 1; continue;
}
// C line 38232
3 => {
vm_block = if !(psab_tab_len).is_null() { 2 } else { 1 }; continue;
}
// C line 38231
4 => {
let _ = { let assigned = core::ptr::null_mut::<*mut u8>(); *(psab_tab) = assigned; assigned };
vm_block = 3; continue;
}
// C line 38230
5 => {
vm_block = if !(psab_tab).is_null() { 4 } else { 3 }; continue;
}
// C line 38229
6 => {
let _ = { let assigned = (((0 as i32)) as usize); *(psize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 38228
7 => {
let _ = crate::cutils::dbuf_free(core::ptr::addr_of_mut!((*(s)).dbuf));
vm_block = 6; continue;
}
// C line 38227
8 => {
let _ = js_free(ctx, (((*(s)).idx_to_atom) as *mut c_void));
vm_block = 7; continue;
}
// C line 38226
9 => {
let _ = js_free(ctx, (((*(s)).atom_to_idx) as *mut c_void));
vm_block = 8; continue;
}
// C line 38225 labels: fail
10 => {
let _ = js_object_list_end(ctx, core::ptr::addr_of_mut!((*(s)).object_list));
vm_block = 9; continue;
}
// C line 38223
11 => {
return ((*(s)).dbuf).buf;
}
// C line 38222
12 => {
let _ = { let assigned = (((*(s)).sab_tab_len) as usize); *(psab_tab_len) = assigned; assigned };
vm_block = 11; continue;
}
// C line 38221
13 => {
vm_block = if !(psab_tab_len).is_null() { 12 } else { 11 }; continue;
}
// C line 38220
14 => {
let _ = { let assigned = (*(s)).sab_tab; *(psab_tab) = assigned; assigned };
vm_block = 13; continue;
}
// C line 38219
15 => {
vm_block = if !(psab_tab).is_null() { 14 } else { 13 }; continue;
}
// C line 38218
16 => {
let _ = { let assigned = ((*(s)).dbuf).size; *(psize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 38217
17 => {
let _ = js_free(ctx, (((*(s)).idx_to_atom) as *mut c_void));
vm_block = 16; continue;
}
// C line 38216
18 => {
let _ = js_free(ctx, (((*(s)).atom_to_idx) as *mut c_void));
vm_block = 17; continue;
}
// C line 38215
19 => {
let _ = js_object_list_end(ctx, core::ptr::addr_of_mut!((*(s)).object_list));
vm_block = 18; continue;
}
// C line 38214
20 => {
vm_block = 10; continue;
}
// C line 38213
21 => {
vm_block = if (JS_WriteObjectAtoms(s)) != 0 { 20 } else { 19 }; continue;
}
// C line 38212
22 => {
vm_block = 10; continue;
}
// C line 38211
23 => {
vm_block = if (JS_WriteObjectRec(s, obj)) != 0 { 22 } else { 21 }; continue;
}
// C line 38209
24 => {
let _ = js_object_list_init(core::ptr::addr_of_mut!((*(s)).object_list));
vm_block = 23; continue;
}
// C line 38208
25 => {
let _ = js_dbuf_init(ctx, core::ptr::addr_of_mut!((*(s)).dbuf));
vm_block = 24; continue;
}
// C line 38205
26 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM_END as i32)) as u32); (*(s)).first_atom = assigned; assigned };
vm_block = 25; continue;
}
// C line 38207
27 => {
let _ = { let assigned = (((1 as i32)) as u32); (*(s)).first_atom = assigned; assigned };
vm_block = 25; continue;
}
// C line 38204
28 => {
vm_block = if (((*(s)).allow_bytecode as i32)) != 0 { 26 } else { 27 }; continue;
}
// C line 38202
29 => {
let _ = { let assigned = (((((flags) & (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))) != ((0 as i32))) as i32); (*(s)).allow_reference = (assigned) as i8; assigned };
vm_block = 28; continue;
}
// C line 38201
30 => {
let _ = { let assigned = (((((flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != ((0 as i32))) as i32); (*(s)).allow_sab = (assigned) as i8; assigned };
vm_block = 29; continue;
}
// C line 38200
31 => {
let _ = { let assigned = (((((flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != ((0 as i32))) as i32); (*(s)).allow_bytecode = (assigned) as i8; assigned };
vm_block = 30; continue;
}
// C line 38199
32 => {
let _ = { let assigned = ctx; (*(s)).ctx = assigned; assigned };
vm_block = 31; continue;
}
// C line 38198
33 => {
let _ = { let dst = (((s) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<BCWriterState>() as usize)) as usize); dst as *mut c_void };
vm_block = 32; continue;
}
// C line 38196
34 => {
s = core::ptr::addr_of_mut!(ss);
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38237. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_WriteObject(mut ctx: *mut JSContext, mut psize: *mut usize, mut obj: JSValue, mut flags: i32) -> *mut u8 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38240
1 => {
return JS_WriteObject2(ctx, psize, obj, flags, core::ptr::null_mut::<*mut *mut u8>(), core::ptr::null_mut::<usize>());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38300. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_read_error_end(mut s: *mut BCReaderState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38305
1 => {
return { let assigned = ((1 as i32)).wrapping_neg(); (*(s)).error_state = assigned; assigned };
}
// C line 38303
2 => {
let _ = JS_ThrowSyntaxError((*(s)).ctx, c"read after the end of the buffer".as_ptr());
vm_block = 1; continue;
}
// C line 38302
3 => {
vm_block = if ((!(((*(s)).error_state) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38308. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_u8(mut s: *mut BCReaderState, mut pval: *mut u8) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38315
1 => {
return (0 as i32);
}
// C line 38314
2 => {
let _ = { let assigned = *({ let old = (*(s)).ptr; (*(s)).ptr = ((*(s)).ptr).offset(1); old }); *(pval) = assigned; assigned };
vm_block = 1; continue;
}
// C line 38312
3 => {
return bc_read_error_end(s);
}
// C line 38311
4 => {
let _ = { let assigned = (((0 as i32)) as u8); *(pval) = assigned; assigned };
vm_block = 3; continue;
}
// C line 38310
5 => {
vm_block = if ((((!(((!((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) < ((((1 as i32)) as i64))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 4 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38318. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_u16(mut s: *mut BCReaderState, mut pval: *mut u16) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: u16 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38330
1 => {
return (0 as i32);
}
// C line 38329
2 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset((((2 as i32)) as isize))) as *const u8; (*(s)).ptr };
vm_block = 1; continue;
}
// C line 38328
3 => {
let _ = { let assigned = v; *(pval) = assigned; assigned };
vm_block = 2; continue;
}
// C line 38327
4 => {
let _ = { let assigned = crate::cutils_header::bswap16(v); v = assigned; assigned };
vm_block = 3; continue;
}
// C line 38326
5 => {
vm_block = if (is_be()) != 0 { 4 } else { 3 }; continue;
}
// C line 38325
6 => {
let _ = { let assigned = ((crate::cutils_header::get_u16((*(s)).ptr)) as u16); v = assigned; assigned };
vm_block = 5; continue;
}
// C line 38323
7 => {
return bc_read_error_end(s);
}
// C line 38322
8 => {
let _ = { let assigned = (((0 as i32)) as u16); *(pval) = assigned; assigned };
vm_block = 7; continue;
}
// C line 38321
9 => {
vm_block = if ((((!(((!((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) < ((((2 as i32)) as i64))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 8 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38333. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_u32(mut s: *mut BCReaderState, mut pval: *mut u32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38345
1 => {
return (0 as i32);
}
// C line 38344
2 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset((((4 as i32)) as isize))) as *const u8; (*(s)).ptr };
vm_block = 1; continue;
}
// C line 38343
3 => {
let _ = { let assigned = v; *(pval) = assigned; assigned };
vm_block = 2; continue;
}
// C line 38342
4 => {
let _ = { let assigned = crate::cutils_header::bswap32(v); v = assigned; assigned };
vm_block = 3; continue;
}
// C line 38341
5 => {
vm_block = if (is_be()) != 0 { 4 } else { 3 }; continue;
}
// C line 38340
6 => {
let _ = { let assigned = crate::cutils_header::get_u32((*(s)).ptr); v = assigned; assigned };
vm_block = 5; continue;
}
// C line 38338
7 => {
return bc_read_error_end(s);
}
// C line 38337
8 => {
let _ = { let assigned = (((0 as i32)) as u32); *(pval) = assigned; assigned };
vm_block = 7; continue;
}
// C line 38336
9 => {
vm_block = if ((((!(((!((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) < ((((4 as i32)) as i64))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 8 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38348. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_u64(mut s: *mut BCReaderState, mut pval: *mut u64) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: u64 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38360
1 => {
return (0 as i32);
}
// C line 38359
2 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset((((8 as i32)) as isize))) as *const u8; (*(s)).ptr };
vm_block = 1; continue;
}
// C line 38358
3 => {
let _ = { let assigned = v; *(pval) = assigned; assigned };
vm_block = 2; continue;
}
// C line 38357
4 => {
let _ = { let assigned = crate::cutils_header::bswap64(v); v = assigned; assigned };
vm_block = 3; continue;
}
// C line 38356
5 => {
vm_block = if (is_be()) != 0 { 4 } else { 3 }; continue;
}
// C line 38355
6 => {
let _ = { let assigned = crate::cutils_header::get_u64((*(s)).ptr); v = assigned; assigned };
vm_block = 5; continue;
}
// C line 38353
7 => {
return bc_read_error_end(s);
}
// C line 38352
8 => {
let _ = { let assigned = (((0 as i32)) as u64); *(pval) = assigned; assigned };
vm_block = 7; continue;
}
// C line 38351
9 => {
vm_block = if ((((!(((!((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) < ((((8 as i32)) as i64))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 8 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38363. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_leb128(s: *mut BCReaderState, pval: *mut u32) -> i32 {
    // quickjs.c:38363-38370: same decoder, error boundary and pointer update.
    let length = get_leb128(pval, (*s).ptr, (*s).buf_end);
    if length < 0 { return bc_read_error_end(s); }
    (*s).ptr = (*s).ptr.offset(length as isize);
    0
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38373. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_sleb128(mut s: *mut BCReaderState, mut pval: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38380
1 => {
return (0 as i32);
}
// C line 38379
2 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset(((ret) as isize))) as *const u8; (*(s)).ptr };
vm_block = 1; continue;
}
// C line 38378
3 => {
return bc_read_error_end(s);
}
// C line 38377
4 => {
vm_block = if ((((!(((!(((((ret) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 3 } else { 2 }; continue;
}
// C line 38376
5 => {
let _ = { let assigned = get_sleb128(pval, (*(s)).ptr, (*(s)).buf_end); ret = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38384. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_leb128_int(mut s: *mut BCReaderState, mut pval: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38386
1 => {
return bc_get_leb128(s, ((pval) as *mut u32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38389. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_leb128_u16(mut s: *mut BCReaderState, mut pval: *mut u16) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: u32 = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38397
1 => {
return (0 as i32);
}
// C line 38396
2 => {
let _ = { let assigned = ((val) as u16); *(pval) = assigned; assigned };
vm_block = 1; continue;
}
// C line 38394
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38393
4 => {
let _ = { let assigned = (((0 as i32)) as u16); *(pval) = assigned; assigned };
vm_block = 3; continue;
}
// C line 38392
5 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(val))) != 0 { 4 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38400. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_buf(mut s: *mut BCReaderState, mut buf: *mut u8, mut buf_len: u32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38408
1 => {
return (0 as i32);
}
// C line 38406
2 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset(((buf_len) as isize))) as *const u8; (*(s)).ptr };
vm_block = 1; continue;
}
// C line 38405
3 => {
let _ = { let dst = (((buf) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping(((((*(s)).ptr) as *const c_void)) as *const u8, dst, (((buf_len) as usize)) as usize); dst as *mut c_void };
vm_block = 2; continue;
}
// C line 38404
4 => {
return bc_read_error_end(s);
}
// C line 38403
5 => {
vm_block = if ((((!(((!(((((((!(!(buf).is_null()) as i32)) != 0) || ((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) < (((buf_len) as i64))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 4 } else { 3 }; continue;
}
// C line 38402
6 => {
vm_block = if ((((buf_len) != ((((0 as i32)) as u32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38411. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_idx_to_atom(s: *mut BCReaderState, patom: *mut JSAtom, mut idx: u32) -> i32 {
    // quickjs.c:38410-38430: keep tagged/immediate/table atom order and every
    // reference increment, including the original error_state/patom writes.
    let atom = if __JS_AtomIsTaggedInt(idx) != 0 {
        idx
    } else if idx < (*s).first_atom {
        JS_DupAtom((*s).ctx, idx)
    } else {
        idx = idx.wrapping_sub((*s).first_atom);
        if idx >= (*s).idx_to_atom_count {
            JS_ThrowSyntaxError_cargs((*s).ctx, c"invalid atom index (pos=%u)".as_ptr(),
                &[ParserFormatArg::Unsigned((*s).ptr.offset_from((*s).buf_start) as u32)]);
            *patom = 0;
            (*s).error_state = -1;
            return -1;
        }
        JS_DupAtom((*s).ctx, *(*s).idx_to_atom.offset(idx as isize))
    };
    *patom = atom;
    0
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38433. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_atom(mut s: *mut BCReaderState, mut patom: *mut JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38440
1 => {
return (0 as i32);
}
// C line 38439
2 => {
let _ = { let assigned = __JS_AtomFromUInt32((v).wrapping_shr(((1 as i32)) as u32)); *(patom) = assigned; assigned };
vm_block = 1; continue;
}
// C line 38442
3 => {
return bc_idx_to_atom(s, patom, (v).wrapping_shr(((1 as i32)) as u32));
}
// C line 38438
4 => {
vm_block = if (((v) & ((((1 as i32)) as u32)))) != 0 { 2 } else { 3 }; continue;
}
// C line 38437
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38436
6 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(v))) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(not(feature="dump-read-object"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38446. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadString(mut s: *mut BCReaderState) -> *mut JSString {
let mut vm_local_storage = Vec::<u64>::new();
let mut len: u32 = core::mem::zeroed();
let mut size: usize = core::mem::zeroed();
let mut is_wide_char: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38486
1 => {
return p;
}
// C line 38477
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 38477
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 38478
4 => {
let _ = { let assigned = crate::cutils_header::bswap16(*((((*(p)).u).str16).as_mut_ptr()).offset((i) as isize)); *((((*(p)).u).str16).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 38477
5 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 38475
6 => {
vm_block = if (is_be()) != 0 { 5 } else { 1 }; continue;
}
// C line 38481
7 => {
let _ = { let assigned = (((0 as i32)) as u8); *((((*(p)).u).str8).as_mut_ptr()).offset((size) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 38474
8 => {
vm_block = if (is_wide_char) != 0 { 6 } else { 7 }; continue;
}
// C line 38473
9 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset(((size) as isize))) as *const u8; (*(s)).ptr };
vm_block = 8; continue;
}
// C line 38472
10 => {
let _ = { let dst = ((((((*(p)).u).str8).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping(((((*(s)).ptr) as *const c_void)) as *const u8, dst, (size) as usize); dst as *mut c_void };
vm_block = 9; continue;
}
// C line 38470
11 => {
return core::ptr::null_mut::<JSString>();
}
// C line 38469
12 => {
let _ = js_free_string((*((*(s)).ctx)).rt, p);
vm_block = 11; continue;
}
// C line 38468
13 => {
let _ = bc_read_error_end(s);
vm_block = 12; continue;
}
// C line 38467
14 => {
vm_block = if (((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) as usize)) < (size)) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line 38466
15 => {
let _ = { let assigned = (((len) as usize)).wrapping_shl((is_wide_char) as u32); size = assigned; assigned };
vm_block = 14; continue;
}
// C line 38464
16 => {
return core::ptr::null_mut::<JSString>();
}
// C line 38463
17 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(s)).error_state = assigned; assigned };
vm_block = 16; continue;
}
// C line 38462
18 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 17 } else { 15 }; continue;
}
// C line 38461
19 => {
let _ = { let assigned = js_alloc_string((*(s)).ctx, ((len) as i32), is_wide_char); p = assigned; assigned };
vm_block = 18; continue;
}
// C line 38459
20 => {
return core::ptr::null_mut::<JSString>();
}
// C line 38458
21 => {
let _ = JS_ThrowInternalError((*(s)).ctx, c"string too long".as_ptr());
vm_block = 20; continue;
}
// C line 38457
22 => {
vm_block = if ((((len) > ((((((1 as i32)).wrapping_shl(((30 as i32)) as u32)).wrapping_sub((1 as i32))) as u32))) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 38456
23 => {
let _ = { len = ((((len) as u32)).wrapping_shr(((1 as i32)) as u32)) as u32; len };
vm_block = 22; continue;
}
// C line 38455
24 => {
let _ = { let assigned = ((((len) & ((((1 as i32)) as u32)))) as i32); is_wide_char = assigned; assigned };
vm_block = 23; continue;
}
// C line 38454
25 => {
return core::ptr::null_mut::<JSString>();
}
// C line 38453
26 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(len))) != 0 { 25 } else { 24 }; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature="dump-read-object")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38446. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadString(mut s: *mut BCReaderState) -> *mut JSString {
let mut vm_local_storage = Vec::<u64>::new();
let mut len: u32 = core::mem::zeroed();
let mut size: usize = core::mem::zeroed();
let mut is_wide_char: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38486
1 => {
return p;
}
// C line 38484
2 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 1; continue;
}
// C line 38484
3 => {
let _ = JS_DumpString((*((*(s)).ctx)).rt, p);
vm_block = 2; continue;
}
// C line 38477
4 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line 38477
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 38478
6 => {
let _ = { let assigned = crate::cutils_header::bswap16(*((((*(p)).u).str16).as_mut_ptr()).offset((i) as isize)); *((((*(p)).u).str16).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 38477
7 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 38475
8 => {
vm_block = if (is_be()) != 0 { 7 } else { 3 }; continue;
}
// C line 38481
9 => {
let _ = { let assigned = (((0 as i32)) as u8); *((((*(p)).u).str8).as_mut_ptr()).offset((size) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 38474
10 => {
vm_block = if (is_wide_char) != 0 { 8 } else { 9 }; continue;
}
// C line 38473
11 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset(((size) as isize))) as *const u8; (*(s)).ptr };
vm_block = 10; continue;
}
// C line 38472
12 => {
let _ = { let dst = ((((((*(p)).u).str8).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping(((((*(s)).ptr) as *const c_void)) as *const u8, dst, (size) as usize); dst as *mut c_void };
vm_block = 11; continue;
}
// C line 38470
13 => {
return core::ptr::null_mut::<JSString>();
}
// C line 38469
14 => {
let _ = js_free_string((*((*(s)).ctx)).rt, p);
vm_block = 13; continue;
}
// C line 38468
15 => {
let _ = bc_read_error_end(s);
vm_block = 14; continue;
}
// C line 38467
16 => {
vm_block = if (((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) as usize)) < (size)) as i32)) != 0 { 15 } else { 12 }; continue;
}
// C line 38466
17 => {
let _ = { let assigned = (((len) as usize)).wrapping_shl((is_wide_char) as u32); size = assigned; assigned };
vm_block = 16; continue;
}
// C line 38464
18 => {
return core::ptr::null_mut::<JSString>();
}
// C line 38463
19 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(s)).error_state = assigned; assigned };
vm_block = 18; continue;
}
// C line 38462
20 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 19 } else { 17 }; continue;
}
// C line 38461
21 => {
let _ = { let assigned = js_alloc_string((*(s)).ctx, ((len) as i32), is_wide_char); p = assigned; assigned };
vm_block = 20; continue;
}
// C line 38459
22 => {
return core::ptr::null_mut::<JSString>();
}
// C line 38458
23 => {
let _ = JS_ThrowInternalError((*(s)).ctx, c"string too long".as_ptr());
vm_block = 22; continue;
}
// C line 38457
24 => {
vm_block = if ((((len) > ((((((1 as i32)).wrapping_shl(((30 as i32)) as u32)).wrapping_sub((1 as i32))) as u32))) as i32)) != 0 { 23 } else { 21 }; continue;
}
// C line 38456
25 => {
let _ = { len = ((((len) as u32)).wrapping_shr(((1 as i32)) as u32)) as u32; len };
vm_block = 24; continue;
}
// C line 38455
26 => {
let _ = { let assigned = ((((len) & ((((1 as i32)) as u32)))) as i32); is_wide_char = assigned; assigned };
vm_block = 25; continue;
}
// C line 38454
27 => {
return core::ptr::null_mut::<JSString>();
}
// C line 38453
28 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(len))) != 0 { 27 } else { 26 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38489. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_get_flags(flags: u32, pidx: *mut i32, n: i32) -> u32 {
    // quickjs.c:38489-38495: fuse the three sequential CFG blocks.
    let value = flags.wrapping_shr(*pidx as u32) & 1u32.wrapping_shl(n as u32).wrapping_sub(1);
    *pidx = (*pidx).wrapping_add(n);
    value
}

#[cfg(not(feature="dump-read-object"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38498. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadFunctionBytecode(mut s: *mut BCReaderState, mut b: *mut JSFunctionBytecode, mut byte_code_offset: i32, mut bc_len: u32) -> i32 {
let mut bc_buf: *mut u8 = core::mem::zeroed();
let mut pos: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut op: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut idx: u32 = core::mem::zeroed();
let mut vm_block: usize = 27;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38553
1 => {
return (0 as i32);
}
// C line 38523
2 => {
// Local CFG fusion of quickjs.c:38522-38551: the original opcode loop,
// with unchanged atom ownership and partial-bytecode cleanup on failure.
while (pos as u32) < bc_len {
    op = *bc_buf.offset(pos as isize) as i32;
    let info_index = if op >= OP_TEMP_START as i32 {
        op.wrapping_add((OP_TEMP_END as i32).wrapping_sub(OP_TEMP_START as i32))
    } else { op };
    let info = &*opcode_info.as_ptr().offset(info_index as isize);
    len = info.size as i32;
    let format = info.fmt as i32;
    if format == OP_FMT_atom as i32 || format == OP_FMT_atom_u8 as i32
        || format == OP_FMT_atom_u16 as i32 || format == OP_FMT_atom_label_u8 as i32
        || format == OP_FMT_atom_label_u16 as i32 {
        idx = crate::cutils_header::get_u32(bc_buf.offset(pos as isize).offset(1));
        if (*s).is_rom_data != 0 {
            JS_DupAtom((*s).ctx, idx as JSAtom);
        } else {
            if bc_idx_to_atom(s, core::ptr::addr_of_mut!(atom), idx) != 0 {
                (*b).byte_code_len = pos;
                return -1;
            }
            crate::cutils_header::put_u32(bc_buf.offset(pos as isize).offset(1), atom);
        }
    }
    pos = pos.wrapping_add(len);
}
return 0;
}
// C line 38551
3 => {
let _ = { pos = ((((pos) as i32)).wrapping_add(len)) as i32; pos };
vm_block = 2; continue;
}
// C line 38549
4 => {
vm_block = 3; continue;
}
// C line 38547
5 => {
vm_block = 3; continue;
}
// C line 38535
6 => {
let _ = JS_DupAtom((*(s)).ctx, ((idx) as JSAtom));
vm_block = 5; continue;
}
// C line 38542
7 => {
let _ = crate::cutils_header::put_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), atom);
vm_block = 5; continue;
}
// C line 38540
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38539
9 => {
let _ = { let assigned = pos; (*(b)).byte_code_len = assigned; assigned };
vm_block = 8; continue;
}
// C line 38537
10 => {
vm_block = if (bc_idx_to_atom(s, core::ptr::addr_of_mut!(atom), idx)) != 0 { 9 } else { 7 }; continue;
}
// C line 38533
11 => {
vm_block = if (((*(s)).is_rom_data as i32)) != 0 { 6 } else { 10 }; continue;
}
// C line 38532
12 => {
let _ = { let assigned = crate::cutils_header::get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); idx = assigned; assigned };
vm_block = 11; continue;
}
// C line 38526
13 => {
vm_block = match (((*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)).fmt) as i32) { x if x == (OP_FMT_atom_label_u16 as i32) => 12, x if x == (OP_FMT_atom_label_u8 as i32) => 12, x if x == (OP_FMT_atom_u16 as i32) => 12, x if x == (OP_FMT_atom_u8 as i32) => 12, x if x == (OP_FMT_atom as i32) => 12, _ => 4, }; continue;
}
// C line 38525
14 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)).size) as i32); len = assigned; assigned };
vm_block = 13; continue;
}
// C line 38524
15 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 14; continue;
}
// C line 38522
16 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 2; continue;
}
// C line 38520
17 => {
let _ = bc_byte_swap(bc_buf, ((bc_len) as i32));
vm_block = 16; continue;
}
// C line 38519
18 => {
vm_block = if (is_be()) != 0 { 17 } else { 16 }; continue;
}
// C line 38517
19 => {
let _ = { let assigned = bc_buf; (*(b)).byte_code_buf = assigned; assigned };
vm_block = 18; continue;
}
// C line 38511
20 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset(((bc_len) as isize))) as *const u8; (*(s)).ptr };
vm_block = 19; continue;
}
// C line 38510
21 => {
let _ = { let assigned = (((*(s)).ptr) as *mut u8); bc_buf = assigned; assigned };
vm_block = 20; continue;
}
// C line 38509
22 => {
return bc_read_error_end(s);
}
// C line 38508
23 => {
vm_block = if ((((!(((!((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) < (((bc_len) as i64))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 22 } else { 21 }; continue;
}
// C line 38515
24 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38514
25 => {
vm_block = if (bc_get_buf(s, bc_buf, bc_len)) != 0 { 24 } else { 19 }; continue;
}
// C line 38513
26 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((byte_code_offset) as isize))) as *mut c_void)) as *mut u8); bc_buf = assigned; assigned };
vm_block = 25; continue;
}
// C line 38506
27 => {
vm_block = if (((*(s)).is_rom_data as i32)) != 0 { 23 } else { 26 }; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature="dump-read-object")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38498. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadFunctionBytecode(mut s: *mut BCReaderState, mut b: *mut JSFunctionBytecode, mut byte_code_offset: i32, mut bc_len: u32) -> i32 {
let mut bc_buf: *mut u8 = core::mem::zeroed();
let mut pos: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut op: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut idx: u32 = core::mem::zeroed();
let mut vm_block: usize = 30;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38553
1 => {
return (0 as i32);
}
// C line 38523
2 => {
// Local CFG fusion of quickjs.c:38522-38551: the original opcode loop,
// with unchanged atom ownership and partial-bytecode cleanup on failure.
while (pos as u32) < bc_len {
    op = *bc_buf.offset(pos as isize) as i32;
    let info_index = if op >= OP_TEMP_START as i32 {
        op.wrapping_add((OP_TEMP_END as i32).wrapping_sub(OP_TEMP_START as i32))
    } else { op };
    let info = &*opcode_info.as_ptr().offset(info_index as isize);
    len = info.size as i32;
    let format = info.fmt as i32;
    if format == OP_FMT_atom as i32 || format == OP_FMT_atom_u8 as i32
        || format == OP_FMT_atom_u16 as i32 || format == OP_FMT_atom_label_u8 as i32
        || format == OP_FMT_atom_label_u16 as i32 {
        idx = crate::cutils_header::get_u32(bc_buf.offset(pos as isize).offset(1));
        if (*s).is_rom_data != 0 {
            JS_DupAtom((*s).ctx, idx as JSAtom);
        } else {
            if bc_idx_to_atom(s, core::ptr::addr_of_mut!(atom), idx) != 0 {
                (*b).byte_code_len = pos;
                return -1;
            }
            crate::cutils_header::put_u32(bc_buf.offset(pos as isize).offset(1), atom);
            bc_read_trace(s, c"at %d, fixup atom: ".as_ptr(), &[QuickJSPrintArg::Int(pos.wrapping_add(1) as u64)]);
            print_atom((*s).ctx, atom);
            quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
        }
    }
    pos = pos.wrapping_add(len);
}
return 0;
}
// C line 38551
3 => {
let _ = { pos = ((((pos) as i32)).wrapping_add(len)) as i32; pos };
vm_block = 2; continue;
}
// C line 38549
4 => {
vm_block = 3; continue;
}
// C line 38547
5 => {
vm_block = 3; continue;
}
// C line 38535
6 => {
let _ = JS_DupAtom((*(s)).ctx, ((idx) as JSAtom));
vm_block = 5; continue;
}
// C line 38544
7 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 5; continue;
}
// C line 38544
8 => {
let _ = print_atom((*(s)).ctx, atom);
vm_block = 7; continue;
}
// C line 38544
9 => {
let _ = bc_read_trace(s, c"at %d, fixup atom: ".as_ptr(), &[QuickJSPrintArg::Int((pos).wrapping_add((1 as i32)) as u64)]);
vm_block = 8; continue;
}
// C line 38542
10 => {
let _ = crate::cutils_header::put_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), atom);
vm_block = 9; continue;
}
// C line 38540
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38539
12 => {
let _ = { let assigned = pos; (*(b)).byte_code_len = assigned; assigned };
vm_block = 11; continue;
}
// C line 38537
13 => {
vm_block = if (bc_idx_to_atom(s, core::ptr::addr_of_mut!(atom), idx)) != 0 { 12 } else { 10 }; continue;
}
// C line 38533
14 => {
vm_block = if (((*(s)).is_rom_data as i32)) != 0 { 6 } else { 13 }; continue;
}
// C line 38532
15 => {
let _ = { let assigned = crate::cutils_header::get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); idx = assigned; assigned };
vm_block = 14; continue;
}
// C line 38526
16 => {
vm_block = match (((*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)).fmt) as i32) { x if x == (OP_FMT_atom_label_u16 as i32) => 15, x if x == (OP_FMT_atom_label_u8 as i32) => 15, x if x == (OP_FMT_atom_u16 as i32) => 15, x if x == (OP_FMT_atom_u8 as i32) => 15, x if x == (OP_FMT_atom as i32) => 15, _ => 4, }; continue;
}
// C line 38525
17 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)).size) as i32); len = assigned; assigned };
vm_block = 16; continue;
}
// C line 38524
18 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 17; continue;
}
// C line 38522
19 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 2; continue;
}
// C line 38520
20 => {
let _ = bc_byte_swap(bc_buf, ((bc_len) as i32));
vm_block = 19; continue;
}
// C line 38519
21 => {
vm_block = if (is_be()) != 0 { 20 } else { 19 }; continue;
}
// C line 38517
22 => {
let _ = { let assigned = bc_buf; (*(b)).byte_code_buf = assigned; assigned };
vm_block = 21; continue;
}
// C line 38511
23 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset(((bc_len) as isize))) as *const u8; (*(s)).ptr };
vm_block = 22; continue;
}
// C line 38510
24 => {
let _ = { let assigned = (((*(s)).ptr) as *mut u8); bc_buf = assigned; assigned };
vm_block = 23; continue;
}
// C line 38509
25 => {
return bc_read_error_end(s);
}
// C line 38508
26 => {
vm_block = if ((((!(((!((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) < (((bc_len) as i64))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 25 } else { 24 }; continue;
}
// C line 38515
27 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38514
28 => {
vm_block = if (bc_get_buf(s, bc_buf, bc_len)) != 0 { 27 } else { 22 }; continue;
}
// C line 38513
29 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((byte_code_offset) as isize))) as *mut c_void)) as *mut u8); bc_buf = assigned; assigned };
vm_block = 28; continue;
}
// C line 38506
30 => {
vm_block = if (((*(s)).is_rom_data as i32)) != 0 { 26 } else { 29 }; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(not(feature="dump-read-object"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38556. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadBigInt(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut n: u32 = core::mem::zeroed();
let mut p: *mut JSBigInt = core::mem::zeroed();
let mut v: js_limb_t = core::mem::zeroed();
let mut v8: u8 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut vm_block: usize = 30;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38605
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38604 labels: fail
2 => {
let _ = JS_FreeValue((*(s)).ctx, obj);
vm_block = 1; continue;
}
// C line 38602
3 => {
return JS_CompactBigInt((*(s)).ctx, p);
}
// C line 38599
4 => {
let _ = { let assigned = v; *(((*(p)).tab).as_mut_ptr()).offset((((*(p)).len).wrapping_sub((((1 as i32)) as u32))) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 38597
5 => {
let _ = { let assigned = ((((((v).wrapping_shl((shift) as u32)) as js_slimb_t)).wrapping_shr((shift) as u32)) as js_limb_t); v = assigned; assigned };
vm_block = 4; continue;
}
// C line 38596
6 => {
vm_block = if ((((shift) != ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 38594
7 => {
let _ = { let assigned = ((((((64 as i32)) as u32)).wrapping_sub((n).wrapping_mul((((8 as i32)) as u32)))) as i32); shift = assigned; assigned };
vm_block = 6; continue;
}
// C line 38589
8 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 12 } else { 7 }; continue;
}
// C line 38589
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 38592
10 => {
let _ = { v = (((((v) as js_limb_t)) | ((((v8) as js_limb_t)).wrapping_shl(((i).wrapping_mul((((8 as i32)) as u32))) as u32)))) as js_limb_t; v };
vm_block = 9; continue;
}
// C line 38591
11 => {
vm_block = 2; continue;
}
// C line 38590
12 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 11 } else { 10 }; continue;
}
// C line 38589
13 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 38588
14 => {
let _ = { let assigned = (((0 as i32)) as js_limb_t); v = assigned; assigned };
vm_block = 13; continue;
}
// C line 38586
15 => {
vm_block = if ((((n) != ((((0 as i32)) as u32))) as i32)) != 0 { 14 } else { 3 }; continue;
}
// C line 38585
16 => {
let _ = { let assigned = ((len) % ((((((64 as i32)) / ((8 as i32)))) as u32))); n = assigned; assigned };
vm_block = 15; continue;
}
// C line 38575
17 => {
vm_block = if ((((i) < (((len) / ((((((64 as i32)) / ((8 as i32)))) as u32))))) as i32)) != 0 { 21 } else { 16 }; continue;
}
// C line 38575
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 38583
19 => {
let _ = { let assigned = v; *(((*(p)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 38581
20 => {
vm_block = 2; continue;
}
// C line 38580
21 => {
vm_block = if (bc_get_u64(s, core::ptr::addr_of_mut!(v))) != 0 { 20 } else { 19 }; continue;
}
// C line 38575
22 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 38574
23 => {
vm_block = 2; continue;
}
// C line 38573
24 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 38572
25 => {
let _ = { let assigned = js_bigint_new((*(s)).ctx, ((((((len).wrapping_sub((((1 as i32)) as u32))) / ((((((64 as i32)) / ((8 as i32)))) as u32)))).wrapping_add((((1 as i32)) as u32))) as i32)); p = assigned; assigned };
vm_block = 24; continue;
}
// C line 38570
26 => {
return __JS_NewShortBigInt((*(s)).ctx, (((0 as i32)) as i64));
}
// C line 38567
27 => {
vm_block = if ((((len) == ((((0 as i32)) as u32))) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 38565
28 => {
vm_block = 2; continue;
}
// C line 38564
29 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(len))) != 0 { 28 } else { 27 }; continue;
}
// C line 38558
30 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 29; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature="dump-read-object")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38556. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadBigInt(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut n: u32 = core::mem::zeroed();
let mut p: *mut JSBigInt = core::mem::zeroed();
let mut v: js_limb_t = core::mem::zeroed();
let mut v8: u8 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut vm_block: usize = 33;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38605
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38604 labels: fail
2 => {
let _ = JS_FreeValue((*(s)).ctx, obj);
vm_block = 1; continue;
}
// C line 38602
3 => {
return JS_CompactBigInt((*(s)).ctx, p);
}
// C line 38601
4 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 3; continue;
}
// C line 38599
5 => {
let _ = { let assigned = v; *(((*(p)).tab).as_mut_ptr()).offset((((*(p)).len).wrapping_sub((((1 as i32)) as u32))) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 38597
6 => {
let _ = { let assigned = ((((((v).wrapping_shl((shift) as u32)) as js_slimb_t)).wrapping_shr((shift) as u32)) as js_limb_t); v = assigned; assigned };
vm_block = 5; continue;
}
// C line 38596
7 => {
vm_block = if ((((shift) != ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 38594
8 => {
let _ = { let assigned = ((((((64 as i32)) as u32)).wrapping_sub((n).wrapping_mul((((8 as i32)) as u32)))) as i32); shift = assigned; assigned };
vm_block = 7; continue;
}
// C line 38589
9 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 13 } else { 8 }; continue;
}
// C line 38589
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 38592
11 => {
let _ = { v = (((((v) as js_limb_t)) | ((((v8) as js_limb_t)).wrapping_shl(((i).wrapping_mul((((8 as i32)) as u32))) as u32)))) as js_limb_t; v };
vm_block = 10; continue;
}
// C line 38591
12 => {
vm_block = 2; continue;
}
// C line 38590
13 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 12 } else { 11 }; continue;
}
// C line 38589
14 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 38588
15 => {
let _ = { let assigned = (((0 as i32)) as js_limb_t); v = assigned; assigned };
vm_block = 14; continue;
}
// C line 38586
16 => {
vm_block = if ((((n) != ((((0 as i32)) as u32))) as i32)) != 0 { 15 } else { 4 }; continue;
}
// C line 38585
17 => {
let _ = { let assigned = ((len) % ((((((64 as i32)) / ((8 as i32)))) as u32))); n = assigned; assigned };
vm_block = 16; continue;
}
// C line 38575
18 => {
vm_block = if ((((i) < (((len) / ((((((64 as i32)) / ((8 as i32)))) as u32))))) as i32)) != 0 { 22 } else { 17 }; continue;
}
// C line 38575
19 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 18; continue;
}
// C line 38583
20 => {
let _ = { let assigned = v; *(((*(p)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 38581
21 => {
vm_block = 2; continue;
}
// C line 38580
22 => {
vm_block = if (bc_get_u64(s, core::ptr::addr_of_mut!(v))) != 0 { 21 } else { 20 }; continue;
}
// C line 38575
23 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 18; continue;
}
// C line 38574
24 => {
vm_block = 2; continue;
}
// C line 38573
25 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 38572
26 => {
let _ = { let assigned = js_bigint_new((*(s)).ctx, ((((((len).wrapping_sub((((1 as i32)) as u32))) / ((((((64 as i32)) / ((8 as i32)))) as u32)))).wrapping_add((((1 as i32)) as u32))) as i32)); p = assigned; assigned };
vm_block = 25; continue;
}
// C line 38570
27 => {
return __JS_NewShortBigInt((*(s)).ctx, (((0 as i32)) as i64));
}
// C line 38569
28 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 27; continue;
}
// C line 38567
29 => {
vm_block = if ((((len) == ((((0 as i32)) as u32))) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 38566
30 => {
let _ = bc_read_trace(s, c"len=%lld\n".as_ptr(), &[QuickJSPrintArg::Int(((len) as i64) as u64)]);
vm_block = 29; continue;
}
// C line 38565
31 => {
vm_block = 2; continue;
}
// C line 38564
32 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(len))) != 0 { 31 } else { 30 }; continue;
}
// C line 38558
33 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 32; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38610. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn BC_add_object_ref1(mut s: *mut BCReaderState, mut p: *mut JSObject) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38619
1 => {
return (0 as i32);
}
// C line 38617
2 => {
let _ = { let assigned = p; *((*(s)).objects).offset(({ let old = (*(s)).objects_count; (*(s)).objects_count = ((*(s)).objects_count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 38616
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 38613
4 => {
vm_block = if (js_resize_array((*(s)).ctx, ((((core::ptr::addr_of_mut!((*(s)).objects)) as *mut c_void)) as *mut *mut c_void), (((size_of::<*mut JSObject>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).objects_size), ((*(s)).objects_count).wrapping_add((1 as i32)))) != 0 { 3 } else { 2 }; continue;
}
// C line 38612
5 => {
vm_block = if (((*(s)).allow_reference as i32)) != 0 { 4 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38622. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn BC_add_object_ref(mut s: *mut BCReaderState, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38624
1 => {
return BC_add_object_ref1(s, ((((obj).u).ptr) as *mut JSObject));
}
_ => std::process::abort(),
} }
}

#[cfg(not(feature="dump-read-object"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38627. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadFunctionTag(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut bc: JSFunctionBytecode = core::mem::zeroed();
let mut b: *mut JSFunctionBytecode = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut v16: u16 = core::mem::zeroed();
let mut v8: u8 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut local_count: i32 = core::mem::zeroed();
let mut cpool_offset: i32 = core::mem::zeroed();
let mut byte_code_offset: i32 = core::mem::zeroed();
let mut closure_var_offset: i32 = core::mem::zeroed();
let mut vardefs_offset: i32 = core::mem::zeroed();
let mut function_size: u64 = core::mem::zeroed();
let mut vd: *mut JSBytecodeVarDef = core::mem::zeroed();
let mut cv: *mut JSClosureVar = core::mem::zeroed();
let mut var_idx: i32 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 138;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38826
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38825 labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 38823
3 => {
return obj;
}
// C line 38822
4 => {
let _ = { let assigned = JS_DupContext(ctx); (*(b)).realm = assigned; assigned };
vm_block = 3; continue;
}
// C line 38813
5 => {
vm_block = if ((((i) < ((*(b)).cpool_count)) as i32)) != 0 { 10 } else { 4 }; continue;
}
// C line 38813
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 38818
7 => {
let _ = { let assigned = val; *((*(b)).cpool).offset((i) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 38817
8 => {
vm_block = 2; continue;
}
// C line 38816
9 => {
vm_block = if (JS_IsException(val)) != 0 { 8 } else { 7 }; continue;
}
// C line 38815
10 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 9; continue;
}
// C line 38813
11 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 38811
12 => {
vm_block = if (((((*(b)).cpool_count) != ((0 as i32))) as i32)) != 0 { 11 } else { 4 }; continue;
}
// C line 38807
13 => {
vm_block = 2; continue;
}
// C line 38806
14 => {
vm_block = if (bc_get_buf(s, ((((*(b)).debug).source) as *mut u8), ((((*(b)).debug).source_len) as u32))) != 0 { 13 } else { 12 }; continue;
}
// C line 38805
15 => {
vm_block = 2; continue;
}
// C line 38804
16 => {
vm_block = if ((!(!(((*(b)).debug).source).is_null()) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 38803
17 => {
let _ = { let assigned = ((js_mallocz(ctx, ((((*(b)).debug).source_len) as usize))) as *mut c_char); ((*(b)).debug).source = assigned; assigned };
vm_block = 16; continue;
}
// C line 38801
18 => {
vm_block = if (((*(b)).debug).source_len) != 0 { 17 } else { 12 }; continue;
}
// C line 38800
19 => {
vm_block = 2; continue;
}
// C line 38799
20 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(((*(b)).debug).source_len))) != 0 { 19 } else { 18 }; continue;
}
// C line 38797
21 => {
vm_block = 2; continue;
}
// C line 38796
22 => {
vm_block = if (bc_get_buf(s, ((*(b)).debug).pc2line_buf, ((((*(b)).debug).pc2line_len) as u32))) != 0 { 21 } else { 20 }; continue;
}
// C line 38795
23 => {
vm_block = 2; continue;
}
// C line 38794
24 => {
vm_block = if ((!(!(((*(b)).debug).pc2line_buf).is_null()) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 38793
25 => {
let _ = { let assigned = ((js_mallocz(ctx, ((((*(b)).debug).pc2line_len) as usize))) as *mut u8); ((*(b)).debug).pc2line_buf = assigned; assigned };
vm_block = 24; continue;
}
// C line 38792
26 => {
vm_block = if (((*(b)).debug).pc2line_len) != 0 { 25 } else { 20 }; continue;
}
// C line 38791
27 => {
vm_block = 2; continue;
}
// C line 38790
28 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(((*(b)).debug).pc2line_len))) != 0 { 27 } else { 26 }; continue;
}
// C line 38786
29 => {
vm_block = 2; continue;
}
// C line 38785
30 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!(((*(b)).debug).filename))) != 0 { 29 } else { 28 }; continue;
}
// C line 38782
31 => {
vm_block = if ((*(b)).has_debug()) != 0 { 30 } else { 12 }; continue;
}
// C line 38779
32 => {
vm_block = 2; continue;
}
// C line 38778
33 => {
vm_block = if (JS_ReadFunctionBytecode(s, b, byte_code_offset, (((*(b)).byte_code_len) as u32))) != 0 { 32 } else { 31 }; continue;
}
// C line 38755
34 => {
vm_block = if ((((i) < ((*(b)).closure_var_count)) as i32)) != 0 { 48 } else { 33 }; continue;
}
// C line 38755
35 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 34; continue;
}
// C line 38769
36 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (4 as i32))) as u8); (*(cv)).set_var_kind((assigned) as _); assigned };
vm_block = 35; continue;
}
// C line 38768
37 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(cv)).set_is_lexical((assigned) as _); assigned };
vm_block = 36; continue;
}
// C line 38767
38 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(cv)).set_is_const((assigned) as _); assigned };
vm_block = 37; continue;
}
// C line 38766
39 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (3 as i32))) as JSClosureTypeEnum); (*(cv)).set_closure_type((assigned) as _); assigned };
vm_block = 38; continue;
}
// C line 38765
40 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 39; continue;
}
// C line 38764
41 => {
vm_block = 2; continue;
}
// C line 38763
42 => {
vm_block = if (bc_get_u16(s, core::ptr::addr_of_mut!(v16))) != 0 { 41 } else { 40 }; continue;
}
// C line 38762
43 => {
let _ = { let assigned = ((var_idx) as u16); (*(cv)).var_idx = assigned; assigned };
vm_block = 42; continue;
}
// C line 38761
44 => {
vm_block = 2; continue;
}
// C line 38760
45 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(var_idx))) != 0 { 44 } else { 43 }; continue;
}
// C line 38759
46 => {
vm_block = 2; continue;
}
// C line 38758
47 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(cv)).var_name))) != 0 { 46 } else { 45 }; continue;
}
// C line 38756
48 => {
cv = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((i) as isize));
vm_block = 47; continue;
}
// C line 38755
49 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 34; continue;
}
// C line 38753
50 => {
vm_block = if (((((*(b)).closure_var_count) != ((0 as i32))) as i32)) != 0 { 49 } else { 33 }; continue;
}
// C line 38730
51 => {
vm_block = if ((((i) < (local_count)) as i32)) != 0 { 68 } else { 50 }; continue;
}
// C line 38730
52 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 51; continue;
}
// C line 38746
53 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(vd)).set_has_scope((assigned) as _); assigned };
vm_block = 52; continue;
}
// C line 38745
54 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(vd)).set_is_captured((assigned) as _); assigned };
vm_block = 53; continue;
}
// C line 38744
55 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(vd)).set_is_lexical((assigned) as _); assigned };
vm_block = 54; continue;
}
// C line 38743
56 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(vd)).set_is_const((assigned) as _); assigned };
vm_block = 55; continue;
}
// C line 38742
57 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (4 as i32))) as u8); (*(vd)).set_var_kind((assigned) as _); assigned };
vm_block = 56; continue;
}
// C line 38741
58 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 57; continue;
}
// C line 38740
59 => {
vm_block = 2; continue;
}
// C line 38739
60 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 59 } else { 58 }; continue;
}
// C line 38738
61 => {
vm_block = 2; continue;
}
// C line 38737
62 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((*(vd)).var_ref_idx))) != 0 { 61 } else { 60 }; continue;
}
// C line 38736
63 => {
let _ = { let old = (*(vd)).scope_next; (*(vd)).scope_next = ((*(vd)).scope_next).wrapping_sub(1); old };
vm_block = 62; continue;
}
// C line 38735
64 => {
vm_block = 2; continue;
}
// C line 38734
65 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(vd)).scope_next))) != 0 { 64 } else { 63 }; continue;
}
// C line 38733
66 => {
vm_block = 2; continue;
}
// C line 38732
67 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(vd)).var_name))) != 0 { 66 } else { 65 }; continue;
}
// C line 38731
68 => {
vd = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((i) as isize));
vm_block = 67; continue;
}
// C line 38730
69 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 51; continue;
}
// C line 38728
70 => {
vm_block = if ((((local_count) != ((0 as i32))) as i32)) != 0 { 69 } else { 50 }; continue;
}
// C line 38717
71 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((b) as *mut c_void) }, tag: (((JS_TAG_FUNCTION_BYTECODE as i32)) as i64) }; obj = assigned; assigned };
vm_block = 70; continue;
}
// C line 38715
72 => {
let _ = add_gc_object((*(ctx)).rt, core::ptr::addr_of_mut!((*(b)).header), (((JS_GC_OBJ_TYPE_FUNCTION_BYTECODE as i32)) as JSGCObjectTypeEnum));
vm_block = 71; continue;
}
// C line 38714
73 => {
let _ = { let assigned = (1 as i32); (*(js_rc(((b) as *mut c_void)))).ref_count = assigned; assigned };
vm_block = 72; continue;
}
// C line 38711
74 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((cpool_offset) as isize))) as *mut c_void)) as *mut JSValue); (*(b)).cpool = assigned; assigned };
vm_block = 73; continue;
}
// C line 38710
75 => {
vm_block = if (((((*(b)).cpool_count) != ((0 as i32))) as i32)) != 0 { 74 } else { 73 }; continue;
}
// C line 38708
76 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((closure_var_offset) as isize))) as *mut c_void)) as *mut JSClosureVar); (*(b)).closure_var = assigned; assigned };
vm_block = 75; continue;
}
// C line 38707
77 => {
vm_block = if (((((*(b)).closure_var_count) != ((0 as i32))) as i32)) != 0 { 76 } else { 75 }; continue;
}
// C line 38705
78 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((vardefs_offset) as isize))) as *mut c_void)) as *mut JSBytecodeVarDef); (*(b)).vardefs = assigned; assigned };
vm_block = 77; continue;
}
// C line 38704
79 => {
vm_block = if ((((local_count) != ((0 as i32))) as i32)) != 0 { 78 } else { 77 }; continue;
}
// C line 38703
80 => {
let _ = { let dst = (((b) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((core::ptr::addr_of_mut!(bc)) as *const c_void)) as *const u8, dst, ((core::mem::offset_of!(JSFunctionBytecode, debug) as usize)) as usize); dst as *mut c_void };
vm_block = 79; continue;
}
// C line 38701
81 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38700
82 => {
vm_block = if ((!(!(b).is_null()) as i32)) != 0 { 81 } else { 80 }; continue;
}
// C line 38699
83 => {
let _ = { let assigned = ((js_mallocz(ctx, ((function_size) as usize))) as *mut JSFunctionBytecode); b = assigned; assigned };
vm_block = 82; continue;
}
// C line 38697
84 => {
return JS_ThrowOutOfMemory(ctx);
}
// C line 38696
85 => {
vm_block = if ((((function_size) > ((((2147483647 as i32)) as u64))) as i32)) != 0 { 84 } else { 83 }; continue;
}
// C line 38693
86 => {
let _ = { function_size = ((((function_size) as u64)).wrapping_add((((bc).byte_code_len) as u64))) as u64; function_size };
vm_block = 85; continue;
}
// C line 38692
87 => {
vm_block = if ((!(((bc).read_only_bytecode()) != 0) as i32)) != 0 { 86 } else { 85 }; continue;
}
// C line 38691
88 => {
let _ = { let assigned = ((function_size) as i32); byte_code_offset = assigned; assigned };
vm_block = 87; continue;
}
// C line 38690
89 => {
let _ = { function_size = ((((function_size) as u64)).wrapping_add(((((bc).closure_var_count) as u64)).wrapping_mul((((size_of::<JSClosureVar>() as usize)) as u64)))) as u64; function_size };
vm_block = 88; continue;
}
// C line 38689
90 => {
let _ = { let assigned = ((function_size) as i32); closure_var_offset = assigned; assigned };
vm_block = 89; continue;
}
// C line 38688
91 => {
let _ = { function_size = ((((function_size) as u64)).wrapping_add((((local_count) as u64)).wrapping_mul((((size_of::<JSBytecodeVarDef>() as usize)) as u64)))) as u64; function_size };
vm_block = 90; continue;
}
// C line 38687
92 => {
let _ = { let assigned = ((function_size) as i32); vardefs_offset = assigned; assigned };
vm_block = 91; continue;
}
// C line 38686
93 => {
let _ = { function_size = ((((function_size) as u64)).wrapping_add(((((bc).cpool_count) as u64)).wrapping_mul((((size_of::<JSValue>() as usize)) as u64)))) as u64; function_size };
vm_block = 92; continue;
}
// C line 38685
94 => {
let _ = { let assigned = ((function_size) as i32); cpool_offset = assigned; assigned };
vm_block = 93; continue;
}
// C line 38681
95 => {
let _ = { let assigned = (((size_of::<JSFunctionBytecode>() as usize)) as u64); function_size = assigned; assigned };
vm_block = 94; continue;
}
// C line 38683
96 => {
let _ = { let assigned = (((core::mem::offset_of!(JSFunctionBytecode, debug) as usize)) as u64); function_size = assigned; assigned };
vm_block = 94; continue;
}
// C line 38680
97 => {
vm_block = if ((bc).has_debug()) != 0 { 95 } else { 96 }; continue;
}
// C line 38678
98 => {
vm_block = 2; continue;
}
// C line 38677
99 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(local_count))) != 0 { 98 } else { 97 }; continue;
}
// C line 38676
100 => {
vm_block = 2; continue;
}
// C line 38675
101 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((bc).byte_code_len))) != 0 { 100 } else { 99 }; continue;
}
// C line 38674
102 => {
vm_block = 2; continue;
}
// C line 38673
103 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((bc).cpool_count))) != 0 { 102 } else { 101 }; continue;
}
// C line 38672
104 => {
vm_block = 2; continue;
}
// C line 38671
105 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((bc).closure_var_count))) != 0 { 104 } else { 103 }; continue;
}
// C line 38670
106 => {
vm_block = 2; continue;
}
// C line 38669
107 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).var_ref_count))) != 0 { 106 } else { 105 }; continue;
}
// C line 38668
108 => {
vm_block = 2; continue;
}
// C line 38667
109 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).stack_size))) != 0 { 108 } else { 107 }; continue;
}
// C line 38666
110 => {
vm_block = 2; continue;
}
// C line 38665
111 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).defined_arg_count))) != 0 { 110 } else { 109 }; continue;
}
// C line 38664
112 => {
vm_block = 2; continue;
}
// C line 38663
113 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).var_count))) != 0 { 112 } else { 111 }; continue;
}
// C line 38662
114 => {
vm_block = 2; continue;
}
// C line 38661
115 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).arg_count))) != 0 { 114 } else { 113 }; continue;
}
// C line 38660
116 => {
vm_block = 2; continue;
}
// C line 38659
117 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((bc).func_name))) != 0 { 116 } else { 115 }; continue;
}
// C line 38658
118 => {
let _ = { let assigned = v8; (bc).js_mode = assigned; assigned };
vm_block = 117; continue;
}
// C line 38657
119 => {
vm_block = 2; continue;
}
// C line 38656
120 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 119 } else { 118 }; continue;
}
// C line 38655
121 => {
let _ = { let assigned = ((((*(s)).is_rom_data as i32)) as u8); (bc).set_read_only_bytecode((assigned) as _); assigned };
vm_block = 120; continue;
}
// C line 38654
122 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_is_direct_or_indirect_eval((assigned) as _); assigned };
vm_block = 121; continue;
}
// C line 38653
123 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_has_debug((assigned) as _); assigned };
vm_block = 122; continue;
}
// C line 38652
124 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_arguments_allowed((assigned) as _); assigned };
vm_block = 123; continue;
}
// C line 38651
125 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_super_allowed((assigned) as _); assigned };
vm_block = 124; continue;
}
// C line 38650
126 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_super_call_allowed((assigned) as _); assigned };
vm_block = 125; continue;
}
// C line 38649
127 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_new_target_allowed((assigned) as _); assigned };
vm_block = 126; continue;
}
// C line 38648
128 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (2 as i32))) as u8); (bc).set_func_kind((assigned) as _); assigned };
vm_block = 127; continue;
}
// C line 38647
129 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_need_home_object((assigned) as _); assigned };
vm_block = 128; continue;
}
// C line 38646
130 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_is_derived_class_constructor((assigned) as _); assigned };
vm_block = 129; continue;
}
// C line 38645
131 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_has_simple_parameter_list((assigned) as _); assigned };
vm_block = 130; continue;
}
// C line 38644
132 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_has_prototype((assigned) as _); assigned };
vm_block = 131; continue;
}
// C line 38643
133 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 132; continue;
}
// C line 38642
134 => {
vm_block = 2; continue;
}
// C line 38641
135 => {
vm_block = if (bc_get_u16(s, core::ptr::addr_of_mut!(v16))) != 0 { 134 } else { 133 }; continue;
}
// C line 38639
136 => {
let _ = { let dst = (((core::ptr::addr_of_mut!(bc)) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<JSFunctionBytecode>() as usize)) as usize); dst as *mut c_void };
vm_block = 135; continue;
}
// C line 38631
137 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 136; continue;
}
// C line 38629
138 => {
ctx = (*(s)).ctx;
vm_block = 137; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature="dump-read-object")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38627. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadFunctionTag(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut bc: JSFunctionBytecode = core::mem::zeroed();
let mut b: *mut JSFunctionBytecode = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut v16: u16 = core::mem::zeroed();
let mut v8: u8 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut local_count: i32 = core::mem::zeroed();
let mut cpool_offset: i32 = core::mem::zeroed();
let mut byte_code_offset: i32 = core::mem::zeroed();
let mut closure_var_offset: i32 = core::mem::zeroed();
let mut vardefs_offset: i32 = core::mem::zeroed();
let mut function_size: u64 = core::mem::zeroed();
let mut vd: *mut JSBytecodeVarDef = core::mem::zeroed();
let mut cv: *mut JSClosureVar = core::mem::zeroed();
let mut var_idx: i32 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 163;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38826
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38825 labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 38823
3 => {
return obj;
}
// C line 38822
4 => {
let _ = { let assigned = JS_DupContext(ctx); (*(b)).realm = assigned; assigned };
vm_block = 3; continue;
}
// C line 38820
5 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 4; continue;
}
// C line 38813
6 => {
vm_block = if ((((i) < ((*(b)).cpool_count)) as i32)) != 0 { 11 } else { 5 }; continue;
}
// C line 38813
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 38818
8 => {
let _ = { let assigned = val; *((*(b)).cpool).offset((i) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 38817
9 => {
vm_block = 2; continue;
}
// C line 38816
10 => {
vm_block = if (JS_IsException(val)) != 0 { 9 } else { 8 }; continue;
}
// C line 38815
11 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 10; continue;
}
// C line 38813
12 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 38812
13 => {
let _ = bc_read_trace(s, c"cpool {\n".as_ptr(), &[]);
vm_block = 12; continue;
}
// C line 38811
14 => {
vm_block = if (((((*(b)).cpool_count) != ((0 as i32))) as i32)) != 0 { 13 } else { 4 }; continue;
}
// C line 38809
15 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 14; continue;
}
// C line 38807
16 => {
vm_block = 2; continue;
}
// C line 38806
17 => {
vm_block = if (bc_get_buf(s, ((((*(b)).debug).source) as *mut u8), ((((*(b)).debug).source_len) as u32))) != 0 { 16 } else { 15 }; continue;
}
// C line 38805
18 => {
vm_block = 2; continue;
}
// C line 38804
19 => {
vm_block = if ((!(!(((*(b)).debug).source).is_null()) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 38803
20 => {
let _ = { let assigned = ((js_mallocz(ctx, ((((*(b)).debug).source_len) as usize))) as *mut c_char); ((*(b)).debug).source = assigned; assigned };
vm_block = 19; continue;
}
// C line 38802
21 => {
let _ = bc_read_trace(s, c"source: %d bytes\n".as_ptr(), &[QuickJSPrintArg::Int(((*(b)).debug).source_len as u64)]);
vm_block = 20; continue;
}
// C line 38801
22 => {
vm_block = if (((*(b)).debug).source_len) != 0 { 21 } else { 15 }; continue;
}
// C line 38800
23 => {
vm_block = 2; continue;
}
// C line 38799
24 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(((*(b)).debug).source_len))) != 0 { 23 } else { 22 }; continue;
}
// C line 38797
25 => {
vm_block = 2; continue;
}
// C line 38796
26 => {
vm_block = if (bc_get_buf(s, ((*(b)).debug).pc2line_buf, ((((*(b)).debug).pc2line_len) as u32))) != 0 { 25 } else { 24 }; continue;
}
// C line 38795
27 => {
vm_block = 2; continue;
}
// C line 38794
28 => {
vm_block = if ((!(!(((*(b)).debug).pc2line_buf).is_null()) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 38793
29 => {
let _ = { let assigned = ((js_mallocz(ctx, ((((*(b)).debug).pc2line_len) as usize))) as *mut u8); ((*(b)).debug).pc2line_buf = assigned; assigned };
vm_block = 28; continue;
}
// C line 38792
30 => {
vm_block = if (((*(b)).debug).pc2line_len) != 0 { 29 } else { 24 }; continue;
}
// C line 38791
31 => {
vm_block = 2; continue;
}
// C line 38790
32 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(((*(b)).debug).pc2line_len))) != 0 { 31 } else { 30 }; continue;
}
// C line 38788
33 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 32; continue;
}
// C line 38788
34 => {
let _ = print_atom((*(s)).ctx, ((*(b)).debug).filename);
vm_block = 33; continue;
}
// C line 38788
35 => {
let _ = bc_read_trace(s, c"filename: ".as_ptr(), &[]);
vm_block = 34; continue;
}
// C line 38786
36 => {
vm_block = 2; continue;
}
// C line 38785
37 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!(((*(b)).debug).filename))) != 0 { 36 } else { 35 }; continue;
}
// C line 38784
38 => {
let _ = bc_read_trace(s, c"debug {\n".as_ptr(), &[]);
vm_block = 37; continue;
}
// C line 38782
39 => {
vm_block = if ((*(b)).has_debug()) != 0 { 38 } else { 14 }; continue;
}
// C line 38780
40 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 39; continue;
}
// C line 38779
41 => {
vm_block = 2; continue;
}
// C line 38778
42 => {
vm_block = if (JS_ReadFunctionBytecode(s, b, byte_code_offset, (((*(b)).byte_code_len) as u32))) != 0 { 41 } else { 40 }; continue;
}
// C line 38777
43 => {
let _ = bc_read_trace(s, c"bytecode {\n".as_ptr(), &[]);
vm_block = 42; continue;
}
// C line 38774
44 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 43; continue;
}
// C line 38755
45 => {
vm_block = if ((((i) < ((*(b)).closure_var_count)) as i32)) != 0 { 62 } else { 44 }; continue;
}
// C line 38755
46 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 45; continue;
}
// C line 38771
47 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 46; continue;
}
// C line 38771
48 => {
let _ = print_atom((*(s)).ctx, (*(cv)).var_name);
vm_block = 47; continue;
}
// C line 38771
49 => {
let _ = bc_read_trace(s, c"name: ".as_ptr(), &[]);
vm_block = 48; continue;
}
// C line 38769
50 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (4 as i32))) as u8); (*(cv)).set_var_kind((assigned) as _); assigned };
vm_block = 49; continue;
}
// C line 38768
51 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(cv)).set_is_lexical((assigned) as _); assigned };
vm_block = 50; continue;
}
// C line 38767
52 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(cv)).set_is_const((assigned) as _); assigned };
vm_block = 51; continue;
}
// C line 38766
53 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (3 as i32))) as JSClosureTypeEnum); (*(cv)).set_closure_type((assigned) as _); assigned };
vm_block = 52; continue;
}
// C line 38765
54 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 53; continue;
}
// C line 38764
55 => {
vm_block = 2; continue;
}
// C line 38763
56 => {
vm_block = if (bc_get_u16(s, core::ptr::addr_of_mut!(v16))) != 0 { 55 } else { 54 }; continue;
}
// C line 38762
57 => {
let _ = { let assigned = ((var_idx) as u16); (*(cv)).var_idx = assigned; assigned };
vm_block = 56; continue;
}
// C line 38761
58 => {
vm_block = 2; continue;
}
// C line 38760
59 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(var_idx))) != 0 { 58 } else { 57 }; continue;
}
// C line 38759
60 => {
vm_block = 2; continue;
}
// C line 38758
61 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(cv)).var_name))) != 0 { 60 } else { 59 }; continue;
}
// C line 38756
62 => {
cv = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((i) as isize));
vm_block = 61; continue;
}
// C line 38755
63 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 45; continue;
}
// C line 38754
64 => {
let _ = bc_read_trace(s, c"closure vars {\n".as_ptr(), &[]);
vm_block = 63; continue;
}
// C line 38753
65 => {
vm_block = if (((((*(b)).closure_var_count) != ((0 as i32))) as i32)) != 0 { 64 } else { 43 }; continue;
}
// C line 38751
66 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 65; continue;
}
// C line 38730
67 => {
vm_block = if ((((i) < (local_count)) as i32)) != 0 { 87 } else { 66 }; continue;
}
// C line 38730
68 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 67; continue;
}
// C line 38748
69 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 68; continue;
}
// C line 38748
70 => {
let _ = print_atom((*(s)).ctx, (*(vd)).var_name);
vm_block = 69; continue;
}
// C line 38748
71 => {
let _ = bc_read_trace(s, c"name: ".as_ptr(), &[]);
vm_block = 70; continue;
}
// C line 38746
72 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(vd)).set_has_scope((assigned) as _); assigned };
vm_block = 71; continue;
}
// C line 38745
73 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(vd)).set_is_captured((assigned) as _); assigned };
vm_block = 72; continue;
}
// C line 38744
74 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(vd)).set_is_lexical((assigned) as _); assigned };
vm_block = 73; continue;
}
// C line 38743
75 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (*(vd)).set_is_const((assigned) as _); assigned };
vm_block = 74; continue;
}
// C line 38742
76 => {
let _ = { let assigned = ((bc_get_flags(((v8) as u32), core::ptr::addr_of_mut!(idx), (4 as i32))) as u8); (*(vd)).set_var_kind((assigned) as _); assigned };
vm_block = 75; continue;
}
// C line 38741
77 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 76; continue;
}
// C line 38740
78 => {
vm_block = 2; continue;
}
// C line 38739
79 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 78 } else { 77 }; continue;
}
// C line 38738
80 => {
vm_block = 2; continue;
}
// C line 38737
81 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((*(vd)).var_ref_idx))) != 0 { 80 } else { 79 }; continue;
}
// C line 38736
82 => {
let _ = { let old = (*(vd)).scope_next; (*(vd)).scope_next = ((*(vd)).scope_next).wrapping_sub(1); old };
vm_block = 81; continue;
}
// C line 38735
83 => {
vm_block = 2; continue;
}
// C line 38734
84 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(vd)).scope_next))) != 0 { 83 } else { 82 }; continue;
}
// C line 38733
85 => {
vm_block = 2; continue;
}
// C line 38732
86 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(vd)).var_name))) != 0 { 85 } else { 84 }; continue;
}
// C line 38731
87 => {
vd = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((i) as isize));
vm_block = 86; continue;
}
// C line 38730
88 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 67; continue;
}
// C line 38729
89 => {
let _ = bc_read_trace(s, c"vars {\n".as_ptr(), &[]);
vm_block = 88; continue;
}
// C line 38728
90 => {
vm_block = if ((((local_count) != ((0 as i32))) as i32)) != 0 { 89 } else { 65 }; continue;
}
// C line 38725
91 => {
let _ = bc_read_trace(s, c"stack=%d bclen=%d locals=%d\n".as_ptr(), &[QuickJSPrintArg::Int((((*(b)).stack_size) as i32) as u64), QuickJSPrintArg::Int((*(b)).byte_code_len as u64), QuickJSPrintArg::Int(local_count as u64)]);
vm_block = 90; continue;
}
// C line 38722
92 => {
let _ = bc_read_trace(s, c"args=%d vars=%d defargs=%d closures=%d cpool=%d\n".as_ptr(), &[QuickJSPrintArg::Int((((*(b)).arg_count) as i32) as u64), QuickJSPrintArg::Int((((*(b)).var_count) as i32) as u64), QuickJSPrintArg::Int((((*(b)).defined_arg_count) as i32) as u64), QuickJSPrintArg::Int((*(b)).closure_var_count as u64), QuickJSPrintArg::Int((*(b)).cpool_count as u64)]);
vm_block = 91; continue;
}
// C line 38720
93 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 92; continue;
}
// C line 38720
94 => {
let _ = print_atom((*(s)).ctx, (*(b)).func_name);
vm_block = 93; continue;
}
// C line 38720
95 => {
let _ = bc_read_trace(s, c"name: ".as_ptr(), &[]);
vm_block = 94; continue;
}
// C line 38717
96 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((b) as *mut c_void) }, tag: (((JS_TAG_FUNCTION_BYTECODE as i32)) as i64) }; obj = assigned; assigned };
vm_block = 95; continue;
}
// C line 38715
97 => {
let _ = add_gc_object((*(ctx)).rt, core::ptr::addr_of_mut!((*(b)).header), (((JS_GC_OBJ_TYPE_FUNCTION_BYTECODE as i32)) as JSGCObjectTypeEnum));
vm_block = 96; continue;
}
// C line 38714
98 => {
let _ = { let assigned = (1 as i32); (*(js_rc(((b) as *mut c_void)))).ref_count = assigned; assigned };
vm_block = 97; continue;
}
// C line 38711
99 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((cpool_offset) as isize))) as *mut c_void)) as *mut JSValue); (*(b)).cpool = assigned; assigned };
vm_block = 98; continue;
}
// C line 38710
100 => {
vm_block = if (((((*(b)).cpool_count) != ((0 as i32))) as i32)) != 0 { 99 } else { 98 }; continue;
}
// C line 38708
101 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((closure_var_offset) as isize))) as *mut c_void)) as *mut JSClosureVar); (*(b)).closure_var = assigned; assigned };
vm_block = 100; continue;
}
// C line 38707
102 => {
vm_block = if (((((*(b)).closure_var_count) != ((0 as i32))) as i32)) != 0 { 101 } else { 100 }; continue;
}
// C line 38705
103 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((vardefs_offset) as isize))) as *mut c_void)) as *mut JSBytecodeVarDef); (*(b)).vardefs = assigned; assigned };
vm_block = 102; continue;
}
// C line 38704
104 => {
vm_block = if ((((local_count) != ((0 as i32))) as i32)) != 0 { 103 } else { 102 }; continue;
}
// C line 38703
105 => {
let _ = { let dst = (((b) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((core::ptr::addr_of_mut!(bc)) as *const c_void)) as *const u8, dst, ((core::mem::offset_of!(JSFunctionBytecode, debug) as usize)) as usize); dst as *mut c_void };
vm_block = 104; continue;
}
// C line 38701
106 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38700
107 => {
vm_block = if ((!(!(b).is_null()) as i32)) != 0 { 106 } else { 105 }; continue;
}
// C line 38699
108 => {
let _ = { let assigned = ((js_mallocz(ctx, ((function_size) as usize))) as *mut JSFunctionBytecode); b = assigned; assigned };
vm_block = 107; continue;
}
// C line 38697
109 => {
return JS_ThrowOutOfMemory(ctx);
}
// C line 38696
110 => {
vm_block = if ((((function_size) > ((((2147483647 as i32)) as u64))) as i32)) != 0 { 109 } else { 108 }; continue;
}
// C line 38693
111 => {
let _ = { function_size = ((((function_size) as u64)).wrapping_add((((bc).byte_code_len) as u64))) as u64; function_size };
vm_block = 110; continue;
}
// C line 38692
112 => {
vm_block = if ((!(((bc).read_only_bytecode()) != 0) as i32)) != 0 { 111 } else { 110 }; continue;
}
// C line 38691
113 => {
let _ = { let assigned = ((function_size) as i32); byte_code_offset = assigned; assigned };
vm_block = 112; continue;
}
// C line 38690
114 => {
let _ = { function_size = ((((function_size) as u64)).wrapping_add(((((bc).closure_var_count) as u64)).wrapping_mul((((size_of::<JSClosureVar>() as usize)) as u64)))) as u64; function_size };
vm_block = 113; continue;
}
// C line 38689
115 => {
let _ = { let assigned = ((function_size) as i32); closure_var_offset = assigned; assigned };
vm_block = 114; continue;
}
// C line 38688
116 => {
let _ = { function_size = ((((function_size) as u64)).wrapping_add((((local_count) as u64)).wrapping_mul((((size_of::<JSBytecodeVarDef>() as usize)) as u64)))) as u64; function_size };
vm_block = 115; continue;
}
// C line 38687
117 => {
let _ = { let assigned = ((function_size) as i32); vardefs_offset = assigned; assigned };
vm_block = 116; continue;
}
// C line 38686
118 => {
let _ = { function_size = ((((function_size) as u64)).wrapping_add(((((bc).cpool_count) as u64)).wrapping_mul((((size_of::<JSValue>() as usize)) as u64)))) as u64; function_size };
vm_block = 117; continue;
}
// C line 38685
119 => {
let _ = { let assigned = ((function_size) as i32); cpool_offset = assigned; assigned };
vm_block = 118; continue;
}
// C line 38681
120 => {
let _ = { let assigned = (((size_of::<JSFunctionBytecode>() as usize)) as u64); function_size = assigned; assigned };
vm_block = 119; continue;
}
// C line 38683
121 => {
let _ = { let assigned = (((core::mem::offset_of!(JSFunctionBytecode, debug) as usize)) as u64); function_size = assigned; assigned };
vm_block = 119; continue;
}
// C line 38680
122 => {
vm_block = if ((bc).has_debug()) != 0 { 120 } else { 121 }; continue;
}
// C line 38678
123 => {
vm_block = 2; continue;
}
// C line 38677
124 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(local_count))) != 0 { 123 } else { 122 }; continue;
}
// C line 38676
125 => {
vm_block = 2; continue;
}
// C line 38675
126 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((bc).byte_code_len))) != 0 { 125 } else { 124 }; continue;
}
// C line 38674
127 => {
vm_block = 2; continue;
}
// C line 38673
128 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((bc).cpool_count))) != 0 { 127 } else { 126 }; continue;
}
// C line 38672
129 => {
vm_block = 2; continue;
}
// C line 38671
130 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((bc).closure_var_count))) != 0 { 129 } else { 128 }; continue;
}
// C line 38670
131 => {
vm_block = 2; continue;
}
// C line 38669
132 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).var_ref_count))) != 0 { 131 } else { 130 }; continue;
}
// C line 38668
133 => {
vm_block = 2; continue;
}
// C line 38667
134 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).stack_size))) != 0 { 133 } else { 132 }; continue;
}
// C line 38666
135 => {
vm_block = 2; continue;
}
// C line 38665
136 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).defined_arg_count))) != 0 { 135 } else { 134 }; continue;
}
// C line 38664
137 => {
vm_block = 2; continue;
}
// C line 38663
138 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).var_count))) != 0 { 137 } else { 136 }; continue;
}
// C line 38662
139 => {
vm_block = 2; continue;
}
// C line 38661
140 => {
vm_block = if (bc_get_leb128_u16(s, core::ptr::addr_of_mut!((bc).arg_count))) != 0 { 139 } else { 138 }; continue;
}
// C line 38660
141 => {
vm_block = 2; continue;
}
// C line 38659
142 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((bc).func_name))) != 0 { 141 } else { 140 }; continue;
}
// C line 38658
143 => {
let _ = { let assigned = v8; (bc).js_mode = assigned; assigned };
vm_block = 142; continue;
}
// C line 38657
144 => {
vm_block = 2; continue;
}
// C line 38656
145 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 144 } else { 143 }; continue;
}
// C line 38655
146 => {
let _ = { let assigned = ((((*(s)).is_rom_data as i32)) as u8); (bc).set_read_only_bytecode((assigned) as _); assigned };
vm_block = 145; continue;
}
// C line 38654
147 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_is_direct_or_indirect_eval((assigned) as _); assigned };
vm_block = 146; continue;
}
// C line 38653
148 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_has_debug((assigned) as _); assigned };
vm_block = 147; continue;
}
// C line 38652
149 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_arguments_allowed((assigned) as _); assigned };
vm_block = 148; continue;
}
// C line 38651
150 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_super_allowed((assigned) as _); assigned };
vm_block = 149; continue;
}
// C line 38650
151 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_super_call_allowed((assigned) as _); assigned };
vm_block = 150; continue;
}
// C line 38649
152 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_new_target_allowed((assigned) as _); assigned };
vm_block = 151; continue;
}
// C line 38648
153 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (2 as i32))) as u8); (bc).set_func_kind((assigned) as _); assigned };
vm_block = 152; continue;
}
// C line 38647
154 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_need_home_object((assigned) as _); assigned };
vm_block = 153; continue;
}
// C line 38646
155 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_is_derived_class_constructor((assigned) as _); assigned };
vm_block = 154; continue;
}
// C line 38645
156 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_has_simple_parameter_list((assigned) as _); assigned };
vm_block = 155; continue;
}
// C line 38644
157 => {
let _ = { let assigned = ((bc_get_flags(((v16) as u32), core::ptr::addr_of_mut!(idx), (1 as i32))) as u8); (bc).set_has_prototype((assigned) as _); assigned };
vm_block = 156; continue;
}
// C line 38643
158 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 157; continue;
}
// C line 38642
159 => {
vm_block = 2; continue;
}
// C line 38641
160 => {
vm_block = if (bc_get_u16(s, core::ptr::addr_of_mut!(v16))) != 0 { 159 } else { 158 }; continue;
}
// C line 38639
161 => {
let _ = { let dst = (((core::ptr::addr_of_mut!(bc)) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<JSFunctionBytecode>() as usize)) as usize); dst as *mut c_void };
vm_block = 160; continue;
}
// C line 38631
162 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 161; continue;
}
// C line 38629
163 => {
ctx = (*(s)).ctx;
vm_block = 162; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(not(feature="dump-read-object"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38829. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadModule(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut m: *mut JSModuleDef = core::mem::zeroed();
let mut module_name: JSAtom = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut v8: u8 = core::mem::zeroed();
let mut rme: *mut JSReqModuleEntry = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut me: *mut JSExportEntry = core::mem::zeroed();
let mut se: *mut JSStarExportEntry = core::mem::zeroed();
let mut mi: *mut JSImportEntry = core::mem::zeroed();
let mut v8_1: u8 = core::mem::zeroed();
let mut vm_block: usize = 91;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38940
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38938
2 => {
let _ = JS_FreeValue(ctx, JSValue { u: JSValueUnion { ptr: ((m) as *mut c_void) }, tag: (((JS_TAG_MODULE as i32)) as i64) });
vm_block = 1; continue;
}
// C line 38937 labels: fail
3 => {
vm_block = if !(m).is_null() { 2 } else { 1 }; continue;
}
// C line 38935
4 => {
return obj;
}
// C line 38934
5 => {
vm_block = 3; continue;
}
// C line 38933
6 => {
vm_block = if (JS_IsException((*(m)).func_obj)) != 0 { 5 } else { 4 }; continue;
}
// C line 38932
7 => {
let _ = { let assigned = JS_ReadObjectRec(s); (*(m)).func_obj = assigned; assigned };
vm_block = 6; continue;
}
// C line 38930
8 => {
let _ = { let assigned = (((((v8) as i32)) != ((0 as i32))) as i32); (*(m)).has_tla = (assigned) as i8; assigned };
vm_block = 7; continue;
}
// C line 38929
9 => {
vm_block = 3; continue;
}
// C line 38928
10 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 9 } else { 8 }; continue;
}
// C line 38913
11 => {
vm_block = if ((((i) < ((*(m)).import_entries_count)) as i32)) != 0 { 22 } else { 10 }; continue;
}
// C line 38913
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 38924
13 => {
vm_block = 3; continue;
}
// C line 38923
14 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(mi)).req_module_idx))) != 0 { 13 } else { 12 }; continue;
}
// C line 38922
15 => {
vm_block = 3; continue;
}
// C line 38921
16 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(mi)).import_name))) != 0 { 15 } else { 14 }; continue;
}
// C line 38920
17 => {
let _ = { let assigned = (((((v8_1) as i32)) != ((0 as i32))) as i32); (*(mi)).is_star = assigned; assigned };
vm_block = 16; continue;
}
// C line 38919
18 => {
vm_block = 3; continue;
}
// C line 38918
19 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8_1))) != 0 { 18 } else { 17 }; continue;
}
// C line 38917
20 => {
vm_block = 3; continue;
}
// C line 38916
21 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(mi)).var_idx))) != 0 { 20 } else { 19 }; continue;
}
// C line 38914
22 => {
mi = core::ptr::addr_of_mut!(*((*(m)).import_entries).offset((i) as isize));
vm_block = 21; continue;
}
// C line 38913
23 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 38912
24 => {
vm_block = 3; continue;
}
// C line 38911
25 => {
vm_block = if ((!(!((*(m)).import_entries).is_null()) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 38910
26 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSImportEntry>() as usize)).wrapping_mul((((*(m)).import_entries_size) as usize)))) as *mut JSImportEntry); (*(m)).import_entries = assigned; assigned };
vm_block = 25; continue;
}
// C line 38909
27 => {
let _ = { let assigned = (*(m)).import_entries_count; (*(m)).import_entries_size = assigned; assigned };
vm_block = 26; continue;
}
// C line 38908
28 => {
vm_block = if (((((*(m)).import_entries_count) != ((0 as i32))) as i32)) != 0 { 27 } else { 10 }; continue;
}
// C line 38907
29 => {
vm_block = 3; continue;
}
// C line 38906
30 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(m)).import_entries_count))) != 0 { 29 } else { 28 }; continue;
}
// C line 38899
31 => {
vm_block = if ((((i) < ((*(m)).star_export_entries_count)) as i32)) != 0 { 35 } else { 30 }; continue;
}
// C line 38899
32 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 38902
33 => {
vm_block = 3; continue;
}
// C line 38901
34 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(se)).req_module_idx))) != 0 { 33 } else { 32 }; continue;
}
// C line 38900
35 => {
se = core::ptr::addr_of_mut!(*((*(m)).star_export_entries).offset((i) as isize));
vm_block = 34; continue;
}
// C line 38899
36 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 31; continue;
}
// C line 38898
37 => {
vm_block = 3; continue;
}
// C line 38897
38 => {
vm_block = if ((!(!((*(m)).star_export_entries).is_null()) as i32)) != 0 { 37 } else { 36 }; continue;
}
// C line 38896
39 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSStarExportEntry>() as usize)).wrapping_mul((((*(m)).star_export_entries_size) as usize)))) as *mut JSStarExportEntry); (*(m)).star_export_entries = assigned; assigned };
vm_block = 38; continue;
}
// C line 38895
40 => {
let _ = { let assigned = (*(m)).star_export_entries_count; (*(m)).star_export_entries_size = assigned; assigned };
vm_block = 39; continue;
}
// C line 38894
41 => {
vm_block = if (((((*(m)).star_export_entries_count) != ((0 as i32))) as i32)) != 0 { 40 } else { 30 }; continue;
}
// C line 38893
42 => {
vm_block = 3; continue;
}
// C line 38892
43 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(m)).star_export_entries_count))) != 0 { 42 } else { 41 }; continue;
}
// C line 38873
44 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 58 } else { 43 }; continue;
}
// C line 38873
45 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 38888
46 => {
vm_block = 3; continue;
}
// C line 38887
47 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(me)).export_name))) != 0 { 46 } else { 45 }; continue;
}
// C line 38880
48 => {
vm_block = 3; continue;
}
// C line 38879
49 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((((*(me)).u).local).var_idx))) != 0 { 48 } else { 47 }; continue;
}
// C line 38885
50 => {
vm_block = 3; continue;
}
// C line 38884
51 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(me)).local_name))) != 0 { 50 } else { 47 }; continue;
}
// C line 38883
52 => {
vm_block = 3; continue;
}
// C line 38882
53 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(((*(me)).u).req_module_idx))) != 0 { 52 } else { 51 }; continue;
}
// C line 38878
54 => {
vm_block = if (((((((*(me)).export_type) as u32)) == ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0 { 49 } else { 53 }; continue;
}
// C line 38877
55 => {
let _ = { let assigned = ((v8) as JSExportTypeEnum); (*(me)).export_type = assigned; assigned };
vm_block = 54; continue;
}
// C line 38876
56 => {
vm_block = 3; continue;
}
// C line 38875
57 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 56 } else { 55 }; continue;
}
// C line 38874
58 => {
me = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize));
vm_block = 57; continue;
}
// C line 38873
59 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 44; continue;
}
// C line 38872
60 => {
vm_block = 3; continue;
}
// C line 38871
61 => {
vm_block = if ((!(!((*(m)).export_entries).is_null()) as i32)) != 0 { 60 } else { 59 }; continue;
}
// C line 38870
62 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSExportEntry>() as usize)).wrapping_mul((((*(m)).export_entries_size) as usize)))) as *mut JSExportEntry); (*(m)).export_entries = assigned; assigned };
vm_block = 61; continue;
}
// C line 38869
63 => {
let _ = { let assigned = (*(m)).export_entries_count; (*(m)).export_entries_size = assigned; assigned };
vm_block = 62; continue;
}
// C line 38868
64 => {
vm_block = if (((((*(m)).export_entries_count) != ((0 as i32))) as i32)) != 0 { 63 } else { 43 }; continue;
}
// C line 38867
65 => {
vm_block = 3; continue;
}
// C line 38866
66 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(m)).export_entries_count))) != 0 { 65 } else { 64 }; continue;
}
// C line 38854
67 => {
vm_block = if ((((i) < ((*(m)).req_module_entries_count)) as i32)) != 0 { 75 } else { 66 }; continue;
}
// C line 38854
68 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 67; continue;
}
// C line 38862
69 => {
let _ = { let assigned = val; (*(rme)).attributes = assigned; assigned };
vm_block = 68; continue;
}
// C line 38861
70 => {
vm_block = 3; continue;
}
// C line 38860
71 => {
vm_block = if (JS_IsException(val)) != 0 { 70 } else { 69 }; continue;
}
// C line 38859
72 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 71; continue;
}
// C line 38858
73 => {
vm_block = 3; continue;
}
// C line 38857
74 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(rme)).module_name))) != 0 { 73 } else { 72 }; continue;
}
// C line 38855
75 => {
rme = core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((i) as isize));
vm_block = 74; continue;
}
// C line 38854
76 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 67; continue;
}
// C line 38853
77 => {
vm_block = 3; continue;
}
// C line 38852
78 => {
vm_block = if ((!(!((*(m)).req_module_entries).is_null()) as i32)) != 0 { 77 } else { 76 }; continue;
}
// C line 38851
79 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSReqModuleEntry>() as usize)).wrapping_mul((((*(m)).req_module_entries_size) as usize)))) as *mut JSReqModuleEntry); (*(m)).req_module_entries = assigned; assigned };
vm_block = 78; continue;
}
// C line 38850
80 => {
let _ = { let assigned = (*(m)).req_module_entries_count; (*(m)).req_module_entries_size = assigned; assigned };
vm_block = 79; continue;
}
// C line 38849
81 => {
vm_block = if (((((*(m)).req_module_entries_count) != ((0 as i32))) as i32)) != 0 { 80 } else { 66 }; continue;
}
// C line 38848
82 => {
vm_block = 3; continue;
}
// C line 38847
83 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(m)).req_module_entries_count))) != 0 { 82 } else { 81 }; continue;
}
// C line 38846
84 => {
let _ = { let assigned = JS_NewModuleValue(ctx, m); obj = assigned; assigned };
vm_block = 83; continue;
}
// C line 38845
85 => {
vm_block = 3; continue;
}
// C line 38844
86 => {
vm_block = if ((!(!(m).is_null()) as i32)) != 0 { 85 } else { 84 }; continue;
}
// C line 38843
87 => {
let _ = { let assigned = js_new_module_def(ctx, module_name); m = assigned; assigned };
vm_block = 86; continue;
}
// C line 38839
88 => {
vm_block = 3; continue;
}
// C line 38838
89 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!(module_name))) != 0 { 88 } else { 87 }; continue;
}
// C line 38833
90 => {
m = core::ptr::null_mut::<JSModuleDef>();
vm_block = 89; continue;
}
// C line 38831
91 => {
ctx = (*(s)).ctx;
vm_block = 90; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature="dump-read-object")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38829. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadModule(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut m: *mut JSModuleDef = core::mem::zeroed();
let mut module_name: JSAtom = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut v8: u8 = core::mem::zeroed();
let mut rme: *mut JSReqModuleEntry = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut me: *mut JSExportEntry = core::mem::zeroed();
let mut se: *mut JSStarExportEntry = core::mem::zeroed();
let mut mi: *mut JSImportEntry = core::mem::zeroed();
let mut v8_1: u8 = core::mem::zeroed();
let mut vm_block: usize = 94;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38940
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38938
2 => {
let _ = JS_FreeValue(ctx, JSValue { u: JSValueUnion { ptr: ((m) as *mut c_void) }, tag: (((JS_TAG_MODULE as i32)) as i64) });
vm_block = 1; continue;
}
// C line 38937 labels: fail
3 => {
vm_block = if !(m).is_null() { 2 } else { 1 }; continue;
}
// C line 38935
4 => {
return obj;
}
// C line 38934
5 => {
vm_block = 3; continue;
}
// C line 38933
6 => {
vm_block = if (JS_IsException((*(m)).func_obj)) != 0 { 5 } else { 4 }; continue;
}
// C line 38932
7 => {
let _ = { let assigned = JS_ReadObjectRec(s); (*(m)).func_obj = assigned; assigned };
vm_block = 6; continue;
}
// C line 38930
8 => {
let _ = { let assigned = (((((v8) as i32)) != ((0 as i32))) as i32); (*(m)).has_tla = (assigned) as i8; assigned };
vm_block = 7; continue;
}
// C line 38929
9 => {
vm_block = 3; continue;
}
// C line 38928
10 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 9 } else { 8 }; continue;
}
// C line 38913
11 => {
vm_block = if ((((i) < ((*(m)).import_entries_count)) as i32)) != 0 { 22 } else { 10 }; continue;
}
// C line 38913
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 38924
13 => {
vm_block = 3; continue;
}
// C line 38923
14 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(mi)).req_module_idx))) != 0 { 13 } else { 12 }; continue;
}
// C line 38922
15 => {
vm_block = 3; continue;
}
// C line 38921
16 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(mi)).import_name))) != 0 { 15 } else { 14 }; continue;
}
// C line 38920
17 => {
let _ = { let assigned = (((((v8_1) as i32)) != ((0 as i32))) as i32); (*(mi)).is_star = assigned; assigned };
vm_block = 16; continue;
}
// C line 38919
18 => {
vm_block = 3; continue;
}
// C line 38918
19 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8_1))) != 0 { 18 } else { 17 }; continue;
}
// C line 38917
20 => {
vm_block = 3; continue;
}
// C line 38916
21 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(mi)).var_idx))) != 0 { 20 } else { 19 }; continue;
}
// C line 38914
22 => {
mi = core::ptr::addr_of_mut!(*((*(m)).import_entries).offset((i) as isize));
vm_block = 21; continue;
}
// C line 38913
23 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 38912
24 => {
vm_block = 3; continue;
}
// C line 38911
25 => {
vm_block = if ((!(!((*(m)).import_entries).is_null()) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 38910
26 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSImportEntry>() as usize)).wrapping_mul((((*(m)).import_entries_size) as usize)))) as *mut JSImportEntry); (*(m)).import_entries = assigned; assigned };
vm_block = 25; continue;
}
// C line 38909
27 => {
let _ = { let assigned = (*(m)).import_entries_count; (*(m)).import_entries_size = assigned; assigned };
vm_block = 26; continue;
}
// C line 38908
28 => {
vm_block = if (((((*(m)).import_entries_count) != ((0 as i32))) as i32)) != 0 { 27 } else { 10 }; continue;
}
// C line 38907
29 => {
vm_block = 3; continue;
}
// C line 38906
30 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(m)).import_entries_count))) != 0 { 29 } else { 28 }; continue;
}
// C line 38899
31 => {
vm_block = if ((((i) < ((*(m)).star_export_entries_count)) as i32)) != 0 { 35 } else { 30 }; continue;
}
// C line 38899
32 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 38902
33 => {
vm_block = 3; continue;
}
// C line 38901
34 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(se)).req_module_idx))) != 0 { 33 } else { 32 }; continue;
}
// C line 38900
35 => {
se = core::ptr::addr_of_mut!(*((*(m)).star_export_entries).offset((i) as isize));
vm_block = 34; continue;
}
// C line 38899
36 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 31; continue;
}
// C line 38898
37 => {
vm_block = 3; continue;
}
// C line 38897
38 => {
vm_block = if ((!(!((*(m)).star_export_entries).is_null()) as i32)) != 0 { 37 } else { 36 }; continue;
}
// C line 38896
39 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSStarExportEntry>() as usize)).wrapping_mul((((*(m)).star_export_entries_size) as usize)))) as *mut JSStarExportEntry); (*(m)).star_export_entries = assigned; assigned };
vm_block = 38; continue;
}
// C line 38895
40 => {
let _ = { let assigned = (*(m)).star_export_entries_count; (*(m)).star_export_entries_size = assigned; assigned };
vm_block = 39; continue;
}
// C line 38894
41 => {
vm_block = if (((((*(m)).star_export_entries_count) != ((0 as i32))) as i32)) != 0 { 40 } else { 30 }; continue;
}
// C line 38893
42 => {
vm_block = 3; continue;
}
// C line 38892
43 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(m)).star_export_entries_count))) != 0 { 42 } else { 41 }; continue;
}
// C line 38873
44 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 58 } else { 43 }; continue;
}
// C line 38873
45 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 38888
46 => {
vm_block = 3; continue;
}
// C line 38887
47 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(me)).export_name))) != 0 { 46 } else { 45 }; continue;
}
// C line 38880
48 => {
vm_block = 3; continue;
}
// C line 38879
49 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((((*(me)).u).local).var_idx))) != 0 { 48 } else { 47 }; continue;
}
// C line 38885
50 => {
vm_block = 3; continue;
}
// C line 38884
51 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(me)).local_name))) != 0 { 50 } else { 47 }; continue;
}
// C line 38883
52 => {
vm_block = 3; continue;
}
// C line 38882
53 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!(((*(me)).u).req_module_idx))) != 0 { 52 } else { 51 }; continue;
}
// C line 38878
54 => {
vm_block = if (((((((*(me)).export_type) as u32)) == ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0 { 49 } else { 53 }; continue;
}
// C line 38877
55 => {
let _ = { let assigned = ((v8) as JSExportTypeEnum); (*(me)).export_type = assigned; assigned };
vm_block = 54; continue;
}
// C line 38876
56 => {
vm_block = 3; continue;
}
// C line 38875
57 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 56 } else { 55 }; continue;
}
// C line 38874
58 => {
me = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize));
vm_block = 57; continue;
}
// C line 38873
59 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 44; continue;
}
// C line 38872
60 => {
vm_block = 3; continue;
}
// C line 38871
61 => {
vm_block = if ((!(!((*(m)).export_entries).is_null()) as i32)) != 0 { 60 } else { 59 }; continue;
}
// C line 38870
62 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSExportEntry>() as usize)).wrapping_mul((((*(m)).export_entries_size) as usize)))) as *mut JSExportEntry); (*(m)).export_entries = assigned; assigned };
vm_block = 61; continue;
}
// C line 38869
63 => {
let _ = { let assigned = (*(m)).export_entries_count; (*(m)).export_entries_size = assigned; assigned };
vm_block = 62; continue;
}
// C line 38868
64 => {
vm_block = if (((((*(m)).export_entries_count) != ((0 as i32))) as i32)) != 0 { 63 } else { 43 }; continue;
}
// C line 38867
65 => {
vm_block = 3; continue;
}
// C line 38866
66 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(m)).export_entries_count))) != 0 { 65 } else { 64 }; continue;
}
// C line 38854
67 => {
vm_block = if ((((i) < ((*(m)).req_module_entries_count)) as i32)) != 0 { 75 } else { 66 }; continue;
}
// C line 38854
68 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 67; continue;
}
// C line 38862
69 => {
let _ = { let assigned = val; (*(rme)).attributes = assigned; assigned };
vm_block = 68; continue;
}
// C line 38861
70 => {
vm_block = 3; continue;
}
// C line 38860
71 => {
vm_block = if (JS_IsException(val)) != 0 { 70 } else { 69 }; continue;
}
// C line 38859
72 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 71; continue;
}
// C line 38858
73 => {
vm_block = 3; continue;
}
// C line 38857
74 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!((*(rme)).module_name))) != 0 { 73 } else { 72 }; continue;
}
// C line 38855
75 => {
rme = core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((i) as isize));
vm_block = 74; continue;
}
// C line 38854
76 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 67; continue;
}
// C line 38853
77 => {
vm_block = 3; continue;
}
// C line 38852
78 => {
vm_block = if ((!(!((*(m)).req_module_entries).is_null()) as i32)) != 0 { 77 } else { 76 }; continue;
}
// C line 38851
79 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSReqModuleEntry>() as usize)).wrapping_mul((((*(m)).req_module_entries_size) as usize)))) as *mut JSReqModuleEntry); (*(m)).req_module_entries = assigned; assigned };
vm_block = 78; continue;
}
// C line 38850
80 => {
let _ = { let assigned = (*(m)).req_module_entries_count; (*(m)).req_module_entries_size = assigned; assigned };
vm_block = 79; continue;
}
// C line 38849
81 => {
vm_block = if (((((*(m)).req_module_entries_count) != ((0 as i32))) as i32)) != 0 { 80 } else { 66 }; continue;
}
// C line 38848
82 => {
vm_block = 3; continue;
}
// C line 38847
83 => {
vm_block = if (bc_get_leb128_int(s, core::ptr::addr_of_mut!((*(m)).req_module_entries_count))) != 0 { 82 } else { 81 }; continue;
}
// C line 38846
84 => {
let _ = { let assigned = JS_NewModuleValue(ctx, m); obj = assigned; assigned };
vm_block = 83; continue;
}
// C line 38845
85 => {
vm_block = 3; continue;
}
// C line 38844
86 => {
vm_block = if ((!(!(m).is_null()) as i32)) != 0 { 85 } else { 84 }; continue;
}
// C line 38843
87 => {
let _ = { let assigned = js_new_module_def(ctx, module_name); m = assigned; assigned };
vm_block = 86; continue;
}
// C line 38841
88 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 87; continue;
}
// C line 38841
89 => {
let _ = print_atom((*(s)).ctx, module_name);
vm_block = 88; continue;
}
// C line 38841
90 => {
let _ = bc_read_trace(s, c"name: ".as_ptr(), &[]);
vm_block = 89; continue;
}
// C line 38839
91 => {
vm_block = 3; continue;
}
// C line 38838
92 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!(module_name))) != 0 { 91 } else { 90 }; continue;
}
// C line 38833
93 => {
m = core::ptr::null_mut::<JSModuleDef>();
vm_block = 92; continue;
}
// C line 38831
94 => {
ctx = (*(s)).ctx;
vm_block = 93; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(not(feature="dump-read-object"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38943. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadObjectTag(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut prop_count: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38976
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38975 labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 38973
3 => {
return obj;
}
// C line 38957
4 => {
vm_block = if ((((i) < (prop_count)) as i32)) != 0 { 15 } else { 3 }; continue;
}
// C line 38957
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 38971
6 => {
vm_block = 2; continue;
}
// C line 38970
7 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 38969
8 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 7; continue;
}
// C line 38968
9 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, obj, atom, val, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 38966
10 => {
vm_block = 2; continue;
}
// C line 38965
11 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 10; continue;
}
// C line 38964
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 9 }; continue;
}
// C line 38963
13 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 12; continue;
}
// C line 38959
14 => {
vm_block = 2; continue;
}
// C line 38958
15 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!(atom))) != 0 { 14 } else { 13 }; continue;
}
// C line 38957
16 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 38956
17 => {
vm_block = 2; continue;
}
// C line 38955
18 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(prop_count))) != 0 { 17 } else { 16 }; continue;
}
// C line 38954
19 => {
vm_block = 2; continue;
}
// C line 38953
20 => {
vm_block = if (BC_add_object_ref(s, obj)) != 0 { 19 } else { 18 }; continue;
}
// C line 38952
21 => {
let _ = { let assigned = JS_NewObject(ctx); obj = assigned; assigned };
vm_block = 20; continue;
}
// C line 38945
22 => {
ctx = (*(s)).ctx;
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature="dump-read-object")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38943. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadObjectTag(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut prop_count: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 38976
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 38975 labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 38973
3 => {
return obj;
}
// C line 38957
4 => {
vm_block = if ((((i) < (prop_count)) as i32)) != 0 { 18 } else { 3 }; continue;
}
// C line 38957
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 38971
6 => {
vm_block = 2; continue;
}
// C line 38970
7 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 38969
8 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 7; continue;
}
// C line 38968
9 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, obj, atom, val, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 38966
10 => {
vm_block = 2; continue;
}
// C line 38965
11 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 10; continue;
}
// C line 38964
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 9 }; continue;
}
// C line 38963
13 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 12; continue;
}
// C line 38961
14 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 13; continue;
}
// C line 38961
15 => {
let _ = print_atom((*(s)).ctx, atom);
vm_block = 14; continue;
}
// C line 38961
16 => {
let _ = bc_read_trace(s, c"propname: ".as_ptr(), &[]);
vm_block = 15; continue;
}
// C line 38959
17 => {
vm_block = 2; continue;
}
// C line 38958
18 => {
vm_block = if (bc_get_atom(s, core::ptr::addr_of_mut!(atom))) != 0 { 17 } else { 16 }; continue;
}
// C line 38957
19 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 38956
20 => {
vm_block = 2; continue;
}
// C line 38955
21 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(prop_count))) != 0 { 20 } else { 19 }; continue;
}
// C line 38954
22 => {
vm_block = 2; continue;
}
// C line 38953
23 => {
vm_block = if (BC_add_object_ref(s, obj)) != 0 { 22 } else { 21 }; continue;
}
// C line 38952
24 => {
let _ = { let assigned = JS_NewObject(ctx); obj = assigned; assigned };
vm_block = 23; continue;
}
// C line 38945
25 => {
ctx = (*(s)).ctx;
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:38979. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadArray(mut s: *mut BCReaderState, mut tag: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut prop_flags: i32 = core::mem::zeroed();
let mut is_template: i32 = core::mem::zeroed();
let mut vm_block: usize = 31;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39021
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39020 labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 39018
3 => {
return obj;
}
// C line 39016
4 => {
let _ = JS_PreventExtensions(ctx, obj);
vm_block = 3; continue;
}
// C line 39014
5 => {
vm_block = 2; continue;
}
// C line 39013
6 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 39012
7 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_raw as i32)) as JSAtom), val, (0 as i32)); ret = assigned; assigned };
vm_block = 6; continue;
}
// C line 39011
8 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 39010
9 => {
vm_block = 2; continue;
}
// C line 39009
10 => {
vm_block = if (JS_IsException(val)) != 0 { 9 } else { 8 }; continue;
}
// C line 39008
11 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 10; continue;
}
// C line 39007
12 => {
vm_block = if (is_template) != 0 { 11 } else { 3 }; continue;
}
// C line 38994
13 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 23 } else { 12 }; continue;
}
// C line 38994
14 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 13; continue;
}
// C line 39005
15 => {
vm_block = 2; continue;
}
// C line 39004
16 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 39002
17 => {
let _ = { let assigned = JS_DefinePropertyValueUint32(ctx, obj, i, val, prop_flags); ret = assigned; assigned };
vm_block = 16; continue;
}
// C line 38999
18 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((2 as i32)) as u32); prop_flags = assigned; assigned };
vm_block = 17; continue;
}
// C line 39001
19 => {
let _ = { let assigned = ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))); prop_flags = assigned; assigned };
vm_block = 17; continue;
}
// C line 38998
20 => {
vm_block = if (is_template) != 0 { 18 } else { 19 }; continue;
}
// C line 38997
21 => {
vm_block = 2; continue;
}
// C line 38996
22 => {
vm_block = if (JS_IsException(val)) != 0 { 21 } else { 20 }; continue;
}
// C line 38995
23 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 22; continue;
}
// C line 38994
24 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 13; continue;
}
// C line 38993
25 => {
vm_block = 2; continue;
}
// C line 38992
26 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(len))) != 0 { 25 } else { 24 }; continue;
}
// C line 38991
27 => {
let _ = { let assigned = (((tag) == ((BC_TAG_TEMPLATE_OBJECT as i32))) as i32); is_template = assigned; assigned };
vm_block = 26; continue;
}
// C line 38990
28 => {
vm_block = 2; continue;
}
// C line 38989
29 => {
vm_block = if (BC_add_object_ref(s, obj)) != 0 { 28 } else { 27 }; continue;
}
// C line 38988
30 => {
let _ = { let assigned = JS_NewArray(ctx); obj = assigned; assigned };
vm_block = 29; continue;
}
// C line 38981
31 => {
ctx = (*(s)).ctx;
vm_block = 30; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39024. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadTypedArray(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut array_buffer: JSValue = core::mem::zeroed();
let mut array_tag: u8 = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut offset: u32 = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut idx: u32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39068
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39067
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 39066 labels: fail
3 => {
let _ = JS_FreeValue(ctx, array_buffer);
vm_block = 2; continue;
}
// C line 39064
4 => {
return obj;
}
// C line 39063
5 => {
let _ = JS_FreeValue(ctx, array_buffer);
vm_block = 4; continue;
}
// C line 39061
6 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); *((*(s)).objects).offset((idx) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 39060
7 => {
vm_block = if (((*(s)).allow_reference as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 39059
8 => {
vm_block = 3; continue;
}
// C line 39058
9 => {
vm_block = if (JS_IsException(obj)) != 0 { 8 } else { 7 }; continue;
}
// C line 39055
10 => {
let _ = { let assigned = js_typed_array_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (3 as i32), (args).as_mut_ptr(), ((JS_CLASS_UINT8C_ARRAY as i32)).wrapping_add(((array_tag) as i32))); obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 39054
11 => {
let _ = { let assigned = JS_NewInt64(ctx, ((len) as i64)); *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 39053
12 => {
let _ = { let assigned = JS_NewInt64(ctx, ((offset) as i64)); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 39052
13 => {
let _ = { let assigned = array_buffer; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 39050
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39049
15 => {
let _ = JS_FreeValue(ctx, array_buffer);
vm_block = 14; continue;
}
// C line 39048
16 => {
vm_block = if ((!(!(js_get_array_buffer(ctx, array_buffer)).is_null()) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 39047
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39046
18 => {
vm_block = if (JS_IsException(array_buffer)) != 0 { 17 } else { 16 }; continue;
}
// C line 39045
19 => {
let _ = { let assigned = JS_ReadObjectRec(s); array_buffer = assigned; assigned };
vm_block = 18; continue;
}
// C line 39044
20 => {
vm_block = 3; continue;
}
// C line 39043
21 => {
vm_block = if (BC_add_object_ref1(s, core::ptr::null_mut::<JSObject>())) != 0 { 20 } else { 19 }; continue;
}
// C line 39042
22 => {
let _ = { let assigned = (((*(s)).objects_count) as u32); idx = assigned; assigned };
vm_block = 21; continue;
}
// C line 39039
23 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39038
24 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(offset))) != 0 { 23 } else { 22 }; continue;
}
// C line 39037
25 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39036
26 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(len))) != 0 { 25 } else { 24 }; continue;
}
// C line 39035
27 => {
return JS_ThrowTypeError(ctx, c"invalid typed array".as_ptr());
}
// C line 39034
28 => {
vm_block = if ((((((array_tag) as i32)) >= ((((JS_CLASS_FLOAT64_ARRAY as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))).wrapping_add((1 as i32)))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 39033
29 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39032
30 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(array_tag))) != 0 { 29 } else { 28 }; continue;
}
// C line 39027
31 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
array_buffer = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 30; continue;
}
// C line 39026
32 => {
ctx = (*(s)).ctx;
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39071. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadArrayBuffer(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut byte_length: u32 = core::mem::zeroed();
let mut max_byte_length: u32 = core::mem::zeroed();
let mut max_byte_length_u64: u64 = core::mem::zeroed();
let mut pmax_byte_length: *mut u64 = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39107
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39106 labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 39104
3 => {
return obj;
}
// C line 39103
4 => {
let _ = { (*(s)).ptr = (((((*(s)).ptr) as *const u8)).offset(((byte_length) as isize))) as *const u8; (*(s)).ptr };
vm_block = 3; continue;
}
// C line 39102
5 => {
vm_block = 2; continue;
}
// C line 39101
6 => {
vm_block = if (BC_add_object_ref(s, obj)) != 0 { 5 } else { 4 }; continue;
}
// C line 39100
7 => {
vm_block = 2; continue;
}
// C line 39099
8 => {
vm_block = if (JS_IsException(obj)) != 0 { 7 } else { 6 }; continue;
}
// C line 39093
9 => {
let _ = { let assigned = js_array_buffer_constructor3(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((byte_length) as u64), pmax_byte_length, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID), (((*(s)).ptr) as *mut u8), Some(js_array_buffer_free), core::ptr::null_mut::<c_void>(), (1 as i32)); obj = assigned; assigned };
vm_block = 8; continue;
}
// C line 39090
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39089
11 => {
let _ = bc_read_error_end(s);
vm_block = 10; continue;
}
// C line 39088
12 => {
vm_block = if ((((!(((!((((((((*(s)).buf_end).offset_from((*(s)).ptr) as i64)) < (((byte_length) as i64))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 11 } else { 9 }; continue;
}
// C line 39086
13 => {
let _ = { let assigned = core::ptr::addr_of_mut!(max_byte_length_u64); pmax_byte_length = assigned; assigned };
vm_block = 12; continue;
}
// C line 39085
14 => {
let _ = { let assigned = ((max_byte_length) as u64); max_byte_length_u64 = assigned; assigned };
vm_block = 13; continue;
}
// C line 39084
15 => {
vm_block = if ((((max_byte_length) != ((4294967295 as u32))) as i32)) != 0 { 14 } else { 12 }; continue;
}
// C line 39083
16 => {
return JS_ThrowTypeError(ctx, c"invalid array buffer".as_ptr());
}
// C line 39082
17 => {
vm_block = if ((((max_byte_length) < (byte_length)) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 39081
18 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39080
19 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(max_byte_length))) != 0 { 18 } else { 17 }; continue;
}
// C line 39079
20 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39078
21 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(byte_length))) != 0 { 20 } else { 19 }; continue;
}
// C line 39075
22 => {
pmax_byte_length = core::ptr::null_mut::<u64>();
vm_block = 21; continue;
}
// C line 39073
23 => {
ctx = (*(s)).ctx;
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39110. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadSharedArrayBuffer(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut byte_length: u32 = core::mem::zeroed();
let mut max_byte_length: u32 = core::mem::zeroed();
let mut max_byte_length_u64: u64 = core::mem::zeroed();
let mut pmax_byte_length: *mut u64 = core::mem::zeroed();
let mut data_ptr: *mut u8 = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut u64: u64 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39145
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39144 labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 39142
3 => {
return obj;
}
// C line 39141
4 => {
vm_block = 2; continue;
}
// C line 39140
5 => {
vm_block = if (BC_add_object_ref(s, obj)) != 0 { 4 } else { 3 }; continue;
}
// C line 39139
6 => {
vm_block = 2; continue;
}
// C line 39138
7 => {
vm_block = if (JS_IsException(obj)) != 0 { 6 } else { 5 }; continue;
}
// C line 39133
8 => {
let _ = { let assigned = js_array_buffer_constructor3(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((byte_length) as u64), pmax_byte_length, (((JS_CLASS_SHARED_ARRAY_BUFFER as i32)) as JSClassID), data_ptr, None, core::ptr::null_mut::<c_void>(), (0 as i32)); obj = assigned; assigned };
vm_block = 7; continue;
}
// C line 39131
9 => {
let _ = { let assigned = ((((u64) as usize)) as *mut u8); data_ptr = assigned; assigned };
vm_block = 8; continue;
}
// C line 39130
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39129
11 => {
vm_block = if (bc_get_u64(s, core::ptr::addr_of_mut!(u64))) != 0 { 10 } else { 9 }; continue;
}
// C line 39127
12 => {
let _ = { let assigned = core::ptr::addr_of_mut!(max_byte_length_u64); pmax_byte_length = assigned; assigned };
vm_block = 11; continue;
}
// C line 39126
13 => {
let _ = { let assigned = ((max_byte_length) as u64); max_byte_length_u64 = assigned; assigned };
vm_block = 12; continue;
}
// C line 39125
14 => {
vm_block = if ((((max_byte_length) != ((4294967295 as u32))) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 39124
15 => {
return JS_ThrowTypeError(ctx, c"invalid array buffer".as_ptr());
}
// C line 39123
16 => {
vm_block = if ((((max_byte_length) < (byte_length)) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 39122
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39121
18 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(max_byte_length))) != 0 { 17 } else { 16 }; continue;
}
// C line 39120
19 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39119
20 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(byte_length))) != 0 { 19 } else { 18 }; continue;
}
// C line 39114
21 => {
pmax_byte_length = core::ptr::null_mut::<u64>();
vm_block = 20; continue;
}
// C line 39112
22 => {
ctx = (*(s)).ctx;
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39148. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadDate(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39171
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39170
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 39169 labels: fail
3 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 2; continue;
}
// C line 39167
4 => {
return obj;
}
// C line 39166
5 => {
let _ = JS_SetObjectData(ctx, obj, val);
vm_block = 4; continue;
}
// C line 39165
6 => {
vm_block = 3; continue;
}
// C line 39164
7 => {
vm_block = if (BC_add_object_ref(s, obj)) != 0 { 6 } else { 5 }; continue;
}
// C line 39163
8 => {
vm_block = 3; continue;
}
// C line 39162
9 => {
vm_block = if (JS_IsException(obj)) != 0 { 8 } else { 7 }; continue;
}
// C line 39160
10 => {
let _ = { let assigned = JS_NewObjectProtoClass(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_DATE as i32)) as isize), (((JS_CLASS_DATE as i32)) as JSClassID)); obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 39158
11 => {
vm_block = 3; continue;
}
// C line 39157
12 => {
let _ = JS_ThrowTypeError(ctx, c"Number tag expected for date".as_ptr());
vm_block = 11; continue;
}
// C line 39156
13 => {
vm_block = if ((!((JS_IsNumber(val)) != 0) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 39155
14 => {
vm_block = 3; continue;
}
// C line 39154
15 => {
vm_block = if (JS_IsException(val)) != 0 { 14 } else { 13 }; continue;
}
// C line 39153
16 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 15; continue;
}
// C line 39151
17 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 16; continue;
}
// C line 39150
18 => {
ctx = (*(s)).ctx;
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39174. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadObjectValue(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39192
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39191
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 39190 labels: fail
3 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 2; continue;
}
// C line 39188
4 => {
return obj;
}
// C line 39187
5 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 4; continue;
}
// C line 39186
6 => {
vm_block = 3; continue;
}
// C line 39185
7 => {
vm_block = if (BC_add_object_ref(s, obj)) != 0 { 6 } else { 5 }; continue;
}
// C line 39184
8 => {
vm_block = 3; continue;
}
// C line 39183
9 => {
vm_block = if (JS_IsException(obj)) != 0 { 8 } else { 7 }; continue;
}
// C line 39182
10 => {
let _ = { let assigned = JS_ToObject(ctx, val); obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 39181
11 => {
vm_block = 3; continue;
}
// C line 39180
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 10 }; continue;
}
// C line 39179
13 => {
let _ = { let assigned = JS_ReadObjectRec(s); val = assigned; assigned };
vm_block = 12; continue;
}
// C line 39177
14 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 13; continue;
}
// C line 39176
15 => {
ctx = (*(s)).ctx;
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(not(feature="dump-read-object"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39195. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadObjectRec(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut tag: u8 = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut val: i32 = core::mem::zeroed();
let mut u: JSFloat64Union = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut val_1: u32 = core::mem::zeroed();
let mut vm_block: usize = 62;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39305
1 => {
return obj;
}
// C line 39301 labels: invalid_tag
2 => {
return JS_ThrowSyntaxError_cargs(ctx, c"invalid tag (tag=%d pos=%u)".as_ptr(), &[ParserFormatArg::Signed((((tag) as i32)) as i32), ParserFormatArg::Unsigned(((((((*(s)).ptr).offset_from((*(s)).buf_start) as i64)) as u32)) as u32)]);
}
// C line 39298
3 => {
vm_block = 1; continue;
}
// C line 39296
4 => {
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: ((*((*(s)).objects).offset((val_1) as isize)) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }); obj = assigned; assigned };
vm_block = 3; continue;
}
// C line 39293
5 => {
return JS_ThrowSyntaxError_cargs(ctx, c"invalid object reference (%u >= %u)".as_ptr(), &[ParserFormatArg::Unsigned((val_1) as u32), ParserFormatArg::Unsigned(((*(s)).objects_count) as u32)]);
}
// C line 39292
6 => {
vm_block = if ((((val_1) >= ((((*(s)).objects_count) as u32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 39290
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39289
8 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(val_1))) != 0 { 7 } else { 6 }; continue;
}
// C line 39288
9 => {
return JS_ThrowSyntaxError(ctx, c"object references are not allowed".as_ptr());
}
// C line 39287
10 => {
vm_block = if ((!((((*(s)).allow_reference as i32)) != 0) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 39283
11 => {
vm_block = 1; continue;
}
// C line 39282
12 => {
let _ = { let assigned = JS_ReadBigInt(s); obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 39280
13 => {
vm_block = 1; continue;
}
// C line 39279
14 => {
let _ = { let assigned = JS_ReadObjectValue(s); obj = assigned; assigned };
vm_block = 13; continue;
}
// C line 39277
15 => {
vm_block = 1; continue;
}
// C line 39276
16 => {
let _ = { let assigned = JS_ReadDate(s); obj = assigned; assigned };
vm_block = 15; continue;
}
// C line 39274
17 => {
vm_block = 1; continue;
}
// C line 39273
18 => {
let _ = { let assigned = JS_ReadSharedArrayBuffer(s); obj = assigned; assigned };
vm_block = 17; continue;
}
// C line 39272
19 => {
vm_block = 2; continue;
}
// C line 39271
20 => {
vm_block = if ((((((!((((*(s)).allow_sab as i32)) != 0) as i32)) != 0) || (((!((((*((*(ctx)).rt)).sab_funcs).sab_dup).is_some()) as i32)) != 0)) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 39269
21 => {
vm_block = 1; continue;
}
// C line 39268
22 => {
let _ = { let assigned = JS_ReadArrayBuffer(s); obj = assigned; assigned };
vm_block = 21; continue;
}
// C line 39266
23 => {
vm_block = 1; continue;
}
// C line 39265
24 => {
let _ = { let assigned = JS_ReadTypedArray(s); obj = assigned; assigned };
vm_block = 23; continue;
}
// C line 39263
25 => {
vm_block = 1; continue;
}
// C line 39262
26 => {
let _ = { let assigned = JS_ReadArray(s, ((tag) as i32)); obj = assigned; assigned };
vm_block = 25; continue;
}
// C line 39259
27 => {
vm_block = 1; continue;
}
// C line 39258
28 => {
let _ = { let assigned = JS_ReadObjectTag(s); obj = assigned; assigned };
vm_block = 27; continue;
}
// C line 39256
29 => {
vm_block = 1; continue;
}
// C line 39255
30 => {
let _ = { let assigned = JS_ReadModule(s); obj = assigned; assigned };
vm_block = 29; continue;
}
// C line 39254
31 => {
vm_block = 2; continue;
}
// C line 39253
32 => {
vm_block = if ((!((((*(s)).allow_bytecode as i32)) != 0) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 39251
33 => {
vm_block = 1; continue;
}
// C line 39250
34 => {
let _ = { let assigned = JS_ReadFunctionTag(s); obj = assigned; assigned };
vm_block = 33; continue;
}
// C line 39249
35 => {
vm_block = 2; continue;
}
// C line 39248
36 => {
vm_block = if ((!((((*(s)).allow_bytecode as i32)) != 0) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 39246
37 => {
vm_block = 1; continue;
}
// C line 39244
38 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) }; obj = assigned; assigned };
vm_block = 37; continue;
}
// C line 39243
39 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39242
40 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 39241
41 => {
let _ = { let assigned = JS_ReadString(s); p = assigned; assigned };
vm_block = 40; continue;
}
// C line 39237
42 => {
vm_block = 1; continue;
}
// C line 39235
43 => {
let _ = { let assigned = __JS_NewFloat64(ctx, (u).d); obj = assigned; assigned };
vm_block = 42; continue;
}
// C line 39233
44 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39232
45 => {
vm_block = if (bc_get_u64(s, core::ptr::addr_of_mut!((u).u64))) != 0 { 44 } else { 43 }; continue;
}
// C line 39228
46 => {
vm_block = 1; continue;
}
// C line 39226
47 => {
let _ = { let assigned = JS_NewInt32(ctx, val); obj = assigned; assigned };
vm_block = 46; continue;
}
// C line 39224
48 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39223
49 => {
vm_block = if (bc_get_sleb128(s, core::ptr::addr_of_mut!(val))) != 0 { 48 } else { 47 }; continue;
}
// C line 39219
50 => {
vm_block = 1; continue;
}
// C line 39218
51 => {
let _ = { let assigned = JS_NewBool(ctx, (((tag) as i32)).wrapping_sub((BC_TAG_BOOL_FALSE as i32))); obj = assigned; assigned };
vm_block = 50; continue;
}
// C line 39215
52 => {
vm_block = 1; continue;
}
// C line 39214
53 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj = assigned; assigned };
vm_block = 52; continue;
}
// C line 39212
54 => {
vm_block = 1; continue;
}
// C line 39211
55 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; obj = assigned; assigned };
vm_block = 54; continue;
}
// C line 39209
56 => {
vm_block = match ((tag) as i32) { x if x == (BC_TAG_OBJECT_REFERENCE as i32) => 10, x if x == (BC_TAG_BIG_INT as i32) => 12, x if x == (BC_TAG_OBJECT_VALUE as i32) => 14, x if x == (BC_TAG_DATE as i32) => 16, x if x == (BC_TAG_SHARED_ARRAY_BUFFER as i32) => 20, x if x == (BC_TAG_ARRAY_BUFFER as i32) => 22, x if x == (BC_TAG_TYPED_ARRAY as i32) => 24, x if x == (BC_TAG_TEMPLATE_OBJECT as i32) => 26, x if x == (BC_TAG_ARRAY as i32) => 26, x if x == (BC_TAG_OBJECT as i32) => 28, x if x == (BC_TAG_MODULE as i32) => 32, x if x == (BC_TAG_FUNCTION_BYTECODE as i32) => 36, x if x == (BC_TAG_STRING as i32) => 41, x if x == (BC_TAG_FLOAT64 as i32) => 45, x if x == (BC_TAG_INT32 as i32) => 49, x if x == (BC_TAG_BOOL_TRUE as i32) => 51, x if x == (BC_TAG_BOOL_FALSE as i32) => 51, x if x == (BC_TAG_UNDEFINED as i32) => 53, x if x == (BC_TAG_NULL as i32) => 55, _ => 2, }; continue;
}
// C line 39205
57 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39204
58 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(tag))) != 0 { 57 } else { 56 }; continue;
}
// C line 39202
59 => {
return JS_ThrowStackOverflow(ctx);
}
// C line 39201
60 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 59 } else { 58 }; continue;
}
// C line 39199
61 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 60; continue;
}
// C line 39197
62 => {
ctx = (*(s)).ctx;
vm_block = 61; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature="dump-read-object")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39195. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadObjectRec(mut s: *mut BCReaderState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut tag: u8 = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut val: i32 = core::mem::zeroed();
let mut u: JSFloat64Union = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut val_1: u32 = core::mem::zeroed();
let mut vm_block: usize = 67;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39305
1 => {
return obj;
}
// C line 39304
2 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 1; continue;
}
// C line 39301 labels: invalid_tag
3 => {
return JS_ThrowSyntaxError_cargs(ctx, c"invalid tag (tag=%d pos=%u)".as_ptr(), &[ParserFormatArg::Signed((((tag) as i32)) as i32), ParserFormatArg::Unsigned(((((((*(s)).ptr).offset_from((*(s)).buf_start) as i64)) as u32)) as u32)]);
}
// C line 39298
4 => {
vm_block = 2; continue;
}
// C line 39296
5 => {
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: ((*((*(s)).objects).offset((val_1) as isize)) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }); obj = assigned; assigned };
vm_block = 4; continue;
}
// C line 39293
6 => {
return JS_ThrowSyntaxError_cargs(ctx, c"invalid object reference (%u >= %u)".as_ptr(), &[ParserFormatArg::Unsigned((val_1) as u32), ParserFormatArg::Unsigned(((*(s)).objects_count) as u32)]);
}
// C line 39292
7 => {
vm_block = if ((((val_1) >= ((((*(s)).objects_count) as u32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 39291
8 => {
let _ = bc_read_trace(s, c"%u\n".as_ptr(), &[QuickJSPrintArg::Int(val_1 as u64)]);
vm_block = 7; continue;
}
// C line 39290
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39289
10 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!(val_1))) != 0 { 9 } else { 8 }; continue;
}
// C line 39288
11 => {
return JS_ThrowSyntaxError(ctx, c"object references are not allowed".as_ptr());
}
// C line 39287
12 => {
vm_block = if ((!((((*(s)).allow_reference as i32)) != 0) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 39283
13 => {
vm_block = 2; continue;
}
// C line 39282
14 => {
let _ = { let assigned = JS_ReadBigInt(s); obj = assigned; assigned };
vm_block = 13; continue;
}
// C line 39280
15 => {
vm_block = 2; continue;
}
// C line 39279
16 => {
let _ = { let assigned = JS_ReadObjectValue(s); obj = assigned; assigned };
vm_block = 15; continue;
}
// C line 39277
17 => {
vm_block = 2; continue;
}
// C line 39276
18 => {
let _ = { let assigned = JS_ReadDate(s); obj = assigned; assigned };
vm_block = 17; continue;
}
// C line 39274
19 => {
vm_block = 2; continue;
}
// C line 39273
20 => {
let _ = { let assigned = JS_ReadSharedArrayBuffer(s); obj = assigned; assigned };
vm_block = 19; continue;
}
// C line 39272
21 => {
vm_block = 3; continue;
}
// C line 39271
22 => {
vm_block = if ((((((!((((*(s)).allow_sab as i32)) != 0) as i32)) != 0) || (((!((((*((*(ctx)).rt)).sab_funcs).sab_dup).is_some()) as i32)) != 0)) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 39269
23 => {
vm_block = 2; continue;
}
// C line 39268
24 => {
let _ = { let assigned = JS_ReadArrayBuffer(s); obj = assigned; assigned };
vm_block = 23; continue;
}
// C line 39266
25 => {
vm_block = 2; continue;
}
// C line 39265
26 => {
let _ = { let assigned = JS_ReadTypedArray(s); obj = assigned; assigned };
vm_block = 25; continue;
}
// C line 39263
27 => {
vm_block = 2; continue;
}
// C line 39262
28 => {
let _ = { let assigned = JS_ReadArray(s, ((tag) as i32)); obj = assigned; assigned };
vm_block = 27; continue;
}
// C line 39259
29 => {
vm_block = 2; continue;
}
// C line 39258
30 => {
let _ = { let assigned = JS_ReadObjectTag(s); obj = assigned; assigned };
vm_block = 29; continue;
}
// C line 39256
31 => {
vm_block = 2; continue;
}
// C line 39255
32 => {
let _ = { let assigned = JS_ReadModule(s); obj = assigned; assigned };
vm_block = 31; continue;
}
// C line 39254
33 => {
vm_block = 3; continue;
}
// C line 39253
34 => {
vm_block = if ((!((((*(s)).allow_bytecode as i32)) != 0) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 39251
35 => {
vm_block = 2; continue;
}
// C line 39250
36 => {
let _ = { let assigned = JS_ReadFunctionTag(s); obj = assigned; assigned };
vm_block = 35; continue;
}
// C line 39249
37 => {
vm_block = 3; continue;
}
// C line 39248
38 => {
vm_block = if ((!((((*(s)).allow_bytecode as i32)) != 0) as i32)) != 0 { 37 } else { 36 }; continue;
}
// C line 39246
39 => {
vm_block = 2; continue;
}
// C line 39244
40 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) }; obj = assigned; assigned };
vm_block = 39; continue;
}
// C line 39243
41 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39242
42 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 39241
43 => {
let _ = { let assigned = JS_ReadString(s); p = assigned; assigned };
vm_block = 42; continue;
}
// C line 39237
44 => {
vm_block = 2; continue;
}
// C line 39235
45 => {
let _ = { let assigned = __JS_NewFloat64(ctx, (u).d); obj = assigned; assigned };
vm_block = 44; continue;
}
// C line 39234
46 => {
let _ = bc_read_trace(s, c"%g\n".as_ptr(), &[QuickJSPrintArg::Float((u).d as f64)]);
vm_block = 45; continue;
}
// C line 39233
47 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39232
48 => {
vm_block = if (bc_get_u64(s, core::ptr::addr_of_mut!((u).u64))) != 0 { 47 } else { 46 }; continue;
}
// C line 39228
49 => {
vm_block = 2; continue;
}
// C line 39226
50 => {
let _ = { let assigned = JS_NewInt32(ctx, val); obj = assigned; assigned };
vm_block = 49; continue;
}
// C line 39225
51 => {
let _ = bc_read_trace(s, c"%d\n".as_ptr(), &[QuickJSPrintArg::Int(val as u64)]);
vm_block = 50; continue;
}
// C line 39224
52 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39223
53 => {
vm_block = if (bc_get_sleb128(s, core::ptr::addr_of_mut!(val))) != 0 { 52 } else { 51 }; continue;
}
// C line 39219
54 => {
vm_block = 2; continue;
}
// C line 39218
55 => {
let _ = { let assigned = JS_NewBool(ctx, (((tag) as i32)).wrapping_sub((BC_TAG_BOOL_FALSE as i32))); obj = assigned; assigned };
vm_block = 54; continue;
}
// C line 39215
56 => {
vm_block = 2; continue;
}
// C line 39214
57 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj = assigned; assigned };
vm_block = 56; continue;
}
// C line 39212
58 => {
vm_block = 2; continue;
}
// C line 39211
59 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; obj = assigned; assigned };
vm_block = 58; continue;
}
// C line 39209
60 => {
vm_block = match ((tag) as i32) { x if x == (BC_TAG_OBJECT_REFERENCE as i32) => 12, x if x == (BC_TAG_BIG_INT as i32) => 14, x if x == (BC_TAG_OBJECT_VALUE as i32) => 16, x if x == (BC_TAG_DATE as i32) => 18, x if x == (BC_TAG_SHARED_ARRAY_BUFFER as i32) => 22, x if x == (BC_TAG_ARRAY_BUFFER as i32) => 24, x if x == (BC_TAG_TYPED_ARRAY as i32) => 26, x if x == (BC_TAG_TEMPLATE_OBJECT as i32) => 28, x if x == (BC_TAG_ARRAY as i32) => 28, x if x == (BC_TAG_OBJECT as i32) => 30, x if x == (BC_TAG_MODULE as i32) => 34, x if x == (BC_TAG_FUNCTION_BYTECODE as i32) => 38, x if x == (BC_TAG_STRING as i32) => 43, x if x == (BC_TAG_FLOAT64 as i32) => 48, x if x == (BC_TAG_INT32 as i32) => 53, x if x == (BC_TAG_BOOL_TRUE as i32) => 55, x if x == (BC_TAG_BOOL_FALSE as i32) => 55, x if x == (BC_TAG_UNDEFINED as i32) => 57, x if x == (BC_TAG_NULL as i32) => 59, _ => 3, }; continue;
}
// C line 39207
61 => {
let _ = bc_read_trace(s, c"%s {\n".as_ptr(), &[QuickJSPrintArg::Str(bc_read_tag_name((tag) as usize).as_ptr() as *const c_char)]);
vm_block = 60; continue;
}
// C line 39205
62 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39204
63 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(tag))) != 0 { 62 } else { 61 }; continue;
}
// C line 39202
64 => {
return JS_ThrowStackOverflow(ctx);
}
// C line 39201
65 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 64 } else { 63 }; continue;
}
// C line 39199
66 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 65; continue;
}
// C line 39197
67 => {
ctx = (*(s)).ctx;
vm_block = 66; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(not(feature="dump-read-object"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39308. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadObjectAtoms(mut s: *mut BCReaderState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v8: u8 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 24;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39345
1 => {
return (0 as i32);
}
// C line 39333
2 => {
vm_block = if ((((((i) as u32)) < ((*(s)).idx_to_atom_count)) as i32)) != 0 { 12 } else { 1 }; continue;
}
// C line 39333
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 39342
4 => {
let _ = { let assigned = (0 as i32); (*(s)).is_rom_data = (assigned) as i8; assigned };
vm_block = 3; continue;
}
// C line 39341
5 => {
vm_block = if (((((((*(s)).is_rom_data as i32)) != 0) && (((((atom) != ((((i) as u32)).wrapping_add((*(s)).first_atom))) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 39340
6 => {
let _ = { let assigned = atom; *((*(s)).idx_to_atom).offset((i) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 39339
7 => {
return { let assigned = ((1 as i32)).wrapping_neg(); (*(s)).error_state = assigned; assigned };
}
// C line 39338
8 => {
vm_block = if ((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 39337
9 => {
let _ = { let assigned = JS_NewAtomStr((*(s)).ctx, p); atom = assigned; assigned };
vm_block = 8; continue;
}
// C line 39336
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39335
11 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 39334
12 => {
let _ = { let assigned = JS_ReadString(s); p = assigned; assigned };
vm_block = 11; continue;
}
// C line 39333
13 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 39331
14 => {
return { let assigned = ((1 as i32)).wrapping_neg(); (*(s)).error_state = assigned; assigned };
}
// C line 39330
15 => {
vm_block = if ((!(!((*(s)).idx_to_atom).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 39328
16 => {
let _ = { let assigned = ((js_mallocz((*(s)).ctx, ((((*(s)).idx_to_atom_count) as usize)).wrapping_mul((size_of::<JSAtom>() as usize)))) as *mut JSAtom); (*(s)).idx_to_atom = assigned; assigned };
vm_block = 15; continue;
}
// C line 39327
17 => {
vm_block = if (((((*(s)).idx_to_atom_count) != ((((0 as i32)) as u32))) as i32)) != 0 { 16 } else { 13 }; continue;
}
// C line 39323
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39322
19 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!((*(s)).idx_to_atom_count))) != 0 { 18 } else { 17 }; continue;
}
// C line 39320
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39318
21 => {
let _ = JS_ThrowSyntaxError_cargs((*(s)).ctx, c"invalid version (%d expected=%d)".as_ptr(), &[ParserFormatArg::Signed((((v8) as i32)) as i32), ParserFormatArg::Signed(((5 as i32)) as i32)]);
vm_block = 20; continue;
}
// C line 39317
22 => {
vm_block = if ((((((v8) as i32)) != ((5 as i32))) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 39316
23 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39315
24 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 23 } else { 22 }; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature="dump-read-object")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39308. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ReadObjectAtoms(mut s: *mut BCReaderState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v8: u8 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39345
1 => {
return (0 as i32);
}
// C line 39344
2 => {
let _ = bc_read_trace(s, c"}\n".as_ptr(), &[]);
vm_block = 1; continue;
}
// C line 39333
3 => {
vm_block = if ((((((i) as u32)) < ((*(s)).idx_to_atom_count)) as i32)) != 0 { 13 } else { 2 }; continue;
}
// C line 39333
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 39342
5 => {
let _ = { let assigned = (0 as i32); (*(s)).is_rom_data = (assigned) as i8; assigned };
vm_block = 4; continue;
}
// C line 39341
6 => {
vm_block = if (((((((*(s)).is_rom_data as i32)) != 0) && (((((atom) != ((((i) as u32)).wrapping_add((*(s)).first_atom))) as i32)) != 0)) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 39340
7 => {
let _ = { let assigned = atom; *((*(s)).idx_to_atom).offset((i) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 39339
8 => {
return { let assigned = ((1 as i32)).wrapping_neg(); (*(s)).error_state = assigned; assigned };
}
// C line 39338
9 => {
vm_block = if ((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 39337
10 => {
let _ = { let assigned = JS_NewAtomStr((*(s)).ctx, p); atom = assigned; assigned };
vm_block = 9; continue;
}
// C line 39336
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39335
12 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 39334
13 => {
let _ = { let assigned = JS_ReadString(s); p = assigned; assigned };
vm_block = 12; continue;
}
// C line 39333
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 39331
15 => {
return { let assigned = ((1 as i32)).wrapping_neg(); (*(s)).error_state = assigned; assigned };
}
// C line 39330
16 => {
vm_block = if ((!(!((*(s)).idx_to_atom).is_null()) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 39328
17 => {
let _ = { let assigned = ((js_mallocz((*(s)).ctx, ((((*(s)).idx_to_atom_count) as usize)).wrapping_mul((size_of::<JSAtom>() as usize)))) as *mut JSAtom); (*(s)).idx_to_atom = assigned; assigned };
vm_block = 16; continue;
}
// C line 39327
18 => {
vm_block = if (((((*(s)).idx_to_atom_count) != ((((0 as i32)) as u32))) as i32)) != 0 { 17 } else { 14 }; continue;
}
// C line 39325
19 => {
let _ = bc_read_trace(s, c"%d atom indexes {\n".as_ptr(), &[QuickJSPrintArg::Int((*(s)).idx_to_atom_count as u64)]);
vm_block = 18; continue;
}
// C line 39323
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39322
21 => {
vm_block = if (bc_get_leb128(s, core::ptr::addr_of_mut!((*(s)).idx_to_atom_count))) != 0 { 20 } else { 19 }; continue;
}
// C line 39320
22 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39318
23 => {
let _ = JS_ThrowSyntaxError_cargs((*(s)).ctx, c"invalid version (%d expected=%d)".as_ptr(), &[ParserFormatArg::Signed((((v8) as i32)) as i32), ParserFormatArg::Signed(((5 as i32)) as i32)]);
vm_block = 22; continue;
}
// C line 39317
24 => {
vm_block = if ((((((v8) as i32)) != ((5 as i32))) as i32)) != 0 { 23 } else { 21 }; continue;
}
// C line 39316
25 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39315
26 => {
vm_block = if (bc_get_u8(s, core::ptr::addr_of_mut!(v8))) != 0 { 25 } else { 24 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39348. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn bc_reader_free(mut s: *mut BCReaderState) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 39357
1 => {
let _ = js_free((*(s)).ctx, (((*(s)).objects) as *mut c_void));
vm_block = 0; continue;
}
// C line 39355
2 => {
let _ = js_free((*(s)).ctx, (((*(s)).idx_to_atom) as *mut c_void));
vm_block = 1; continue;
}
// C line 39352
3 => {
vm_block = if ((((((i) as u32)) < ((*(s)).idx_to_atom_count)) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line 39352
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 39353
5 => {
let _ = JS_FreeAtom((*(s)).ctx, *((*(s)).idx_to_atom).offset((i) as isize));
vm_block = 4; continue;
}
// C line 39352
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 39351
7 => {
vm_block = if !((*(s)).idx_to_atom).is_null() { 6 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39360. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_ReadObject(mut ctx: *mut JSContext, mut buf: *const u8, mut buf_len: usize, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ss: BCReaderState = core::mem::zeroed();
let mut s: *mut BCReaderState = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39388
1 => {
return obj;
}
// C line 39387
2 => {
let _ = bc_reader_free(s);
vm_block = 1; continue;
}
// C line 39383
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; obj = assigned; assigned };
vm_block = 2; continue;
}
// C line 39385
4 => {
let _ = { let assigned = JS_ReadObjectRec(s); obj = assigned; assigned };
vm_block = 2; continue;
}
// C line 39382
5 => {
vm_block = if (JS_ReadObjectAtoms(s)) != 0 { 3 } else { 4 }; continue;
}
// C line 39379
6 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM_END as i32)) as u32); (*(s)).first_atom = assigned; assigned };
vm_block = 5; continue;
}
// C line 39381
7 => {
let _ = { let assigned = (((1 as i32)) as u32); (*(s)).first_atom = assigned; assigned };
vm_block = 5; continue;
}
// C line 39378
8 => {
vm_block = if (((*(s)).allow_bytecode as i32)) != 0 { 6 } else { 7 }; continue;
}
// C line 39377
9 => {
let _ = { let assigned = (((((flags) & (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))) != ((0 as i32))) as i32); (*(s)).allow_reference = (assigned) as i8; assigned };
vm_block = 8; continue;
}
// C line 39376
10 => {
let _ = { let assigned = (((((flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != ((0 as i32))) as i32); (*(s)).allow_sab = (assigned) as i8; assigned };
vm_block = 9; continue;
}
// C line 39375
11 => {
let _ = { let assigned = (((((flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != ((0 as i32))) as i32); (*(s)).is_rom_data = (assigned) as i8; assigned };
vm_block = 10; continue;
}
// C line 39374
12 => {
let _ = { let assigned = (((((flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != ((0 as i32))) as i32); (*(s)).allow_bytecode = (assigned) as i8; assigned };
vm_block = 11; continue;
}
// C line 39373
13 => {
let _ = { let assigned = buf; (*(s)).ptr = assigned; assigned };
vm_block = 12; continue;
}
// C line 39372
14 => {
let _ = { let assigned = (buf).offset(((buf_len) as isize)); (*(s)).buf_end = assigned; assigned };
vm_block = 13; continue;
}
// C line 39371
15 => {
let _ = { let assigned = buf; (*(s)).buf_start = assigned; assigned };
vm_block = 14; continue;
}
// C line 39370
16 => {
let _ = { let assigned = ctx; (*(s)).ctx = assigned; assigned };
vm_block = 15; continue;
}
// C line 39369
17 => {
let _ = { let dst = (((s) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<BCReaderState>() as usize)) as usize); dst as *mut c_void };
vm_block = 16; continue;
}
// C line 39367
18 => {
let _ = { (*(ctx)).binary_object_size = (((((*(ctx)).binary_object_size) as usize)).wrapping_add(buf_len)) as i32; (*(ctx)).binary_object_size };
vm_block = 17; continue;
}
// C line 39366
19 => {
let _ = { (*(ctx)).binary_object_count = (((((*(ctx)).binary_object_count) as i32)).wrapping_add((1 as i32))) as u16; (*(ctx)).binary_object_count };
vm_block = 18; continue;
}
// C line 39363
20 => {
s = core::ptr::addr_of_mut!(ss);
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}
