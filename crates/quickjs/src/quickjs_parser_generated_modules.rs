// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29680. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_export_entry2(mut ctx: *mut JSContext, mut s: *mut JSParseState, mut m: *mut JSModuleDef, mut local_name: JSAtom, mut export_name: JSAtom, mut export_type: JSExportTypeEnum) -> *mut JSExportEntry {
let mut vm_local_storage = Vec::<u64>::new();
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut buf1: [c_char; 64] = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29708
1 => {
return me;
}
// C line 29707
2 => {
let _ = { let assigned = export_type; (*(me)).export_type = assigned; assigned };
vm_block = 1; continue;
}
// C line 29706
3 => {
let _ = { let assigned = JS_DupAtom(ctx, export_name); (*(me)).export_name = assigned; assigned };
vm_block = 2; continue;
}
// C line 29705
4 => {
let _ = { let assigned = JS_DupAtom(ctx, local_name); (*(me)).local_name = assigned; assigned };
vm_block = 3; continue;
}
// C line 29704
5 => {
let _ = { let dst = (((me) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<JSExportEntry>() as usize)) as usize); dst as *mut c_void };
vm_block = 4; continue;
}
// C line 29703
6 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset(({ let old = (*(m)).export_entries_count; (*(m)).export_entries_count = ((*(m)).export_entries_count).wrapping_add(1); old }) as isize)); me = assigned; assigned };
vm_block = 5; continue;
}
// C line 29702
7 => {
return core::ptr::null_mut::<JSExportEntry>();
}
// C line 29698
8 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(m)).export_entries)) as *mut *mut c_void), (((size_of::<JSExportEntry>() as usize)) as i32), core::ptr::addr_of_mut!((*(m)).export_entries_size), ((*(m)).export_entries_count).wrapping_add((1 as i32)))) != 0 { 7 } else { 6 }; continue;
}
// C line 29695
9 => {
return core::ptr::null_mut::<JSExportEntry>();
}
// C line 29690
10 => {
let _ = js_parse_error_cargs(s, c"duplicate exported name '%s'".as_ptr(), &[ParserFormatArg::C((JS_AtomGetStr(ctx, (buf1).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), export_name)) as *const c_char)]);
vm_block = 9; continue;
}
// C line 29693
11 => {
let _ = __JS_ThrowSyntaxErrorAtom(ctx, export_name, c"duplicate exported name '%s'".as_ptr());
vm_block = 9; continue;
}
// C line 29689
12 => {
vm_block = if !(s).is_null() { 10 } else { 11 }; continue;
}
// C line 29687
13 => {
vm_block = if !(find_export_entry(ctx, m, export_name)).is_null() { 12 } else { 8 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29711. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_export_entry(mut s: *mut JSParseState, mut m: *mut JSModuleDef, mut local_name: JSAtom, mut export_name: JSAtom, mut export_type: JSExportTypeEnum) -> *mut JSExportEntry {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29715
1 => {
return add_export_entry2((*(s)).ctx, s, m, local_name, export_name, export_type);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29719. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_star_export_entry(mut ctx: *mut JSContext, mut m: *mut JSModuleDef, mut req_module_idx: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut se: *mut JSStarExportEntry = core::ptr::null_mut();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29731
1 => {
return (0 as i32);
}
// C line 29730
2 => {
let _ = { let assigned = req_module_idx; (*(se)).req_module_idx = assigned; assigned };
vm_block = 1; continue;
}
// C line 29729
3 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(m)).star_export_entries).offset(({ let old = (*(m)).star_export_entries_count; (*(m)).star_export_entries_count = ((*(m)).star_export_entries_count).wrapping_add(1); old }) as isize)); se = assigned; assigned };
vm_block = 2; continue;
}
// C line 29728
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29724
5 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(m)).star_export_entries)) as *mut *mut c_void), (((size_of::<JSStarExportEntry>() as usize)) as i32), core::ptr::addr_of_mut!((*(m)).star_export_entries_size), ((*(m)).star_export_entries_count).wrapping_add((1 as i32)))) != 0 { 4 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29735. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_NewCModule(mut ctx: *mut JSContext, mut name_str: *const c_char, mut func: Option<JSModuleInitFunc>) -> *mut JSModuleDef {
let mut vm_local_storage = Vec::<u64>::new();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut name: JSAtom = 0;
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29747
1 => {
return m;
}
// C line 29746
2 => {
let _ = { let assigned = func; (*(m)).init_func = assigned; assigned };
vm_block = 1; continue;
}
// C line 29745
3 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29744
4 => {
vm_block = if ((!(!(m).is_null()) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 29743
5 => {
let _ = { let assigned = js_new_module_def(ctx, name); m = assigned; assigned };
vm_block = 4; continue;
}
// C line 29742
6 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29741
7 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 29740
8 => {
let _ = { let assigned = JS_NewAtom(ctx, name_str); name = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29750. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_AddModuleExport(mut ctx: *mut JSContext, mut m: *mut JSModuleDef, mut export_name: *const c_char) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut name: JSAtom = 0;
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29761
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29763
2 => {
return (0 as i32);
}
// C line 29760
3 => {
vm_block = if ((!(!(me).is_null()) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 29759
4 => {
let _ = JS_FreeAtom(ctx, name);
vm_block = 3; continue;
}
// C line 29757
5 => {
let _ = { let assigned = add_export_entry2(ctx, core::ptr::null_mut::<JSParseState>(), m, (((0 as i32)) as JSAtom), name, (((JS_EXPORT_TYPE_LOCAL as i32)) as JSExportTypeEnum)); me = assigned; assigned };
vm_block = 4; continue;
}
// C line 29756
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29755
7 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 29754
8 => {
let _ = { let assigned = JS_NewAtom(ctx, export_name); name = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29766. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_SetModuleExport(mut ctx: *mut JSContext, mut m: *mut JSModuleDef, mut export_name: *const c_char, mut val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut name: JSAtom = 0;
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29782
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29781 labels: fail
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 29779
3 => {
return (0 as i32);
}
// C line 29778
4 => {
let _ = set_value(ctx, (*((((*(me)).u).local).var_ref)).pvalue, val);
vm_block = 3; continue;
}
// C line 29777
5 => {
vm_block = 2; continue;
}
// C line 29776
6 => {
vm_block = if ((!(!(me).is_null()) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 29775
7 => {
let _ = JS_FreeAtom(ctx, name);
vm_block = 6; continue;
}
// C line 29774
8 => {
let _ = { let assigned = find_export_entry(ctx, m, name); me = assigned; assigned };
vm_block = 7; continue;
}
// C line 29773
9 => {
vm_block = 2; continue;
}
// C line 29772
10 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 29771
11 => {
let _ = { let assigned = JS_NewAtom(ctx, export_name); name = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29821. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_default_module_normalize_name(mut ctx: *mut JSContext, mut base_name: *const c_char, mut name: *const c_char) -> *mut c_char {
let mut vm_local_storage = Vec::<u64>::new();
let mut filename: *mut c_char = core::ptr::null_mut();
let mut p: *mut c_char = core::ptr::null_mut();
let mut r: *const c_char = core::ptr::null();
let mut cap: i32 = 0;
let mut len: i32 = 0;
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29877
1 => {
return filename;
}
// C line 29875
2 => {
let _ = crate::cutils::pstrcat(filename, cap, r);
vm_block = 1; continue;
}
// C line 29874
3 => {
let _ = crate::cutils::pstrcat(filename, cap, c"/".as_ptr());
vm_block = 2; continue;
}
// C line 29873
4 => {
vm_block = if ((((((*(filename).offset(((0 as i32)) as isize)) as i32)) != ((0 as i32))) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 29850
5 => {
vm_block = 21; continue;
}
// C line 29852
6 => {
let _ = { r = ((((r) as *const c_char)).offset((((2 as i32)) as isize))) as *const c_char; r };
vm_block = 5; continue;
}
// C line 29868
7 => {
let _ = { r = ((((r) as *const c_char)).offset((((3 as i32)) as isize))) as *const c_char; r };
vm_block = 5; continue;
}
// C line 29867
8 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(p) = assigned; assigned };
vm_block = 7; continue;
}
// C line 29866
9 => {
let _ = { let old = p; p = (p).offset(-1); old };
vm_block = 8; continue;
}
// C line 29865
10 => {
vm_block = if ((((p) > (filename)) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 29864
11 => {
vm_block = 4; continue;
}
// C line 29863
12 => {
vm_block = if ((((((!((parser_strcmp(p, c".".as_ptr())) != 0) as i32)) != 0) || (((!((parser_strcmp(p, c"..".as_ptr())) != 0) as i32)) != 0)) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 29860
13 => {
let _ = { let assigned = filename; p = assigned; assigned };
vm_block = 12; continue;
}
// C line 29862
14 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 12; continue;
}
// C line 29859
15 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 13 } else { 14 }; continue;
}
// C line 29858
16 => {
let _ = { let assigned = parser_strrchr(filename, (47 as i32)); p = assigned; assigned };
vm_block = 15; continue;
}
// C line 29857
17 => {
vm_block = 4; continue;
}
// C line 29856
18 => {
vm_block = if ((((((*(filename).offset(((0 as i32)) as isize)) as i32)) == ((0 as i32))) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 29870
19 => {
vm_block = 4; continue;
}
// C line 29853
20 => {
vm_block = if ((((((((((((((*(r).offset(((0 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0) && (((((((*(r).offset(((1 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(r).offset(((2 as i32)) as isize)) as i32)) == ((47 as i32))) as i32)) != 0)) as i32)) != 0 { 18 } else { 19 }; continue;
}
// C line 29851
21 => {
vm_block = if ((((((((((*(r).offset(((0 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0) && (((((((*(r).offset(((1 as i32)) as isize)) as i32)) == ((47 as i32))) as i32)) != 0)) as i32)) != 0 { 6 } else { 20 }; continue;
}
// C line 29849
22 => {
let _ = { let assigned = name; r = assigned; assigned };
vm_block = 5; continue;
}
// C line 29846
23 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(filename).offset((len) as isize) = assigned; assigned };
vm_block = 22; continue;
}
// C line 29845
24 => {
let _ = { let dst = (((filename) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((base_name) as *const c_void)) as *const u8, dst, (((len) as usize)) as usize); dst as *mut c_void };
vm_block = 23; continue;
}
// C line 29844
25 => {
return core::ptr::null_mut::<c_char>();
}
// C line 29843
26 => {
vm_block = if ((!(!(filename).is_null()) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 29842
27 => {
let _ = { let assigned = ((js_malloc(ctx, ((cap) as usize))) as *mut c_char); filename = assigned; assigned };
vm_block = 26; continue;
}
// C line 29841
28 => {
let _ = { let assigned = (((((((len) as usize)).wrapping_add(parser_strlen(name))).wrapping_add((((1 as i32)) as usize))).wrapping_add((((1 as i32)) as usize))) as i32); cap = assigned; assigned };
vm_block = 27; continue;
}
// C line 29837
29 => {
let _ = { let assigned = ((((p).offset_from(base_name) as i64)) as i32); len = assigned; assigned };
vm_block = 28; continue;
}
// C line 29839
30 => {
let _ = { let assigned = (0 as i32); len = assigned; assigned };
vm_block = 28; continue;
}
// C line 29836
31 => {
vm_block = if !(p).is_null() { 29 } else { 30 }; continue;
}
// C line 29835
32 => {
let _ = { let assigned = parser_strrchr(base_name, (47 as i32)); p = assigned; assigned };
vm_block = 31; continue;
}
// C line 29832
33 => {
return js_strdup(ctx, name);
}
// C line 29830
34 => {
vm_block = if ((((((*(name).offset(((0 as i32)) as isize)) as i32)) != ((46 as i32))) as i32)) != 0 { 33 } else { 32 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29880. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_find_loaded_module(mut ctx: *mut JSContext, mut name: JSAtom) -> *mut JSModuleDef {
let mut vm_local_storage = Vec::<u64>::new();
let mut el: *mut list_head = core::ptr::null_mut();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29891
1 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29886
2 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(ctx)).loaded_modules))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 29886
3 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 29889
4 => {
return m;
}
// C line 29888
5 => {
vm_block = if (((((*(m)).module_name) == (name)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 29887
6 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSModuleDef, link) as usize)) as isize))) as *mut JSModuleDef); m = assigned; assigned };
vm_block = 5; continue;
}
// C line 29886
7 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(ctx)).loaded_modules))).next; el = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29895. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_host_resolve_imported_module(mut ctx: *mut JSContext, mut base_cname: *const c_char, mut cname1: *const c_char, mut attributes: JSValue) -> *mut JSModuleDef {
let mut vm_local_storage = Vec::<u64>::new();
let mut rt: *mut JSRuntime = core::ptr::null_mut();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut cname: *mut c_char = core::ptr::null_mut();
let mut module_name: JSAtom = 0;
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29944
1 => {
return m;
}
// C line 29943
2 => {
let _ = js_free(ctx, ((cname) as *mut c_void));
vm_block = 1; continue;
}
// C line 29939
3 => {
let _ = { let assigned = (((*(rt)).u).module_loader_func2).expect("registered parser callback")(ctx, cname, (*(rt)).module_loader_opaque, attributes); m = assigned; assigned };
vm_block = 2; continue;
}
// C line 29941
4 => {
let _ = { let assigned = (((*(rt)).u).module_loader_func).expect("registered parser callback")(ctx, cname, (*(rt)).module_loader_opaque); m = assigned; assigned };
vm_block = 2; continue;
}
// C line 29938
5 => {
vm_block = if ((*(rt)).module_loader_has_attr) != 0 { 3 } else { 4 }; continue;
}
// C line 29936
6 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29935
7 => {
let _ = js_free(ctx, ((cname) as *mut c_void));
vm_block = 6; continue;
}
// C line 29933
8 => {
let _ = JS_ThrowReferenceError_cargs(ctx, c"could not load module '%s'".as_ptr(), &[ParserFormatArg::C((cname) as *const c_char)]);
vm_block = 7; continue;
}
// C line 29931
9 => {
vm_block = if ((!((((*(rt)).u).module_loader_func).is_some()) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line 29928
10 => {
let _ = JS_FreeAtom(ctx, module_name);
vm_block = 9; continue;
}
// C line 29925
11 => {
return m;
}
// C line 29924
12 => {
let _ = JS_FreeAtom(ctx, module_name);
vm_block = 11; continue;
}
// C line 29923
13 => {
let _ = js_free(ctx, ((cname) as *mut c_void));
vm_block = 12; continue;
}
// C line 29922
14 => {
vm_block = if !(m).is_null() { 13 } else { 10 }; continue;
}
// C line 29921
15 => {
let _ = { let assigned = js_find_loaded_module(ctx, module_name); m = assigned; assigned };
vm_block = 14; continue;
}
// C line 29917
16 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29916
17 => {
let _ = js_free(ctx, ((cname) as *mut c_void));
vm_block = 16; continue;
}
// C line 29915
18 => {
vm_block = if ((((module_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 17 } else { 15 }; continue;
}
// C line 29914
19 => {
let _ = { let assigned = JS_NewAtom(ctx, cname); module_name = assigned; assigned };
vm_block = 18; continue;
}
// C line 29912
20 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29911
21 => {
vm_block = if ((!(!(cname).is_null()) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 29906
22 => {
let _ = { let assigned = js_default_module_normalize_name(ctx, base_cname, cname1); cname = assigned; assigned };
vm_block = 21; continue;
}
// C line 29908
23 => {
let _ = { let assigned = ((*(rt)).module_normalize_func).expect("registered parser callback")(ctx, base_cname, cname1, (*(rt)).module_loader_opaque); cname = assigned; assigned };
vm_block = 21; continue;
}
// C line 29905
24 => {
vm_block = if ((!(((*(rt)).module_normalize_func).is_some()) as i32)) != 0 { 22 } else { 23 }; continue;
}
// C line 29900
25 => {
rt = (*(ctx)).rt;
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29947. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_host_resolve_imported_module_atom(mut ctx: *mut JSContext, mut base_module_name: JSAtom, mut module_name1: JSAtom, mut attributes: JSValue) -> *mut JSModuleDef {
let mut vm_local_storage = Vec::<u64>::new();
let mut base_cname: *const c_char = core::ptr::null();
let mut cname: *const c_char = core::ptr::null();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29966
1 => {
return m;
}
// C line 29965
2 => {
let _ = JS_FreeCString(ctx, cname);
vm_block = 1; continue;
}
// C line 29964
3 => {
let _ = JS_FreeCString(ctx, base_cname);
vm_block = 2; continue;
}
// C line 29963
4 => {
let _ = { let assigned = js_host_resolve_imported_module(ctx, base_cname, cname, attributes); m = assigned; assigned };
vm_block = 3; continue;
}
// C line 29961
5 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29960
6 => {
let _ = JS_FreeCString(ctx, base_cname);
vm_block = 5; continue;
}
// C line 29959
7 => {
vm_block = if ((!(!(cname).is_null()) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 29958
8 => {
let _ = { let assigned = JS_AtomToCString(ctx, module_name1); cname = assigned; assigned };
vm_block = 7; continue;
}
// C line 29957
9 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29956
10 => {
vm_block = if ((!(!(base_cname).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 29955
11 => {
let _ = { let assigned = JS_AtomToCString(ctx, base_module_name); base_cname = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29980. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_resolve_entry(mut s: *mut JSResolveState, mut m: *mut JSModuleDef, mut name: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut re: *mut JSResolveEntry = core::ptr::null_mut();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29989
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29984
2 => {
vm_block = if ((((i) < ((*(s)).count)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 29984
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 29987
4 => {
return i;
}
// C line 29986
5 => {
vm_block = if (((((((((*(re)).module) == (m)) as i32)) != 0) && ((((((*(re)).name) == (name)) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 29985
6 => {
re = core::ptr::addr_of_mut!(*((*(s)).array).offset((i) as isize));
vm_block = 5; continue;
}
// C line 29984
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29992. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_resolve_entry(mut ctx: *mut JSContext, mut s: *mut JSResolveState, mut m: *mut JSModuleDef, mut name: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut re: *mut JSResolveEntry = core::ptr::null_mut();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30004
1 => {
return (0 as i32);
}
// C line 30003
2 => {
let _ = { let assigned = JS_DupAtom(ctx, name); (*(re)).name = assigned; assigned };
vm_block = 1; continue;
}
// C line 30002
3 => {
let _ = { let assigned = m; (*(re)).module = assigned; assigned };
vm_block = 2; continue;
}
// C line 30001
4 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).array).offset(({ let old = (*(s)).count; (*(s)).count = ((*(s)).count).wrapping_add(1); old }) as isize)); re = assigned; assigned };
vm_block = 3; continue;
}
// C line 30000
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29997
6 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(s)).array)) as *mut *mut c_void), (((size_of::<JSResolveEntry>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).size), ((*(s)).count).wrapping_add((1 as i32)))) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30015. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_resolve_export1(mut ctx: *mut JSContext, mut pmodule: *mut *mut JSModuleDef, mut pme: *mut *mut JSExportEntry, mut m: *mut JSModuleDef, mut export_name: JSAtom, mut s: *mut JSResolveState) -> JSResolveResultEnum {
let mut vm_local_storage = Vec::<u64>::new();
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut m1: *mut JSModuleDef = core::ptr::null_mut();
let mut i: i32 = 0;
let mut se: *mut JSStarExportEntry = core::ptr::null_mut();
let mut m1_1: *mut JSModuleDef = core::ptr::null_mut();
let mut res_m: *mut JSModuleDef = core::ptr::null_mut();
let mut res_me: *mut JSExportEntry = core::ptr::null_mut();
let mut ret: JSResolveResultEnum = core::mem::zeroed();
let mut vm_block: usize = 38;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30036
1 => {
return (((JS_RESOLVE_RES_FOUND as i32)) as JSResolveResultEnum);
}
// C line 30035
2 => {
let _ = { let assigned = me; *(pme) = assigned; assigned };
vm_block = 1; continue;
}
// C line 30034
3 => {
let _ = { let assigned = m; *(pmodule) = assigned; assigned };
vm_block = 2; continue;
}
// C line 30045
4 => {
return (((JS_RESOLVE_RES_FOUND as i32)) as JSResolveResultEnum);
}
// C line 30044
5 => {
let _ = { let assigned = me; *(pme) = assigned; assigned };
vm_block = 4; continue;
}
// C line 30043
6 => {
let _ = { let assigned = m; *(pmodule) = assigned; assigned };
vm_block = 5; continue;
}
// C line 30047
7 => {
return js_resolve_export1(ctx, pmodule, pme, m1, (*(me)).local_name, s);
}
// C line 30041
8 => {
vm_block = if (((((*(me)).local_name) == ((((crate::quickjs_atom::JS_ATOM__star_ as i32)) as JSAtom))) as i32)) != 0 { 6 } else { 7 }; continue;
}
// C line 30040
9 => {
let _ = { let assigned = (*((*(m)).req_module_entries).offset((((*(me)).u).req_module_idx) as isize)).module; m1 = assigned; assigned };
vm_block = 8; continue;
}
// C line 30032
10 => {
vm_block = if (((((((*(me)).export_type) as u32)) == ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0 { 3 } else { 9 }; continue;
}
// C line 30085
11 => {
return (((JS_RESOLVE_RES_NOT_FOUND as i32)) as JSResolveResultEnum);
}
// C line 30083
12 => {
return (((JS_RESOLVE_RES_FOUND as i32)) as JSResolveResultEnum);
}
// C line 30082
13 => {
vm_block = if ((((*(pme)) != (core::ptr::null_mut::<JSExportEntry>())) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 30056
14 => {
vm_block = if ((((i) < ((*(m)).star_export_entries_count)) as i32)) != 0 { 28 } else { 13 }; continue;
}
// C line 30056
15 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 30067
16 => {
return ret;
}
// C line 30074
17 => {
return (((JS_RESOLVE_RES_AMBIGUOUS as i32)) as JSResolveResultEnum);
}
// C line 30073
18 => {
let _ = { let assigned = core::ptr::null_mut::<JSExportEntry>(); *(pme) = assigned; assigned };
vm_block = 17; continue;
}
// C line 30072
19 => {
let _ = { let assigned = core::ptr::null_mut::<JSModuleDef>(); *(pmodule) = assigned; assigned };
vm_block = 18; continue;
}
// C line 30070
20 => {
vm_block = if ((((((((*(pmodule)) != (res_m)) as i32)) != 0) || ((((((*(res_me)).local_name) != ((*(*(pme))).local_name)) as i32)) != 0)) as i32)) != 0 { 19 } else { 15 }; continue;
}
// C line 30078
21 => {
let _ = { let assigned = res_me; *(pme) = assigned; assigned };
vm_block = 15; continue;
}
// C line 30077
22 => {
let _ = { let assigned = res_m; *(pmodule) = assigned; assigned };
vm_block = 21; continue;
}
// C line 30069
23 => {
vm_block = if ((((*(pme)) != (core::ptr::null_mut::<JSExportEntry>())) as i32)) != 0 { 20 } else { 22 }; continue;
}
// C line 30068
24 => {
vm_block = if ((((((ret) as i32)) == ((JS_RESOLVE_RES_FOUND as i32))) as i32)) != 0 { 23 } else { 15 }; continue;
}
// C line 30065
25 => {
vm_block = if ((((((((((ret) as i32)) == ((JS_RESOLVE_RES_AMBIGUOUS as i32))) as i32)) != 0) || (((((((ret) as i32)) == ((JS_RESOLVE_RES_EXCEPTION as i32))) as i32)) != 0)) as i32)) != 0 { 16 } else { 24 }; continue;
}
// C line 30063
26 => {
let _ = { let assigned = js_resolve_export1(ctx, core::ptr::addr_of_mut!(res_m), core::ptr::addr_of_mut!(res_me), m1_1, export_name, s); ret = assigned; assigned };
vm_block = 25; continue;
}
// C line 30062
27 => {
let _ = { let assigned = (*((*(m)).req_module_entries).offset(((*(se)).req_module_idx) as isize)).module; m1_1 = assigned; assigned };
vm_block = 26; continue;
}
// C line 30057
28 => {
se = core::ptr::addr_of_mut!(*((*(m)).star_export_entries).offset((i) as isize));
vm_block = 27; continue;
}
// C line 30056
29 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 14; continue;
}
// C line 30052
30 => {
vm_block = if ((((export_name) != ((((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom))) as i32)) != 0 { 29 } else { 11 }; continue;
}
// C line 30031
31 => {
vm_block = if !(me).is_null() { 10 } else { 30 }; continue;
}
// C line 30030
32 => {
let _ = { let assigned = find_export_entry(ctx, m, export_name); me = assigned; assigned };
vm_block = 31; continue;
}
// C line 30029
33 => {
return (((JS_RESOLVE_RES_EXCEPTION as i32)) as JSResolveResultEnum);
}
// C line 30028
34 => {
vm_block = if ((((add_resolve_entry(ctx, s, m, export_name)) < ((0 as i32))) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 30027
35 => {
return (((JS_RESOLVE_RES_CIRCULAR as i32)) as JSResolveResultEnum);
}
// C line 30026
36 => {
vm_block = if ((((find_resolve_entry(s, m, export_name)) >= ((0 as i32))) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 30025
37 => {
let _ = { let assigned = core::ptr::null_mut::<JSExportEntry>(); *(pme) = assigned; assigned };
vm_block = 36; continue;
}
// C line 30024
38 => {
let _ = { let assigned = core::ptr::null_mut::<JSModuleDef>(); *(pmodule) = assigned; assigned };
vm_block = 37; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30092. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_resolve_export(mut ctx: *mut JSContext, mut pmodule: *mut *mut JSModuleDef, mut pme: *mut *mut JSExportEntry, mut m: *mut JSModuleDef, mut export_name: JSAtom) -> JSResolveResultEnum {
let mut vm_local_storage = Vec::<u64>::new();
let mut ss: JSResolveState = core::mem::zeroed();
let mut s: *mut JSResolveState = core::ptr::null_mut();
let mut i: i32 = 0;
let mut ret: JSResolveResultEnum = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30112
1 => {
return ret;
}
// C line 30110
2 => {
let _ = js_free(ctx, (((*(s)).array) as *mut c_void));
vm_block = 1; continue;
}
// C line 30108
3 => {
vm_block = if ((((i) < ((*(s)).count)) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line 30108
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 30109
5 => {
let _ = JS_FreeAtom(ctx, (*((*(s)).array).offset((i) as isize)).name);
vm_block = 4; continue;
}
// C line 30108
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 30106
7 => {
let _ = { let assigned = js_resolve_export1(ctx, pmodule, pme, m, export_name, s); ret = assigned; assigned };
vm_block = 6; continue;
}
// C line 30104
8 => {
let _ = { let assigned = (0 as i32); (*(s)).count = assigned; assigned };
vm_block = 7; continue;
}
// C line 30103
9 => {
let _ = { let assigned = (0 as i32); (*(s)).size = assigned; assigned };
vm_block = 8; continue;
}
// C line 30102
10 => {
let _ = { let assigned = core::ptr::null_mut::<JSResolveEntry>(); (*(s)).array = assigned; assigned };
vm_block = 9; continue;
}
// C line 30098
11 => {
s = core::ptr::addr_of_mut!(ss);
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30115. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_resolve_export_throw_error(mut ctx: *mut JSContext, mut res: JSResolveResultEnum, mut m: *mut JSModuleDef, mut export_name: JSAtom) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut buf1: [c_char; 64] = core::mem::zeroed();
let mut buf2: [c_char; 64] = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 30139
1 => {
vm_block = 0; continue;
}
// C line 30136
2 => {
let _ = JS_ThrowSyntaxError_cargs(ctx, c"export '%s' in module '%s' is ambiguous".as_ptr(), &[ParserFormatArg::C((JS_AtomGetStr(ctx, (buf1).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), export_name)) as *const c_char), ParserFormatArg::C((JS_AtomGetStr(ctx, (buf2).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*(m)).module_name)) as *const c_char)]);
vm_block = 1; continue;
}
// C line 30134
3 => {
vm_block = 0; continue;
}
// C line 30131
4 => {
let _ = JS_ThrowSyntaxError_cargs(ctx, c"circular reference when looking for export '%s' in module '%s'".as_ptr(), &[ParserFormatArg::C((JS_AtomGetStr(ctx, (buf1).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), export_name)) as *const c_char), ParserFormatArg::C((JS_AtomGetStr(ctx, (buf2).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*(m)).module_name)) as *const c_char)]);
vm_block = 3; continue;
}
// C line 30129
5 => {
vm_block = 0; continue;
}
// C line 30126
6 => {
let _ = JS_ThrowSyntaxError_cargs(ctx, c"Could not find export '%s' in module '%s'".as_ptr(), &[ParserFormatArg::C((JS_AtomGetStr(ctx, (buf1).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), export_name)) as *const c_char), ParserFormatArg::C((JS_AtomGetStr(ctx, (buf2).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*(m)).module_name)) as *const c_char)]);
vm_block = 5; continue;
}
// C line 30123
7 => {
vm_block = 0; continue;
}
// C line 30121
8 => {
vm_block = match ((res) as i32) { x if x == (JS_RESOLVE_RES_AMBIGUOUS as i32) => 2, x if x == (JS_RESOLVE_RES_CIRCULAR as i32) => 4, x if x == (JS_RESOLVE_RES_NOT_FOUND as i32) => 6, x if x == (JS_RESOLVE_RES_EXCEPTION as i32) => 7, _ => 6, }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30169. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_exported_name(mut s: *mut GetExportNamesState, mut name: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30176
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30172
2 => {
vm_block = if ((((i) < ((*(s)).exported_names_count)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 30172
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 30174
4 => {
return i;
}
// C line 30173
5 => {
vm_block = if (((((*((*(s)).exported_names).offset((i) as isize)).export_name) == (name)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 30172
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30179. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_exported_names(mut ctx: *mut JSContext, mut s: *mut GetExportNamesState, mut m: *mut JSModuleDef, mut from_star: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut en: *mut ExportedNameEntry = core::ptr::null_mut();
let mut i: i32 = 0;
let mut j: i32 = 0;
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut se: *mut JSStarExportEntry = core::ptr::null_mut();
let mut m1: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 33;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30225
1 => {
return (0 as i32);
}
// C line 30218
2 => {
vm_block = if ((((i) < ((*(m)).star_export_entries_count)) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line 30218
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 30223
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30222
5 => {
vm_block = if (get_exported_names(ctx, s, m1, (1 as i32))) != 0 { 4 } else { 3 }; continue;
}
// C line 30221
6 => {
let _ = { let assigned = (*((*(m)).req_module_entries).offset(((*(se)).req_module_idx) as isize)).module; m1 = assigned; assigned };
vm_block = 5; continue;
}
// C line 30219
7 => {
se = core::ptr::addr_of_mut!(*((*(m)).star_export_entries).offset((i) as isize));
vm_block = 6; continue;
}
// C line 30218
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 30196
9 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 24 } else { 8 }; continue;
}
// C line 30196
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 30210
11 => {
let _ = { let assigned = core::ptr::null_mut::<JSExportEntry>(); ((*(en)).u).me = assigned; assigned };
vm_block = 10; continue;
}
// C line 30212
12 => {
let _ = { let assigned = me; ((*(en)).u).me = assigned; assigned };
vm_block = 10; continue;
}
// C line 30209
13 => {
vm_block = if (((((from_star) != 0) || ((((((((*(me)).export_type) as u32)) != ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 11 } else { 12 }; continue;
}
// C line 30207
14 => {
let _ = { let assigned = (*(me)).export_name; (*(en)).export_name = assigned; assigned };
vm_block = 13; continue;
}
// C line 30206
15 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).exported_names).offset(({ let old = (*(s)).exported_names_count; (*(s)).exported_names_count = ((*(s)).exported_names_count).wrapping_add(1); old }) as isize)); en = assigned; assigned };
vm_block = 14; continue;
}
// C line 30205
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30202
17 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(s)).exported_names)) as *mut *mut c_void), (((size_of::<ExportedNameEntry>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).exported_names_size), ((*(s)).exported_names_count).wrapping_add((1 as i32)))) != 0 { 16 } else { 15 }; continue;
}
// C line 30215
18 => {
let _ = { let assigned = core::ptr::null_mut::<JSExportEntry>(); ((*(en)).u).me = assigned; assigned };
vm_block = 10; continue;
}
// C line 30214
19 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).exported_names).offset((j) as isize)); en = assigned; assigned };
vm_block = 18; continue;
}
// C line 30201
20 => {
vm_block = if ((((j) < ((0 as i32))) as i32)) != 0 { 17 } else { 19 }; continue;
}
// C line 30200
21 => {
let _ = { let assigned = find_exported_name(s, (*(me)).export_name); j = assigned; assigned };
vm_block = 20; continue;
}
// C line 30199
22 => {
vm_block = 10; continue;
}
// C line 30198
23 => {
vm_block = if (((((from_star) != 0) && ((((((*(me)).export_name) == ((((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 30197
24 => {
me = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize));
vm_block = 23; continue;
}
// C line 30196
25 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 30194
26 => {
let _ = { let assigned = m; *((*(s)).modules).offset(({ let old = (*(s)).modules_count; (*(s)).modules_count = ((*(s)).modules_count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 30193
27 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30191
28 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(s)).modules)) as *mut *mut c_void), (((size_of::<*mut JSModuleDef>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).modules_size), ((*(s)).modules_count).wrapping_add((1 as i32)))) != 0 { 27 } else { 26 }; continue;
}
// C line 30187
29 => {
vm_block = if ((((i) < ((*(s)).modules_count)) as i32)) != 0 { 32 } else { 28 }; continue;
}
// C line 30187
30 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 29; continue;
}
// C line 30189
31 => {
return (0 as i32);
}
// C line 30188
32 => {
vm_block = if ((((*((*(s)).modules).offset((i) as isize)) == (m)) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 30187
33 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 29; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30238. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn exported_names_cmp(mut p1: *const c_void, mut p2: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut me1: *const ExportedNameEntry = core::ptr::null();
let mut me2: *const ExportedNameEntry = core::ptr::null();
let mut str1: JSValue = core::mem::zeroed();
let mut str2: JSValue = core::mem::zeroed();
let mut ret: i32 = 0;
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30258
1 => {
return ret;
}
// C line 30257
2 => {
let _ = JS_FreeValue(ctx, str2);
vm_block = 1; continue;
}
// C line 30256
3 => {
let _ = JS_FreeValue(ctx, str1);
vm_block = 2; continue;
}
// C line 30251
4 => {
let _ = { let assigned = (0 as i32); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 30253
5 => {
let _ = { let assigned = js_string_compare(ctx, ((((str1).u).ptr) as *mut JSString), ((((str2).u).ptr) as *mut JSString)); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 30249
6 => {
vm_block = if (((((JS_IsException(str1)) != 0) || ((JS_IsException(str2)) != 0)) as i32)) != 0 { 4 } else { 5 }; continue;
}
// C line 30248
7 => {
let _ = { let assigned = JS_AtomToString(ctx, (*(me2)).export_name); str2 = assigned; assigned };
vm_block = 6; continue;
}
// C line 30247
8 => {
let _ = { let assigned = JS_AtomToString(ctx, (*(me1)).export_name); str1 = assigned; assigned };
vm_block = 7; continue;
}
// C line 30242
9 => {
me2 = ((p2) as *const ExportedNameEntry);
vm_block = 8; continue;
}
// C line 30241
10 => {
me1 = ((p1) as *const ExportedNameEntry);
vm_block = 9; continue;
}
// C line 30240
11 => {
ctx = ((opaque) as *mut JSContext);
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30261. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_module_ns_autoinit(mut ctx: *mut JSContext, mut p: *mut JSObject, mut atom: JSAtom, mut opaque: *mut c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut res: JSResolveResultEnum = core::mem::zeroed();
let mut res_me: *mut JSExportEntry = core::ptr::null_mut();
let mut res_m: *mut JSModuleDef = core::ptr::null_mut();
let mut var_ref: *mut JSVarRef = core::ptr::null_mut();
let mut p1: *mut JSObject = core::ptr::null_mut();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30277
1 => {
return JS_GetModuleNamespace(ctx, (*((*(res_m)).req_module_entries).offset((((*(res_me)).u).req_module_idx) as isize)).module);
}
// C line 30286
2 => {
return JSValue { u: JSValueUnion { ptr: ((var_ref) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) };
}
// C line 30280
3 => {
let _ = { let assigned = (((*(res_me)).u).local).var_ref; var_ref = assigned; assigned };
vm_block = 2; continue;
}
// C line 30283
4 => {
let _ = { let assigned = *((((*(p1)).u).func).var_refs).offset(((((*(res_me)).u).local).var_idx) as isize); var_ref = assigned; assigned };
vm_block = 2; continue;
}
// C line 30282
5 => {
p1 = (((((*(res_m)).func_obj).u).ptr) as *mut JSObject);
vm_block = 4; continue;
}
// C line 30279
6 => {
vm_block = if !((((*(res_me)).u).local).var_ref).is_null() { 3 } else { 5 }; continue;
}
// C line 30276
7 => {
vm_block = if (((((*(res_me)).local_name) == ((((crate::quickjs_atom::JS_ATOM__star_ as i32)) as JSAtom))) as i32)) != 0 { 1 } else { 6 }; continue;
}
// C line 30274
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 30273
9 => {
let _ = js_resolve_export_throw_error(ctx, res, m, atom);
vm_block = 8; continue;
}
// C line 30271
10 => {
vm_block = if ((((((res) as i32)) != ((JS_RESOLVE_RES_FOUND as i32))) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 30270
11 => {
let _ = { let assigned = js_resolve_export(ctx, core::ptr::addr_of_mut!(res_m), core::ptr::addr_of_mut!(res_me), m, atom); res = assigned; assigned };
vm_block = 10; continue;
}
// C line 30264
12 => {
m = ((opaque) as *mut JSModuleDef);
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30290. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_build_module_ns(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::ptr::null_mut();
let mut s_s: GetExportNamesState = core::mem::zeroed();
let mut s: *mut GetExportNamesState = core::ptr::null_mut();
let mut i: i32 = 0;
let mut ret: i32 = 0;
let mut pr: *mut JSProperty = core::ptr::null_mut();
let mut en: *mut ExportedNameEntry = core::ptr::null_mut();
let mut res: JSResolveResultEnum = core::mem::zeroed();
let mut res_me: *mut JSExportEntry = core::ptr::null_mut();
let mut res_m: *mut JSModuleDef = core::ptr::null_mut();
let mut p1: *mut JSObject = core::ptr::null_mut();
let mut en_1: *mut ExportedNameEntry = core::ptr::null_mut();
let mut var_ref: *mut JSVarRef = core::ptr::null_mut();
let mut vm_block: usize = 57;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30392
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 30391
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 30390 labels: fail
3 => {
let _ = js_free(ctx, (((*(s)).exported_names) as *mut c_void));
vm_block = 2; continue;
}
// C line 30388
4 => {
return obj;
}
// C line 30387
5 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(p)).set_extensible((assigned) as _); assigned };
vm_block = 4; continue;
}
// C line 30383
6 => {
let _ = JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_Symbol_toStringTag as i32)) as JSAtom), JS_AtomToString(ctx, (((crate::quickjs_atom::JS_ATOM_Module as i32)) as JSAtom)), (0 as i32));
vm_block = 5; continue;
}
// C line 30381
7 => {
let _ = js_free(ctx, (((*(s)).exported_names) as *mut c_void));
vm_block = 6; continue;
}
// C line 30352
8 => {
vm_block = if ((((i) < ((*(s)).exported_names_count)) as i32)) != 0 { 22 } else { 7 }; continue;
}
// C line 30352
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 30377
10 => {
vm_block = 9; continue;
}
// C line 30375
11 => {
vm_block = 9; continue;
}
// C line 30374
12 => {
vm_block = 3; continue;
}
// C line 30370
13 => {
vm_block = if ((((JS_DefineAutoInitProperty(ctx, obj, (*(en_1)).export_name, (((JS_AUTOINIT_ID_MODULE_NS as i32)) as JSAutoInitIDEnum), ((m) as *mut c_void), ((((1 as i32)).wrapping_shl(((2 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 30366
14 => {
vm_block = 9; continue;
}
// C line 30364
15 => {
let _ = { let assigned = var_ref; ((*(pr)).u).var_ref = assigned; assigned };
vm_block = 14; continue;
}
// C line 30363
16 => {
let _ = { let old = (*(js_rc(((var_ref) as *mut c_void)))).ref_count; (*(js_rc(((var_ref) as *mut c_void)))).ref_count = ((*(js_rc(((var_ref) as *mut c_void)))).ref_count).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 30362
17 => {
vm_block = 3; continue;
}
// C line 30361
18 => {
vm_block = if ((!(!(pr).is_null()) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 30358
19 => {
let _ = { let assigned = add_property(ctx, p, (*(en_1)).export_name, ((((((1 as i32)).wrapping_shl(((2 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((2 as i32)).wrapping_shl(((4 as i32)) as u32)))); pr = assigned; assigned };
vm_block = 18; continue;
}
// C line 30357
20 => {
var_ref = ((*(en_1)).u).var_ref;
vm_block = 19; continue;
}
// C line 30354
21 => {
vm_block = match (((*(en_1)).export_type) as u32) { x if x == (((EXPORTED_NAME_DELAYED as i32)) as u32) => 13, x if x == (((EXPORTED_NAME_NORMAL as i32)) as u32) => 20, _ => 10, }; continue;
}
// C line 30353
22 => {
en_1 = core::ptr::addr_of_mut!(*((*(s)).exported_names).offset((i) as isize));
vm_block = 21; continue;
}
// C line 30352
23 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 30349
24 => {
let _ = crate::cutils::rqsort((((*(s)).exported_names) as *mut c_void), (((*(s)).exported_names_count) as usize), (size_of::<ExportedNameEntry>() as usize), exported_names_cmp, ((ctx) as *mut c_void));
vm_block = 23; continue;
}
// C line 30310
25 => {
vm_block = if ((((i) < ((*(s)).exported_names_count)) as i32)) != 0 { 46 } else { 24 }; continue;
}
// C line 30310
26 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 25; continue;
}
// C line 30329
27 => {
let _ = { let assigned = (((EXPORTED_NAME_AMBIGUOUS as i32)) as ExportedNameEntryEnum); (*(en)).export_type = assigned; assigned };
vm_block = 26; continue;
}
// C line 30327
28 => {
vm_block = 3; continue;
}
// C line 30326
29 => {
let _ = js_resolve_export_throw_error(ctx, res, m, (*(en)).export_name);
vm_block = 28; continue;
}
// C line 30325
30 => {
vm_block = if ((((((res) as i32)) != ((JS_RESOLVE_RES_AMBIGUOUS as i32))) as i32)) != 0 { 29 } else { 27 }; continue;
}
// C line 30332
31 => {
let _ = { let assigned = (((EXPORTED_NAME_DELAYED as i32)) as ExportedNameEntryEnum); (*(en)).export_type = assigned; assigned };
vm_block = 26; continue;
}
// C line 30341
32 => {
let _ = { let assigned = (((EXPORTED_NAME_DELAYED as i32)) as ExportedNameEntryEnum); (*(en)).export_type = assigned; assigned };
vm_block = 26; continue;
}
// C line 30343
33 => {
let _ = { let assigned = (((EXPORTED_NAME_NORMAL as i32)) as ExportedNameEntryEnum); (*(en)).export_type = assigned; assigned };
vm_block = 26; continue;
}
// C line 30340
34 => {
vm_block = if ((((((*(en)).u).var_ref) == (core::ptr::null_mut::<JSVarRef>())) as i32)) != 0 { 32 } else { 33 }; continue;
}
// C line 30335
35 => {
let _ = { let assigned = (((*(res_me)).u).local).var_ref; ((*(en)).u).var_ref = assigned; assigned };
vm_block = 34; continue;
}
// C line 30338
36 => {
let _ = { let assigned = *((((*(p1)).u).func).var_refs).offset(((((*(res_me)).u).local).var_idx) as isize); ((*(en)).u).var_ref = assigned; assigned };
vm_block = 34; continue;
}
// C line 30337
37 => {
p1 = (((((*(res_m)).func_obj).u).ptr) as *mut JSObject);
vm_block = 36; continue;
}
// C line 30334
38 => {
vm_block = if !((((*(res_me)).u).local).var_ref).is_null() { 35 } else { 37 }; continue;
}
// C line 30331
39 => {
vm_block = if (((((*(res_me)).local_name) == ((((crate::quickjs_atom::JS_ATOM__star_ as i32)) as JSAtom))) as i32)) != 0 { 31 } else { 38 }; continue;
}
// C line 30324
40 => {
vm_block = if ((((((res) as i32)) != ((JS_RESOLVE_RES_FOUND as i32))) as i32)) != 0 { 30 } else { 39 }; continue;
}
// C line 30319
41 => {
let _ = { let assigned = (((JS_RESOLVE_RES_FOUND as i32)) as JSResolveResultEnum); res = assigned; assigned };
vm_block = 40; continue;
}
// C line 30318
42 => {
let _ = { let assigned = m; res_m = assigned; assigned };
vm_block = 41; continue;
}
// C line 30317
43 => {
let _ = { let assigned = ((*(en)).u).me; res_me = assigned; assigned };
vm_block = 42; continue;
}
// C line 30321
44 => {
let _ = { let assigned = js_resolve_export(ctx, core::ptr::addr_of_mut!(res_m), core::ptr::addr_of_mut!(res_me), m, (*(en)).export_name); res = assigned; assigned };
vm_block = 40; continue;
}
// C line 30316
45 => {
vm_block = if !(((*(en)).u).me).is_null() { 43 } else { 44 }; continue;
}
// C line 30311
46 => {
en = core::ptr::addr_of_mut!(*((*(s)).exported_names).offset((i) as isize));
vm_block = 45; continue;
}
// C line 30310
47 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 25; continue;
}
// C line 30307
48 => {
vm_block = 3; continue;
}
// C line 30306
49 => {
vm_block = if (ret) != 0 { 48 } else { 47 }; continue;
}
// C line 30305
50 => {
let _ = js_free(ctx, (((*(s)).modules) as *mut c_void));
vm_block = 49; continue;
}
// C line 30304
51 => {
let _ = { let assigned = get_exported_names(ctx, s, m, (0 as i32)); ret = assigned; assigned };
vm_block = 50; continue;
}
// C line 30303
52 => {
let _ = { let dst = (((s) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<GetExportNamesState>() as usize)) as usize); dst as *mut c_void };
vm_block = 51; continue;
}
// C line 30301
53 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 52; continue;
}
// C line 30300
54 => {
return obj;
}
// C line 30299
55 => {
vm_block = if (JS_IsException(obj)) != 0 { 54 } else { 53 }; continue;
}
// C line 30298
56 => {
let _ = { let assigned = JS_NewObjectClass(ctx, (JS_CLASS_MODULE_NS as i32)); obj = assigned; assigned };
vm_block = 55; continue;
}
// C line 30294
57 => {
s = core::ptr::addr_of_mut!(s_s);
vm_block = 56; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30395. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_GetModuleNamespace(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30404
1 => {
return JS_DupValue(ctx, (*(m)).module_ns);
}
// C line 30402
2 => {
let _ = { let assigned = val; (*(m)).module_ns = assigned; assigned };
vm_block = 1; continue;
}
// C line 30401
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 30400
4 => {
vm_block = if (JS_IsException(val)) != 0 { 3 } else { 2 }; continue;
}
// C line 30399
5 => {
let _ = { let assigned = js_build_module_ns(ctx, m); val = assigned; assigned };
vm_block = 4; continue;
}
// C line 30397
6 => {
vm_block = if (JS_IsUndefined((*(m)).module_ns)) != 0 { 5 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30408. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_resolve_module(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut m1: *mut JSModuleDef = core::ptr::null_mut();
let mut rme: *mut JSReqModuleEntry = core::ptr::null_mut();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30436
1 => {
return (0 as i32);
}
// C line 30423
2 => {
vm_block = if ((((i) < ((*(m)).req_module_entries_count)) as i32)) != 0 { 10 } else { 1 }; continue;
}
// C line 30423
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 30434
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30433
5 => {
vm_block = if ((((js_resolve_module(ctx, m1)) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 30430
6 => {
let _ = { let assigned = m1; (*(rme)).module = assigned; assigned };
vm_block = 5; continue;
}
// C line 30429
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30428
8 => {
vm_block = if ((!(!(m1).is_null()) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 30425
9 => {
let _ = { let assigned = js_host_resolve_imported_module_atom(ctx, (*(m)).module_name, (*(rme)).module_name, (*(rme)).attributes); m1 = assigned; assigned };
vm_block = 8; continue;
}
// C line 30424
10 => {
rme = core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((i) as isize));
vm_block = 9; continue;
}
// C line 30423
11 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 30421
12 => {
let _ = { let assigned = (1 as i32); (*(m)).resolved = (assigned) as i8; assigned };
vm_block = 11; continue;
}
// C line 30414
13 => {
return (0 as i32);
}
// C line 30413
14 => {
vm_block = if (((*(m)).resolved as i32)) != 0 { 13 } else { 12 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30440. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_create_module_bytecode_function(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut b: *mut JSFunctionBytecode = core::ptr::null_mut();
let mut func_obj: JSValue = core::mem::zeroed();
let mut bfunc: JSValue = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30459
1 => {
return (0 as i32);
}
// C line 30457
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30456
3 => {
let _ = JS_FreeValue(ctx, func_obj);
vm_block = 2; continue;
}
// C line 30455
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(m)).func_obj = assigned; assigned };
vm_block = 3; continue;
}
// C line 30454
5 => {
vm_block = if (JS_IsException(func_obj)) != 0 { 4 } else { 1 }; continue;
}
// C line 30453
6 => {
let _ = { let assigned = js_closure2(ctx, func_obj, b, core::ptr::null_mut::<*mut JSVarRef>(), core::ptr::null_mut::<JSStackFrame>(), (1 as i32), m); func_obj = assigned; assigned };
vm_block = 5; continue;
}
// C line 30452
7 => {
let _ = { let assigned = ((((bfunc).u).ptr) as *mut JSFunctionBytecode); b = assigned; assigned };
vm_block = 6; continue;
}
// C line 30451
8 => {
let _ = { let assigned = func_obj; (*(m)).func_obj = assigned; assigned };
vm_block = 7; continue;
}
// C line 30450
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30449
10 => {
vm_block = if (JS_IsException(func_obj)) != 0 { 9 } else { 8 }; continue;
}
// C line 30446
11 => {
let _ = { let assigned = JS_NewObjectProtoClass(ctx, (*(ctx)).function_proto, (((JS_CLASS_BYTECODE_FUNCTION as i32)) as JSClassID)); func_obj = assigned; assigned };
vm_block = 10; continue;
}
// C line 30445
12 => {
let _ = { let assigned = (*(m)).func_obj; bfunc = assigned; assigned };
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30463. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_create_module_function(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut is_c_module: i32 = 0;
let mut i: i32 = 0;
let mut var_ref: *mut JSVarRef = core::ptr::null_mut();
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut rme: *mut JSReqModuleEntry = core::ptr::null_mut();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30499
1 => {
return (0 as i32);
}
// C line 30493
2 => {
vm_block = if ((((i) < ((*(m)).req_module_entries_count)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 30493
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 30496
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30495
5 => {
vm_block = if ((((js_create_module_function(ctx, (*(rme)).module)) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 30494
6 => {
rme = core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((i) as isize));
vm_block = 5; continue;
}
// C line 30493
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 30489
8 => {
let _ = { let assigned = (1 as i32); (*(m)).func_created = (assigned) as i8; assigned };
vm_block = 7; continue;
}
// C line 30476
9 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 16 } else { 8 }; continue;
}
// C line 30476
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 30482
11 => {
let _ = { let assigned = var_ref; (((*(me)).u).local).var_ref = assigned; assigned };
vm_block = 10; continue;
}
// C line 30481
12 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30480
13 => {
vm_block = if ((!(!(var_ref).is_null()) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 30479
14 => {
let _ = { let assigned = js_create_var_ref(ctx, (0 as i32)); var_ref = assigned; assigned };
vm_block = 13; continue;
}
// C line 30478
15 => {
vm_block = if (((((((*(me)).export_type) as u32)) == ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0 { 14 } else { 10 }; continue;
}
// C line 30477
16 => {
me = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize));
vm_block = 15; continue;
}
// C line 30476
17 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 30487
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30486
19 => {
vm_block = if (js_create_module_bytecode_function(ctx, m)) != 0 { 18 } else { 8 }; continue;
}
// C line 30474
20 => {
vm_block = if (is_c_module) != 0 { 17 } else { 19 }; continue;
}
// C line 30472
21 => {
let _ = { let assigned = (((*(m)).init_func).is_some() as i32); is_c_module = assigned; assigned };
vm_block = 20; continue;
}
// C line 30470
22 => {
return (0 as i32);
}
// C line 30469
23 => {
vm_block = if (((*(m)).func_created as i32)) != 0 { 22 } else { 21 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30505. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_inner_module_linking(mut ctx: *mut JSContext, mut m: *mut JSModuleDef, mut pstack_top: *mut *mut JSModuleDef, mut index: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut mi: *mut JSImportEntry = core::ptr::null_mut();
let mut m1: *mut JSModuleDef = core::ptr::null_mut();
let mut var_refs: *mut *mut JSVarRef = core::ptr::null_mut();
let mut var_ref: *mut JSVarRef = core::ptr::null_mut();
let mut p: *mut JSObject = core::ptr::null_mut();
let mut is_c_module: i32 = 0;
let mut ret_val: JSValue = core::mem::zeroed();
let mut rme: *mut JSReqModuleEntry = core::ptr::null_mut();
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut ret: JSResolveResultEnum = core::mem::zeroed();
let mut res_me: *mut JSExportEntry = core::ptr::null_mut();
let mut res_m: *mut JSModuleDef = core::ptr::null_mut();
let mut m1_1: *mut JSModuleDef = core::ptr::null_mut();
let mut val: JSValue = core::mem::zeroed();
let mut ret_1: JSResolveResultEnum = core::mem::zeroed();
let mut res_me_1: *mut JSExportEntry = core::ptr::null_mut();
let mut res_m_1: *mut JSModuleDef = core::ptr::null_mut();
let mut p1: *mut JSObject = core::ptr::null_mut();
let mut val_1: JSValue = core::mem::zeroed();
let mut m2: *mut JSModuleDef = core::ptr::null_mut();
let mut me_1: *mut JSExportEntry = core::ptr::null_mut();
let mut vm_block: usize = 90;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30699 labels: fail
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30697
2 => {
return index;
}
// C line 30684
3 => {
vm_block = 8; continue;
}
// C line 30690
4 => {
vm_block = 2; continue;
}
// C line 30689
5 => {
vm_block = if ((((m1) == (m)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 30688
6 => {
let _ = { let assigned = (((JS_MODULE_STATUS_LINKED as i32)) as JSModuleStatus); (*(m1)).status = (assigned) as u8; assigned };
vm_block = 5; continue;
}
// C line 30687
7 => {
let _ = { let assigned = (*(m1)).stack_prev; *(pstack_top) = assigned; assigned };
vm_block = 6; continue;
}
// C line 30686
8 => {
let _ = { let assigned = *(pstack_top); m1 = assigned; assigned };
vm_block = 7; continue;
}
// C line 30683
9 => {
vm_block = if (((((*(m)).dfs_index) == ((*(m)).dfs_ancestor_index)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 30682
10 => {
let _ = if ((((!((((((*(m)).dfs_ancestor_index) <= ((*(m)).dfs_index)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 9; continue;
}
// C line 30679
11 => {
let _ = JS_FreeValue(ctx, ret_val);
vm_block = 10; continue;
}
// C line 30678
12 => {
vm_block = 1; continue;
}
// C line 30677
13 => {
vm_block = if (JS_IsException(ret_val)) != 0 { 12 } else { 11 }; continue;
}
// C line 30676
14 => {
let _ = { let assigned = JS_Call(ctx, (*(m)).func_obj, JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>()); ret_val = assigned; assigned };
vm_block = 13; continue;
}
// C line 30666
15 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 21 } else { 14 }; continue;
}
// C line 30666
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 30671
17 => {
let _ = { let assigned = var_ref; (((*(me_1)).u).local).var_ref = assigned; assigned };
vm_block = 16; continue;
}
// C line 30670
18 => {
let _ = { let old = (*(js_rc(((var_ref) as *mut c_void)))).ref_count; (*(js_rc(((var_ref) as *mut c_void)))).ref_count = ((*(js_rc(((var_ref) as *mut c_void)))).ref_count).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 30669
19 => {
let _ = { let assigned = *(var_refs).offset(((((*(me_1)).u).local).var_idx) as isize); var_ref = assigned; assigned };
vm_block = 18; continue;
}
// C line 30668
20 => {
vm_block = if (((((((*(me_1)).export_type) as u32)) == ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0 { 19 } else { 16 }; continue;
}
// C line 30667
21 => {
me_1 = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize));
vm_block = 20; continue;
}
// C line 30666
22 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 30600
23 => {
vm_block = if ((((i) < ((*(m)).import_entries_count)) as i32)) != 0 { 52 } else { 22 }; continue;
}
// C line 30600
24 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 30614
25 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(((*(*(var_refs).offset(((*(mi)).var_idx) as isize))).u).value), val);
vm_block = 24; continue;
}
// C line 30613
26 => {
vm_block = 1; continue;
}
// C line 30612
27 => {
vm_block = if (JS_IsException(val)) != 0 { 26 } else { 25 }; continue;
}
// C line 30611
28 => {
let _ = { let assigned = JS_GetModuleNamespace(ctx, m1); val = assigned; assigned };
vm_block = 27; continue;
}
// C line 30644
29 => {
let _ = { let assigned = var_ref; *(var_refs).offset(((*(mi)).var_idx) as isize) = assigned; assigned };
vm_block = 24; continue;
}
// C line 30643
30 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(((*(var_ref)).u).value), val_1);
vm_block = 29; continue;
}
// C line 30641
31 => {
vm_block = 1; continue;
}
// C line 30640
32 => {
let _ = JS_FreeValue(ctx, val_1);
vm_block = 31; continue;
}
// C line 30639
33 => {
vm_block = if ((!(!(var_ref).is_null()) as i32)) != 0 { 32 } else { 30 }; continue;
}
// C line 30638
34 => {
let _ = { let assigned = js_create_var_ref(ctx, (1 as i32)); var_ref = assigned; assigned };
vm_block = 33; continue;
}
// C line 30637
35 => {
vm_block = 1; continue;
}
// C line 30636
36 => {
vm_block = if (JS_IsException(val_1)) != 0 { 35 } else { 34 }; continue;
}
// C line 30635
37 => {
let _ = { let assigned = JS_GetModuleNamespace(ctx, m2); val_1 = assigned; assigned };
vm_block = 36; continue;
}
// C line 30634
38 => {
let _ = { let assigned = (*((*(res_m_1)).req_module_entries).offset((((*(res_me_1)).u).req_module_idx) as isize)).module; m2 = assigned; assigned };
vm_block = 37; continue;
}
// C line 30655
39 => {
let _ = { let assigned = var_ref; *(var_refs).offset(((*(mi)).var_idx) as isize) = assigned; assigned };
vm_block = 24; continue;
}
// C line 30654
40 => {
let _ = { let old = (*(js_rc(((var_ref) as *mut c_void)))).ref_count; (*(js_rc(((var_ref) as *mut c_void)))).ref_count = ((*(js_rc(((var_ref) as *mut c_void)))).ref_count).wrapping_add(1); old };
vm_block = 39; continue;
}
// C line 30652
41 => {
let _ = { let assigned = *((((*(p1)).u).func).var_refs).offset(((((*(res_me_1)).u).local).var_idx) as isize); var_ref = assigned; assigned };
vm_block = 40; continue;
}
// C line 30651
42 => {
let _ = { let assigned = (((((*(res_m_1)).func_obj).u).ptr) as *mut JSObject); p1 = assigned; assigned };
vm_block = 41; continue;
}
// C line 30650
43 => {
vm_block = if ((!(!(var_ref).is_null()) as i32)) != 0 { 42 } else { 40 }; continue;
}
// C line 30649
44 => {
let _ = { let assigned = (((*(res_me_1)).u).local).var_ref; var_ref = assigned; assigned };
vm_block = 43; continue;
}
// C line 30630
45 => {
vm_block = if (((((*(res_me_1)).local_name) == ((((crate::quickjs_atom::JS_ATOM__star_ as i32)) as JSAtom))) as i32)) != 0 { 38 } else { 44 }; continue;
}
// C line 30628
46 => {
vm_block = 1; continue;
}
// C line 30627
47 => {
let _ = js_resolve_export_throw_error(ctx, ret_1, m1, (*(mi)).import_name);
vm_block = 46; continue;
}
// C line 30626
48 => {
vm_block = if ((((((ret_1) as i32)) != ((JS_RESOLVE_RES_FOUND as i32))) as i32)) != 0 { 47 } else { 45 }; continue;
}
// C line 30624
49 => {
let _ = { let assigned = js_resolve_export(ctx, core::ptr::addr_of_mut!(res_m_1), core::ptr::addr_of_mut!(res_me_1), m1, (*(mi)).import_name); ret_1 = assigned; assigned };
vm_block = 48; continue;
}
// C line 30608
50 => {
vm_block = if ((*(mi)).is_star) != 0 { 28 } else { 49 }; continue;
}
// C line 30607
51 => {
let _ = { let assigned = (*((*(m)).req_module_entries).offset(((*(mi)).req_module_idx) as isize)).module; m1 = assigned; assigned };
vm_block = 50; continue;
}
// C line 30601
52 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(m)).import_entries).offset((i) as isize)); mi = assigned; assigned };
vm_block = 51; continue;
}
// C line 30600
53 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 23; continue;
}
// C line 30598
54 => {
let _ = { let assigned = (((*(p)).u).func).var_refs; var_refs = assigned; assigned };
vm_block = 53; continue;
}
// C line 30597
55 => {
let _ = { let assigned = (((((*(m)).func_obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 54; continue;
}
// C line 30596
56 => {
vm_block = if ((!((is_c_module) != 0) as i32)) != 0 { 55 } else { 10 }; continue;
}
// C line 30594
57 => {
let _ = { let assigned = (((*(m)).init_func).is_some() as i32); is_c_module = assigned; assigned };
vm_block = 56; continue;
}
// C line 30566
58 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 66 } else { 57 }; continue;
}
// C line 30566
59 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 58; continue;
}
// C line 30577
60 => {
vm_block = 1; continue;
}
// C line 30576
61 => {
let _ = js_resolve_export_throw_error(ctx, ret, m, (*(me)).export_name);
vm_block = 60; continue;
}
// C line 30575
62 => {
vm_block = if ((((((ret) as i32)) != ((JS_RESOLVE_RES_FOUND as i32))) as i32)) != 0 { 61 } else { 59 }; continue;
}
// C line 30574
63 => {
let _ = { let assigned = js_resolve_export(ctx, core::ptr::addr_of_mut!(res_m), core::ptr::addr_of_mut!(res_me), m1_1, (*(me)).local_name); ret = assigned; assigned };
vm_block = 62; continue;
}
// C line 30573
64 => {
let _ = { let assigned = (*((*(m)).req_module_entries).offset((((*(me)).u).req_module_idx) as isize)).module; m1_1 = assigned; assigned };
vm_block = 63; continue;
}
// C line 30568
65 => {
vm_block = if (((((((((((*(me)).export_type) as u32)) == ((((JS_EXPORT_TYPE_INDIRECT as i32)) as u32))) as i32)) != 0) && ((((((*(me)).local_name) != ((((crate::quickjs_atom::JS_ATOM__star_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 64 } else { 59 }; continue;
}
// C line 30567
66 => {
me = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize));
vm_block = 65; continue;
}
// C line 30566
67 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 58; continue;
}
// C line 30543
68 => {
vm_block = if ((((i) < ((*(m)).req_module_entries_count)) as i32)) != 0 { 77 } else { 67 }; continue;
}
// C line 30543
69 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 68; continue;
}
// C line 30554
70 => {
let _ = { let assigned = crate::cutils_header::min_int((*(m)).dfs_ancestor_index, (*(m1)).dfs_ancestor_index); (*(m)).dfs_ancestor_index = assigned; assigned };
vm_block = 69; continue;
}
// C line 30553
71 => {
vm_block = if ((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKING as i32))) as i32)) != 0 { 70 } else { 69 }; continue;
}
// C line 30549
72 => {
let _ = if ((((!(((((((((((((((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKING as i32))) as i32)) != 0) || (((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKED as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 71; continue;
}
// C line 30548
73 => {
vm_block = 1; continue;
}
// C line 30547
74 => {
vm_block = if ((((index) < ((0 as i32))) as i32)) != 0 { 73 } else { 72 }; continue;
}
// C line 30546
75 => {
let _ = { let assigned = js_inner_module_linking(ctx, m1, pstack_top, index); index = assigned; assigned };
vm_block = 74; continue;
}
// C line 30545
76 => {
let _ = { let assigned = (*(rme)).module; m1 = assigned; assigned };
vm_block = 75; continue;
}
// C line 30544
77 => {
rme = core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((i) as isize));
vm_block = 76; continue;
}
// C line 30543
78 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 68; continue;
}
// C line 30541
79 => {
let _ = { let assigned = m; *(pstack_top) = assigned; assigned };
vm_block = 78; continue;
}
// C line 30540
80 => {
let _ = { let assigned = *(pstack_top); (*(m)).stack_prev = assigned; assigned };
vm_block = 79; continue;
}
// C line 30538
81 => {
let _ = { let old = index; index = (index).wrapping_add(1); old };
vm_block = 80; continue;
}
// C line 30537
82 => {
let _ = { let assigned = index; (*(m)).dfs_ancestor_index = assigned; assigned };
vm_block = 81; continue;
}
// C line 30536
83 => {
let _ = { let assigned = index; (*(m)).dfs_index = assigned; assigned };
vm_block = 82; continue;
}
// C line 30535
84 => {
let _ = { let assigned = (((JS_MODULE_STATUS_LINKING as i32)) as JSModuleStatus); (*(m)).status = (assigned) as u8; assigned };
vm_block = 83; continue;
}
// C line 30534
85 => {
let _ = if ((((!(((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_UNLINKED as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 84; continue;
}
// C line 30532
86 => {
return index;
}
// C line 30528
87 => {
vm_block = if ((((((((((((((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKING as i32))) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKED as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0 { 86 } else { 85 }; continue;
}
// C line 30518
88 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30517
89 => {
let _ = JS_ThrowStackOverflow(ctx);
vm_block = 88; continue;
}
// C line 30516
90 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 89 } else { 87 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30704. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_link_module(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut stack_top: *mut JSModuleDef = core::ptr::null_mut();
let mut m1: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30732
1 => {
return (0 as i32);
}
// C line 30729
2 => {
let _ = if ((((!(((((((((((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKED as i32))) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 1; continue;
}
// C line 30728
3 => {
let _ = if ((((!(((((stack_top) == (core::ptr::null_mut::<JSModuleDef>())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 2; continue;
}
// C line 30726
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 30720
5 => {
vm_block = if ((((stack_top) != (core::ptr::null_mut::<JSModuleDef>())) as i32)) != 0 { 9 } else { 4 }; continue;
}
// C line 30724
6 => {
let _ = { let assigned = (*(m1)).stack_prev; stack_top = assigned; assigned };
vm_block = 5; continue;
}
// C line 30723
7 => {
let _ = { let assigned = (((JS_MODULE_STATUS_UNLINKED as i32)) as JSModuleStatus); (*(m1)).status = (assigned) as u8; assigned };
vm_block = 6; continue;
}
// C line 30722
8 => {
let _ = if ((((!(((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKING as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 7; continue;
}
// C line 30721
9 => {
let _ = { let assigned = stack_top; m1 = assigned; assigned };
vm_block = 8; continue;
}
// C line 30719
10 => {
vm_block = if ((((js_inner_module_linking(ctx, m, core::ptr::addr_of_mut!(stack_top), (0 as i32))) < ((0 as i32))) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 30718
11 => {
let _ = { let assigned = core::ptr::null_mut::<JSModuleDef>(); stack_top = assigned; assigned };
vm_block = 10; continue;
}
// C line 30714
12 => {
let _ = if ((((!(((((((((((((((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_UNLINKED as i32))) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKED as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30737. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_GetScriptOrModuleName(mut ctx: *mut JSContext, mut n_stack_levels: i32) -> JSAtom {
let mut vm_local_storage = Vec::<u64>::new();
let mut sf: *mut JSStackFrame = core::ptr::null_mut();
let mut b: *mut JSFunctionBytecode = core::ptr::null_mut();
let mut p: *mut JSObject = core::ptr::null_mut();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30753
1 => {
vm_block = 14; continue;
}
// C line 30763
2 => {
return JS_DupAtom(ctx, ((*(b)).debug).filename);
}
// C line 30762
3 => {
return (((0 as i32)) as JSAtom);
}
// C line 30761
4 => {
vm_block = if ((!(((*(b)).has_debug()) != 0) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 30767
5 => {
return (((0 as i32)) as JSAtom);
}
// C line 30766
6 => {
vm_block = if ((!(!(sf).is_null()) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 30765
7 => {
let _ = { let assigned = (*(sf)).prev_frame; sf = assigned; assigned };
vm_block = 6; continue;
}
// C line 30760
8 => {
vm_block = if ((!(((*(b)).is_direct_or_indirect_eval()) != 0) as i32)) != 0 { 4 } else { 7 }; continue;
}
// C line 30759
9 => {
let _ = { let assigned = (((*(p)).u).func).function_bytecode; b = assigned; assigned };
vm_block = 8; continue;
}
// C line 30758
10 => {
return (((0 as i32)) as JSAtom);
}
// C line 30757
11 => {
vm_block = if ((!((js_class_has_bytecode((((*(p)).class_id) as JSClassID))) != 0) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 30756
12 => {
let _ = { let assigned = (((((*(sf)).cur_func).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 11; continue;
}
// C line 30755
13 => {
return (((0 as i32)) as JSAtom);
}
// C line 30754
14 => {
vm_block = if ((((((((*(sf)).cur_func).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 30748
15 => {
vm_block = if (((({ let old = n_stack_levels; n_stack_levels = (n_stack_levels).wrapping_sub(1); old }) > ((0 as i32))) as i32)) != 0 { 18 } else { 1 }; continue;
}
// C line 30751
16 => {
return (((0 as i32)) as JSAtom);
}
// C line 30750
17 => {
vm_block = if ((!(!(sf).is_null()) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 30749
18 => {
let _ = { let assigned = (*(sf)).prev_frame; sf = assigned; assigned };
vm_block = 17; continue;
}
// C line 30747
19 => {
return (((0 as i32)) as JSAtom);
}
// C line 30746
20 => {
vm_block = if ((!(!(sf).is_null()) as i32)) != 0 { 19 } else { 15 }; continue;
}
// C line 30745
21 => {
let _ = { let assigned = (*((*(ctx)).rt)).current_stack_frame; sf = assigned; assigned };
vm_block = 20; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30777. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_GetImportMeta(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30788
1 => {
return JS_DupValue(ctx, obj);
}
// C line 30786
2 => {
let _ = { let assigned = obj; (*(m)).meta_obj = assigned; assigned };
vm_block = 1; continue;
}
// C line 30785
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 30784
4 => {
vm_block = if (JS_IsException(obj)) != 0 { 3 } else { 2 }; continue;
}
// C line 30783
5 => {
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); obj = assigned; assigned };
vm_block = 4; continue;
}
// C line 30782
6 => {
vm_block = if (JS_IsUndefined(obj)) != 0 { 5 } else { 1 }; continue;
}
// C line 30781
7 => {
let _ = { let assigned = (*(m)).meta_obj; obj = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30791. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_import_meta(mut ctx: *mut JSContext) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut filename: JSAtom = 0;
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30809
1 => {
return JS_GetImportMeta(ctx, m);
}
// C line 30807
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 30806 labels: fail
3 => {
let _ = JS_ThrowTypeError(ctx, c"import.meta not supported in this context".as_ptr());
vm_block = 2; continue;
}
// C line 30804
4 => {
vm_block = if ((!(!(m).is_null()) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 30803
5 => {
let _ = JS_FreeAtom(ctx, filename);
vm_block = 4; continue;
}
// C line 30802
6 => {
let _ = { let assigned = js_find_loaded_module(ctx, filename); m = assigned; assigned };
vm_block = 5; continue;
}
// C line 30798
7 => {
vm_block = 3; continue;
}
// C line 30797
8 => {
vm_block = if ((((filename) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 30796
9 => {
let _ = { let assigned = JS_GetScriptOrModuleName(ctx, (0 as i32)); filename = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30812. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_NewModuleValue(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30814
1 => {
return JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: ((m) as *mut c_void) }, tag: (((JS_TAG_MODULE as i32)) as i64) });
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30817. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_load_module_rejected(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut resolving_funcs: *mut JSValue = core::ptr::null_mut();
let mut error: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30832
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 30831
2 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 1; continue;
}
// C line 30829
3 => {
let _ = { let assigned = JS_Call(ctx, *(resolving_funcs).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 30826
4 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); error = assigned; assigned };
vm_block = 3; continue;
}
// C line 30828
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; error = assigned; assigned };
vm_block = 3; continue;
}
// C line 30825
6 => {
vm_block = if ((((argc) >= ((1 as i32))) as i32)) != 0 { 4 } else { 5 }; continue;
}
// C line 30820
7 => {
resolving_funcs = func_data;
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30835. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_load_module_fulfilled(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut resolving_funcs: *mut JSValue = core::ptr::null_mut();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut ret: JSValue = core::mem::zeroed();
let mut ns: JSValue = core::mem::zeroed();
let mut err: JSValue = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30853
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 30852
2 => {
let _ = JS_FreeValue(ctx, ns);
vm_block = 1; continue;
}
// C line 30851
3 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 2; continue;
}
// C line 30849
4 => {
let _ = { let assigned = JS_Call(ctx, *(resolving_funcs).offset(((0 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(ns)); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 30847
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 30846
6 => {
let _ = js_load_module_rejected(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(err), (0 as i32), func_data);
vm_block = 5; continue;
}
// C line 30845
7 => {
err = JS_GetException(ctx);
vm_block = 6; continue;
}
// C line 30844
8 => {
vm_block = if (JS_IsException(ns)) != 0 { 7 } else { 4 }; continue;
}
// C line 30843
9 => {
let _ = { let assigned = JS_GetModuleNamespace(ctx, m); ns = assigned; assigned };
vm_block = 8; continue;
}
// C line 30839
10 => {
m = ((((*(func_data).offset(((2 as i32)) as isize)).u).ptr) as *mut JSModuleDef);
vm_block = 9; continue;
}
// C line 30838
11 => {
resolving_funcs = func_data;
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30856. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_LoadModuleInternal(mut ctx: *mut JSContext, mut basename: *const c_char, mut filename: *const c_char, mut resolving_funcs: *mut JSValue, mut attributes: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut evaluate_promise: JSValue = core::mem::zeroed();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut ret: JSValue = core::mem::zeroed();
let mut err: JSValue = core::mem::zeroed();
let mut func_obj: JSValue = core::mem::zeroed();
let mut evaluate_resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut func_data: [JSValue; 3] = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 30899
1 => {
let _ = JS_FreeValue(ctx, evaluate_promise);
vm_block = 0; continue;
}
// C line 30898
2 => {
let _ = JS_FreeValue(ctx, *((evaluate_resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 1; continue;
}
// C line 30897
3 => {
let _ = JS_FreeValue(ctx, *((evaluate_resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 2; continue;
}
// C line 30896
4 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 3; continue;
}
// C line 30895
5 => {
let _ = { let assigned = js_promise_then(ctx, evaluate_promise, (2 as i32), (evaluate_resolving_funcs).as_mut_ptr()); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 30894
6 => {
let _ = JS_FreeValue(ctx, func_obj);
vm_block = 5; continue;
}
// C line 30893
7 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_load_module_rejected), (0 as i32), (0 as i32), (3 as i32), (func_data).as_mut_ptr()); *((evaluate_resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 30892
8 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_load_module_fulfilled), (0 as i32), (0 as i32), (3 as i32), (func_data).as_mut_ptr()); *((evaluate_resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 30891
9 => {
let _ = { let assigned = func_obj; *((func_data).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 30890
10 => {
let _ = { let assigned = *(resolving_funcs).offset(((1 as i32)) as isize); *((func_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 30889
11 => {
let _ = { let assigned = *(resolving_funcs).offset(((0 as i32)) as isize); *((func_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 30888
12 => {
let _ = { let assigned = JS_NewModuleValue(ctx, m); func_obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 30885
13 => {
return;
}
// C line 30884
14 => {
let _ = JS_FreeValue(ctx, err);
vm_block = 13; continue;
}
// C line 30883
15 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 14; continue;
}
// C line 30881
16 => {
let _ = { let assigned = JS_Call(ctx, *(resolving_funcs).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(err)); ret = assigned; assigned };
vm_block = 15; continue;
}
// C line 30880 labels: fail
17 => {
let _ = { let assigned = JS_GetException(ctx); err = assigned; assigned };
vm_block = 16; continue;
}
// C line 30878
18 => {
vm_block = if (JS_IsException(evaluate_promise)) != 0 { 17 } else { 12 }; continue;
}
// C line 30877
19 => {
let _ = { let assigned = JS_EvalFunction(ctx, func_obj); evaluate_promise = assigned; assigned };
vm_block = 18; continue;
}
// C line 30876
20 => {
let _ = { let assigned = JS_NewModuleValue(ctx, m); func_obj = assigned; assigned };
vm_block = 19; continue;
}
// C line 30872
21 => {
vm_block = 17; continue;
}
// C line 30871
22 => {
let _ = js_free_modules(ctx, (((JS_FREE_MODULE_NOT_RESOLVED as i32)) as JSFreeModuleEnum));
vm_block = 21; continue;
}
// C line 30870
23 => {
vm_block = if ((((js_resolve_module(ctx, m)) < ((0 as i32))) as i32)) != 0 { 22 } else { 20 }; continue;
}
// C line 30868
24 => {
vm_block = 17; continue;
}
// C line 30867
25 => {
vm_block = if ((!(!(m).is_null()) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 30866
26 => {
let _ = { let assigned = js_host_resolve_imported_module(ctx, basename, filename, attributes); m = assigned; assigned };
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30904. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_LoadModule(mut ctx: *mut JSContext, mut basename: *const c_char, mut filename: *const c_char) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30916
1 => {
return promise;
}
// C line 30915
2 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 1; continue;
}
// C line 30914
3 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 2; continue;
}
// C line 30912
4 => {
let _ = JS_LoadModuleInternal(ctx, basename, filename, (resolving_funcs).as_mut_ptr(), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) });
vm_block = 3; continue;
}
// C line 30911
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 30910
6 => {
vm_block = if (JS_IsException(promise)) != 0 { 5 } else { 4 }; continue;
}
// C line 30909
7 => {
let _ = { let assigned = JS_NewPromiseCapability(ctx, (resolving_funcs).as_mut_ptr()); promise = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30919. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dynamic_import_job(mut ctx: *mut JSContext, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut resolving_funcs: *mut JSValue = core::ptr::null_mut();
let mut basename_val: JSValue = core::mem::zeroed();
let mut specifier: JSValue = core::mem::zeroed();
let mut attributes: JSValue = core::mem::zeroed();
let mut basename: *const c_char = core::ptr::null();
let mut filename: *const c_char = core::ptr::null();
let mut ret: JSValue = core::mem::zeroed();
let mut err: JSValue = core::mem::zeroed();
let mut vm_block: usize = 24;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 30953
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 30952
2 => {
let _ = JS_FreeCString(ctx, basename);
vm_block = 1; continue;
}
// C line 30951
3 => {
let _ = JS_FreeValue(ctx, err);
vm_block = 2; continue;
}
// C line 30950
4 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 3; continue;
}
// C line 30948
5 => {
let _ = { let assigned = JS_Call(ctx, *(resolving_funcs).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(err)); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 30947 labels: exception
6 => {
let _ = { let assigned = JS_GetException(ctx); err = assigned; assigned };
vm_block = 5; continue;
}
// C line 30945
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 30944
8 => {
let _ = JS_FreeCString(ctx, basename);
vm_block = 7; continue;
}
// C line 30943
9 => {
let _ = JS_FreeCString(ctx, filename);
vm_block = 8; continue;
}
// C line 30941
10 => {
let _ = JS_LoadModuleInternal(ctx, basename, filename, resolving_funcs, attributes);
vm_block = 9; continue;
}
// C line 30939
11 => {
vm_block = 6; continue;
}
// C line 30938
12 => {
vm_block = if ((!(!(filename).is_null()) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 30937
13 => {
let _ = { let assigned = JS_ToCString(ctx, specifier); filename = assigned; assigned };
vm_block = 12; continue;
}
// C line 30935
14 => {
vm_block = 6; continue;
}
// C line 30934
15 => {
vm_block = if ((!(!(basename).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 30933
16 => {
let _ = { let assigned = JS_ToCString(ctx, basename_val); basename = assigned; assigned };
vm_block = 15; continue;
}
// C line 30931
17 => {
vm_block = 6; continue;
}
// C line 30930
18 => {
let _ = JS_ThrowTypeError(ctx, c"no function filename for import()".as_ptr());
vm_block = 17; continue;
}
// C line 30929
19 => {
vm_block = if ((!((JS_IsString(basename_val)) != 0) as i32)) != 0 { 18 } else { 16 }; continue;
}
// C line 30926
20 => {
basename = core::ptr::null_mut::<c_char>();
vm_block = 19; continue;
}
// C line 30925
21 => {
attributes = *(argv).offset(((4 as i32)) as isize);
vm_block = 20; continue;
}
// C line 30924
22 => {
specifier = *(argv).offset(((3 as i32)) as isize);
vm_block = 21; continue;
}
// C line 30923
23 => {
basename_val = *(argv).offset(((2 as i32)) as isize);
vm_block = 22; continue;
}
// C line 30922
24 => {
resolving_funcs = argv;
vm_block = 23; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:30956. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dynamic_import(mut ctx: *mut JSContext, mut specifier: JSValue, mut options: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut basename: JSAtom = 0;
let mut promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut basename_val: JSValue = core::mem::zeroed();
let mut err: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut specifier_str: JSValue = core::mem::zeroed();
let mut attributes: JSValue = core::mem::zeroed();
let mut attributes_obj: JSValue = core::mem::zeroed();
let mut args: [JSValue; 5] = core::mem::zeroed();
let mut atoms: *mut JSPropertyEnum = core::ptr::null_mut();
let mut atoms_len: u32 = 0;
let mut i: u32 = 0;
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 64;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31053
1 => {
vm_block = 12; continue;
}
// C line 31052
2 => {
let _ = JS_FreeValue(ctx, err);
vm_block = 1; continue;
}
// C line 31051
3 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 2; continue;
}
// C line 31049
4 => {
let _ = { let assigned = JS_Call(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(err)); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 31048
5 => {
let _ = { let assigned = JS_GetException(ctx); err = assigned; assigned };
vm_block = 4; continue;
}
// C line 31047 labels: exception
6 => {
let _ = JS_FreeValue(ctx, attributes_obj);
vm_block = 5; continue;
}
// C line 31045
7 => {
return promise;
}
// C line 31044
8 => {
let _ = JS_FreeValue(ctx, attributes);
vm_block = 7; continue;
}
// C line 31043
9 => {
let _ = JS_FreeValue(ctx, specifier_str);
vm_block = 8; continue;
}
// C line 31042
10 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 9; continue;
}
// C line 31041
11 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 10; continue;
}
// C line 31040 labels: done
12 => {
let _ = JS_FreeValue(ctx, basename_val);
vm_block = 11; continue;
}
// C line 31038
13 => {
let _ = js_host_dynamic_import_or_enqueue(ctx, (5 as i32), (args).as_mut_ptr());
vm_block = 12; continue;
}
// C line 31034
14 => {
let _ = { let assigned = attributes; *((args).as_mut_ptr()).offset(((4 as i32)) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 31033
15 => {
let _ = { let assigned = specifier_str; *((args).as_mut_ptr()).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 31032
16 => {
let _ = { let assigned = basename_val; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 31031
17 => {
let _ = { let assigned = *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 31030
18 => {
let _ = { let assigned = *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 31026
19 => {
let _ = JS_FreeValue(ctx, attributes_obj);
vm_block = 18; continue;
}
// C line 31024
20 => {
vm_block = 6; continue;
}
// C line 31022
21 => {
vm_block = if ((((((*((*(ctx)).rt)).module_check_attrs).is_some()) && (((((((*((*(ctx)).rt)).module_check_attrs).expect("registered parser callback")(ctx, (*((*(ctx)).rt)).module_loader_opaque, attributes)) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 31021
22 => {
let _ = JS_FreePropertyEnum(ctx, atoms, atoms_len);
vm_block = 21; continue;
}
// C line 31005
23 => {
vm_block = if ((((i) < (atoms_len)) as i32)) != 0 { 34 } else { 22 }; continue;
}
// C line 31005
24 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 31018
25 => {
vm_block = 6; continue;
}
// C line 31017 labels: exception1
26 => {
let _ = JS_FreePropertyEnum(ctx, atoms, atoms_len);
vm_block = 25; continue;
}
// C line 31014
27 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, attributes, (*(atoms).offset((i) as isize)).atom, val, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 26 } else { 24 }; continue;
}
// C line 31012
28 => {
vm_block = 26; continue;
}
// C line 31011
29 => {
let _ = JS_ThrowTypeError(ctx, c"module attribute values must be strings".as_ptr());
vm_block = 28; continue;
}
// C line 31010
30 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 29; continue;
}
// C line 31009
31 => {
vm_block = if ((!((JS_IsString(val)) != 0) as i32)) != 0 { 30 } else { 27 }; continue;
}
// C line 31008
32 => {
vm_block = 26; continue;
}
// C line 31007
33 => {
vm_block = if (JS_IsException(val)) != 0 { 32 } else { 31 }; continue;
}
// C line 31006
34 => {
let _ = { let assigned = JS_GetProperty(ctx, attributes_obj, (*(atoms).offset((i) as isize)).atom); val = assigned; assigned };
vm_block = 33; continue;
}
// C line 31005
35 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 23; continue;
}
// C line 31003
36 => {
vm_block = 6; continue;
}
// C line 31001
37 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(atoms), core::ptr::addr_of_mut!(atoms_len), ((((attributes_obj).u).ptr) as *mut JSObject), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((4 as i32)) as u32))))) != 0 { 36 } else { 35 }; continue;
}
// C line 31000
38 => {
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); attributes = assigned; assigned };
vm_block = 37; continue;
}
// C line 30998
39 => {
vm_block = 6; continue;
}
// C line 30997
40 => {
let _ = JS_ThrowTypeError(ctx, c"options.with must be an object".as_ptr());
vm_block = 39; continue;
}
// C line 30996
41 => {
vm_block = if ((!((JS_IsObject(attributes_obj)) != 0) as i32)) != 0 { 40 } else { 38 }; continue;
}
// C line 30991
42 => {
vm_block = if ((!((JS_IsUndefined(attributes_obj)) != 0) as i32)) != 0 { 41 } else { 18 }; continue;
}
// C line 30990
43 => {
vm_block = 6; continue;
}
// C line 30989
44 => {
vm_block = if (JS_IsException(attributes_obj)) != 0 { 43 } else { 42 }; continue;
}
// C line 30988
45 => {
let _ = { let assigned = JS_GetProperty(ctx, options, (((crate::quickjs_atom::JS_ATOM_with as i32)) as JSAtom)); attributes_obj = assigned; assigned };
vm_block = 44; continue;
}
// C line 30986
46 => {
vm_block = 6; continue;
}
// C line 30985
47 => {
let _ = JS_ThrowTypeError(ctx, c"options must be an object".as_ptr());
vm_block = 46; continue;
}
// C line 30984
48 => {
vm_block = if ((!((JS_IsObject(options)) != 0) as i32)) != 0 { 47 } else { 45 }; continue;
}
// C line 30983
49 => {
vm_block = if ((!((JS_IsUndefined(options)) != 0) as i32)) != 0 { 48 } else { 18 }; continue;
}
// C line 30981
50 => {
vm_block = 6; continue;
}
// C line 30980
51 => {
vm_block = if (JS_IsException(specifier_str)) != 0 { 50 } else { 49 }; continue;
}
// C line 30979
52 => {
let _ = { let assigned = JS_ToString(ctx, specifier); specifier_str = assigned; assigned };
vm_block = 51; continue;
}
// C line 30975
53 => {
return promise;
}
// C line 30974
54 => {
let _ = JS_FreeValue(ctx, basename_val);
vm_block = 53; continue;
}
// C line 30973
55 => {
vm_block = if (JS_IsException(promise)) != 0 { 54 } else { 52 }; continue;
}
// C line 30972
56 => {
let _ = { let assigned = JS_NewPromiseCapability(ctx, (resolving_funcs).as_mut_ptr()); promise = assigned; assigned };
vm_block = 55; continue;
}
// C line 30970
57 => {
return basename_val;
}
// C line 30969
58 => {
vm_block = if (JS_IsException(basename_val)) != 0 { 57 } else { 56 }; continue;
}
// C line 30968
59 => {
let _ = JS_FreeAtom(ctx, basename);
vm_block = 58; continue;
}
// C line 30965
60 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; basename_val = assigned; assigned };
vm_block = 59; continue;
}
// C line 30967
61 => {
let _ = { let assigned = JS_AtomToValue(ctx, basename); basename_val = assigned; assigned };
vm_block = 59; continue;
}
// C line 30964
62 => {
vm_block = if ((((basename) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 60 } else { 61 }; continue;
}
// C line 30963
63 => {
let _ = { let assigned = JS_GetScriptOrModuleName(ctx, (0 as i32)); basename = assigned; assigned };
vm_block = 62; continue;
}
// C line 30960
64 => {
specifier_str = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
attributes = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
attributes_obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 63; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31056. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_set_module_evaluated(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut value: JSValue = core::mem::zeroed();
let mut ret_val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 31065
1 => {
let _ = JS_FreeValue(ctx, ret_val);
vm_block = 0; continue;
}
// C line 31063
2 => {
let _ = { let assigned = JS_Call(ctx, *(((*(m)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(value)); ret_val = assigned; assigned };
vm_block = 1; continue;
}
// C line 31062
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
vm_block = 2; continue;
}
// C line 31061
4 => {
let _ = if ((((!((((((*(m)).cycle_root) == (m)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 3; continue;
}
// C line 31059
5 => {
vm_block = if ((!((JS_IsUndefined((*(m)).promise)) != 0) as i32)) != 0 { 4 } else { 0 }; continue;
}
// C line 31058
6 => {
let _ = { let assigned = (((JS_MODULE_STATUS_EVALUATED as i32)) as JSModuleStatus); (*(m)).status = (assigned) as u8; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31076. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_in_exec_module_list(mut exec_list: *mut ExecModuleList, mut m: *mut JSModuleDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31083
1 => {
return (0 as i32);
}
// C line 31079
2 => {
vm_block = if ((((i) < ((*(exec_list)).count)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 31079
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 31081
4 => {
return (1 as i32);
}
// C line 31080
5 => {
vm_block = if ((((*((*(exec_list)).tab).offset((i) as isize)) == (m)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 31079
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31086. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn gather_available_ancestors(mut ctx: *mut JSContext, mut module: *mut JSModuleDef, mut exec_list: *mut ExecModuleList) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31116
1 => {
return (0 as i32);
}
// C line 31095
2 => {
vm_block = if ((((i) < ((*(module)).async_parent_modules_count)) as i32)) != 0 { 17 } else { 1 }; continue;
}
// C line 31095
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 31111
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31110
5 => {
vm_block = if (gather_available_ancestors(ctx, m, exec_list)) != 0 { 4 } else { 3 }; continue;
}
// C line 31109
6 => {
vm_block = if ((!((((*(m)).has_tla as i32)) != 0) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 31108
7 => {
let _ = { let assigned = m; *((*(exec_list)).tab).offset(({ let old = (*(exec_list)).count; (*(exec_list)).count = ((*(exec_list)).count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 31106
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31105
9 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(exec_list)).tab)) as *mut *mut c_void), (((size_of::<*mut JSModuleDef>() as usize)) as i32), core::ptr::addr_of_mut!((*(exec_list)).size), ((*(exec_list)).count).wrapping_add((1 as i32)))) != 0 { 8 } else { 7 }; continue;
}
// C line 31104
10 => {
vm_block = if (((((*(m)).pending_async_dependencies) == ((0 as i32))) as i32)) != 0 { 9 } else { 3 }; continue;
}
// C line 31103
11 => {
let _ = { let old = (*(m)).pending_async_dependencies; (*(m)).pending_async_dependencies = ((*(m)).pending_async_dependencies).wrapping_sub(1); old };
vm_block = 10; continue;
}
// C line 31102
12 => {
let _ = if ((((!((((((*(m)).pending_async_dependencies) > ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 11; continue;
}
// C line 31101
13 => {
let _ = if ((((!(((*(m)).async_evaluation) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 31100
14 => {
let _ = if ((((!(((!((((*(m)).eval_has_exception as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 13; continue;
}
// C line 31099
15 => {
let _ = if ((((!(((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 14; continue;
}
// C line 31097
16 => {
vm_block = if ((((((!((find_in_exec_module_list(exec_list, m)) != 0) as i32)) != 0) && (((!((((*((*(m)).cycle_root)).eval_has_exception as i32)) != 0) as i32)) != 0)) as i32)) != 0 { 15 } else { 3 }; continue;
}
// C line 31096
17 => {
m = *((*(module)).async_parent_modules).offset((i) as isize);
vm_block = 16; continue;
}
// C line 31095
18 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 31093
19 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31092
20 => {
let _ = JS_ThrowStackOverflow(ctx);
vm_block = 19; continue;
}
// C line 31091
21 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 20 } else { 18 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31119. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn exec_module_list_cmp(mut p1: *const c_void, mut p2: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut m1: *mut JSModuleDef = core::ptr::null_mut();
let mut m2: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31123
1 => {
return (((((*(m1)).async_evaluation_timestamp) > ((*(m2)).async_evaluation_timestamp)) as i32)).wrapping_sub(((((*(m1)).async_evaluation_timestamp) < ((*(m2)).async_evaluation_timestamp)) as i32));
}
// C line 31122
2 => {
m2 = *(((p2) as *mut *mut JSModuleDef));
vm_block = 1; continue;
}
// C line 31121
3 => {
m1 = *(((p1) as *mut *mut JSModuleDef));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31139. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_module_execution_rejected(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut module: *mut JSModuleDef = core::ptr::null_mut();
let mut error: JSValue = core::mem::zeroed();
let mut i: i32 = 0;
let mut ret_val: JSValue = core::mem::zeroed();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut m_obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31181
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 31174
2 => {
vm_block = if ((((i) < ((*(module)).async_parent_modules_count)) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line 31174
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 31179
4 => {
let _ = JS_FreeValue(ctx, m_obj);
vm_block = 3; continue;
}
// C line 31177
5 => {
let _ = js_async_module_execution_rejected(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error), (0 as i32), core::ptr::addr_of_mut!(m_obj));
vm_block = 4; continue;
}
// C line 31176
6 => {
m_obj = JS_NewModuleValue(ctx, m);
vm_block = 5; continue;
}
// C line 31175
7 => {
m = *((*(module)).async_parent_modules).offset((i) as isize);
vm_block = 6; continue;
}
// C line 31174
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 31171
9 => {
let _ = JS_FreeValue(ctx, ret_val);
vm_block = 8; continue;
}
// C line 31169
10 => {
let _ = { let assigned = JS_Call(ctx, *(((*(module)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error)); ret_val = assigned; assigned };
vm_block = 9; continue;
}
// C line 31168
11 => {
let _ = if ((((!((((((*(module)).cycle_root) == (module)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 10; continue;
}
// C line 31166
12 => {
vm_block = if ((!((JS_IsUndefined((*(module)).promise)) != 0) as i32)) != 0 { 11 } else { 8 }; continue;
}
// C line 31164
13 => {
let _ = { let assigned = (0 as i32); (*(module)).async_evaluation = assigned; assigned };
vm_block = 12; continue;
}
// C line 31163
14 => {
let _ = { let assigned = (((JS_MODULE_STATUS_EVALUATED as i32)) as JSModuleStatus); (*(module)).status = (assigned) as u8; assigned };
vm_block = 13; continue;
}
// C line 31162
15 => {
let _ = { let assigned = JS_DupValue(ctx, error); (*(module)).eval_exception = assigned; assigned };
vm_block = 14; continue;
}
// C line 31161
16 => {
let _ = { let assigned = (1 as i32); (*(module)).eval_has_exception = (assigned) as i8; assigned };
vm_block = 15; continue;
}
// C line 31159
17 => {
let _ = if ((((!(((*(module)).async_evaluation) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 16; continue;
}
// C line 31158
18 => {
let _ = if ((((!(((!((((*(module)).eval_has_exception as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 17; continue;
}
// C line 31157
19 => {
let _ = if ((((!(((((((((*(module)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 18; continue;
}
// C line 31154
20 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 31153
21 => {
let _ = if ((((!((((*(module)).eval_has_exception as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 20; continue;
}
// C line 31152
22 => {
vm_block = if ((((((((*(module)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 31150
23 => {
return JS_ThrowStackOverflow(ctx);
}
// C line 31149
24 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 23 } else { 22 }; continue;
}
// C line 31143
25 => {
error = *(argv).offset(((0 as i32)) as isize);
vm_block = 24; continue;
}
// C line 31142
26 => {
module = ((((*(func_data).offset(((0 as i32)) as isize)).u).ptr) as *mut JSModuleDef);
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31184. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_module_execution_fulfilled(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut module: *mut JSModuleDef = core::ptr::null_mut();
let mut exec_list_s: ExecModuleList = core::mem::zeroed();
let mut exec_list: *mut ExecModuleList = core::ptr::null_mut();
let mut i: i32 = 0;
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut error: JSValue = core::mem::zeroed();
let mut m_obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31242
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 31241
2 => {
let _ = js_free(ctx, (((*(exec_list)).tab) as *mut c_void));
vm_block = 1; continue;
}
// C line 31217
3 => {
vm_block = if ((((i) < ((*(exec_list)).count)) as i32)) != 0 { 16 } else { 2 }; continue;
}
// C line 31217
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 31223
5 => {
let _ = if ((((!((((*(m)).eval_has_exception as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 4; continue;
}
// C line 31225
6 => {
let _ = js_execute_async_module(ctx, m);
vm_block = 4; continue;
}
// C line 31234
7 => {
let _ = JS_FreeValue(ctx, error);
vm_block = 4; continue;
}
// C line 31233
8 => {
let _ = JS_FreeValue(ctx, m_obj);
vm_block = 7; continue;
}
// C line 31230
9 => {
let _ = js_async_module_execution_rejected(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error), (0 as i32), core::ptr::addr_of_mut!(m_obj));
vm_block = 8; continue;
}
// C line 31229
10 => {
m_obj = JS_NewModuleValue(ctx, m);
vm_block = 9; continue;
}
// C line 31237
11 => {
let _ = js_set_module_evaluated(ctx, m);
vm_block = 4; continue;
}
// C line 31236
12 => {
let _ = { let assigned = (0 as i32); (*(m)).async_evaluation = assigned; assigned };
vm_block = 11; continue;
}
// C line 31228
13 => {
vm_block = if ((((js_execute_sync_module(ctx, m, core::ptr::addr_of_mut!(error))) < ((0 as i32))) as i32)) != 0 { 10 } else { 12 }; continue;
}
// C line 31224
14 => {
vm_block = if (((*(m)).has_tla as i32)) != 0 { 6 } else { 13 }; continue;
}
// C line 31222
15 => {
vm_block = if ((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0 { 5 } else { 14 }; continue;
}
// C line 31218
16 => {
m = *((*(exec_list)).tab).offset((i) as isize);
vm_block = 15; continue;
}
// C line 31217
17 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 31214
18 => {
let _ = crate::cutils::rqsort((((*(exec_list)).tab) as *mut c_void), (((*(exec_list)).count) as usize), (size_of::<*mut JSModuleDef>() as usize), exec_module_list_cmp, core::ptr::null_mut::<c_void>());
vm_block = 17; continue;
}
// C line 31210
19 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 31209
20 => {
let _ = js_free(ctx, (((*(exec_list)).tab) as *mut c_void));
vm_block = 19; continue;
}
// C line 31208
21 => {
vm_block = if ((((gather_available_ancestors(ctx, module, exec_list)) < ((0 as i32))) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 31206
22 => {
let _ = { let assigned = (0 as i32); (*(exec_list)).size = assigned; assigned };
vm_block = 21; continue;
}
// C line 31205
23 => {
let _ = { let assigned = (0 as i32); (*(exec_list)).count = assigned; assigned };
vm_block = 22; continue;
}
// C line 31204
24 => {
let _ = { let assigned = core::ptr::null_mut::<*mut JSModuleDef>(); (*(exec_list)).tab = assigned; assigned };
vm_block = 23; continue;
}
// C line 31202
25 => {
let _ = js_set_module_evaluated(ctx, module);
vm_block = 24; continue;
}
// C line 31201
26 => {
let _ = { let assigned = (0 as i32); (*(module)).async_evaluation = assigned; assigned };
vm_block = 25; continue;
}
// C line 31200
27 => {
let _ = if ((((!(((*(module)).async_evaluation) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 26; continue;
}
// C line 31199
28 => {
let _ = if ((((!(((!((((*(module)).eval_has_exception as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 27; continue;
}
// C line 31198
29 => {
let _ = if ((((!(((((((((*(module)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 31196
30 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 31195
31 => {
let _ = if ((((!((((*(module)).eval_has_exception as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 30; continue;
}
// C line 31194
32 => {
vm_block = if ((((((((*(module)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0 { 31 } else { 29 }; continue;
}
// C line 31188
33 => {
exec_list = core::ptr::addr_of_mut!(exec_list_s);
vm_block = 32; continue;
}
// C line 31187
34 => {
module = ((((*(func_data).offset(((0 as i32)) as isize)).u).ptr) as *mut JSModuleDef);
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31245. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_execute_async_module(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut promise: JSValue = core::mem::zeroed();
let mut m_obj: JSValue = core::mem::zeroed();
let mut resolve_funcs: [JSValue; 2] = core::mem::zeroed();
let mut ret_val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31264
1 => {
return (0 as i32);
}
// C line 31263
2 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 1; continue;
}
// C line 31262
3 => {
let _ = JS_FreeValue(ctx, *((resolve_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 2; continue;
}
// C line 31261
4 => {
let _ = JS_FreeValue(ctx, *((resolve_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 3; continue;
}
// C line 31260
5 => {
let _ = JS_FreeValue(ctx, m_obj);
vm_block = 4; continue;
}
// C line 31259
6 => {
let _ = JS_FreeValue(ctx, ret_val);
vm_block = 5; continue;
}
// C line 31258
7 => {
let _ = { let assigned = js_promise_then(ctx, promise, (2 as i32), (resolve_funcs).as_mut_ptr()); ret_val = assigned; assigned };
vm_block = 6; continue;
}
// C line 31257
8 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_async_module_execution_rejected), (0 as i32), (0 as i32), (1 as i32), core::ptr::addr_of_mut!(m_obj)); *((resolve_funcs).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 31256
9 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_async_module_execution_fulfilled), (0 as i32), (0 as i32), (1 as i32), core::ptr::addr_of_mut!(m_obj)); *((resolve_funcs).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 31255
10 => {
let _ = { let assigned = JS_NewModuleValue(ctx, m); m_obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 31254
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31253
12 => {
vm_block = if (JS_IsException(promise)) != 0 { 11 } else { 10 }; continue;
}
// C line 31252
13 => {
let _ = { let assigned = js_async_function_call(ctx, (*(m)).func_obj, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>(), (0 as i32)); promise = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31268. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_execute_sync_module(mut ctx: *mut JSContext, mut m: *mut JSModuleDef, mut pvalue: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut promise: JSValue = core::mem::zeroed();
let mut state: JSPromiseStateEnum = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31301
1 => {
return (0 as i32);
}
// C line 31300
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(pvalue) = assigned; assigned };
vm_block = 1; continue;
}
// C line 31277
3 => {
vm_block = 10; continue;
}
// C line 31276
4 => {
vm_block = if ((((((*(m)).init_func).expect("registered parser callback")(ctx, m)) < ((0 as i32))) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 31287
5 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 2; continue;
}
// C line 31291
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31290
7 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 6; continue;
}
// C line 31289
8 => {
let _ = { let assigned = JS_PromiseResult(ctx, promise); *(pvalue) = assigned; assigned };
vm_block = 7; continue;
}
// C line 31297
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31296 labels: fail
10 => {
let _ = { let assigned = JS_GetException(ctx); *(pvalue) = assigned; assigned };
vm_block = 9; continue;
}
// C line 31294
11 => {
let _ = JS_ThrowTypeError(ctx, c"promise is pending".as_ptr());
vm_block = 10; continue;
}
// C line 31293
12 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 11; continue;
}
// C line 31288
13 => {
vm_block = if ((((((state) as u32)) == ((((JS_PROMISE_REJECTED as i32)) as u32))) as i32)) != 0 { 8 } else { 12 }; continue;
}
// C line 31286
14 => {
vm_block = if ((((((state) as u32)) == ((((JS_PROMISE_FULFILLED as i32)) as u32))) as i32)) != 0 { 5 } else { 13 }; continue;
}
// C line 31285
15 => {
let _ = { let assigned = JS_PromiseState(ctx, promise); state = assigned; assigned };
vm_block = 14; continue;
}
// C line 31284
16 => {
vm_block = 10; continue;
}
// C line 31283
17 => {
vm_block = if (JS_IsException(promise)) != 0 { 16 } else { 15 }; continue;
}
// C line 31282
18 => {
let _ = { let assigned = js_async_function_call(ctx, (*(m)).func_obj, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>(), (0 as i32)); promise = assigned; assigned };
js_host_observe_internal_module_promise(ctx, m, promise);
vm_block = 17; continue;
}
// C line 31274
19 => {
vm_block = if ((*(m)).init_func).is_some() { 4 } else { 18 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31306. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_inner_module_evaluation(mut ctx: *mut JSContext, mut m: *mut JSModuleDef, mut index: i32, mut pstack_top: *mut *mut JSModuleDef, mut pvalue: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut m1: *mut JSModuleDef = core::ptr::null_mut();
let mut i: i32 = 0;
let mut rme: *mut JSReqModuleEntry = core::ptr::null_mut();
let mut vm_block: usize = 67;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31413
1 => {
return index;
}
// C line 31412
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(pvalue) = assigned; assigned };
vm_block = 1; continue;
}
// C line 31397
3 => {
vm_block = 11; continue;
}
// C line 31409
4 => {
vm_block = 2; continue;
}
// C line 31408
5 => {
vm_block = if ((((m1) == (m)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 31407
6 => {
let _ = { let assigned = m; (*(m1)).cycle_root = assigned; assigned };
vm_block = 5; continue;
}
// C line 31402
7 => {
let _ = { let assigned = (((JS_MODULE_STATUS_EVALUATED as i32)) as JSModuleStatus); (*(m1)).status = (assigned) as u8; assigned };
vm_block = 6; continue;
}
// C line 31404
8 => {
let _ = { let assigned = (((JS_MODULE_STATUS_EVALUATING_ASYNC as i32)) as JSModuleStatus); (*(m1)).status = (assigned) as u8; assigned };
vm_block = 6; continue;
}
// C line 31401
9 => {
vm_block = if ((!(((*(m1)).async_evaluation) != 0) as i32)) != 0 { 7 } else { 8 }; continue;
}
// C line 31400
10 => {
let _ = { let assigned = (*(m1)).stack_prev; *(pstack_top) = assigned; assigned };
vm_block = 9; continue;
}
// C line 31399
11 => {
let _ = { let assigned = *(pstack_top); m1 = assigned; assigned };
vm_block = 10; continue;
}
// C line 31396
12 => {
vm_block = if (((((*(m)).dfs_index) == ((*(m)).dfs_ancestor_index)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 31395
13 => {
let _ = if ((((!((((((*(m)).dfs_ancestor_index) <= ((*(m)).dfs_index)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 31382
14 => {
let _ = { let assigned = { let old = (*((*(ctx)).rt)).module_async_evaluation_next_timestamp; (*((*(ctx)).rt)).module_async_evaluation_next_timestamp = ((*((*(ctx)).rt)).module_async_evaluation_next_timestamp).wrapping_add(1); old }; (*(m)).async_evaluation_timestamp = assigned; assigned };
vm_block = 13; continue;
}
// C line 31381
15 => {
let _ = { let assigned = (1 as i32); (*(m)).async_evaluation = assigned; assigned };
vm_block = 14; continue;
}
// C line 31380
16 => {
let _ = if ((((!(((!(((*(m)).async_evaluation) != 0) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 15; continue;
}
// C line 31389
17 => {
let _ = js_execute_async_module(ctx, m);
vm_block = 13; continue;
}
// C line 31387
18 => {
let _ = { let assigned = { let old = (*((*(ctx)).rt)).module_async_evaluation_next_timestamp; (*((*(ctx)).rt)).module_async_evaluation_next_timestamp = ((*((*(ctx)).rt)).module_async_evaluation_next_timestamp).wrapping_add(1); old }; (*(m)).async_evaluation_timestamp = assigned; assigned };
vm_block = 17; continue;
}
// C line 31386
19 => {
let _ = { let assigned = (1 as i32); (*(m)).async_evaluation = assigned; assigned };
vm_block = 18; continue;
}
// C line 31385
20 => {
let _ = if ((((!(((!(((*(m)).async_evaluation) != 0) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 19; continue;
}
// C line 31392
21 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31391
22 => {
vm_block = if ((((js_execute_sync_module(ctx, m, pvalue)) < ((0 as i32))) as i32)) != 0 { 21 } else { 13 }; continue;
}
// C line 31384
23 => {
vm_block = if (((*(m)).has_tla as i32)) != 0 { 20 } else { 22 }; continue;
}
// C line 31379
24 => {
vm_block = if (((((*(m)).pending_async_dependencies) > ((0 as i32))) as i32)) != 0 { 16 } else { 23 }; continue;
}
// C line 31348
25 => {
vm_block = if ((((i) < ((*(m)).req_module_entries_count)) as i32)) != 0 { 45 } else { 24 }; continue;
}
// C line 31348
26 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 25; continue;
}
// C line 31375
27 => {
let _ = { let assigned = m; *((*(m1)).async_parent_modules).offset(({ let old = (*(m1)).async_parent_modules_count; (*(m1)).async_parent_modules_count = ((*(m1)).async_parent_modules_count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 26; continue;
}
// C line 31373
28 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31372
29 => {
let _ = { let assigned = JS_GetException(ctx); *(pvalue) = assigned; assigned };
vm_block = 28; continue;
}
// C line 31371
30 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(m1)).async_parent_modules)) as *mut *mut c_void), (((size_of::<*mut JSModuleDef>() as usize)) as i32), core::ptr::addr_of_mut!((*(m1)).async_parent_modules_size), ((*(m1)).async_parent_modules_count).wrapping_add((1 as i32)))) != 0 { 29 } else { 27 }; continue;
}
// C line 31370
31 => {
let _ = { let old = (*(m)).pending_async_dependencies; (*(m)).pending_async_dependencies = ((*(m)).pending_async_dependencies).wrapping_add(1); old };
vm_block = 30; continue;
}
// C line 31369
32 => {
vm_block = if ((*(m1)).async_evaluation) != 0 { 31 } else { 26 }; continue;
}
// C line 31358
33 => {
let _ = { let assigned = crate::cutils_header::min_int((*(m)).dfs_ancestor_index, (*(m1)).dfs_ancestor_index); (*(m)).dfs_ancestor_index = assigned; assigned };
vm_block = 32; continue;
}
// C line 31366
34 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31365
35 => {
let _ = { let assigned = JS_DupValue(ctx, (*(m1)).eval_exception); *(pvalue) = assigned; assigned };
vm_block = 34; continue;
}
// C line 31364
36 => {
vm_block = if (((*(m1)).eval_has_exception as i32)) != 0 { 35 } else { 32 }; continue;
}
// C line 31362
37 => {
let _ = if ((((!(((((((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0) || (((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 36; continue;
}
// C line 31361
38 => {
let _ = { let assigned = (*(m1)).cycle_root; m1 = assigned; assigned };
vm_block = 37; continue;
}
// C line 31357
39 => {
vm_block = if ((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING as i32))) as i32)) != 0 { 33 } else { 38 }; continue;
}
// C line 31354
40 => {
let _ = if ((((!(((((((((((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING as i32))) as i32)) != 0) || (((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 39; continue;
}
// C line 31353
41 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31352
42 => {
vm_block = if ((((index) < ((0 as i32))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 31351
43 => {
let _ = { let assigned = js_inner_module_evaluation(ctx, m1, index, pstack_top, pvalue); index = assigned; assigned };
vm_block = 42; continue;
}
// C line 31350
44 => {
let _ = { let assigned = (*(rme)).module; m1 = assigned; assigned };
vm_block = 43; continue;
}
// C line 31349
45 => {
rme = core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((i) as isize));
vm_block = 44; continue;
}
// C line 31348
46 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 25; continue;
}
// C line 31346
47 => {
let _ = { let assigned = m; *(pstack_top) = assigned; assigned };
vm_block = 46; continue;
}
// C line 31345
48 => {
let _ = { let assigned = *(pstack_top); (*(m)).stack_prev = assigned; assigned };
vm_block = 47; continue;
}
// C line 31343
49 => {
let _ = { let old = index; index = (index).wrapping_add(1); old };
vm_block = 48; continue;
}
// C line 31342
50 => {
let _ = { let assigned = (0 as i32); (*(m)).pending_async_dependencies = assigned; assigned };
vm_block = 49; continue;
}
// C line 31341
51 => {
let _ = { let assigned = index; (*(m)).dfs_ancestor_index = assigned; assigned };
vm_block = 50; continue;
}
// C line 31340
52 => {
let _ = { let assigned = index; (*(m)).dfs_index = assigned; assigned };
vm_block = 51; continue;
}
// C line 31339
53 => {
let _ = { let assigned = (((JS_MODULE_STATUS_EVALUATING as i32)) as JSModuleStatus); (*(m)).status = (assigned) as u8; assigned };
vm_block = 52; continue;
}
// C line 31337
54 => {
let _ = if ((((!(((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKED as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 53; continue;
}
// C line 31335
55 => {
return index;
}
// C line 31334
56 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(pvalue) = assigned; assigned };
vm_block = 55; continue;
}
// C line 31333
57 => {
vm_block = if ((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING as i32))) as i32)) != 0 { 56 } else { 54 }; continue;
}
// C line 31327
58 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31326
59 => {
let _ = { let assigned = JS_DupValue(ctx, (*(m)).eval_exception); *(pvalue) = assigned; assigned };
vm_block = 58; continue;
}
// C line 31330
60 => {
return index;
}
// C line 31329
61 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(pvalue) = assigned; assigned };
vm_block = 60; continue;
}
// C line 31325
62 => {
vm_block = if (((*(m)).eval_has_exception as i32)) != 0 { 59 } else { 61 }; continue;
}
// C line 31323
63 => {
vm_block = if ((((((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0 { 62 } else { 57 }; continue;
}
// C line 31320
64 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31319
65 => {
let _ = { let assigned = JS_GetException(ctx); *(pvalue) = assigned; assigned };
vm_block = 64; continue;
}
// C line 31318
66 => {
let _ = JS_ThrowStackOverflow(ctx);
vm_block = 65; continue;
}
// C line 31317
67 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 66 } else { 63 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31418. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_evaluate_module(mut ctx: *mut JSContext, mut m: *mut JSModuleDef) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut m1: *mut JSModuleDef = core::ptr::null_mut();
let mut stack_top: *mut JSModuleDef = core::ptr::null_mut();
let mut ret_val: JSValue = core::mem::zeroed();
let mut result: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31474
1 => {
return JS_DupValue(ctx, (*(m)).promise);
}
// C line 31456
2 => {
let _ = JS_FreeValue(ctx, ret_val);
vm_block = 1; continue;
}
// C line 31454
3 => {
let _ = { let assigned = JS_Call(ctx, *(((*(m)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!((*(m)).eval_exception)); ret_val = assigned; assigned };
vm_block = 2; continue;
}
// C line 31453
4 => {
let _ = if ((((!((((*(m)).eval_has_exception as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 3; continue;
}
// C line 31452
5 => {
let _ = if ((((!(((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 4; continue;
}
// C line 31451
6 => {
let _ = JS_FreeValue(ctx, result);
vm_block = 5; continue;
}
// C line 31442
7 => {
vm_block = if ((((stack_top) != (core::ptr::null_mut::<JSModuleDef>())) as i32)) != 0 { 14 } else { 6 }; continue;
}
// C line 31449
8 => {
let _ = { let assigned = (*(m1)).stack_prev; stack_top = assigned; assigned };
vm_block = 7; continue;
}
// C line 31448
9 => {
let _ = { let assigned = m; (*(m1)).cycle_root = assigned; assigned };
vm_block = 8; continue;
}
// C line 31447
10 => {
let _ = { let assigned = JS_DupValue(ctx, result); (*(m1)).eval_exception = assigned; assigned };
vm_block = 9; continue;
}
// C line 31446
11 => {
let _ = { let assigned = (1 as i32); (*(m1)).eval_has_exception = (assigned) as i8; assigned };
vm_block = 10; continue;
}
// C line 31445
12 => {
let _ = { let assigned = (((JS_MODULE_STATUS_EVALUATED as i32)) as JSModuleStatus); (*(m1)).status = (assigned) as u8; assigned };
vm_block = 11; continue;
}
// C line 31444
13 => {
let _ = if ((((!(((((((((*(m1)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 31443
14 => {
let _ = { let assigned = stack_top; m1 = assigned; assigned };
vm_block = 13; continue;
}
// C line 31472
15 => {
let _ = if ((((!(((((stack_top) == (core::ptr::null_mut::<JSModuleDef>())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 1; continue;
}
// C line 31470
16 => {
let _ = JS_FreeValue(ctx, ret_val);
vm_block = 15; continue;
}
// C line 31468
17 => {
let _ = { let assigned = JS_Call(ctx, *(((*(m)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(value)); ret_val = assigned; assigned };
vm_block = 16; continue;
}
// C line 31467
18 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
vm_block = 17; continue;
}
// C line 31466
19 => {
let _ = if ((((!(((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 18; continue;
}
// C line 31464
20 => {
vm_block = if ((!(((*(m)).async_evaluation) != 0) as i32)) != 0 { 19 } else { 15 }; continue;
}
// C line 31463
21 => {
let _ = if ((((!(((!((((*(m)).eval_has_exception as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 20; continue;
}
// C line 31461
22 => {
let _ = if ((((!(((((((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 21; continue;
}
// C line 31441
23 => {
vm_block = if ((((js_inner_module_evaluation(ctx, m, (0 as i32), core::ptr::addr_of_mut!(stack_top), core::ptr::addr_of_mut!(result))) < ((0 as i32))) as i32)) != 0 { 7 } else { 22 }; continue;
}
// C line 31440
24 => {
let _ = { let assigned = core::ptr::null_mut::<JSModuleDef>(); stack_top = assigned; assigned };
vm_block = 23; continue;
}
// C line 31438
25 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 31437
26 => {
vm_block = if (JS_IsException((*(m)).promise)) != 0 { 25 } else { 24 }; continue;
}
// C line 31436
27 => {
let _ = { let assigned = JS_NewPromiseCapability(ctx, ((*(m)).resolving_funcs).as_mut_ptr()); (*(m)).promise = assigned; assigned };
vm_block = 26; continue;
}
// C line 31435
28 => {
return JS_DupValue(ctx, (*(m)).promise);
}
// C line 31434
29 => {
vm_block = if ((!((JS_IsUndefined((*(m)).promise)) != 0) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 31431
30 => {
let _ = { let assigned = (*(m)).cycle_root; m = assigned; assigned };
vm_block = 29; continue;
}
// C line 31429
31 => {
vm_block = if ((((((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 31426
32 => {
let _ = if ((((!(((((((((((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_LINKED as i32))) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATING_ASYNC as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(m)).status as JSModuleStatus)) as i32)) == ((JS_MODULE_STATUS_EVALUATED as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31477. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_with_clause(mut s: *mut JSParseState, mut rme: *mut JSReqModuleEntry) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut key: JSAtom = 0;
let mut ret: i32 = 0;
let mut key_token_ptr: *const u8 = core::ptr::null();
let mut attributes: JSValue = core::mem::zeroed();
let mut vm_block: usize = 48;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31544
1 => {
return js_parse_expect(s, (125 as i32));
}
// C line 31542
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31539
3 => {
vm_block = if ((((((((((!((JS_IsUndefined((*(rme)).attributes)) != 0) as i32)) != 0) && (((*((*(ctx)).rt)).module_check_attrs).is_some())) as i32)) != 0) && (((((((*((*(ctx)).rt)).module_check_attrs).expect("registered parser callback")(ctx, (*((*(ctx)).rt)).module_loader_opaque, (*(rme)).attributes)) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 31488
4 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 43 } else { 3 }; continue;
}
// C line 31537
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31536
6 => {
vm_block = if (next_token(s)) != 0 { 5 } else { 4 }; continue;
}
// C line 31535
7 => {
vm_block = 3; continue;
}
// C line 31534
8 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 31533
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31532
10 => {
vm_block = if (next_token(s)) != 0 { 9 } else { 8 }; continue;
}
// C line 31531
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31530
12 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 31529
13 => {
let _ = JS_FreeAtom(ctx, key);
vm_block = 12; continue;
}
// C line 31527
14 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, (*(rme)).attributes, key, JS_DupValue(ctx, ((((*(s)).token).u).str).str), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 13; continue;
}
// C line 31523
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31525
16 => {
return js_parse_error(s, c"duplicate with key".as_ptr());
}
// C line 31522
17 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 15 } else { 16 }; continue;
}
// C line 31521
18 => {
let _ = JS_FreeAtom(ctx, key);
vm_block = 17; continue;
}
// C line 31520
19 => {
vm_block = if ((((ret) != ((0 as i32))) as i32)) != 0 { 18 } else { 14 }; continue;
}
// C line 31519
20 => {
let _ = { let assigned = JS_HasProperty(ctx, (*(rme)).attributes, key); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 31517
21 => {
let _ = { let assigned = attributes; (*(rme)).attributes = assigned; assigned };
vm_block = 20; continue;
}
// C line 31515
22 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31514
23 => {
let _ = JS_FreeAtom(ctx, key);
vm_block = 22; continue;
}
// C line 31513
24 => {
vm_block = if (JS_IsException(attributes)) != 0 { 23 } else { 21 }; continue;
}
// C line 31512
25 => {
attributes = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) });
vm_block = 24; continue;
}
// C line 31511
26 => {
vm_block = if (JS_IsUndefined((*(rme)).attributes)) != 0 { 25 } else { 20 }; continue;
}
// C line 31509
27 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31508
28 => {
let _ = js_parse_error_pos(s, key_token_ptr, c"string expected".as_ptr());
vm_block = 27; continue;
}
// C line 31507
29 => {
vm_block = if ((((((*(s)).token).val) != ((TOK_STRING as i32))) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 31505
30 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31504
31 => {
let _ = JS_FreeAtom(ctx, key);
vm_block = 30; continue;
}
// C line 31503
32 => {
vm_block = if (js_parse_expect(s, (58 as i32))) != 0 { 31 } else { 29 }; continue;
}
// C line 31502
33 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31501
34 => {
vm_block = if (next_token(s)) != 0 { 33 } else { 32 }; continue;
}
// C line 31493
35 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31492
36 => {
vm_block = if ((((key) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 31491
37 => {
let _ = { let assigned = JS_ValueToAtom(ctx, ((((*(s)).token).u).str).str); key = assigned; assigned };
vm_block = 36; continue;
}
// C line 31499
38 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); key = assigned; assigned };
vm_block = 34; continue;
}
// C line 31497
39 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31496
40 => {
let _ = js_parse_error(s, c"identifier expected".as_ptr());
vm_block = 39; continue;
}
// C line 31495
41 => {
vm_block = if ((!((token_is_ident(((*(s)).token).val)) != 0) as i32)) != 0 { 40 } else { 38 }; continue;
}
// C line 31490
42 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_STRING as i32))) as i32)) != 0 { 37 } else { 41 }; continue;
}
// C line 31489
43 => {
let _ = { let assigned = ((*(s)).token).ptr; key_token_ptr = assigned; assigned };
vm_block = 42; continue;
}
// C line 31487
44 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31486
45 => {
vm_block = if (js_parse_expect(s, (123 as i32))) != 0 { 44 } else { 4 }; continue;
}
// C line 31485
46 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31484
47 => {
vm_block = if (next_token(s)) != 0 { 46 } else { 45 }; continue;
}
// C line 31479
48 => {
ctx = (*(s)).ctx;
vm_block = 47; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31548. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_from_clause(mut s: *mut JSParseState, mut m: *mut JSModuleDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut module_name: JSAtom = 0;
let mut idx: i32 = 0;
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31579
1 => {
return idx;
}
// C line 31577
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31576
3 => {
vm_block = if (js_parse_with_clause(s, core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((idx) as isize)))) != 0 { 2 } else { 1 }; continue;
}
// C line 31575
4 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_WITH as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 31574
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31573
6 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 31572
7 => {
let _ = JS_FreeAtom((*(s)).ctx, module_name);
vm_block = 6; continue;
}
// C line 31571
8 => {
let _ = { let assigned = add_req_module_entry((*(s)).ctx, m, module_name); idx = assigned; assigned };
vm_block = 7; continue;
}
// C line 31568
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31567
10 => {
let _ = JS_FreeAtom((*(s)).ctx, module_name);
vm_block = 9; continue;
}
// C line 31566
11 => {
vm_block = if (next_token(s)) != 0 { 10 } else { 8 }; continue;
}
// C line 31565
12 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31564
13 => {
vm_block = if ((((module_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 31563
14 => {
let _ = { let assigned = JS_ValueToAtom((*(s)).ctx, ((((*(s)).token).u).str).str); module_name = assigned; assigned };
vm_block = 13; continue;
}
// C line 31561
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31560
16 => {
let _ = js_parse_error(s, c"string expected".as_ptr());
vm_block = 15; continue;
}
// C line 31559
17 => {
vm_block = if ((((((*(s)).token).val) != ((TOK_STRING as i32))) as i32)) != 0 { 16 } else { 14 }; continue;
}
// C line 31558
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31557
19 => {
vm_block = if (next_token(s)) != 0 { 18 } else { 17 }; continue;
}
// C line 31555
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31554
21 => {
let _ = js_parse_error(s, c"from clause expected".as_ptr());
vm_block = 20; continue;
}
// C line 31553
22 => {
vm_block = if ((!((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_from as i32)) as JSAtom))) != 0) as i32)) != 0 { 21 } else { 19 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31582. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_export(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut local_name: JSAtom = 0;
let mut export_name: JSAtom = 0;
let mut first_export: i32 = 0;
let mut idx: i32 = 0;
let mut i: i32 = 0;
let mut tok: i32 = 0;
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut vm_block: usize = 104;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31738
1 => {
return js_parse_expect_semi(s);
}
// C line 31736
2 => {
return js_parse_error(s, c"invalid export syntax".as_ptr());
}
// C line 31734
3 => {
return js_parse_var(s, (1 as i32), tok, (1 as i32));
}
// C line 31730
4 => {
vm_block = 1; continue;
}
// C line 31729
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31727
6 => {
vm_block = if ((!(!(add_export_entry(s, m, local_name, (((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom), (((JS_EXPORT_TYPE_LOCAL as i32)) as JSExportTypeEnum))).is_null()) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 31725
7 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 6; continue;
}
// C line 31724
8 => {
let _ = emit_atom(s, local_name);
vm_block = 7; continue;
}
// C line 31723
9 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 8; continue;
}
// C line 31722
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31721
11 => {
vm_block = if ((((define_var(s, (*(s)).cur_func, local_name, (((JS_VAR_DEF_LET as i32)) as JSVarDefEnum))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 31720
12 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM__default_ as i32)) as JSAtom); local_name = assigned; assigned };
vm_block = 11; continue;
}
// C line 31716
13 => {
let _ = set_object_name(s, (((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom));
vm_block = 12; continue;
}
// C line 31703
14 => {
return js_parse_class(s, (0 as i32), (((JS_PARSE_EXPORT_DEFAULT as i32)) as JSParseExportEnum));
}
// C line 31707
15 => {
return js_parse_function_decl2(s, (((JS_PARSE_FUNC_STATEMENT as i32)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), ((*(s)).token).ptr, (((JS_PARSE_EXPORT_DEFAULT as i32)) as JSParseExportEnum), core::ptr::null_mut::<*mut JSFunctionDef>());
}
// C line 31713
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31712
17 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 16 } else { 13 }; continue;
}
// C line 31704
18 => {
vm_block = if ((((((((((*(s)).token).val) == ((TOK_FUNCTION as i32))) as i32)) != 0) || ((((((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0) && (((((peek_token(s, (1 as i32))) == ((TOK_FUNCTION as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 15 } else { 17 }; continue;
}
// C line 31702
19 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_CLASS as i32))) as i32)) != 0 { 14 } else { 18 }; continue;
}
// C line 31700
20 => {
vm_block = 1; continue;
}
// C line 31692
21 => {
let _ = { let assigned = idx; ((*(me)).u).req_module_idx = assigned; assigned };
vm_block = 20; continue;
}
// C line 31691
22 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31690
23 => {
vm_block = if ((!(!(me).is_null()) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 31689
24 => {
let _ = JS_FreeAtom(ctx, export_name);
vm_block = 23; continue;
}
// C line 31687
25 => {
let _ = { let assigned = add_export_entry(s, m, (((crate::quickjs_atom::JS_ATOM__star_ as i32)) as JSAtom), export_name, (((JS_EXPORT_TYPE_INDIRECT as i32)) as JSExportTypeEnum)); me = assigned; assigned };
vm_block = 24; continue;
}
// C line 31686
26 => {
vm_block = 67; continue;
}
// C line 31685
27 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 31684
28 => {
let _ = { let assigned = js_parse_from_clause(s, m); idx = assigned; assigned };
vm_block = 27; continue;
}
// C line 31683
29 => {
vm_block = 67; continue;
}
// C line 31682
30 => {
vm_block = if (next_token(s)) != 0 { 29 } else { 28 }; continue;
}
// C line 31681
31 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); export_name = assigned; assigned };
vm_block = 30; continue;
}
// C line 31679
32 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31678
33 => {
let _ = js_parse_error(s, c"identifier expected".as_ptr());
vm_block = 32; continue;
}
// C line 31677
34 => {
vm_block = if ((!((token_is_ident(((*(s)).token).val)) != 0) as i32)) != 0 { 33 } else { 31 }; continue;
}
// C line 31676
35 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31675
36 => {
vm_block = if (next_token(s)) != 0 { 35 } else { 34 }; continue;
}
// C line 31698
37 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31697
38 => {
vm_block = if ((((add_star_export_entry(ctx, m, idx)) < ((0 as i32))) as i32)) != 0 { 37 } else { 20 }; continue;
}
// C line 31696
39 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31695
40 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 31694
41 => {
let _ = { let assigned = js_parse_from_clause(s, m); idx = assigned; assigned };
vm_block = 40; continue;
}
// C line 31673
42 => {
vm_block = if (token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_as as i32)) as JSAtom))) != 0 { 36 } else { 41 }; continue;
}
// C line 31671
43 => {
vm_block = 1; continue;
}
// C line 31665
44 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 48 } else { 43 }; continue;
}
// C line 31665
45 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 31668
46 => {
let _ = { let assigned = idx; ((*(me)).u).req_module_idx = assigned; assigned };
vm_block = 45; continue;
}
// C line 31667
47 => {
let _ = { let assigned = (((JS_EXPORT_TYPE_INDIRECT as i32)) as JSExportTypeEnum); (*(me)).export_type = assigned; assigned };
vm_block = 46; continue;
}
// C line 31666
48 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize)); me = assigned; assigned };
vm_block = 47; continue;
}
// C line 31665
49 => {
let _ = { let assigned = first_export; i = assigned; assigned };
vm_block = 44; continue;
}
// C line 31664
50 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31663
51 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 50 } else { 49 }; continue;
}
// C line 31662
52 => {
let _ = { let assigned = js_parse_from_clause(s, m); idx = assigned; assigned };
vm_block = 51; continue;
}
// C line 31661
53 => {
vm_block = if (token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_from as i32)) as JSAtom))) != 0 { 52 } else { 43 }; continue;
}
// C line 31660
54 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31659
55 => {
vm_block = if (js_parse_expect(s, (125 as i32))) != 0 { 54 } else { 53 }; continue;
}
// C line 31611
56 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 91 } else { 55 }; continue;
}
// C line 31657
57 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31656
58 => {
vm_block = if (next_token(s)) != 0 { 57 } else { 56 }; continue;
}
// C line 31655
59 => {
vm_block = 55; continue;
}
// C line 31654
60 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 59 } else { 58 }; continue;
}
// C line 31653
61 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31652
62 => {
vm_block = if ((!(!(me).is_null()) as i32)) != 0 { 61 } else { 60 }; continue;
}
// C line 31651
63 => {
let _ = JS_FreeAtom(ctx, export_name);
vm_block = 62; continue;
}
// C line 31650
64 => {
let _ = JS_FreeAtom(ctx, local_name);
vm_block = 63; continue;
}
// C line 31648
65 => {
let _ = { let assigned = add_export_entry(s, m, local_name, export_name, (((JS_EXPORT_TYPE_LOCAL as i32)) as JSExportTypeEnum)); me = assigned; assigned };
vm_block = 64; continue;
}
// C line 31643
66 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31642 labels: fail1
67 => {
let _ = JS_FreeAtom(ctx, export_name);
vm_block = 66; continue;
}
// C line 31640 labels: fail
68 => {
let _ = JS_FreeAtom(ctx, local_name);
vm_block = 67; continue;
}
// C line 31638
69 => {
vm_block = if (next_token(s)) != 0 { 68 } else { 65 }; continue;
}
// C line 31630
70 => {
vm_block = 68; continue;
}
// C line 31629
71 => {
vm_block = if ((((export_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 70 } else { 69 }; continue;
}
// C line 31628
72 => {
let _ = { let assigned = JS_ValueToAtom((*(s)).ctx, ((((*(s)).token).u).str).str); export_name = assigned; assigned };
vm_block = 71; continue;
}
// C line 31626
73 => {
vm_block = 68; continue;
}
// C line 31625
74 => {
let _ = js_parse_error(s, c"contains unpaired surrogate".as_ptr());
vm_block = 73; continue;
}
// C line 31624
75 => {
vm_block = if ((((js_string_find_invalid_codepoint(((((((((*(s)).token).u).str).str).u).ptr) as *mut JSString))) >= ((0 as i32))) as i32)) != 0 { 74 } else { 72 }; continue;
}
// C line 31636
76 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); export_name = assigned; assigned };
vm_block = 69; continue;
}
// C line 31634
77 => {
vm_block = 68; continue;
}
// C line 31633
78 => {
let _ = js_parse_error(s, c"identifier expected".as_ptr());
vm_block = 77; continue;
}
// C line 31632
79 => {
vm_block = if ((!((token_is_ident(((*(s)).token).val)) != 0) as i32)) != 0 { 78 } else { 76 }; continue;
}
// C line 31623
80 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_STRING as i32))) as i32)) != 0 { 75 } else { 79 }; continue;
}
// C line 31622
81 => {
vm_block = 68; continue;
}
// C line 31621
82 => {
vm_block = if (next_token(s)) != 0 { 81 } else { 80 }; continue;
}
// C line 31646
83 => {
let _ = { let assigned = JS_DupAtom(ctx, local_name); export_name = assigned; assigned };
vm_block = 65; continue;
}
// C line 31620
84 => {
vm_block = if (token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_as as i32)) as JSAtom))) != 0 { 82 } else { 83 }; continue;
}
// C line 31619
85 => {
vm_block = 68; continue;
}
// C line 31618
86 => {
vm_block = if (next_token(s)) != 0 { 85 } else { 84 }; continue;
}
// C line 31617
87 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); export_name = assigned; assigned };
vm_block = 86; continue;
}
// C line 31616
88 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); local_name = assigned; assigned };
vm_block = 87; continue;
}
// C line 31614
89 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31613
90 => {
let _ = js_parse_error(s, c"identifier expected".as_ptr());
vm_block = 89; continue;
}
// C line 31612
91 => {
vm_block = if ((!((token_is_ident(((*(s)).token).val)) != 0) as i32)) != 0 { 90 } else { 88 }; continue;
}
// C line 31610
92 => {
let _ = { let assigned = (*(m)).export_entries_count; first_export = assigned; assigned };
vm_block = 56; continue;
}
// C line 31608
93 => {
vm_block = match tok { x if x == (TOK_CONST as i32) => 3, x if x == (TOK_LET as i32) => 3, x if x == (TOK_VAR as i32) => 3, x if x == (TOK_DEFAULT as i32) => 19, x if x == (42 as i32) => 42, x if x == (123 as i32) => 92, _ => 2, }; continue;
}
// C line 31606
94 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31605
95 => {
vm_block = if (next_token(s)) != 0 { 94 } else { 93 }; continue;
}
// C line 31595
96 => {
return js_parse_class(s, (0 as i32), (((JS_PARSE_EXPORT_NAMED as i32)) as JSParseExportEnum));
}
// C line 31599
97 => {
return js_parse_function_decl2(s, (((JS_PARSE_FUNC_STATEMENT as i32)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), ((*(s)).token).ptr, (((JS_PARSE_EXPORT_NAMED as i32)) as JSParseExportEnum), core::ptr::null_mut::<*mut JSFunctionDef>());
}
// C line 31596
98 => {
vm_block = if ((((((((tok) == ((TOK_FUNCTION as i32))) as i32)) != 0) || ((((((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0) && (((((peek_token(s, (1 as i32))) == ((TOK_FUNCTION as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 97 } else { 95 }; continue;
}
// C line 31594
99 => {
vm_block = if ((((tok) == ((TOK_CLASS as i32))) as i32)) != 0 { 96 } else { 98 }; continue;
}
// C line 31593
100 => {
let _ = { let assigned = ((*(s)).token).val; tok = assigned; assigned };
vm_block = 99; continue;
}
// C line 31591
101 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31590
102 => {
vm_block = if (next_token(s)) != 0 { 101 } else { 100 }; continue;
}
// C line 31585
103 => {
m = (*((*(s)).cur_func)).module;
vm_block = 102; continue;
}
// C line 31584
104 => {
ctx = (*(s)).ctx;
vm_block = 103; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31747. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_import(mut s: *mut JSParseState, mut m: *mut JSModuleDef, mut local_name: JSAtom, mut import_name: JSAtom, mut is_star: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut i: i32 = 0;
let mut var_idx: i32 = 0;
let mut mi: *mut JSImportEntry = core::ptr::null_mut();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31779
1 => {
return (0 as i32);
}
// C line 31778
2 => {
let _ = { let assigned = is_star; (*(mi)).is_star = assigned; assigned };
vm_block = 1; continue;
}
// C line 31777
3 => {
let _ = { let assigned = var_idx; (*(mi)).var_idx = assigned; assigned };
vm_block = 2; continue;
}
// C line 31776
4 => {
let _ = { let assigned = JS_DupAtom(ctx, import_name); (*(mi)).import_name = assigned; assigned };
vm_block = 3; continue;
}
// C line 31775
5 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(m)).import_entries).offset(({ let old = (*(m)).import_entries_count; (*(m)).import_entries_count = ((*(m)).import_entries_count).wrapping_add(1); old }) as isize)); mi = assigned; assigned };
vm_block = 4; continue;
}
// C line 31774
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31770
7 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(m)).import_entries)) as *mut *mut c_void), (((size_of::<JSImportEntry>() as usize)) as i32), core::ptr::addr_of_mut!((*(m)).import_entries_size), ((*(m)).import_entries_count).wrapping_add((1 as i32)))) != 0 { 6 } else { 5 }; continue;
}
// C line 31769
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31768
9 => {
vm_block = if ((((var_idx) < ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 31764
10 => {
let _ = { let assigned = add_closure_var(ctx, (*(s)).cur_func, ((if (is_star) != 0 { (JS_CLOSURE_MODULE_DECL as i32) } else { (JS_CLOSURE_MODULE_IMPORT as i32) }) as JSClosureTypeEnum), (*(m)).import_entries_count, local_name, (1 as i32), (1 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); var_idx = assigned; assigned };
vm_block = 9; continue;
}
// C line 31758
11 => {
vm_block = if ((((i) < ((*((*(s)).cur_func)).closure_var_count)) as i32)) != 0 { 14 } else { 10 }; continue;
}
// C line 31758
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 31760
13 => {
return js_parse_error(s, c"duplicate import binding".as_ptr());
}
// C line 31759
14 => {
vm_block = if (((((*((*((*(s)).cur_func)).closure_var).offset((i) as isize)).var_name) == (local_name)) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 31758
15 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 31757
16 => {
vm_block = if ((((local_name) != ((((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom))) as i32)) != 0 { 15 } else { 10 }; continue;
}
// C line 31755
17 => {
return js_parse_error(s, c"invalid import binding".as_ptr());
}
// C line 31754
18 => {
vm_block = if ((((((((local_name) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0) || (((((local_name) == ((((crate::quickjs_atom::JS_ATOM_eval as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 31750
19 => {
ctx = (*(s)).ctx;
vm_block = 18; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31782. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_import(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut local_name: JSAtom = 0;
let mut import_name: JSAtom = 0;
let mut module_name: JSAtom = 0;
let mut first_import: i32 = 0;
let mut i: i32 = 0;
let mut idx: i32 = 0;
let mut is_string: i32 = 0;
let mut vm_block: usize = 103;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31914
1 => {
return js_parse_expect_semi(s);
}
// C line 31911
2 => {
vm_block = if ((((i) < ((*(m)).import_entries_count)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 31911
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 31912
4 => {
let _ = { let assigned = idx; (*((*(m)).import_entries).offset((i) as isize)).req_module_idx = assigned; assigned };
vm_block = 3; continue;
}
// C line 31911
5 => {
let _ = { let assigned = first_import; i = assigned; assigned };
vm_block = 2; continue;
}
// C line 31807
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31806
7 => {
vm_block = if (js_parse_with_clause(s, core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset((idx) as isize)))) != 0 { 6 } else { 5 }; continue;
}
// C line 31805
8 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_WITH as i32))) as i32)) != 0 { 7 } else { 5 }; continue;
}
// C line 31804
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31803
10 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 31802
11 => {
let _ = JS_FreeAtom(ctx, module_name);
vm_block = 10; continue;
}
// C line 31801
12 => {
let _ = { let assigned = add_req_module_entry(ctx, m, module_name); idx = assigned; assigned };
vm_block = 11; continue;
}
// C line 31799
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31798
14 => {
let _ = JS_FreeAtom(ctx, module_name);
vm_block = 13; continue;
}
// C line 31797
15 => {
vm_block = if (next_token(s)) != 0 { 14 } else { 12 }; continue;
}
// C line 31796
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31795
17 => {
vm_block = if ((((module_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 31794
18 => {
let _ = { let assigned = JS_ValueToAtom(ctx, ((((*(s)).token).u).str).str); module_name = assigned; assigned };
vm_block = 17; continue;
}
// C line 31909
19 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31908
20 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 19 } else { 5 }; continue;
}
// C line 31907 labels: end_import_clause
21 => {
let _ = { let assigned = js_parse_from_clause(s, m); idx = assigned; assigned };
vm_block = 20; continue;
}
// C line 31847
22 => {
let _ = JS_FreeAtom(ctx, local_name);
vm_block = 21; continue;
}
// C line 31846
23 => {
vm_block = 60; continue;
}
// C line 31845
24 => {
vm_block = if (add_import(s, m, local_name, import_name, (1 as i32))) != 0 { 23 } else { 22 }; continue;
}
// C line 31844
25 => {
vm_block = 60; continue;
}
// C line 31843
26 => {
vm_block = if (next_token(s)) != 0 { 25 } else { 24 }; continue;
}
// C line 31842
27 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM__star_ as i32)) as JSAtom); import_name = assigned; assigned };
vm_block = 26; continue;
}
// C line 31841
28 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); local_name = assigned; assigned };
vm_block = 27; continue;
}
// C line 31839
29 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31838
30 => {
let _ = js_parse_error(s, c"identifier expected".as_ptr());
vm_block = 29; continue;
}
// C line 31837
31 => {
vm_block = if ((!((token_is_ident(((*(s)).token).val)) != 0) as i32)) != 0 { 30 } else { 28 }; continue;
}
// C line 31836
32 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31835
33 => {
vm_block = if (next_token(s)) != 0 { 32 } else { 31 }; continue;
}
// C line 31834
34 => {
return js_parse_error(s, c"expecting 'as'".as_ptr());
}
// C line 31833
35 => {
vm_block = if ((!((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_as as i32)) as JSAtom))) != 0) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 31832
36 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31831
37 => {
vm_block = if (next_token(s)) != 0 { 36 } else { 35 }; continue;
}
// C line 31904
38 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31903
39 => {
vm_block = if (js_parse_expect(s, (125 as i32))) != 0 { 38 } else { 21 }; continue;
}
// C line 31852
40 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 79 } else { 39 }; continue;
}
// C line 31901
41 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31900
42 => {
vm_block = if (next_token(s)) != 0 { 41 } else { 40 }; continue;
}
// C line 31899
43 => {
vm_block = 39; continue;
}
// C line 31898
44 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 43 } else { 42 }; continue;
}
// C line 31897
45 => {
let _ = JS_FreeAtom(ctx, import_name);
vm_block = 44; continue;
}
// C line 31896
46 => {
let _ = JS_FreeAtom(ctx, local_name);
vm_block = 45; continue;
}
// C line 31895
47 => {
vm_block = 60; continue;
}
// C line 31894
48 => {
vm_block = if (add_import(s, m, local_name, import_name, (0 as i32))) != 0 { 47 } else { 46 }; continue;
}
// C line 31883
49 => {
vm_block = 60; continue;
}
// C line 31882
50 => {
vm_block = if (next_token(s)) != 0 { 49 } else { 48 }; continue;
}
// C line 31881
51 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); local_name = assigned; assigned };
vm_block = 50; continue;
}
// C line 31879
52 => {
vm_block = 60; continue;
}
// C line 31878
53 => {
let _ = js_parse_error(s, c"identifier expected".as_ptr());
vm_block = 52; continue;
}
// C line 31877
54 => {
vm_block = if ((!((token_is_ident(((*(s)).token).val)) != 0) as i32)) != 0 { 53 } else { 51 }; continue;
}
// C line 31876
55 => {
vm_block = 60; continue;
}
// C line 31875
56 => {
vm_block = if (next_token(s)) != 0 { 55 } else { 54 }; continue;
}
// C line 31892
57 => {
let _ = { let assigned = JS_DupAtom(ctx, import_name); local_name = assigned; assigned };
vm_block = 48; continue;
}
// C line 31890
58 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31889
59 => {
let _ = JS_FreeAtom(ctx, import_name);
vm_block = 58; continue;
}
// C line 31888 labels: fail
60 => {
let _ = JS_FreeAtom(ctx, local_name);
vm_block = 59; continue;
}
// C line 31886
61 => {
let _ = js_parse_error(s, c"expecting 'as'".as_ptr());
vm_block = 60; continue;
}
// C line 31885
62 => {
vm_block = if (is_string) != 0 { 61 } else { 57 }; continue;
}
// C line 31874
63 => {
vm_block = if (token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_as as i32)) as JSAtom))) != 0 { 56 } else { 62 }; continue;
}
// C line 31873
64 => {
vm_block = 60; continue;
}
// C line 31872
65 => {
vm_block = if (next_token(s)) != 0 { 64 } else { 63 }; continue;
}
// C line 31871
66 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); local_name = assigned; assigned };
vm_block = 65; continue;
}
// C line 31862
67 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31861
68 => {
vm_block = if ((((import_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 67 } else { 66 }; continue;
}
// C line 31860
69 => {
let _ = { let assigned = JS_ValueToAtom((*(s)).ctx, ((((*(s)).token).u).str).str); import_name = assigned; assigned };
vm_block = 68; continue;
}
// C line 31858
70 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31857
71 => {
let _ = js_parse_error(s, c"contains unpaired surrogate".as_ptr());
vm_block = 70; continue;
}
// C line 31856
72 => {
vm_block = if ((((js_string_find_invalid_codepoint(((((((((*(s)).token).u).str).str).u).ptr) as *mut JSString))) >= ((0 as i32))) as i32)) != 0 { 71 } else { 69 }; continue;
}
// C line 31855
73 => {
let _ = { let assigned = (1 as i32); is_string = assigned; assigned };
vm_block = 72; continue;
}
// C line 31869
74 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); import_name = assigned; assigned };
vm_block = 66; continue;
}
// C line 31867
75 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31866
76 => {
let _ = js_parse_error(s, c"identifier expected".as_ptr());
vm_block = 75; continue;
}
// C line 31865
77 => {
vm_block = if ((!((token_is_ident(((*(s)).token).val)) != 0) as i32)) != 0 { 76 } else { 74 }; continue;
}
// C line 31864
78 => {
let _ = { let assigned = (0 as i32); is_string = assigned; assigned };
vm_block = 77; continue;
}
// C line 31854
79 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_STRING as i32))) as i32)) != 0 { 73 } else { 78 }; continue;
}
// C line 31850
80 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31849
81 => {
vm_block = if (next_token(s)) != 0 { 80 } else { 40 }; continue;
}
// C line 31848
82 => {
vm_block = if ((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0 { 81 } else { 21 }; continue;
}
// C line 31829
83 => {
vm_block = if ((((((*(s)).token).val) == ((42 as i32))) as i32)) != 0 { 37 } else { 82 }; continue;
}
// C line 31826
84 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31825
85 => {
vm_block = if (next_token(s)) != 0 { 84 } else { 83 }; continue;
}
// C line 31824
86 => {
vm_block = 21; continue;
}
// C line 31823
87 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 86 } else { 85 }; continue;
}
// C line 31821
88 => {
let _ = JS_FreeAtom(ctx, local_name);
vm_block = 87; continue;
}
// C line 31820
89 => {
vm_block = 60; continue;
}
// C line 31819
90 => {
vm_block = if (add_import(s, m, local_name, import_name, (0 as i32))) != 0 { 89 } else { 88 }; continue;
}
// C line 31818
91 => {
vm_block = 60; continue;
}
// C line 31817
92 => {
vm_block = if (next_token(s)) != 0 { 91 } else { 90 }; continue;
}
// C line 31816
93 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom); import_name = assigned; assigned };
vm_block = 92; continue;
}
// C line 31815
94 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); local_name = assigned; assigned };
vm_block = 93; continue;
}
// C line 31812
95 => {
return js_parse_error_reserved_identifier(s);
}
// C line 31811
96 => {
vm_block = if (((((*(s)).token).u).ident).is_reserved) != 0 { 95 } else { 94 }; continue;
}
// C line 31810
97 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0 { 96 } else { 83 }; continue;
}
// C line 31793
98 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_STRING as i32))) as i32)) != 0 { 18 } else { 97 }; continue;
}
// C line 31792
99 => {
let _ = { let assigned = (*(m)).import_entries_count; first_import = assigned; assigned };
vm_block = 98; continue;
}
// C line 31790
100 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31789
101 => {
vm_block = if (next_token(s)) != 0 { 100 } else { 99 }; continue;
}
// C line 31785
102 => {
m = (*((*(s)).cur_func)).module;
vm_block = 101; continue;
}
// C line 31784
103 => {
ctx = (*(s)).ctx;
vm_block = 102; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31917. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_source_element(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut tok: i32 = 0;
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 31942
1 => {
return (0 as i32);
}
// C line 31928
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31925
3 => {
vm_block = if (js_parse_function_decl(s, (((JS_PARSE_FUNC_STATEMENT as i32)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), ((*(s)).token).ptr)) != 0 { 2 } else { 1 }; continue;
}
// C line 31931
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31930
5 => {
vm_block = if (js_parse_export(s)) != 0 { 4 } else { 1 }; continue;
}
// C line 31937
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31936
7 => {
vm_block = if (js_parse_import(s)) != 0 { 6 } else { 1 }; continue;
}
// C line 31940
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 31939
9 => {
vm_block = if (js_parse_statement_or_decl(s, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) != 0 { 8 } else { 1 }; continue;
}
// C line 31932
10 => {
vm_block = if ((((((((((((((*(s)).token).val) == ((TOK_IMPORT as i32))) as i32)) != 0) && (!((*(fd)).module).is_null())) as i32)) != 0) && ((((((((({ let assigned = peek_token(s, (0 as i32)); tok = assigned; assigned }) != ((40 as i32))) as i32)) != 0) && (((((tok) != ((46 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 7 } else { 9 }; continue;
}
// C line 31929
11 => {
vm_block = if ((((((((((*(s)).token).val) == ((TOK_EXPORT as i32))) as i32)) != 0) && (!((*(fd)).module).is_null())) as i32)) != 0 { 5 } else { 10 }; continue;
}
// C line 31922
12 => {
vm_block = if ((((((((((*(s)).token).val) == ((TOK_FUNCTION as i32))) as i32)) != 0) || ((((((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0) && (((((peek_token(s, (1 as i32))) == ((TOK_FUNCTION as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 3 } else { 11 }; continue;
}
// C line 31919
13 => {
fd = (*(s)).cur_func;
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}
