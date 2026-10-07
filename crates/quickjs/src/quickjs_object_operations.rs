// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn JS_CopyDataProperties(mut ctx: *mut JSContext, mut target: JSValue, mut source: JSValue, mut excluded: JSValue, mut setprop: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut tab_atom: *mut JSPropertyEnum = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut tab_atom_count: u32 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut pexcl: *mut JSObject = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut gpn_flags: i32 = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut is_enumerable: i32 = core::mem::zeroed();
let mut em: *const JSClassExoticMethods = core::mem::zeroed();
let mut vm_block: usize = 44;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16995
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: exception
2 => {
let _ = JS_FreePropertyEnum(ctx, tab_atom, tab_atom_count);
vm_block = 1; continue;
}
// C line 16992
3 => {
return (0 as i32);
}
// C line 16991
4 => {
let _ = JS_FreePropertyEnum(ctx, tab_atom, tab_atom_count);
vm_block = 3; continue;
}
// C line 16959
5 => {
vm_block = if ((((i) < (tab_atom_count)) as i32)) != 0 { 30 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 16989
7 => {
vm_block = 2; continue;
}
// C line 16988
8 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 16984
9 => {
let _ = { let assigned = JS_SetProperty(ctx, target, (*(tab_atom).offset((i) as isize)).atom, val); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 16986
10 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, target, (*(tab_atom).offset((i) as isize)).atom, val, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 16983
11 => {
vm_block = if (setprop) != 0 { 9 } else { 10 }; continue;
}
// C line 16982
12 => {
vm_block = 2; continue;
}
// C line 16981
13 => {
vm_block = if (JS_IsException(val)) != 0 { 12 } else { 11 }; continue;
}
// C line 16980
14 => {
let _ = { let assigned = JS_GetProperty(ctx, source, (*(tab_atom).offset((i) as isize)).atom); val = assigned; assigned };
vm_block = 13; continue;
}
// C line 16978
15 => {
vm_block = 6; continue;
}
// C line 16977
16 => {
vm_block = if ((!((is_enumerable) != 0) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 16976
17 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 16; continue;
}
// C line 16975
18 => {
let _ = { let assigned = ((((((desc).flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != ((0 as i32))) as i32); is_enumerable = assigned; assigned };
vm_block = 17; continue;
}
// C line 16974
19 => {
vm_block = 6; continue;
}
// C line 16973
20 => {
vm_block = if ((!((ret) != 0) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 16972
21 => {
vm_block = 2; continue;
}
// C line 16971
22 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 16970
23 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), p, (*(tab_atom).offset((i) as isize)).atom); ret = assigned; assigned };
vm_block = 22; continue;
}
// C line 16968
24 => {
vm_block = if ((!((((gpn_flags) & (((1 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0 { 23 } else { 14 }; continue;
}
// C line 16965
25 => {
vm_block = 6; continue;
}
// C line 16964
26 => {
vm_block = 2; continue;
}
// C line 16963
27 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 16962
28 => {
vm_block = if (ret) != 0 { 27 } else { 24 }; continue;
}
// C line 16961
29 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::null_mut::<JSPropertyDescriptor>(), pexcl, (*(tab_atom).offset((i) as isize)).atom); ret = assigned; assigned };
vm_block = 28; continue;
}
// C line 16960
30 => {
vm_block = if !(pexcl).is_null() { 29 } else { 24 }; continue;
}
// C line 16959
31 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 16957
32 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16955
33 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(tab_atom), core::ptr::addr_of_mut!(tab_atom_count), p, gpn_flags)) != 0 { 32 } else { 31 }; continue;
}
// C line 16952
34 => {
let _ = { gpn_flags = ((gpn_flags) & ((!(((1 as i32)).wrapping_shl(((4 as i32)) as u32))))); gpn_flags };
vm_block = 33; continue;
}
// C line 16951
35 => {
vm_block = if ((((!(em).is_null()) && (((*(em)).get_own_property_names).is_some())) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 16948
36 => {
em = (*((*((*(ctx)).rt)).class_array).offset(((*(p)).class_id) as isize)).exotic;
vm_block = 35; continue;
}
// C line 16947
37 => {
vm_block = if ((*(p)).is_exotic()) != 0 { 36 } else { 33 }; continue;
}
// C line 16946
38 => {
let _ = { let assigned = ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((4 as i32)) as u32))); gpn_flags = assigned; assigned };
vm_block = 37; continue;
}
// C line 16944
39 => {
let _ = { let assigned = ((((source).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 38; continue;
}
// C line 16942
40 => {
let _ = { let assigned = ((((excluded).u).ptr) as *mut JSObject); pexcl = assigned; assigned };
vm_block = 39; continue;
}
// C line 16941
41 => {
vm_block = if (((((((excluded).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 16939
42 => {
return (0 as i32);
}
// C line 16938
43 => {
vm_block = if (((((((source).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 42 } else { 41 }; continue;
}
// C line 16933
44 => {
pexcl = core::ptr::null_mut::<JSObject>();
vm_block = 43; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
pub unsafe fn JS_IsArray(mut ctx: *mut JSContext, mut val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 14562
1 => {
return ((((((*(p)).class_id) as i32)) == ((JS_CLASS_ARRAY as i32))) as i32);
}
// C line 14561
2 => {
p = ((((val).u).ptr) as *mut JSObject);
vm_block = 1; continue;
}
// C line 14564
3 => {
return (0 as i32);
}
// C line 14560
4 => {
vm_block = if (((((((val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 14559
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 14558
6 => {
vm_block = if (js_resolve_proxy(ctx, core::ptr::addr_of_mut!(val), (1 as i32))) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_resolve_proxy(mut ctx: *mut JSContext, mut pval: *mut JSValue, mut throw_exception: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut depth: i32 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51207
1 => {
return (0 as i32);
}
// C line 51190
2 => {
vm_block = if (((((((*(pval)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 15 } else { 1 }; continue;
}
// C line 51205
3 => {
let _ = { let assigned = (*(s)).target; *(pval) = assigned; assigned };
vm_block = 2; continue;
}
// C line 51203
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 51202
5 => {
let _ = JS_ThrowTypeErrorRevokedProxy(ctx);
vm_block = 4; continue;
}
// C line 51201
6 => {
vm_block = if (throw_exception) != 0 { 5 } else { 4 }; continue;
}
// C line 51200
7 => {
vm_block = if ((*(s)).is_revoked) != 0 { 6 } else { 3 }; continue;
}
// C line 51199
8 => {
let _ = { let assigned = ((((*(p)).u).opaque) as *mut JSProxyData); s = assigned; assigned };
vm_block = 7; continue;
}
// C line 51197
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 51196
10 => {
let _ = JS_ThrowStackOverflow(ctx);
vm_block = 9; continue;
}
// C line 51195
11 => {
vm_block = if (throw_exception) != 0 { 10 } else { 9 }; continue;
}
// C line 51194
12 => {
vm_block = if (((({ let old = depth; depth = (depth).wrapping_add(1); old }) > ((1000 as i32))) as i32)) != 0 { 11 } else { 8 }; continue;
}
// C line 51193
13 => {
vm_block = 1; continue;
}
// C line 51192
14 => {
vm_block = if (((((((*(p)).class_id) as i32)) != ((JS_CLASS_PROXY as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 51191
15 => {
let _ = { let assigned = ((((*(pval)).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 14; continue;
}
// C line 51186
16 => {
depth = (0 as i32);
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}
