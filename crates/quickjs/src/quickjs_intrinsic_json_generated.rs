// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49092. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_parse_expect(mut s: *mut JSParseState, mut tok: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49098
1 => {
return json_next_token(s);
}
// C line 49096
2 => {
return js_parse_error_cargs(s, c"expecting '%c'".as_ptr(), &[ParserFormatArg::Char((tok) as u8)]);
}
// C line 49094
3 => {
vm_block = if ((((((*(s)).token).val) != (tok)) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49130. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_parse_record_init_obj(mut ctx: *mut JSContext, mut pr: *mut JSONParseRecord, mut val: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 49136
1 => {
let _ = { let assigned = (((0 as i32)) as u32); (((*(pr)).u).obj).hash_size = assigned; assigned };
vm_block = 0; continue;
}
// C line 49135
2 => {
let _ = { let assigned = core::ptr::null_mut::<u32>(); (((*(pr)).u).obj).hash_table = assigned; assigned };
vm_block = 1; continue;
}
// C line 49134
3 => {
let _ = { let assigned = core::ptr::null_mut::<JSONParseRecordEntry>(); (((*(pr)).u).obj).entries = assigned; assigned };
vm_block = 2; continue;
}
// C line 49133
4 => {
let _ = { let assigned = (0 as i32); (((*(pr)).u).obj).count = assigned; assigned };
vm_block = 3; continue;
}
// C line 49132
5 => {
let _ = { let assigned = JS_DupValue(ctx, val); (*(pr)).value = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49139. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_parse_record_init_array(mut ctx: *mut JSContext, mut pr: *mut JSONParseRecord, mut val: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 49143
1 => {
let _ = { let assigned = core::ptr::null_mut::<JSONParseRecord>(); (((*(pr)).u).array).elements = assigned; assigned };
vm_block = 0; continue;
}
// C line 49142
2 => {
let _ = { let assigned = (0 as i32); (((*(pr)).u).array).count = assigned; assigned };
vm_block = 1; continue;
}
// C line 49141
3 => {
let _ = { let assigned = JS_DupValue(ctx, val); (*(pr)).value = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49146. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_parse_record_init_primitive(mut ctx: *mut JSContext, mut pr: *mut JSONParseRecord, mut val: JSValue, mut source_pos: u32, mut source_len: u32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 49151
1 => {
let _ = { let assigned = source_len; (((*(pr)).u).primitive).source_len = assigned; assigned };
vm_block = 0; continue;
}
// C line 49150
2 => {
let _ = { let assigned = source_pos; (((*(pr)).u).primitive).source_pos = assigned; assigned };
vm_block = 1; continue;
}
// C line 49149
3 => {
let _ = { let assigned = JS_DupValue(ctx, val); (*(pr)).value = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49154. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_parse_record_resize_hash(mut ctx: *mut JSContext, mut po: *mut JSONParseRecordObject, mut new_hash_size: u32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: u32 = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut new_hash_table: *mut u32 = core::mem::zeroed();
let mut e: *mut JSONParseRecordEntry = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49175
1 => {
return (0 as i32);
}
// C line 49169
2 => {
vm_block = if ((((i) < ((((*(po)).count) as u32))) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line 49169
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 49173
4 => {
let _ = { let assigned = i; *((*(po)).hash_table).offset((h) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 49172
5 => {
let _ = { let assigned = *((*(po)).hash_table).offset((h) as isize); (*(e)).hash_next = assigned; assigned };
vm_block = 4; continue;
}
// C line 49171
6 => {
let _ = { let assigned = (((*(e)).atom) & (((*(po)).hash_size).wrapping_sub((((1 as i32)) as u32)))); h = assigned; assigned };
vm_block = 5; continue;
}
// C line 49170
7 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(po)).entries).offset((i) as isize)); e = assigned; assigned };
vm_block = 6; continue;
}
// C line 49169
8 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 49166
9 => {
vm_block = if ((((i) < ((*(po)).hash_size)) as i32)) != 0 { 11 } else { 8 }; continue;
}
// C line 49166
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 49167
11 => {
let _ = { let assigned = ((((1 as i32)).wrapping_neg()) as u32); *((*(po)).hash_table).offset((i) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 49166
12 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 49164
13 => {
let _ = { let assigned = new_hash_size; (*(po)).hash_size = assigned; assigned };
vm_block = 12; continue;
}
// C line 49163
14 => {
let _ = { let assigned = new_hash_table; (*(po)).hash_table = assigned; assigned };
vm_block = 13; continue;
}
// C line 49162
15 => {
let _ = js_free(ctx, (((*(po)).hash_table) as *mut c_void));
vm_block = 14; continue;
}
// C line 49161
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49160
17 => {
vm_block = if ((!(!(new_hash_table).is_null()) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 49159
18 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<u32>() as usize)).wrapping_mul(((new_hash_size) as usize)))) as *mut u32); new_hash_table = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49178. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_parse_record_add(mut ctx: *mut JSContext, mut pr: *mut JSONParseRecord, mut key: JSAtom, mut psize: *mut i32) -> *mut JSONParseRecord {
let mut vm_local_storage = Vec::<u64>::new();
let mut po: *mut JSONParseRecordObject = core::mem::zeroed();
let mut e: *mut JSONParseRecordEntry = core::mem::zeroed();
let mut pr1: *mut JSONParseRecord = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut hash_bits: i32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49205
1 => {
return pr1;
}
// C line 49203
2 => {
let _ = { let assigned = ((((*(po)).count).wrapping_sub((1 as i32))) as u32); *((*(po)).hash_table).offset((h) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 49202
3 => {
let _ = { let assigned = *((*(po)).hash_table).offset((h) as isize); (*(e)).hash_next = assigned; assigned };
vm_block = 2; continue;
}
// C line 49201
4 => {
let _ = { let assigned = ((key) & (((*(po)).hash_size).wrapping_sub((((1 as i32)) as u32)))); h = assigned; assigned };
vm_block = 3; continue;
}
// C line 49200
5 => {
vm_block = if (((((*(po)).hash_size) != ((((0 as i32)) as u32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 49199
6 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(pr1)).value = assigned; assigned };
vm_block = 5; continue;
}
// C line 49198
7 => {
let _ = { let assigned = core::ptr::addr_of_mut!((*(e)).parse_record); pr1 = assigned; assigned };
vm_block = 6; continue;
}
// C line 49197
8 => {
let _ = { let assigned = JS_DupAtom(ctx, key); (*(e)).atom = assigned; assigned };
vm_block = 7; continue;
}
// C line 49196
9 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(po)).entries).offset(({ let old = (*(po)).count; (*(po)).count = ((*(po)).count).wrapping_add(1); old }) as isize)); e = assigned; assigned };
vm_block = 8; continue;
}
// C line 49193
10 => {
return core::ptr::null_mut::<JSONParseRecord>();
}
// C line 49192
11 => {
vm_block = if (json_parse_record_resize_hash(ctx, po, ((((1 as i32)).wrapping_shl((hash_bits) as u32)) as u32))) != 0 { 10 } else { 9 }; continue;
}
// C line 49191
12 => {
hash_bits = ((32 as i32)).wrapping_sub(clz32((((*(po)).count) as u32)));
vm_block = 11; continue;
}
// C line 49190
13 => {
vm_block = if (((((((((*(po)).count) >= ((8 as i32))) as i32)) != 0) && (((((((((*(po)).count).wrapping_add((1 as i32))) as u32)) > ((*(po)).hash_size)) as i32)) != 0)) as i32)) != 0 { 12 } else { 9 }; continue;
}
// C line 49187
14 => {
return core::ptr::null_mut::<JSONParseRecord>();
}
// C line 49185
15 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(po)).entries)) as *mut *mut c_void), (((size_of::<JSONParseRecordEntry>() as usize)) as i32), psize, ((*(po)).count).wrapping_add((1 as i32)))) != 0 { 14 } else { 13 }; continue;
}
// C line 49180
16 => {
po = core::ptr::addr_of_mut!(((*(pr)).u).obj);
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49208. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_parse_record_find(mut pr: *mut JSONParseRecord, mut key: JSAtom) -> *mut JSONParseRecord {
let mut vm_local_storage = Vec::<u64>::new();
let mut po: *mut JSONParseRecordObject = core::mem::zeroed();
let mut e: *mut JSONParseRecordEntry = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49229
1 => {
return core::ptr::null_mut::<JSONParseRecord>();
}
// C line 49215
2 => {
vm_block = if ((((i) < ((((*(po)).count) as u32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 49215
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 49217
4 => {
return core::ptr::addr_of_mut!((*((*(po)).entries).offset((i) as isize)).parse_record);
}
// C line 49216
5 => {
vm_block = if (((((*((*(po)).entries).offset((i) as isize)).atom) == (key)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 49215
6 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 49222
7 => {
vm_block = if ((((i) != (((((1 as i32)).wrapping_neg()) as u32))) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line 49226
8 => {
let _ = { let assigned = (*(e)).hash_next; i = assigned; assigned };
vm_block = 7; continue;
}
// C line 49225
9 => {
return core::ptr::addr_of_mut!((*(e)).parse_record);
}
// C line 49224
10 => {
vm_block = if (((((*(e)).atom) == (key)) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 49223
11 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(po)).entries).offset((i) as isize)); e = assigned; assigned };
vm_block = 10; continue;
}
// C line 49221
12 => {
let _ = { let assigned = *((*(po)).hash_table).offset((h) as isize); i = assigned; assigned };
vm_block = 7; continue;
}
// C line 49220
13 => {
let _ = { let assigned = ((key) & (((*(po)).hash_size).wrapping_sub((((1 as i32)) as u32)))); h = assigned; assigned };
vm_block = 12; continue;
}
// C line 49214
14 => {
vm_block = if (((((*(po)).hash_size) == ((((0 as i32)) as u32))) as i32)) != 0 { 6 } else { 13 }; continue;
}
// C line 49210
15 => {
po = core::ptr::addr_of_mut!(((*(pr)).u).obj);
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49232. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_free_parse_record(mut ctx: *mut JSContext, mut pr: *mut JSONParseRecord) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 49253
1 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(pr)).value = assigned; assigned };
vm_block = 0; continue;
}
// C line 49252
2 => {
let _ = JS_FreeValue(ctx, (*(pr)).value);
vm_block = 1; continue;
}
// C line 49242
3 => {
let _ = js_free(ctx, (((((*(pr)).u).array).elements) as *mut c_void));
vm_block = 2; continue;
}
// C line 49239
4 => {
vm_block = if ((((i) < ((((*(pr)).u).array).count)) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line 49239
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 49240
6 => {
let _ = json_free_parse_record(ctx, core::ptr::addr_of_mut!(*((((*(pr)).u).array).elements).offset((i) as isize)));
vm_block = 5; continue;
}
// C line 49239
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 49249
8 => {
let _ = js_free(ctx, (((((*(pr)).u).obj).hash_table) as *mut c_void));
vm_block = 2; continue;
}
// C line 49248
9 => {
let _ = js_free(ctx, (((((*(pr)).u).obj).entries) as *mut c_void));
vm_block = 8; continue;
}
// C line 49244
10 => {
vm_block = if ((((i) < ((((*(pr)).u).obj).count)) as i32)) != 0 { 13 } else { 9 }; continue;
}
// C line 49244
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 49246
12 => {
let _ = json_free_parse_record(ctx, core::ptr::addr_of_mut!((*((((*(pr)).u).obj).entries).offset((i) as isize)).parse_record));
vm_block = 11; continue;
}
// C line 49245
13 => {
let _ = JS_FreeAtom(ctx, (*((((*(pr)).u).obj).entries).offset((i) as isize)).atom);
vm_block = 12; continue;
}
// C line 49244
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 49238
15 => {
vm_block = if (JS_IsArray(ctx, (*(pr)).value)) != 0 { 7 } else { 14 }; continue;
}
// C line 49237
16 => {
vm_block = if (JS_IsObject((*(pr)).value)) != 0 { 15 } else { 2 }; continue;
}
// C line 49236
17 => {
return;
}
// C line 49235
18 => {
vm_block = if ((!(!(pr).is_null()) as i32)) != 0 { 17 } else { 16 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49257. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn json_parse_value(mut s: *mut JSParseState, mut pr: *mut JSONParseRecord) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut prop_val: JSValue = core::mem::zeroed();
let mut prop_name: JSAtom = core::mem::zeroed();
let mut pr1: *mut JSONParseRecord = core::mem::zeroed();
let mut pr_size: i32 = core::mem::zeroed();
let mut el: JSValue = core::mem::zeroed();
let mut idx: u32 = core::mem::zeroed();
let mut pr1_1: *mut JSONParseRecord = core::mem::zeroed();
let mut pr_size_1: i32 = core::mem::zeroed();
let mut vm_block: usize = 118;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49440
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 49439
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 49438 labels: fail
3 => {
let _ = json_free_parse_record(ctx, pr);
vm_block = 2; continue;
}
// C line 49436
4 => {
return val;
}
// C line 49434
5 => {
vm_block = 3; continue;
}
// C line 49429
6 => {
let _ = js_parse_error(s, c"Unexpected end of JSON input".as_ptr());
vm_block = 5; continue;
}
// C line 49431
7 => {
let _ = js_parse_error_cargs(s, c"unexpected token: '%.*s'".as_ptr(), &[ParserFormatArg::Slice((((*(s)).token).ptr) as *const c_char, ((((((*(s)).buf_ptr).offset_from(((*(s)).token).ptr) as i64)) as i32)) as i32)]);
vm_block = 5; continue;
}
// C line 49428 labels: def_token
8 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_EOF as i32))) as i32)) != 0 { 6 } else { 7 }; continue;
}
// C line 49425
9 => {
vm_block = 4; continue;
}
// C line 49424
10 => {
vm_block = 3; continue;
}
// C line 49423
11 => {
vm_block = if (json_next_token(s)) != 0 { 10 } else { 9 }; continue;
}
// C line 49403
12 => {
let _ = json_parse_record_init_primitive(ctx, pr, val, ((((((*(s)).token).ptr).offset_from((*(s)).buf_start) as i64)) as u32), (((((*(s)).buf_ptr).offset_from(((*(s)).token).ptr) as i64)) as u32));
vm_block = 11; continue;
}
// C line 49402
13 => {
vm_block = if !(pr).is_null() { 12 } else { 11 }; continue;
}
// C line 49401
14 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_true as i32)) as JSAtom))) as i32)); val = assigned; assigned };
vm_block = 13; continue;
}
// C line 49410
15 => {
let _ = json_parse_record_init_primitive(ctx, pr, val, ((((((*(s)).token).ptr).offset_from((*(s)).buf_start) as i64)) as u32), (((((*(s)).buf_ptr).offset_from(((*(s)).token).ptr) as i64)) as u32));
vm_block = 11; continue;
}
// C line 49409
16 => {
vm_block = if !(pr).is_null() { 15 } else { 11 }; continue;
}
// C line 49408
17 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; val = assigned; assigned };
vm_block = 16; continue;
}
// C line 49417
18 => {
let _ = { let assigned = JS_NewFloat64((*(s)).ctx, ((f32::NAN) as f64)); val = assigned; assigned };
vm_block = 11; continue;
}
// C line 49419
19 => {
let _ = { let assigned = JS_NewFloat64((*(s)).ctx, ((f32::INFINITY) as f64)); val = assigned; assigned };
vm_block = 11; continue;
}
// C line 49421
20 => {
vm_block = 8; continue;
}
// C line 49418
21 => {
vm_block = if ((((((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_Infinity as i32)) as JSAtom))) as i32)) != 0) && (((*(s)).ext_json) != 0)) as i32)) != 0 { 19 } else { 20 }; continue;
}
// C line 49414
22 => {
vm_block = if ((((((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_NaN as i32)) as JSAtom))) as i32)) != 0) && (((*(s)).ext_json) != 0)) as i32)) != 0 { 18 } else { 21 }; continue;
}
// C line 49407
23 => {
vm_block = if ((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_null as i32)) as JSAtom))) as i32)) != 0 { 17 } else { 22 }; continue;
}
// C line 49399
24 => {
vm_block = if ((((((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_false as i32)) as JSAtom))) as i32)) != 0) || (((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_true as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 14 } else { 23 }; continue;
}
// C line 49397
25 => {
vm_block = 4; continue;
}
// C line 49396
26 => {
vm_block = 3; continue;
}
// C line 49395
27 => {
vm_block = if (json_next_token(s)) != 0 { 26 } else { 25 }; continue;
}
// C line 49391
28 => {
let _ = json_parse_record_init_primitive(ctx, pr, val, ((((((*(s)).token).ptr).offset_from((*(s)).buf_start) as i64)) as u32), (((((*(s)).buf_ptr).offset_from(((*(s)).token).ptr) as i64)) as u32));
vm_block = 27; continue;
}
// C line 49390
29 => {
vm_block = if !(pr).is_null() { 28 } else { 27 }; continue;
}
// C line 49389
30 => {
let _ = { let assigned = ((((*(s)).token).u).num).val; val = assigned; assigned };
vm_block = 29; continue;
}
// C line 49387
31 => {
vm_block = 4; continue;
}
// C line 49386
32 => {
vm_block = 3; continue;
}
// C line 49385
33 => {
vm_block = if (json_next_token(s)) != 0 { 32 } else { 31 }; continue;
}
// C line 49381
34 => {
let _ = json_parse_record_init_primitive(ctx, pr, val, ((((((*(s)).token).ptr).offset_from((*(s)).buf_start) as i64)) as u32), (((((*(s)).buf_ptr).offset_from(((*(s)).token).ptr) as i64)) as u32));
vm_block = 33; continue;
}
// C line 49380
35 => {
vm_block = if !(pr).is_null() { 34 } else { 33 }; continue;
}
// C line 49379
36 => {
let _ = { let assigned = JS_DupValue(ctx, ((((*(s)).token).u).str).str); val = assigned; assigned };
vm_block = 35; continue;
}
// C line 49377
37 => {
vm_block = 4; continue;
}
// C line 49375
38 => {
vm_block = 3; continue;
}
// C line 49374
39 => {
vm_block = if (json_parse_expect(s, (93 as i32))) != 0 { 38 } else { 37 }; continue;
}
// C line 49349
40 => {
vm_block = 59; continue;
}
// C line 49371
41 => {
vm_block = 39; continue;
}
// C line 49370
42 => {
vm_block = if ((((((*(s)).ext_json) != 0) && (((((((*(s)).token).val) == ((93 as i32))) as i32)) != 0)) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 49369
43 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 42; continue;
}
// C line 49368
44 => {
vm_block = 3; continue;
}
// C line 49367
45 => {
vm_block = if (json_next_token(s)) != 0 { 44 } else { 43 }; continue;
}
// C line 49366
46 => {
vm_block = 39; continue;
}
// C line 49365
47 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 49364
48 => {
vm_block = 3; continue;
}
// C line 49363
49 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 49362
50 => {
let _ = { let assigned = JS_DefinePropertyValueUint32(ctx, val, idx, el, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 49; continue;
}
// C line 49361
51 => {
vm_block = 3; continue;
}
// C line 49360
52 => {
vm_block = if (JS_IsException(el)) != 0 { 51 } else { 50 }; continue;
}
// C line 49359
53 => {
let _ = { let assigned = json_parse_value(s, pr1_1); el = assigned; assigned };
vm_block = 52; continue;
}
// C line 49355
54 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(pr1_1)).value = assigned; assigned };
vm_block = 53; continue;
}
// C line 49354
55 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((((*(pr)).u).array).elements).offset(({ let old = (((*(pr)).u).array).count; (((*(pr)).u).array).count = ((((*(pr)).u).array).count).wrapping_add(1); old }) as isize)); pr1_1 = assigned; assigned };
vm_block = 54; continue;
}
// C line 49353
56 => {
vm_block = 3; continue;
}
// C line 49351
57 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((((*(pr)).u).array).elements)) as *mut *mut c_void), (((size_of::<JSONParseRecord>() as usize)) as i32), core::ptr::addr_of_mut!(pr_size_1), ((((*(pr)).u).array).count).wrapping_add((1 as i32)))) != 0 { 56 } else { 55 }; continue;
}
// C line 49357
58 => {
let _ = { let assigned = core::ptr::null_mut::<JSONParseRecord>(); pr1_1 = assigned; assigned };
vm_block = 53; continue;
}
// C line 49350
59 => {
vm_block = if !(pr).is_null() { 57 } else { 58 }; continue;
}
// C line 49348
60 => {
let _ = { let assigned = (((0 as i32)) as u32); idx = assigned; assigned };
vm_block = 40; continue;
}
// C line 49347
61 => {
vm_block = if ((((((*(s)).token).val) != ((93 as i32))) as i32)) != 0 { 60 } else { 39 }; continue;
}
// C line 49345
62 => {
let _ = { let assigned = (0 as i32); pr_size_1 = assigned; assigned };
vm_block = 61; continue;
}
// C line 49344
63 => {
let _ = json_parse_record_init_array(ctx, pr, val);
vm_block = 62; continue;
}
// C line 49343
64 => {
vm_block = if !(pr).is_null() { 63 } else { 61 }; continue;
}
// C line 49342
65 => {
vm_block = 3; continue;
}
// C line 49341
66 => {
vm_block = if (JS_IsException(val)) != 0 { 65 } else { 64 }; continue;
}
// C line 49340
67 => {
let _ = { let assigned = JS_NewArray(ctx); val = assigned; assigned };
vm_block = 66; continue;
}
// C line 49339
68 => {
vm_block = 3; continue;
}
// C line 49338
69 => {
vm_block = if (json_next_token(s)) != 0 { 68 } else { 67 }; continue;
}
// C line 49330
70 => {
vm_block = 4; continue;
}
// C line 49328
71 => {
vm_block = 3; continue;
}
// C line 49327
72 => {
vm_block = if (json_parse_expect(s, (125 as i32))) != 0 { 71 } else { 70 }; continue;
}
// C line 49285
73 => {
vm_block = 104; continue;
}
// C line 49324
74 => {
vm_block = 72; continue;
}
// C line 49323
75 => {
vm_block = if ((((((*(s)).ext_json) != 0) && (((((((*(s)).token).val) == ((125 as i32))) as i32)) != 0)) as i32)) != 0 { 74 } else { 73 }; continue;
}
// C line 49322
76 => {
vm_block = 3; continue;
}
// C line 49321
77 => {
vm_block = if (json_next_token(s)) != 0 { 76 } else { 75 }; continue;
}
// C line 49320
78 => {
vm_block = 72; continue;
}
// C line 49319
79 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 78 } else { 77 }; continue;
}
// C line 49317
80 => {
vm_block = 3; continue;
}
// C line 49316
81 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 80 } else { 79 }; continue;
}
// C line 49315
82 => {
let _ = JS_FreeAtom(ctx, prop_name);
vm_block = 81; continue;
}
// C line 49313
83 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, val, prop_name, prop_val, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 82; continue;
}
// C line 49311
84 => {
vm_block = 3; continue;
}
// C line 49310 labels: fail1
85 => {
let _ = JS_FreeAtom(ctx, prop_name);
vm_block = 84; continue;
}
// C line 49308
86 => {
vm_block = if (JS_IsException(prop_val)) != 0 { 85 } else { 83 }; continue;
}
// C line 49307
87 => {
let _ = { let assigned = json_parse_value(s, pr1); prop_val = assigned; assigned };
vm_block = 86; continue;
}
// C line 49303
88 => {
vm_block = 85; continue;
}
// C line 49302
89 => {
vm_block = if ((!(!(pr1).is_null()) as i32)) != 0 { 88 } else { 87 }; continue;
}
// C line 49301
90 => {
let _ = { let assigned = json_parse_record_add(ctx, pr, prop_name, core::ptr::addr_of_mut!(pr_size)); pr1 = assigned; assigned };
vm_block = 89; continue;
}
// C line 49305
91 => {
let _ = { let assigned = core::ptr::null_mut::<JSONParseRecord>(); pr1 = assigned; assigned };
vm_block = 87; continue;
}
// C line 49300
92 => {
vm_block = if !(pr).is_null() { 90 } else { 91 }; continue;
}
// C line 49299
93 => {
vm_block = 85; continue;
}
// C line 49298
94 => {
vm_block = if (json_parse_expect(s, (58 as i32))) != 0 { 93 } else { 92 }; continue;
}
// C line 49297
95 => {
vm_block = 85; continue;
}
// C line 49296
96 => {
vm_block = if (json_next_token(s)) != 0 { 95 } else { 94 }; continue;
}
// C line 49289
97 => {
vm_block = 3; continue;
}
// C line 49288
98 => {
vm_block = if ((((prop_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 97 } else { 96 }; continue;
}
// C line 49287
99 => {
let _ = { let assigned = JS_ValueToAtom(ctx, ((((*(s)).token).u).str).str); prop_name = assigned; assigned };
vm_block = 98; continue;
}
// C line 49291
100 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); prop_name = assigned; assigned };
vm_block = 96; continue;
}
// C line 49294
101 => {
vm_block = 3; continue;
}
// C line 49293
102 => {
let _ = js_parse_error(s, c"expecting property name".as_ptr());
vm_block = 101; continue;
}
// C line 49290
103 => {
vm_block = if ((((((*(s)).ext_json) != 0) && (((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0)) as i32)) != 0 { 100 } else { 102 }; continue;
}
// C line 49286
104 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_STRING as i32))) as i32)) != 0 { 99 } else { 103 }; continue;
}
// C line 49284
105 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 73 } else { 72 }; continue;
}
// C line 49282
106 => {
let _ = { let assigned = (0 as i32); pr_size = assigned; assigned };
vm_block = 105; continue;
}
// C line 49281
107 => {
let _ = json_parse_record_init_obj(ctx, pr, val);
vm_block = 106; continue;
}
// C line 49280
108 => {
vm_block = if !(pr).is_null() { 107 } else { 105 }; continue;
}
// C line 49279
109 => {
vm_block = 3; continue;
}
// C line 49278
110 => {
vm_block = if (JS_IsException(val)) != 0 { 109 } else { 108 }; continue;
}
// C line 49277
111 => {
let _ = { let assigned = JS_NewObject(ctx); val = assigned; assigned };
vm_block = 110; continue;
}
// C line 49276
112 => {
vm_block = 3; continue;
}
// C line 49275
113 => {
vm_block = if (json_next_token(s)) != 0 { 112 } else { 111 }; continue;
}
// C line 49267
114 => {
vm_block = match ((*(s)).token).val { x if x == (TOK_IDENT as i32) => 24, x if x == (TOK_NUMBER as i32) => 30, x if x == (TOK_STRING as i32) => 36, x if x == (91 as i32) => 69, x if x == (123 as i32) => 113, _ => 8, }; continue;
}
// C line 49264
115 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(pr)).value = assigned; assigned };
vm_block = 114; continue;
}
// C line 49263
116 => {
vm_block = if !(pr).is_null() { 115 } else { 114 }; continue;
}
// C line 49260
117 => {
val = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) };
vm_block = 116; continue;
}
// C line 49259
118 => {
ctx = (*(s)).ctx;
vm_block = 117; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49443. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_ParseJSON3(mut ctx: *mut JSContext, mut buf: *const c_char, mut buf_len: usize, mut filename: *const c_char, mut flags: i32, mut pr: *mut JSONParseRecord) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s1: JSParseState = core::mem::zeroed();
let mut s: *mut JSParseState = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49466
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 49465
2 => {
let _ = free_token(s, core::ptr::addr_of_mut!((*(s)).token));
vm_block = 1; continue;
}
// C line 49464 labels: fail
3 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 2; continue;
}
// C line 49462
4 => {
return val;
}
// C line 49459
5 => {
vm_block = 3; continue;
}
// C line 49458
6 => {
let _ = json_free_parse_record(ctx, pr);
vm_block = 5; continue;
}
// C line 49457
7 => {
vm_block = if (js_parse_error(s, c"unexpected data at the end".as_ptr())) != 0 { 6 } else { 4 }; continue;
}
// C line 49456
8 => {
vm_block = if ((((((*(s)).token).val) != ((TOK_EOF as i32))) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 49455
9 => {
vm_block = 3; continue;
}
// C line 49454
10 => {
vm_block = if (JS_IsException(val)) != 0 { 9 } else { 8 }; continue;
}
// C line 49453
11 => {
let _ = { let assigned = json_parse_value(s, pr); val = assigned; assigned };
vm_block = 10; continue;
}
// C line 49452
12 => {
vm_block = 3; continue;
}
// C line 49451
13 => {
vm_block = if (json_next_token(s)) != 0 { 12 } else { 11 }; continue;
}
// C line 49450
14 => {
let _ = { let assigned = (((((flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != ((0 as i32))) as i32); (*(s)).ext_json = assigned; assigned };
vm_block = 13; continue;
}
// C line 49449
15 => {
let _ = js_parse_init(ctx, s, buf, buf_len, filename);
vm_block = 14; continue;
}
// C line 49447
16 => {
val = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 15; continue;
}
// C line 49446
17 => {
s = core::ptr::addr_of_mut!(s1);
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49469. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_ParseJSON2(mut ctx: *mut JSContext, mut buf: *const c_char, mut buf_len: usize, mut filename: *const c_char, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49472
1 => {
return JS_ParseJSON3(ctx, buf, buf_len, filename, flags, core::ptr::null_mut::<JSONParseRecord>());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49475. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_ParseJSON(mut ctx: *mut JSContext, mut buf: *const c_char, mut buf_len: usize, mut filename: *const c_char) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49478
1 => {
return JS_ParseJSON3(ctx, buf, buf_len, filename, (0 as i32), core::ptr::null_mut::<JSONParseRecord>());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49482. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn internalize_json_property(mut ctx: *mut JSContext, mut holder: JSValue, mut name: JSAtom, mut reviver: JSValue, mut text_str: *const c_char, mut pr: *mut JSONParseRecord) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut new_el: JSValue = core::mem::zeroed();
let mut name_val: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut context: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut is_array: i32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut prop: JSAtom = core::mem::zeroed();
let mut atoms: *mut JSPropertyEnum = core::mem::zeroed();
let mut idx: u32 = core::mem::zeroed();
let mut vm_block: usize = 71;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49584
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 49583
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 49582
3 => {
let _ = JS_FreeValue(ctx, context);
vm_block = 2; continue;
}
// C line 49581 labels: fail
4 => {
let _ = JS_FreePropertyEnum(ctx, atoms, len);
vm_block = 3; continue;
}
// C line 49579
5 => {
return res;
}
// C line 49578
6 => {
let _ = JS_FreeValue(ctx, context);
vm_block = 5; continue;
}
// C line 49577
7 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 6; continue;
}
// C line 49576
8 => {
let _ = JS_FreeValue(ctx, name_val);
vm_block = 7; continue;
}
// C line 49575
9 => {
let _ = { let assigned = JS_Call(ctx, reviver, holder, (3 as i32), (args).as_mut_ptr()); res = assigned; assigned };
vm_block = 8; continue;
}
// C line 49574
10 => {
let _ = { let assigned = context; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 49573
11 => {
let _ = { let assigned = val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 49572
12 => {
let _ = { let assigned = name_val; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 49571
13 => {
vm_block = 4; continue;
}
// C line 49570
14 => {
vm_block = if (JS_IsException(name_val)) != 0 { 13 } else { 12 }; continue;
}
// C line 49569
15 => {
let _ = { let assigned = JS_AtomToValue(ctx, name); name_val = assigned; assigned };
vm_block = 14; continue;
}
// C line 49568
16 => {
let _ = { let assigned = core::ptr::null_mut::<JSPropertyEnum>(); atoms = assigned; assigned };
vm_block = 15; continue;
}
// C line 49567
17 => {
let _ = JS_FreePropertyEnum(ctx, atoms, len);
vm_block = 16; continue;
}
// C line 49535
18 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 34 } else { 17 }; continue;
}
// C line 49535
19 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 18; continue;
}
// C line 49555
20 => {
vm_block = 4; continue;
}
// C line 49554
21 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 49553
22 => {
let _ = JS_FreeAtom(ctx, prop);
vm_block = 21; continue;
}
// C line 49549
23 => {
let _ = { let assigned = JS_DeleteProperty(ctx, val, prop, (0 as i32)); ret = assigned; assigned };
vm_block = 22; continue;
}
// C line 49551
24 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, val, prop, new_el, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 22; continue;
}
// C line 49548
25 => {
vm_block = if (JS_IsUndefined(new_el)) != 0 { 23 } else { 24 }; continue;
}
// C line 49546
26 => {
vm_block = 4; continue;
}
// C line 49545
27 => {
let _ = JS_FreeAtom(ctx, prop);
vm_block = 26; continue;
}
// C line 49544
28 => {
vm_block = if (JS_IsException(new_el)) != 0 { 27 } else { 25 }; continue;
}
// C line 49543
29 => {
let _ = { let assigned = internalize_json_property(ctx, val, prop, reviver, text_str, pr); new_el = assigned; assigned };
vm_block = 28; continue;
}
// C line 49539
30 => {
vm_block = 4; continue;
}
// C line 49538
31 => {
vm_block = if ((((prop) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 49537
32 => {
let _ = { let assigned = JS_NewAtomUInt32(ctx, i); prop = assigned; assigned };
vm_block = 31; continue;
}
// C line 49541
33 => {
let _ = { let assigned = JS_DupAtom(ctx, (*(atoms).offset((i) as isize)).atom); prop = assigned; assigned };
vm_block = 29; continue;
}
// C line 49536
34 => {
vm_block = if (is_array) != 0 { 32 } else { 33 }; continue;
}
// C line 49535
35 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 18; continue;
}
// C line 49529
36 => {
vm_block = 4; continue;
}
// C line 49528
37 => {
vm_block = if (js_get_length32(ctx, core::ptr::addr_of_mut!(len), val)) != 0 { 36 } else { 35 }; continue;
}
// C line 49533
38 => {
vm_block = 4; continue;
}
// C line 49532
39 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 38 } else { 35 }; continue;
}
// C line 49531
40 => {
let _ = { let assigned = JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(atoms), core::ptr::addr_of_mut!(len), ((((val).u).ptr) as *mut JSObject), ((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 39; continue;
}
// C line 49527
41 => {
vm_block = if (is_array) != 0 { 37 } else { 40 }; continue;
}
// C line 49526
42 => {
vm_block = 4; continue;
}
// C line 49525
43 => {
vm_block = if ((((is_array) < ((0 as i32))) as i32)) != 0 { 42 } else { 41 }; continue;
}
// C line 49524
44 => {
let _ = { let assigned = JS_IsArray(ctx, val); is_array = assigned; assigned };
vm_block = 43; continue;
}
// C line 49564
45 => {
vm_block = 4; continue;
}
// C line 49563
46 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, context, (((crate::quickjs_atom::JS_ATOM_source as i32)) as JSAtom), new_el, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 45 } else { 17 }; continue;
}
// C line 49562
47 => {
vm_block = 4; continue;
}
// C line 49561
48 => {
vm_block = if (JS_IsException(new_el)) != 0 { 47 } else { 46 }; continue;
}
// C line 49559
49 => {
let _ = { let assigned = JS_NewStringLen(ctx, (text_str).offset((((((*(pr)).u).primitive).source_pos) as isize)), (((((*(pr)).u).primitive).source_len) as usize)); new_el = assigned; assigned };
vm_block = 48; continue;
}
// C line 49558
50 => {
vm_block = if !(pr).is_null() { 49 } else { 17 }; continue;
}
// C line 49523
51 => {
vm_block = if (JS_IsObject(val)) != 0 { 44 } else { 50 }; continue;
}
// C line 49521
52 => {
vm_block = 4; continue;
}
// C line 49520
53 => {
vm_block = if (JS_IsException(context)) != 0 { 52 } else { 51 }; continue;
}
// C line 49519
54 => {
let _ = { let assigned = JS_NewObject(ctx); context = assigned; assigned };
vm_block = 53; continue;
}
// C line 49515
55 => {
let _ = { let assigned = core::ptr::null_mut::<JSONParseRecord>(); pr = assigned; assigned };
vm_block = 54; continue;
}
// C line 49514
56 => {
vm_block = if ((((!(pr).is_null()) && (((!((js_same_value(ctx, (*(pr)).value, val)) != 0) as i32)) != 0)) as i32)) != 0 { 55 } else { 54 }; continue;
}
// C line 49506
57 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((((*(pr)).u).array).elements).offset((idx) as isize)); pr = assigned; assigned };
vm_block = 56; continue;
}
// C line 49508
58 => {
let _ = { let assigned = core::ptr::null_mut::<JSONParseRecord>(); pr = assigned; assigned };
vm_block = 56; continue;
}
// C line 49505
59 => {
vm_block = if ((((idx) < ((((((*(pr)).u).array).count) as u32))) as i32)) != 0 { 57 } else { 58 }; continue;
}
// C line 49504
60 => {
idx = __JS_AtomToUInt32(name);
vm_block = 59; continue;
}
// C line 49503
61 => {
vm_block = if (__JS_AtomIsTaggedInt(name)) != 0 { 60 } else { 56 }; continue;
}
// C line 49512
62 => {
let _ = { let assigned = json_parse_record_find(pr, name); pr = assigned; assigned };
vm_block = 56; continue;
}
// C line 49502
63 => {
vm_block = if (JS_IsArray(ctx, (*(pr)).value)) != 0 { 61 } else { 62 }; continue;
}
// C line 49501
64 => {
vm_block = if !(pr).is_null() { 63 } else { 54 }; continue;
}
// C line 49499
65 => {
return val;
}
// C line 49498
66 => {
vm_block = if (JS_IsException(val)) != 0 { 65 } else { 64 }; continue;
}
// C line 49497
67 => {
let _ = { let assigned = JS_GetProperty(ctx, holder, name); val = assigned; assigned };
vm_block = 66; continue;
}
// C line 49494
68 => {
return JS_ThrowStackOverflow(ctx);
}
// C line 49493
69 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 68 } else { 67 }; continue;
}
// C line 49491
70 => {
atoms = core::ptr::null_mut::<JSPropertyEnum>();
vm_block = 69; continue;
}
// C line 49489
71 => {
len = (((0 as i32)) as u32);
vm_block = 70; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49587. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_json_parse(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut len: usize = core::mem::zeroed();
let mut pr_s: JSONParseRecord = core::mem::zeroed();
let mut pr: *mut JSONParseRecord = core::mem::zeroed();
let mut pr1: *mut JSONParseRecord = core::mem::zeroed();
let mut root: JSValue = core::mem::zeroed();
let mut reviver: JSValue = core::mem::zeroed();
let mut size: i32 = core::mem::zeroed();
let mut vm_block: usize = 30;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49637
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 49636 labels: fail
2 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 1; continue;
}
// C line 49634
3 => {
return obj;
}
// C line 49633
4 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 3; continue;
}
// C line 49629
5 => {
let _ = JS_FreeValue(ctx, root);
vm_block = 4; continue;
}
// C line 49628
6 => {
let _ = json_free_parse_record(ctx, pr);
vm_block = 5; continue;
}
// C line 49626
7 => {
let _ = { let assigned = internalize_json_property(ctx, root, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom), reviver, str, pr); obj = assigned; assigned };
vm_block = 6; continue;
}
// C line 49623
8 => {
vm_block = 2; continue;
}
// C line 49622
9 => {
let _ = JS_FreeValue(ctx, root);
vm_block = 8; continue;
}
// C line 49621 labels: fail1
10 => {
let _ = json_free_parse_record(ctx, pr);
vm_block = 9; continue;
}
// C line 49619
11 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 10; continue;
}
// C line 49617
12 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, root, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom), obj, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 11 } else { 7 }; continue;
}
// C line 49615
13 => {
vm_block = 10; continue;
}
// C line 49614
14 => {
vm_block = if (JS_IsException(obj)) != 0 { 13 } else { 12 }; continue;
}
// C line 49613
15 => {
let _ = { let assigned = JS_ParseJSON3(ctx, str, len, c"<input>".as_ptr(), (0 as i32), pr1); obj = assigned; assigned };
vm_block = 14; continue;
}
// C line 49611
16 => {
vm_block = 10; continue;
}
// C line 49610
17 => {
vm_block = if ((!(!(pr1).is_null()) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 49609
18 => {
let _ = { let assigned = json_parse_record_add(ctx, pr, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom), core::ptr::addr_of_mut!(size)); pr1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 49608
19 => {
let _ = { let assigned = (0 as i32); size = assigned; assigned };
vm_block = 18; continue;
}
// C line 49607
20 => {
let _ = json_parse_record_init_obj(ctx, pr, root);
vm_block = 19; continue;
}
// C line 49606
21 => {
vm_block = 2; continue;
}
// C line 49605
22 => {
vm_block = if (JS_IsException(root)) != 0 { 21 } else { 20 }; continue;
}
// C line 49604
23 => {
let _ = { let assigned = JS_NewObject(ctx); root = assigned; assigned };
vm_block = 22; continue;
}
// C line 49603
24 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); reviver = assigned; assigned };
vm_block = 23; continue;
}
// C line 49598
25 => {
pr = core::ptr::addr_of_mut!(pr_s);
vm_block = 24; continue;
}
// C line 49631
26 => {
let _ = { let assigned = JS_ParseJSON3(ctx, str, len, c"<input>".as_ptr(), (0 as i32), core::ptr::null_mut::<JSONParseRecord>()); obj = assigned; assigned };
vm_block = 4; continue;
}
// C line 49597
27 => {
vm_block = if ((((((((argc) > ((1 as i32))) as i32)) != 0) && ((JS_IsFunction(ctx, *(argv).offset(((1 as i32)) as isize))) != 0)) as i32)) != 0 { 25 } else { 26 }; continue;
}
// C line 49596
28 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 49595
29 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 49594
30 => {
let _ = { let assigned = JS_ToCStringLen(ctx, core::ptr::addr_of_mut!(len), *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 29; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49640. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_json_isRawJSON(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49646
1 => {
return JS_NewBool(ctx, ((((((*(p)).class_id) as i32)) == ((JS_CLASS_RAWJSON as i32))) as i32));
}
// C line 49645
2 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 1; continue;
}
// C line 49648
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 49644
4 => {
vm_block = if (((((((obj).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 49643
5 => {
obj = *(argv).offset(((0 as i32)) as isize);
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49652. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_valid_raw_json_char(mut c: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49654
1 => {
return (((((((((((((((((((c) >= ((97 as i32))) as i32)) != 0) && (((((c) <= ((122 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((c) >= ((48 as i32))) as i32)) != 0) && (((((c) <= ((57 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((c) == ((45 as i32))) as i32)) != 0)) as i32)) != 0) || (((((c) == ((34 as i32))) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49660. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_json_rawJSON(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49693
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 49692 labels: fail
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 49690
3 => {
return obj;
}
// C line 49689
4 => {
let _ = JS_PreventExtensions(ctx, obj);
vm_block = 3; continue;
}
// C line 49687
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 49686
6 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 5; continue;
}
// C line 49685
7 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_rawJSON as i32)) as JSAtom), str, ((1 as i32)).wrapping_shl(((2 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 49684
8 => {
vm_block = 2; continue;
}
// C line 49683
9 => {
vm_block = if (JS_IsException(obj)) != 0 { 8 } else { 7 }; continue;
}
// C line 49682
10 => {
let _ = { let assigned = JS_NewObjectProtoClass(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }, (((JS_CLASS_RAWJSON as i32)) as JSClassID)); obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 49680
11 => {
let _ = JS_FreeValue(ctx, res);
vm_block = 10; continue;
}
// C line 49678
12 => {
vm_block = 2; continue;
}
// C line 49677 labels: syntax_error
13 => {
let _ = JS_ThrowSyntaxError(ctx, c"invalid rawJSON string".as_ptr());
vm_block = 12; continue;
}
// C line 49675
14 => {
vm_block = if (JS_IsException(res)) != 0 { 13 } else { 11 }; continue;
}
// C line 49674
15 => {
let _ = { let assigned = js_json_parse(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(str)); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 49672
16 => {
vm_block = 13; continue;
}
// C line 49669
17 => {
vm_block = if (((((((((((((((*(p)).len()) as i32)) == ((0 as i32))) as i32)) != 0) || (((!((is_valid_raw_json_char(string_get(p, (0 as i32)))) != 0) as i32)) != 0)) as i32)) != 0) || (((!((is_valid_raw_json_char(string_get(p, ((((*(p)).len()) as i32)).wrapping_sub((1 as i32))))) != 0) as i32)) != 0)) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 49668
18 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 17; continue;
}
// C line 49667
19 => {
return str;
}
// C line 49666
20 => {
vm_block = if (JS_IsException(str)) != 0 { 19 } else { 18 }; continue;
}
// C line 49665
21 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 20; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49706. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ToQuotedString(mut ctx: *mut JSContext, mut b: *mut StringBuffer, mut val1: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut buf: [c_char; 16] = core::mem::zeroed();
let mut vm_block: usize = 38;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49765
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49764 labels: fail
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 49762
3 => {
return (0 as i32);
}
// C line 49761
4 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 3; continue;
}
// C line 49760
5 => {
vm_block = 2; continue;
}
// C line 49759
6 => {
vm_block = if (string_buffer_putc8(b, (((34 as i32)) as u32))) != 0 { 5 } else { 4 }; continue;
}
// C line 49721
7 => {
vm_block = if ((((i) < ((((*(p)).len()) as i32))) as i32)) != 0 { 31 } else { 6 }; continue;
}
// C line 49756
8 => {
vm_block = 7; continue;
}
// C line 49751
9 => {
vm_block = 2; continue;
}
// C line 49750
10 => {
vm_block = if (string_buffer_puts8(b, (buf).as_mut_ptr())) != 0 { 9 } else { 8 }; continue;
}
// C line 49749
11 => {
let _ = json_format_unicode((buf).as_mut_ptr(), (size_of::<[c_char; 16]>() as usize), c);
vm_block = 10; continue;
}
// C line 49754
12 => {
vm_block = 2; continue;
}
// C line 49753
13 => {
vm_block = if (string_buffer_putc(b, c)) != 0 { 12 } else { 8 }; continue;
}
// C line 49748
14 => {
vm_block = if ((((((((c) < ((((32 as i32)) as u32))) as i32)) != 0) || ((crate::cutils_header::is_surrogate(c)) != 0)) as i32)) != 0 { 11 } else { 13 }; continue;
}
// C line 49746
15 => {
vm_block = 7; continue;
}
// C line 49745
16 => {
vm_block = 2; continue;
}
// C line 49744
17 => {
vm_block = if (string_buffer_putc8(b, c)) != 0 { 16 } else { 15 }; continue;
}
// C line 49743
18 => {
vm_block = 2; continue;
}
// C line 49742 labels: quote
19 => {
vm_block = if (string_buffer_putc8(b, (((92 as i32)) as u32))) != 0 { 18 } else { 17 }; continue;
}
// C line 49738
20 => {
vm_block = 19; continue;
}
// C line 49737
21 => {
let _ = { let assigned = (((102 as i32)) as u32); c = assigned; assigned };
vm_block = 20; continue;
}
// C line 49735
22 => {
vm_block = 19; continue;
}
// C line 49734
23 => {
let _ = { let assigned = (((98 as i32)) as u32); c = assigned; assigned };
vm_block = 22; continue;
}
// C line 49732
24 => {
vm_block = 19; continue;
}
// C line 49731
25 => {
let _ = { let assigned = (((110 as i32)) as u32); c = assigned; assigned };
vm_block = 24; continue;
}
// C line 49729
26 => {
vm_block = 19; continue;
}
// C line 49728
27 => {
let _ = { let assigned = (((114 as i32)) as u32); c = assigned; assigned };
vm_block = 26; continue;
}
// C line 49726
28 => {
vm_block = 19; continue;
}
// C line 49725
29 => {
let _ = { let assigned = (((116 as i32)) as u32); c = assigned; assigned };
vm_block = 28; continue;
}
// C line 49723
30 => {
vm_block = match c { x if x == (((92 as i32)) as u32) => 19, x if x == (((34 as i32)) as u32) => 19, x if x == (((12 as i32)) as u32) => 21, x if x == (((8 as i32)) as u32) => 23, x if x == (((10 as i32)) as u32) => 25, x if x == (((13 as i32)) as u32) => 27, x if x == (((9 as i32)) as u32) => 29, _ => 14, }; continue;
}
// C line 49722
31 => {
let _ = { let assigned = ((string_getc(p, core::ptr::addr_of_mut!(i))) as u32); c = assigned; assigned };
vm_block = 30; continue;
}
// C line 49721
32 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 7; continue;
}
// C line 49720
33 => {
vm_block = 2; continue;
}
// C line 49719
34 => {
vm_block = if (string_buffer_putc8(b, (((34 as i32)) as u32))) != 0 { 33 } else { 32 }; continue;
}
// C line 49717
35 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 34; continue;
}
// C line 49716
36 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49715
37 => {
vm_block = if (JS_IsException(val)) != 0 { 36 } else { 35 }; continue;
}
// C line 49714
38 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, val1); val = assigned; assigned };
vm_block = 37; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49768. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ToQuotedStringFree(mut ctx: *mut JSContext, mut b: *mut StringBuffer, mut val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49771
1 => {
return ret;
}
// C line 49770
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 49769
3 => {
ret = JS_ToQuotedString(ctx, b, val);
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49774. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_json_check(mut ctx: *mut JSContext, mut jsc: *mut JSONStringifyContext, mut holder: JSValue, mut val: JSValue, mut key: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut f: JSValue = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49829
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 49828 labels: exception
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 49825
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 49824
4 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 3; continue;
}
// C line 49822
5 => {
vm_block = 4; continue;
}
// C line 49820
6 => {
return val;
}
// C line 49810
7 => {
vm_block = 4; continue;
}
// C line 49809
8 => {
vm_block = if (JS_IsFunction(ctx, val)) != 0 { 7 } else { 6 }; continue;
}
// C line 49807
9 => {
vm_block = match (((val).tag) as i32) { x if x == (JS_TAG_EXCEPTION as i32) => 6, x if x == (JS_TAG_BIG_INT as i32) => 6, x if x == (JS_TAG_SHORT_BIG_INT as i32) => 6, x if x == (JS_TAG_NULL as i32) => 6, x if x == (JS_TAG_BOOL as i32) => 6, x if x == (JS_TAG_FLOAT64 as i32) => 6, x if x == (JS_TAG_INT as i32) => 6, x if x == (JS_TAG_STRING_ROPE as i32) => 6, x if x == (JS_TAG_STRING as i32) => 6, x if x == (JS_TAG_OBJECT as i32) => 8, _ => 5, }; continue;
}
// C line 49804
10 => {
vm_block = 2; continue;
}
// C line 49803
11 => {
vm_block = if (JS_IsException(val)) != 0 { 10 } else { 9 }; continue;
}
// C line 49802
12 => {
let _ = { let assigned = v; val = assigned; assigned };
vm_block = 11; continue;
}
// C line 49801
13 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 12; continue;
}
// C line 49800
14 => {
let _ = { let assigned = JS_Call(ctx, (*(jsc)).replacer_func, holder, (2 as i32), (args).as_mut_ptr()); v = assigned; assigned };
vm_block = 13; continue;
}
// C line 49799
15 => {
let _ = { let assigned = val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 49798
16 => {
let _ = { let assigned = key; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 49797
17 => {
vm_block = if ((!((JS_IsUndefined((*(jsc)).replacer_func)) != 0) as i32)) != 0 { 16 } else { 9 }; continue;
}
// C line 49791
18 => {
vm_block = 2; continue;
}
// C line 49790
19 => {
vm_block = if (JS_IsException(val)) != 0 { 18 } else { 17 }; continue;
}
// C line 49789
20 => {
let _ = { let assigned = v; val = assigned; assigned };
vm_block = 19; continue;
}
// C line 49788
21 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 20; continue;
}
// C line 49787
22 => {
let _ = { let assigned = JS_CallFree(ctx, f, val, (1 as i32), core::ptr::addr_of_mut!(key)); v = assigned; assigned };
vm_block = 21; continue;
}
// C line 49793
23 => {
let _ = JS_FreeValue(ctx, f);
vm_block = 17; continue;
}
// C line 49786
24 => {
vm_block = if (JS_IsFunction(ctx, f)) != 0 { 22 } else { 23 }; continue;
}
// C line 49785
25 => {
vm_block = 2; continue;
}
// C line 49784
26 => {
vm_block = if (JS_IsException(f)) != 0 { 25 } else { 24 }; continue;
}
// C line 49783
27 => {
f = JS_GetProperty(ctx, val, (((crate::quickjs_atom::JS_ATOM_toJSON as i32)) as JSAtom));
vm_block = 26; continue;
}
// C line 49782
28 => {
vm_block = if (((((JS_IsObject(val)) != 0) || ((JS_IsBigInt(ctx, val)) != 0)) as i32)) != 0 { 27 } else { 17 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49832. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_json_to_str(mut ctx: *mut JSContext, mut jsc: *mut JSONStringifyContext, mut holder: JSValue, mut val: JSValue, mut indent: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut indent1: JSValue = core::mem::zeroed();
let mut sep: JSValue = core::mem::zeroed();
let mut sep1: JSValue = core::mem::zeroed();
let mut tab: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut cl: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut has_content: i32 = core::mem::zeroed();
let mut val1: JSValue = core::mem::zeroed();
let mut vm_block: usize = 148;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50021
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50020
2 => {
let _ = JS_FreeValue(ctx, prop);
vm_block = 1; continue;
}
// C line 50019
3 => {
let _ = JS_FreeValue(ctx, indent1);
vm_block = 2; continue;
}
// C line 50018
4 => {
let _ = JS_FreeValue(ctx, sep1);
vm_block = 3; continue;
}
// C line 50017
5 => {
let _ = JS_FreeValue(ctx, sep);
vm_block = 4; continue;
}
// C line 50016
6 => {
let _ = JS_FreeValue(ctx, tab);
vm_block = 5; continue;
}
// C line 50015 labels: exception
7 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 6; continue;
}
// C line 50011
8 => {
return (0 as i32);
}
// C line 50010
9 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 8; continue;
}
// C line 50008
10 => {
vm_block = 7; continue;
}
// C line 50007
11 => {
let _ = JS_ThrowTypeError(ctx, c"Do not know how to serialize a BigInt".as_ptr());
vm_block = 10; continue;
}
// C line 50003 labels: concat_value
12 => {
return string_buffer_concat_value_free((*(jsc)).b, val);
}
// C line 49998
13 => {
vm_block = 12; continue;
}
// C line 49996
14 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; val = assigned; assigned };
vm_block = 13; continue;
}
// C line 49995
15 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((val).u).float64).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (((val).u).float64).is_finite() as i32 } else { (((val).u).float64).is_finite() as i32 } }) != 0) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 49993
16 => {
return JS_ToQuotedStringFree(ctx, (*(jsc)).b, val);
}
// C line 49990 labels: concat_primitive
17 => {
vm_block = match (((val).tag) as i32) { x if x == (JS_TAG_BIG_INT as i32) => 11, x if x == (JS_TAG_SHORT_BIG_INT as i32) => 11, x if x == (JS_TAG_NULL as i32) => 12, x if x == (JS_TAG_BOOL as i32) => 12, x if x == (JS_TAG_INT as i32) => 12, x if x == (JS_TAG_FLOAT64 as i32) => 15, x if x == (JS_TAG_STRING_ROPE as i32) => 16, x if x == (JS_TAG_STRING as i32) => 16, _ => 9, }; continue;
}
// C line 49987
18 => {
return (0 as i32);
}
// C line 49986
19 => {
let _ = JS_FreeValue(ctx, prop);
vm_block = 18; continue;
}
// C line 49985
20 => {
let _ = JS_FreeValue(ctx, indent1);
vm_block = 19; continue;
}
// C line 49984
21 => {
let _ = JS_FreeValue(ctx, sep1);
vm_block = 20; continue;
}
// C line 49983
22 => {
let _ = JS_FreeValue(ctx, sep);
vm_block = 21; continue;
}
// C line 49982
23 => {
let _ = JS_FreeValue(ctx, tab);
vm_block = 22; continue;
}
// C line 49981
24 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 23; continue;
}
// C line 49980
25 => {
vm_block = 7; continue;
}
// C line 49979
26 => {
vm_block = if (check_exception_free(ctx, js_array_pop(ctx, (*(jsc)).stack, (0 as i32), core::ptr::null_mut::<JSValue>(), (0 as i32)))) != 0 { 25 } else { 24 }; continue;
}
// C line 49935
27 => {
let _ = string_buffer_putc8((*(jsc)).b, (((93 as i32)) as u32));
vm_block = 26; continue;
}
// C line 49933
28 => {
let _ = string_buffer_concat_value((*(jsc)).b, indent);
vm_block = 27; continue;
}
// C line 49932
29 => {
let _ = string_buffer_putc8((*(jsc)).b, (((10 as i32)) as u32));
vm_block = 28; continue;
}
// C line 49931
30 => {
vm_block = if ((((((((len) > ((((0 as i32)) as i64))) as i32)) != 0) && (((!((JS_IsEmptyString((*(jsc)).gap)) != 0) as i32)) != 0)) as i32)) != 0 { 29 } else { 27 }; continue;
}
// C line 49910
31 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 50 } else { 30 }; continue;
}
// C line 49910
32 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 49929
33 => {
vm_block = 7; continue;
}
// C line 49928
34 => {
vm_block = if (js_json_to_str(ctx, jsc, val, v, indent1)) != 0 { 33 } else { 32 }; continue;
}
// C line 49927
35 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; v = assigned; assigned };
vm_block = 34; continue;
}
// C line 49926
36 => {
vm_block = if (JS_IsUndefined(v)) != 0 { 35 } else { 34 }; continue;
}
// C line 49925
37 => {
vm_block = 7; continue;
}
// C line 49924
38 => {
vm_block = if (JS_IsException(v)) != 0 { 37 } else { 36 }; continue;
}
// C line 49923
39 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; prop = assigned; assigned };
vm_block = 38; continue;
}
// C line 49922
40 => {
let _ = JS_FreeValue(ctx, prop);
vm_block = 39; continue;
}
// C line 49921
41 => {
let _ = { let assigned = js_json_check(ctx, jsc, val, v, prop); v = assigned; assigned };
vm_block = 40; continue;
}
// C line 49920
42 => {
vm_block = 7; continue;
}
// C line 49919
43 => {
vm_block = if (JS_IsException(prop)) != 0 { 42 } else { 41 }; continue;
}
// C line 49918
44 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_NewInt64(ctx, i)); prop = assigned; assigned };
vm_block = 43; continue;
}
// C line 49916
45 => {
vm_block = 7; continue;
}
// C line 49915
46 => {
vm_block = if (JS_IsException(v)) != 0 { 45 } else { 44 }; continue;
}
// C line 49914
47 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, val, i); v = assigned; assigned };
vm_block = 46; continue;
}
// C line 49913
48 => {
let _ = string_buffer_concat_value((*(jsc)).b, sep);
vm_block = 47; continue;
}
// C line 49912
49 => {
let _ = string_buffer_putc8((*(jsc)).b, (((44 as i32)) as u32));
vm_block = 48; continue;
}
// C line 49911
50 => {
vm_block = if ((((i) > ((((0 as i32)) as i64))) as i32)) != 0 { 49 } else { 48 }; continue;
}
// C line 49910
51 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 31; continue;
}
// C line 49909
52 => {
let _ = string_buffer_putc8((*(jsc)).b, (((91 as i32)) as u32));
vm_block = 51; continue;
}
// C line 49908
53 => {
vm_block = 7; continue;
}
// C line 49907
54 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), val)) != 0 { 53 } else { 52 }; continue;
}
// C line 49977
55 => {
let _ = string_buffer_putc8((*(jsc)).b, (((125 as i32)) as u32));
vm_block = 26; continue;
}
// C line 49975
56 => {
let _ = string_buffer_concat_value((*(jsc)).b, indent);
vm_block = 55; continue;
}
// C line 49974
57 => {
let _ = string_buffer_putc8((*(jsc)).b, (((10 as i32)) as u32));
vm_block = 56; continue;
}
// C line 49973
58 => {
vm_block = if (((((has_content) != 0) && (((!((JS_IsEmptyString((*(jsc)).gap)) != 0) as i32)) != 0)) as i32)) != 0 { 57 } else { 55 }; continue;
}
// C line 49947
59 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 82 } else { 58 }; continue;
}
// C line 49947
60 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 59; continue;
}
// C line 49970
61 => {
let _ = { let assigned = (1 as i32); has_content = assigned; assigned };
vm_block = 60; continue;
}
// C line 49969
62 => {
vm_block = 7; continue;
}
// C line 49968
63 => {
vm_block = if (js_json_to_str(ctx, jsc, val, v, indent1)) != 0 { 62 } else { 61 }; continue;
}
// C line 49967
64 => {
let _ = string_buffer_concat_value((*(jsc)).b, sep1);
vm_block = 63; continue;
}
// C line 49966
65 => {
let _ = string_buffer_putc8((*(jsc)).b, (((58 as i32)) as u32));
vm_block = 64; continue;
}
// C line 49964
66 => {
vm_block = 7; continue;
}
// C line 49963
67 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 66; continue;
}
// C line 49962
68 => {
vm_block = if (JS_ToQuotedString(ctx, (*(jsc)).b, prop)) != 0 { 67 } else { 65 }; continue;
}
// C line 49961
69 => {
let _ = string_buffer_concat_value((*(jsc)).b, sep);
vm_block = 68; continue;
}
// C line 49960
70 => {
let _ = string_buffer_putc8((*(jsc)).b, (((44 as i32)) as u32));
vm_block = 69; continue;
}
// C line 49959
71 => {
vm_block = if (has_content) != 0 { 70 } else { 69 }; continue;
}
// C line 49958
72 => {
vm_block = if ((!((JS_IsUndefined(v)) != 0) as i32)) != 0 { 71 } else { 60 }; continue;
}
// C line 49957
73 => {
vm_block = 7; continue;
}
// C line 49956
74 => {
vm_block = if (JS_IsException(v)) != 0 { 73 } else { 72 }; continue;
}
// C line 49955
75 => {
let _ = { let assigned = js_json_check(ctx, jsc, val, v, prop); v = assigned; assigned };
vm_block = 74; continue;
}
// C line 49954
76 => {
vm_block = 7; continue;
}
// C line 49953
77 => {
vm_block = if (JS_IsException(v)) != 0 { 76 } else { 75 }; continue;
}
// C line 49952
78 => {
let _ = { let assigned = JS_GetPropertyValue(ctx, val, JS_DupValue(ctx, prop)); v = assigned; assigned };
vm_block = 77; continue;
}
// C line 49951
79 => {
vm_block = 7; continue;
}
// C line 49950
80 => {
vm_block = if (JS_IsException(prop)) != 0 { 79 } else { 78 }; continue;
}
// C line 49949
81 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, tab, i); prop = assigned; assigned };
vm_block = 80; continue;
}
// C line 49948
82 => {
let _ = JS_FreeValue(ctx, prop);
vm_block = 81; continue;
}
// C line 49947
83 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 59; continue;
}
// C line 49946
84 => {
let _ = { let assigned = (0 as i32); has_content = assigned; assigned };
vm_block = 83; continue;
}
// C line 49945
85 => {
let _ = string_buffer_putc8((*(jsc)).b, (((123 as i32)) as u32));
vm_block = 84; continue;
}
// C line 49944
86 => {
vm_block = 7; continue;
}
// C line 49943
87 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), tab)) != 0 { 86 } else { 85 }; continue;
}
// C line 49942
88 => {
vm_block = 7; continue;
}
// C line 49941
89 => {
vm_block = if (JS_IsException(tab)) != 0 { 88 } else { 87 }; continue;
}
// C line 49938
90 => {
let _ = { let assigned = JS_DupValue(ctx, (*(jsc)).property_list); tab = assigned; assigned };
vm_block = 89; continue;
}
// C line 49940
91 => {
let _ = { let assigned = js_object_keys(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(val), (JS_ITERATOR_KIND_KEY as i32)); tab = assigned; assigned };
vm_block = 89; continue;
}
// C line 49937
92 => {
vm_block = if ((!((JS_IsUndefined((*(jsc)).property_list)) != 0) as i32)) != 0 { 90 } else { 91 }; continue;
}
// C line 49906
93 => {
vm_block = if (ret) != 0 { 54 } else { 92 }; continue;
}
// C line 49905
94 => {
vm_block = 7; continue;
}
// C line 49904
95 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 94 } else { 93 }; continue;
}
// C line 49903
96 => {
let _ = { let assigned = JS_IsArray(ctx, val); ret = assigned; assigned };
vm_block = 95; continue;
}
// C line 49902
97 => {
vm_block = 7; continue;
}
// C line 49901
98 => {
vm_block = if (check_exception_free(ctx, v)) != 0 { 97 } else { 96 }; continue;
}
// C line 49900
99 => {
let _ = { let assigned = js_array_push(ctx, (*(jsc)).stack, (1 as i32), core::ptr::addr_of_mut!(val), (0 as i32)); v = assigned; assigned };
vm_block = 98; continue;
}
// C line 49895
100 => {
vm_block = 7; continue;
}
// C line 49894
101 => {
vm_block = if (JS_IsException(sep1)) != 0 { 100 } else { 99 }; continue;
}
// C line 49893
102 => {
let _ = { let assigned = js_new_string8(ctx, c" ".as_ptr()); sep1 = assigned; assigned };
vm_block = 101; continue;
}
// C line 49892
103 => {
vm_block = 7; continue;
}
// C line 49891
104 => {
vm_block = if (JS_IsException(sep)) != 0 { 103 } else { 102 }; continue;
}
// C line 49890
105 => {
let _ = { let assigned = JS_ConcatString3(ctx, c"\n".as_ptr(), JS_DupValue(ctx, indent1), c"".as_ptr()); sep = assigned; assigned };
vm_block = 104; continue;
}
// C line 49898
106 => {
let _ = { let assigned = JS_DupValue(ctx, (*(jsc)).empty); sep1 = assigned; assigned };
vm_block = 99; continue;
}
// C line 49897
107 => {
let _ = { let assigned = JS_DupValue(ctx, (*(jsc)).empty); sep = assigned; assigned };
vm_block = 106; continue;
}
// C line 49889
108 => {
vm_block = if ((!((JS_IsEmptyString((*(jsc)).gap)) != 0) as i32)) != 0 { 105 } else { 107 }; continue;
}
// C line 49888
109 => {
vm_block = 7; continue;
}
// C line 49887
110 => {
vm_block = if (JS_IsException(indent1)) != 0 { 109 } else { 108 }; continue;
}
// C line 49886
111 => {
let _ = { let assigned = JS_ConcatString(ctx, JS_DupValue(ctx, indent), JS_DupValue(ctx, (*(jsc)).gap)); indent1 = assigned; assigned };
vm_block = 110; continue;
}
// C line 49884
112 => {
vm_block = 7; continue;
}
// C line 49883
113 => {
let _ = JS_ThrowTypeError(ctx, c"circular reference".as_ptr());
vm_block = 112; continue;
}
// C line 49882
114 => {
vm_block = if (JS_ToBoolFree(ctx, v)) != 0 { 113 } else { 111 }; continue;
}
// C line 49881
115 => {
vm_block = 7; continue;
}
// C line 49880
116 => {
vm_block = if (JS_IsException(v)) != 0 { 115 } else { 114 }; continue;
}
// C line 49879
117 => {
let _ = { let assigned = js_array_includes(ctx, (*(jsc)).stack, (1 as i32), core::ptr::addr_of_mut!(val)); v = assigned; assigned };
vm_block = 116; continue;
}
// C line 49860
118 => {
vm_block = 17; continue;
}
// C line 49859
119 => {
vm_block = 7; continue;
}
// C line 49858
120 => {
vm_block = if (JS_IsException(val)) != 0 { 119 } else { 118 }; continue;
}
// C line 49857
121 => {
let _ = { let assigned = JS_ToStringFree(ctx, val); val = assigned; assigned };
vm_block = 120; continue;
}
// C line 49865
122 => {
vm_block = 17; continue;
}
// C line 49864
123 => {
vm_block = 7; continue;
}
// C line 49863
124 => {
vm_block = if (JS_IsException(val)) != 0 { 123 } else { 122 }; continue;
}
// C line 49862
125 => {
let _ = { let assigned = JS_ToNumberFree(ctx, val); val = assigned; assigned };
vm_block = 124; continue;
}
// C line 49869
126 => {
vm_block = 17; continue;
}
// C line 49868
127 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(val), JS_DupValue(ctx, ((*(p)).u).object_data));
vm_block = 126; continue;
}
// C line 49877
128 => {
vm_block = 12; continue;
}
// C line 49876
129 => {
let _ = { let assigned = val1; val = assigned; assigned };
vm_block = 128; continue;
}
// C line 49875
130 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 129; continue;
}
// C line 49874
131 => {
vm_block = 7; continue;
}
// C line 49873
132 => {
vm_block = if (JS_IsException(val1)) != 0 { 131 } else { 130 }; continue;
}
// C line 49872
133 => {
let _ = { let assigned = JS_GetProperty(ctx, val, (((crate::quickjs_atom::JS_ATOM_rawJSON as i32)) as JSAtom)); val1 = assigned; assigned };
vm_block = 132; continue;
}
// C line 49870
134 => {
vm_block = if ((((cl) == ((JS_CLASS_RAWJSON as i32))) as i32)) != 0 { 133 } else { 117 }; continue;
}
// C line 49866
135 => {
vm_block = if ((((((((cl) == ((JS_CLASS_BOOLEAN as i32))) as i32)) != 0) || (((((cl) == ((JS_CLASS_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0 { 127 } else { 134 }; continue;
}
// C line 49861
136 => {
vm_block = if ((((cl) == ((JS_CLASS_NUMBER as i32))) as i32)) != 0 { 125 } else { 135 }; continue;
}
// C line 49856
137 => {
vm_block = if ((((cl) == ((JS_CLASS_STRING as i32))) as i32)) != 0 { 121 } else { 136 }; continue;
}
// C line 49855
138 => {
let _ = { let assigned = (((*(p)).class_id) as i32); cl = assigned; assigned };
vm_block = 137; continue;
}
// C line 49854
139 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 138; continue;
}
// C line 49853
140 => {
vm_block = if (JS_IsObject(val)) != 0 { 139 } else { 17 }; continue;
}
// C line 49850
141 => {
vm_block = 7; continue;
}
// C line 49849
142 => {
let _ = JS_ThrowStackOverflow(ctx);
vm_block = 141; continue;
}
// C line 49848
143 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 142 } else { 140 }; continue;
}
// C line 49846
144 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; prop = assigned; assigned };
vm_block = 143; continue;
}
// C line 49845
145 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; tab = assigned; assigned };
vm_block = 144; continue;
}
// C line 49844
146 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; sep1 = assigned; assigned };
vm_block = 145; continue;
}
// C line 49843
147 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; sep = assigned; assigned };
vm_block = 146; continue;
}
// C line 49842
148 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; indent1 = assigned; assigned };
vm_block = 147; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50024. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_JSONStringify(mut ctx: *mut JSContext, mut obj: JSValue, mut replacer: JSValue, mut space0: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut jsc_s: JSONStringifyContext = core::mem::zeroed();
let mut jsc: *mut JSONStringifyContext = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut space: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut wrapper: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut j: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut present: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut p_1: *mut JSObject = core::mem::zeroed();
let mut n_1: i32 = core::mem::zeroed();
let mut p_2: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 97;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50155
1 => {
return ret;
}
// C line 50154
2 => {
let _ = JS_FreeValue(ctx, (*(jsc)).stack);
vm_block = 1; continue;
}
// C line 50153
3 => {
let _ = JS_FreeValue(ctx, (*(jsc)).property_list);
vm_block = 2; continue;
}
// C line 50152
4 => {
let _ = JS_FreeValue(ctx, (*(jsc)).gap);
vm_block = 3; continue;
}
// C line 50151
5 => {
let _ = JS_FreeValue(ctx, (*(jsc)).empty);
vm_block = 4; continue;
}
// C line 50150 labels: done
6 => {
let _ = JS_FreeValue(ctx, wrapper);
vm_block = 5; continue;
}
// C line 50148 labels: done1
7 => {
let _ = string_buffer_free((*(jsc)).b);
vm_block = 6; continue;
}
// C line 50146 labels: exception
8 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 50143
9 => {
vm_block = 6; continue;
}
// C line 50142
10 => {
let _ = { let assigned = string_buffer_end((*(jsc)).b); ret = assigned; assigned };
vm_block = 9; continue;
}
// C line 50140
11 => {
vm_block = 8; continue;
}
// C line 50139
12 => {
vm_block = if (js_json_to_str(ctx, jsc, wrapper, val, (*(jsc)).empty)) != 0 { 11 } else { 10 }; continue;
}
// C line 50137
13 => {
vm_block = 7; continue;
}
// C line 50136
14 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 13; continue;
}
// C line 50135
15 => {
vm_block = if (JS_IsUndefined(val)) != 0 { 14 } else { 12 }; continue;
}
// C line 50134
16 => {
vm_block = 8; continue;
}
// C line 50133
17 => {
vm_block = if (JS_IsException(val)) != 0 { 16 } else { 15 }; continue;
}
// C line 50132
18 => {
let _ = { let assigned = js_json_check(ctx, jsc, wrapper, val, (*(jsc)).empty); val = assigned; assigned };
vm_block = 17; continue;
}
// C line 50130
19 => {
let _ = { let assigned = JS_DupValue(ctx, obj); val = assigned; assigned };
vm_block = 18; continue;
}
// C line 50129
20 => {
vm_block = 8; continue;
}
// C line 50127
21 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, wrapper, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom), JS_DupValue(ctx, obj), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 50126
22 => {
vm_block = 8; continue;
}
// C line 50125
23 => {
vm_block = if (JS_IsException(wrapper)) != 0 { 22 } else { 21 }; continue;
}
// C line 50124
24 => {
let _ = { let assigned = JS_NewObject(ctx); wrapper = assigned; assigned };
vm_block = 23; continue;
}
// C line 50123
25 => {
vm_block = 8; continue;
}
// C line 50122
26 => {
vm_block = if (JS_IsException((*(jsc)).gap)) != 0 { 25 } else { 24 }; continue;
}
// C line 50121
27 => {
let _ = JS_FreeValue(ctx, space);
vm_block = 26; continue;
}
// C line 50114
28 => {
let _ = { let assigned = js_new_string8_len(ctx, c"          ".as_ptr(), n_1); (*(jsc)).gap = assigned; assigned };
vm_block = 27; continue;
}
// C line 50113
29 => {
vm_block = 8; continue;
}
// C line 50112
30 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(n_1), space, (0 as i32), (10 as i32), (0 as i32))) != 0 { 29 } else { 28 }; continue;
}
// C line 50117
31 => {
let _ = { let assigned = js_sub_string(ctx, p_2, (0 as i32), crate::cutils_header::min_int((((*(p_2)).len()) as i32), (10 as i32))); (*(jsc)).gap = assigned; assigned };
vm_block = 27; continue;
}
// C line 50116
32 => {
p_2 = ((((space).u).ptr) as *mut JSString);
vm_block = 31; continue;
}
// C line 50119
33 => {
let _ = { let assigned = JS_DupValue(ctx, (*(jsc)).empty); (*(jsc)).gap = assigned; assigned };
vm_block = 27; continue;
}
// C line 50115
34 => {
vm_block = if (JS_IsString(space)) != 0 { 32 } else { 33 }; continue;
}
// C line 50110
35 => {
vm_block = if (JS_IsNumber(space)) != 0 { 30 } else { 34 }; continue;
}
// C line 50107
36 => {
vm_block = 8; continue;
}
// C line 50106
37 => {
let _ = JS_FreeValue(ctx, space);
vm_block = 36; continue;
}
// C line 50105
38 => {
vm_block = if (JS_IsException(space)) != 0 { 37 } else { 35 }; continue;
}
// C line 50101
39 => {
let _ = { let assigned = JS_ToNumberFree(ctx, space); space = assigned; assigned };
vm_block = 38; continue;
}
// C line 50103
40 => {
let _ = { let assigned = JS_ToStringFree(ctx, space); space = assigned; assigned };
vm_block = 38; continue;
}
// C line 50102
41 => {
vm_block = if (((((((*(p_1)).class_id) as i32)) == ((JS_CLASS_STRING as i32))) as i32)) != 0 { 40 } else { 38 }; continue;
}
// C line 50100
42 => {
vm_block = if (((((((*(p_1)).class_id) as i32)) == ((JS_CLASS_NUMBER as i32))) as i32)) != 0 { 39 } else { 41 }; continue;
}
// C line 50099
43 => {
p_1 = ((((space).u).ptr) as *mut JSObject);
vm_block = 42; continue;
}
// C line 50098
44 => {
vm_block = if (JS_IsObject(space)) != 0 { 43 } else { 35 }; continue;
}
// C line 50097
45 => {
let _ = { let assigned = JS_DupValue(ctx, space0); space = assigned; assigned };
vm_block = 44; continue;
}
// C line 50047
46 => {
let _ = { let assigned = replacer; (*(jsc)).replacer_func = assigned; assigned };
vm_block = 45; continue;
}
// C line 50059
47 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 73 } else { 45 }; continue;
}
// C line 50059
48 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 47; continue;
}
// C line 50090
49 => {
let _ = JS_SetPropertyInt64(ctx, (*(jsc)).property_list, { let old = j; j = (j).wrapping_add(1); old }, v);
vm_block = 48; continue;
}
// C line 50092
50 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 48; continue;
}
// C line 50089
51 => {
vm_block = if ((!((JS_ToBoolFree(ctx, present)) != 0) as i32)) != 0 { 49 } else { 50 }; continue;
}
// C line 50087
52 => {
vm_block = 8; continue;
}
// C line 50086
53 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 52; continue;
}
// C line 50085
54 => {
vm_block = if (JS_IsException(present)) != 0 { 53 } else { 51 }; continue;
}
// C line 50083
55 => {
let _ = { let assigned = js_array_includes(ctx, (*(jsc)).property_list, (1 as i32), core::ptr::addr_of_mut!(v)); present = assigned; assigned };
vm_block = 54; continue;
}
// C line 50070
56 => {
vm_block = 8; continue;
}
// C line 50069
57 => {
vm_block = if (JS_IsException(v)) != 0 { 56 } else { 55 }; continue;
}
// C line 50068
58 => {
let _ = { let assigned = JS_ToStringFree(ctx, v); v = assigned; assigned };
vm_block = 57; continue;
}
// C line 50073
59 => {
vm_block = 48; continue;
}
// C line 50072
60 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 59; continue;
}
// C line 50066
61 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_STRING as i32))) as i32)) != 0) || ((((((((*(p)).class_id) as i32)) == ((JS_CLASS_NUMBER as i32))) as i32)) != 0)) as i32)) != 0 { 58 } else { 60 }; continue;
}
// C line 50065
62 => {
p = ((((v).u).ptr) as *mut JSObject);
vm_block = 61; continue;
}
// C line 50078
63 => {
vm_block = 8; continue;
}
// C line 50077
64 => {
vm_block = if (JS_IsException(v)) != 0 { 63 } else { 55 }; continue;
}
// C line 50076
65 => {
let _ = { let assigned = JS_ToStringFree(ctx, v); v = assigned; assigned };
vm_block = 64; continue;
}
// C line 50081
66 => {
vm_block = 48; continue;
}
// C line 50080
67 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 66; continue;
}
// C line 50079
68 => {
vm_block = if ((!((JS_IsString(v)) != 0) as i32)) != 0 { 67 } else { 55 }; continue;
}
// C line 50075
69 => {
vm_block = if (JS_IsNumber(v)) != 0 { 65 } else { 68 }; continue;
}
// C line 50064
70 => {
vm_block = if (JS_IsObject(v)) != 0 { 62 } else { 69 }; continue;
}
// C line 50063
71 => {
vm_block = 8; continue;
}
// C line 50062
72 => {
vm_block = if (JS_IsException(v)) != 0 { 71 } else { 70 }; continue;
}
// C line 50061
73 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, replacer, i); v = assigned; assigned };
vm_block = 72; continue;
}
// C line 50059
74 => {
let _ = { let assigned = { let assigned = (((0 as i32)) as i64); j = assigned; assigned }; i = assigned; assigned };
vm_block = 47; continue;
}
// C line 50058
75 => {
vm_block = 8; continue;
}
// C line 50057
76 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(n), replacer)) != 0 { 75 } else { 74 }; continue;
}
// C line 50056
77 => {
vm_block = 8; continue;
}
// C line 50055
78 => {
vm_block = if (JS_IsException((*(jsc)).property_list)) != 0 { 77 } else { 76 }; continue;
}
// C line 50054
79 => {
let _ = { let assigned = JS_NewArray(ctx); (*(jsc)).property_list = assigned; assigned };
vm_block = 78; continue;
}
// C line 50052
80 => {
vm_block = if (res) != 0 { 79 } else { 45 }; continue;
}
// C line 50051
81 => {
vm_block = 8; continue;
}
// C line 50050
82 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 81 } else { 80 }; continue;
}
// C line 50049
83 => {
let _ = { let assigned = JS_IsArray(ctx, replacer); res = assigned; assigned };
vm_block = 82; continue;
}
// C line 50046
84 => {
vm_block = if (JS_IsFunction(ctx, replacer)) != 0 { 46 } else { 83 }; continue;
}
// C line 50045
85 => {
vm_block = 8; continue;
}
// C line 50044
86 => {
vm_block = if (JS_IsException((*(jsc)).stack)) != 0 { 85 } else { 84 }; continue;
}
// C line 50043
87 => {
let _ = { let assigned = JS_NewArray(ctx); (*(jsc)).stack = assigned; assigned };
vm_block = 86; continue;
}
// C line 50042
88 => {
let _ = string_buffer_init(ctx, (*(jsc)).b, (0 as i32));
vm_block = 87; continue;
}
// C line 50040
89 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; wrapper = assigned; assigned };
vm_block = 88; continue;
}
// C line 50039
90 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 89; continue;
}
// C line 50038
91 => {
let _ = { let assigned = JS_AtomToString(ctx, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom)); (*(jsc)).empty = assigned; assigned };
vm_block = 90; continue;
}
// C line 50037
92 => {
let _ = { let assigned = core::ptr::addr_of_mut!(b_s); (*(jsc)).b = assigned; assigned };
vm_block = 91; continue;
}
// C line 50036
93 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(jsc)).gap = assigned; assigned };
vm_block = 92; continue;
}
// C line 50035
94 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(jsc)).property_list = assigned; assigned };
vm_block = 93; continue;
}
// C line 50034
95 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(jsc)).stack = assigned; assigned };
vm_block = 94; continue;
}
// C line 50033
96 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(jsc)).replacer_func = assigned; assigned };
vm_block = 95; continue;
}
// C line 50028
97 => {
jsc = core::ptr::addr_of_mut!(jsc_s);
vm_block = 96; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50158. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_json_stringify(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50162
1 => {
return JS_JSONStringify(ctx, *(argv).offset(((0 as i32)) as isize), *(argv).offset(((1 as i32)) as isize), *(argv).offset(((2 as i32)) as isize));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50177. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_AddIntrinsicJSON(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50180
1 => {
return JS_SetPropertyFunctionList(ctx, (*(ctx)).global_obj, (js_json_obj).as_ptr(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32));
}
_ => std::process::abort(),
} }
}
