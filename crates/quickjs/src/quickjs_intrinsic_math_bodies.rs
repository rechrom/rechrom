// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46698. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_fmin(mut a: f64, mut b: f64) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut a1: JSFloat64Union = core::mem::zeroed();
let mut b1: JSFloat64Union = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46705
1 => {
return (a1).d;
}
// C line 46704
2 => {
let _ = { (a1).u64 = (((a1).u64) | ((b1).u64)); (a1).u64 };
vm_block = 1; continue;
}
// C line 46703
3 => {
let _ = { let assigned = b; (b1).d = assigned; assigned };
vm_block = 2; continue;
}
// C line 46702
4 => {
let _ = { let assigned = a; (a1).d = assigned; assigned };
vm_block = 3; continue;
}
// C line 46707
5 => {
return (a).min(b);
}
// C line 46700
6 => {
vm_block = if ((((((((a) == ((((0 as i32)) as f64))) as i32)) != 0) && (((((b) == ((((0 as i32)) as f64))) as i32)) != 0)) as i32)) != 0 { 4 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46712. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_fmax(mut a: f64, mut b: f64) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut a1: JSFloat64Union = core::mem::zeroed();
let mut b1: JSFloat64Union = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46719
1 => {
return (a1).d;
}
// C line 46718
2 => {
let _ = { (a1).u64 = (((a1).u64) & ((b1).u64)); (a1).u64 };
vm_block = 1; continue;
}
// C line 46717
3 => {
let _ = { let assigned = b; (b1).d = assigned; assigned };
vm_block = 2; continue;
}
// C line 46716
4 => {
let _ = { let assigned = a; (a1).d = assigned; assigned };
vm_block = 3; continue;
}
// C line 46721
5 => {
return (a).max(b);
}
// C line 46714
6 => {
vm_block = if ((((((((a) == ((((0 as i32)) as f64))) as i32)) != 0) && (((((b) == ((((0 as i32)) as f64))) as i32)) != 0)) as i32)) != 0 { 4 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46725. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_min_max(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut is_max: i32 = core::mem::zeroed();
let mut r: f64 = core::mem::zeroed();
let mut a: f64 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut tag: u32 = core::mem::zeroed();
let mut a1: i32 = core::mem::zeroed();
let mut r1: i32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46753
1 => {
return JS_NewInt32(ctx, r1);
}
// C line 46740
2 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 46748
4 => {
let _ = { let assigned = crate::cutils_header::max_int(r1, a1); r1 = assigned; assigned };
vm_block = 3; continue;
}
// C line 46750
5 => {
let _ = { let assigned = crate::cutils_header::min_int(r1, a1); r1 = assigned; assigned };
vm_block = 3; continue;
}
// C line 46747
6 => {
vm_block = if (is_max) != 0 { 4 } else { 5 }; continue;
}
// C line 46746
7 => {
let _ = { let assigned = ((((*(argv).offset((i) as isize)).u).uint64) as i32); a1 = assigned; assigned };
vm_block = 6; continue;
}
// C line 46744
8 => {
vm_block = 15; continue;
}
// C line 46743
9 => {
let _ = { let assigned = ((r1) as f64); r = assigned; assigned };
vm_block = 8; continue;
}
// C line 46742
10 => {
vm_block = if ((((tag) != ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 46741
11 => {
let _ = { let assigned = (((((*(argv).offset((i) as isize)).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 10; continue;
}
// C line 46740
12 => {
let _ = { let assigned = (1 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 46739
13 => {
r1 = ((((*(argv).offset(((0 as i32)) as isize)).u).uint64) as i32);
vm_block = 12; continue;
}
// C line 46774
14 => {
return JS_NewFloat64(ctx, r);
}
// C line 46759 labels: generic_case
15 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 24 } else { 14 }; continue;
}
// C line 46772
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 46764
17 => {
let _ = { let assigned = a; r = assigned; assigned };
vm_block = 16; continue;
}
// C line 46767
18 => {
let _ = { let assigned = js_fmax(r, a); r = assigned; assigned };
vm_block = 16; continue;
}
// C line 46769
19 => {
let _ = { let assigned = js_fmin(r, a); r = assigned; assigned };
vm_block = 16; continue;
}
// C line 46766
20 => {
vm_block = if (is_max) != 0 { 18 } else { 19 }; continue;
}
// C line 46763
21 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((a) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (a).is_nan() as i32 } else { (a).is_nan() as i32 } }) != 0 { 17 } else { 20 }; continue;
}
// C line 46762
22 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((r) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (r).is_nan() as i32 } else { (r).is_nan() as i32 } }) != 0) as i32)) != 0 { 21 } else { 16 }; continue;
}
// C line 46761
23 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46760
24 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(a), *(argv).offset((i) as isize))) != 0 { 23 } else { 22 }; continue;
}
// C line 46757
25 => {
let _ = { let assigned = (1 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 46756
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46755
27 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(r), *(argv).offset(((0 as i32)) as isize))) != 0 { 26 } else { 25 }; continue;
}
// C line 46738
28 => {
vm_block = if ((((tag) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0 { 13 } else { 27 }; continue;
}
// C line 46737
29 => {
let _ = { let assigned = (((((*(argv).offset(((0 as i32)) as isize)).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 28; continue;
}
// C line 46734
30 => {
return __JS_NewFloat64(ctx, if (is_max) != 0 { (((-((1 as f64)))) / ((0 as f64))) } else { (((1 as f64)) / ((0 as f64))) });
}
// C line 46733
31 => {
vm_block = if ((((!(((!(((((argc) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 30 } else { 29 }; continue;
}
// C line 46728
32 => {
is_max = magic;
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46778. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_sign(mut a: f64) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46783
1 => {
return ((((1 as i32)).wrapping_neg()) as f64);
}
// C line 46785
2 => {
return (((1 as i32)) as f64);
}
// C line 46782
3 => {
vm_block = if ((((a) < ((((0 as i32)) as f64))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 46781
4 => {
return a;
}
// C line 46780
5 => {
vm_block = if (((((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((a) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (a).is_nan() as i32 } else { (a).is_nan() as i32 } }) != 0) || (((((a) == ((0 as f64))) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46788. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_round(mut a: f64) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut u: JSFloat64Union = core::mem::zeroed();
let mut frac_mask: u64 = core::mem::zeroed();
let mut one: u64 = core::mem::zeroed();
let mut e: u32 = core::mem::zeroed();
let mut s: u32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46813
1 => {
return (u).d;
}
// C line 46800
2 => {
let _ = { let assigned = (((((u).u64) & (((((1 as i32)) as u64)).wrapping_shl(((63 as i32)) as u32)))) | (((((1023 as i32)) as u64)).wrapping_shl(((52 as i32)) as u32))); (u).u64 = assigned; assigned };
vm_block = 1; continue;
}
// C line 46803
3 => {
let _ = { (u).u64 = (((u).u64) & (((((1 as i32)) as u64)).wrapping_shl(((63 as i32)) as u32))); (u).u64 };
vm_block = 1; continue;
}
// C line 46798
4 => {
vm_block = if ((((((((e) == (((((1023 as i32)).wrapping_sub((1 as i32))) as u32))) as i32)) != 0) && ((((((u).u64) != ((((13826050856027422720 as usize)) as u64))) as i32)) != 0)) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 46810
5 => {
let _ = { (u).u64 = (((u).u64) & ((!(frac_mask)))); (u).u64 };
vm_block = 1; continue;
}
// C line 46809
6 => {
let _ = { (u).u64 = ((u).u64).wrapping_add(((one).wrapping_shr(((1 as i32)) as u32)).wrapping_sub(((s) as u64))); (u).u64 };
vm_block = 5; continue;
}
// C line 46808
7 => {
let _ = { let assigned = (one).wrapping_sub((((1 as i32)) as u64)); frac_mask = assigned; assigned };
vm_block = 6; continue;
}
// C line 46807
8 => {
let _ = { let assigned = ((((1 as i32)) as u64)).wrapping_shl((((((52 as i32)) as u32)).wrapping_sub((e).wrapping_sub((((1023 as i32)) as u32)))) as u32); one = assigned; assigned };
vm_block = 7; continue;
}
// C line 46806
9 => {
let _ = { let assigned = ((((u).u64).wrapping_shr(((63 as i32)) as u32)) as u32); s = assigned; assigned };
vm_block = 8; continue;
}
// C line 46805
10 => {
vm_block = if ((((e) < (((((1023 as i32)).wrapping_add((52 as i32))) as u32))) as i32)) != 0 { 9 } else { 1 }; continue;
}
// C line 46796
11 => {
vm_block = if ((((e) < ((((1023 as i32)) as u32))) as i32)) != 0 { 4 } else { 10 }; continue;
}
// C line 46795
12 => {
let _ = { let assigned = ((((((u).u64).wrapping_shr(((52 as i32)) as u32)) & ((((2047 as i32)) as u64)))) as u32); e = assigned; assigned };
vm_block = 11; continue;
}
// C line 46794
13 => {
let _ = { let assigned = a; (u).d = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46816. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_hypot(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut r: f64 = core::mem::zeroed();
let mut a: f64 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46837
1 => {
return JS_NewFloat64(ctx, r);
}
// C line 46827
2 => {
let _ = { let assigned = (r).abs(); r = assigned; assigned };
vm_block = 1; continue;
}
// C line 46830
3 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 46833
5 => {
let _ = { let assigned = (r).hypot(a); r = assigned; assigned };
vm_block = 4; continue;
}
// C line 46832
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46831
7 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(a), *(argv).offset((i) as isize))) != 0 { 6 } else { 5 }; continue;
}
// C line 46830
8 => {
let _ = { let assigned = (1 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 46826
9 => {
vm_block = if ((((argc) == ((1 as i32))) as i32)) != 0 { 2 } else { 8 }; continue;
}
// C line 46825
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46824
11 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(r), *(argv).offset(((0 as i32)) as isize))) != 0 { 10 } else { 9 }; continue;
}
// C line 46823
12 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line 46822
13 => {
let _ = { let assigned = (((0 as i32)) as f64); r = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46840. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_f16round(mut a: f64) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46842
1 => {
return crate::cutils_header::fromfp16(crate::cutils_header::tofp16(a));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46845. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_fround(mut a: f64) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46847
1 => {
return ((((a) as f32)) as f64);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46850. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_imul(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut a: u32 = core::mem::zeroed();
let mut b: u32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut d: i32 = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46862
1 => {
return JS_NewInt32(ctx, d);
}
// C line 46861
2 => {
let _ = { let dst = (((core::ptr::addr_of_mut!(d)) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((core::ptr::addr_of_mut!(c)) as *const c_void)) as *const u8, dst, ((size_of::<i32>() as usize)) as usize); dst as *mut c_void };
vm_block = 1; continue;
}
// C line 46860
3 => {
let _ = { let assigned = (a).wrapping_mul(b); c = assigned; assigned };
vm_block = 2; continue;
}
// C line 46859
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46858
5 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(b), *(argv).offset(((1 as i32)) as isize))) != 0 { 4 } else { 3 }; continue;
}
// C line 46857
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46856
7 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(a), *(argv).offset(((0 as i32)) as isize))) != 0 { 6 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46865. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_clz32(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut a: u32 = core::mem::zeroed();
let mut r: u32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46876
1 => {
return JS_NewInt32(ctx, ((r) as i32));
}
// C line 46873
2 => {
let _ = { let assigned = (((32 as i32)) as u32); r = assigned; assigned };
vm_block = 1; continue;
}
// C line 46875
3 => {
let _ = { let assigned = ((crate::cutils_header::clz32(a)) as u32); r = assigned; assigned };
vm_block = 1; continue;
}
// C line 46872
4 => {
vm_block = if ((((a) == ((((0 as i32)) as u32))) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 46871
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46870
6 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(a), *(argv).offset(((0 as i32)) as isize))) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46902. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn sum_precise_init(mut s: *mut SumPreciseState) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 46907
1 => {
let _ = { let assigned = (0 as i32); (*(s)).n_limbs = assigned; assigned };
vm_block = 0; continue;
}
// C line 46906
2 => {
let _ = { let assigned = (((250 as i32)) as u32); (*(s)).counter = assigned; assigned };
vm_block = 1; continue;
}
// C line 46905
3 => {
let _ = { let assigned = (((SUM_PRECISE_STATE_FINITE as i32)) as SumPreciseStateEnum); (*(s)).state = assigned; assigned };
vm_block = 2; continue;
}
// C line 46904
4 => {
let _ = { let dst = (((((*(s)).acc).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<[i64; 39]>() as usize)) as usize); dst as *mut c_void };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46910. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn sum_precise_renorm(mut s: *mut SumPreciseState) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: i64 = core::mem::zeroed();
let mut carry: i64 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 46924
1 => {
let _ = { let assigned = carry; *(((*(s)).acc).as_mut_ptr()).offset(({ let old = (*(s)).n_limbs; (*(s)).n_limbs = ((*(s)).n_limbs).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 0; continue;
}
// C line 46923
2 => {
vm_block = if ((((((((carry) != ((((0 as i32)) as i64))) as i32)) != 0) && ((((((*(s)).n_limbs) < ((39 as i32))) as i32)) != 0)) as i32)) != 0 { 1 } else { 0 }; continue;
}
// C line 46916
3 => {
vm_block = if ((((i) < ((*(s)).n_limbs)) as i32)) != 0 { 7 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 46919
5 => {
let _ = { let assigned = (v).wrapping_shr(((56 as i32)) as u32); carry = assigned; assigned };
vm_block = 4; continue;
}
// C line 46918
6 => {
let _ = { let assigned = ((((((v) as u64)) & ((((((1 as i32)) as u64)).wrapping_shl(((56 as i32)) as u32)).wrapping_sub((((1 as i32)) as u64))))) as i64); *(((*(s)).acc).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 46917
7 => {
let _ = { let assigned = (*(((*(s)).acc).as_mut_ptr()).offset((i) as isize)).wrapping_add(carry); v = assigned; assigned };
vm_block = 6; continue;
}
// C line 46916
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 46915
9 => {
let _ = { let assigned = (((0 as i32)) as i64); carry = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46927. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn sum_precise_add(mut s: *mut SumPreciseState, mut d: f64) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut a: u64 = core::mem::zeroed();
let mut m: u64 = core::mem::zeroed();
let mut a0: u64 = core::mem::zeroed();
let mut a1: u64 = core::mem::zeroed();
let mut sgn: i32 = core::mem::zeroed();
let mut e: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut shift: u32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 46943
1 => {
let _ = { let assigned = (((SUM_PRECISE_STATE_NAN as i32)) as SumPreciseStateEnum); (*(s)).state = assigned; assigned };
vm_block = 0; continue;
}
// C line 46945
2 => {
let _ = { let assigned = ((((SUM_PRECISE_STATE_INFINITY as i32)).wrapping_add(sgn)) as SumPreciseStateEnum); (*(s)).state = assigned; assigned };
vm_block = 0; continue;
}
// C line 46940
3 => {
vm_block = if (((((((((((((((*(s)).state) as u32)) == ((((SUM_PRECISE_STATE_NAN as i32)) as u32))) as i32)) != 0) || ((((((((((((*(s)).state) as u32)) == ((((SUM_PRECISE_STATE_MINUS_INFINITY as i32)) as u32))) as i32)) != 0) && (((!((sgn) != 0) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || ((((((((((((*(s)).state) as u32)) == ((((SUM_PRECISE_STATE_INFINITY as i32)) as u32))) as i32)) != 0) && ((sgn) != 0)) as i32)) != 0)) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 46949
4 => {
let _ = { let assigned = (((SUM_PRECISE_STATE_NAN as i32)) as SumPreciseStateEnum); (*(s)).state = assigned; assigned };
vm_block = 0; continue;
}
// C line 46938
5 => {
vm_block = if ((((m) == ((((0 as i32)) as u64))) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 46955
6 => {
let _ = { let assigned = (1 as i32); (*(s)).n_limbs = assigned; assigned };
vm_block = 0; continue;
}
// C line 46954
7 => {
vm_block = if (((((((((*(s)).n_limbs) == ((0 as i32))) as i32)) != 0) && (((!((sgn) != 0) as i32)) != 0)) as i32)) != 0 { 6 } else { 0 }; continue;
}
// C line 46960
8 => {
vm_block = 22; continue;
}
// C line 46959
9 => {
let _ = { let assigned = (((0 as i32)) as u32); shift = assigned; assigned };
vm_block = 8; continue;
}
// C line 46958
10 => {
let _ = { let assigned = (0 as i32); p = assigned; assigned };
vm_block = 9; continue;
}
// C line 46952
11 => {
vm_block = if ((((!(((!(((((m) == ((((0 as i32)) as u64))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 7 } else { 10 }; continue;
}
// C line 46985
12 => {
let _ = sum_precise_renorm(s);
vm_block = 0; continue;
}
// C line 46984
13 => {
let _ = { let assigned = (((250 as i32)) as u32); (*(s)).counter = assigned; assigned };
vm_block = 12; continue;
}
// C line 46983
14 => {
vm_block = if ((((!(((!((((({ (*(s)).counter = ((*(s)).counter).wrapping_sub(1); (*(s)).counter }) == ((((0 as i32)) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 13 } else { 0 }; continue;
}
// C line 46981
15 => {
let _ = { let assigned = crate::cutils_header::max_int((*(s)).n_limbs, (p).wrapping_add((2 as i32))); (*(s)).n_limbs = assigned; assigned };
vm_block = 14; continue;
}
// C line 46976
16 => {
let _ = { *(((*(s)).acc).as_mut_ptr()).offset(((p).wrapping_add((1 as i32))) as isize) = (*(((*(s)).acc).as_mut_ptr()).offset(((p).wrapping_add((1 as i32))) as isize)).wrapping_add((a1) as i64); *(((*(s)).acc).as_mut_ptr()).offset(((p).wrapping_add((1 as i32))) as isize) };
vm_block = 15; continue;
}
// C line 46975
17 => {
let _ = { *(((*(s)).acc).as_mut_ptr()).offset((p) as isize) = (*(((*(s)).acc).as_mut_ptr()).offset((p) as isize)).wrapping_add((a0) as i64); *(((*(s)).acc).as_mut_ptr()).offset((p) as isize) };
vm_block = 16; continue;
}
// C line 46979
18 => {
let _ = { *(((*(s)).acc).as_mut_ptr()).offset(((p).wrapping_add((1 as i32))) as isize) = (*(((*(s)).acc).as_mut_ptr()).offset(((p).wrapping_add((1 as i32))) as isize)).wrapping_sub((a1) as i64); *(((*(s)).acc).as_mut_ptr()).offset(((p).wrapping_add((1 as i32))) as isize) };
vm_block = 15; continue;
}
// C line 46978
19 => {
let _ = { *(((*(s)).acc).as_mut_ptr()).offset((p) as isize) = (*(((*(s)).acc).as_mut_ptr()).offset((p) as isize)).wrapping_sub((a0) as i64); *(((*(s)).acc).as_mut_ptr()).offset((p) as isize) };
vm_block = 18; continue;
}
// C line 46974
20 => {
vm_block = if ((!((sgn) != 0) as i32)) != 0 { 17 } else { 19 }; continue;
}
// C line 46973
21 => {
let _ = { let assigned = (m).wrapping_shr((((((56 as i32)) as u32)).wrapping_sub(shift)) as u32); a1 = assigned; assigned };
vm_block = 20; continue;
}
// C line ? labels: add
22 => {
let _ = { let assigned = (((m).wrapping_shl((shift) as u32)) & ((((((1 as i32)) as u64)).wrapping_shl(((56 as i32)) as u32)).wrapping_sub((((1 as i32)) as u64)))); a0 = assigned; assigned };
vm_block = 21; continue;
}
// C line 46970
23 => {
let _ = { shift = ((shift) % ((((56 as i32)) as u32))); shift };
vm_block = 22; continue;
}
// C line 46969
24 => {
let _ = { let assigned = ((((shift) / ((((56 as i32)) as u32)))) as i32); p = assigned; assigned };
vm_block = 23; continue;
}
// C line 46966
25 => {
let _ = { let assigned = (((e).wrapping_sub((1 as i32))) as u32); shift = assigned; assigned };
vm_block = 24; continue;
}
// C line 46965
26 => {
let _ = { m = ((m) | (((((1 as i32)) as u64)).wrapping_shl(((52 as i32)) as u32))); m };
vm_block = 25; continue;
}
// C line 46951
27 => {
vm_block = if ((((e) == ((0 as i32))) as i32)) != 0 { 11 } else { 26 }; continue;
}
// C line 46937
28 => {
vm_block = if ((((!(((!(((((e) == ((2047 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 5 } else { 27 }; continue;
}
// C line 46936
29 => {
let _ = { let assigned = ((a) & ((((((1 as i32)) as u64)).wrapping_shl(((52 as i32)) as u32)).wrapping_sub((((1 as i32)) as u64)))); m = assigned; assigned };
vm_block = 28; continue;
}
// C line 46935
30 => {
let _ = { let assigned = (((((a).wrapping_shr(((52 as i32)) as u32)) & ((((((1 as i32)).wrapping_shl(((11 as i32)) as u32)).wrapping_sub((1 as i32))) as u64)))) as i32); e = assigned; assigned };
vm_block = 29; continue;
}
// C line 46934
31 => {
let _ = { let assigned = (((a).wrapping_shr(((63 as i32)) as u32)) as i32); sgn = assigned; assigned };
vm_block = 30; continue;
}
// C line 46933
32 => {
let _ = { let assigned = crate::cutils_header::float64_as_uint64(d); a = assigned; assigned };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46990. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn sum_precise_get_result(mut s: *mut SumPreciseState) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut n: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut e: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut is_neg: i32 = core::mem::zeroed();
let mut m: u64 = core::mem::zeroed();
let mut addend: u64 = core::mem::zeroed();
let mut v: u64 = core::mem::zeroed();
let mut carry: u64 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut shift1: i32 = core::mem::zeroed();
let mut nz: u64 = core::mem::zeroed();
let mut vm_block: usize = 53;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47084
1 => {
return crate::cutils_header::uint64_as_float64((((((is_neg) as u64)).wrapping_shl(((63 as i32)) as u32)) | (((((2047 as i32)) as u64)).wrapping_shl(((52 as i32)) as u32))));
}
// C line 47087
2 => {
return crate::cutils_header::uint64_as_float64((((((((is_neg) as u64)).wrapping_shl(((63 as i32)) as u32)) | ((((e) as u64)).wrapping_shl(((52 as i32)) as u32)))) | (m)));
}
// C line 47086
3 => {
let _ = { m = ((m) & ((((((1 as i32)) as u64)).wrapping_shl(((52 as i32)) as u32)).wrapping_sub((((1 as i32)) as u64)))); m };
vm_block = 2; continue;
}
// C line 47082
4 => {
vm_block = if ((((!(((!(((((e) >= ((2047 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1 } else { 3 }; continue;
}
// C line 47081
5 => {
let _ = { let old = e; e = (e).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 47080
6 => {
vm_block = if ((((m) == (((((1 as i32)) as u64)).wrapping_shl(((53 as i32)) as u32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 47078
7 => {
let _ = { let assigned = ((m).wrapping_add(addend)).wrapping_shr((((56 as i32)).wrapping_sub((53 as i32))) as u32); m = assigned; assigned };
vm_block = 6; continue;
}
// C line 47077
8 => {
let _ = { let assigned = ((((((1 as i32)).wrapping_shl(((((56 as i32)).wrapping_sub((53 as i32))).wrapping_sub((1 as i32))) as u32)).wrapping_sub((1 as i32))) as u64)).wrapping_add((((m).wrapping_shr((((56 as i32)).wrapping_sub((53 as i32))) as u32)) & ((((1 as i32)) as u64)))); addend = assigned; assigned };
vm_block = 7; continue;
}
// C line 47068
9 => {
vm_block = if ((((p) > ((0 as i32))) as i32)) != 0 { 13 } else { 8 }; continue;
}
// C line 47072
10 => {
vm_block = 8; continue;
}
// C line 47071
11 => {
let _ = { m = ((m) | ((((1 as i32)) as u64))); m };
vm_block = 10; continue;
}
// C line 47070
12 => {
vm_block = if ((((*(((*(s)).acc).as_mut_ptr()).offset((p) as isize)) != ((((0 as i32)) as i64))) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 47069
13 => {
let _ = { let old = p; p = (p).wrapping_sub(1); old };
vm_block = 12; continue;
}
// C line 47066
14 => {
vm_block = if ((((((m) & ((((((1 as i32)).wrapping_shl((((56 as i32)).wrapping_sub((53 as i32))) as u32)).wrapping_sub((1 as i32))) as u64)))) == (((((1 as i32)).wrapping_shl(((((56 as i32)).wrapping_sub((53 as i32))).wrapping_sub((1 as i32))) as u32)) as u64))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 47063
15 => {
let _ = { let assigned = ((((m) | ((((*(((*(s)).acc).as_mut_ptr()).offset((p) as isize)).wrapping_shr((shift1) as u32)) as u64)))) | ((((((nz) != ((((0 as i32)) as u64))) as i32)) as u64))); m = assigned; assigned };
vm_block = 14; continue;
}
// C line 47062
16 => {
let _ = { let assigned = ((((*(((*(s)).acc).as_mut_ptr()).offset((p) as isize)) as u64)) & ((((((1 as i32)) as u64)).wrapping_shl((shift1) as u32)).wrapping_sub((((1 as i32)) as u64)))); nz = assigned; assigned };
vm_block = 15; continue;
}
// C line 47061
17 => {
let _ = { let assigned = ((56 as i32)).wrapping_sub(shift); shift1 = assigned; assigned };
vm_block = 16; continue;
}
// C line 47060
18 => {
let _ = { let old = p; p = (p).wrapping_sub(1); old };
vm_block = 17; continue;
}
// C line 47057
19 => {
vm_block = if ((((p) > ((0 as i32))) as i32)) != 0 { 18 } else { 14 }; continue;
}
// C line 47056
20 => {
let _ = { m = (m).wrapping_shl((shift) as u32); m };
vm_block = 19; continue;
}
// C line 47055
21 => {
vm_block = if ((((shift) != ((0 as i32))) as i32)) != 0 { 20 } else { 14 }; continue;
}
// C line 47054
22 => {
let _ = { let assigned = ((e).wrapping_sub(shift)).wrapping_sub((52 as i32)); e = assigned; assigned };
vm_block = 21; continue;
}
// C line 47053
23 => {
let _ = { let assigned = (crate::cutils_header::clz64(m)).wrapping_sub(((64 as i32)).wrapping_sub((56 as i32))); shift = assigned; assigned };
vm_block = 22; continue;
}
// C line 47052
24 => {
let _ = { let assigned = ((*(((*(s)).acc).as_mut_ptr()).offset((p) as isize)) as u64); m = assigned; assigned };
vm_block = 23; continue;
}
// C line 47051
25 => {
let _ = { let assigned = (n).wrapping_sub((1 as i32)); p = assigned; assigned };
vm_block = 24; continue;
}
// C line 47050
26 => {
let _ = { let assigned = (n).wrapping_mul((56 as i32)); e = assigned; assigned };
vm_block = 25; continue;
}
// C line 47048
27 => {
return crate::cutils_header::uint64_as_float64((((((is_neg) as u64)).wrapping_shl(((63 as i32)) as u32)) | (((*(((*(s)).acc).as_mut_ptr()).offset(((0 as i32)) as isize)) as u64))));
}
// C line 47047
28 => {
vm_block = if ((((((((n) == ((1 as i32))) as i32)) != 0) && (((((((*(((*(s)).acc).as_mut_ptr()).offset(((0 as i32)) as isize)) as u64)) < (((((1 as i32)) as u64)).wrapping_shl(((52 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 47043
29 => {
vm_block = if ((((((((n) > ((1 as i32))) as i32)) != 0) && (((((*(((*(s)).acc).as_mut_ptr()).offset(((n).wrapping_sub((1 as i32))) as isize)) == ((((0 as i32)) as i64))) as i32)) != 0)) as i32)) != 0 { 30 } else { 28 }; continue;
}
// C line 47044
30 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 29; continue;
}
// C line 47042
31 => {
let _ = { let assigned = (((((((*(((*(s)).acc).as_mut_ptr()).offset(((n).wrapping_sub((1 as i32))) as isize)).wrapping_neg()) as u64)).wrapping_add(carry)).wrapping_sub((((1 as i32)) as u64))) as i64); *(((*(s)).acc).as_mut_ptr()).offset(((n).wrapping_sub((1 as i32))) as isize) = assigned; assigned };
vm_block = 29; continue;
}
// C line 47037
32 => {
vm_block = if ((((i) < ((n).wrapping_sub((1 as i32)))) as i32)) != 0 { 36 } else { 31 }; continue;
}
// C line ?
33 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 32; continue;
}
// C line 47040
34 => {
let _ = { let assigned = ((((v) & ((((((1 as i32)) as u64)).wrapping_shl(((56 as i32)) as u32)).wrapping_sub((((1 as i32)) as u64))))) as i64); *(((*(s)).acc).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 33; continue;
}
// C line 47039
35 => {
let _ = { let assigned = (v).wrapping_shr(((56 as i32)) as u32); carry = assigned; assigned };
vm_block = 34; continue;
}
// C line 47038
36 => {
let _ = { let assigned = (((((((1 as i32)) as u64)).wrapping_shl(((56 as i32)) as u32)).wrapping_sub((((1 as i32)) as u64))).wrapping_sub(((*(((*(s)).acc).as_mut_ptr()).offset((i) as isize)) as u64))).wrapping_add(carry); v = assigned; assigned };
vm_block = 35; continue;
}
// C line 47037
37 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 32; continue;
}
// C line 47036
38 => {
let _ = { let assigned = (((1 as i32)) as u64); carry = assigned; assigned };
vm_block = 37; continue;
}
// C line 47031
39 => {
vm_block = if (is_neg) != 0 { 38 } else { 28 }; continue;
}
// C line 47030
40 => {
let _ = { let assigned = (((*(((*(s)).acc).as_mut_ptr()).offset(((n).wrapping_sub((1 as i32))) as isize)) < ((((0 as i32)) as i64))) as i32); is_neg = assigned; assigned };
vm_block = 39; continue;
}
// C line 47029
41 => {
return (0 as f64);
}
// C line 47028
42 => {
vm_block = if ((((n) == ((0 as i32))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 47025
43 => {
vm_block = if ((((((((n) > ((0 as i32))) as i32)) != 0) && (((((*(((*(s)).acc).as_mut_ptr()).offset(((n).wrapping_sub((1 as i32))) as isize)) == ((((0 as i32)) as i64))) as i32)) != 0)) as i32)) != 0 { 44 } else { 42 }; continue;
}
// C line 47026
44 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 43; continue;
}
// C line 47022
45 => {
return (-((0 as f64)));
}
// C line 47021
46 => {
vm_block = if ((((n) == ((0 as i32))) as i32)) != 0 { 45 } else { 43 }; continue;
}
// C line 47019
47 => {
let _ = { let assigned = (*(s)).n_limbs; n = assigned; assigned };
vm_block = 46; continue;
}
// C line 47007
48 => {
let _ = sum_precise_renorm(s);
vm_block = 47; continue;
}
// C line 47003
49 => {
return ((f32::NAN) as f64);
}
// C line 47001
50 => {
return (((-(f32::INFINITY))) as f64);
}
// C line 46999
51 => {
return ((f32::INFINITY) as f64);
}
// C line 46996
52 => {
vm_block = match (((*(s)).state) as u32) { x if x == (((SUM_PRECISE_STATE_NAN as i32)) as u32) => 49, x if x == (((SUM_PRECISE_STATE_MINUS_INFINITY as i32)) as u32) => 50, x if x == (((SUM_PRECISE_STATE_INFINITY as i32)) as u32) => 51, _ => 51, }; continue;
}
// C line 46995
53 => {
vm_block = if (((((((*(s)).state) as u32)) != ((((SUM_PRECISE_STATE_FINITE as i32)) as u32))) as i32)) != 0 { 52 } else { 48 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47091. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_sumPrecise(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut iter: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut tag: u32 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut s_s: SumPreciseState = core::mem::zeroed();
let mut s: *mut SumPreciseState = core::mem::zeroed();
let mut vm_block: usize = 29;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47131
1 => {
return ret;
}
// C line 47130
2 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line 47127
4 => {
let _ = { let assigned = __JS_NewFloat64(ctx, sum_precise_get_result(s)); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 47108
5 => {
vm_block = 20; continue;
}
// C line 47125
6 => {
let _ = sum_precise_add(s, d);
vm_block = 5; continue;
}
// C line 47116
7 => {
let _ = { let assigned = ((item).u).float64; d = assigned; assigned };
vm_block = 6; continue;
}
// C line 47118
8 => {
let _ = { let assigned = ((((((item).u).uint64) as i32)) as f64); d = assigned; assigned };
vm_block = 6; continue;
}
// C line 47123
9 => {
vm_block = 3; continue;
}
// C line 47122
10 => {
let _ = JS_IteratorClose(ctx, iter, (1 as i32));
vm_block = 9; continue;
}
// C line 47121
11 => {
let _ = JS_ThrowTypeError(ctx, c"not a number".as_ptr());
vm_block = 10; continue;
}
// C line 47120
12 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 11; continue;
}
// C line 47117
13 => {
vm_block = if ((((tag) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0 { 8 } else { 12 }; continue;
}
// C line 47115
14 => {
vm_block = if ((((tag) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 7 } else { 13 }; continue;
}
// C line 47114
15 => {
let _ = { let assigned = (((((item).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 14; continue;
}
// C line 47113
16 => {
vm_block = 4; continue;
}
// C line 47112
17 => {
vm_block = if (done) != 0 { 16 } else { 15 }; continue;
}
// C line 47111
18 => {
vm_block = 3; continue;
}
// C line 47110
19 => {
vm_block = if (JS_IsException(item)) != 0 { 18 } else { 17 }; continue;
}
// C line 47109
20 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 19; continue;
}
// C line 47107
21 => {
let _ = sum_precise_init(s);
vm_block = 5; continue;
}
// C line 47106
22 => {
vm_block = 3; continue;
}
// C line 47105
23 => {
vm_block = if (JS_IsException(next)) != 0 { 22 } else { 21 }; continue;
}
// C line 47104
24 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 23; continue;
}
// C line 47103
25 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 24; continue;
}
// C line 47102
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47101
27 => {
vm_block = if (JS_IsException(iter)) != 0 { 26 } else { 25 }; continue;
}
// C line 47100
28 => {
let _ = { let assigned = JS_GetIterator(ctx, *(argv).offset(((0 as i32)) as isize), (0 as i32)); iter = assigned; assigned };
vm_block = 27; continue;
}
// C line 47098
29 => {
s = core::ptr::addr_of_mut!(s_s);
vm_block = 28; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47135. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn xorshift64star(mut pstate: *mut u64) -> u64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut x: u64 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47143
1 => {
return (x).wrapping_mul((((2685821657736338717 as i64)) as u64));
}
// C line 47142
2 => {
let _ = { let assigned = x; *(pstate) = assigned; assigned };
vm_block = 1; continue;
}
// C line 47141
3 => {
let _ = { x = ((x) ^ ((x).wrapping_shr(((27 as i32)) as u32))); x };
vm_block = 2; continue;
}
// C line 47140
4 => {
let _ = { x = ((x) ^ ((x).wrapping_shl(((25 as i32)) as u32))); x };
vm_block = 3; continue;
}
// C line 47139
5 => {
let _ = { x = ((x) ^ ((x).wrapping_shr(((12 as i32)) as u32))); x };
vm_block = 4; continue;
}
// C line 47138
6 => {
let _ = { let assigned = *(pstate); x = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47156. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_math_random(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut u: JSFloat64Union = core::mem::zeroed();
let mut v: u64 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47165
1 => {
return __JS_NewFloat64(ctx, (((u).d) - ((1 as f64))));
}
// C line 47164
2 => {
let _ = { let assigned = ((((((1023 as i32)) as u64)).wrapping_shl(((52 as i32)) as u32)) | ((v).wrapping_shr(((12 as i32)) as u32))); (u).u64 = assigned; assigned };
vm_block = 1; continue;
}
// C line 47162
3 => {
let _ = { let assigned = xorshift64star(core::ptr::addr_of_mut!((*(ctx)).random_state)); v = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}
