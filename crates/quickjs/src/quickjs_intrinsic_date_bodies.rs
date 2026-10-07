// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54788. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn math_mod(mut a: i64, mut b: i64) -> i64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut m: i64 = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54791
1 => {
return (m).wrapping_add(((((((m) < ((((0 as i32)) as i64))) as i32)) as i64)).wrapping_mul(b));
}
// C line 54790
2 => {
m = ((a) % (b));
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54794. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn floor_div(mut a: i64, mut b: i64) -> i64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut m: i64 = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54797
1 => {
return (((a).wrapping_sub((m).wrapping_add(((((((m) < ((((0 as i32)) as i64))) as i32)) as i64)).wrapping_mul(b)))) / (b));
}
// C line 54796
2 => {
m = ((a) % (b));
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54803. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ThisTimeValue(mut ctx: *mut JSContext, mut valp: *mut f64, mut this_val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54811
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54810
2 => {
let _ = JS_ThrowTypeError(ctx, c"not a Date object".as_ptr());
vm_block = 1; continue;
}
// C line 54808
3 => {
return JS_ToFloat64(ctx, valp, ((*(p)).u).object_data);
}
// C line 54807
4 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_DATE as i32))) as i32)) != 0) && ((JS_IsNumber(((*(p)).u).object_data)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 54806
5 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 4; continue;
}
// C line 54805
6 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 5 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54814. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_SetThisTimeValue(mut ctx: *mut JSContext, mut this_val: JSValue, mut v: f64) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54824
1 => {
return JS_ThrowTypeError(ctx, c"not a Date object".as_ptr());
}
// C line 54821
2 => {
return JS_DupValue(ctx, ((*(p)).u).object_data);
}
// C line 54820
3 => {
let _ = { let assigned = JS_NewFloat64(ctx, v); ((*(p)).u).object_data = assigned; assigned };
vm_block = 2; continue;
}
// C line 54819
4 => {
let _ = JS_FreeValue(ctx, ((*(p)).u).object_data);
vm_block = 3; continue;
}
// C line 54818
5 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_DATE as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 54817
6 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 5; continue;
}
// C line 54816
7 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 6 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54827. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn days_from_year(mut y: i64) -> i64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54828
1 => {
return (((((((365 as i32)) as i64)).wrapping_mul((y).wrapping_sub((((1970 as i32)) as i64)))).wrapping_add(floor_div((y).wrapping_sub((((1969 as i32)) as i64)), (((4 as i32)) as i64)))).wrapping_sub(floor_div((y).wrapping_sub((((1901 as i32)) as i64)), (((100 as i32)) as i64)))).wrapping_add(floor_div((y).wrapping_sub((((1601 as i32)) as i64)), (((400 as i32)) as i64)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54832. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn days_in_year(mut y: i64) -> i64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54833
1 => {
return ((((((365 as i32)).wrapping_add((!((((y) % ((((4 as i32)) as i64)))) != 0) as i32))).wrapping_sub((!((((y) % ((((100 as i32)) as i64)))) != 0) as i32))).wrapping_add((!((((y) % ((((400 as i32)) as i64)))) != 0) as i32))) as i64);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54837. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn year_from_days(mut days: *mut i64) -> i64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut y: i64 = core::mem::zeroed();
let mut d1: i64 = core::mem::zeroed();
let mut nd: i64 = core::mem::zeroed();
let mut d: i64 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54856
1 => {
return y;
}
// C line 54855
2 => {
let _ = { let assigned = d1; *(days) = assigned; assigned };
vm_block = 1; continue;
}
// C line 54842
3 => {
vm_block = 12; continue;
}
// C line 54846
4 => {
let _ = { d1 = (d1).wrapping_add(days_in_year(y)); d1 };
vm_block = 3; continue;
}
// C line 54845
5 => {
let _ = { let old = y; y = (y).wrapping_sub(1); old };
vm_block = 4; continue;
}
// C line 54852
6 => {
let _ = { let old = y; y = (y).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 54851
7 => {
let _ = { d1 = (d1).wrapping_sub(nd); d1 };
vm_block = 6; continue;
}
// C line 54850
8 => {
vm_block = 2; continue;
}
// C line 54849
9 => {
vm_block = if ((((d1) < (nd)) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 54848
10 => {
let _ = { let assigned = days_in_year(y); nd = assigned; assigned };
vm_block = 9; continue;
}
// C line 54844
11 => {
vm_block = if ((((d1) < ((((0 as i32)) as i64))) as i32)) != 0 { 5 } else { 10 }; continue;
}
// C line 54843
12 => {
let _ = { let assigned = (d).wrapping_sub(days_from_year(y)); d1 = assigned; assigned };
vm_block = 11; continue;
}
// C line 54839
13 => {
let _ = { let assigned = (floor_div((d).wrapping_mul((((10000 as i32)) as i64)), (((3652425 as i32)) as i64))).wrapping_add((((1970 as i32)) as i64)); y = assigned; assigned };
vm_block = 3; continue;
}
// C line 54838
14 => {
d = *(days);
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54863. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_date_fields(mut ctx: *mut JSContext, mut obj: JSValue, mut fields: *mut f64, mut is_local: i32, mut force: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut dval: f64 = core::mem::zeroed();
let mut d: i64 = core::mem::zeroed();
let mut days: i64 = core::mem::zeroed();
let mut wd: i64 = core::mem::zeroed();
let mut y: i64 = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut md: i64 = core::mem::zeroed();
let mut h: i64 = core::mem::zeroed();
let mut m: i64 = core::mem::zeroed();
let mut s: i64 = core::mem::zeroed();
let mut ms: i64 = core::mem::zeroed();
let mut tz: i64 = core::mem::zeroed();
let mut vm_block: usize = 40;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54913
1 => {
return (1 as i32);
}
// C line 54912
2 => {
let _ = { let assigned = ((tz) as f64); *(fields).offset(((8 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 54911
3 => {
let _ = { let assigned = ((wd) as f64); *(fields).offset(((7 as i32)) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 54910
4 => {
let _ = { let assigned = ((ms) as f64); *(fields).offset(((6 as i32)) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 54909
5 => {
let _ = { let assigned = ((s) as f64); *(fields).offset(((5 as i32)) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 54908
6 => {
let _ = { let assigned = ((m) as f64); *(fields).offset(((4 as i32)) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 54907
7 => {
let _ = { let assigned = ((h) as f64); *(fields).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 54906
8 => {
let _ = { let assigned = (((days).wrapping_add((((1 as i32)) as i64))) as f64); *(fields).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 54905
9 => {
let _ = { let assigned = ((i) as f64); *(fields).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 54904
10 => {
let _ = { let assigned = ((y) as f64); *(fields).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 54896
11 => {
vm_block = if ((((i) < ((((11 as i32)) as i64))) as i32)) != 0 { 18 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 54902
13 => {
let _ = { days = (days).wrapping_sub(md); days };
vm_block = 12; continue;
}
// C line 54901
14 => {
vm_block = 10; continue;
}
// C line 54900
15 => {
vm_block = if ((((days) < (md)) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 54899
16 => {
let _ = { md = (md).wrapping_add((days_in_year(y)).wrapping_sub((((365 as i32)) as i64))); md };
vm_block = 15; continue;
}
// C line 54898
17 => {
vm_block = if ((((i) == ((((1 as i32)) as i64))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 54897
18 => {
let _ = { let assigned = ((*((month_days).as_ptr()).offset((i) as isize)) as i64); md = assigned; assigned };
vm_block = 17; continue;
}
// C line 54896
19 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 54894
20 => {
let _ = { let assigned = year_from_days(core::ptr::addr_of_mut!(days)); y = assigned; assigned };
vm_block = 19; continue;
}
// C line 54893
21 => {
let _ = { let assigned = math_mod((days).wrapping_add((((4 as i32)) as i64)), (((7 as i32)) as i64)); wd = assigned; assigned };
vm_block = 20; continue;
}
// C line 54892
22 => {
let _ = { let assigned = (((h).wrapping_sub(m)) / ((((60 as i32)) as i64))); h = assigned; assigned };
vm_block = 21; continue;
}
// C line 54891
23 => {
let _ = { let assigned = ((h) % ((((60 as i32)) as i64))); m = assigned; assigned };
vm_block = 22; continue;
}
// C line 54890
24 => {
let _ = { let assigned = (((h).wrapping_sub(s)) / ((((60 as i32)) as i64))); h = assigned; assigned };
vm_block = 23; continue;
}
// C line 54889
25 => {
let _ = { let assigned = ((h) % ((((60 as i32)) as i64))); s = assigned; assigned };
vm_block = 24; continue;
}
// C line 54888
26 => {
let _ = { let assigned = (((h).wrapping_sub(ms)) / ((((1000 as i32)) as i64))); h = assigned; assigned };
vm_block = 25; continue;
}
// C line 54887
27 => {
let _ = { let assigned = ((h) % ((((1000 as i32)) as i64))); ms = assigned; assigned };
vm_block = 26; continue;
}
// C line 54886
28 => {
let _ = { let assigned = (((d).wrapping_sub(h)) / ((((86400000 as i32)) as i64))); days = assigned; assigned };
vm_block = 27; continue;
}
// C line 54885
29 => {
let _ = { let assigned = math_mod(d, (((86400000 as i32)) as i64)); h = assigned; assigned };
vm_block = 28; continue;
}
// C line 54875
30 => {
let _ = { let assigned = (((0 as i32)) as i64); d = assigned; assigned };
vm_block = 29; continue;
}
// C line 54874
31 => {
return (0 as i32);
}
// C line 54873
32 => {
vm_block = if ((!((force) != 0) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 54880
33 => {
let _ = { d = (d).wrapping_add((tz).wrapping_mul((((60000 as i32)) as i64))); d };
vm_block = 29; continue;
}
// C line 54879
34 => {
let _ = { let assigned = (((getTimezoneOffset(d)).wrapping_neg()) as i64); tz = assigned; assigned };
vm_block = 33; continue;
}
// C line 54878
35 => {
vm_block = if (is_local) != 0 { 34 } else { 29 }; continue;
}
// C line 54877
36 => {
let _ = { let assigned = ((dval) as i64); d = assigned; assigned };
vm_block = 35; continue;
}
// C line 54872
37 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((dval) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (dval).is_nan() as i32 } else { (dval).is_nan() as i32 } }) != 0 { 32 } else { 36 }; continue;
}
// C line 54870
38 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54869
39 => {
vm_block = if (JS_ThisTimeValue(ctx, core::ptr::addr_of_mut!(dval), obj)) != 0 { 38 } else { 37 }; continue;
}
// C line 54867
40 => {
tz = (((0 as i32)) as i64);
vm_block = 39; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54916. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn time_clip(mut t: f64) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54918
1 => {
return (((t).trunc()) + ((0 as f64)));
}
// C line 54920
2 => {
return ((f32::NAN) as f64);
}
// C line 54917
3 => {
vm_block = if ((((((((t) >= ((-((8.64E+15 as f64))))) as i32)) != 0) && (((((t) <= ((8.64E+15 as f64))) as i32)) != 0)) as i32)) != 0 { 1 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54925. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn set_date_fields(mut fields: *mut f64, mut is_local: i32) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut y: f64 = core::mem::zeroed();
let mut m: f64 = core::mem::zeroed();
let mut dt: f64 = core::mem::zeroed();
let mut ym: f64 = core::mem::zeroed();
let mut mn: f64 = core::mem::zeroed();
let mut day: f64 = core::mem::zeroed();
let mut h: f64 = core::mem::zeroed();
let mut s: f64 = core::mem::zeroed();
let mut milli: f64 = core::mem::zeroed();
let mut time: f64 = core::mem::zeroed();
let mut tv: f64 = core::mem::zeroed();
let mut yi: i32 = core::mem::zeroed();
let mut mi: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut days: i64 = core::mem::zeroed();
let mut temp: f64 = core::mem::zeroed();
let mut ti: i64 = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54979
1 => {
return time_clip(tv);
}
// C line 54977
2 => {
let _ = { tv = ((tv) + ((((getTimezoneOffset(ti)).wrapping_mul((60000 as i32))) as f64))); tv };
vm_block = 1; continue;
}
// C line 54976
3 => {
ti = if ((((tv) < ((((((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64))) as f64))) as i32)) != 0 { (((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64)) } else { if ((((tv) >= ((9.2233720368547758E+18 as f64))) as i32)) != 0 { (9223372036854775807 as i64) } else { ((tv) as i64) } };
vm_block = 2; continue;
}
// C line 54975
4 => {
vm_block = if (is_local) != 0 { 3 } else { 1 }; continue;
}
// C line 54972
5 => {
return ((f32::NAN) as f64);
}
// C line 54971
6 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((tv) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (tv).is_finite() as i32 } else { (tv).is_finite() as i32 } }) != 0) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 54970
7 => {
let _ = { let assigned = (({ let assigned = ((day) * ((((86400000 as i32)) as f64))); ptr::write_volatile(ptr::addr_of_mut!(temp), assigned); assigned }) + (time)); tv = assigned; assigned };
vm_block = 6; continue;
}
// C line 54967
8 => {
let _ = { time = ((time) + (milli)); time };
vm_block = 7; continue;
}
// C line 54966
9 => {
let _ = { time = ((time) + ({ let assigned = ((s) * ((((1000 as i32)) as f64))); ptr::write_volatile(ptr::addr_of_mut!(temp), assigned); assigned })); time };
vm_block = 8; continue;
}
// C line 54965
10 => {
let _ = { time = ((time) + ({ let assigned = ((m) * ((((60000 as i32)) as f64))); ptr::write_volatile(ptr::addr_of_mut!(temp), assigned); assigned })); time };
vm_block = 9; continue;
}
// C line 54964
11 => {
let _ = { let assigned = ((h) * ((((3600000 as i32)) as f64))); time = assigned; assigned };
vm_block = 10; continue;
}
// C line 54956
12 => {
let _ = { let assigned = *(fields).offset(((6 as i32)) as isize); milli = assigned; assigned };
vm_block = 11; continue;
}
// C line 54955
13 => {
let _ = { let assigned = *(fields).offset(((5 as i32)) as isize); s = assigned; assigned };
vm_block = 12; continue;
}
// C line 54954
14 => {
let _ = { let assigned = *(fields).offset(((4 as i32)) as isize); m = assigned; assigned };
vm_block = 13; continue;
}
// C line 54953
15 => {
let _ = { let assigned = *(fields).offset(((3 as i32)) as isize); h = assigned; assigned };
vm_block = 14; continue;
}
// C line 54950
16 => {
let _ = { let assigned = ((((((days) as f64)) + (dt))) - ((((1 as i32)) as f64))); day = assigned; assigned };
vm_block = 15; continue;
}
// C line 54945
17 => {
vm_block = if ((((i) < (mi)) as i32)) != 0 { 21 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 54948
19 => {
let _ = { days = (days).wrapping_add((days_in_year(((yi) as i64))).wrapping_sub((((365 as i32)) as i64))); days };
vm_block = 18; continue;
}
// C line 54947
20 => {
vm_block = if ((((i) == ((1 as i32))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 54946
21 => {
let _ = { days = (days).wrapping_add(((*((month_days).as_ptr()).offset((i) as isize)) as i64)); days };
vm_block = 20; continue;
}
// C line 54945
22 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 54944
23 => {
let _ = { let assigned = days_from_year(((yi) as i64)); days = assigned; assigned };
vm_block = 22; continue;
}
// C line 54943
24 => {
let _ = { let assigned = ((mn) as i32); mi = assigned; assigned };
vm_block = 23; continue;
}
// C line 54942
25 => {
let _ = { let assigned = ((ym) as i32); yi = assigned; assigned };
vm_block = 24; continue;
}
// C line 54940
26 => {
return ((f32::NAN) as f64);
}
// C line 54939
27 => {
vm_block = if ((((((((ym) < (((((271821 as i32)).wrapping_neg()) as f64))) as i32)) != 0) || (((((ym) > ((((275760 as i32)) as f64))) as i32)) != 0)) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 54938
28 => {
let _ = { mn = ((mn) + ((((12 as i32)) as f64))); mn };
vm_block = 27; continue;
}
// C line 54937
29 => {
vm_block = if ((((mn) < ((((0 as i32)) as f64))) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 54936
30 => {
let _ = { let assigned = (m % (((12 as i32)) as f64)); mn = assigned; assigned };
vm_block = 29; continue;
}
// C line 54935
31 => {
let _ = { let assigned = ((y) + ((((m) / ((((12 as i32)) as f64)))).floor())); ym = assigned; assigned };
vm_block = 30; continue;
}
// C line 54934
32 => {
let _ = { let assigned = *(fields).offset(((2 as i32)) as isize); dt = assigned; assigned };
vm_block = 31; continue;
}
// C line 54933
33 => {
let _ = { let assigned = *(fields).offset(((1 as i32)) as isize); m = assigned; assigned };
vm_block = 32; continue;
}
// C line 54932
34 => {
let _ = { let assigned = *(fields).offset(((0 as i32)) as isize); y = assigned; assigned };
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54982. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn set_date_fields_checked(mut fields: *mut f64, mut is_local: i32) -> f64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut a: f64 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54994
1 => {
return set_date_fields(fields, is_local);
}
// C line 54986
2 => {
vm_block = if ((((i) < ((7 as i32))) as i32)) != 0 { 9 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 54992
4 => {
let _ = { *(fields).offset(((0 as i32)) as isize) = ((*(fields).offset(((0 as i32)) as isize)) + ((((1900 as i32)) as f64))); *(fields).offset(((0 as i32)) as isize) };
vm_block = 3; continue;
}
// C line 54991
5 => {
vm_block = if ((((((((((((i) == ((0 as i32))) as i32)) != 0) && (((((*(fields).offset(((0 as i32)) as isize)) >= ((((0 as i32)) as f64))) as i32)) != 0)) as i32)) != 0) && (((((*(fields).offset(((0 as i32)) as isize)) < ((((100 as i32)) as f64))) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 54990
6 => {
let _ = { let assigned = (a).trunc(); *(fields).offset((i) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 54989
7 => {
return ((f32::NAN) as f64);
}
// C line 54988
8 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((a) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (a).is_finite() as i32 } else { (a).is_finite() as i32 } }) != 0) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 54987
9 => {
let _ = { let assigned = *(fields).offset((i) as isize); a = assigned; assigned };
vm_block = 8; continue;
}
// C line 54986
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54997. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_date_field(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut fields: [f64; 9] = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut is_local: i32 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55015
1 => {
return JS_NewFloat64(ctx, *((fields).as_mut_ptr()).offset((n) as isize));
}
// C line 55013
2 => {
let _ = { *((fields).as_mut_ptr()).offset(((0 as i32)) as isize) = ((*((fields).as_mut_ptr()).offset(((0 as i32)) as isize)) - ((((1900 as i32)) as f64))); *((fields).as_mut_ptr()).offset(((0 as i32)) as isize) };
vm_block = 1; continue;
}
// C line 55012
3 => {
vm_block = if (((magic) & ((256 as i32)))) != 0 { 2 } else { 1 }; continue;
}
// C line 55010
4 => {
return JSValue { u: JSValueUnion { float64: ((f32::NAN) as f64) }, tag: (((JS_TAG_FLOAT64 as i32)) as i64) };
}
// C line 55009
5 => {
vm_block = if ((!((res) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 55008
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55007
7 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 55006
8 => {
let _ = { let assigned = get_date_fields(ctx, this_val, (fields).as_mut_ptr(), is_local, (0 as i32)); res = assigned; assigned };
vm_block = 7; continue;
}
// C line 55005
9 => {
let _ = { let assigned = (((magic).wrapping_shr(((4 as i32)) as u32)) & ((15 as i32))); n = assigned; assigned };
vm_block = 8; continue;
}
// C line 55004
10 => {
let _ = { let assigned = ((magic) & ((15 as i32))); is_local = assigned; assigned };
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55018. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn set_date_field(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut fields: [f64; 9] = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut first_field: i32 = core::mem::zeroed();
let mut end_field: i32 = core::mem::zeroed();
let mut is_local: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut res1: i32 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut a: f64 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55052
1 => {
return JS_SetThisTimeValue(ctx, this_val, d);
}
// C line 55050
2 => {
let _ = { let assigned = set_date_fields((fields).as_mut_ptr(), is_local); d = assigned; assigned };
vm_block = 1; continue;
}
// C line 55049
3 => {
vm_block = if (((((res) != 0) && (((((argc) > ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 55047
4 => {
return JSValue { u: JSValueUnion { float64: ((f32::NAN) as f64) }, tag: (((JS_TAG_FLOAT64 as i32)) as i64) };
}
// C line 55046
5 => {
vm_block = if ((!((res1) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 55038
6 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 12 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 55043
8 => {
let _ = { let assigned = (a).trunc(); *((fields).as_mut_ptr()).offset(((first_field).wrapping_add(i)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 55042
9 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 8; continue;
}
// C line 55041
10 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((a) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (a).is_finite() as i32 } else { (a).is_finite() as i32 } }) != 0) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 55040
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55039
12 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(a), *(argv).offset((i) as isize))) != 0 { 11 } else { 10 }; continue;
}
// C line 55038
13 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 55037
14 => {
let _ = { let assigned = crate::cutils_header::min_int(argc, (end_field).wrapping_sub(first_field)); n = assigned; assigned };
vm_block = 13; continue;
}
// C line 55034
15 => {
let _ = { let assigned = res; res1 = assigned; assigned };
vm_block = 14; continue;
}
// C line 55033
16 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55032
17 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 55031
18 => {
let _ = { let assigned = get_date_fields(ctx, this_val, (fields).as_mut_ptr(), is_local, (((first_field) == ((0 as i32))) as i32)); res = assigned; assigned };
vm_block = 17; continue;
}
// C line 55029
19 => {
let _ = { let assigned = ((magic) & ((15 as i32))); is_local = assigned; assigned };
vm_block = 18; continue;
}
// C line 55028
20 => {
let _ = { let assigned = (((magic).wrapping_shr(((4 as i32)) as u32)) & ((15 as i32))); end_field = assigned; assigned };
vm_block = 19; continue;
}
// C line 55027
21 => {
let _ = { let assigned = (((magic).wrapping_shr(((8 as i32)) as u32)) & ((15 as i32))); first_field = assigned; assigned };
vm_block = 20; continue;
}
// C line 55026
22 => {
let _ = { let assigned = ((f32::NAN) as f64); d = assigned; assigned };
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55176. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut rv: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut val: f64 = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut dv: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut fields: [f64; 7] = core::mem::zeroed();
let mut s: JSValue = core::mem::zeroed();
let mut vm_block: usize = 41;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55244
1 => {
return rv;
}
// C line 55242
2 => {
let _ = { let assigned = s; rv = assigned; assigned };
vm_block = 1; continue;
}
// C line 55241
3 => {
let _ = JS_FreeValue(ctx, rv);
vm_block = 2; continue;
}
// C line 55240
4 => {
let _ = { let assigned = get_date_string(ctx, rv, (0 as i32), core::ptr::null_mut::<JSValue>(), (19 as i32)); s = assigned; assigned };
vm_block = 3; continue;
}
// C line 55237
5 => {
vm_block = if ((((((!((JS_IsException(rv)) != 0) as i32)) != 0) && ((JS_IsUndefined(new_target)) != 0)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 55235
6 => {
let _ = JS_SetObjectData(ctx, rv, JS_NewFloat64(ctx, val));
vm_block = 5; continue;
}
// C line 55234
7 => {
vm_block = if ((!((JS_IsException(rv)) != 0) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line ? labels: has_val
8 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_DATE as i32)); rv = assigned; assigned };
vm_block = 7; continue;
}
// C line 55190
9 => {
let _ = { let assigned = ((date_now()) as f64); val = assigned; assigned };
vm_block = 8; continue;
}
// C line 55214
10 => {
let _ = { let assigned = time_clip(val); val = assigned; assigned };
vm_block = 8; continue;
}
// C line 55209
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55208
12 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(val), dv)) != 0 { 11 } else { 10 }; continue;
}
// C line 55207
13 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55206
14 => {
vm_block = if (JS_IsException(dv)) != 0 { 13 } else { 12 }; continue;
}
// C line 55205
15 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 14; continue;
}
// C line 55204
16 => {
let _ = { let assigned = js_Date_parse(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(v)); dv = assigned; assigned };
vm_block = 15; continue;
}
// C line 55212
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55211
18 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(val), v)) != 0 { 17 } else { 10 }; continue;
}
// C line 55203
19 => {
vm_block = if (JS_IsString(v)) != 0 { 16 } else { 18 }; continue;
}
// C line 55202
20 => {
let _ = { let assigned = JS_ToPrimitive(ctx, *(argv).offset(((0 as i32)) as isize), (2 as i32)); v = assigned; assigned };
vm_block = 19; continue;
}
// C line 55199
21 => {
vm_block = 8; continue;
}
// C line 55198
22 => {
let _ = { let assigned = time_clip(val); val = assigned; assigned };
vm_block = 21; continue;
}
// C line 55197
23 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55196
24 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(val), ((*(p)).u).object_data)) != 0 { 23 } else { 22 }; continue;
}
// C line 55195
25 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_DATE as i32))) as i32)) != 0) && ((JS_IsNumber(((*(p)).u).object_data)) != 0)) as i32)) != 0 { 24 } else { 20 }; continue;
}
// C line 55194
26 => {
p = ((((*(argv).offset(((0 as i32)) as isize)).u).ptr) as *mut JSObject);
vm_block = 25; continue;
}
// C line 55193
27 => {
vm_block = if (((((((*(argv).offset(((0 as i32)) as isize)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 26 } else { 20 }; continue;
}
// C line 55223
28 => {
let _ = { let assigned = set_date_fields_checked((fields).as_mut_ptr(), (1 as i32)); val = assigned; assigned };
vm_block = 8; continue;
}
// C line 55219
29 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 32 } else { 28 }; continue;
}
// C line ?
30 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 29; continue;
}
// C line 55221
31 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55220
32 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(*((fields).as_mut_ptr()).offset((i) as isize)), *(argv).offset((i) as isize))) != 0 { 31 } else { 30 }; continue;
}
// C line 55219
33 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 29; continue;
}
// C line 55218
34 => {
let _ = { let assigned = (7 as i32); n = assigned; assigned };
vm_block = 33; continue;
}
// C line 55217
35 => {
vm_block = if ((((n) > ((7 as i32))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 55216
36 => {
fields = [(((0 as i32)) as f64), (((0 as i32)) as f64), (((1 as i32)) as f64), (((0 as i32)) as f64), (((0 as i32)) as f64), (((0 as i32)) as f64), (((0 as i32)) as f64)];
vm_block = 35; continue;
}
// C line 55191
37 => {
vm_block = if ((((n) == ((1 as i32))) as i32)) != 0 { 27 } else { 36 }; continue;
}
// C line 55189
38 => {
vm_block = if ((((n) == ((0 as i32))) as i32)) != 0 { 9 } else { 37 }; continue;
}
// C line 55188
39 => {
let _ = { let assigned = argc; n = assigned; assigned };
vm_block = 38; continue;
}
// C line 55186
40 => {
let _ = { let assigned = (0 as i32); argc = assigned; assigned };
vm_block = 39; continue;
}
// C line 55184
41 => {
vm_block = if (JS_IsUndefined(new_target)) != 0 { 40 } else { 39 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55247. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_Date_UTC(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut fields: [f64; 7] = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55263
1 => {
return JS_NewFloat64(ctx, set_date_fields_checked((fields).as_mut_ptr(), (0 as i32)));
}
// C line 55259
2 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 55261
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55260
5 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(*((fields).as_mut_ptr()).offset((i) as isize)), *(argv).offset((i) as isize))) != 0 { 4 } else { 3 }; continue;
}
// C line 55259
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 55258
7 => {
let _ = { let assigned = (7 as i32); n = assigned; assigned };
vm_block = 6; continue;
}
// C line 55257
8 => {
vm_block = if ((((n) > ((7 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 55256
9 => {
return JSValue { u: JSValueUnion { float64: ((f32::NAN) as f64) }, tag: (((JS_TAG_FLOAT64 as i32)) as i64) };
}
// C line 55255
10 => {
vm_block = if ((((n) == ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 55254
11 => {
let _ = { let assigned = argc; n = assigned; assigned };
vm_block = 10; continue;
}
// C line 55251
12 => {
fields = [(((0 as i32)) as f64), (((0 as i32)) as f64), (((1 as i32)) as f64), (((0 as i32)) as f64), (((0 as i32)) as f64), (((0 as i32)) as f64), (((0 as i32)) as f64)];
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55268. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_skip_char(mut sp: *const u8, mut pp: *mut i32, mut c: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55271
1 => {
return (1 as i32);
}
// C line 55270
2 => {
let _ = { *(pp) = (*(pp)).wrapping_add((1 as i32)); *(pp) };
vm_block = 1; continue;
}
// C line 55273
3 => {
return (0 as i32);
}
// C line 55269
4 => {
vm_block = if ((((((*(sp).offset((*(pp)) as isize)) as i32)) == (c)) as i32)) != 0 { 2 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55278. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_skip_spaces(mut sp: *const u8, mut pp: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55282
1 => {
return c;
}
// C line 55280
2 => {
vm_block = if (((({ let assigned = ((*(sp).offset((*(pp)) as isize)) as i32); c = assigned; assigned }) == ((32 as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 55281
3 => {
let _ = { *(pp) = (*(pp)).wrapping_add((1 as i32)); *(pp) };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55286. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_skip_separators(mut sp: *const u8, mut pp: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55290
1 => {
return c;
}
// C line 55288
2 => {
vm_block = if (((((((((((((((({ let assigned = ((*(sp).offset((*(pp)) as isize)) as i32); c = assigned; assigned }) == ((45 as i32))) as i32)) != 0) || (((((c) == ((47 as i32))) as i32)) != 0)) as i32)) != 0) || (((((c) == ((46 as i32))) as i32)) != 0)) as i32)) != 0) || (((((c) == ((44 as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 55289
3 => {
let _ = { *(pp) = (*(pp)).wrapping_add((1 as i32)); *(pp) };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55294. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_skip_until(mut sp: *const u8, mut pp: *mut i32, mut stoplist: *const c_char) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55298
1 => {
return c;
}
// C line 55296
2 => {
vm_block = if ((!(!(js_date_strchr(stoplist, { let assigned = ((*(sp).offset((*(pp)) as isize)) as i32); c = assigned; assigned })).is_null()) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 55297
3 => {
let _ = { *(pp) = (*(pp)).wrapping_add((1 as i32)); *(pp) };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55302. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_get_digits(mut sp: *const u8, mut pp: *mut i32, mut pval: *mut i32, mut min_digits: i32, mut max_digits: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut p_start: i32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55322
1 => {
return (1 as i32);
}
// C line 55321
2 => {
let _ = { let assigned = p; *(pp) = assigned; assigned };
vm_block = 1; continue;
}
// C line 55320
3 => {
let _ = { let assigned = v; *(pval) = assigned; assigned };
vm_block = 2; continue;
}
// C line 55319
4 => {
return (0 as i32);
}
// C line 55318
5 => {
vm_block = if (((((p).wrapping_sub(p_start)) < (min_digits)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 55309
6 => {
vm_block = if (((((((({ let assigned = ((*(sp).offset((p) as isize)) as i32); c = assigned; assigned }) >= ((48 as i32))) as i32)) != 0) && (((((c) <= ((57 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 5 }; continue;
}
// C line 55316
7 => {
vm_block = 5; continue;
}
// C line 55315
8 => {
vm_block = if (((((p).wrapping_sub(p_start)) == (max_digits)) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 55314
9 => {
let _ = { let old = p; p = (p).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 55313
10 => {
let _ = { let assigned = (((v).wrapping_mul((10 as i32))).wrapping_add(c)).wrapping_sub((48 as i32)); v = assigned; assigned };
vm_block = 9; continue;
}
// C line 55312
11 => {
return (0 as i32);
}
// C line 55311
12 => {
vm_block = if ((((v) >= ((100000000 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 55308
13 => {
let _ = { let assigned = p; p_start = assigned; assigned };
vm_block = 6; continue;
}
// C line 55306
14 => {
p = *(pp);
vm_block = 13; continue;
}
// C line 55305
15 => {
v = (0 as i32);
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55325. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_get_milliseconds(mut sp: *const u8, mut pp: *mut i32, mut pval: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut mul: i32 = core::mem::zeroed();
let mut ms: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut p_start: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55347
1 => {
return (1 as i32);
}
// C line 55344
2 => {
let _ = { let assigned = p; *(pp) = assigned; assigned };
vm_block = 1; continue;
}
// C line 55343
3 => {
let _ = { let assigned = ms; *(pval) = assigned; assigned };
vm_block = 2; continue;
}
// C line 55341
4 => {
vm_block = if ((((p) > (p_start)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 55334
5 => {
vm_block = if (((((((({ let assigned = ((*(sp).offset((p) as isize)) as i32); c = assigned; assigned }) >= ((48 as i32))) as i32)) != 0) && (((((c) <= ((57 as i32))) as i32)) != 0)) as i32)) != 0 { 10 } else { 4 }; continue;
}
// C line 55339
6 => {
vm_block = 4; continue;
}
// C line 55338
7 => {
vm_block = if (((((p).wrapping_sub(p_start)) == ((9 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 55337
8 => {
let _ = { let old = p; p = (p).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 55336
9 => {
let _ = { mul = ((mul) / ((10 as i32))); mul };
vm_block = 8; continue;
}
// C line 55335
10 => {
let _ = { ms = (ms).wrapping_add(((c).wrapping_sub((48 as i32))).wrapping_mul(mul)); ms };
vm_block = 9; continue;
}
// C line 55333
11 => {
let _ = { let assigned = p; p_start = assigned; assigned };
vm_block = 5; continue;
}
// C line 55332
12 => {
let _ = { let old = p; p = (p).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 55331
13 => {
vm_block = if ((((((((c) == ((46 as i32))) as i32)) != 0) || (((((c) == ((44 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 1 }; continue;
}
// C line 55330
14 => {
let _ = { let assigned = ((*(sp).offset((p) as isize)) as i32); c = assigned; assigned };
vm_block = 13; continue;
}
// C line 55328
15 => {
mul = (100 as i32);
ms = (0 as i32);
p = *(pp);
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55350. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn upper_ascii(mut c: u8) -> u8 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55351
1 => {
return ((if ((((((((((c) as i32)) >= ((97 as i32))) as i32)) != 0) && (((((((c) as i32)) <= ((122 as i32))) as i32)) != 0)) as i32)) != 0 { ((((c) as i32)).wrapping_sub((97 as i32))).wrapping_add((65 as i32)) } else { ((c) as i32) }) as u8);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55354. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_get_tzoffset(mut sp: *const u8, mut pp: *mut i32, mut tzp: *mut i32, mut strict: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut tz: i32 = core::mem::zeroed();
let mut sgn: i32 = core::mem::zeroed();
let mut hh: i32 = core::mem::zeroed();
let mut mm: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut vm_block: usize = 31;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55394
1 => {
return (1 as i32);
}
// C line 55393
2 => {
let _ = { let assigned = tz; *(tzp) = assigned; assigned };
vm_block = 1; continue;
}
// C line 55392
3 => {
let _ = { let assigned = p; *(pp) = assigned; assigned };
vm_block = 2; continue;
}
// C line 55387
4 => {
let _ = { let assigned = (tz).wrapping_neg(); tz = assigned; assigned };
vm_block = 3; continue;
}
// C line 55386
5 => {
vm_block = if ((((sgn) != ((43 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 55385
6 => {
let _ = { let assigned = ((hh).wrapping_mul((60 as i32))).wrapping_add(mm); tz = assigned; assigned };
vm_block = 5; continue;
}
// C line 55384
7 => {
return (0 as i32);
}
// C line 55383
8 => {
vm_block = if ((((((((hh) > ((23 as i32))) as i32)) != 0) || (((((mm) > ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 55371
9 => {
let _ = { let assigned = ((hh) / ((100 as i32))); hh = assigned; assigned };
vm_block = 8; continue;
}
// C line 55370
10 => {
let _ = { let assigned = ((hh) % ((100 as i32))); mm = assigned; assigned };
vm_block = 9; continue;
}
// C line 55377
11 => {
return (0 as i32);
}
// C line 55376
12 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(mm), (2 as i32), (2 as i32))) != 0) as i32)) != 0 { 11 } else { 8 }; continue;
}
// C line 55380
13 => {
return (0 as i32);
}
// C line 55379
14 => {
vm_block = if (strict) != 0 { 13 } else { 8 }; continue;
}
// C line 55374
15 => {
vm_block = if (string_skip_char(sp, core::ptr::addr_of_mut!(p), (58 as i32))) != 0 { 12 } else { 14 }; continue;
}
// C line 55373
16 => {
let _ = { let assigned = (0 as i32); mm = assigned; assigned };
vm_block = 15; continue;
}
// C line 55369
17 => {
vm_block = if ((((n) > ((2 as i32))) as i32)) != 0 { 10 } else { 16 }; continue;
}
// C line 55365
18 => {
vm_block = if ((((n) > ((4 as i32))) as i32)) != 0 { 20 } else { 17 }; continue;
}
// C line 55367
19 => {
let _ = { hh = ((hh) / ((100 as i32))); hh };
vm_block = 18; continue;
}
// C line 55366
20 => {
let _ = { n = (n).wrapping_sub((2 as i32)); n };
vm_block = 19; continue;
}
// C line 55364
21 => {
return (0 as i32);
}
// C line 55363
22 => {
vm_block = if (((((((((strict) != 0) && (((((n) != ((2 as i32))) as i32)) != 0)) as i32)) != 0) && (((((n) != ((4 as i32))) as i32)) != 0)) as i32)) != 0 { 21 } else { 18 }; continue;
}
// C line 55362
23 => {
let _ = { let assigned = (p).wrapping_sub(n); n = assigned; assigned };
vm_block = 22; continue;
}
// C line 55361
24 => {
return (0 as i32);
}
// C line 55360
25 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(hh), (1 as i32), (0 as i32))) != 0) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 55359
26 => {
n = p;
vm_block = 25; continue;
}
// C line 55390
27 => {
return (0 as i32);
}
// C line 55389
28 => {
vm_block = if ((((sgn) != ((90 as i32))) as i32)) != 0 { 27 } else { 3 }; continue;
}
// C line 55358
29 => {
vm_block = if ((((((((sgn) == ((43 as i32))) as i32)) != 0) || (((((sgn) == ((45 as i32))) as i32)) != 0)) as i32)) != 0 { 26 } else { 28 }; continue;
}
// C line 55357
30 => {
let _ = { let assigned = ((*(sp).offset(({ let old = p; p = (p).wrapping_add(1); old }) as isize)) as i32); sgn = assigned; assigned };
vm_block = 29; continue;
}
// C line 55355
31 => {
tz = (0 as i32);
p = *(pp);
vm_block = 30; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55397. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_match(mut sp: *const u8, mut pp: *mut i32, mut s: *const c_char) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: i32 = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55405
1 => {
return (1 as i32);
}
// C line 55404
2 => {
let _ = { let assigned = p; *(pp) = assigned; assigned };
vm_block = 1; continue;
}
// C line 55399
3 => {
vm_block = if ((((((*(s)) as i32)) != ((0 as i32))) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 55402
4 => {
let _ = { let old = p; p = (p).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 55401
5 => {
return (0 as i32);
}
// C line 55400
6 => {
vm_block = if ((((((upper_ascii(*(sp).offset((p) as isize))) as i32)) != (((upper_ascii(((*({ let old = s; s = (s).offset(1); old })) as u8))) as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 55398
7 => {
p = *(pp);
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55408. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_abbrev(mut sp: *const u8, mut p: i32, mut list: *const c_char, mut count: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut n: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55419
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 55411
2 => {
vm_block = if ((((n) < (count)) as i32)) != 0 { 10 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 55412
4 => {
vm_block = 9; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 55416
6 => {
return n;
}
// C line 55415
7 => {
vm_block = if ((((i) == ((2 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 55414
8 => {
vm_block = 3; continue;
}
// C line 55413
9 => {
vm_block = if ((((((upper_ascii(*(sp).offset(((p).wrapping_add(i)) as isize))) as i32)) != (((upper_ascii(((*(list).offset((((n).wrapping_mul((3 as i32))).wrapping_add(i)) as isize)) as u8))) as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 55412
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 55411
11 => {
let _ = { let assigned = (0 as i32); n = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55422. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_get_month(mut sp: *const u8, mut pp: *mut i32, mut pval: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut n: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55431
1 => {
return (1 as i32);
}
// C line 55430
2 => {
let _ = { *(pp) = (*(pp)).wrapping_add((3 as i32)); *(pp) };
vm_block = 1; continue;
}
// C line 55429
3 => {
let _ = { let assigned = (n).wrapping_add((1 as i32)); *(pval) = assigned; assigned };
vm_block = 2; continue;
}
// C line 55427
4 => {
return (0 as i32);
}
// C line 55426
5 => {
vm_block = if ((((n) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 55425
6 => {
let _ = { let assigned = find_abbrev(sp, *(pp), ((month_names).as_ptr()) as *const c_char, (12 as i32)); n = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55435. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_parse_isostring(mut sp: *const u8, mut fields: *mut i32, mut is_local: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut sgn: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut vm_block: usize = 42;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55493
1 => {
return (((((*(sp).offset((p) as isize)) as i32)) == ((0 as i32))) as i32);
}
// C line 55490
2 => {
return (0 as i32);
}
// C line 55489
3 => {
vm_block = if ((!((string_get_tzoffset(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((8 as i32)) as isize)), (1 as i32))) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 55488
4 => {
let _ = { let assigned = (0 as i32); *(is_local) = assigned; assigned };
vm_block = 3; continue;
}
// C line 55487
5 => {
vm_block = if (*(sp).offset((p) as isize)) != 0 { 4 } else { 1 }; continue;
}
// C line 55483
6 => {
let _ = string_get_milliseconds(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((6 as i32)) as isize)));
vm_block = 5; continue;
}
// C line 55482
7 => {
return (0 as i32);
}
// C line 55481
8 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((5 as i32)) as isize)), (2 as i32), (2 as i32))) != 0) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 55480
9 => {
vm_block = if (string_skip_char(sp, core::ptr::addr_of_mut!(p), (58 as i32))) != 0 { 8 } else { 5 }; continue;
}
// C line 55478
10 => {
return (1 as i32);
}
// C line 55477
11 => {
let _ = { let assigned = (100 as i32); *(fields).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 55474
12 => {
vm_block = if ((((((((((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((3 as i32)) as isize)), (2 as i32), (2 as i32))) != 0) as i32)) != 0) || (((!((string_skip_char(sp, core::ptr::addr_of_mut!(p), (58 as i32))) != 0) as i32)) != 0)) as i32)) != 0) || (((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((4 as i32)) as isize)), (2 as i32), (2 as i32))) != 0) as i32)) != 0)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 55473
13 => {
let _ = { let assigned = (1 as i32); *(is_local) = assigned; assigned };
vm_block = 12; continue;
}
// C line 55472
14 => {
vm_block = if (string_skip_char(sp, core::ptr::addr_of_mut!(p), (84 as i32))) != 0 { 13 } else { 5 }; continue;
}
// C line 55469
15 => {
return (0 as i32);
}
// C line 55468
16 => {
vm_block = if ((((*(fields).offset(((2 as i32)) as isize)) < ((1 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 55467
17 => {
return (0 as i32);
}
// C line 55466
18 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((2 as i32)) as isize)), (2 as i32), (2 as i32))) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 55465
19 => {
vm_block = if (string_skip_char(sp, core::ptr::addr_of_mut!(p), (45 as i32))) != 0 { 18 } else { 14 }; continue;
}
// C line 55464
20 => {
let _ = { *(fields).offset(((1 as i32)) as isize) = (*(fields).offset(((1 as i32)) as isize)).wrapping_sub((1 as i32)); *(fields).offset(((1 as i32)) as isize) };
vm_block = 19; continue;
}
// C line 55463
21 => {
return (0 as i32);
}
// C line 55462
22 => {
vm_block = if ((((*(fields).offset(((1 as i32)) as isize)) < ((1 as i32))) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 55461
23 => {
return (0 as i32);
}
// C line 55460
24 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((1 as i32)) as isize)), (2 as i32), (2 as i32))) != 0) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 55459
25 => {
vm_block = if (string_skip_char(sp, core::ptr::addr_of_mut!(p), (45 as i32))) != 0 { 24 } else { 14 }; continue;
}
// C line 55453
26 => {
let _ = { let assigned = (*(fields).offset(((0 as i32)) as isize)).wrapping_neg(); *(fields).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 55452
27 => {
return (0 as i32);
}
// C line 55451
28 => {
vm_block = if ((((*(fields).offset(((0 as i32)) as isize)) == ((0 as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 55450
29 => {
vm_block = if ((((sgn) == ((45 as i32))) as i32)) != 0 { 28 } else { 25 }; continue;
}
// C line 55449
30 => {
return (0 as i32);
}
// C line 55448
31 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((0 as i32)) as isize)), (6 as i32), (6 as i32))) != 0) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 55447
32 => {
let _ = { let old = p; p = (p).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 55457
33 => {
return (0 as i32);
}
// C line 55456
34 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((0 as i32)) as isize)), (4 as i32), (4 as i32))) != 0) as i32)) != 0 { 33 } else { 25 }; continue;
}
// C line 55446
35 => {
vm_block = if ((((((((sgn) == ((45 as i32))) as i32)) != 0) || (((((sgn) == ((43 as i32))) as i32)) != 0)) as i32)) != 0 { 32 } else { 34 }; continue;
}
// C line 55445
36 => {
let _ = { let assigned = ((*(sp).offset((p) as isize)) as i32); sgn = assigned; assigned };
vm_block = 35; continue;
}
// C line 55442
37 => {
let _ = { let assigned = (0 as i32); *(is_local) = assigned; assigned };
vm_block = 36; continue;
}
// C line 55439
38 => {
vm_block = if ((((i) < ((9 as i32))) as i32)) != 0 { 40 } else { 37 }; continue;
}
// C line ?
39 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 38; continue;
}
// C line 55440
40 => {
let _ = { let assigned = (((i) == ((2 as i32))) as i32); *(fields).offset((i) as isize) = assigned; assigned };
vm_block = 39; continue;
}
// C line 55439
41 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 38; continue;
}
// C line 55436
42 => {
p = (0 as i32);
vm_block = 41; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55520. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_get_tzabbr(mut sp: *const u8, mut pp: *mut i32, mut offset: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: usize = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55527
1 => {
return (0 as i32);
}
// C line 55521
2 => {
vm_block = if ((((i) < ((((size_of::<[DateTzAbbreviation; 18]>() as usize)) / ((size_of::<DateTzAbbreviation>() as usize))))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 55524
4 => {
return (1 as i32);
}
// C line 55523
5 => {
let _ = { let assigned = (((*((js_tzabbr).as_ptr()).offset((i) as isize)).offset) as i32); *(offset) = assigned; assigned };
vm_block = 4; continue;
}
// C line 55522
6 => {
vm_block = if (string_match(sp, pp, ((*((js_tzabbr).as_ptr()).offset((i) as isize)).name).as_ptr())) != 0 { 5 } else { 3 }; continue;
}
// C line 55521
7 => {
i = (((0 as i32)) as usize);
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55531. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_parse_otherstring(mut sp: *const u8, mut fields: *mut i32, mut is_local: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut val: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut p_start: i32 = core::mem::zeroed();
let mut num: [i32; 3] = core::mem::zeroed();
let mut has_year: i32 = core::mem::zeroed();
let mut has_mon: i32 = core::mem::zeroed();
let mut has_time: i32 = core::mem::zeroed();
let mut num_index: i32 = core::mem::zeroed();
let mut level: i32 = core::mem::zeroed();
let mut vm_block: usize = 105;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55677
1 => {
return (1 as i32);
}
// C line 55676
2 => {
let _ = { *(fields).offset(((1 as i32)) as isize) = (*(fields).offset(((1 as i32)) as isize)).wrapping_sub((1 as i32)); *(fields).offset(((1 as i32)) as isize) };
vm_block = 1; continue;
}
// C line 55675
3 => {
return (0 as i32);
}
// C line 55674
4 => {
vm_block = if ((((((((*(fields).offset(((1 as i32)) as isize)) < ((1 as i32))) as i32)) != 0) || (((((*(fields).offset(((2 as i32)) as isize)) < ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line ?
5 => {
return (0 as i32);
}
// C line 55670
6 => {
vm_block = 4; continue;
}
// C line 55669
7 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((1 as i32)) as isize); *(fields).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 55668
8 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((0 as i32)) as isize); *(fields).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 55667
9 => {
let _ = { let assigned = ((*((num).as_mut_ptr()).offset(((2 as i32)) as isize)).wrapping_add(((((*((num).as_mut_ptr()).offset(((2 as i32)) as isize)) < ((100 as i32))) as i32)).wrapping_mul((1900 as i32)))).wrapping_add(((((*((num).as_mut_ptr()).offset(((2 as i32)) as isize)) < ((50 as i32))) as i32)).wrapping_mul((100 as i32))); *(fields).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 55665
10 => {
vm_block = 4; continue;
}
// C line 55656
11 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((1 as i32)) as isize); *(fields).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 55655
12 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((0 as i32)) as isize); *(fields).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 55660
13 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((0 as i32)) as isize); *(fields).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 55659
14 => {
let _ = { let assigned = ((*((num).as_mut_ptr()).offset(((1 as i32)) as isize)).wrapping_add(((((*((num).as_mut_ptr()).offset(((1 as i32)) as isize)) < ((100 as i32))) as i32)).wrapping_mul((1900 as i32)))).wrapping_add(((((*((num).as_mut_ptr()).offset(((1 as i32)) as isize)) < ((50 as i32))) as i32)).wrapping_mul((100 as i32))); *(fields).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 55663
15 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((1 as i32)) as isize); *(fields).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 55662
16 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((0 as i32)) as isize); *(fields).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 55658
17 => {
vm_block = if (has_mon) != 0 { 14 } else { 16 }; continue;
}
// C line 55654
18 => {
vm_block = if (has_year) != 0 { 12 } else { 17 }; continue;
}
// C line 55652
19 => {
vm_block = 4; continue;
}
// C line 55649
20 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((0 as i32)) as isize); *(fields).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 55651
21 => {
let _ = { let assigned = *((num).as_mut_ptr()).offset(((0 as i32)) as isize); *(fields).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 55648
22 => {
vm_block = if (has_mon) != 0 { 20 } else { 21 }; continue;
}
// C line 55646
23 => {
vm_block = 4; continue;
}
// C line 55645
24 => {
return (0 as i32);
}
// C line 55644
25 => {
vm_block = if ((!((has_year) != 0) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 55642
26 => {
vm_block = match num_index { x if x == (3 as i32) => 9, x if x == (2 as i32) => 18, x if x == (1 as i32) => 22, x if x == (0 as i32) => 25, _ => 5, }; continue;
}
// C line 55640
27 => {
return (0 as i32);
}
// C line 55639
28 => {
vm_block = if ((((((num_index).wrapping_add(has_year)).wrapping_add(has_mon)) > ((3 as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 55550
29 => {
vm_block = if (string_skip_spaces(sp, core::ptr::addr_of_mut!(p))) != 0 { 92 } else { 28 }; continue;
}
// C line 55637
30 => {
let _ = string_skip_separators(sp, core::ptr::addr_of_mut!(p));
vm_block = 29; continue;
}
// C line 55554
31 => {
let _ = { let assigned = (0 as i32); *(is_local) = assigned; assigned };
vm_block = 30; continue;
}
// C line 55564
32 => {
let _ = { let assigned = (1 as i32); has_year = assigned; assigned };
vm_block = 30; continue;
}
// C line 55563
33 => {
let _ = { let assigned = val; *(fields).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 32; continue;
}
// C line 55561
34 => {
let _ = { let assigned = (val).wrapping_neg(); val = assigned; assigned };
vm_block = 33; continue;
}
// C line 55560
35 => {
return (0 as i32);
}
// C line 55559
36 => {
vm_block = if ((((val) == ((0 as i32))) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 55558
37 => {
vm_block = if ((((c) == ((45 as i32))) as i32)) != 0 { 36 } else { 33 }; continue;
}
// C line 55557
38 => {
vm_block = if (string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(val), (1 as i32), (0 as i32))) != 0 { 37 } else { 30 }; continue;
}
// C line 55556
39 => {
let _ = { let old = p; p = (p).wrapping_add(1); old };
vm_block = 38; continue;
}
// C line 55553
40 => {
vm_block = if (((((has_time) != 0) && ((string_get_tzoffset(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((8 as i32)) as isize)), (0 as i32))) != 0)) as i32)) != 0 { 31 } else { 39 }; continue;
}
// C line 55582
41 => {
let _ = { let assigned = (0 as i32); *(is_local) = assigned; assigned };
vm_block = 30; continue;
}
// C line 55580
42 => {
vm_block = if ((((((((((((((*(sp).offset((p) as isize)) as i32)) == ((43 as i32))) as i32)) != 0) || (((((((*(sp).offset((p) as isize)) as i32)) == ((45 as i32))) as i32)) != 0)) as i32)) != 0) && ((string_get_tzoffset(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((8 as i32)) as isize)), (0 as i32))) != 0)) as i32)) != 0 { 41 } else { 30 }; continue;
}
// C line 55579
43 => {
let _ = { let assigned = (1 as i32); has_time = assigned; assigned };
vm_block = 42; continue;
}
// C line 55577
44 => {
let _ = string_get_milliseconds(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((6 as i32)) as isize)));
vm_block = 43; continue;
}
// C line 55576
45 => {
return (0 as i32);
}
// C line 55575
46 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((5 as i32)) as isize)), (1 as i32), (2 as i32))) != 0) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 55574
47 => {
vm_block = if (string_skip_char(sp, core::ptr::addr_of_mut!(p), (58 as i32))) != 0 { 46 } else { 43 }; continue;
}
// C line 55573
48 => {
return (0 as i32);
}
// C line 55572
49 => {
vm_block = if ((!((string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((4 as i32)) as isize)), (1 as i32), (2 as i32))) != 0) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 55571
50 => {
let _ = { let assigned = val; *(fields).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 49; continue;
}
// C line 55587
51 => {
let _ = { let assigned = (1 as i32); has_year = assigned; assigned };
vm_block = 30; continue;
}
// C line 55586
52 => {
let _ = { let assigned = val; *(fields).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 51; continue;
}
// C line 55591
53 => {
let _ = { let assigned = (1 as i32); has_year = assigned; assigned };
vm_block = 30; continue;
}
// C line 55590
54 => {
let _ = { let assigned = ((val).wrapping_add(((((val) < ((100 as i32))) as i32)).wrapping_mul((1900 as i32)))).wrapping_add(((((val) < ((50 as i32))) as i32)).wrapping_mul((100 as i32))); *(fields).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 53; continue;
}
// C line 55595
55 => {
let _ = { let assigned = val; *((num).as_mut_ptr()).offset(({ let old = num_index; num_index = (num_index).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 30; continue;
}
// C line 55594
56 => {
return (0 as i32);
}
// C line 55593
57 => {
vm_block = if ((((num_index) == ((3 as i32))) as i32)) != 0 { 56 } else { 55 }; continue;
}
// C line 55589
58 => {
vm_block = if ((((((((((((val) < ((1 as i32))) as i32)) != 0) || (((((val) > ((31 as i32))) as i32)) != 0)) as i32)) != 0) && (((!((has_year) != 0) as i32)) != 0)) as i32)) != 0 { 54 } else { 57 }; continue;
}
// C line 55585
59 => {
vm_block = if (((((((((p).wrapping_sub(p_start)) > ((2 as i32))) as i32)) != 0) && (((!((has_year) != 0) as i32)) != 0)) as i32)) != 0 { 52 } else { 58 }; continue;
}
// C line 55569
60 => {
vm_block = if (string_skip_char(sp, core::ptr::addr_of_mut!(p), (58 as i32))) != 0 { 50 } else { 59 }; continue;
}
// C line 55601
61 => {
let _ = string_skip_until(sp, core::ptr::addr_of_mut!(p), c"0123456789 -/(".as_ptr());
vm_block = 30; continue;
}
// C line 55600
62 => {
let _ = { let assigned = (1 as i32); has_mon = assigned; assigned };
vm_block = 61; continue;
}
// C line 55606
63 => {
vm_block = 29; continue;
}
// C line 55605
64 => {
let _ = { *(fields).offset(((3 as i32)) as isize) = (*(fields).offset(((3 as i32)) as isize)).wrapping_add((12 as i32)); *(fields).offset(((3 as i32)) as isize) };
vm_block = 63; continue;
}
// C line 55604
65 => {
vm_block = if ((((*(fields).offset(((3 as i32)) as isize)) < ((12 as i32))) as i32)) != 0 { 64 } else { 63 }; continue;
}
// C line 55611
66 => {
vm_block = 29; continue;
}
// C line 55610
67 => {
let _ = { *(fields).offset(((3 as i32)) as isize) = (*(fields).offset(((3 as i32)) as isize)).wrapping_sub((12 as i32)); *(fields).offset(((3 as i32)) as isize) };
vm_block = 66; continue;
}
// C line 55609
68 => {
vm_block = if ((((*(fields).offset(((3 as i32)) as isize)) == ((12 as i32))) as i32)) != 0 { 67 } else { 66 }; continue;
}
// C line 55615
69 => {
vm_block = 29; continue;
}
// C line 55614
70 => {
let _ = { let assigned = (0 as i32); *(is_local) = assigned; assigned };
vm_block = 69; continue;
}
// C line 55627
71 => {
return (0 as i32);
}
// C line 55626
72 => {
vm_block = if ((((level) > ((0 as i32))) as i32)) != 0 { 71 } else { 30 }; continue;
}
// C line 55619
73 => {
vm_block = if (((({ let assigned = ((*(sp).offset((p) as isize)) as i32); c = assigned; assigned }) != ((0 as i32))) as i32)) != 0 { 78 } else { 72 }; continue;
}
// C line 55624
74 => {
vm_block = 72; continue;
}
// C line 55623
75 => {
vm_block = if ((!((level) != 0) as i32)) != 0 { 74 } else { 73 }; continue;
}
// C line 55622
76 => {
let _ = { level = (level).wrapping_sub((((c) == ((41 as i32))) as i32)); level };
vm_block = 75; continue;
}
// C line 55621
77 => {
let _ = { level = (level).wrapping_add((((c) == ((40 as i32))) as i32)); level };
vm_block = 76; continue;
}
// C line 55620
78 => {
let _ = { let old = p; p = (p).wrapping_add(1); old };
vm_block = 77; continue;
}
// C line 55618
79 => {
level = (0 as i32);
vm_block = 73; continue;
}
// C line 55630
80 => {
return (0 as i32);
}
// C line 55635
81 => {
let _ = string_skip_until(sp, core::ptr::addr_of_mut!(p), c" -/(".as_ptr());
vm_block = 30; continue;
}
// C line 55633
82 => {
return (0 as i32);
}
// C line 55632
83 => {
vm_block = if ((((has_year).wrapping_add(has_mon)).wrapping_add(has_time)).wrapping_add(num_index)) != 0 { 82 } else { 81 }; continue;
}
// C line 55629
84 => {
vm_block = if ((((c) == ((41 as i32))) as i32)) != 0 { 80 } else { 83 }; continue;
}
// C line 55617
85 => {
vm_block = if ((((c) == ((40 as i32))) as i32)) != 0 { 79 } else { 84 }; continue;
}
// C line 55613
86 => {
vm_block = if (string_get_tzabbr(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((8 as i32)) as isize)))) != 0 { 70 } else { 85 }; continue;
}
// C line 55608
87 => {
vm_block = if (((((has_time) != 0) && ((string_match(sp, core::ptr::addr_of_mut!(p), c"AM".as_ptr())) != 0)) as i32)) != 0 { 68 } else { 86 }; continue;
}
// C line 55603
88 => {
vm_block = if (((((has_time) != 0) && ((string_match(sp, core::ptr::addr_of_mut!(p), c"PM".as_ptr())) != 0)) as i32)) != 0 { 65 } else { 87 }; continue;
}
// C line 55599
89 => {
vm_block = if (string_get_month(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(*(fields).offset(((1 as i32)) as isize)))) != 0 { 62 } else { 88 }; continue;
}
// C line 55568
90 => {
vm_block = if (string_get_digits(sp, core::ptr::addr_of_mut!(p), core::ptr::addr_of_mut!(val), (1 as i32), (0 as i32))) != 0 { 60 } else { 89 }; continue;
}
// C line 55552
91 => {
vm_block = if (((((((({ let assigned = ((*(sp).offset((p) as isize)) as i32); c = assigned; assigned }) == ((43 as i32))) as i32)) != 0) || (((((c) == ((45 as i32))) as i32)) != 0)) as i32)) != 0 { 40 } else { 90 }; continue;
}
// C line 55551
92 => {
let _ = { let assigned = p; p_start = assigned; assigned };
vm_block = 91; continue;
}
// C line 55548
93 => {
let _ = { let assigned = (1 as i32); *(is_local) = assigned; assigned };
vm_block = 29; continue;
}
// C line 55545
94 => {
vm_block = if ((((i) < ((9 as i32))) as i32)) != 0 { 96 } else { 93 }; continue;
}
// C line ?
95 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 94; continue;
}
// C line 55546
96 => {
let _ = { let assigned = (0 as i32); *(fields).offset((i) as isize) = assigned; assigned };
vm_block = 95; continue;
}
// C line 55545
97 => {
let _ = { let assigned = (3 as i32); i = assigned; assigned };
vm_block = 94; continue;
}
// C line 55544
98 => {
let _ = { let assigned = (1 as i32); *(fields).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 97; continue;
}
// C line 55543
99 => {
let _ = { let assigned = (1 as i32); *(fields).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 98; continue;
}
// C line 55542
100 => {
let _ = { let assigned = (2001 as i32); *(fields).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 99; continue;
}
// C line 55539
101 => {
num_index = (0 as i32);
vm_block = 100; continue;
}
// C line 55538
102 => {
has_time = (0 as i32);
vm_block = 101; continue;
}
// C line 55537
103 => {
has_mon = (0 as i32);
vm_block = 102; continue;
}
// C line 55536
104 => {
has_year = (0 as i32);
vm_block = 103; continue;
}
// C line 55534
105 => {
p = (0 as i32);
vm_block = 104; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55680. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_Date_parse(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: JSValue = core::mem::zeroed();
let mut rv: JSValue = core::mem::zeroed();
let mut fields: [i32; 9] = core::mem::zeroed();
let mut fields1: [f64; 9] = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut sp: *mut JSString = core::mem::zeroed();
let mut buf: [u8; 128] = core::mem::zeroed();
let mut is_local: i32 = core::mem::zeroed();
let mut field_max: [i32; 6] = core::mem::zeroed();
let mut valid: i32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55727
1 => {
return rv;
}
// C line 55726
2 => {
let _ = JS_FreeValue(ctx, s);
vm_block = 1; continue;
}
// C line 55723
3 => {
let _ = { let assigned = JS_NewFloat64(ctx, d); rv = assigned; assigned };
vm_block = 2; continue;
}
// C line 55722
4 => {
let _ = { let assigned = ((set_date_fields((fields1).as_mut_ptr(), is_local)) - ((((*((fields).as_mut_ptr()).offset(((8 as i32)) as isize)).wrapping_mul((60000 as i32))) as f64))); d = assigned; assigned };
vm_block = 3; continue;
}
// C line 55720
5 => {
vm_block = if ((((i) < ((7 as i32))) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 55721
7 => {
let _ = { let assigned = ((*((fields).as_mut_ptr()).offset((i) as isize)) as f64); *((fields1).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 55720
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 55719
9 => {
vm_block = if (valid) != 0 { 8 } else { 2 }; continue;
}
// C line 55718
10 => {
let _ = { let assigned = (0 as i32); valid = assigned; assigned };
vm_block = 9; continue;
}
// C line 55717
11 => {
vm_block = if ((((((((*((fields).as_mut_ptr()).offset(((3 as i32)) as isize)) == ((24 as i32))) as i32)) != 0) && ((((((*((fields).as_mut_ptr()).offset(((4 as i32)) as isize)) | (*((fields).as_mut_ptr()).offset(((5 as i32)) as isize)))) | (*((fields).as_mut_ptr()).offset(((6 as i32)) as isize)))) != 0)) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 55712
12 => {
vm_block = if ((((i) < ((6 as i32))) as i32)) != 0 { 15 } else { 11 }; continue;
}
// C line ?
13 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 55714
14 => {
let _ = { let assigned = (0 as i32); valid = assigned; assigned };
vm_block = 13; continue;
}
// C line 55713
15 => {
vm_block = if ((((*((fields).as_mut_ptr()).offset((i) as isize)) > (*((field_max).as_ptr()).offset((i) as isize))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 55712
16 => {
let _ = { let assigned = (1 as i32); i = assigned; assigned };
vm_block = 12; continue;
}
// C line 55710
17 => {
valid = (1 as i32);
vm_block = 16; continue;
}
// C line 55709
18 => {
field_max = [(0 as i32), (11 as i32), (31 as i32), (24 as i32), (59 as i32), (59 as i32)];
vm_block = 17; continue;
}
// C line 55707
19 => {
vm_block = if (((((js_date_parse_isostring((buf).as_mut_ptr(), (fields).as_mut_ptr(), core::ptr::addr_of_mut!(is_local))) != 0) || ((js_date_parse_otherstring((buf).as_mut_ptr(), (fields).as_mut_ptr(), core::ptr::addr_of_mut!(is_local))) != 0)) as i32)) != 0 { 18 } else { 2 }; continue;
}
// C line 55706
20 => {
let _ = { let assigned = (((0 as i32)) as u8); *((buf).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 55700
21 => {
vm_block = if ((((((((i) < ((((*(sp)).len()) as i32))) as i32)) != 0) && (((((i) < (((((((size_of::<[u8; 128]>() as usize)) / ((size_of::<u8>() as usize)))) as i32)).wrapping_sub((1 as i32)))) as i32)) != 0)) as i32)) != 0 { 26 } else { 20 }; continue;
}
// C line ?
22 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 21; continue;
}
// C line 55704
23 => {
let _ = { let assigned = ((c) as u8); *((buf).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 22; continue;
}
// C line 55703
24 => {
let _ = { let assigned = if ((((c) == ((8722 as i32))) as i32)) != 0 { (45 as i32) } else { (120 as i32) }; c = assigned; assigned };
vm_block = 23; continue;
}
// C line 55702
25 => {
vm_block = if ((((c) > ((255 as i32))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 55701
26 => {
let _ = { let assigned = string_get(sp, i); c = assigned; assigned };
vm_block = 25; continue;
}
// C line 55700
27 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 21; continue;
}
// C line 55698
28 => {
let _ = { let assigned = ((((s).u).ptr) as *mut JSString); sp = assigned; assigned };
vm_block = 27; continue;
}
// C line 55696
29 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55695
30 => {
vm_block = if (JS_IsException(s)) != 0 { 29 } else { 28 }; continue;
}
// C line 55694
31 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); s = assigned; assigned };
vm_block = 30; continue;
}
// C line 55692
32 => {
let _ = { let assigned = JSValue { u: JSValueUnion { float64: ((f32::NAN) as f64) }, tag: (((JS_TAG_FLOAT64 as i32)) as i64) }; rv = assigned; assigned };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55730. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_Date_now(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55734
1 => {
return JS_NewInt64(ctx, date_now());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55737. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_Symbol_toPrimitive(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut hint: JSAtom = core::mem::zeroed();
let mut hint_num: i32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55766
1 => {
return JS_ToPrimitive(ctx, obj, ((hint_num) | (((1 as i32)).wrapping_shl(((4 as i32)) as u32))));
}
// C line ?
2 => {
return JS_ThrowTypeError(ctx, c"invalid hint".as_ptr());
}
// C line 55762
3 => {
vm_block = 1; continue;
}
// C line 55761
4 => {
let _ = { let assigned = (0 as i32); hint_num = assigned; assigned };
vm_block = 3; continue;
}
// C line 55758
5 => {
vm_block = 1; continue;
}
// C line 55757
6 => {
let _ = { let assigned = (1 as i32); hint_num = assigned; assigned };
vm_block = 5; continue;
}
// C line 55754
7 => {
vm_block = match hint { x if x == (((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom) => 4, x if x == (((crate::quickjs_atom::JS_ATOM_string as i32)) as JSAtom) => 4, x if x == (((crate::quickjs_atom::JS_ATOM_integer as i32)) as JSAtom) => 6, x if x == (((crate::quickjs_atom::JS_ATOM_number as i32)) as JSAtom) => 6, _ => 2, }; continue;
}
// C line 55752
8 => {
let _ = JS_FreeAtom(ctx, hint);
vm_block = 7; continue;
}
// C line 55751
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55750
10 => {
vm_block = if ((((hint) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 55749
11 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(argv).offset(((0 as i32)) as isize)); hint = assigned; assigned };
vm_block = 10; continue;
}
// C line 55748
12 => {
vm_block = if (JS_IsString(*(argv).offset(((0 as i32)) as isize))) != 0 { 11 } else { 7 }; continue;
}
// C line 55746
13 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 55745
14 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 55742
15 => {
hint = (((0 as i32)) as JSAtom);
vm_block = 14; continue;
}
// C line 55741
16 => {
obj = this_val;
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55769. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_getTimezoneOffset(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: f64 = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55778
1 => {
return JSValue { u: JSValueUnion { float64: ((f32::NAN) as f64) }, tag: (((JS_TAG_FLOAT64 as i32)) as i64) };
}
// C line 55781
2 => {
return JS_NewInt64(ctx, ((getTimezoneOffset((((v).trunc()) as i64))) as i64));
}
// C line 55777
3 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((v) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (v).is_nan() as i32 } else { (v).is_nan() as i32 } }) != 0 { 1 } else { 2 }; continue;
}
// C line 55776
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55775
5 => {
vm_block = if (JS_ThisTimeValue(ctx, core::ptr::addr_of_mut!(v), this_val)) != 0 { 4 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55784. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_getTime(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: f64 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55792
1 => {
return JS_NewFloat64(ctx, v);
}
// C line 55791
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55790
3 => {
vm_block = if (JS_ThisTimeValue(ctx, core::ptr::addr_of_mut!(v), this_val)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55795. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_setTime(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: f64 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55803
1 => {
return JS_SetThisTimeValue(ctx, this_val, time_clip(v));
}
// C line 55802
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55801
3 => {
vm_block = if (((((JS_ThisTimeValue(ctx, core::ptr::addr_of_mut!(v), this_val)) != 0) || ((JS_ToFloat64(ctx, core::ptr::addr_of_mut!(v), *(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55806. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_setYear(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut y: f64 = core::mem::zeroed();
let mut args: [JSValue; 1] = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55822
1 => {
return set_date_field(ctx, this_val, (1 as i32), (args).as_mut_ptr(), (17 as i32));
}
// C line 55821
2 => {
let _ = { let assigned = JS_NewFloat64(ctx, y); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 55819
3 => {
let _ = { y = ((y) + ((((1900 as i32)) as f64))); y };
vm_block = 2; continue;
}
// C line 55818
4 => {
vm_block = if ((((((((y) >= ((((0 as i32)) as f64))) as i32)) != 0) && (((((y) < ((((100 as i32)) as f64))) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 55817
5 => {
let _ = { let assigned = (y).trunc(); y = assigned; assigned };
vm_block = 4; continue;
}
// C line 55816
6 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((y) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (y).is_finite() as i32 } else { (y).is_finite() as i32 } }) != 0 { 5 } else { 2 }; continue;
}
// C line 55815
7 => {
let _ = { let assigned = y; y = assigned; assigned };
vm_block = 6; continue;
}
// C line 55814
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55813
9 => {
vm_block = if (((((JS_ThisTimeValue(ctx, core::ptr::addr_of_mut!(y), this_val)) != 0) || ((JS_ToFloat64(ctx, core::ptr::addr_of_mut!(y), *(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 8 } else { 7 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55825. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_date_toJSON(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut tv: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut rv: JSValue = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55860
1 => {
return rv;
}
// C line 55859
2 => {
let _ = JS_FreeValue(ctx, tv);
vm_block = 1; continue;
}
// C line ? labels: done, exception
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 55855
4 => {
let _ = { let assigned = JS_CallFree(ctx, method, obj, (0 as i32), core::ptr::null_mut::<JSValue>()); rv = assigned; assigned };
vm_block = 3; continue;
}
// C line 55853
5 => {
vm_block = 3; continue;
}
// C line 55852
6 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 5; continue;
}
// C line 55851
7 => {
let _ = JS_ThrowTypeError(ctx, c"object needs toISOString method".as_ptr());
vm_block = 6; continue;
}
// C line 55850
8 => {
vm_block = if ((!((JS_IsFunction(ctx, method)) != 0) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 55849
9 => {
vm_block = 3; continue;
}
// C line 55848
10 => {
vm_block = if (JS_IsException(method)) != 0 { 9 } else { 8 }; continue;
}
// C line 55847
11 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_toISOString as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 10; continue;
}
// C line 55844
12 => {
vm_block = 3; continue;
}
// C line 55843
13 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; rv = assigned; assigned };
vm_block = 12; continue;
}
// C line 55842
14 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_finite() as i32 } else { (d).is_finite() as i32 } }) != 0) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 55841
15 => {
vm_block = 3; continue;
}
// C line 55840
16 => {
vm_block = if ((((JS_ToFloat64(ctx, core::ptr::addr_of_mut!(d), tv)) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 55839
17 => {
vm_block = if (JS_IsNumber(tv)) != 0 { 16 } else { 11 }; continue;
}
// C line 55838
18 => {
vm_block = 3; continue;
}
// C line 55837
19 => {
vm_block = if (JS_IsException(tv)) != 0 { 18 } else { 17 }; continue;
}
// C line 55836
20 => {
let _ = { let assigned = JS_ToPrimitive(ctx, obj, (1 as i32)); tv = assigned; assigned };
vm_block = 19; continue;
}
// C line 55835
21 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 20; continue;
}
// C line 55833
22 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; tv = assigned; assigned };
vm_block = 21; continue;
}
// C line 55832
23 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; rv = assigned; assigned };
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55919. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_NewDate(mut ctx: *mut JSContext, mut epoch_ms: f64) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55925
1 => {
return obj;
}
// C line 55924
2 => {
let _ = JS_SetObjectData(ctx, obj, __JS_NewFloat64(ctx, time_clip(epoch_ms)));
vm_block = 1; continue;
}
// C line 55923
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 55922
4 => {
vm_block = if (JS_IsException(obj)) != 0 { 3 } else { 2 }; continue;
}
// C line 55921
5 => {
obj = js_create_from_ctor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (JS_CLASS_DATE as i32));
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55928. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_AddIntrinsicDate(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 55942
1 => {
return (0 as i32);
}
// C line 55941
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 55940
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 55939
4 => {
vm_block = if (JS_IsException(obj)) != 0 { 3 } else { 2 }; continue;
}
// C line 55933
5 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_DATE as i32), c"Date".as_ptr(), Some(js_date_constructor), (7 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_date_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 3]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_date_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 47]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}
