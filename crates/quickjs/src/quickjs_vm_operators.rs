// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14720. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_unary_arith_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut op: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut tag: u32 = core::mem::zeroed();
let mut buf1: JSBigIntBuf = core::mem::zeroed();
let mut p1: *mut JSBigInt = core::mem::zeroed();
let mut v64: i64 = core::mem::zeroed();
let mut v_1: i64 = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut buf2: JSBigIntBuf = core::mem::zeroed();
let mut p2: *mut JSBigInt = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 74;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 14857
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: exception
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 14854
3 => {
return (0 as i32);
}
// C line 14852
4 => {
vm_block = 3; continue;
}
// C line 14850
5 => {
let _ = { let assigned = __JS_NewFloat64(ctx, d); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line ?
6 => {
let _ = std::process::abort();
vm_block = 5; continue;
}
// C line 14846
7 => {
vm_block = 5; continue;
}
// C line 14845
8 => {
let _ = { let assigned = (-(d)); d = assigned; assigned };
vm_block = 7; continue;
}
// C line 14843
9 => {
vm_block = 5; continue;
}
// C line 14841
10 => {
vm_block = 5; continue;
}
// C line 14840
11 => {
let _ = { d = ((d) + (((v) as f64))); d };
vm_block = 10; continue;
}
// C line 14839
12 => {
let _ = { let assigned = (((((((2 as i32)) as u32)).wrapping_mul((((op) as u32)).wrapping_sub((((OP_dec as i32)) as u32)))).wrapping_sub((((1 as i32)) as u32))) as i32); v = assigned; assigned };
vm_block = 11; continue;
}
// C line 14836
13 => {
vm_block = match ((op) as u32) { x if x == (((OP_neg as i32)) as u32) => 8, x if x == (((OP_plus as i32)) as u32) => 9, x if x == (((OP_dec as i32)) as u32) => 12, x if x == (((OP_inc as i32)) as u32) => 12, _ => 6, }; continue;
}
// C line 14835 labels: handle_float64
14 => {
let _ = { let assigned = ((op1).u).float64; d = assigned; assigned };
vm_block = 13; continue;
}
// C line 14830
15 => {
vm_block = 3; continue;
}
// C line 14828
16 => {
let _ = { let assigned = JS_CompactBigInt(ctx, r); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 14827
17 => {
vm_block = 2; continue;
}
// C line 14826
18 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 14825
19 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 18; continue;
}
// C line ?
20 => {
let _ = std::process::abort();
vm_block = 19; continue;
}
// C line 14821
21 => {
vm_block = 19; continue;
}
// C line 14820
22 => {
let _ = { let assigned = js_bigint_not(ctx, p1); r = assigned; assigned };
vm_block = 21; continue;
}
// C line 14818
23 => {
vm_block = 19; continue;
}
// C line 14817
24 => {
let _ = { let assigned = js_bigint_neg(ctx, p1); r = assigned; assigned };
vm_block = 23; continue;
}
// C line 14815
25 => {
vm_block = 19; continue;
}
// C line 14813
26 => {
let _ = { let assigned = js_bigint_add(ctx, p1, p2, (0 as i32)); r = assigned; assigned };
vm_block = 25; continue;
}
// C line 14812
27 => {
let _ = { let assigned = js_bigint_set_si(core::ptr::addr_of_mut!(buf2), (((((((2 as i32)) as u32)).wrapping_mul((((op) as u32)).wrapping_sub((((OP_dec as i32)) as u32)))).wrapping_sub((((1 as i32)) as u32))) as js_slimb_t)); p2 = assigned; assigned };
vm_block = 26; continue;
}
// C line 14806
28 => {
vm_block = 2; continue;
}
// C line 14805
29 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 28; continue;
}
// C line 14804
30 => {
let _ = JS_ThrowTypeError(ctx, c"bigint argument with unary +".as_ptr());
vm_block = 29; continue;
}
// C line 14802 labels: bigint_slow_case1
31 => {
vm_block = match ((op) as u32) { x if x == (((OP_not as i32)) as u32) => 22, x if x == (((OP_neg as i32)) as u32) => 24, x if x == (((OP_dec as i32)) as u32) => 27, x if x == (((OP_inc as i32)) as u32) => 27, x if x == (((OP_plus as i32)) as u32) => 30, _ => 20, }; continue;
}
// C line 14800
32 => {
let _ = { let assigned = ((((op1).u).ptr) as *mut JSBigInt); p1 = assigned; assigned };
vm_block = 31; continue;
}
// C line 14796
33 => {
vm_block = 3; continue;
}
// C line ?
34 => {
let _ = std::process::abort();
vm_block = 33; continue;
}
// C line 14791
35 => {
vm_block = 33; continue;
}
// C line 14790
36 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, (v_1).wrapping_neg()); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 35; continue;
}
// C line 14788
37 => {
vm_block = 31; continue;
}
// C line ? labels: bigint_slow_case
38 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf1), op1); p1 = assigned; assigned };
vm_block = 37; continue;
}
// C line 14785
39 => {
vm_block = if ((((v_1) == ((((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 38 } else { 36 }; continue;
}
// C line 14784
40 => {
let _ = { let assigned = ((op1).u).short_big_int; v_1 = assigned; assigned };
vm_block = 39; continue;
}
// C line 14782
41 => {
vm_block = 33; continue;
}
// C line 14781
42 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, (v_1).wrapping_sub((((1 as i32)) as i64))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 41; continue;
}
// C line 14780
43 => {
vm_block = 38; continue;
}
// C line 14779
44 => {
vm_block = if ((((v_1) == ((((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 43 } else { 42 }; continue;
}
// C line 14777
45 => {
vm_block = 33; continue;
}
// C line 14776
46 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, (v_1).wrapping_add((((1 as i32)) as i64))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 45; continue;
}
// C line 14775
47 => {
vm_block = 38; continue;
}
// C line 14774
48 => {
vm_block = if ((((v_1) == ((9223372036854775807 as i64))) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 14772
49 => {
vm_block = 2; continue;
}
// C line 14771
50 => {
let _ = JS_ThrowTypeError(ctx, c"bigint argument with unary +".as_ptr());
vm_block = 49; continue;
}
// C line 14769
51 => {
vm_block = match ((op) as u32) { x if x == (((OP_neg as i32)) as u32) => 40, x if x == (((OP_dec as i32)) as u32) => 44, x if x == (((OP_inc as i32)) as u32) => 48, x if x == (((OP_plus as i32)) as u32) => 50, _ => 34, }; continue;
}
// C line 14768
52 => {
let _ = { let assigned = ((op1).u).short_big_int; v_1 = assigned; assigned };
vm_block = 51; continue;
}
// C line 14764
53 => {
vm_block = 3; continue;
}
// C line 14762
54 => {
let _ = { let assigned = JS_NewInt64(ctx, v64); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 53; continue;
}
// C line ?
55 => {
let _ = std::process::abort();
vm_block = 54; continue;
}
// C line 14758
56 => {
vm_block = 54; continue;
}
// C line 14754
57 => {
return (0 as i32);
}
// C line 14753
58 => {
let _ = { let assigned = __JS_NewFloat64(ctx, (-((0 as f64)))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 57; continue;
}
// C line 14756
59 => {
let _ = { let assigned = (v64).wrapping_neg(); v64 = assigned; assigned };
vm_block = 56; continue;
}
// C line 14752
60 => {
vm_block = if ((((v64) == ((((0 as i32)) as i64))) as i32)) != 0 { 58 } else { 59 }; continue;
}
// C line 14750
61 => {
vm_block = 54; continue;
}
// C line 14748
62 => {
vm_block = 54; continue;
}
// C line 14747
63 => {
let _ = { v64 = (v64).wrapping_add(((v) as i64)); v64 };
vm_block = 62; continue;
}
// C line 14746
64 => {
let _ = { let assigned = (((((((2 as i32)) as u32)).wrapping_mul((((op) as u32)).wrapping_sub((((OP_dec as i32)) as u32)))).wrapping_sub((((1 as i32)) as u32))) as i32); v = assigned; assigned };
vm_block = 63; continue;
}
// C line 14743
65 => {
vm_block = match ((op) as u32) { x if x == (((OP_neg as i32)) as u32) => 60, x if x == (((OP_plus as i32)) as u32) => 61, x if x == (((OP_dec as i32)) as u32) => 64, x if x == (((OP_inc as i32)) as u32) => 64, _ => 55, }; continue;
}
// C line 14742
66 => {
let _ = { let assigned = ((((((op1).u).uint64) as i32)) as i64); v64 = assigned; assigned };
vm_block = 65; continue;
}
// C line 14738
67 => {
vm_block = match tag { x if x == (((JS_TAG_BIG_INT as i32)) as u32) => 32, x if x == (((JS_TAG_SHORT_BIG_INT as i32)) as u32) => 52, x if x == (((JS_TAG_INT as i32)) as u32) => 66, _ => 14, }; continue;
}
// C line 14737
68 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 67; continue;
}
// C line 14736
69 => {
vm_block = 2; continue;
}
// C line 14735
70 => {
vm_block = if (JS_IsException(op1)) != 0 { 69 } else { 68 }; continue;
}
// C line 14734
71 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 70; continue;
}
// C line 14733
72 => {
vm_block = 14; continue;
}
// C line 14732
73 => {
vm_block = if (((((((((op1).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 72 } else { 71 }; continue;
}
// C line 14730
74 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 73; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14860. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_post_inc_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut op: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 14874
1 => {
return js_unary_arith_slow(ctx, (sp).offset((((1 as i32)) as isize)), ((((((op) as u32)).wrapping_sub((((OP_post_dec as i32)) as u32))).wrapping_add((((OP_dec as i32)) as u32))) as i32));
}
// C line 14873
2 => {
let _ = { let assigned = JS_DupValue(ctx, op1); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 14872
3 => {
let _ = { let assigned = op1; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 14870
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 14869
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 14868
6 => {
vm_block = if (JS_IsException(op1)) != 0 { 5 } else { 3 }; continue;
}
// C line 14867
7 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 6; continue;
}
// C line 14866
8 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14877. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_not_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut v1: i32 = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 14903
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: exception
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 14900
3 => {
return (0 as i32);
}
// C line 14886
4 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, (!(((op1).u).short_big_int))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 14893
5 => {
let _ = { let assigned = JS_CompactBigInt(ctx, r); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 14892
6 => {
vm_block = 2; continue;
}
// C line 14891
7 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 14890
8 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 7; continue;
}
// C line 14889
9 => {
let _ = { let assigned = js_bigint_not(ctx, ((((op1).u).ptr) as *const JSBigInt)); r = assigned; assigned };
vm_block = 8; continue;
}
// C line 14898
10 => {
let _ = { let assigned = JS_NewInt32(ctx, (!(v1))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 14897
11 => {
vm_block = 2; continue;
}
// C line 14896
12 => {
vm_block = if ((((!(((!((JS_ToInt32Free(ctx, core::ptr::addr_of_mut!(v1), op1)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 11 } else { 10 }; continue;
}
// C line 14887
13 => {
vm_block = if (((((((op1).tag) as i32)) == ((JS_TAG_BIG_INT as i32))) as i32)) != 0 { 9 } else { 12 }; continue;
}
// C line 14885
14 => {
vm_block = if (((((((op1).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 4 } else { 13 }; continue;
}
// C line 14884
15 => {
vm_block = 2; continue;
}
// C line 14883
16 => {
vm_block = if (JS_IsException(op1)) != 0 { 15 } else { 14 }; continue;
}
// C line 14882
17 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 16; continue;
}
// C line 14881
18 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14906. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_binary_arith_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut op: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut tag1: u32 = core::mem::zeroed();
let mut tag2: u32 = core::mem::zeroed();
let mut d1: f64 = core::mem::zeroed();
let mut d2: f64 = core::mem::zeroed();
let mut v1: js_slimb_t = core::mem::zeroed();
let mut v2: js_slimb_t = core::mem::zeroed();
let mut v: js_sdlimb_t = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut v1_1: i32 = core::mem::zeroed();
let mut v2_1: i32 = core::mem::zeroed();
let mut v_1: i64 = core::mem::zeroed();
let mut p1: *mut JSBigInt = core::mem::zeroed();
let mut p2: *mut JSBigInt = core::mem::zeroed();
let mut r_1: *mut JSBigInt = core::mem::zeroed();
let mut buf1: JSBigIntBuf = core::mem::zeroed();
let mut buf2: JSBigIntBuf = core::mem::zeroed();
let mut dr: f64 = core::mem::zeroed();
let mut vm_block: usize = 113;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15090
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15089
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 15086
4 => {
return (0 as i32);
}
// C line 15013
5 => {
let _ = { let assigned = JS_NewInt64(ctx, v_1); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line ?
6 => {
let _ = std::process::abort();
vm_block = 5; continue;
}
// C line 15009
7 => {
return (0 as i32);
}
// C line 15008
8 => {
let _ = { let assigned = JS_NewFloat64(ctx, js_pow(((v1_1) as f64), ((v2_1) as f64))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 15006
9 => {
vm_block = 5; continue;
}
// C line 15002
10 => {
return (0 as i32);
}
// C line 15001
11 => {
let _ = { let assigned = JS_NewFloat64(ctx, (((v1_1) as f64) % ((v2_1) as f64))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 15004
12 => {
let _ = { let assigned = ((((v1_1) as i64)) % (((v2_1) as i64))); v_1 = assigned; assigned };
vm_block = 9; continue;
}
// C line 15000
13 => {
vm_block = if ((((((((v1_1) < ((0 as i32))) as i32)) != 0) || (((((v2_1) <= ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 11 } else { 12 }; continue;
}
// C line 14998
14 => {
return (0 as i32);
}
// C line 14997
15 => {
let _ = { let assigned = JS_NewFloat64(ctx, ((((v1_1) as f64)) / (((v2_1) as f64)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 14995
16 => {
vm_block = 5; continue;
}
// C line 14993
17 => {
return (0 as i32);
}
// C line 14992
18 => {
let _ = { let assigned = __JS_NewFloat64(ctx, (-((0 as f64)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 14991
19 => {
vm_block = if ((((((((v_1) == ((((0 as i32)) as i64))) as i32)) != 0) && (((((((v1_1) | (v2_1))) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 18 } else { 16 }; continue;
}
// C line 14990
20 => {
let _ = { let assigned = (((v1_1) as i64)).wrapping_mul(((v2_1) as i64)); v_1 = assigned; assigned };
vm_block = 19; continue;
}
// C line 14988
21 => {
vm_block = 5; continue;
}
// C line 14987
22 => {
let _ = { let assigned = (((v1_1) as i64)).wrapping_sub(((v2_1) as i64)); v_1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 14985
23 => {
vm_block = match ((op) as u32) { x if x == (((OP_pow as i32)) as u32) => 8, x if x == (((OP_mod as i32)) as u32) => 13, x if x == (((OP_div as i32)) as u32) => 15, x if x == (((OP_mul as i32)) as u32) => 20, x if x == (((OP_sub as i32)) as u32) => 22, _ => 6, }; continue;
}
// C line 14984
24 => {
let _ = { let assigned = ((((op2).u).uint64) as i32); v2_1 = assigned; assigned };
vm_block = 23; continue;
}
// C line 14983
25 => {
let _ = { let assigned = ((((op1).u).uint64) as i32); v1_1 = assigned; assigned };
vm_block = 24; continue;
}
// C line 15054
26 => {
let _ = { let assigned = JS_CompactBigInt(ctx, r_1); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 15053
27 => {
vm_block = 3; continue;
}
// C line 15052
28 => {
vm_block = if ((!(!(r_1).is_null()) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 15051
29 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 28; continue;
}
// C line 15050
30 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 29; continue;
}
// C line ?
31 => {
let _ = std::process::abort();
vm_block = 30; continue;
}
// C line 15046
32 => {
vm_block = 30; continue;
}
// C line 15045
33 => {
let _ = { let assigned = js_bigint_pow(ctx, p1, p2); r_1 = assigned; assigned };
vm_block = 32; continue;
}
// C line 15043
34 => {
vm_block = 30; continue;
}
// C line 15042
35 => {
let _ = { let assigned = js_bigint_divrem(ctx, p1, p2, (1 as i32)); r_1 = assigned; assigned };
vm_block = 34; continue;
}
// C line 15040
36 => {
vm_block = 30; continue;
}
// C line 15039
37 => {
let _ = { let assigned = js_bigint_divrem(ctx, p1, p2, (0 as i32)); r_1 = assigned; assigned };
vm_block = 36; continue;
}
// C line 15037
38 => {
vm_block = 30; continue;
}
// C line 15036
39 => {
let _ = { let assigned = js_bigint_mul(ctx, p1, p2); r_1 = assigned; assigned };
vm_block = 38; continue;
}
// C line 15034
40 => {
vm_block = 30; continue;
}
// C line 15033
41 => {
let _ = { let assigned = js_bigint_add(ctx, p1, p2, (1 as i32)); r_1 = assigned; assigned };
vm_block = 40; continue;
}
// C line 15031
42 => {
vm_block = 30; continue;
}
// C line 15030
43 => {
let _ = { let assigned = js_bigint_add(ctx, p1, p2, (0 as i32)); r_1 = assigned; assigned };
vm_block = 42; continue;
}
// C line 15028
44 => {
vm_block = match ((op) as u32) { x if x == (((OP_pow as i32)) as u32) => 33, x if x == (((OP_mod as i32)) as u32) => 35, x if x == (((OP_div as i32)) as u32) => 37, x if x == (((OP_mul as i32)) as u32) => 39, x if x == (((OP_sub as i32)) as u32) => 41, x if x == (((OP_add as i32)) as u32) => 43, _ => 31, }; continue;
}
// C line 15025
45 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf2), op2); p2 = assigned; assigned };
vm_block = 44; continue;
}
// C line 15027
46 => {
let _ = { let assigned = ((((op2).u).ptr) as *mut JSBigInt); p2 = assigned; assigned };
vm_block = 44; continue;
}
// C line 15024
47 => {
vm_block = if (((((((op2).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 45 } else { 46 }; continue;
}
// C line 15021
48 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf1), op1); p1 = assigned; assigned };
vm_block = 47; continue;
}
// C line 15023
49 => {
let _ = { let assigned = ((((op1).u).ptr) as *mut JSBigInt); p1 = assigned; assigned };
vm_block = 47; continue;
}
// C line 15020 labels: slow_big_int
50 => {
vm_block = if (((((((op1).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 48 } else { 49 }; continue;
}
// C line 15084
51 => {
let _ = { let assigned = __JS_NewFloat64(ctx, dr); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line ?
52 => {
let _ = std::process::abort();
vm_block = 51; continue;
}
// C line 15080
53 => {
vm_block = 51; continue;
}
// C line 15079
54 => {
let _ = { let assigned = js_pow(d1, d2); dr = assigned; assigned };
vm_block = 53; continue;
}
// C line 15077
55 => {
vm_block = 51; continue;
}
// C line 15076
56 => {
let _ = { let assigned = (d1 % d2); dr = assigned; assigned };
vm_block = 55; continue;
}
// C line 15074
57 => {
vm_block = 51; continue;
}
// C line 15073
58 => {
let _ = { let assigned = ((d1) / (d2)); dr = assigned; assigned };
vm_block = 57; continue;
}
// C line 15071
59 => {
vm_block = 51; continue;
}
// C line 15070
60 => {
let _ = { let assigned = ((d1) * (d2)); dr = assigned; assigned };
vm_block = 59; continue;
}
// C line 15068
61 => {
vm_block = 51; continue;
}
// C line 15067
62 => {
let _ = { let assigned = ((d1) - (d2)); dr = assigned; assigned };
vm_block = 61; continue;
}
// C line 15065 labels: handle_float64
63 => {
vm_block = match ((op) as u32) { x if x == (((OP_pow as i32)) as u32) => 54, x if x == (((OP_mod as i32)) as u32) => 56, x if x == (((OP_div as i32)) as u32) => 58, x if x == (((OP_mul as i32)) as u32) => 60, x if x == (((OP_sub as i32)) as u32) => 62, _ => 52, }; continue;
}
// C line 15063
64 => {
vm_block = 3; continue;
}
// C line 15062
65 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d2), op2)) != 0 { 64 } else { 63 }; continue;
}
// C line 15060
66 => {
vm_block = 3; continue;
}
// C line 15059
67 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 66; continue;
}
// C line 15058
68 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d1), op1)) != 0 { 67 } else { 65 }; continue;
}
// C line 15014
69 => {
vm_block = if ((((((((((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag2) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 50 } else { 68 }; continue;
}
// C line 14980
70 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 25 } else { 69 }; continue;
}
// C line 14978
71 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 70; continue;
}
// C line 14977
72 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 71; continue;
}
// C line 14975
73 => {
vm_block = 3; continue;
}
// C line 14974
74 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 73; continue;
}
// C line 14973
75 => {
vm_block = if (JS_IsException(op2)) != 0 { 74 } else { 72 }; continue;
}
// C line 14972
76 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op2); op2 = assigned; assigned };
vm_block = 75; continue;
}
// C line 14970
77 => {
vm_block = 3; continue;
}
// C line 14969
78 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 77; continue;
}
// C line 14968
79 => {
vm_block = if (JS_IsException(op1)) != 0 { 78 } else { 76 }; continue;
}
// C line 14967
80 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 79; continue;
}
// C line 14965
81 => {
return (0 as i32);
}
// C line 14958
82 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, ((v) as i64)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 81; continue;
}
// C line 14963
83 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((r) as *mut c_void) }, tag: (((JS_TAG_BIG_INT as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 81; continue;
}
// C line 14962
84 => {
vm_block = 3; continue;
}
// C line 14961
85 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 84 } else { 83 }; continue;
}
// C line 14960
86 => {
r = js_bigint_new_di(ctx, v);
vm_block = 85; continue;
}
// C line 14957
87 => {
vm_block = if ((((!(((!(((((((((v) >= ((((((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64))) as js_sdlimb_t))) as i32)) != 0) && (((((v) <= ((((9223372036854775807 as i64)) as js_sdlimb_t))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 82 } else { 86 }; continue;
}
// C line ?
88 => {
let _ = std::process::abort();
vm_block = 87; continue;
}
// C line 14953
89 => {
vm_block = 50; continue;
}
// C line 14951
90 => {
return (0 as i32);
}
// C line 14950
91 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, ((v1) % (v2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 90; continue;
}
// C line 14948
92 => {
vm_block = 50; continue;
}
// C line 14945
93 => {
vm_block = if ((((((((v2) == ((((0 as i32)) as js_slimb_t))) as i32)) != 0) || (((((((((((v1) as js_limb_t)) == (((((1 as i32)) as js_limb_t)).wrapping_shl((((64 as i32)).wrapping_sub((1 as i32))) as u32))) as i32)) != 0) && (((((v2) == (((((1 as i32)).wrapping_neg()) as js_slimb_t))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 92 } else { 91 }; continue;
}
// C line 14943
94 => {
return (0 as i32);
}
// C line 14942
95 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, ((v1) / (v2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 94; continue;
}
// C line 14940
96 => {
vm_block = 50; continue;
}
// C line 14937
97 => {
vm_block = if ((((((((v2) == ((((0 as i32)) as js_slimb_t))) as i32)) != 0) || (((((((((((v1) as js_limb_t)) == (((((1 as i32)) as js_limb_t)).wrapping_shl((((64 as i32)).wrapping_sub((1 as i32))) as u32))) as i32)) != 0) && (((((v2) == (((((1 as i32)).wrapping_neg()) as js_slimb_t))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 96 } else { 95 }; continue;
}
// C line 14935
98 => {
vm_block = 87; continue;
}
// C line 14934
99 => {
let _ = { let assigned = (((v1) as js_sdlimb_t)).wrapping_mul(((v2) as js_sdlimb_t)); v = assigned; assigned };
vm_block = 98; continue;
}
// C line 14932
100 => {
vm_block = 87; continue;
}
// C line 14931
101 => {
let _ = { let assigned = (((v1) as js_sdlimb_t)).wrapping_sub(((v2) as js_sdlimb_t)); v = assigned; assigned };
vm_block = 100; continue;
}
// C line 14929
102 => {
vm_block = match ((op) as u32) { x if x == (((OP_pow as i32)) as u32) => 89, x if x == (((OP_mod as i32)) as u32) => 93, x if x == (((OP_div as i32)) as u32) => 97, x if x == (((OP_mul as i32)) as u32) => 99, x if x == (((OP_sub as i32)) as u32) => 101, _ => 88, }; continue;
}
// C line 14928
103 => {
let _ = { let assigned = ((op2).u).short_big_int; v2 = assigned; assigned };
vm_block = 102; continue;
}
// C line 14927
104 => {
let _ = { let assigned = ((op1).u).short_big_int; v1 = assigned; assigned };
vm_block = 103; continue;
}
// C line 14924
105 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 104 } else { 80 }; continue;
}
// C line 14921
106 => {
vm_block = 63; continue;
}
// C line 14920
107 => {
let _ = { let assigned = ((op2).u).float64; d2 = assigned; assigned };
vm_block = 106; continue;
}
// C line 14919
108 => {
let _ = { let assigned = ((op1).u).float64; d1 = assigned; assigned };
vm_block = 107; continue;
}
// C line 14918
109 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 108 } else { 105 }; continue;
}
// C line 14916
110 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 109; continue;
}
// C line 14915
111 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 110; continue;
}
// C line 14914
112 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 111; continue;
}
// C line 14913
113 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 112; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15098. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_add_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut tag1: u32 = core::mem::zeroed();
let mut tag2: u32 = core::mem::zeroed();
let mut d1: f64 = core::mem::zeroed();
let mut d2: f64 = core::mem::zeroed();
let mut v1: js_slimb_t = core::mem::zeroed();
let mut v2: js_slimb_t = core::mem::zeroed();
let mut v: js_sdlimb_t = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut v1_1: i32 = core::mem::zeroed();
let mut v2_1: i32 = core::mem::zeroed();
let mut v_1: i64 = core::mem::zeroed();
let mut p1: *mut JSBigInt = core::mem::zeroed();
let mut p2: *mut JSBigInt = core::mem::zeroed();
let mut r_1: *mut JSBigInt = core::mem::zeroed();
let mut buf1: JSBigIntBuf = core::mem::zeroed();
let mut buf2: JSBigIntBuf = core::mem::zeroed();
let mut d1_1: f64 = core::mem::zeroed();
let mut d2_1: f64 = core::mem::zeroed();
let mut vm_block: usize = 74;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15211
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15210
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 15207
4 => {
return (0 as i32);
}
// C line 15176
5 => {
let _ = { let assigned = JS_NewInt64(ctx, v_1); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 15175
6 => {
let _ = { let assigned = (((v1_1) as i64)).wrapping_add(((v2_1) as i64)); v_1 = assigned; assigned };
vm_block = 5; continue;
}
// C line 15174
7 => {
let _ = { let assigned = ((((op2).u).uint64) as i32); v2_1 = assigned; assigned };
vm_block = 6; continue;
}
// C line 15173
8 => {
let _ = { let assigned = ((((op1).u).uint64) as i32); v1_1 = assigned; assigned };
vm_block = 7; continue;
}
// C line 15195
9 => {
let _ = { let assigned = JS_CompactBigInt(ctx, r_1); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 15194
10 => {
vm_block = 3; continue;
}
// C line 15193
11 => {
vm_block = if ((!(!(r_1).is_null()) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 15192
12 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 11; continue;
}
// C line 15191
13 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 12; continue;
}
// C line 15190
14 => {
let _ = { let assigned = js_bigint_add(ctx, p1, p2, (0 as i32)); r_1 = assigned; assigned };
vm_block = 13; continue;
}
// C line 15187
15 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf2), op2); p2 = assigned; assigned };
vm_block = 14; continue;
}
// C line 15189
16 => {
let _ = { let assigned = ((((op2).u).ptr) as *mut JSBigInt); p2 = assigned; assigned };
vm_block = 14; continue;
}
// C line 15186
17 => {
vm_block = if (((((((op2).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 15 } else { 16 }; continue;
}
// C line 15183
18 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf1), op1); p1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 15185
19 => {
let _ = { let assigned = ((((op1).u).ptr) as *mut JSBigInt); p1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 15182
20 => {
vm_block = if (((((((op1).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 18 } else { 19 }; continue;
}
// C line 15205
21 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1_1) + (d2_1))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 15204
22 => {
vm_block = 3; continue;
}
// C line 15203
23 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d2_1), op2)) != 0 { 22 } else { 21 }; continue;
}
// C line 15201
24 => {
vm_block = 3; continue;
}
// C line 15200
25 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 24; continue;
}
// C line 15199
26 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d1_1), op1)) != 0 { 25 } else { 23 }; continue;
}
// C line 15177
27 => {
vm_block = if ((((((((((((tag1) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((((((tag2) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 20 } else { 26 }; continue;
}
// C line 15170
28 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 8 } else { 27 }; continue;
}
// C line 15168
29 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 28; continue;
}
// C line 15167
30 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 29; continue;
}
// C line 15165
31 => {
vm_block = 3; continue;
}
// C line 15164
32 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 31; continue;
}
// C line 15163
33 => {
vm_block = if (JS_IsException(op2)) != 0 { 32 } else { 30 }; continue;
}
// C line 15162
34 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op2); op2 = assigned; assigned };
vm_block = 33; continue;
}
// C line 15160
35 => {
vm_block = 3; continue;
}
// C line 15159
36 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 35; continue;
}
// C line 15158
37 => {
vm_block = if (JS_IsException(op1)) != 0 { 36 } else { 34 }; continue;
}
// C line 15157
38 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 37; continue;
}
// C line 15154
39 => {
return (0 as i32);
}
// C line 15153
40 => {
vm_block = 3; continue;
}
// C line 15152
41 => {
vm_block = if (JS_IsException(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0 { 40 } else { 39 }; continue;
}
// C line 15151
42 => {
let _ = { let assigned = JS_ConcatString(ctx, op1, op2); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 41; continue;
}
// C line 15150
43 => {
vm_block = if (((((tag_is_string(tag1)) != 0) || ((tag_is_string(tag2)) != 0)) as i32)) != 0 { 42 } else { 38 }; continue;
}
// C line 15147
44 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 43; continue;
}
// C line 15146
45 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 44; continue;
}
// C line 15144
46 => {
vm_block = 3; continue;
}
// C line 15143
47 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 46; continue;
}
// C line 15142
48 => {
vm_block = if (JS_IsException(op2)) != 0 { 47 } else { 45 }; continue;
}
// C line 15141
49 => {
let _ = { let assigned = JS_ToPrimitiveFree(ctx, op2, (2 as i32)); op2 = assigned; assigned };
vm_block = 48; continue;
}
// C line 15138
50 => {
vm_block = 3; continue;
}
// C line 15137
51 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 50; continue;
}
// C line 15136
52 => {
vm_block = if (JS_IsException(op1)) != 0 { 51 } else { 49 }; continue;
}
// C line 15135
53 => {
let _ = { let assigned = JS_ToPrimitiveFree(ctx, op1, (2 as i32)); op1 = assigned; assigned };
vm_block = 52; continue;
}
// C line 15134
54 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_OBJECT as i32)) as u32))) as i32)) != 0) || (((((tag2) == ((((JS_TAG_OBJECT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 53 } else { 43 }; continue;
}
// C line 15131
55 => {
return (0 as i32);
}
// C line 15124
56 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, ((v) as i64)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 55; continue;
}
// C line 15129
57 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((r) as *mut c_void) }, tag: (((JS_TAG_BIG_INT as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 55; continue;
}
// C line 15128
58 => {
vm_block = 3; continue;
}
// C line 15127
59 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 58 } else { 57 }; continue;
}
// C line 15126
60 => {
r = js_bigint_new_di(ctx, v);
vm_block = 59; continue;
}
// C line 15123
61 => {
vm_block = if ((((!(((!(((((((((v) >= ((((((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64))) as js_sdlimb_t))) as i32)) != 0) && (((((v) <= ((((9223372036854775807 as i64)) as js_sdlimb_t))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 56 } else { 60 }; continue;
}
// C line 15122
62 => {
let _ = { let assigned = (((v1) as js_sdlimb_t)).wrapping_add(((v2) as js_sdlimb_t)); v = assigned; assigned };
vm_block = 61; continue;
}
// C line 15121
63 => {
let _ = { let assigned = ((op2).u).short_big_int; v2 = assigned; assigned };
vm_block = 62; continue;
}
// C line 15120
64 => {
let _ = { let assigned = ((op1).u).short_big_int; v1 = assigned; assigned };
vm_block = 63; continue;
}
// C line 15117
65 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 64 } else { 54 }; continue;
}
// C line 15114
66 => {
return (0 as i32);
}
// C line 15113
67 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1) + (d2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 66; continue;
}
// C line 15112
68 => {
let _ = { let assigned = ((op2).u).float64; d2 = assigned; assigned };
vm_block = 67; continue;
}
// C line 15111
69 => {
let _ = { let assigned = ((op1).u).float64; d1 = assigned; assigned };
vm_block = 68; continue;
}
// C line 15109
70 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 69 } else { 65 }; continue;
}
// C line 15107
71 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 70; continue;
}
// C line 15106
72 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 71; continue;
}
// C line 15104
73 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 72; continue;
}
// C line 15103
74 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 73; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15214. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_binary_logic_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut op: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut tag1: u32 = core::mem::zeroed();
let mut tag2: u32 = core::mem::zeroed();
let mut v1: u32 = core::mem::zeroed();
let mut v2: u32 = core::mem::zeroed();
let mut r: u32 = core::mem::zeroed();
let mut v1_1: js_slimb_t = core::mem::zeroed();
let mut v2_1: js_slimb_t = core::mem::zeroed();
let mut v: js_slimb_t = core::mem::zeroed();
let mut vd: js_sdlimb_t = core::mem::zeroed();
let mut r_1: *mut JSBigInt = core::mem::zeroed();
let mut p1: *mut JSBigInt = core::mem::zeroed();
let mut p2: *mut JSBigInt = core::mem::zeroed();
let mut r_2: *mut JSBigInt = core::mem::zeroed();
let mut buf1: JSBigIntBuf = core::mem::zeroed();
let mut buf2: JSBigIntBuf = core::mem::zeroed();
let mut shift: js_slimb_t = core::mem::zeroed();
let mut vm_block: usize = 101;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15372
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15371
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 15368
4 => {
return (0 as i32);
}
// C line 15339
5 => {
let _ = { let assigned = JS_CompactBigInt(ctx, r_2); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 15338
6 => {
vm_block = 3; continue;
}
// C line 15337
7 => {
vm_block = if ((!(!(r_2).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 15336
8 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 7; continue;
}
// C line 15335
9 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 8; continue;
}
// C line ?
10 => {
let _ = std::process::abort();
vm_block = 9; continue;
}
// C line 15331
11 => {
vm_block = 9; continue;
}
// C line 15327
12 => {
let _ = { let assigned = js_bigint_shl(ctx, p1, ((shift) as u32)); r_2 = assigned; assigned };
vm_block = 11; continue;
}
// C line 15329
13 => {
let _ = { let assigned = js_bigint_shr(ctx, p1, (((shift).wrapping_neg()) as u32)); r_2 = assigned; assigned };
vm_block = 11; continue;
}
// C line 15326
14 => {
vm_block = if ((((shift) >= ((((0 as i32)) as js_slimb_t))) as i32)) != 0 { 12 } else { 13 }; continue;
}
// C line 15325
15 => {
let _ = { let assigned = (shift).wrapping_neg(); shift = assigned; assigned };
vm_block = 14; continue;
}
// C line 15324
16 => {
vm_block = if ((((((op) as u32)) == ((((OP_sar as i32)) as u32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 15321
17 => {
let _ = { let assigned = (((2147483647 as i32)) as js_slimb_t); shift = assigned; assigned };
vm_block = 16; continue;
}
// C line 15323
18 => {
let _ = { let assigned = ((((2147483647 as i32)).wrapping_neg()) as js_slimb_t); shift = assigned; assigned };
vm_block = 16; continue;
}
// C line 15322
19 => {
vm_block = if ((((shift) < (((((2147483647 as i32)).wrapping_neg()) as js_slimb_t))) as i32)) != 0 { 18 } else { 16 }; continue;
}
// C line 15320
20 => {
vm_block = if ((((shift) > ((((2147483647 as i32)) as js_slimb_t))) as i32)) != 0 { 17 } else { 19 }; continue;
}
// C line 15319
21 => {
let _ = { let assigned = js_bigint_get_si_sat(p2); shift = assigned; assigned };
vm_block = 20; continue;
}
// C line 15314
22 => {
vm_block = 9; continue;
}
// C line 15313
23 => {
let _ = { let assigned = js_bigint_logic(ctx, p1, p2, op); r_2 = assigned; assigned };
vm_block = 22; continue;
}
// C line 15309
24 => {
vm_block = match ((op) as u32) { x if x == (((OP_sar as i32)) as u32) => 21, x if x == (((OP_shl as i32)) as u32) => 21, x if x == (((OP_xor as i32)) as u32) => 23, x if x == (((OP_or as i32)) as u32) => 23, x if x == (((OP_and as i32)) as u32) => 23, _ => 10, }; continue;
}
// C line 15306
25 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf2), op2); p2 = assigned; assigned };
vm_block = 24; continue;
}
// C line 15308
26 => {
let _ = { let assigned = ((((op2).u).ptr) as *mut JSBigInt); p2 = assigned; assigned };
vm_block = 24; continue;
}
// C line 15305
27 => {
vm_block = if (((((((op2).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 25 } else { 26 }; continue;
}
// C line 15302
28 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf1), op1); p1 = assigned; assigned };
vm_block = 27; continue;
}
// C line 15304
29 => {
let _ = { let assigned = ((((op1).u).ptr) as *mut JSBigInt); p1 = assigned; assigned };
vm_block = 27; continue;
}
// C line 15301 labels: slow_big_int
30 => {
vm_block = if (((((((op1).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 28 } else { 29 }; continue;
}
// C line 15366
31 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line ?
32 => {
let _ = std::process::abort();
vm_block = 31; continue;
}
// C line 15362
33 => {
vm_block = 31; continue;
}
// C line 15361
34 => {
let _ = { let assigned = ((v1) ^ (v2)); r = assigned; assigned };
vm_block = 33; continue;
}
// C line 15359
35 => {
vm_block = 31; continue;
}
// C line 15358
36 => {
let _ = { let assigned = ((v1) | (v2)); r = assigned; assigned };
vm_block = 35; continue;
}
// C line 15356
37 => {
vm_block = 31; continue;
}
// C line 15355
38 => {
let _ = { let assigned = ((v1) & (v2)); r = assigned; assigned };
vm_block = 37; continue;
}
// C line 15353
39 => {
vm_block = 31; continue;
}
// C line 15352
40 => {
let _ = { let assigned = (((((v1) as i32)).wrapping_shr((((v2) & ((((31 as i32)) as u32)))) as u32)) as u32); r = assigned; assigned };
vm_block = 39; continue;
}
// C line 15350
41 => {
vm_block = 31; continue;
}
// C line 15349
42 => {
let _ = { let assigned = (v1).wrapping_shl((((v2) & ((((31 as i32)) as u32)))) as u32); r = assigned; assigned };
vm_block = 41; continue;
}
// C line 15347
43 => {
vm_block = match ((op) as u32) { x if x == (((OP_xor as i32)) as u32) => 34, x if x == (((OP_or as i32)) as u32) => 36, x if x == (((OP_and as i32)) as u32) => 38, x if x == (((OP_sar as i32)) as u32) => 40, x if x == (((OP_shl as i32)) as u32) => 42, _ => 32, }; continue;
}
// C line 15346
44 => {
vm_block = 3; continue;
}
// C line 15345
45 => {
vm_block = if ((((!(((!((JS_ToInt32Free(ctx, ((core::ptr::addr_of_mut!(v2)) as *mut i32), op2)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 44 } else { 43 }; continue;
}
// C line 15343
46 => {
vm_block = 3; continue;
}
// C line 15342
47 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 46; continue;
}
// C line 15341
48 => {
vm_block = if ((((!(((!((JS_ToInt32Free(ctx, ((core::ptr::addr_of_mut!(v1)) as *mut i32), op1)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 47 } else { 45 }; continue;
}
// C line 15296
49 => {
vm_block = if ((((((((((((tag1) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((((((tag2) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 30 } else { 48 }; continue;
}
// C line 15295
50 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 49; continue;
}
// C line 15294
51 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 50; continue;
}
// C line 15291
52 => {
vm_block = 3; continue;
}
// C line 15290
53 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 52; continue;
}
// C line 15289
54 => {
vm_block = if (JS_IsException(op2)) != 0 { 53 } else { 51 }; continue;
}
// C line 15288
55 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op2); op2 = assigned; assigned };
vm_block = 54; continue;
}
// C line 15286
56 => {
vm_block = 3; continue;
}
// C line 15285
57 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 56; continue;
}
// C line 15284
58 => {
vm_block = if (JS_IsException(op1)) != 0 { 57 } else { 55 }; continue;
}
// C line 15283
59 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 58; continue;
}
// C line 15281
60 => {
return (0 as i32);
}
// C line 15280
61 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, v); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 60; continue;
}
// C line ?
62 => {
let _ = std::process::abort();
vm_block = 61; continue;
}
// C line 15276
63 => {
vm_block = 61; continue;
}
// C line 15268
64 => {
let _ = { let assigned = ((vd) as js_slimb_t); v = assigned; assigned };
vm_block = 63; continue;
}
// C line 15274
65 => {
return (0 as i32);
}
// C line 15273
66 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((r_1) as *mut c_void) }, tag: (((JS_TAG_BIG_INT as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 65; continue;
}
// C line 15272
67 => {
vm_block = 3; continue;
}
// C line 15271
68 => {
vm_block = if ((!(!(r_1).is_null()) as i32)) != 0 { 67 } else { 66 }; continue;
}
// C line 15270
69 => {
r_1 = js_bigint_new_di(ctx, vd);
vm_block = 68; continue;
}
// C line 15266
70 => {
vm_block = if ((((!(((!(((((((((vd) >= ((((((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64))) as js_sdlimb_t))) as i32)) != 0) && (((((vd) <= ((((9223372036854775807 as i64)) as js_sdlimb_t))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 64 } else { 69 }; continue;
}
// C line ? labels: bigint_shl
71 => {
let _ = { let assigned = (((((v1_1) as js_dlimb_t)).wrapping_shl((v2_1) as u32)) as js_sdlimb_t); vd = assigned; assigned };
vm_block = 70; continue;
}
// C line 15257
72 => {
vm_block = 30; continue;
}
// C line 15262
73 => {
vm_block = 80; continue;
}
// C line 15261
74 => {
let _ = { let assigned = (v2_1).wrapping_neg(); v2_1 = assigned; assigned };
vm_block = 73; continue;
}
// C line 15260
75 => {
vm_block = 30; continue;
}
// C line 15259
76 => {
vm_block = if ((((v2_1) < ((((((64 as i32)).wrapping_sub((1 as i32))).wrapping_neg()) as js_slimb_t))) as i32)) != 0 { 75 } else { 74 }; continue;
}
// C line 15258
77 => {
vm_block = if ((((v2_1) < ((((0 as i32)) as js_slimb_t))) as i32)) != 0 { 76 } else { 71 }; continue;
}
// C line 15256
78 => {
vm_block = if ((((v2_1) > (((((64 as i32)).wrapping_sub((1 as i32))) as js_slimb_t))) as i32)) != 0 { 72 } else { 77 }; continue;
}
// C line 15254
79 => {
vm_block = 61; continue;
}
// C line ? labels: bigint_sar
80 => {
let _ = { let assigned = (v1_1).wrapping_shr((v2_1) as u32); v = assigned; assigned };
vm_block = 79; continue;
}
// C line 15245
81 => {
vm_block = 30; continue;
}
// C line 15250
82 => {
vm_block = 71; continue;
}
// C line 15249
83 => {
let _ = { let assigned = (v2_1).wrapping_neg(); v2_1 = assigned; assigned };
vm_block = 82; continue;
}
// C line 15248
84 => {
vm_block = 30; continue;
}
// C line 15247
85 => {
vm_block = if ((((v2_1) < ((((((64 as i32)).wrapping_sub((1 as i32))).wrapping_neg()) as js_slimb_t))) as i32)) != 0 { 84 } else { 83 }; continue;
}
// C line 15246
86 => {
vm_block = if ((((v2_1) < ((((0 as i32)) as js_slimb_t))) as i32)) != 0 { 85 } else { 80 }; continue;
}
// C line 15244
87 => {
vm_block = if ((((v2_1) > (((((64 as i32)).wrapping_sub((1 as i32))) as js_slimb_t))) as i32)) != 0 { 81 } else { 86 }; continue;
}
// C line 15242
88 => {
vm_block = 61; continue;
}
// C line 15241
89 => {
let _ = { let assigned = ((v1_1) ^ (v2_1)); v = assigned; assigned };
vm_block = 88; continue;
}
// C line 15239
90 => {
vm_block = 61; continue;
}
// C line 15238
91 => {
let _ = { let assigned = ((v1_1) | (v2_1)); v = assigned; assigned };
vm_block = 90; continue;
}
// C line 15236
92 => {
vm_block = 61; continue;
}
// C line 15235
93 => {
let _ = { let assigned = ((v1_1) & (v2_1)); v = assigned; assigned };
vm_block = 92; continue;
}
// C line 15233
94 => {
vm_block = match ((op) as u32) { x if x == (((OP_shl as i32)) as u32) => 78, x if x == (((OP_sar as i32)) as u32) => 87, x if x == (((OP_xor as i32)) as u32) => 89, x if x == (((OP_or as i32)) as u32) => 91, x if x == (((OP_and as i32)) as u32) => 93, _ => 62, }; continue;
}
// C line 15231
95 => {
let _ = { let assigned = ((op2).u).short_big_int; v2_1 = assigned; assigned };
vm_block = 94; continue;
}
// C line 15230
96 => {
let _ = { let assigned = ((op1).u).short_big_int; v1_1 = assigned; assigned };
vm_block = 95; continue;
}
// C line 15227
97 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 96 } else { 59 }; continue;
}
// C line 15225
98 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 97; continue;
}
// C line 15224
99 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 98; continue;
}
// C line 15223
100 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 99; continue;
}
// C line 15222
101 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 100; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15376. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ToBigIntBuf(mut ctx: *mut JSContext, mut buf1: *mut JSBigIntBuf, mut op1: JSValue) -> *mut JSBigInt {
let mut vm_local_storage = Vec::<u64>::new();
let mut p1: *mut JSBigInt = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15394
1 => {
return p1;
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 15390
3 => {
vm_block = 1; continue;
}
// C line 15389
4 => {
let _ = { let assigned = ((((op1).u).ptr) as *mut JSBigInt); p1 = assigned; assigned };
vm_block = 3; continue;
}
// C line 15387
5 => {
vm_block = 1; continue;
}
// C line 15386
6 => {
let _ = { let assigned = js_bigint_set_short(buf1, op1); p1 = assigned; assigned };
vm_block = 5; continue;
}
// C line 15384
7 => {
vm_block = 1; continue;
}
// C line 15383
8 => {
let _ = { let assigned = js_bigint_set_si(buf1, ((((((op1).u).uint64) as i32)) as js_slimb_t)); p1 = assigned; assigned };
vm_block = 7; continue;
}
// C line 15381
9 => {
vm_block = match (((op1).tag) as i32) { x if x == (JS_TAG_BIG_INT as i32) => 4, x if x == (JS_TAG_SHORT_BIG_INT as i32) => 6, x if x == (JS_TAG_INT as i32) => 8, _ => 2, }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15399. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_compare_bigint(mut ctx: *mut JSContext, mut op: i32, mut op1: JSValue, mut op2: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut res: i32 = core::mem::zeroed();
let mut val: i32 = core::mem::zeroed();
let mut tag1: i32 = core::mem::zeroed();
let mut tag2: i32 = core::mem::zeroed();
let mut buf1: JSBigIntBuf = core::mem::zeroed();
let mut buf2: JSBigIntBuf = core::mem::zeroed();
let mut p1: *mut JSBigInt = core::mem::zeroed();
let mut p2: *mut JSBigInt = core::mem::zeroed();
let mut v1: js_slimb_t = core::mem::zeroed();
let mut v2: js_slimb_t = core::mem::zeroed();
let mut vm_block: usize = 41;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15465
1 => {
return res;
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 15461
3 => {
vm_block = 1; continue;
}
// C line 15460
4 => {
let _ = { let assigned = (((val) == ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 3; continue;
}
// C line 15458
5 => {
vm_block = 1; continue;
}
// C line 15457
6 => {
let _ = { let assigned = (((val) >= ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15455
7 => {
vm_block = 1; continue;
}
// C line 15454
8 => {
let _ = { let assigned = (((val) > ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 7; continue;
}
// C line 15452
9 => {
vm_block = 1; continue;
}
// C line 15451
10 => {
let _ = { let assigned = (((val) <= ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 9; continue;
}
// C line 15449
11 => {
vm_block = 1; continue;
}
// C line 15448
12 => {
let _ = { let assigned = (((val) < ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 11; continue;
}
// C line 15446
13 => {
vm_block = match ((op) as u32) { x if x == (((OP_eq as i32)) as u32) => 4, x if x == (((OP_gte as i32)) as u32) => 6, x if x == (((OP_gt as i32)) as u32) => 8, x if x == (((OP_lte as i32)) as u32) => 10, x if x == (((OP_lt as i32)) as u32) => 12, _ => 2, }; continue;
}
// C line 15420
14 => {
let _ = { let assigned = ((((v1) > (v2)) as i32)).wrapping_sub((((v1) < (v2)) as i32)); val = assigned; assigned };
vm_block = 13; continue;
}
// C line 15417
15 => {
let _ = { let assigned = ((((((op2).u).uint64) as i32)) as js_slimb_t); v2 = assigned; assigned };
vm_block = 14; continue;
}
// C line 15419
16 => {
let _ = { let assigned = ((op2).u).short_big_int; v2 = assigned; assigned };
vm_block = 14; continue;
}
// C line 15416
17 => {
vm_block = if ((((tag2) == ((JS_TAG_INT as i32))) as i32)) != 0 { 15 } else { 16 }; continue;
}
// C line 15413
18 => {
let _ = { let assigned = ((((((op1).u).uint64) as i32)) as js_slimb_t); v1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 15415
19 => {
let _ = { let assigned = ((op1).u).short_big_int; v1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 15412
20 => {
vm_block = if ((((tag1) == ((JS_TAG_INT as i32))) as i32)) != 0 { 18 } else { 19 }; continue;
}
// C line 15443
21 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 13; continue;
}
// C line 15442
22 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 21; continue;
}
// C line 15427
23 => {
let _ = { let assigned = (val).wrapping_neg(); val = assigned; assigned };
vm_block = 22; continue;
}
// C line 15426
24 => {
vm_block = 30; continue;
}
// C line 15425
25 => {
vm_block = if ((((val) == ((2 as i32))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 15424
26 => {
let _ = { let assigned = js_bigint_float64_cmp(ctx, p2, ((op1).u).float64); val = assigned; assigned };
vm_block = 25; continue;
}
// C line 15423
27 => {
let _ = { let assigned = JS_ToBigIntBuf(ctx, core::ptr::addr_of_mut!(buf2), op2); p2 = assigned; assigned };
vm_block = 26; continue;
}
// C line 15435
28 => {
return (0 as i32);
}
// C line 15434
29 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 28; continue;
}
// C line ? labels: unordered
30 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 29; continue;
}
// C line 15431
31 => {
vm_block = if ((((val) == ((2 as i32))) as i32)) != 0 { 30 } else { 22 }; continue;
}
// C line 15430
32 => {
let _ = { let assigned = js_bigint_float64_cmp(ctx, p1, ((op2).u).float64); val = assigned; assigned };
vm_block = 31; continue;
}
// C line 15429
33 => {
let _ = { let assigned = JS_ToBigIntBuf(ctx, core::ptr::addr_of_mut!(buf1), op1); p1 = assigned; assigned };
vm_block = 32; continue;
}
// C line 15440
34 => {
let _ = { let assigned = js_bigint_cmp(ctx, p1, p2); val = assigned; assigned };
vm_block = 22; continue;
}
// C line 15439
35 => {
let _ = { let assigned = JS_ToBigIntBuf(ctx, core::ptr::addr_of_mut!(buf2), op2); p2 = assigned; assigned };
vm_block = 34; continue;
}
// C line 15438
36 => {
let _ = { let assigned = JS_ToBigIntBuf(ctx, core::ptr::addr_of_mut!(buf1), op1); p1 = assigned; assigned };
vm_block = 35; continue;
}
// C line 15428
37 => {
vm_block = if ((((tag2) == ((JS_TAG_FLOAT64 as i32))) as i32)) != 0 { 33 } else { 36 }; continue;
}
// C line 15422
38 => {
vm_block = if ((((tag1) == ((JS_TAG_FLOAT64 as i32))) as i32)) != 0 { 27 } else { 37 }; continue;
}
// C line 15408
39 => {
vm_block = if ((((((((((((tag1) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0) || (((((tag1) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) && (((((((((tag2) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0) || (((((tag2) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 20 } else { 38 }; continue;
}
// C line 15407
40 => {
let _ = { let assigned = (((op2).tag) as i32); tag2 = assigned; assigned };
vm_block = 39; continue;
}
// C line 15406
41 => {
let _ = { let assigned = (((op1).tag) as i32); tag1 = assigned; assigned };
vm_block = 40; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15468. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_relational_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut op: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut tag1: u32 = core::mem::zeroed();
let mut tag2: u32 = core::mem::zeroed();
let mut d1: f64 = core::mem::zeroed();
let mut d2: f64 = core::mem::zeroed();
let mut vm_block: usize = 75;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15599
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15598
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 15595
4 => {
return (0 as i32);
}
// C line ? labels: done
5 => {
let _ = { let assigned = JS_NewBool(ctx, res); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 15515
6 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 5; continue;
}
// C line 15514
7 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 6; continue;
}
// C line 15512
8 => {
vm_block = 7; continue;
}
// C line 15511
9 => {
let _ = { let assigned = (((res) >= ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 8; continue;
}
// C line 15508
10 => {
vm_block = 7; continue;
}
// C line 15507
11 => {
let _ = { let assigned = (((res) > ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 10; continue;
}
// C line 15505
12 => {
vm_block = 7; continue;
}
// C line 15504
13 => {
let _ = { let assigned = (((res) <= ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 12; continue;
}
// C line 15502
14 => {
vm_block = 7; continue;
}
// C line 15501
15 => {
let _ = { let assigned = (((res) < ((0 as i32))) as i32); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 15499
16 => {
vm_block = match ((op) as u32) { x if x == (((OP_gte as i32)) as u32) => 9, x if x == (((OP_gt as i32)) as u32) => 11, x if x == (((OP_lte as i32)) as u32) => 13, x if x == (((OP_lt as i32)) as u32) => 15, _ => 9, }; continue;
}
// C line 15494
17 => {
let _ = { let assigned = js_string_compare(ctx, ((((op1).u).ptr) as *mut JSString), ((((op2).u).ptr) as *mut JSString)); res = assigned; assigned };
vm_block = 16; continue;
}
// C line 15497
18 => {
let _ = { let assigned = js_string_rope_compare(ctx, op1, op2, (0 as i32)); res = assigned; assigned };
vm_block = 16; continue;
}
// C line 15493
19 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_STRING as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_STRING as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 18 }; continue;
}
// C line 15519
20 => {
vm_block = 36; continue;
}
// C line 15560
21 => {
let _ = { let assigned = js_compare_bigint(ctx, op, op1, op2); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15589
22 => {
vm_block = 5; continue;
}
// C line 15588
23 => {
let _ = { let assigned = (((d1) >= (d2)) as i32); res = assigned; assigned };
vm_block = 22; continue;
}
// C line 15585
24 => {
vm_block = 5; continue;
}
// C line 15584
25 => {
let _ = { let assigned = (((d1) > (d2)) as i32); res = assigned; assigned };
vm_block = 24; continue;
}
// C line 15582
26 => {
vm_block = 5; continue;
}
// C line 15581
27 => {
let _ = { let assigned = (((d1) <= (d2)) as i32); res = assigned; assigned };
vm_block = 26; continue;
}
// C line 15579
28 => {
vm_block = 5; continue;
}
// C line 15578
29 => {
let _ = { let assigned = (((d1) < (d2)) as i32); res = assigned; assigned };
vm_block = 28; continue;
}
// C line 15576
30 => {
vm_block = match ((op) as u32) { x if x == (((OP_gte as i32)) as u32) => 23, x if x == (((OP_gt as i32)) as u32) => 25, x if x == (((OP_lte as i32)) as u32) => 27, x if x == (((OP_lt as i32)) as u32) => 29, _ => 23, }; continue;
}
// C line 15572
31 => {
let _ = { let assigned = ((op2).u).float64; d2 = assigned; assigned };
vm_block = 30; continue;
}
// C line 15574
32 => {
let _ = { let assigned = ((((((op2).u).uint64) as i32)) as f64); d2 = assigned; assigned };
vm_block = 30; continue;
}
// C line 15571
33 => {
vm_block = if ((((tag2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 31 } else { 32 }; continue;
}
// C line 15567
34 => {
let _ = { let assigned = ((op1).u).float64; d1 = assigned; assigned };
vm_block = 33; continue;
}
// C line 15569
35 => {
let _ = { let assigned = ((((((op1).u).uint64) as i32)) as f64); d1 = assigned; assigned };
vm_block = 33; continue;
}
// C line 15566 labels: float64_compare
36 => {
vm_block = if ((((tag1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 34 } else { 35 }; continue;
}
// C line 15558
37 => {
vm_block = if ((((((((((((((((tag1) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((tag2) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 21 } else { 36 }; continue;
}
// C line 15556
38 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 37; continue;
}
// C line 15555
39 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 38; continue;
}
// C line 15539
40 => {
vm_block = 5; continue;
}
// C line 15538
41 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 40; continue;
}
// C line 15537
42 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 41; continue;
}
// C line ? labels: invalid_bigint_string
43 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 42; continue;
}
// C line 15533
44 => {
vm_block = if (((((((((((op2).tag) as i32)) != ((JS_TAG_BIG_INT as i32))) as i32)) != 0) && ((((((((op2).tag) as i32)) != ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0 { 43 } else { 39 }; continue;
}
// C line 15532
45 => {
let _ = { let assigned = JS_StringToBigInt(ctx, op2); op2 = assigned; assigned };
vm_block = 44; continue;
}
// C line 15531
46 => {
vm_block = if (tag_is_string(tag2)) != 0 { 45 } else { 39 }; continue;
}
// C line 15529
47 => {
vm_block = 43; continue;
}
// C line 15527
48 => {
vm_block = if (((((((((((op1).tag) as i32)) != ((JS_TAG_BIG_INT as i32))) as i32)) != 0) && ((((((((op1).tag) as i32)) != ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 15526
49 => {
let _ = { let assigned = JS_StringToBigInt(ctx, op1); op1 = assigned; assigned };
vm_block = 48; continue;
}
// C line 15525
50 => {
vm_block = if (tag_is_string(tag1)) != 0 { 49 } else { 46 }; continue;
}
// C line 15551
51 => {
vm_block = 3; continue;
}
// C line 15550
52 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 51; continue;
}
// C line 15549
53 => {
vm_block = if (JS_IsException(op2)) != 0 { 52 } else { 39 }; continue;
}
// C line 15548
54 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op2); op2 = assigned; assigned };
vm_block = 53; continue;
}
// C line 15546
55 => {
vm_block = 3; continue;
}
// C line 15545
56 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 55; continue;
}
// C line 15544
57 => {
vm_block = if (JS_IsException(op1)) != 0 { 56 } else { 54 }; continue;
}
// C line 15543
58 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 57; continue;
}
// C line 15521
59 => {
vm_block = if ((((((((((((((((tag1) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && ((tag_is_string(tag2)) != 0)) as i32)) != 0) || (((((((((((((tag2) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && ((tag_is_string(tag1)) != 0)) as i32)) != 0)) as i32)) != 0 { 50 } else { 58 }; continue;
}
// C line 15516
60 => {
vm_block = if ((((((((((((tag1) <= ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((((((tag2) <= ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 20 } else { 59 }; continue;
}
// C line 15492
61 => {
vm_block = if (((((tag_is_string(tag1)) != 0) && ((tag_is_string(tag2)) != 0)) as i32)) != 0 { 19 } else { 60 }; continue;
}
// C line 15490
62 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 61; continue;
}
// C line 15489
63 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 62; continue;
}
// C line 15487
64 => {
vm_block = 3; continue;
}
// C line 15486
65 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 64; continue;
}
// C line 15485
66 => {
vm_block = if (JS_IsException(op2)) != 0 { 65 } else { 63 }; continue;
}
// C line 15484
67 => {
let _ = { let assigned = JS_ToPrimitiveFree(ctx, op2, (1 as i32)); op2 = assigned; assigned };
vm_block = 66; continue;
}
// C line 15482
68 => {
vm_block = 3; continue;
}
// C line 15481
69 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 68; continue;
}
// C line 15480
70 => {
vm_block = if (JS_IsException(op1)) != 0 { 69 } else { 67 }; continue;
}
// C line 15479
71 => {
let _ = { let assigned = JS_ToPrimitiveFree(ctx, op1, (1 as i32)); op1 = assigned; assigned };
vm_block = 70; continue;
}
// C line 15478
72 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 71; continue;
}
// C line 15477
73 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 72; continue;
}
// C line 15476
74 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 73; continue;
}
// C line 15475
75 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 74; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15609. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_eq_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut is_neq: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut tag1: u32 = core::mem::zeroed();
let mut tag2: u32 = core::mem::zeroed();
let mut d1: f64 = core::mem::zeroed();
let mut d2: f64 = core::mem::zeroed();
let mut vm_block: usize = 70;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15726
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15725
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 15722
4 => {
return (0 as i32);
}
// C line ? labels: done
5 => {
let _ = { let assigned = JS_NewBool(ctx, ((res) ^ (is_neq))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 15623
6 => {
let _ = { let assigned = (((((((op1).u).uint64) as i32)) == (((((op2).u).uint64) as i32))) as i32); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15639
7 => {
let _ = { let assigned = (((d1) == (d2)) as i32); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15635
8 => {
let _ = { let assigned = ((op2).u).float64; d2 = assigned; assigned };
vm_block = 7; continue;
}
// C line 15637
9 => {
let _ = { let assigned = ((((((op2).u).uint64) as i32)) as f64); d2 = assigned; assigned };
vm_block = 7; continue;
}
// C line 15634
10 => {
vm_block = if ((((tag2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 8 } else { 9 }; continue;
}
// C line 15630
11 => {
let _ = { let assigned = ((op1).u).float64; d1 = assigned; assigned };
vm_block = 10; continue;
}
// C line 15632
12 => {
let _ = { let assigned = ((((((op1).u).uint64) as i32)) as f64); d1 = assigned; assigned };
vm_block = 10; continue;
}
// C line 15629
13 => {
vm_block = if ((((tag1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 11 } else { 12 }; continue;
}
// C line 15641
14 => {
let _ = { let assigned = js_compare_bigint(ctx, (OP_eq as i32), op1, op2); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15624
15 => {
vm_block = if ((((((((((((tag1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) && (((((((((tag2) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) || (((((tag2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((((tag2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) && (((((((((tag1) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 13 } else { 14 }; continue;
}
// C line 15622
16 => {
vm_block = if ((((((((tag1) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 6 } else { 15 }; continue;
}
// C line 15644
17 => {
let _ = { let assigned = js_strict_eq2(ctx, op1, op2, (((JS_EQ_STRICT as i32)) as JSStrictEqModeEnum)); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15647
18 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15650
19 => {
let _ = { let assigned = js_strict_eq2(ctx, op1, op2, (((JS_EQ_STRICT as i32)) as JSStrictEqModeEnum)); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15685
20 => {
let _ = { let assigned = js_strict_eq2(ctx, op1, op2, (((JS_EQ_STRICT as i32)) as JSStrictEqModeEnum)); res = assigned; assigned };
vm_block = 5; continue;
}
// C line 15670
21 => {
vm_block = 5; continue;
}
// C line 15669
22 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 21; continue;
}
// C line 15668
23 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 22; continue;
}
// C line ? labels: invalid_bigint_string
24 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 23; continue;
}
// C line 15664
25 => {
vm_block = if (((((((((((op2).tag) as i32)) != ((JS_TAG_BIG_INT as i32))) as i32)) != 0) && ((((((((op2).tag) as i32)) != ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0 { 24 } else { 20 }; continue;
}
// C line 15663
26 => {
let _ = { let assigned = JS_StringToBigInt(ctx, op2); op2 = assigned; assigned };
vm_block = 25; continue;
}
// C line 15662
27 => {
vm_block = if (tag_is_string(tag2)) != 0 { 26 } else { 20 }; continue;
}
// C line 15660
28 => {
vm_block = 24; continue;
}
// C line 15658
29 => {
vm_block = if (((((((((((op1).tag) as i32)) != ((JS_TAG_BIG_INT as i32))) as i32)) != 0) && ((((((((op1).tag) as i32)) != ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 15657
30 => {
let _ = { let assigned = JS_StringToBigInt(ctx, op1); op1 = assigned; assigned };
vm_block = 29; continue;
}
// C line 15656
31 => {
vm_block = if (tag_is_string(tag1)) != 0 { 30 } else { 27 }; continue;
}
// C line 15682
32 => {
vm_block = 3; continue;
}
// C line 15681
33 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 32; continue;
}
// C line 15680
34 => {
vm_block = if (JS_IsException(op2)) != 0 { 33 } else { 20 }; continue;
}
// C line 15679
35 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op2); op2 = assigned; assigned };
vm_block = 34; continue;
}
// C line 15677
36 => {
vm_block = 3; continue;
}
// C line 15676
37 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 36; continue;
}
// C line 15675
38 => {
vm_block = if (JS_IsException(op1)) != 0 { 37 } else { 35 }; continue;
}
// C line 15674
39 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 38; continue;
}
// C line 15654
40 => {
vm_block = if ((((((((((((((((tag1) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((tag2) == ((((JS_TAG_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((tag2) == ((((JS_TAG_SHORT_BIG_INT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 31 } else { 39 }; continue;
}
// C line 15688
41 => {
vm_block = 68; continue;
}
// C line 15687
42 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((op1).u).uint64) as i32)); op1 = assigned; assigned };
vm_block = 41; continue;
}
// C line 15691
43 => {
vm_block = 68; continue;
}
// C line 15690
44 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((op2).u).uint64) as i32)); op2 = assigned; assigned };
vm_block = 43; continue;
}
// C line 15706
45 => {
vm_block = 68; continue;
}
// C line 15704
46 => {
vm_block = 3; continue;
}
// C line 15703
47 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 46; continue;
}
// C line 15702
48 => {
vm_block = if (JS_IsException(op2)) != 0 { 47 } else { 45 }; continue;
}
// C line 15701
49 => {
let _ = { let assigned = JS_ToPrimitiveFree(ctx, op2, (2 as i32)); op2 = assigned; assigned };
vm_block = 48; continue;
}
// C line 15699
50 => {
vm_block = 3; continue;
}
// C line 15698
51 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 50; continue;
}
// C line 15697
52 => {
vm_block = if (JS_IsException(op1)) != 0 { 51 } else { 49 }; continue;
}
// C line 15696
53 => {
let _ = { let assigned = JS_ToPrimitiveFree(ctx, op1, (2 as i32)); op1 = assigned; assigned };
vm_block = 52; continue;
}
// C line 15718
54 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 5; continue;
}
// C line 15717
55 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 54; continue;
}
// C line 15713
56 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 55; continue;
}
// C line 15715
57 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 55; continue;
}
// C line 15709
58 => {
vm_block = if (((((((((JS_IsHTMLDDA(ctx, op1)) != 0) && (((((((((tag2) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag2) == ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || ((((((JS_IsHTMLDDA(ctx, op2)) != 0) && (((((((((tag1) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag1) == ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 56 } else { 57 }; continue;
}
// C line 15692
59 => {
vm_block = if ((((((((((((tag1) == ((((JS_TAG_OBJECT as i32)) as u32))) as i32)) != 0) && ((((((((((tag_is_number(tag2)) != 0) || ((tag_is_string(tag2)) != 0)) as i32)) != 0) || (((((tag2) == ((((JS_TAG_SYMBOL as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((((tag2) == ((((JS_TAG_OBJECT as i32)) as u32))) as i32)) != 0) && ((((((((((tag_is_number(tag1)) != 0) || ((tag_is_string(tag1)) != 0)) as i32)) != 0) || (((((tag1) == ((((JS_TAG_SYMBOL as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 53 } else { 58 }; continue;
}
// C line 15689
60 => {
vm_block = if ((((tag2) == ((((JS_TAG_BOOL as i32)) as u32))) as i32)) != 0 { 44 } else { 59 }; continue;
}
// C line 15686
61 => {
vm_block = if ((((tag1) == ((((JS_TAG_BOOL as i32)) as u32))) as i32)) != 0 { 42 } else { 60 }; continue;
}
// C line 15651
62 => {
vm_block = if (((((((((tag_is_string(tag1)) != 0) && ((tag_is_number(tag2)) != 0)) as i32)) != 0) || ((((((tag_is_string(tag2)) != 0) && ((tag_is_number(tag1)) != 0)) as i32)) != 0)) as i32)) != 0 { 40 } else { 61 }; continue;
}
// C line 15648
63 => {
vm_block = if (((((tag_is_string(tag1)) != 0) && ((tag_is_string(tag2)) != 0)) as i32)) != 0 { 19 } else { 62 }; continue;
}
// C line 15645
64 => {
vm_block = if ((((((((((((tag1) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) && (((((tag2) == ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((((((tag2) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) && (((((tag1) == ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 18 } else { 63 }; continue;
}
// C line 15643
65 => {
vm_block = if ((((tag1) == (tag2)) as i32)) != 0 { 17 } else { 64 }; continue;
}
// C line 15621
66 => {
vm_block = if (((((tag_is_number(tag1)) != 0) && ((tag_is_number(tag2)) != 0)) as i32)) != 0 { 16 } else { 65 }; continue;
}
// C line 15620
67 => {
let _ = { let assigned = (((((op2).tag) as i32)) as u32); tag2 = assigned; assigned };
vm_block = 66; continue;
}
// C line ? labels: redo
68 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag1 = assigned; assigned };
vm_block = 67; continue;
}
// C line 15617
69 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 68; continue;
}
// C line 15616
70 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 69; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15729. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_shr_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut v1: u32 = core::mem::zeroed();
let mut v2: u32 = core::mem::zeroed();
let mut r: u32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15764
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15763
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 15760
4 => {
return (0 as i32);
}
// C line 15759
5 => {
let _ = { let assigned = JS_NewUint32(ctx, r); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 15758
6 => {
let _ = { let assigned = (v1).wrapping_shr((((v2) & ((((31 as i32)) as u32)))) as u32); r = assigned; assigned };
vm_block = 5; continue;
}
// C line 15757
7 => {
let _ = JS_ToUint32Free(ctx, core::ptr::addr_of_mut!(v2), op2);
vm_block = 6; continue;
}
// C line 15756
8 => {
let _ = JS_ToUint32Free(ctx, core::ptr::addr_of_mut!(v1), op1);
vm_block = 7; continue;
}
// C line 15753
9 => {
vm_block = 3; continue;
}
// C line 15752
10 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 9; continue;
}
// C line 15751
11 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 10; continue;
}
// C line 15750
12 => {
let _ = JS_ThrowTypeError(ctx, c"bigint operands are forbidden for >>>".as_ptr());
vm_block = 11; continue;
}
// C line 15746
13 => {
vm_block = if (((((((((((((((((((op1).tag) as i32)) == ((JS_TAG_BIG_INT as i32))) as i32)) != 0) || ((((((((op1).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((op2).tag) as i32)) == ((JS_TAG_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((op2).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 8 }; continue;
}
// C line 15744
14 => {
vm_block = 3; continue;
}
// C line 15743
15 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 14; continue;
}
// C line 15742
16 => {
vm_block = if (JS_IsException(op2)) != 0 { 15 } else { 13 }; continue;
}
// C line 15741
17 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op2); op2 = assigned; assigned };
vm_block = 16; continue;
}
// C line 15739
18 => {
vm_block = 3; continue;
}
// C line 15738
19 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 18; continue;
}
// C line 15737
20 => {
vm_block = if (JS_IsException(op1)) != 0 { 19 } else { 17 }; continue;
}
// C line 15736
21 => {
let _ = { let assigned = JS_ToNumericFree(ctx, op1); op1 = assigned; assigned };
vm_block = 20; continue;
}
// C line 15735
22 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 21; continue;
}
// C line 15734
23 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15929. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_strict_eq_slow(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut is_neq: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15935
1 => {
return (0 as i32);
}
// C line 15934
2 => {
let _ = { let assigned = JS_NewBool(ctx, ((res) ^ (is_neq))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 15933
3 => {
let _ = { let assigned = js_strict_eq2(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (((JS_EQ_STRICT as i32)) as JSStrictEqModeEnum)); res = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15938. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_operator_in(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15961
1 => {
return (0 as i32);
}
// C line 15960
2 => {
let _ = { let assigned = JS_NewBool(ctx, ret); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 15959
3 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 2; continue;
}
// C line 15958
4 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 3; continue;
}
// C line 15957
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15956
6 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 15955
7 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 6; continue;
}
// C line 15954
8 => {
let _ = { let assigned = JS_HasProperty(ctx, op2, atom); ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 15953
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15952
10 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 9 } else { 8 }; continue;
}
// C line 15951
11 => {
let _ = { let assigned = JS_ValueToAtom(ctx, op1); atom = assigned; assigned };
vm_block = 10; continue;
}
// C line 15949
12 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15948
13 => {
let _ = JS_ThrowTypeError(ctx, c"invalid 'in' operand".as_ptr());
vm_block = 12; continue;
}
// C line 15947
14 => {
vm_block = if (((((((op2).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 15945
15 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 14; continue;
}
// C line 15944
16 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:15964. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_operator_private_in(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut pr: *mut JSProperty = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 15998
1 => {
return (0 as i32);
}
// C line 15997
2 => {
let _ = { let assigned = JS_NewBool(ctx, ret); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 15996
3 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 2; continue;
}
// C line 15995
4 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 3; continue;
}
// C line 15980
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15979
6 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 15978
7 => {
let _ = { let assigned = JS_CheckBrand(ctx, op1, op2); ret = assigned; assigned };
vm_block = 6; continue;
}
// C line 15993
8 => {
let _ = { let assigned = (((prs) != (((core::ptr::null_mut::<c_void>()) as *mut JSShapeProperty))) as i32); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 15992
9 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 8; continue;
}
// C line 15991
10 => {
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr), p, atom); prs = assigned; assigned };
vm_block = 9; continue;
}
// C line 15990
11 => {
let _ = { let assigned = ((((op1).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 10; continue;
}
// C line 15989
12 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15988
13 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 12 } else { 11 }; continue;
}
// C line 15987
14 => {
let _ = { let assigned = JS_ValueToAtom(ctx, op2); atom = assigned; assigned };
vm_block = 13; continue;
}
// C line 15976
15 => {
vm_block = if (JS_IsObject(op2)) != 0 { 7 } else { 14 }; continue;
}
// C line 15974
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 15973
17 => {
let _ = JS_ThrowTypeError(ctx, c"invalid 'in' operand".as_ptr());
vm_block = 16; continue;
}
// C line 15972
18 => {
vm_block = if (((((((op1).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 17 } else { 15 }; continue;
}
// C line 15970
19 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 18; continue;
}
// C line 15969
20 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16019. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_operator_instanceof(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16032
1 => {
return (0 as i32);
}
// C line 16031
2 => {
let _ = { let assigned = JS_NewBool(ctx, ret); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 16030
3 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 2; continue;
}
// C line 16029
4 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 3; continue;
}
// C line 16028
5 => {
return ret;
}
// C line 16027
6 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 16026
7 => {
let _ = { let assigned = JS_IsInstanceOf(ctx, op1, op2); ret = assigned; assigned };
vm_block = 6; continue;
}
// C line 16025
8 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 7; continue;
}
// C line 16024
9 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16035. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_operator_typeof(mut ctx: *mut JSContext, mut op1: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut atom: JSAtom = core::mem::zeroed();
let mut tag: u32 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16083
1 => {
return ((atom) as i32);
}
// C line 16081
2 => {
vm_block = 1; continue;
}
// C line ?
3 => {
let _ = { let assigned = (((JS_ATOM_unknown as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 2; continue;
}
// C line 16078
4 => {
vm_block = 1; continue;
}
// C line 16077
5 => {
let _ = { let assigned = (((JS_ATOM_symbol as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 4; continue;
}
// C line 16075
6 => {
vm_block = 1; continue;
}
// C line ? labels: obj_type
7 => {
let _ = { let assigned = (((JS_ATOM_object as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 6; continue;
}
// C line 16071
8 => {
vm_block = 1; continue;
}
// C line 16065
9 => {
let _ = { let assigned = (((JS_ATOM_undefined as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 8; continue;
}
// C line 16067
10 => {
let _ = { let assigned = (((JS_ATOM_function as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 8; continue;
}
// C line 16069
11 => {
vm_block = 7; continue;
}
// C line 16066
12 => {
vm_block = if (JS_IsFunction(ctx, op1)) != 0 { 10 } else { 11 }; continue;
}
// C line 16064
13 => {
vm_block = if ((((!(((!(((*(p)).is_HTMLDDA()) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 9 } else { 12 }; continue;
}
// C line 16063
14 => {
let _ = { let assigned = ((((op1).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 13; continue;
}
// C line 16059
15 => {
vm_block = 1; continue;
}
// C line 16058
16 => {
let _ = { let assigned = (((JS_ATOM_string as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 15; continue;
}
// C line 16055
17 => {
vm_block = 1; continue;
}
// C line 16054
18 => {
let _ = { let assigned = (((JS_ATOM_boolean as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 17; continue;
}
// C line 16052
19 => {
vm_block = 1; continue;
}
// C line 16051
20 => {
let _ = { let assigned = (((JS_ATOM_undefined as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 19; continue;
}
// C line 16049
21 => {
vm_block = 1; continue;
}
// C line 16048
22 => {
let _ = { let assigned = (((JS_ATOM_number as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 21; continue;
}
// C line 16045
23 => {
vm_block = 1; continue;
}
// C line 16044
24 => {
let _ = { let assigned = (((JS_ATOM_bigint as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 23; continue;
}
// C line 16041
25 => {
vm_block = match tag { x if x == (((JS_TAG_SYMBOL as i32)) as u32) => 5, x if x == (((JS_TAG_NULL as i32)) as u32) => 7, x if x == (((JS_TAG_OBJECT as i32)) as u32) => 14, x if x == (((JS_TAG_STRING_ROPE as i32)) as u32) => 16, x if x == (((JS_TAG_STRING as i32)) as u32) => 16, x if x == (((JS_TAG_BOOL as i32)) as u32) => 18, x if x == (((JS_TAG_UNDEFINED as i32)) as u32) => 20, x if x == (((JS_TAG_FLOAT64 as i32)) as u32) => 22, x if x == (((JS_TAG_INT as i32)) as u32) => 22, x if x == (((JS_TAG_BIG_INT as i32)) as u32) => 24, x if x == (((JS_TAG_SHORT_BIG_INT as i32)) as u32) => 24, _ => 3, }; continue;
}
// C line 16040
26 => {
let _ = { let assigned = (((((op1).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16086. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_operator_delete(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16104
1 => {
return (0 as i32);
}
// C line 16103
2 => {
let _ = { let assigned = JS_NewBool(ctx, ret); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 16102
3 => {
let _ = JS_FreeValue(ctx, op2);
vm_block = 2; continue;
}
// C line 16101
4 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 3; continue;
}
// C line 16100
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16099
6 => {
vm_block = if ((((!(((!(((((ret) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 5 } else { 4 }; continue;
}
// C line 16098
7 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 6; continue;
}
// C line 16097
8 => {
let _ = { let assigned = JS_DeleteProperty(ctx, op1, atom, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 16096
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16095
10 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 9 } else { 8 }; continue;
}
// C line 16094
11 => {
let _ = { let assigned = JS_ValueToAtom(ctx, op2); atom = assigned; assigned };
vm_block = 10; continue;
}
// C line 16093
12 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 11; continue;
}
// C line 16092
13 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}
