// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54493. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_get_hex(mut p: *mut JSString, mut k: i32, mut n: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut h: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54500
1 => {
return c;
}
// C line 54495
2 => {
vm_block = if (((({ let old = n; n = (n).wrapping_sub(1); old }) > ((0 as i32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 54498
3 => {
let _ = { let assigned = (((c).wrapping_shl(((4 as i32)) as u32)) | (h)); c = assigned; assigned };
vm_block = 2; continue;
}
// C line 54497
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54496
5 => {
vm_block = if (((({ let assigned = crate::cutils_header::from_hex(string_get(p, { let old = k; k = (k).wrapping_add(1); old })); h = assigned; assigned }) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 54494
6 => {
c = (0 as i32);
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54503. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn isURIReserved(mut c: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54504
1 => {
return (((((((c) < ((256 as i32))) as i32)) != 0) && (((((js_uri_memchr(((c";/?:@&=+$,#".as_ptr()) as *const c_void), c, ((size_of::<[c_char; 12]>() as usize)).wrapping_sub((((1 as i32)) as usize)))) != (core::ptr::null_mut::<c_void>())) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54517. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn hex_decode(mut ctx: *mut JSContext, mut p: *mut JSString, mut k: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54525
1 => {
return c;
}
// C line 54523
2 => {
return js_throw_URIError(ctx, c"expecting hex digit".as_ptr());
}
// C line 54522
3 => {
vm_block = if (((((((((k).wrapping_add((2 as i32))) >= ((((*(p)).len()) as i32))) as i32)) != 0) || ((((({ let assigned = string_get_hex(p, (k).wrapping_add((1 as i32)), (2 as i32)); c = assigned; assigned }) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 54521
4 => {
return js_throw_URIError(ctx, c"expecting %%".as_ptr());
}
// C line 54520
5 => {
vm_block = if ((((((((k) >= ((((*(p)).len()) as i32))) as i32)) != 0) || (((((string_get(p, k)) != ((37 as i32))) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54528. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_global_decodeURI(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut isComponent: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut c1: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut c_min: i32 = core::mem::zeroed();
let mut vm_block: usize = 52;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54601
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54600
2 => {
let _ = string_buffer_free(b);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 2; continue;
}
// C line 54596
4 => {
return string_buffer_end(b);
}
// C line 54595
5 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 4; continue;
}
// C line 54543
6 => {
vm_block = if ((((k) < ((((*(p)).len()) as i32))) as i32)) != 0 { 45 } else { 5 }; continue;
}
// C line 54593
7 => {
let _ = string_buffer_putc(b, ((c) as u32));
vm_block = 6; continue;
}
// C line 54553
8 => {
let _ = { k = (k).wrapping_sub((2 as i32)); k };
vm_block = 7; continue;
}
// C line 54552
9 => {
let _ = { let assigned = (37 as i32); c = assigned; assigned };
vm_block = 8; continue;
}
// C line 54551
10 => {
vm_block = if ((((((!((isComponent) != 0) as i32)) != 0) && ((isURIReserved(c)) != 0)) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 54587
11 => {
vm_block = 3; continue;
}
// C line 54586
12 => {
let _ = js_throw_URIError(ctx, c"malformed UTF-8".as_ptr());
vm_block = 11; continue;
}
// C line 54585
13 => {
vm_block = if ((((((((((((c) < (c_min)) as i32)) != 0) || (((((c) > ((1114111 as i32))) as i32)) != 0)) as i32)) != 0) || ((crate::cutils_header::is_surrogate(((c) as u32))) != 0)) as i32)) != 0 { 12 } else { 7 }; continue;
}
// C line 54574
14 => {
vm_block = if (((({ let old = n; n = (n).wrapping_sub(1); old }) > ((0 as i32))) as i32)) != 0 { 22 } else { 13 }; continue;
}
// C line 54583
15 => {
let _ = { let assigned = (((c).wrapping_shl(((6 as i32)) as u32)) | (((c1) & ((63 as i32))))); c = assigned; assigned };
vm_block = 14; continue;
}
// C line 54581
16 => {
vm_block = 13; continue;
}
// C line 54580
17 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 16; continue;
}
// C line 54579
18 => {
vm_block = if ((((((c1) & ((192 as i32)))) != ((128 as i32))) as i32)) != 0 { 17 } else { 15 }; continue;
}
// C line 54578
19 => {
let _ = { k = (k).wrapping_add((3 as i32)); k };
vm_block = 18; continue;
}
// C line 54577
20 => {
vm_block = 3; continue;
}
// C line 54576
21 => {
vm_block = if ((((c1) < ((0 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 54575
22 => {
let _ = { let assigned = hex_decode(ctx, p, k); c1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 54560
23 => {
let _ = { c = ((c) & ((31 as i32))); c };
vm_block = 14; continue;
}
// C line 54559
24 => {
let _ = { let assigned = (128 as i32); c_min = assigned; assigned };
vm_block = 23; continue;
}
// C line 54558
25 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 24; continue;
}
// C line 54564
26 => {
let _ = { c = ((c) & ((15 as i32))); c };
vm_block = 14; continue;
}
// C line 54563
27 => {
let _ = { let assigned = (2048 as i32); c_min = assigned; assigned };
vm_block = 26; continue;
}
// C line 54562
28 => {
let _ = { let assigned = (2 as i32); n = assigned; assigned };
vm_block = 27; continue;
}
// C line 54568
29 => {
let _ = { c = ((c) & ((7 as i32))); c };
vm_block = 14; continue;
}
// C line 54567
30 => {
let _ = { let assigned = (65536 as i32); c_min = assigned; assigned };
vm_block = 29; continue;
}
// C line 54566
31 => {
let _ = { let assigned = (3 as i32); n = assigned; assigned };
vm_block = 30; continue;
}
// C line 54572
32 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 14; continue;
}
// C line 54571
33 => {
let _ = { let assigned = (1 as i32); c_min = assigned; assigned };
vm_block = 32; continue;
}
// C line 54570
34 => {
let _ = { let assigned = (0 as i32); n = assigned; assigned };
vm_block = 33; continue;
}
// C line 54565
35 => {
vm_block = if ((((((((c) >= ((240 as i32))) as i32)) != 0) && (((((c) <= ((247 as i32))) as i32)) != 0)) as i32)) != 0 { 31 } else { 34 }; continue;
}
// C line 54561
36 => {
vm_block = if ((((((((c) >= ((224 as i32))) as i32)) != 0) && (((((c) <= ((239 as i32))) as i32)) != 0)) as i32)) != 0 { 28 } else { 35 }; continue;
}
// C line 54557
37 => {
vm_block = if ((((((((c) >= ((192 as i32))) as i32)) != 0) && (((((c) <= ((223 as i32))) as i32)) != 0)) as i32)) != 0 { 25 } else { 36 }; continue;
}
// C line 54550
38 => {
vm_block = if ((((c) < ((128 as i32))) as i32)) != 0 { 10 } else { 37 }; continue;
}
// C line 54549
39 => {
let _ = { k = (k).wrapping_add((3 as i32)); k };
vm_block = 38; continue;
}
// C line 54548
40 => {
vm_block = 3; continue;
}
// C line 54547
41 => {
vm_block = if ((((c) < ((0 as i32))) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 54546
42 => {
let _ = { let assigned = hex_decode(ctx, p, k); c = assigned; assigned };
vm_block = 41; continue;
}
// C line 54591
43 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 54545
44 => {
vm_block = if ((((c) == ((37 as i32))) as i32)) != 0 { 42 } else { 43 }; continue;
}
// C line 54544
45 => {
let _ = { let assigned = string_get(p, k); c = assigned; assigned };
vm_block = 44; continue;
}
// C line 54543
46 => {
let _ = { let assigned = (0 as i32); k = assigned; assigned };
vm_block = 6; continue;
}
// C line 54542
47 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 46; continue;
}
// C line 54540
48 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 47; continue;
}
// C line 54538
49 => {
return str;
}
// C line 54537
50 => {
vm_block = if (JS_IsException(str)) != 0 { 49 } else { 48 }; continue;
}
// C line 54536
51 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 50; continue;
}
// C line 54532
52 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 51; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54604. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn isUnescaped(mut c: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut unescaped_chars: [c_char; 70] = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54610
1 => {
return (((((((c) < ((256 as i32))) as i32)) != 0) && (!(js_uri_memchr((((unescaped_chars).as_ptr()) as *const c_void), c, ((size_of::<[c_char; 70]>() as usize)).wrapping_sub((((1 as i32)) as usize)))).is_null())) as i32);
}
// C line 54605
2 => {
unescaped_chars = *c"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789@*_+-./".as_ptr().cast::<[c_char; 70]>();
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54614. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn isURIUnescaped(mut c: i32, mut isComponent: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54615
1 => {
return (((((((c) < ((256 as i32))) as i32)) != 0) && (((((((((((((((((((((((((c) >= ((97 as i32))) as i32)) != 0) && (((((c) <= ((122 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((c) >= ((65 as i32))) as i32)) != 0) && (((((c) <= ((90 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((((c) >= ((48 as i32))) as i32)) != 0) && (((((c) <= ((57 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((js_uri_memchr(((c"-_.!~*'()".as_ptr()) as *const c_void), c, ((size_of::<[c_char; 10]>() as usize)).wrapping_sub((((1 as i32)) as usize)))) != (core::ptr::null_mut::<c_void>())) as i32)) != 0)) as i32)) != 0) || (((((((!((isComponent) != 0) as i32)) != 0) && ((isURIReserved(c)) != 0)) as i32)) != 0)) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54623. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn encodeURI_hex(mut b: *mut StringBuffer, mut c: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut buf: [u8; 6] = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut hex: *const c_char = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54636
1 => {
return string_buffer_write8(b, (buf).as_mut_ptr(), n);
}
// C line 54635
2 => {
let _ = { let assigned = ((*(hex).offset(((((c).wrapping_shr(((0 as i32)) as u32)) & ((15 as i32)))) as isize)) as u8); *((buf).as_mut_ptr()).offset(({ let old = n; n = (n).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 54634
3 => {
let _ = { let assigned = ((*(hex).offset(((((c).wrapping_shr(((4 as i32)) as u32)) & ((15 as i32)))) as isize)) as u8); *((buf).as_mut_ptr()).offset(({ let old = n; n = (n).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 54632
4 => {
let _ = { let assigned = ((*(hex).offset(((((c).wrapping_shr(((8 as i32)) as u32)) & ((15 as i32)))) as isize)) as u8); *((buf).as_mut_ptr()).offset(({ let old = n; n = (n).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 54631
5 => {
let _ = { let assigned = ((*(hex).offset(((((c).wrapping_shr(((12 as i32)) as u32)) & ((15 as i32)))) as isize)) as u8); *((buf).as_mut_ptr()).offset(({ let old = n; n = (n).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 54630
6 => {
let _ = { let assigned = (((117 as i32)) as u8); *((buf).as_mut_ptr()).offset(({ let old = n; n = (n).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 54629
7 => {
vm_block = if ((((c) >= ((256 as i32))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line 54628
8 => {
let _ = { let assigned = (((37 as i32)) as u8); *((buf).as_mut_ptr()).offset(({ let old = n; n = (n).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 54626
9 => {
hex = c"0123456789ABCDEF".as_ptr();
vm_block = 8; continue;
}
// C line 54625
10 => {
n = (0 as i32);
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54639. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_global_encodeURI(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut isComponent: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut c1: i32 = core::mem::zeroed();
let mut vm_block: usize = 40;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54701
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54700
2 => {
let _ = string_buffer_free(b);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 2; continue;
}
// C line 54696
4 => {
return string_buffer_end(b);
}
// C line 54695
5 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 4; continue;
}
// C line 54654
6 => {
vm_block = if ((((k) < ((((*(p)).len()) as i32))) as i32)) != 0 { 33 } else { 5 }; continue;
}
// C line 54658
7 => {
let _ = string_buffer_putc16(b, ((c) as u32));
vm_block = 6; continue;
}
// C line 54677
8 => {
let _ = encodeURI_hex(b, c);
vm_block = 6; continue;
}
// C line 54691
9 => {
let _ = encodeURI_hex(b, ((((c) & ((63 as i32)))) | ((128 as i32))));
vm_block = 6; continue;
}
// C line 54681
10 => {
let _ = encodeURI_hex(b, (((c).wrapping_shr(((6 as i32)) as u32)) | ((192 as i32))));
vm_block = 9; continue;
}
// C line 54689
11 => {
let _ = encodeURI_hex(b, (((((c).wrapping_shr(((6 as i32)) as u32)) & ((63 as i32)))) | ((128 as i32))));
vm_block = 9; continue;
}
// C line 54684
12 => {
let _ = encodeURI_hex(b, (((c).wrapping_shr(((12 as i32)) as u32)) | ((224 as i32))));
vm_block = 11; continue;
}
// C line 54687
13 => {
let _ = encodeURI_hex(b, (((((c).wrapping_shr(((12 as i32)) as u32)) & ((63 as i32)))) | ((128 as i32))));
vm_block = 11; continue;
}
// C line 54686
14 => {
let _ = encodeURI_hex(b, (((c).wrapping_shr(((18 as i32)) as u32)) | ((240 as i32))));
vm_block = 13; continue;
}
// C line 54683
15 => {
vm_block = if ((((c) < ((65536 as i32))) as i32)) != 0 { 12 } else { 14 }; continue;
}
// C line 54680
16 => {
vm_block = if ((((c) < ((2048 as i32))) as i32)) != 0 { 10 } else { 15 }; continue;
}
// C line 54676
17 => {
vm_block = if ((((c) < ((128 as i32))) as i32)) != 0 { 8 } else { 16 }; continue;
}
// C line 54662
18 => {
vm_block = 3; continue;
}
// C line 54661
19 => {
let _ = js_throw_URIError(ctx, c"invalid character".as_ptr());
vm_block = 18; continue;
}
// C line 54674
20 => {
let _ = { let assigned = ((from_surrogate(((c) as u32), ((c1) as u32))) as i32); c = assigned; assigned };
vm_block = 17; continue;
}
// C line 54672
21 => {
vm_block = 3; continue;
}
// C line 54671
22 => {
let _ = js_throw_URIError(ctx, c"expecting surrogate pair".as_ptr());
vm_block = 21; continue;
}
// C line 54670
23 => {
vm_block = if ((!((is_lo_surrogate(((c1) as u32))) != 0) as i32)) != 0 { 22 } else { 20 }; continue;
}
// C line 54669
24 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 54668
25 => {
let _ = { let assigned = string_get(p, k); c1 = assigned; assigned };
vm_block = 24; continue;
}
// C line 54666
26 => {
vm_block = 3; continue;
}
// C line 54665
27 => {
let _ = js_throw_URIError(ctx, c"expecting surrogate pair".as_ptr());
vm_block = 26; continue;
}
// C line 54664
28 => {
vm_block = if ((((k) >= ((((*(p)).len()) as i32))) as i32)) != 0 { 27 } else { 25 }; continue;
}
// C line 54663
29 => {
vm_block = if (is_hi_surrogate(((c) as u32))) != 0 { 28 } else { 17 }; continue;
}
// C line 54660
30 => {
vm_block = if (is_lo_surrogate(((c) as u32))) != 0 { 19 } else { 29 }; continue;
}
// C line 54657
31 => {
vm_block = if (isURIUnescaped(c, isComponent)) != 0 { 7 } else { 30 }; continue;
}
// C line 54656
32 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 54655
33 => {
let _ = { let assigned = string_get(p, k); c = assigned; assigned };
vm_block = 32; continue;
}
// C line 54654
34 => {
let _ = { let assigned = (0 as i32); k = assigned; assigned };
vm_block = 6; continue;
}
// C line 54653
35 => {
let _ = string_buffer_init(ctx, b, (((*(p)).len()) as i32));
vm_block = 34; continue;
}
// C line 54652
36 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 35; continue;
}
// C line 54650
37 => {
return str;
}
// C line 54649
38 => {
vm_block = if (JS_IsException(str)) != 0 { 37 } else { 36 }; continue;
}
// C line 54648
39 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 38; continue;
}
// C line 54644
40 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 39; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54704. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_global_escape(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54727
1 => {
return string_buffer_end(b);
}
// C line 54726
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 54718
3 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 8 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 54721
5 => {
let _ = string_buffer_putc16(b, ((c) as u32));
vm_block = 4; continue;
}
// C line 54723
6 => {
let _ = encodeURI_hex(b, c);
vm_block = 4; continue;
}
// C line 54720
7 => {
vm_block = if (isUnescaped(c)) != 0 { 5 } else { 6 }; continue;
}
// C line 54719
8 => {
let _ = { let assigned = string_get(p, i); c = assigned; assigned };
vm_block = 7; continue;
}
// C line 54718
9 => {
let _ = { let _ = { let assigned = (0 as i32); i = assigned; assigned }; { let assigned = (((*(p)).len()) as i32); len = assigned; assigned } };
vm_block = 3; continue;
}
// C line 54717
10 => {
let _ = string_buffer_init(ctx, b, (((*(p)).len()) as i32));
vm_block = 9; continue;
}
// C line 54716
11 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 10; continue;
}
// C line 54714
12 => {
return str;
}
// C line 54713
13 => {
vm_block = if (JS_IsException(str)) != 0 { 12 } else { 11 }; continue;
}
// C line 54712
14 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 13; continue;
}
// C line 54708
15 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54730. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_global_unescape(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54762
1 => {
return string_buffer_end(b);
}
// C line 54761
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 54744
3 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 13 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 54759
5 => {
let _ = string_buffer_putc16(b, ((c) as u32));
vm_block = 4; continue;
}
// C line 54751
6 => {
let _ = { i = (i).wrapping_add(((6 as i32)).wrapping_sub((1 as i32))); i };
vm_block = 5; continue;
}
// C line 54750
7 => {
let _ = { let assigned = n; c = assigned; assigned };
vm_block = 6; continue;
}
// C line 54756
8 => {
let _ = { i = (i).wrapping_add(((3 as i32)).wrapping_sub((1 as i32))); i };
vm_block = 5; continue;
}
// C line 54755
9 => {
let _ = { let assigned = n; c = assigned; assigned };
vm_block = 8; continue;
}
// C line 54753
10 => {
vm_block = if (((((((((i).wrapping_add((3 as i32))) <= (len)) as i32)) != 0) && ((((({ let assigned = string_get_hex(p, (i).wrapping_add((1 as i32)), (2 as i32)); n = assigned; assigned }) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 9 } else { 5 }; continue;
}
// C line 54747
11 => {
vm_block = if (((((((((((((i).wrapping_add((6 as i32))) <= (len)) as i32)) != 0) && (((((string_get(p, (i).wrapping_add((1 as i32)))) == ((117 as i32))) as i32)) != 0)) as i32)) != 0) && ((((({ let assigned = string_get_hex(p, (i).wrapping_add((2 as i32)), (4 as i32)); n = assigned; assigned }) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 7 } else { 10 }; continue;
}
// C line 54746
12 => {
vm_block = if ((((c) == ((37 as i32))) as i32)) != 0 { 11 } else { 5 }; continue;
}
// C line 54745
13 => {
let _ = { let assigned = string_get(p, i); c = assigned; assigned };
vm_block = 12; continue;
}
// C line 54744
14 => {
let _ = { let _ = { let assigned = (0 as i32); i = assigned; assigned }; { let assigned = (((*(p)).len()) as i32); len = assigned; assigned } };
vm_block = 3; continue;
}
// C line 54743
15 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 14; continue;
}
// C line 54742
16 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 15; continue;
}
// C line 54740
17 => {
return str;
}
// C line 54739
18 => {
vm_block = if (JS_IsException(str)) != 0 { 17 } else { 16 }; continue;
}
// C line 54738
19 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 18; continue;
}
// C line 54734
20 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39763. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_global_eval(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39766
1 => {
return JS_EvalObject(ctx, (*(ctx)).global_obj, *(argv).offset(((0 as i32)) as isize), ((3 as i32)).wrapping_shl(((0 as i32)) as u32), ((1 as i32)).wrapping_neg());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56284. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_AddIntrinsicBaseObjects(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj1: JSValue = core::mem::zeroed();
let mut obj2: JSValue = core::mem::zeroed();
let mut ft: JSCFunctionType = core::mem::zeroed();
let mut vm_block: usize = 95;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56501
1 => {
return (0 as i32);
}
// C line 56500
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56499
3 => {
vm_block = if (JS_AddIntrinsicBigInt(ctx)) != 0 { 2 } else { 1 }; continue;
}
// C line 56496
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56493
5 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, (*(ctx)).global_obj, (((crate::quickjs_atom::JS_ATOM_globalThis as i32)) as JSAtom), JS_DupValue(ctx, (*(ctx)).global_obj), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 56491
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56490
7 => {
vm_block = if (JS_IsException((*(ctx)).eval_obj)) != 0 { 6 } else { 5 }; continue;
}
// C line 56489
8 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(ctx)).global_obj, (((crate::quickjs_atom::JS_ATOM_eval as i32)) as JSAtom)); (*(ctx)).eval_obj = assigned; assigned };
vm_block = 7; continue;
}
// C line 56486
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56483
10 => {
vm_block = if (JS_SetConstructor2(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_GENERATOR_FUNCTION as i32)) as isize), *((*(ctx)).class_proto).offset(((JS_CLASS_GENERATOR as i32)) as isize), ((1 as i32)).wrapping_shl(((0 as i32)) as u32), ((1 as i32)).wrapping_shl(((0 as i32)) as u32))) != 0 { 9 } else { 8 }; continue;
}
// C line 56482
11 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 10; continue;
}
// C line 56481
12 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56480
13 => {
vm_block = if (JS_IsException(obj1)) != 0 { 12 } else { 11 }; continue;
}
// C line 56473
14 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_GENERATOR_FUNCTION as i32), c"GeneratorFunction".as_ptr(), (ft).generic, (1 as i32), (((JS_CFUNC_constructor_or_func_magic as i32)) as JSCFunctionEnum), (JS_FUNC_GENERATOR as i32), (*(ctx)).function_ctor, core::ptr::null_mut::<JSCFunctionListEntry>(), (0 as i32), ptr::addr_of!(js_generator_function_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))); obj1 = assigned; assigned };
vm_block = 13; continue;
}
// C line 56472
15 => {
let _ = { let assigned = js_function_constructor; (ft).generic_magic = Some(assigned); assigned };
vm_block = 14; continue;
}
// C line 56470
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56469
17 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_GENERATOR as i32)) as isize))) != 0 { 16 } else { 15 }; continue;
}
// C line 56465
18 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), ptr::addr_of!(js_generator_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 4]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_GENERATOR as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 56462
19 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 18; continue;
}
// C line 56461
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56460
21 => {
vm_block = if (JS_IsException(obj1)) != 0 { 20 } else { 19 }; continue;
}
// C line 56454
22 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_SYMBOL as i32), c"Symbol".as_ptr(), Some(js_symbol_constructor), (0 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_symbol_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 15]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_symbol_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 5]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 56451
23 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56450
24 => {
vm_block = if (JS_SetPropertyFunctionList(ctx, (*(ctx)).global_obj, (js_reflect_obj).as_ptr(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32))) != 0 { 23 } else { 22 }; continue;
}
// C line 56447
25 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56446
26 => {
vm_block = if (JS_SetPropertyFunctionList(ctx, (*(ctx)).global_obj, (js_math_obj).as_ptr(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32))) != 0 { 25 } else { 24 }; continue;
}
// C line 56445
27 => {
let _ = js_random_init(ctx);
vm_block = 26; continue;
}
// C line 56442
28 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56441
29 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_STRING_ITERATOR as i32)) as isize))) != 0 { 28 } else { 27 }; continue;
}
// C line 56437
30 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), ptr::addr_of!(js_string_iterator_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_STRING_ITERATOR as i32)) as isize) = assigned; assigned };
vm_block = 29; continue;
}
// C line 56435
31 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56434
32 => {
vm_block = if (JS_SetObjectData(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_STRING as i32)) as isize), JS_AtomToString(ctx, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom)))) != 0 { 31 } else { 30 }; continue;
}
// C line 56433
33 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 32; continue;
}
// C line 56432
34 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56431
35 => {
vm_block = if (JS_IsException(obj1)) != 0 { 34 } else { 33 }; continue;
}
// C line 56425
36 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_STRING as i32), c"String".as_ptr(), Some(js_string_constructor), (1 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_string_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 3]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_string_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 50]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((1 as i32)).wrapping_shl(((1 as i32)) as u32)); obj1 = assigned; assigned };
vm_block = 35; continue;
}
// C line 56422
37 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56421
38 => {
vm_block = if (JS_SetObjectData(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_BOOLEAN as i32)) as isize), JS_NewBool(ctx, (0 as i32)))) != 0 { 37 } else { 36 }; continue;
}
// C line 56420
39 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 38; continue;
}
// C line 56419
40 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56418
41 => {
vm_block = if (JS_IsException(obj1)) != 0 { 40 } else { 39 }; continue;
}
// C line 56412
42 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_BOOLEAN as i32), c"Boolean".as_ptr(), Some(js_boolean_constructor), (1 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, core::ptr::null_mut::<JSCFunctionListEntry>(), (0 as i32), ptr::addr_of!(js_boolean_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((1 as i32)).wrapping_shl(((1 as i32)) as u32)); obj1 = assigned; assigned };
vm_block = 41; continue;
}
// C line 56409
43 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56408
44 => {
vm_block = if (JS_SetObjectData(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_NUMBER as i32)) as isize), JS_NewInt32(ctx, (0 as i32)))) != 0 { 43 } else { 42 }; continue;
}
// C line 56407
45 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 44; continue;
}
// C line 56406
46 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56405
47 => {
vm_block = if (JS_IsException(obj1)) != 0 { 46 } else { 45 }; continue;
}
// C line 56399
48 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_NUMBER as i32), c"Number".as_ptr(), Some(js_number_constructor), (1 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_number_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 14]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_number_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 6]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((1 as i32)).wrapping_shl(((1 as i32)) as u32)); obj1 = assigned; assigned };
vm_block = 47; continue;
}
// C line 56396
49 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56394
50 => {
vm_block = if (JS_SetPropertyFunctionList(ctx, (*(ctx)).global_obj, ptr::addr_of!(js_global_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 15]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32))) != 0 { 49 } else { 48 }; continue;
}
// C line 56389
51 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56388
52 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_ARRAY_ITERATOR as i32)) as isize))) != 0 { 51 } else { 50 }; continue;
}
// C line 56384
53 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), ptr::addr_of!(js_array_iterator_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_ARRAY_ITERATOR as i32)) as isize) = assigned; assigned };
vm_block = 52; continue;
}
// C line 56382
54 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56381
55 => {
vm_block = if (JS_IsException((*(ctx)).array_proto_values)) != 0 { 54 } else { 53 }; continue;
}
// C line 56379
56 => {
let _ = { let assigned = JS_GetProperty(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ARRAY as i32)) as isize), (((crate::quickjs_atom::JS_ATOM_values as i32)) as JSAtom)); (*(ctx)).array_proto_values = assigned; assigned };
vm_block = 55; continue;
}
// C line 56376
57 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56375
58 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR_WRAP as i32)) as isize))) != 0 { 57 } else { 56 }; continue;
}
// C line 56371
59 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), ptr::addr_of!(js_iterator_wrap_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR_WRAP as i32)) as isize) = assigned; assigned };
vm_block = 58; continue;
}
// C line 56369
60 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56368
61 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR_HELPER as i32)) as isize))) != 0 { 60 } else { 59 }; continue;
}
// C line 56364
62 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), ptr::addr_of!(js_iterator_helper_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 3]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR_HELPER as i32)) as isize) = assigned; assigned };
vm_block = 61; continue;
}
// C line 56363
63 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56362
64 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR_CONCAT as i32)) as isize))) != 0 { 63 } else { 62 }; continue;
}
// C line 56358
65 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), ptr::addr_of!(js_iterator_concat_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 3]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR_CONCAT as i32)) as isize) = assigned; assigned };
vm_block = 64; continue;
}
// C line 56356
66 => {
let _ = { let assigned = obj2; (*(ctx)).iterator_ctor = assigned; assigned };
vm_block = 65; continue;
}
// C line 56355
67 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 66; continue;
}
// C line 56353
68 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56352
69 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 68; continue;
}
// C line 56351
70 => {
let _ = JS_FreeValue(ctx, obj2);
vm_block = 69; continue;
}
// C line 56347
71 => {
vm_block = if ((((JS_DefineProperty(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), (((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj1, obj1, ((((((1 as i32)).wrapping_shl(((11 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 70 } else { 67 }; continue;
}
// C line 56345
72 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56344
73 => {
let _ = JS_FreeValue(ctx, obj2);
vm_block = 72; continue;
}
// C line 56343
74 => {
vm_block = if (JS_IsException(obj1)) != 0 { 73 } else { 71 }; continue;
}
// C line 56341
75 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_iterator_constructor_getset), (0 as i32), (0 as i32), (1 as i32), core::ptr::addr_of_mut!(obj2)); obj1 = assigned; assigned };
vm_block = 74; continue;
}
// C line 56336
76 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56335
77 => {
vm_block = if (JS_IsException(obj2)) != 0 { 76 } else { 75 }; continue;
}
// C line 56329
78 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_ITERATOR as i32), c"Iterator".as_ptr(), Some(js_iterator_constructor), (0 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_iterator_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_iterator_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 13]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj2 = assigned; assigned };
vm_block = 77; continue;
}
// C line 56326
79 => {
let _ = { let assigned = obj1; (*(ctx)).function_ctor = assigned; assigned };
vm_block = 78; continue;
}
// C line 56325
80 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56324
81 => {
vm_block = if (JS_IsException(obj1)) != 0 { 80 } else { 79 }; continue;
}
// C line 56318
82 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_BYTECODE_FUNCTION as i32), c"Function".as_ptr(), (ft).generic, (1 as i32), (((JS_CFUNC_constructor_or_func_magic as i32)) as JSCFunctionEnum), (JS_FUNC_NORMAL as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, core::ptr::null_mut::<JSCFunctionListEntry>(), (0 as i32), ptr::addr_of!(js_function_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 8]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((1 as i32)).wrapping_shl(((2 as i32)) as u32)); obj1 = assigned; assigned };
vm_block = 81; continue;
}
// C line 56317
83 => {
let _ = { let assigned = js_function_constructor; (ft).generic_magic = Some(assigned); assigned };
vm_block = 82; continue;
}
// C line 56314
84 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 83; continue;
}
// C line 56313
85 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56312
86 => {
vm_block = if (JS_IsException(obj1)) != 0 { 85 } else { 84 }; continue;
}
// C line 56306
87 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_OBJECT as i32), c"Object".as_ptr(), Some(js_object_constructor), (1 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_object_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 23]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_object_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 11]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((1 as i32)).wrapping_shl(((2 as i32)) as u32)); obj1 = assigned; assigned };
vm_block = 86; continue;
}
// C line 56303
88 => {
let _ = JS_FreeValue(ctx, js_object_seal(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!((*(ctx)).throw_type_error), (1 as i32)));
vm_block = 87; continue;
}
// C line 56302
89 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56298
90 => {
vm_block = if ((((JS_DefineProperty(ctx, (*(ctx)).function_proto, (((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (*(ctx)).throw_type_error, (*(ctx)).throw_type_error, ((((((((1 as i32)).wrapping_shl(((11 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 89 } else { 88 }; continue;
}
// C line 56297
91 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56293
92 => {
vm_block = if ((((JS_DefineProperty(ctx, (*(ctx)).function_proto, (((crate::quickjs_atom::JS_ATOM_caller as i32)) as JSAtom), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (*(ctx)).throw_type_error, (*(ctx)).throw_type_error, ((((((((1 as i32)).wrapping_shl(((11 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 91 } else { 90 }; continue;
}
// C line 56291
93 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56290
94 => {
vm_block = if (JS_IsException((*(ctx)).throw_type_error)) != 0 { 93 } else { 92 }; continue;
}
// C line 56289
95 => {
let _ = { let assigned = JS_NewCFunction(ctx, Some(js_throw_type_error), core::ptr::null_mut::<c_char>(), (0 as i32)); (*(ctx)).throw_type_error = assigned; assigned };
vm_block = 94; continue;
}
_ => std::process::abort(),
} }
}
