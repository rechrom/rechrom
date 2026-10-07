// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:11709. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_new_di(mut ctx: *mut JSContext, mut a: js_sdlimb_t) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 11724
1 => {
return r;
}
// C line 11716
2 => {
let _ = { let assigned = ((a) as js_limb_t); *(((*(r)).tab).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 11715
3 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 11714
4 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 11713
5 => {
let _ = { let assigned = js_bigint_new(ctx, (1 as i32)); r = assigned; assigned };
vm_block = 4; continue;
}
// C line 11722
6 => {
let _ = { let assigned = (((a).wrapping_shr(((64 as i32)) as u32)) as js_limb_t); *(((*(r)).tab).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 11721
7 => {
let _ = { let assigned = ((a) as js_limb_t); *(((*(r)).tab).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 11720
8 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 11719
9 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 11718
10 => {
let _ = { let assigned = js_bigint_new(ctx, (2 as i32)); r = assigned; assigned };
vm_block = 9; continue;
}
// C line 11712
11 => {
vm_block = if ((((a) == (((((a) as js_slimb_t)) as js_sdlimb_t))) as i32)) != 0 { 5 } else { 10 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:11860. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_mul(mut ctx: *mut JSContext, mut a: *const JSBigInt, mut b: *const JSBigInt) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 11875
1 => {
return js_bigint_normalize(ctx, r);
}
// C line 11874
2 => {
let _ = mp_sub((((*(r)).tab).as_mut_ptr()).offset((((*(b)).len) as isize)), (((*(r)).tab).as_mut_ptr()).offset((((*(b)).len) as isize)), ((*(a)).tab).as_ptr(), (((*(a)).len) as i32), (((0 as i32)) as js_limb_t));
vm_block = 1; continue;
}
// C line 11873
3 => {
vm_block = if (js_bigint_sign(b)) != 0 { 2 } else { 1 }; continue;
}
// C line 11872
4 => {
let _ = mp_sub((((*(r)).tab).as_mut_ptr()).offset((((*(a)).len) as isize)), (((*(r)).tab).as_mut_ptr()).offset((((*(a)).len) as isize)), ((*(b)).tab).as_ptr(), (((*(b)).len) as i32), (((0 as i32)) as js_limb_t));
vm_block = 3; continue;
}
// C line 11871
5 => {
vm_block = if (js_bigint_sign(a)) != 0 { 4 } else { 3 }; continue;
}
// C line 11868
6 => {
let _ = mp_mul_basecase(((*(r)).tab).as_mut_ptr(), ((*(a)).tab).as_ptr(), (((*(a)).len) as js_limb_t), ((*(b)).tab).as_ptr(), (((*(b)).len) as js_limb_t));
vm_block = 5; continue;
}
// C line 11867
7 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 11866
8 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 11865
9 => {
let _ = { let assigned = js_bigint_new(ctx, ((((*(a)).len).wrapping_add((*(b)).len)) as i32)); r = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:11880. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_divrem(mut ctx: *mut JSContext, mut a: *const JSBigInt, mut b: *const JSBigInt, mut is_rem: i32) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut q: *mut JSBigInt = core::mem::zeroed();
let mut tabb: *mut js_limb_t = core::mem::zeroed();
let mut h: js_limb_t = core::mem::zeroed();
let mut na: i32 = core::mem::zeroed();
let mut nb: i32 = core::mem::zeroed();
let mut a_sign: i32 = core::mem::zeroed();
let mut b_sign: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut vm_block: usize = 62;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 11969
1 => {
return r;
}
// C line 11968
2 => {
let _ = { let assigned = js_bigint_normalize1(ctx, r, nb); r = assigned; assigned };
vm_block = 1; continue;
}
// C line 11967
3 => {
let _ = mp_neg(((*(r)).tab).as_mut_ptr(), ((*(r)).tab).as_mut_ptr(), nb);
vm_block = 2; continue;
}
// C line 11966
4 => {
vm_block = if (a_sign) != 0 { 3 } else { 2 }; continue;
}
// C line 11965
5 => {
let _ = { let assigned = (((0 as i32)) as js_limb_t); *(((*(r)).tab).as_mut_ptr()).offset(({ let old = nb; nb = (nb).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 11964
6 => {
let _ = mp_shr(((*(r)).tab).as_mut_ptr(), ((*(r)).tab).as_mut_ptr(), nb, shift, (((0 as i32)) as js_limb_t));
vm_block = 5; continue;
}
// C line 11963
7 => {
vm_block = if ((((shift) != ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 11962
8 => {
let _ = js_free(ctx, ((q) as *mut c_void));
vm_block = 7; continue;
}
// C line 11977
9 => {
return q;
}
// C line 11976
10 => {
let _ = { let assigned = js_bigint_normalize(ctx, q); q = assigned; assigned };
vm_block = 9; continue;
}
// C line 11974
11 => {
let _ = mp_neg(((*(q)).tab).as_mut_ptr(), ((*(q)).tab).as_mut_ptr(), (((*(q)).len) as i32));
vm_block = 10; continue;
}
// C line 11973
12 => {
vm_block = if (((a_sign) ^ (b_sign))) != 0 { 11 } else { 10 }; continue;
}
// C line 11972
13 => {
let _ = { let assigned = (((0 as i32)) as js_limb_t); *(((*(q)).tab).as_mut_ptr()).offset((((na).wrapping_sub(nb)).wrapping_add((1 as i32))) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 11971
14 => {
let _ = js_free(ctx, ((r) as *mut c_void));
vm_block = 13; continue;
}
// C line 11961
15 => {
vm_block = if (is_rem) != 0 { 8 } else { 14 }; continue;
}
// C line 11959
16 => {
let _ = js_free(ctx, ((tabb) as *mut c_void));
vm_block = 15; continue;
}
// C line 11958
17 => {
let _ = mp_divnorm(((*(q)).tab).as_mut_ptr(), ((*(r)).tab).as_mut_ptr(), ((na) as js_limb_t), tabb, ((nb) as js_limb_t));
vm_block = 16; continue;
}
// C line 11953
18 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 11952
19 => {
let _ = js_free(ctx, ((tabb) as *mut c_void));
vm_block = 18; continue;
}
// C line 11951
20 => {
let _ = js_free(ctx, ((r) as *mut c_void));
vm_block = 19; continue;
}
// C line 11950
21 => {
vm_block = if ((!(!(q).is_null()) as i32)) != 0 { 20 } else { 17 }; continue;
}
// C line 11949
22 => {
let _ = { let assigned = js_bigint_new(ctx, ((na).wrapping_sub(nb)).wrapping_add((2 as i32))); q = assigned; assigned };
vm_block = 21; continue;
}
// C line 11946
23 => {
let _ = { let assigned = h; *(((*(r)).tab).as_mut_ptr()).offset(({ let old = na; na = (na).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 22; continue;
}
// C line 11945
24 => {
vm_block = if ((((h) != ((((0 as i32)) as js_limb_t))) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 11944
25 => {
let _ = { let assigned = mp_shl(((*(r)).tab).as_mut_ptr(), ((*(r)).tab).as_mut_ptr(), na, shift); h = assigned; assigned };
vm_block = 24; continue;
}
// C line 11943
26 => {
let _ = mp_shl(tabb, tabb, nb, shift);
vm_block = 25; continue;
}
// C line 11942
27 => {
vm_block = if ((((shift) != ((0 as i32))) as i32)) != 0 { 26 } else { 22 }; continue;
}
// C line 11941
28 => {
let _ = { let assigned = ((js_limb_clz(*(tabb).offset(((nb).wrapping_sub((1 as i32))) as isize))) as i32); shift = assigned; assigned };
vm_block = 27; continue;
}
// C line 11933
29 => {
return r;
}
// C line 11932
30 => {
let _ = { let dst = (((((*(r)).tab).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(a)).tab).as_ptr()) as *const c_void)) as *const u8, dst, (((((*(a)).len) as usize)).wrapping_mul((size_of::<js_limb_t>() as usize))) as usize); dst as *mut c_void };
vm_block = 29; continue;
}
// C line 11931
31 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 11930
32 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 11929
33 => {
let _ = { let assigned = js_bigint_new(ctx, (((*(a)).len) as i32)); r = assigned; assigned };
vm_block = 32; continue;
}
// C line 11936
34 => {
return js_bigint_new_si(ctx, (((0 as i32)) as js_slimb_t));
}
// C line 11927
35 => {
vm_block = if (is_rem) != 0 { 33 } else { 34 }; continue;
}
// C line 11926
36 => {
let _ = js_free(ctx, ((tabb) as *mut c_void));
vm_block = 35; continue;
}
// C line 11925
37 => {
let _ = js_free(ctx, ((r) as *mut c_void));
vm_block = 36; continue;
}
// C line 11924
38 => {
vm_block = if ((((na) < (nb)) as i32)) != 0 { 37 } else { 28 }; continue;
}
// C line 11920
39 => {
vm_block = if ((((((((nb) > ((1 as i32))) as i32)) != 0) && (((((*(tabb).offset(((nb).wrapping_sub((1 as i32))) as isize)) == ((((0 as i32)) as js_limb_t))) as i32)) != 0)) as i32)) != 0 { 40 } else { 38 }; continue;
}
// C line 11921
40 => {
let _ = { let old = nb; nb = (nb).wrapping_sub(1); old };
vm_block = 39; continue;
}
// C line 11915
41 => {
let _ = mp_neg(tabb, ((*(b)).tab).as_ptr(), nb);
vm_block = 39; continue;
}
// C line 11917
42 => {
let _ = { let dst = (((tabb) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(b)).tab).as_ptr()) as *const c_void)) as *const u8, dst, ((((nb) as usize)).wrapping_mul((size_of::<js_limb_t>() as usize))) as usize); dst as *mut c_void };
vm_block = 39; continue;
}
// C line 11914
43 => {
vm_block = if (b_sign) != 0 { 41 } else { 42 }; continue;
}
// C line 11912
44 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 11911
45 => {
let _ = js_free(ctx, ((r) as *mut c_void));
vm_block = 44; continue;
}
// C line 11910
46 => {
vm_block = if ((!(!(tabb).is_null()) as i32)) != 0 { 45 } else { 43 }; continue;
}
// C line 11909
47 => {
let _ = { let assigned = ((js_malloc(ctx, (((nb) as usize)).wrapping_mul((size_of::<js_limb_t>() as usize)))) as *mut js_limb_t); tabb = assigned; assigned };
vm_block = 46; continue;
}
// C line 11906
48 => {
vm_block = if ((((((((na) > ((1 as i32))) as i32)) != 0) && (((((*(((*(r)).tab).as_mut_ptr()).offset(((na).wrapping_sub((1 as i32))) as isize)) == ((((0 as i32)) as js_limb_t))) as i32)) != 0)) as i32)) != 0 { 49 } else { 47 }; continue;
}
// C line 11907
49 => {
let _ = { let old = na; na = (na).wrapping_sub(1); old };
vm_block = 48; continue;
}
// C line 11901
50 => {
let _ = mp_neg(((*(r)).tab).as_mut_ptr(), ((*(a)).tab).as_ptr(), na);
vm_block = 48; continue;
}
// C line 11903
51 => {
let _ = { let dst = (((((*(r)).tab).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(a)).tab).as_ptr()) as *const c_void)) as *const u8, dst, ((((na) as usize)).wrapping_mul((size_of::<js_limb_t>() as usize))) as usize); dst as *mut c_void };
vm_block = 48; continue;
}
// C line 11900
52 => {
vm_block = if (a_sign) != 0 { 50 } else { 51 }; continue;
}
// C line 11899
53 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 11898
54 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 53 } else { 52 }; continue;
}
// C line 11897
55 => {
let _ = { let assigned = js_bigint_new(ctx, (na).wrapping_add((2 as i32))); r = assigned; assigned };
vm_block = 54; continue;
}
// C line 11895
56 => {
let _ = { let assigned = (((*(b)).len) as i32); nb = assigned; assigned };
vm_block = 55; continue;
}
// C line 11894
57 => {
let _ = { let assigned = (((*(a)).len) as i32); na = assigned; assigned };
vm_block = 56; continue;
}
// C line 11893
58 => {
let _ = { let assigned = js_bigint_sign(b); b_sign = assigned; assigned };
vm_block = 57; continue;
}
// C line 11892
59 => {
let _ = { let assigned = js_bigint_sign(a); a_sign = assigned; assigned };
vm_block = 58; continue;
}
// C line 11889
60 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 11888
61 => {
let _ = JS_ThrowRangeError(ctx, c"BigInt division by zero".as_ptr());
vm_block = 60; continue;
}
// C line 11887
62 => {
vm_block = if (((((((((*(b)).len) == ((((1 as i32)) as u32))) as i32)) != 0) && (((((*(((*(b)).tab).as_ptr()).offset(((0 as i32)) as isize)) == ((((0 as i32)) as js_limb_t))) as i32)) != 0)) as i32)) != 0 { 61 } else { 59 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:11982. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_logic(mut ctx: *mut JSContext, mut a: *const JSBigInt, mut b: *const JSBigInt, mut op: i32) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut b_sign: js_limb_t = core::mem::zeroed();
let mut a_len: i32 = core::mem::zeroed();
let mut b_len: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut tmp: *const JSBigInt = core::mem::zeroed();
let mut vm_block: usize = 40;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 12031
1 => {
return js_bigint_normalize(ctx, r);
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 12027
3 => {
vm_block = 1; continue;
}
// C line 12024
4 => {
vm_block = if ((((i) < (a_len)) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 12025
6 => {
let _ = { let assigned = ((*(((*(a)).tab).as_ptr()).offset((i) as isize)) ^ (b_sign)); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 12024
7 => {
let _ = { let assigned = b_len; i = assigned; assigned };
vm_block = 4; continue;
}
// C line 12021
8 => {
vm_block = if ((((i) < (b_len)) as i32)) != 0 { 10 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 12022
10 => {
let _ = { let assigned = ((*(((*(a)).tab).as_ptr()).offset((i) as isize)) ^ (*(((*(b)).tab).as_ptr()).offset((i) as isize))); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 12021
11 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 12019
12 => {
vm_block = 1; continue;
}
// C line 12016
13 => {
vm_block = if ((((i) < (a_len)) as i32)) != 0 { 15 } else { 12 }; continue;
}
// C line ?
14 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 13; continue;
}
// C line 12017
15 => {
let _ = { let assigned = ((*(((*(a)).tab).as_ptr()).offset((i) as isize)) & (b_sign)); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 12016
16 => {
let _ = { let assigned = b_len; i = assigned; assigned };
vm_block = 13; continue;
}
// C line 12013
17 => {
vm_block = if ((((i) < (b_len)) as i32)) != 0 { 19 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 12014
19 => {
let _ = { let assigned = ((*(((*(a)).tab).as_ptr()).offset((i) as isize)) & (*(((*(b)).tab).as_ptr()).offset((i) as isize))); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 12013
20 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 12011
21 => {
vm_block = 1; continue;
}
// C line 12008
22 => {
vm_block = if ((((i) < (a_len)) as i32)) != 0 { 24 } else { 21 }; continue;
}
// C line ?
23 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 22; continue;
}
// C line 12009
24 => {
let _ = { let assigned = ((*(((*(a)).tab).as_ptr()).offset((i) as isize)) | (b_sign)); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 23; continue;
}
// C line 12008
25 => {
let _ = { let assigned = b_len; i = assigned; assigned };
vm_block = 22; continue;
}
// C line 12005
26 => {
vm_block = if ((((i) < (b_len)) as i32)) != 0 { 28 } else { 25 }; continue;
}
// C line ?
27 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 12006
28 => {
let _ = { let assigned = ((*(((*(a)).tab).as_ptr()).offset((i) as isize)) | (*(((*(b)).tab).as_ptr()).offset((i) as isize))); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 27; continue;
}
// C line 12005
29 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 26; continue;
}
// C line 12003
30 => {
vm_block = match ((op) as u32) { x if x == (((OP_xor as i32)) as u32) => 11, x if x == (((OP_and as i32)) as u32) => 20, x if x == (((OP_or as i32)) as u32) => 29, _ => 2, }; continue;
}
// C line 12002
31 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12001
32 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 12000
33 => {
let _ = { let assigned = js_bigint_new(ctx, a_len); r = assigned; assigned };
vm_block = 32; continue;
}
// C line 11998
34 => {
let _ = { let assigned = (((js_bigint_sign(b)).wrapping_neg()) as js_limb_t); b_sign = assigned; assigned };
vm_block = 33; continue;
}
// C line 11997
35 => {
let _ = { let assigned = (((*(b)).len) as i32); b_len = assigned; assigned };
vm_block = 34; continue;
}
// C line 11996
36 => {
let _ = { let assigned = (((*(a)).len) as i32); a_len = assigned; assigned };
vm_block = 35; continue;
}
// C line 11993
37 => {
let _ = { let assigned = tmp; b = assigned; assigned };
vm_block = 36; continue;
}
// C line 11992
38 => {
let _ = { let assigned = b; a = assigned; assigned };
vm_block = 37; continue;
}
// C line 11991
39 => {
let _ = { let assigned = a; tmp = assigned; assigned };
vm_block = 38; continue;
}
// C line 11989
40 => {
vm_block = if (((((*(a)).len) < ((*(b)).len)) as i32)) != 0 { 39 } else { 36 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:12034. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_not(mut ctx: *mut JSContext, mut a: *const JSBigInt) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 12046
1 => {
return r;
}
// C line 12042
2 => {
vm_block = if ((((((i) as u32)) < ((*(a)).len)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 12043
4 => {
let _ = { let assigned = (!(*(((*(a)).tab).as_ptr()).offset((i) as isize))); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 12042
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 12041
6 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12040
7 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 12039
8 => {
let _ = { let assigned = js_bigint_new(ctx, (((*(a)).len) as i32)); r = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:12049. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_shl(mut ctx: *mut JSContext, mut a: *const JSBigInt, mut shift1: u32) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut l: js_limb_t = core::mem::zeroed();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 12075
1 => {
return r;
}
// C line 12066
2 => {
vm_block = if ((((((i) as u32)) < ((*(a)).len)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 12067
4 => {
let _ = { let assigned = *(((*(a)).tab).as_ptr()).offset((i) as isize); *(((*(r)).tab).as_mut_ptr()).offset(((i).wrapping_add(d)) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 12066
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 12073
6 => {
let _ = { let assigned = js_bigint_extend(ctx, r, l); r = assigned; assigned };
vm_block = 1; continue;
}
// C line 12072
7 => {
let _ = { l = ((l) | ((((((1 as i32)).wrapping_neg()) as js_limb_t)).wrapping_shl((shift) as u32))); l };
vm_block = 6; continue;
}
// C line 12071
8 => {
vm_block = if (js_bigint_sign(a)) != 0 { 7 } else { 6 }; continue;
}
// C line 12070
9 => {
let _ = { let assigned = mp_shl((((*(r)).tab).as_mut_ptr()).offset(((d) as isize)), ((*(a)).tab).as_ptr(), (((*(a)).len) as i32), shift); l = assigned; assigned };
vm_block = 8; continue;
}
// C line 12065
10 => {
vm_block = if ((((shift) == ((0 as i32))) as i32)) != 0 { 5 } else { 9 }; continue;
}
// C line 12063
11 => {
vm_block = if ((((i) < (d)) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 12064
13 => {
let _ = { let assigned = (((0 as i32)) as js_limb_t); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 12063
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 12062
15 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12061
16 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 12060
17 => {
let _ = { let assigned = js_bigint_new(ctx, ((((*(a)).len).wrapping_add(((d) as u32))) as i32)); r = assigned; assigned };
vm_block = 16; continue;
}
// C line 12059
18 => {
let _ = { let assigned = ((((shift1) % ((((64 as i32)) as u32)))) as i32); shift = assigned; assigned };
vm_block = 17; continue;
}
// C line 12058
19 => {
let _ = { let assigned = ((((shift1) / ((((64 as i32)) as u32)))) as i32); d = assigned; assigned };
vm_block = 18; continue;
}
// C line 12057
20 => {
return js_bigint_new_si(ctx, (((0 as i32)) as js_slimb_t));
}
// C line 12056
21 => {
vm_block = if (((((((((*(a)).len) == ((((1 as i32)) as u32))) as i32)) != 0) && (((((*(((*(a)).tab).as_ptr()).offset(((0 as i32)) as isize)) == ((((0 as i32)) as js_limb_t))) as i32)) != 0)) as i32)) != 0 { 20 } else { 19 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:12078. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_shr(mut ctx: *mut JSContext, mut a: *const JSBigInt, mut shift1: u32) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut a_sign: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 12102
1 => {
return r;
}
// C line 12094
2 => {
vm_block = if ((((i) < (n1)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 12095
4 => {
let _ = { let assigned = *(((*(a)).tab).as_ptr()).offset(((i).wrapping_add(d)) as isize); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 12094
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 12100
6 => {
let _ = { let assigned = js_bigint_normalize(ctx, r); r = assigned; assigned };
vm_block = 1; continue;
}
// C line 12099
7 => {
let _ = mp_shr(((*(r)).tab).as_mut_ptr(), (((*(a)).tab).as_ptr()).offset(((d) as isize)), n1, shift, (((a_sign).wrapping_neg()) as js_limb_t));
vm_block = 6; continue;
}
// C line 12093
8 => {
vm_block = if ((((shift) == ((0 as i32))) as i32)) != 0 { 5 } else { 7 }; continue;
}
// C line 12092
9 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12091
10 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 12090
11 => {
let _ = { let assigned = js_bigint_new(ctx, n1); r = assigned; assigned };
vm_block = 10; continue;
}
// C line 12089
12 => {
let _ = { let assigned = ((((*(a)).len).wrapping_sub(((d) as u32))) as i32); n1 = assigned; assigned };
vm_block = 11; continue;
}
// C line 12088
13 => {
return js_bigint_new_si(ctx, (((a_sign).wrapping_neg()) as js_slimb_t));
}
// C line 12087
14 => {
vm_block = if ((((((d) as u32)) >= ((*(a)).len)) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 12086
15 => {
let _ = { let assigned = js_bigint_sign(a); a_sign = assigned; assigned };
vm_block = 14; continue;
}
// C line 12085
16 => {
let _ = { let assigned = ((((shift1) % ((((64 as i32)) as u32)))) as i32); shift = assigned; assigned };
vm_block = 15; continue;
}
// C line 12084
17 => {
let _ = { let assigned = ((((shift1) / ((((64 as i32)) as u32)))) as i32); d = assigned; assigned };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:12105. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_pow(mut ctx: *mut JSContext, mut a: *const JSBigInt, mut b: *mut JSBigInt) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut e: u32 = core::mem::zeroed();
let mut n_bits: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut r1: *mut JSBigInt = core::mem::zeroed();
let mut v: js_limb_t = core::mem::zeroed();
let mut is_neg: i32 = core::mem::zeroed();
let mut e1: u64 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut vm_block: usize = 60;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 12185
1 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line ? labels: overflow
2 => {
let _ = JS_ThrowRangeError(ctx, c"BigInt is too large".as_ptr());
vm_block = 1; continue;
}
// C line 12182
3 => {
return r;
}
// C line 12168
4 => {
vm_block = if ((((i) >= ((0 as i32))) as i32)) != 0 { 16 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_sub(1); old };
vm_block = 4; continue;
}
// C line 12179
6 => {
let _ = { let assigned = r1; r = assigned; assigned };
vm_block = 5; continue;
}
// C line 12178
7 => {
let _ = js_free(ctx, ((r) as *mut c_void));
vm_block = 6; continue;
}
// C line 12177
8 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12176
9 => {
vm_block = if ((!(!(r1).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 12175
10 => {
let _ = { let assigned = js_bigint_mul(ctx, r, a); r1 = assigned; assigned };
vm_block = 9; continue;
}
// C line 12174
11 => {
vm_block = if ((((e).wrapping_shr((i) as u32)) & ((((1 as i32)) as u32)))) != 0 { 10 } else { 5 }; continue;
}
// C line 12173
12 => {
let _ = { let assigned = r1; r = assigned; assigned };
vm_block = 11; continue;
}
// C line 12172
13 => {
let _ = js_free(ctx, ((r) as *mut c_void));
vm_block = 12; continue;
}
// C line 12171
14 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12170
15 => {
vm_block = if ((!(!(r1).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 12169
16 => {
let _ = { let assigned = js_bigint_mul(ctx, r, r); r1 = assigned; assigned };
vm_block = 15; continue;
}
// C line 12168
17 => {
let _ = { let assigned = (n_bits).wrapping_sub((2 as i32)); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 12167
18 => {
let _ = { let dst = (((((*(r)).tab).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(a)).tab).as_ptr()) as *const c_void)) as *const u8, dst, (((((*(a)).len) as usize)).wrapping_mul((size_of::<js_limb_t>() as usize))) as usize); dst as *mut c_void };
vm_block = 17; continue;
}
// C line 12166
19 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12165
20 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 12164
21 => {
let _ = { let assigned = js_bigint_new(ctx, (((*(a)).len) as i32)); r = assigned; assigned };
vm_block = 20; continue;
}
// C line 12162
22 => {
let _ = { let assigned = ((32 as i32)).wrapping_sub(clz32(e)); n_bits = assigned; assigned };
vm_block = 21; continue;
}
// C line 12161
23 => {
let _ = { let assigned = ((*(((*(b)).tab).as_mut_ptr()).offset(((0 as i32)) as isize)) as u32); e = assigned; assigned };
vm_block = 22; continue;
}
// C line 12160
24 => {
vm_block = 2; continue;
}
// C line 12159
25 => {
vm_block = if ((((*(((*(b)).tab).as_mut_ptr()).offset(((0 as i32)) as isize)) > ((((2147483647 as i32)) as js_limb_t))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 12158
26 => {
vm_block = 2; continue;
}
// C line 12157
27 => {
vm_block = if (((((*(b)).len) > ((((1 as i32)) as u32))) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 12118
28 => {
return js_bigint_new_si(ctx, (((1 as i32)) as js_slimb_t));
}
// C line 12154
29 => {
return r;
}
// C line 12152
30 => {
let _ = { let assigned = (((((1 as i32)).wrapping_sub(((2 as i32)).wrapping_mul(is_neg))) as js_limb_t)).wrapping_shl((((e) % ((((64 as i32)) as u32)))) as u32); *(((*(r)).tab).as_mut_ptr()).offset((((e) / ((((64 as i32)) as u32)))) as isize) = assigned; assigned };
vm_block = 29; continue;
}
// C line 12151
31 => {
let _ = { let dst = (((((*(r)).tab).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, (((size_of::<js_limb_t>() as usize)).wrapping_mul((((*(r)).len) as usize))) as usize); dst as *mut c_void };
vm_block = 30; continue;
}
// C line 12150
32 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12149
33 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 12147
34 => {
let _ = { let assigned = js_bigint_new(ctx, (((((((e).wrapping_add((((64 as i32)) as u32))).wrapping_add((((1 as i32)) as u32))).wrapping_sub(((is_neg) as u32))) / ((((64 as i32)) as u32)))) as i32)); r = assigned; assigned };
vm_block = 33; continue;
}
// C line 12146
35 => {
let _ = { let assigned = ((((*(((*(b)).tab).as_mut_ptr()).offset(((0 as i32)) as isize)) & ((((1 as i32)) as js_limb_t)))) as i32); is_neg = assigned; assigned };
vm_block = 34; continue;
}
// C line 12145
36 => {
vm_block = if (is_neg) != 0 { 35 } else { 34 }; continue;
}
// C line 12144
37 => {
let _ = { let assigned = ((e1) as u32); e = assigned; assigned };
vm_block = 36; continue;
}
// C line 12143
38 => {
vm_block = 2; continue;
}
// C line 12142
39 => {
vm_block = if ((((e1) > ((((((((1024 as i32)).wrapping_mul((1024 as i32))) / ((64 as i32)))).wrapping_mul((64 as i32))) as u64))) as i32)) != 0 { 38 } else { 37 }; continue;
}
// C line 12141
40 => {
let _ = { let assigned = (((e) as u64)).wrapping_mul(((n) as u64)); e1 = assigned; assigned };
vm_block = 39; continue;
}
// C line 12140
41 => {
let _ = { let assigned = ((*(((*(b)).tab).as_mut_ptr()).offset(((0 as i32)) as isize)) as u32); e = assigned; assigned };
vm_block = 40; continue;
}
// C line 12139
42 => {
vm_block = 2; continue;
}
// C line 12138
43 => {
vm_block = if ((((*(((*(b)).tab).as_mut_ptr()).offset(((0 as i32)) as isize)) > ((((2147483647 as i32)) as js_limb_t))) as i32)) != 0 { 42 } else { 41 }; continue;
}
// C line 12137
44 => {
vm_block = 2; continue;
}
// C line 12136
45 => {
vm_block = if (((((*(b)).len) > ((((1 as i32)) as u32))) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 12135
46 => {
let _ = { let assigned = (((((((64 as i32)).wrapping_sub((1 as i32))) as js_limb_t)).wrapping_sub(js_limb_clz(v))) as i32); n = assigned; assigned };
vm_block = 45; continue;
}
// C line 12131
47 => {
vm_block = if ((((((v) & ((v).wrapping_sub((((1 as i32)) as js_limb_t))))) == ((((0 as i32)) as js_limb_t))) as i32)) != 0 { 46 } else { 27 }; continue;
}
// C line 12130
48 => {
let _ = { let assigned = (v).wrapping_neg(); v = assigned; assigned };
vm_block = 47; continue;
}
// C line 12129
49 => {
vm_block = if (is_neg) != 0 { 48 } else { 47 }; continue;
}
// C line 12128
50 => {
let _ = { let assigned = (((((v) as js_slimb_t)) < ((((0 as i32)) as js_slimb_t))) as i32); is_neg = assigned; assigned };
vm_block = 49; continue;
}
// C line 12125
51 => {
return js_bigint_new_si(ctx, ((v) as js_slimb_t));
}
// C line 12127
52 => {
return js_bigint_new_si(ctx, ((((((1 as i32)) as js_limb_t)).wrapping_sub(((((2 as i32)) as js_limb_t)).wrapping_mul(((*(((*(b)).tab).as_mut_ptr()).offset(((0 as i32)) as isize)) & ((((1 as i32)) as js_limb_t)))))) as js_slimb_t));
}
// C line 12126
53 => {
vm_block = if ((((v) == (((((1 as i32)).wrapping_neg()) as js_limb_t))) as i32)) != 0 { 52 } else { 50 }; continue;
}
// C line 12124
54 => {
vm_block = if ((((v) <= ((((1 as i32)) as js_limb_t))) as i32)) != 0 { 51 } else { 53 }; continue;
}
// C line 12123
55 => {
let _ = { let assigned = *(((*(a)).tab).as_ptr()).offset(((0 as i32)) as isize); v = assigned; assigned };
vm_block = 54; continue;
}
// C line 12119
56 => {
vm_block = if (((((*(a)).len) == ((((1 as i32)) as u32))) as i32)) != 0 { 55 } else { 27 }; continue;
}
// C line 12116
57 => {
vm_block = if (((((((((*(b)).len) == ((((1 as i32)) as u32))) as i32)) != 0) && (((((*(((*(b)).tab).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((((0 as i32)) as js_limb_t))) as i32)) != 0)) as i32)) != 0 { 28 } else { 56 }; continue;
}
// C line 12114
58 => {
return core::ptr::null_mut::<JSBigInt>();
}
// C line 12113
59 => {
let _ = JS_ThrowRangeError(ctx, c"BigInt negative exponent".as_ptr());
vm_block = 58; continue;
}
// C line 12112
60 => {
vm_block = if (js_bigint_sign(b)) != 0 { 59 } else { 57 }; continue;
}
_ => std::process::abort(),
} }
}
