// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_obj_to_desc(mut ctx: *mut JSContext, mut d: *mut JSPropertyDescriptor, mut desc: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut getter: JSValue = core::mem::zeroed();
let mut setter: JSValue = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut prop_1: JSValue = core::mem::zeroed();
let mut prop_2: JSValue = core::mem::zeroed();
let mut vm_block: usize = 57;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39923
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39922
2 => {
let _ = JS_FreeValue(ctx, setter);
vm_block = 1; continue;
}
// C line 39921
3 => {
let _ = JS_FreeValue(ctx, getter);
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 3; continue;
}
// C line 39918
5 => {
return (0 as i32);
}
// C line 39917
6 => {
let _ = { let assigned = setter; (*(d)).setter = assigned; assigned };
vm_block = 5; continue;
}
// C line 39916
7 => {
let _ = { let assigned = getter; (*(d)).getter = assigned; assigned };
vm_block = 6; continue;
}
// C line 39915
8 => {
let _ = { let assigned = val; (*(d)).value = assigned; assigned };
vm_block = 7; continue;
}
// C line 39914
9 => {
let _ = { let assigned = flags; (*(d)).flags = assigned; assigned };
vm_block = 8; continue;
}
// C line 39912
10 => {
vm_block = 4; continue;
}
// C line 39911
11 => {
let _ = JS_ThrowTypeError(ctx, c"cannot have setter/getter and value or writable".as_ptr());
vm_block = 10; continue;
}
// C line 39909
12 => {
vm_block = if (((((((flags) & (((((1 as i32)).wrapping_shl(((12 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((11 as i32)) as u32)))))) != 0) && ((((flags) & (((((1 as i32)).wrapping_shl(((13 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((9 as i32)) as u32)))))) != 0)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 39906
13 => {
vm_block = 4; continue;
}
// C line 39905
14 => {
let _ = JS_ThrowTypeError(ctx, c"invalid setter".as_ptr());
vm_block = 13; continue;
}
// C line 39903
15 => {
vm_block = if (((((JS_IsException(setter)) != 0) || (((!((((((JS_IsUndefined(setter)) != 0) || ((JS_IsFunction(ctx, setter)) != 0)) as i32)) != 0) as i32)) != 0)) as i32)) != 0 { 14 } else { 12 }; continue;
}
// C line 39902
16 => {
let _ = { let assigned = JS_GetProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_set as i32)) as JSAtom)); setter = assigned; assigned };
vm_block = 15; continue;
}
// C line 39901
17 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32))); flags };
vm_block = 16; continue;
}
// C line 39900
18 => {
vm_block = if (JS_HasProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_set as i32)) as JSAtom))) != 0 { 17 } else { 12 }; continue;
}
// C line 39897
19 => {
vm_block = 4; continue;
}
// C line 39896
20 => {
let _ = JS_ThrowTypeError(ctx, c"invalid getter".as_ptr());
vm_block = 19; continue;
}
// C line 39894
21 => {
vm_block = if (((((JS_IsException(getter)) != 0) || (((!((((((JS_IsUndefined(getter)) != 0) || ((JS_IsFunction(ctx, getter)) != 0)) as i32)) != 0) as i32)) != 0)) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 39893
22 => {
let _ = { let assigned = JS_GetProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_get as i32)) as JSAtom)); getter = assigned; assigned };
vm_block = 21; continue;
}
// C line 39892
23 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((11 as i32)) as u32))); flags };
vm_block = 22; continue;
}
// C line 39891
24 => {
vm_block = if (JS_HasProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_get as i32)) as JSAtom))) != 0 { 23 } else { 18 }; continue;
}
// C line 39889
25 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))); flags };
vm_block = 24; continue;
}
// C line 39888
26 => {
vm_block = if (JS_ToBoolFree(ctx, prop_2)) != 0 { 25 } else { 24 }; continue;
}
// C line 39887
27 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((9 as i32)) as u32))); flags };
vm_block = 26; continue;
}
// C line 39886
28 => {
vm_block = 4; continue;
}
// C line 39885
29 => {
vm_block = if (JS_IsException(prop_2)) != 0 { 28 } else { 27 }; continue;
}
// C line 39884
30 => {
prop_2 = JS_GetProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_writable as i32)) as JSAtom));
vm_block = 29; continue;
}
// C line 39883
31 => {
vm_block = if (JS_HasProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_writable as i32)) as JSAtom))) != 0 { 30 } else { 24 }; continue;
}
// C line 39881
32 => {
vm_block = 4; continue;
}
// C line 39880
33 => {
vm_block = if (JS_IsException(val)) != 0 { 32 } else { 31 }; continue;
}
// C line 39879
34 => {
let _ = { let assigned = JS_GetProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom)); val = assigned; assigned };
vm_block = 33; continue;
}
// C line 39878
35 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((13 as i32)) as u32))); flags };
vm_block = 34; continue;
}
// C line 39877
36 => {
vm_block = if (JS_HasProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom))) != 0 { 35 } else { 31 }; continue;
}
// C line 39875
37 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32))); flags };
vm_block = 36; continue;
}
// C line 39874
38 => {
vm_block = if (JS_ToBoolFree(ctx, prop_1)) != 0 { 37 } else { 36 }; continue;
}
// C line 39873
39 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((8 as i32)) as u32))); flags };
vm_block = 38; continue;
}
// C line 39872
40 => {
vm_block = 4; continue;
}
// C line 39871
41 => {
vm_block = if (JS_IsException(prop_1)) != 0 { 40 } else { 39 }; continue;
}
// C line 39870
42 => {
prop_1 = JS_GetProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_configurable as i32)) as JSAtom));
vm_block = 41; continue;
}
// C line 39869
43 => {
vm_block = if (JS_HasProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_configurable as i32)) as JSAtom))) != 0 { 42 } else { 36 }; continue;
}
// C line 39867
44 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))); flags };
vm_block = 43; continue;
}
// C line 39866
45 => {
vm_block = if (JS_ToBoolFree(ctx, prop)) != 0 { 44 } else { 43 }; continue;
}
// C line 39865
46 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((10 as i32)) as u32))); flags };
vm_block = 45; continue;
}
// C line 39864
47 => {
vm_block = 4; continue;
}
// C line 39863
48 => {
vm_block = if (JS_IsException(prop)) != 0 { 47 } else { 46 }; continue;
}
// C line 39862
49 => {
prop = JS_GetProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_enumerable as i32)) as JSAtom));
vm_block = 48; continue;
}
// C line 39861
50 => {
vm_block = if (JS_HasProperty(ctx, desc, (((crate::quickjs_atom::JS_ATOM_enumerable as i32)) as JSAtom))) != 0 { 49 } else { 43 }; continue;
}
// C line 39860
51 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; setter = assigned; assigned };
vm_block = 50; continue;
}
// C line 39859
52 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; getter = assigned; assigned };
vm_block = 51; continue;
}
// C line 39858
53 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 52; continue;
}
// C line 39857
54 => {
let _ = { let assigned = (0 as i32); flags = assigned; assigned };
vm_block = 53; continue;
}
// C line 39855
55 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39854
56 => {
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
vm_block = 55; continue;
}
// C line 39853
57 => {
vm_block = if ((!((JS_IsObject(desc)) != 0) as i32)) != 0 { 56 } else { 54 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn JS_DefinePropertyDesc(mut ctx: *mut JSContext, mut obj: JSValue, mut prop: JSAtom, mut desc: JSValue, mut flags: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: JSPropertyDescriptor = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39939
1 => {
return ret;
}
// C line 39938
2 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(d));
vm_block = 1; continue;
}
// C line 39936
3 => {
let _ = { let assigned = JS_DefineProperty(ctx, obj, prop, (d).value, (d).getter, (d).setter, (((d).flags) | (flags))); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 39934
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39933
5 => {
vm_block = if ((((js_obj_to_desc(ctx, core::ptr::addr_of_mut!(d), desc)) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn JS_ObjectDefineProperties(mut ctx: *mut JSContext, mut obj: JSValue, mut properties: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut props: JSValue = core::mem::zeroed();
let mut desc: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut atoms: *mut JSPropertyEnum = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39978
1 => {
return ret;
}
// C line 39977
2 => {
let _ = JS_FreeValue(ctx, desc);
vm_block = 1; continue;
}
// C line 39976
3 => {
let _ = JS_FreeValue(ctx, props);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_FreePropertyEnum(ctx, atoms, len);
vm_block = 3; continue;
}
// C line 39972
5 => {
let _ = { let assigned = (0 as i32); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 39964
6 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 13 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 39970
8 => {
vm_block = 4; continue;
}
// C line 39969
9 => {
vm_block = if ((((JS_DefinePropertyDesc(ctx, obj, (*(atoms).offset((i) as isize)).atom, desc, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 39968
10 => {
vm_block = 4; continue;
}
// C line 39967
11 => {
vm_block = if (JS_IsException(desc)) != 0 { 10 } else { 9 }; continue;
}
// C line 39966
12 => {
let _ = { let assigned = JS_GetProperty(ctx, props, (*(atoms).offset((i) as isize)).atom); desc = assigned; assigned };
vm_block = 11; continue;
}
// C line 39965
13 => {
let _ = JS_FreeValue(ctx, desc);
vm_block = 12; continue;
}
// C line 39964
14 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 39963
15 => {
vm_block = 4; continue;
}
// C line 39962
16 => {
vm_block = if ((((JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(atoms), core::ptr::addr_of_mut!(len), p, ((((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 39960
17 => {
let _ = { let assigned = ((((props).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 16; continue;
}
// C line 39959
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39958
19 => {
vm_block = if (JS_IsException(props)) != 0 { 18 } else { 17 }; continue;
}
// C line 39957
20 => {
let _ = { let assigned = JS_ToObject(ctx, properties); props = assigned; assigned };
vm_block = 19; continue;
}
// C line 39956
21 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; desc = assigned; assigned };
vm_block = 20; continue;
}
// C line 39954
22 => {
return ((1 as i32)).wrapping_neg();
}
// C line 39953
23 => {
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
vm_block = 22; continue;
}
// C line 39952
24 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 23 } else { 21 }; continue;
}
// C line 39950
25 => {
ret = ((1 as i32)).wrapping_neg();
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: JSValue = core::mem::zeroed();
let mut tag: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40001
1 => {
return ret;
}
// C line 39988
2 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_OBJECT as i32)); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 39998
3 => {
vm_block = 1; continue;
}
// C line ?
4 => {
let _ = { let assigned = JS_ToObject(ctx, *(argv).offset(((0 as i32)) as isize)); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 39995
5 => {
vm_block = 1; continue;
}
// C line 39994
6 => {
let _ = { let assigned = JS_NewObject(ctx); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 39991
7 => {
vm_block = match tag { x if x == (JS_TAG_UNDEFINED as i32) => 6, x if x == (JS_TAG_NULL as i32) => 6, _ => 4, }; continue;
}
// C line 39990
8 => {
tag = (((*(argv).offset(((0 as i32)) as isize)).tag) as i32);
vm_block = 7; continue;
}
// C line 39985
9 => {
vm_block = if ((((((!((JS_IsUndefined(new_target)) != 0) as i32)) != 0) && (((((((((new_target).u).ptr) as *mut JSObject)) != (((((JS_GetActiveFunction(ctx)).u).ptr) as *mut JSObject))) as i32)) != 0)) as i32)) != 0 { 2 } else { 8 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_create(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut proto: JSValue = core::mem::zeroed();
let mut props: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40023
1 => {
return obj;
}
// C line 40020
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40019
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 40018
4 => {
vm_block = if (JS_ObjectDefineProperties(ctx, obj, props)) != 0 { 3 } else { 1 }; continue;
}
// C line 40017
5 => {
vm_block = if ((!((JS_IsUndefined(props)) != 0) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 40016
6 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); props = assigned; assigned };
vm_block = 5; continue;
}
// C line 40015
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40014
8 => {
vm_block = if (JS_IsException(obj)) != 0 { 7 } else { 6 }; continue;
}
// C line 40013
9 => {
let _ = { let assigned = JS_NewObjectProto(ctx, proto); obj = assigned; assigned };
vm_block = 8; continue;
}
// C line 40012
10 => {
return JS_ThrowTypeError(ctx, c"not a prototype".as_ptr());
}
// C line 40011
11 => {
vm_block = if ((((((!((JS_IsObject(proto)) != 0) as i32)) != 0) && (((!((JS_IsNull(proto)) != 0) as i32)) != 0)) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 40010
12 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); proto = assigned; assigned };
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_getPrototypeOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40039
1 => {
return JS_GetPrototype(ctx, val);
}
// C line 40037
2 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 40035
3 => {
vm_block = if (((((((((magic) != 0) || ((((((((val).tag) as i32)) == ((JS_TAG_NULL as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((val).tag) as i32)) == ((JS_TAG_UNDEFINED as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 40032
4 => {
vm_block = if (((((((val).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 40031
5 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); val = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_setPrototypeOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40049
1 => {
return JS_DupValue(ctx, obj);
}
// C line 40048
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40047
3 => {
vm_block = if ((((JS_SetPrototypeInternal(ctx, obj, *(argv).offset(((1 as i32)) as isize), (1 as i32))) < ((0 as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 40046
4 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_defineProperty(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut desc: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40075
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40077
2 => {
return JS_NewBool(ctx, ret);
}
// C line 40079
3 => {
return JS_DupValue(ctx, obj);
}
// C line 40076
4 => {
vm_block = if (magic) != 0 { 2 } else { 3 }; continue;
}
// C line 40074
5 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 4 }; continue;
}
// C line 40073
6 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 5; continue;
}
// C line 40072
7 => {
let _ = { let assigned = JS_DefinePropertyDesc(ctx, obj, atom, desc, flags); ret = assigned; assigned };
vm_block = 6; continue;
}
// C line 40071
8 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))); flags };
vm_block = 7; continue;
}
// C line 40070
9 => {
vm_block = if ((!((magic) != 0) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 40069
10 => {
let _ = { let assigned = (0 as i32); flags = assigned; assigned };
vm_block = 9; continue;
}
// C line 40068
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40067
12 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 11 } else { 10 }; continue;
}
// C line 40066
13 => {
let _ = { let assigned = JS_ValueToAtom(ctx, prop); atom = assigned; assigned };
vm_block = 12; continue;
}
// C line 40065
14 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 40064
15 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 40062
16 => {
let _ = { let assigned = *(argv).offset(((2 as i32)) as isize); desc = assigned; assigned };
vm_block = 15; continue;
}
// C line 40061
17 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); prop = assigned; assigned };
vm_block = 16; continue;
}
// C line 40060
18 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_defineProperties(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40090
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40092
2 => {
return JS_DupValue(ctx, obj);
}
// C line 40089
3 => {
vm_block = if (JS_ObjectDefineProperties(ctx, obj, *(argv).offset(((1 as i32)) as isize))) != 0 { 1 } else { 2 }; continue;
}
// C line 40087
4 => {
obj = *(argv).offset(((0 as i32)) as isize);
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object___defineGetter__(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut get: JSValue = core::mem::zeroed();
let mut set: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40136
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40138
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 40135
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 40134
4 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 3; continue;
}
// C line 40133
5 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 4; continue;
}
// C line 40132
6 => {
let _ = { let assigned = JS_DefineProperty(ctx, obj, atom, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, get, set, flags); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 40126
7 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32))); flags };
vm_block = 6; continue;
}
// C line 40125
8 => {
let _ = { let assigned = value; set = assigned; assigned };
vm_block = 7; continue;
}
// C line 40124
9 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; get = assigned; assigned };
vm_block = 8; continue;
}
// C line 40130
10 => {
let _ = { flags = ((flags) | (((1 as i32)).wrapping_shl(((11 as i32)) as u32))); flags };
vm_block = 6; continue;
}
// C line 40129
11 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; set = assigned; assigned };
vm_block = 10; continue;
}
// C line 40128
12 => {
let _ = { let assigned = value; get = assigned; assigned };
vm_block = 11; continue;
}
// C line 40123
13 => {
vm_block = if (magic) != 0 { 9 } else { 12 }; continue;
}
// C line 40120
14 => {
let _ = { let assigned = ((((((((((1 as i32)).wrapping_shl(((14 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((10 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32))); flags = assigned; assigned };
vm_block = 13; continue;
}
// C line 40118
15 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40117
16 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 15; continue;
}
// C line 40116
17 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 16 } else { 14 }; continue;
}
// C line 40115
18 => {
let _ = { let assigned = JS_ValueToAtom(ctx, prop); atom = assigned; assigned };
vm_block = 17; continue;
}
// C line 40113
19 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40112
20 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 19; continue;
}
// C line 40111
21 => {
vm_block = if (check_function(ctx, value)) != 0 { 20 } else { 18 }; continue;
}
// C line 40109
22 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40108
23 => {
vm_block = if (JS_IsException(obj)) != 0 { 22 } else { 21 }; continue;
}
// C line 40107
24 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 23; continue;
}
// C line 40105
25 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); value = assigned; assigned };
vm_block = 24; continue;
}
// C line 40104
26 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); prop = assigned; assigned };
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_getOwnPropertyDescriptor(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut prop: JSValue = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40203
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40202
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 2; continue;
}
// C line 40199
4 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 3; continue;
}
// C line ? labels: exception1
5 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 4; continue;
}
// C line 40195
6 => {
return ret;
}
// C line 40194
7 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 6; continue;
}
// C line 40193
8 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 7; continue;
}
// C line 40190
9 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 8; continue;
}
// C line 40189
10 => {
vm_block = 5; continue;
}
// C line 40185
11 => {
vm_block = if ((((((((JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_enumerable as i32)) as JSAtom), JS_NewBool(ctx, (((desc).flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))), flags)) < ((0 as i32))) as i32)) != 0) || (((((JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_configurable as i32)) as JSAtom), JS_NewBool(ctx, (((desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))), flags)) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 40178
12 => {
vm_block = 5; continue;
}
// C line 40176
13 => {
vm_block = if ((((((((JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_get as i32)) as JSAtom), JS_DupValue(ctx, (desc).getter), flags)) < ((0 as i32))) as i32)) != 0) || (((((JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_set as i32)) as JSAtom), JS_DupValue(ctx, (desc).setter), flags)) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 40183
14 => {
vm_block = 5; continue;
}
// C line 40180
15 => {
vm_block = if ((((((((JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom), JS_DupValue(ctx, (desc).value), flags)) < ((0 as i32))) as i32)) != 0) || (((((JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_writable as i32)) as JSAtom), JS_NewBool(ctx, (((desc).flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))), flags)) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 14 } else { 11 }; continue;
}
// C line 40175
16 => {
vm_block = if ((((desc).flags) & (((1 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0 { 13 } else { 15 }; continue;
}
// C line 40174
17 => {
let _ = { let assigned = ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))); flags = assigned; assigned };
vm_block = 16; continue;
}
// C line 40173
18 => {
vm_block = 5; continue;
}
// C line 40172
19 => {
vm_block = if (JS_IsException(ret)) != 0 { 18 } else { 17 }; continue;
}
// C line 40171
20 => {
let _ = { let assigned = JS_NewObject(ctx); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 40170
21 => {
vm_block = if (res) != 0 { 20 } else { 8 }; continue;
}
// C line 40169
22 => {
vm_block = 3; continue;
}
// C line 40168
23 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 40167
24 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), ((((obj).u).ptr) as *mut JSObject), atom); res = assigned; assigned };
vm_block = 23; continue;
}
// C line 40166
25 => {
vm_block = if (((((((obj).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 24 } else { 8 }; continue;
}
// C line 40165
26 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 25; continue;
}
// C line 40164
27 => {
vm_block = 3; continue;
}
// C line 40163
28 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 27 } else { 26 }; continue;
}
// C line 40162
29 => {
let _ = { let assigned = JS_ValueToAtom(ctx, prop); atom = assigned; assigned };
vm_block = 28; continue;
}
// C line 40161
30 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); prop = assigned; assigned };
vm_block = 29; continue;
}
// C line 40155
31 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)); obj = assigned; assigned };
vm_block = 30; continue;
}
// C line 40154
32 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 40153
33 => {
vm_block = if (((((((*(argv).offset(((0 as i32)) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 40159
34 => {
return obj;
}
// C line 40158
35 => {
vm_block = if (JS_IsException(obj)) != 0 { 34 } else { 30 }; continue;
}
// C line 40157
36 => {
let _ = { let assigned = JS_ToObject(ctx, *(argv).offset(((0 as i32)) as isize)); obj = assigned; assigned };
vm_block = 35; continue;
}
// C line 40151
37 => {
vm_block = if (magic) != 0 { 33 } else { 36 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_getOwnPropertyDescriptors(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut r: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut props: *mut JSPropertyEnum = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut atomValue: JSValue = core::mem::zeroed();
let mut desc: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40254
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40253
2 => {
let _ = JS_FreeValue(ctx, r);
vm_block = 1; continue;
}
// C line 40252
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_FreePropertyEnum(ctx, props, len);
vm_block = 3; continue;
}
// C line 40248
5 => {
return r;
}
// C line 40247
6 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 5; continue;
}
// C line 40246
7 => {
let _ = JS_FreePropertyEnum(ctx, props, len);
vm_block = 6; continue;
}
// C line 40227
8 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 21 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 40243
10 => {
vm_block = 4; continue;
}
// C line 40241
11 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, r, (*(props).offset((i) as isize)).atom, desc, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 40240
12 => {
vm_block = if ((!((JS_IsUndefined(desc)) != 0) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 40239
13 => {
vm_block = 4; continue;
}
// C line 40238
14 => {
vm_block = if (JS_IsException(desc)) != 0 { 13 } else { 12 }; continue;
}
// C line 40237
15 => {
let _ = JS_FreeValue(ctx, atomValue);
vm_block = 14; continue;
}
// C line 40236
16 => {
let _ = { let assigned = js_object_getOwnPropertyDescriptor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (2 as i32), (args).as_mut_ptr(), (0 as i32)); desc = assigned; assigned };
vm_block = 15; continue;
}
// C line 40235
17 => {
let _ = { let assigned = atomValue; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 40234
18 => {
let _ = { let assigned = obj; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 40233
19 => {
vm_block = 4; continue;
}
// C line 40232
20 => {
vm_block = if (JS_IsException(atomValue)) != 0 { 19 } else { 18 }; continue;
}
// C line 40231
21 => {
let _ = { let assigned = JS_AtomToValue(ctx, (*(props).offset((i) as isize)).atom); atomValue = assigned; assigned };
vm_block = 20; continue;
}
// C line 40227
22 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 40226
23 => {
vm_block = 4; continue;
}
// C line 40225
24 => {
vm_block = if (JS_IsException(r)) != 0 { 23 } else { 22 }; continue;
}
// C line 40224
25 => {
let _ = { let assigned = JS_NewObject(ctx); r = assigned; assigned };
vm_block = 24; continue;
}
// C line 40223
26 => {
vm_block = 4; continue;
}
// C line 40221
27 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(props), core::ptr::addr_of_mut!(len), p, ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))) != 0 { 26 } else { 25 }; continue;
}
// C line 40220
28 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 27; continue;
}
// C line 40218
29 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40217
30 => {
vm_block = if (JS_IsException(obj)) != 0 { 29 } else { 28 }; continue;
}
// C line 40216
31 => {
let _ = { let assigned = JS_ToObject(ctx, *(argv).offset(((0 as i32)) as isize)); obj = assigned; assigned };
vm_block = 30; continue;
}
// C line 40215
32 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; r = assigned; assigned };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn JS_GetOwnPropertyNames2(mut ctx: *mut JSContext, mut obj1: JSValue, mut flags: i32, mut kind: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut r: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut key: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut atoms: *mut JSPropertyEnum = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut j: u32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 56;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40333
1 => {
return r;
}
// C line 40332
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: done
3 => {
let _ = JS_FreePropertyEnum(ctx, atoms, len);
vm_block = 2; continue;
}
// C line 40329
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; r = assigned; assigned };
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_FreeValue(ctx, r);
vm_block = 4; continue;
}
// C line ? labels: exception1
6 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 5; continue;
}
// C line 40323
7 => {
vm_block = 3; continue;
}
// C line 40276
8 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 44 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 40321
10 => {
vm_block = 5; continue;
}
// C line 40320
11 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, r, (({ let old = j; j = (j).wrapping_add(1); old }) as i64), val, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 40318
12 => {
vm_block = 11; continue;
}
// C line 40317
13 => {
vm_block = 6; continue;
}
// C line 40316
14 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, val, (((1 as i32)) as i64), value, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 40315
15 => {
vm_block = 6; continue;
}
// C line 40314
16 => {
vm_block = if (JS_IsException(value)) != 0 { 15 } else { 14 }; continue;
}
// C line 40313
17 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, atom); value = assigned; assigned };
vm_block = 16; continue;
}
// C line 40312
18 => {
vm_block = 6; continue;
}
// C line 40311
19 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, val, (((0 as i32)) as i64), key, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 40310
20 => {
vm_block = 6; continue;
}
// C line 40309
21 => {
vm_block = if (JS_IsException(key)) != 0 { 20 } else { 19 }; continue;
}
// C line 40308
22 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom); key = assigned; assigned };
vm_block = 21; continue;
}
// C line 40307
23 => {
vm_block = 5; continue;
}
// C line 40306
24 => {
vm_block = if (JS_IsException(val)) != 0 { 23 } else { 22 }; continue;
}
// C line 40305
25 => {
let _ = { let assigned = JS_NewArray(ctx); val = assigned; assigned };
vm_block = 24; continue;
}
// C line 40303
26 => {
vm_block = 11; continue;
}
// C line 40302
27 => {
vm_block = 5; continue;
}
// C line 40301
28 => {
vm_block = if (JS_IsException(val)) != 0 { 27 } else { 26 }; continue;
}
// C line 40300
29 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, atom); val = assigned; assigned };
vm_block = 28; continue;
}
// C line 40298
30 => {
vm_block = 11; continue;
}
// C line 40297
31 => {
vm_block = 5; continue;
}
// C line 40296
32 => {
vm_block = if (JS_IsException(val)) != 0 { 31 } else { 30 }; continue;
}
// C line 40295
33 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom); val = assigned; assigned };
vm_block = 32; continue;
}
// C line 40292
34 => {
vm_block = match kind { x if x == (JS_ITERATOR_KIND_KEY_AND_VALUE as i32) => 25, x if x == (JS_ITERATOR_KIND_VALUE as i32) => 29, x if x == (JS_ITERATOR_KIND_KEY as i32) => 33, _ => 33, }; continue;
}
// C line 40290
35 => {
vm_block = 9; continue;
}
// C line 40289
36 => {
vm_block = if ((!(((((desc).flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 40288
37 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 36; continue;
}
// C line 40287
38 => {
vm_block = 9; continue;
}
// C line 40286
39 => {
vm_block = if ((!((res) != 0) as i32)) != 0 { 38 } else { 37 }; continue;
}
// C line 40285
40 => {
vm_block = 5; continue;
}
// C line 40284
41 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 40283
42 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), p, atom); res = assigned; assigned };
vm_block = 41; continue;
}
// C line 40278
43 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0 { 42 } else { 34 }; continue;
}
// C line 40277
44 => {
atom = (*(atoms).offset((i) as isize)).atom;
vm_block = 43; continue;
}
// C line 40276
45 => {
let _ = { let assigned = { let assigned = (((0 as i32)) as u32); i = assigned; assigned }; j = assigned; assigned };
vm_block = 8; continue;
}
// C line 40275
46 => {
vm_block = 5; continue;
}
// C line 40274
47 => {
vm_block = if (JS_IsException(r)) != 0 { 46 } else { 45 }; continue;
}
// C line 40273
48 => {
let _ = { let assigned = JS_NewArray(ctx); r = assigned; assigned };
vm_block = 47; continue;
}
// C line 40272
49 => {
vm_block = 5; continue;
}
// C line 40271
50 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(atoms), core::ptr::addr_of_mut!(len), p, ((flags) & ((!(((1 as i32)).wrapping_shl(((4 as i32)) as u32))))))) != 0 { 49 } else { 48 }; continue;
}
// C line 40270
51 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 50; continue;
}
// C line 40269
52 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40268
53 => {
vm_block = if (JS_IsException(obj)) != 0 { 52 } else { 51 }; continue;
}
// C line 40267
54 => {
let _ = { let assigned = JS_ToObject(ctx, obj1); obj = assigned; assigned };
vm_block = 53; continue;
}
// C line 40266
55 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 54; continue;
}
// C line 40265
56 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; r = assigned; assigned };
vm_block = 55; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_getOwnPropertyNames(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40339
1 => {
return JS_GetOwnPropertyNames2(ctx, *(argv).offset(((0 as i32)) as isize), ((1 as i32)).wrapping_shl(((0 as i32)) as u32), (JS_ITERATOR_KIND_KEY as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_getOwnPropertySymbols(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40346
1 => {
return JS_GetOwnPropertyNames2(ctx, *(argv).offset(((0 as i32)) as isize), ((1 as i32)).wrapping_shl(((1 as i32)) as u32), (JS_ITERATOR_KIND_KEY as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_keys(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut kind: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40353
1 => {
return JS_GetOwnPropertyNames2(ctx, *(argv).offset(((0 as i32)) as isize), ((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32))), kind);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_isExtensible(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut reflect: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40372
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40374
2 => {
return JS_NewBool(ctx, ret);
}
// C line 40371
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 40370
4 => {
let _ = { let assigned = JS_IsExtensible(ctx, obj); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 40366
5 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 40368
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 40365
7 => {
vm_block = if (reflect) != 0 { 5 } else { 6 }; continue;
}
// C line 40364
8 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 40363
9 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_preventExtensions(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut reflect: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40394
1 => {
return JS_NewBool(ctx, ret);
}
// C line 40398
2 => {
return JS_DupValue(ctx, obj);
}
// C line 40397
3 => {
return JS_ThrowTypeError(ctx, c"proxy preventExtensions handler returned false".as_ptr());
}
// C line 40396
4 => {
vm_block = if ((!((ret) != 0) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 40393
5 => {
vm_block = if (reflect) != 0 { 1 } else { 4 }; continue;
}
// C line 40392
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40391
7 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 40390
8 => {
let _ = { let assigned = JS_PreventExtensions(ctx, obj); ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 40386
9 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 40388
10 => {
return JS_DupValue(ctx, obj);
}
// C line 40385
11 => {
vm_block = if (reflect) != 0 { 9 } else { 10 }; continue;
}
// C line 40384
12 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 11 } else { 8 }; continue;
}
// C line 40383
13 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_hasOwnProperty(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40423
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40425
2 => {
return JS_NewBool(ctx, ret);
}
// C line 40422
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 40421
4 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 3; continue;
}
// C line 40420
5 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 4; continue;
}
// C line 40419
6 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::null_mut::<JSPropertyDescriptor>(), p, atom); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 40418
7 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 6; continue;
}
// C line 40416
8 => {
return obj;
}
// C line 40415
9 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 8; continue;
}
// C line 40414
10 => {
vm_block = if (JS_IsException(obj)) != 0 { 9 } else { 7 }; continue;
}
// C line 40413
11 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 10; continue;
}
// C line 40412
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40411
13 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 12 } else { 11 }; continue;
}
// C line 40410
14 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(argv).offset(((0 as i32)) as isize)); atom = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_hasOwn(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40449
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40451
2 => {
return JS_NewBool(ctx, ret);
}
// C line 40448
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 40447
4 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 3; continue;
}
// C line 40446
5 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 4; continue;
}
// C line 40445
6 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::null_mut::<JSPropertyDescriptor>(), p, atom); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 40444
7 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 6; continue;
}
// C line 40442
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40441
9 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 8; continue;
}
// C line 40440
10 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 9 } else { 7 }; continue;
}
// C line 40439
11 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(argv).offset(((1 as i32)) as isize)); atom = assigned; assigned };
vm_block = 10; continue;
}
// C line 40438
12 => {
return obj;
}
// C line 40437
13 => {
vm_block = if (JS_IsException(obj)) != 0 { 12 } else { 11 }; continue;
}
// C line 40436
14 => {
let _ = { let assigned = JS_ToObject(ctx, *(argv).offset(((0 as i32)) as isize)); obj = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_valueOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40457
1 => {
return JS_ToObject(ctx, this_val);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_toString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut tag: JSValue = core::mem::zeroed();
let mut is_array: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 29;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40512
1 => {
return JS_ConcatString3(ctx, c"[object ".as_ptr(), tag, c"]".as_ptr());
}
// C line 40469
2 => {
let _ = { let assigned = js_new_string8(ctx, c"Null".as_ptr()); tag = assigned; assigned };
vm_block = 1; continue;
}
// C line 40471
3 => {
let _ = { let assigned = js_new_string8(ctx, c"Undefined".as_ptr()); tag = assigned; assigned };
vm_block = 1; continue;
}
// C line 40509
4 => {
let _ = { let assigned = JS_AtomToString(ctx, atom); tag = assigned; assigned };
vm_block = 1; continue;
}
// C line 40508
5 => {
let _ = JS_FreeValue(ctx, tag);
vm_block = 4; continue;
}
// C line 40507
6 => {
vm_block = if ((!((JS_IsString(tag)) != 0) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 40506
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40505
8 => {
vm_block = if (JS_IsException(tag)) != 0 { 7 } else { 6 }; continue;
}
// C line 40504
9 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 8; continue;
}
// C line 40503
10 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_Symbol_toStringTag as i32)) as JSAtom)); tag = assigned; assigned };
vm_block = 9; continue;
}
// C line 40482
11 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM_Array as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 10; continue;
}
// C line 40484
12 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM_Function as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 10; continue;
}
// C line 40500
13 => {
vm_block = 10; continue;
}
// C line ?
14 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM_Object as i32)) as JSAtom); atom = assigned; assigned };
vm_block = 13; continue;
}
// C line 40497
15 => {
vm_block = 10; continue;
}
// C line 40496
16 => {
let _ = { let assigned = (*((*((*(ctx)).rt)).class_array).offset(((*(p)).class_id) as isize)).class_name; atom = assigned; assigned };
vm_block = 15; continue;
}
// C line 40487
17 => {
vm_block = match (((*(p)).class_id) as i32) { x if x == (JS_CLASS_REGEXP as i32) => 16, x if x == (JS_CLASS_DATE as i32) => 16, x if x == (JS_CLASS_NUMBER as i32) => 16, x if x == (JS_CLASS_BOOLEAN as i32) => 16, x if x == (JS_CLASS_ERROR as i32) => 16, x if x == (JS_CLASS_MAPPED_ARGUMENTS as i32) => 16, x if x == (JS_CLASS_ARGUMENTS as i32) => 16, x if x == (JS_CLASS_STRING as i32) => 16, _ => 14, }; continue;
}
// C line 40486
18 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 17; continue;
}
// C line 40483
19 => {
vm_block = if (JS_IsFunction(ctx, obj)) != 0 { 12 } else { 18 }; continue;
}
// C line 40481
20 => {
vm_block = if (is_array) != 0 { 11 } else { 19 }; continue;
}
// C line 40479
21 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40478
22 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 21; continue;
}
// C line 40477
23 => {
vm_block = if ((((is_array) < ((0 as i32))) as i32)) != 0 { 22 } else { 20 }; continue;
}
// C line 40476
24 => {
let _ = { let assigned = JS_IsArray(ctx, obj); is_array = assigned; assigned };
vm_block = 23; continue;
}
// C line 40475
25 => {
return obj;
}
// C line 40474
26 => {
vm_block = if (JS_IsException(obj)) != 0 { 25 } else { 24 }; continue;
}
// C line 40473
27 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 26; continue;
}
// C line 40470
28 => {
vm_block = if (JS_IsUndefined(this_val)) != 0 { 3 } else { 27 }; continue;
}
// C line 40468
29 => {
vm_block = if (JS_IsNull(this_val)) != 0 { 2 } else { 28 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_toLocaleString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40518
1 => {
return JS_Invoke(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_toString as i32)) as JSAtom), (0 as i32), core::ptr::null_mut::<JSValue>());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_assign(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut s: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40546
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40545
2 => {
let _ = JS_FreeValue(ctx, s);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 40542
4 => {
return obj;
}
// C line 40532
5 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 13 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 40539
7 => {
let _ = JS_FreeValue(ctx, s);
vm_block = 6; continue;
}
// C line 40538
8 => {
vm_block = 3; continue;
}
// C line 40537
9 => {
vm_block = if (JS_CopyDataProperties(ctx, obj, s, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32))) != 0 { 8 } else { 7 }; continue;
}
// C line 40536
10 => {
vm_block = 3; continue;
}
// C line 40535
11 => {
vm_block = if (JS_IsException(s)) != 0 { 10 } else { 9 }; continue;
}
// C line 40534
12 => {
let _ = { let assigned = JS_ToObject(ctx, *(argv).offset((i) as isize)); s = assigned; assigned };
vm_block = 11; continue;
}
// C line 40533
13 => {
vm_block = if ((((((!((JS_IsNull(*(argv).offset((i) as isize))) != 0) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset((i) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 12 } else { 6 }; continue;
}
// C line 40532
14 => {
let _ = { let assigned = (1 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 40531
15 => {
vm_block = 3; continue;
}
// C line 40530
16 => {
vm_block = if (JS_IsException(obj)) != 0 { 15 } else { 14 }; continue;
}
// C line 40529
17 => {
let _ = { let assigned = JS_ToObject(ctx, *(argv).offset(((0 as i32)) as isize)); obj = assigned; assigned };
vm_block = 16; continue;
}
// C line 40528
18 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; s = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_seal(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut freeze_flag: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut props: *mut JSPropertyEnum = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut desc_flags: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut prop: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 31;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40597
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreePropertyEnum(ctx, props, len);
vm_block = 1; continue;
}
// C line 40593
3 => {
return JS_DupValue(ctx, obj);
}
// C line 40592
4 => {
let _ = JS_FreePropertyEnum(ctx, props, len);
vm_block = 3; continue;
}
// C line 40573
5 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 18 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 40590
7 => {
vm_block = 2; continue;
}
// C line 40588
8 => {
vm_block = if ((((JS_DefineProperty(ctx, obj, prop, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, desc_flags)) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 40585
9 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 8; continue;
}
// C line 40584
10 => {
let _ = { desc_flags = ((desc_flags) | (((1 as i32)).wrapping_shl(((9 as i32)) as u32))); desc_flags };
vm_block = 9; continue;
}
// C line 40583
11 => {
vm_block = if ((((desc).flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0 { 10 } else { 9 }; continue;
}
// C line 40582
12 => {
vm_block = if (res) != 0 { 11 } else { 8 }; continue;
}
// C line 40581
13 => {
vm_block = 2; continue;
}
// C line 40580
14 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 40579
15 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), p, prop); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 40578
16 => {
vm_block = if (freeze_flag) != 0 { 15 } else { 8 }; continue;
}
// C line 40577
17 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl(((14 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((8 as i32)) as u32))); desc_flags = assigned; assigned };
vm_block = 16; continue;
}
// C line 40575
18 => {
prop = (*(props).offset((i) as isize)).atom;
vm_block = 17; continue;
}
// C line 40573
19 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 40571
20 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40570
21 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(props), core::ptr::addr_of_mut!(len), p, flags)) != 0 { 20 } else { 19 }; continue;
}
// C line 40569
22 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))); flags = assigned; assigned };
vm_block = 21; continue;
}
// C line 40568
23 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 22; continue;
}
// C line 40565
24 => {
return JS_ThrowTypeError(ctx, c"proxy preventExtensions handler returned false".as_ptr());
}
// C line 40564
25 => {
vm_block = if ((!((res) != 0) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 40563
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40562
27 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 40561
28 => {
let _ = { let assigned = JS_PreventExtensions(ctx, obj); res = assigned; assigned };
vm_block = 27; continue;
}
// C line 40559
29 => {
return JS_DupValue(ctx, obj);
}
// C line 40558
30 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 29 } else { 28 }; continue;
}
// C line 40552
31 => {
obj = *(argv).offset(((0 as i32)) as isize);
vm_block = 30; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_isSealed(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut is_frozen: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut props: *mut JSPropertyEnum = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut prop: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 27;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40643
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreePropertyEnum(ctx, props, len);
vm_block = 1; continue;
}
// C line 40639
3 => {
return JS_NewBool(ctx, res);
}
// C line ? labels: done
4 => {
let _ = JS_FreePropertyEnum(ctx, props, len);
vm_block = 3; continue;
}
// C line 40636
5 => {
let _ = { res = ((res) ^ ((1 as i32))); res };
vm_block = 4; continue;
}
// C line 40635
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40634
7 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 40633
8 => {
let _ = { let assigned = JS_IsExtensible(ctx, obj); res = assigned; assigned };
vm_block = 7; continue;
}
// C line 40617
9 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 19 } else { 8 }; continue;
}
// C line ?
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 40629
11 => {
vm_block = 4; continue;
}
// C line 40628
12 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 11; continue;
}
// C line 40626
13 => {
vm_block = if ((((((((desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) || ((((((is_frozen) != 0) && (((((desc).flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0)) as i32)) != 0)) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 40625
14 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 13; continue;
}
// C line 40624
15 => {
vm_block = if (res) != 0 { 14 } else { 10 }; continue;
}
// C line 40623
16 => {
vm_block = 2; continue;
}
// C line 40622
17 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 40621
18 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), p, prop); res = assigned; assigned };
vm_block = 17; continue;
}
// C line 40619
19 => {
prop = (*(props).offset((i) as isize)).atom;
vm_block = 18; continue;
}
// C line 40617
20 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 40615
21 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40614
22 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(props), core::ptr::addr_of_mut!(len), p, flags)) != 0 { 21 } else { 20 }; continue;
}
// C line 40613
23 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))); flags = assigned; assigned };
vm_block = 22; continue;
}
// C line 40612
24 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 23; continue;
}
// C line 40610
25 => {
return JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 40609
26 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 40603
27 => {
obj = *(argv).offset(((0 as i32)) as isize);
vm_block = 26; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_fromEntries(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut next_method: JSValue = core::mem::zeroed();
let mut iterable: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut key: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut vm_block: usize = 42;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40709
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40708
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 40707
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line 40706
4 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 3; continue;
}
// C line 40704
5 => {
let _ = JS_IteratorClose(ctx, iter, (1 as i32));
vm_block = 4; continue;
}
// C line 40702 labels: fail
6 => {
vm_block = if (JS_IsObject(iter)) != 0 { 5 } else { 4 }; continue;
}
// C line 40700
7 => {
return obj;
}
// C line 40699
8 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 7; continue;
}
// C line 40698
9 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 8; continue;
}
// C line 40668
10 => {
vm_block = 31; continue;
}
// C line 40696
11 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 10; continue;
}
// C line 40694
12 => {
vm_block = 6; continue;
}
// C line ? labels: fail1
13 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 12; continue;
}
// C line 40690
14 => {
vm_block = if ((((JS_DefinePropertyValueValue(ctx, obj, key, value, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 40688
15 => {
vm_block = 13; continue;
}
// C line 40687
16 => {
let _ = JS_FreeValue(ctx, key);
vm_block = 15; continue;
}
// C line 40686
17 => {
vm_block = if (JS_IsException(value)) != 0 { 16 } else { 14 }; continue;
}
// C line 40685
18 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, item, (((1 as i32)) as u32)); value = assigned; assigned };
vm_block = 17; continue;
}
// C line 40684
19 => {
vm_block = 13; continue;
}
// C line 40683
20 => {
vm_block = if (JS_IsException(key)) != 0 { 19 } else { 18 }; continue;
}
// C line 40682
21 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, item, (((0 as i32)) as u32)); key = assigned; assigned };
vm_block = 20; continue;
}
// C line 40680
22 => {
vm_block = 13; continue;
}
// C line 40679
23 => {
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
vm_block = 22; continue;
}
// C line 40678
24 => {
vm_block = if ((!((JS_IsObject(item)) != 0) as i32)) != 0 { 23 } else { 21 }; continue;
}
// C line 40677
25 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
vm_block = 24; continue;
}
// C line 40676
26 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; key = assigned; assigned };
vm_block = 25; continue;
}
// C line 40674
27 => {
vm_block = 9; continue;
}
// C line 40673
28 => {
vm_block = if (done) != 0 { 27 } else { 26 }; continue;
}
// C line 40672
29 => {
vm_block = 6; continue;
}
// C line 40671
30 => {
vm_block = if (JS_IsException(item)) != 0 { 29 } else { 28 }; continue;
}
// C line 40670
31 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next_method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 30; continue;
}
// C line 40666
32 => {
vm_block = 6; continue;
}
// C line 40665
33 => {
vm_block = if (JS_IsException(next_method)) != 0 { 32 } else { 10 }; continue;
}
// C line 40664
34 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next_method = assigned; assigned };
vm_block = 33; continue;
}
// C line 40663
35 => {
vm_block = 6; continue;
}
// C line 40662
36 => {
vm_block = if (JS_IsException(iter)) != 0 { 35 } else { 34 }; continue;
}
// C line 40661
37 => {
let _ = { let assigned = JS_GetIterator(ctx, iterable, (0 as i32)); iter = assigned; assigned };
vm_block = 36; continue;
}
// C line 40659
38 => {
return obj;
}
// C line 40658
39 => {
vm_block = if (JS_IsException(obj)) != 0 { 38 } else { 37 }; continue;
}
// C line 40657
40 => {
let _ = { let assigned = JS_NewObject(ctx); obj = assigned; assigned };
vm_block = 39; continue;
}
// C line 40655
41 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); iterable = assigned; assigned };
vm_block = 40; continue;
}
// C line 40649
42 => {
next_method = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 41; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_is(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40715
1 => {
return JS_NewBool(ctx, js_same_value(ctx, *(argv).offset(((0 as i32)) as isize), *(argv).offset(((1 as i32)) as isize)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn JS_SpeciesConstructor(mut ctx: *mut JSContext, mut obj: JSValue, mut defaultConstructor: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctor: JSValue = core::mem::zeroed();
let mut species: JSValue = core::mem::zeroed();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40745
1 => {
return species;
}
// C line 40743
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40742
3 => {
let _ = JS_FreeValue(ctx, species);
vm_block = 2; continue;
}
// C line 40741
4 => {
let _ = JS_ThrowTypeErrorNotAConstructor(ctx, species);
vm_block = 3; continue;
}
// C line 40740
5 => {
vm_block = if ((!((JS_IsConstructor(ctx, species)) != 0) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 40739
6 => {
return JS_DupValue(ctx, defaultConstructor);
}
// C line 40738
7 => {
vm_block = if (((((JS_IsUndefined(species)) != 0) || ((JS_IsNull(species)) != 0)) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 40737
8 => {
return species;
}
// C line 40736
9 => {
vm_block = if (JS_IsException(species)) != 0 { 8 } else { 7 }; continue;
}
// C line 40735
10 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 9; continue;
}
// C line 40734
11 => {
let _ = { let assigned = JS_GetProperty(ctx, ctor, (((crate::quickjs_atom::JS_ATOM_Symbol_species as i32)) as JSAtom)); species = assigned; assigned };
vm_block = 10; continue;
}
// C line 40732
12 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 40731
13 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 12; continue;
}
// C line 40730
14 => {
vm_block = if ((!((JS_IsObject(ctor)) != 0) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 40729
15 => {
return JS_DupValue(ctx, defaultConstructor);
}
// C line 40728
16 => {
vm_block = if (JS_IsUndefined(ctor)) != 0 { 15 } else { 14 }; continue;
}
// C line 40727
17 => {
return ctor;
}
// C line 40726
18 => {
vm_block = if (JS_IsException(ctor)) != 0 { 17 } else { 16 }; continue;
}
// C line 40725
19 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom)); ctor = assigned; assigned };
vm_block = 18; continue;
}
// C line 40724
20 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 40723
21 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 20 } else { 19 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_get___proto__(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40757
1 => {
return ret;
}
// C line 40756
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 40755
3 => {
let _ = { let assigned = JS_GetPrototype(ctx, val); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 40754
4 => {
return val;
}
// C line 40753
5 => {
vm_block = if (JS_IsException(val)) != 0 { 4 } else { 3 }; continue;
}
// C line 40752
6 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); val = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_set___proto__(mut ctx: *mut JSContext, mut this_val: JSValue, mut proto: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40768
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40770
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 40767
3 => {
vm_block = if ((((JS_SetPrototypeInternal(ctx, this_val, proto, (1 as i32))) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 40766
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 40765
5 => {
vm_block = if ((((((!((JS_IsObject(proto)) != 0) as i32)) != 0) && (((!((JS_IsNull(proto)) != 0) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 40764
6 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 40763
7 => {
vm_block = if (((((JS_IsUndefined(this_val)) != 0) || ((JS_IsNull(this_val)) != 0)) as i32)) != 0 { 6 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_isPrototypeOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut v1: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40810
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40809
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, v1);
vm_block = 2; continue;
}
// C line 40805
4 => {
return JS_NewBool(ctx, res);
}
// C line 40804
5 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 4; continue;
}
// C line 40803
6 => {
let _ = JS_FreeValue(ctx, v1);
vm_block = 5; continue;
}
// C line 40787
7 => {
vm_block = 18; continue;
}
// C line 40801
8 => {
vm_block = 3; continue;
}
// C line 40800
9 => {
vm_block = if (js_poll_interrupts(ctx)) != 0 { 8 } else { 7 }; continue;
}
// C line 40797
10 => {
vm_block = 6; continue;
}
// C line 40796
11 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 10; continue;
}
// C line 40795
12 => {
vm_block = if ((((((((obj).u).ptr) as *mut JSObject)) == (((((v1).u).ptr) as *mut JSObject))) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 40793
13 => {
vm_block = 6; continue;
}
// C line 40792
14 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 13; continue;
}
// C line 40791
15 => {
vm_block = if (JS_IsNull(v1)) != 0 { 14 } else { 12 }; continue;
}
// C line 40790
16 => {
vm_block = 3; continue;
}
// C line 40789
17 => {
vm_block = if (JS_IsException(v1)) != 0 { 16 } else { 15 }; continue;
}
// C line 40788
18 => {
let _ = { let assigned = JS_GetPrototypeFree(ctx, v1); v1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 40786
19 => {
let _ = { let assigned = JS_DupValue(ctx, v); v1 = assigned; assigned };
vm_block = 7; continue;
}
// C line 40785
20 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 40784
21 => {
vm_block = if (JS_IsException(obj)) != 0 { 20 } else { 19 }; continue;
}
// C line 40783
22 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 21; continue;
}
// C line 40782
23 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 40781
24 => {
vm_block = if ((!((JS_IsObject(v)) != 0) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 40780
25 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); v = assigned; assigned };
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_propertyIsEnumerable(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut prop: JSAtom = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut has_prop: i32 = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40841
1 => {
return res;
}
// C line 40840
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeAtom(ctx, prop);
vm_block = 2; continue;
}
// C line 40833
4 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 3; continue;
}
// C line 40832
5 => {
let _ = { let assigned = JS_NewBool(ctx, (((desc).flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))); res = assigned; assigned };
vm_block = 4; continue;
}
// C line 40835
6 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; res = assigned; assigned };
vm_block = 3; continue;
}
// C line 40831
7 => {
vm_block = if (has_prop) != 0 { 5 } else { 6 }; continue;
}
// C line 40830
8 => {
vm_block = 3; continue;
}
// C line 40829
9 => {
vm_block = if ((((has_prop) < ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 40828
10 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), ((((obj).u).ptr) as *mut JSObject), prop); has_prop = assigned; assigned };
vm_block = 9; continue;
}
// C line 40826
11 => {
vm_block = 3; continue;
}
// C line 40825
12 => {
vm_block = if (JS_IsException(obj)) != 0 { 11 } else { 10 }; continue;
}
// C line 40824
13 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 12; continue;
}
// C line 40823
14 => {
vm_block = 3; continue;
}
// C line 40822
15 => {
vm_block = if ((((!(((!(((((prop) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 14 } else { 13 }; continue;
}
// C line 40821
16 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(argv).offset(((0 as i32)) as isize)); prop = assigned; assigned };
vm_block = 15; continue;
}
// C line 40816
17 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
res = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object___lookupGetter__(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut setter: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut prop: JSAtom = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut has_prop: i32 = core::mem::zeroed();
let mut vm_block: usize = 29;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 40886
1 => {
return res;
}
// C line 40885
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeAtom(ctx, prop);
vm_block = 2; continue;
}
// C line 40859
4 => {
vm_block = 21; continue;
}
// C line 40880
5 => {
vm_block = 3; continue;
}
// C line 40879
6 => {
vm_block = if (js_poll_interrupts(ctx)) != 0 { 5 } else { 4 }; continue;
}
// C line 40876
7 => {
vm_block = 3; continue;
}
// C line 40875
8 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; res = assigned; assigned };
vm_block = 7; continue;
}
// C line 40874
9 => {
vm_block = if (JS_IsNull(obj)) != 0 { 8 } else { 6 }; continue;
}
// C line 40873
10 => {
vm_block = 3; continue;
}
// C line 40872
11 => {
vm_block = if (JS_IsException(obj)) != 0 { 10 } else { 9 }; continue;
}
// C line 40871
12 => {
let _ = { let assigned = JS_GetPrototypeFree(ctx, obj); obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 40869
13 => {
vm_block = 3; continue;
}
// C line 40868
14 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 13; continue;
}
// C line 40865
15 => {
let _ = { let assigned = JS_DupValue(ctx, if (setter) != 0 { (desc).setter } else { (desc).getter }); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 40867
16 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; res = assigned; assigned };
vm_block = 14; continue;
}
// C line 40864
17 => {
vm_block = if ((((desc).flags) & (((1 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0 { 15 } else { 16 }; continue;
}
// C line 40863
18 => {
vm_block = if (has_prop) != 0 { 17 } else { 12 }; continue;
}
// C line 40862
19 => {
vm_block = 3; continue;
}
// C line 40861
20 => {
vm_block = if ((((has_prop) < ((0 as i32))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 40860
21 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), ((((obj).u).ptr) as *mut JSObject), prop); has_prop = assigned; assigned };
vm_block = 20; continue;
}
// C line 40857
22 => {
vm_block = 3; continue;
}
// C line 40856
23 => {
vm_block = if ((((!(((!(((((prop) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 22 } else { 4 }; continue;
}
// C line 40855
24 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(argv).offset(((0 as i32)) as isize)); prop = assigned; assigned };
vm_block = 23; continue;
}
// C line 40854
25 => {
vm_block = 3; continue;
}
// C line 40853
26 => {
vm_block = if (JS_IsException(obj)) != 0 { 25 } else { 24 }; continue;
}
// C line 40852
27 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 26; continue;
}
// C line 40848
28 => {
prop = (((0 as i32)) as JSAtom);
vm_block = 27; continue;
}
// C line 40847
29 => {
res = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
vm_block = 28; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:?. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_object_groupBy(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut is_map: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut cb: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut groups: JSValue = core::mem::zeroed();
let mut key: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut key_atom: JSAtom = core::mem::zeroed();
let mut idx: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 83;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52229
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52228
2 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 1; continue;
}
// C line 52227
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line 52226
4 => {
let _ = JS_FreeValue(ctx, groups);
vm_block = 3; continue;
}
// C line 52225
5 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 4; continue;
}
// C line 52224
6 => {
let _ = JS_FreeValue(ctx, key);
vm_block = 5; continue;
}
// C line 52223
7 => {
let _ = JS_FreeValue(ctx, prop);
vm_block = 6; continue;
}
// C line ? labels: exception
8 => {
let _ = JS_FreeAtom(ctx, key_atom);
vm_block = 7; continue;
}
// C line ? labels: iterator_close_exception
9 => {
let _ = JS_IteratorClose(ctx, iter, (1 as i32));
vm_block = 8; continue;
}
// C line 52217
10 => {
return groups;
}
// C line 52216
11 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 10; continue;
}
// C line 52215
12 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 11; continue;
}
// C line 52151
13 => {
vm_block = 62; continue;
}
// C line ?
14 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 13; continue;
}
// C line 52212
15 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; v = assigned; assigned };
vm_block = 14; continue;
}
// C line 52211
16 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); key_atom = assigned; assigned };
vm_block = 15; continue;
}
// C line 52210
17 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; key = assigned; assigned };
vm_block = 16; continue;
}
// C line 52209
18 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; prop = assigned; assigned };
vm_block = 17; continue;
}
// C line 52208
19 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 18; continue;
}
// C line 52207
20 => {
let _ = JS_FreeAtom(ctx, key_atom);
vm_block = 19; continue;
}
// C line 52206
21 => {
let _ = JS_FreeValue(ctx, key);
vm_block = 20; continue;
}
// C line 52205
22 => {
let _ = JS_FreeValue(ctx, prop);
vm_block = 21; continue;
}
// C line 52202
23 => {
vm_block = 8; continue;
}
// C line 52201
24 => {
vm_block = if (JS_IsException(res)) != 0 { 23 } else { 22 }; continue;
}
// C line 52200
25 => {
let _ = { let assigned = js_array_push(ctx, prop, (1 as i32), core::ptr::addr_of_mut!(v), (0 as i32)); res = assigned; assigned };
vm_block = 24; continue;
}
// C line 52191
26 => {
let _ = JS_FreeValue(ctx, res);
vm_block = 25; continue;
}
// C line 52190
27 => {
vm_block = 8; continue;
}
// C line 52189
28 => {
vm_block = if (JS_IsException(res)) != 0 { 27 } else { 26 }; continue;
}
// C line 52188
29 => {
let _ = { let assigned = js_map_set(ctx, groups, (2 as i32), (args).as_mut_ptr(), (0 as i32)); res = assigned; assigned };
vm_block = 28; continue;
}
// C line 52187
30 => {
let _ = { let assigned = prop; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 29; continue;
}
// C line 52186
31 => {
let _ = { let assigned = key; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 30; continue;
}
// C line 52196
32 => {
vm_block = 8; continue;
}
// C line 52194
33 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, groups, key_atom, prop, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 32 } else { 25 }; continue;
}
// C line 52193
34 => {
let _ = { let assigned = JS_DupValue(ctx, prop); prop = assigned; assigned };
vm_block = 33; continue;
}
// C line 52185
35 => {
vm_block = if (is_map) != 0 { 31 } else { 34 }; continue;
}
// C line 52184
36 => {
vm_block = 8; continue;
}
// C line 52183
37 => {
vm_block = if (JS_IsException(prop)) != 0 { 36 } else { 35 }; continue;
}
// C line 52182
38 => {
let _ = { let assigned = JS_NewArray(ctx); prop = assigned; assigned };
vm_block = 37; continue;
}
// C line 52181
39 => {
vm_block = if (JS_IsUndefined(prop)) != 0 { 38 } else { 25 }; continue;
}
// C line 52179
40 => {
vm_block = 8; continue;
}
// C line 52178
41 => {
vm_block = if (JS_IsException(prop)) != 0 { 40 } else { 39 }; continue;
}
// C line 52169
42 => {
let _ = { let assigned = js_map_get(ctx, groups, (1 as i32), core::ptr::addr_of_mut!(key), (0 as i32)); prop = assigned; assigned };
vm_block = 41; continue;
}
// C line 52176
43 => {
let _ = { let assigned = JS_GetProperty(ctx, groups, key_atom); prop = assigned; assigned };
vm_block = 41; continue;
}
// C line 52175
44 => {
vm_block = 9; continue;
}
// C line 52174
45 => {
vm_block = if ((((key_atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 52173
46 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; key = assigned; assigned };
vm_block = 45; continue;
}
// C line 52172
47 => {
let _ = JS_FreeValue(ctx, key);
vm_block = 46; continue;
}
// C line 52171
48 => {
let _ = { let assigned = JS_ValueToAtom(ctx, key); key_atom = assigned; assigned };
vm_block = 47; continue;
}
// C line 52168
49 => {
vm_block = if (is_map) != 0 { 42 } else { 48 }; continue;
}
// C line 52166
50 => {
vm_block = 9; continue;
}
// C line 52165
51 => {
vm_block = if (JS_IsException(key)) != 0 { 50 } else { 49 }; continue;
}
// C line 52164
52 => {
let _ = { let assigned = JS_Call(ctx, cb, (*(ctx)).global_obj, (2 as i32), (args).as_mut_ptr()); key = assigned; assigned };
vm_block = 51; continue;
}
// C line 52163
53 => {
let _ = { let assigned = JS_NewInt64(ctx, idx); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 52; continue;
}
// C line 52162
54 => {
let _ = { let assigned = v; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 53; continue;
}
// C line 52160
55 => {
vm_block = 12; continue;
}
// C line 52159
56 => {
vm_block = if (done) != 0 { 55 } else { 54 }; continue;
}
// C line 52158
57 => {
vm_block = 8; continue;
}
// C line 52157
58 => {
vm_block = if (JS_IsException(v)) != 0 { 57 } else { 56 }; continue;
}
// C line 52156
59 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); v = assigned; assigned };
vm_block = 58; continue;
}
// C line 52154
60 => {
vm_block = 9; continue;
}
// C line 52153
61 => {
let _ = JS_ThrowTypeError(ctx, c"too many elements".as_ptr());
vm_block = 60; continue;
}
// C line 52152
62 => {
vm_block = if ((((idx) >= ((((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 61 } else { 59 }; continue;
}
// C line 52151
63 => {
let _ = { let assigned = (((0 as i32)) as i64); idx = assigned; assigned };
vm_block = 13; continue;
}
// C line 52149
64 => {
vm_block = 8; continue;
}
// C line 52148
65 => {
vm_block = if (JS_IsException(groups)) != 0 { 64 } else { 63 }; continue;
}
// C line 52144
66 => {
let _ = { let assigned = js_map_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>(), (0 as i32)); groups = assigned; assigned };
vm_block = 65; continue;
}
// C line 52146
67 => {
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); groups = assigned; assigned };
vm_block = 65; continue;
}
// C line 52143
68 => {
vm_block = if (is_map) != 0 { 66 } else { 67 }; continue;
}
// C line 52141
69 => {
vm_block = 8; continue;
}
// C line 52140
70 => {
vm_block = if (JS_IsException(next)) != 0 { 69 } else { 68 }; continue;
}
// C line 52139
71 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 70; continue;
}
// C line 52137
72 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; groups = assigned; assigned };
vm_block = 71; continue;
}
// C line 52136
73 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; prop = assigned; assigned };
vm_block = 72; continue;
}
// C line 52135
74 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; v = assigned; assigned };
vm_block = 73; continue;
}
// C line 52134
75 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); key_atom = assigned; assigned };
vm_block = 74; continue;
}
// C line 52133
76 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; key = assigned; assigned };
vm_block = 75; continue;
}
// C line 52131
77 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52130
78 => {
vm_block = if (JS_IsException(iter)) != 0 { 77 } else { 76 }; continue;
}
// C line 52129
79 => {
let _ = { let assigned = JS_GetIterator(ctx, *(argv).offset(((0 as i32)) as isize), (0 as i32)); iter = assigned; assigned };
vm_block = 78; continue;
}
// C line 52127
80 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52126
81 => {
vm_block = if (check_function(ctx, cb)) != 0 { 80 } else { 79 }; continue;
}
// C line 52125
82 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); cb = assigned; assigned };
vm_block = 81; continue;
}
// C line 52120
83 => {
key_atom = (((0 as i32)) as JSAtom);
vm_block = 82; continue;
}
_ => std::process::abort(),
} }
}
