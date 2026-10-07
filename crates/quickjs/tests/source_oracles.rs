//! C runs as an independent test executable; no runtime FFI.
use std::{path::PathBuf, process::Command};
#[path = "unicode_table_dump.rs"]
mod unicode;
fn oracle(file: &str, extra: &[&str]) -> Vec<u8> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = crate_root.join("../../vendor/quickjs-2026-06-04");
    let directory = std::env::temp_dir().join(format!("quickjs-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join(file);
    let short = if cfg!(feature = "short-opcodes") {
        "-DSHORT_OPCODES=1"
    } else {
        "-DSHORT_OPCODES=0"
    };
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", short, "-I"])
        .arg(upstream)
        .arg(crate_root.join("tests").join(format!("{file}.c")))
        .args(extra)
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = Command::new(&output).output().unwrap();
    let _ = std::fs::remove_file(&output);
    assert!(
        result.status.success(),
        "C oracle crashed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    result.stdout
}
fn compare(c: Vec<u8>, rust: Vec<u8>) {
    assert_eq!(c.len(), rust.len(), "different serialized length");
    if let Some(index) = c.iter().zip(&rust).position(|(a, b)| a != b) {
        panic!(
            "C/Rust differ at byte {index}: C={}, Rust={}",
            c[index], rust[index]
        );
    }
}
#[test]
fn complete_unicode_header_matches_official_c() {
    compare(oracle("unicode_table_oracle", &[]), unicode::dump_unicode());
}
#[test]
fn atom_opcode_regexp_and_list_match_official_c() {
    compare(oracle("core_tables_oracle", &[]), dump_core());
}
fn dump_core() -> Vec<u8> {
    use quickjs::{libregexp_opcode::*, list::*, quickjs_atom::*, quickjs_opcode::*};
    let mut o = Vec::new();
    fn num(o: &mut Vec<u8>, v: u32) {
        o.extend(v.to_le_bytes());
    }
    num(&mut o, JS_ATOM_END);
    o.extend(js_atom_init);
    num(&mut o, JS_ATOM_null);
    num(&mut o, JS_ATOM_false);
    num(&mut o, JS_ATOM_true);
    num(&mut o, JS_ATOM_if);
    num(&mut o, JS_ATOM_else);
    num(&mut o, JS_ATOM_return);
    num(&mut o, JS_ATOM_var);
    num(&mut o, JS_ATOM_this);
    num(&mut o, JS_ATOM_delete);
    num(&mut o, JS_ATOM_void);
    num(&mut o, JS_ATOM_typeof);
    num(&mut o, JS_ATOM_new);
    num(&mut o, JS_ATOM_in);
    num(&mut o, JS_ATOM_instanceof);
    num(&mut o, JS_ATOM_do);
    num(&mut o, JS_ATOM_while);
    num(&mut o, JS_ATOM_for);
    num(&mut o, JS_ATOM_break);
    num(&mut o, JS_ATOM_continue);
    num(&mut o, JS_ATOM_switch);
    num(&mut o, JS_ATOM_case);
    num(&mut o, JS_ATOM_default);
    num(&mut o, JS_ATOM_throw);
    num(&mut o, JS_ATOM_try);
    num(&mut o, JS_ATOM_catch);
    num(&mut o, JS_ATOM_finally);
    num(&mut o, JS_ATOM_function);
    num(&mut o, JS_ATOM_debugger);
    num(&mut o, JS_ATOM_with);
    num(&mut o, JS_ATOM_class);
    num(&mut o, JS_ATOM_const);
    num(&mut o, JS_ATOM_enum);
    num(&mut o, JS_ATOM_export);
    num(&mut o, JS_ATOM_extends);
    num(&mut o, JS_ATOM_import);
    num(&mut o, JS_ATOM_super);
    num(&mut o, JS_ATOM_implements);
    num(&mut o, JS_ATOM_interface);
    num(&mut o, JS_ATOM_let);
    num(&mut o, JS_ATOM_package);
    num(&mut o, JS_ATOM_private);
    num(&mut o, JS_ATOM_protected);
    num(&mut o, JS_ATOM_public);
    num(&mut o, JS_ATOM_static);
    num(&mut o, JS_ATOM_yield);
    num(&mut o, JS_ATOM_await);
    num(&mut o, JS_ATOM_empty_string);
    num(&mut o, JS_ATOM_keys);
    num(&mut o, JS_ATOM_size);
    num(&mut o, JS_ATOM_length);
    num(&mut o, JS_ATOM_fileName);
    num(&mut o, JS_ATOM_lineNumber);
    num(&mut o, JS_ATOM_columnNumber);
    num(&mut o, JS_ATOM_message);
    num(&mut o, JS_ATOM_cause);
    num(&mut o, JS_ATOM_errors);
    num(&mut o, JS_ATOM_stack);
    num(&mut o, JS_ATOM_name);
    num(&mut o, JS_ATOM_toString);
    num(&mut o, JS_ATOM_toLocaleString);
    num(&mut o, JS_ATOM_valueOf);
    num(&mut o, JS_ATOM_eval);
    num(&mut o, JS_ATOM_prototype);
    num(&mut o, JS_ATOM_constructor);
    num(&mut o, JS_ATOM_configurable);
    num(&mut o, JS_ATOM_writable);
    num(&mut o, JS_ATOM_enumerable);
    num(&mut o, JS_ATOM_value);
    num(&mut o, JS_ATOM_get);
    num(&mut o, JS_ATOM_set);
    num(&mut o, JS_ATOM_of);
    num(&mut o, JS_ATOM___proto__);
    num(&mut o, JS_ATOM_undefined);
    num(&mut o, JS_ATOM_number);
    num(&mut o, JS_ATOM_boolean);
    num(&mut o, JS_ATOM_string);
    num(&mut o, JS_ATOM_object);
    num(&mut o, JS_ATOM_symbol);
    num(&mut o, JS_ATOM_integer);
    num(&mut o, JS_ATOM_unknown);
    num(&mut o, JS_ATOM_arguments);
    num(&mut o, JS_ATOM_callee);
    num(&mut o, JS_ATOM_caller);
    num(&mut o, JS_ATOM__eval_);
    num(&mut o, JS_ATOM__ret_);
    num(&mut o, JS_ATOM__var_);
    num(&mut o, JS_ATOM__arg_var_);
    num(&mut o, JS_ATOM__with_);
    num(&mut o, JS_ATOM_lastIndex);
    num(&mut o, JS_ATOM_target);
    num(&mut o, JS_ATOM_index);
    num(&mut o, JS_ATOM_input);
    num(&mut o, JS_ATOM_defineProperties);
    num(&mut o, JS_ATOM_apply);
    num(&mut o, JS_ATOM_join);
    num(&mut o, JS_ATOM_concat);
    num(&mut o, JS_ATOM_split);
    num(&mut o, JS_ATOM_construct);
    num(&mut o, JS_ATOM_getPrototypeOf);
    num(&mut o, JS_ATOM_setPrototypeOf);
    num(&mut o, JS_ATOM_isExtensible);
    num(&mut o, JS_ATOM_preventExtensions);
    num(&mut o, JS_ATOM_has);
    num(&mut o, JS_ATOM_deleteProperty);
    num(&mut o, JS_ATOM_defineProperty);
    num(&mut o, JS_ATOM_getOwnPropertyDescriptor);
    num(&mut o, JS_ATOM_ownKeys);
    num(&mut o, JS_ATOM_add);
    num(&mut o, JS_ATOM_done);
    num(&mut o, JS_ATOM_next);
    num(&mut o, JS_ATOM_values);
    num(&mut o, JS_ATOM_source);
    num(&mut o, JS_ATOM_flags);
    num(&mut o, JS_ATOM_global);
    num(&mut o, JS_ATOM_unicode);
    num(&mut o, JS_ATOM_raw);
    num(&mut o, JS_ATOM_rawJSON);
    num(&mut o, JS_ATOM_new_target);
    num(&mut o, JS_ATOM_this_active_func);
    num(&mut o, JS_ATOM_home_object);
    num(&mut o, JS_ATOM_computed_field);
    num(&mut o, JS_ATOM_static_computed_field);
    num(&mut o, JS_ATOM_class_fields_init);
    num(&mut o, JS_ATOM_brand);
    num(&mut o, JS_ATOM_hash_constructor);
    num(&mut o, JS_ATOM_as);
    num(&mut o, JS_ATOM_from);
    num(&mut o, JS_ATOM_meta);
    num(&mut o, JS_ATOM__default_);
    num(&mut o, JS_ATOM__star_);
    num(&mut o, JS_ATOM_Module);
    num(&mut o, JS_ATOM_then);
    num(&mut o, JS_ATOM_resolve);
    num(&mut o, JS_ATOM_reject);
    num(&mut o, JS_ATOM_promise);
    num(&mut o, JS_ATOM_proxy);
    num(&mut o, JS_ATOM_revoke);
    num(&mut o, JS_ATOM_async);
    num(&mut o, JS_ATOM_exec);
    num(&mut o, JS_ATOM_groups);
    num(&mut o, JS_ATOM_indices);
    num(&mut o, JS_ATOM_status);
    num(&mut o, JS_ATOM_reason);
    num(&mut o, JS_ATOM_globalThis);
    num(&mut o, JS_ATOM_bigint);
    num(&mut o, JS_ATOM_minus_zero);
    num(&mut o, JS_ATOM_Infinity);
    num(&mut o, JS_ATOM_minus_Infinity);
    num(&mut o, JS_ATOM_NaN);
    num(&mut o, JS_ATOM_hasIndices);
    num(&mut o, JS_ATOM_ignoreCase);
    num(&mut o, JS_ATOM_multiline);
    num(&mut o, JS_ATOM_dotAll);
    num(&mut o, JS_ATOM_sticky);
    num(&mut o, JS_ATOM_unicodeSets);
    num(&mut o, JS_ATOM_not_equal);
    num(&mut o, JS_ATOM_timed_out);
    num(&mut o, JS_ATOM_ok);
    num(&mut o, JS_ATOM_toISOString);
    num(&mut o, JS_ATOM_alphabet);
    num(&mut o, JS_ATOM_lastChunkHandling);
    num(&mut o, JS_ATOM_omitPadding);
    num(&mut o, JS_ATOM_toJSON);
    num(&mut o, JS_ATOM_maxByteLength);
    num(&mut o, JS_ATOM_Object);
    num(&mut o, JS_ATOM_Array);
    num(&mut o, JS_ATOM_Error);
    num(&mut o, JS_ATOM_Number);
    num(&mut o, JS_ATOM_String);
    num(&mut o, JS_ATOM_Boolean);
    num(&mut o, JS_ATOM_Symbol);
    num(&mut o, JS_ATOM_Arguments);
    num(&mut o, JS_ATOM_Math);
    num(&mut o, JS_ATOM_JSON);
    num(&mut o, JS_ATOM_Date);
    num(&mut o, JS_ATOM_Function);
    num(&mut o, JS_ATOM_GeneratorFunction);
    num(&mut o, JS_ATOM_ForInIterator);
    num(&mut o, JS_ATOM_RegExp);
    num(&mut o, JS_ATOM_ArrayBuffer);
    num(&mut o, JS_ATOM_SharedArrayBuffer);
    num(&mut o, JS_ATOM_Uint8ClampedArray);
    num(&mut o, JS_ATOM_Int8Array);
    num(&mut o, JS_ATOM_Uint8Array);
    num(&mut o, JS_ATOM_Int16Array);
    num(&mut o, JS_ATOM_Uint16Array);
    num(&mut o, JS_ATOM_Int32Array);
    num(&mut o, JS_ATOM_Uint32Array);
    num(&mut o, JS_ATOM_BigInt64Array);
    num(&mut o, JS_ATOM_BigUint64Array);
    num(&mut o, JS_ATOM_Float16Array);
    num(&mut o, JS_ATOM_Float32Array);
    num(&mut o, JS_ATOM_Float64Array);
    num(&mut o, JS_ATOM_DataView);
    num(&mut o, JS_ATOM_BigInt);
    num(&mut o, JS_ATOM_WeakRef);
    num(&mut o, JS_ATOM_FinalizationRegistry);
    num(&mut o, JS_ATOM_Map);
    num(&mut o, JS_ATOM_Set);
    num(&mut o, JS_ATOM_WeakMap);
    num(&mut o, JS_ATOM_WeakSet);
    num(&mut o, JS_ATOM_Iterator);
    num(&mut o, JS_ATOM_IteratorHelper);
    num(&mut o, JS_ATOM_IteratorConcat);
    num(&mut o, JS_ATOM_IteratorWrap);
    num(&mut o, JS_ATOM_Map_Iterator);
    num(&mut o, JS_ATOM_Set_Iterator);
    num(&mut o, JS_ATOM_Array_Iterator);
    num(&mut o, JS_ATOM_String_Iterator);
    num(&mut o, JS_ATOM_RegExp_String_Iterator);
    num(&mut o, JS_ATOM_Generator);
    num(&mut o, JS_ATOM_Proxy);
    num(&mut o, JS_ATOM_Promise);
    num(&mut o, JS_ATOM_PromiseResolveFunction);
    num(&mut o, JS_ATOM_PromiseRejectFunction);
    num(&mut o, JS_ATOM_AsyncFunction);
    num(&mut o, JS_ATOM_AsyncFunctionResolve);
    num(&mut o, JS_ATOM_AsyncFunctionReject);
    num(&mut o, JS_ATOM_AsyncGeneratorFunction);
    num(&mut o, JS_ATOM_AsyncGenerator);
    num(&mut o, JS_ATOM_EvalError);
    num(&mut o, JS_ATOM_RangeError);
    num(&mut o, JS_ATOM_ReferenceError);
    num(&mut o, JS_ATOM_SyntaxError);
    num(&mut o, JS_ATOM_TypeError);
    num(&mut o, JS_ATOM_URIError);
    num(&mut o, JS_ATOM_InternalError);
    num(&mut o, JS_ATOM_AggregateError);
    num(&mut o, JS_ATOM_Private_brand);
    num(&mut o, JS_ATOM_Symbol_toPrimitive);
    num(&mut o, JS_ATOM_Symbol_iterator);
    num(&mut o, JS_ATOM_Symbol_match);
    num(&mut o, JS_ATOM_Symbol_matchAll);
    num(&mut o, JS_ATOM_Symbol_replace);
    num(&mut o, JS_ATOM_Symbol_search);
    num(&mut o, JS_ATOM_Symbol_split);
    num(&mut o, JS_ATOM_Symbol_toStringTag);
    num(&mut o, JS_ATOM_Symbol_isConcatSpreadable);
    num(&mut o, JS_ATOM_Symbol_hasInstance);
    num(&mut o, JS_ATOM_Symbol_species);
    num(&mut o, JS_ATOM_Symbol_unscopables);
    num(&mut o, JS_ATOM_Symbol_asyncIterator);
    num(&mut o, OP_FMT_none as u32);
    num(&mut o, OP_FMT_none_int as u32);
    num(&mut o, OP_FMT_none_loc as u32);
    num(&mut o, OP_FMT_none_arg as u32);
    num(&mut o, OP_FMT_none_var_ref as u32);
    num(&mut o, OP_FMT_u8 as u32);
    num(&mut o, OP_FMT_i8 as u32);
    num(&mut o, OP_FMT_loc8 as u32);
    num(&mut o, OP_FMT_const8 as u32);
    num(&mut o, OP_FMT_label8 as u32);
    num(&mut o, OP_FMT_u16 as u32);
    num(&mut o, OP_FMT_i16 as u32);
    num(&mut o, OP_FMT_label16 as u32);
    num(&mut o, OP_FMT_npop as u32);
    num(&mut o, OP_FMT_npopx as u32);
    num(&mut o, OP_FMT_npop_u16 as u32);
    num(&mut o, OP_FMT_loc as u32);
    num(&mut o, OP_FMT_arg as u32);
    num(&mut o, OP_FMT_var_ref as u32);
    num(&mut o, OP_FMT_u32 as u32);
    num(&mut o, OP_FMT_i32 as u32);
    num(&mut o, OP_FMT_const as u32);
    num(&mut o, OP_FMT_label as u32);
    num(&mut o, OP_FMT_atom as u32);
    num(&mut o, OP_FMT_atom_u8 as u32);
    num(&mut o, OP_FMT_atom_u16 as u32);
    num(&mut o, OP_FMT_atom_label_u8 as u32);
    num(&mut o, OP_FMT_atom_label_u16 as u32);
    num(&mut o, OP_FMT_label_u16 as u32);
    num(&mut o, OP_COUNT as u32);
    num(&mut o, OP_TEMP_START as u32);
    num(&mut o, OP_TEMP_END as u32);
    num(&mut o, OP_invalid as u32);
    num(&mut o, OP_push_i32 as u32);
    num(&mut o, OP_push_const as u32);
    num(&mut o, OP_fclosure as u32);
    num(&mut o, OP_push_atom_value as u32);
    num(&mut o, OP_private_symbol as u32);
    num(&mut o, OP_undefined as u32);
    num(&mut o, OP_null as u32);
    num(&mut o, OP_push_this as u32);
    num(&mut o, OP_push_false as u32);
    num(&mut o, OP_push_true as u32);
    num(&mut o, OP_object as u32);
    num(&mut o, OP_special_object as u32);
    num(&mut o, OP_rest as u32);
    num(&mut o, OP_drop as u32);
    num(&mut o, OP_nip as u32);
    num(&mut o, OP_nip1 as u32);
    num(&mut o, OP_dup as u32);
    num(&mut o, OP_dup1 as u32);
    num(&mut o, OP_dup2 as u32);
    num(&mut o, OP_dup3 as u32);
    num(&mut o, OP_insert2 as u32);
    num(&mut o, OP_insert3 as u32);
    num(&mut o, OP_insert4 as u32);
    num(&mut o, OP_perm3 as u32);
    num(&mut o, OP_perm4 as u32);
    num(&mut o, OP_perm5 as u32);
    num(&mut o, OP_swap as u32);
    num(&mut o, OP_swap2 as u32);
    num(&mut o, OP_rot3l as u32);
    num(&mut o, OP_rot3r as u32);
    num(&mut o, OP_rot4l as u32);
    num(&mut o, OP_rot5l as u32);
    num(&mut o, OP_call_constructor as u32);
    num(&mut o, OP_call as u32);
    num(&mut o, OP_tail_call as u32);
    num(&mut o, OP_call_method as u32);
    num(&mut o, OP_tail_call_method as u32);
    num(&mut o, OP_array_from as u32);
    num(&mut o, OP_apply as u32);
    num(&mut o, OP_return as u32);
    num(&mut o, OP_return_undef as u32);
    num(&mut o, OP_check_ctor_return as u32);
    num(&mut o, OP_check_ctor as u32);
    num(&mut o, OP_init_ctor as u32);
    num(&mut o, OP_check_brand as u32);
    num(&mut o, OP_add_brand as u32);
    num(&mut o, OP_return_async as u32);
    num(&mut o, OP_throw as u32);
    num(&mut o, OP_throw_error as u32);
    num(&mut o, OP_eval as u32);
    num(&mut o, OP_apply_eval as u32);
    num(&mut o, OP_regexp as u32);
    num(&mut o, OP_get_super as u32);
    num(&mut o, OP_import as u32);
    num(&mut o, OP_get_var_undef as u32);
    num(&mut o, OP_get_var as u32);
    num(&mut o, OP_put_var as u32);
    num(&mut o, OP_put_var_init as u32);
    num(&mut o, OP_get_ref_value as u32);
    num(&mut o, OP_put_ref_value as u32);
    num(&mut o, OP_get_field as u32);
    num(&mut o, OP_get_field2 as u32);
    num(&mut o, OP_put_field as u32);
    num(&mut o, OP_get_private_field as u32);
    num(&mut o, OP_put_private_field as u32);
    num(&mut o, OP_define_private_field as u32);
    num(&mut o, OP_get_array_el as u32);
    num(&mut o, OP_get_array_el2 as u32);
    num(&mut o, OP_get_array_el3 as u32);
    num(&mut o, OP_put_array_el as u32);
    num(&mut o, OP_get_super_value as u32);
    num(&mut o, OP_put_super_value as u32);
    num(&mut o, OP_define_field as u32);
    num(&mut o, OP_set_name as u32);
    num(&mut o, OP_set_name_computed as u32);
    num(&mut o, OP_set_proto as u32);
    num(&mut o, OP_set_home_object as u32);
    num(&mut o, OP_define_array_el as u32);
    num(&mut o, OP_append as u32);
    num(&mut o, OP_copy_data_properties as u32);
    num(&mut o, OP_define_method as u32);
    num(&mut o, OP_define_method_computed as u32);
    num(&mut o, OP_define_class as u32);
    num(&mut o, OP_define_class_computed as u32);
    num(&mut o, OP_get_loc as u32);
    num(&mut o, OP_put_loc as u32);
    num(&mut o, OP_set_loc as u32);
    num(&mut o, OP_get_arg as u32);
    num(&mut o, OP_put_arg as u32);
    num(&mut o, OP_set_arg as u32);
    num(&mut o, OP_get_var_ref as u32);
    num(&mut o, OP_put_var_ref as u32);
    num(&mut o, OP_set_var_ref as u32);
    num(&mut o, OP_set_loc_uninitialized as u32);
    num(&mut o, OP_get_loc_check as u32);
    num(&mut o, OP_put_loc_check as u32);
    num(&mut o, OP_set_loc_check as u32);
    num(&mut o, OP_put_loc_check_init as u32);
    num(&mut o, OP_get_loc_checkthis as u32);
    num(&mut o, OP_get_var_ref_check as u32);
    num(&mut o, OP_put_var_ref_check as u32);
    num(&mut o, OP_put_var_ref_check_init as u32);
    num(&mut o, OP_close_loc as u32);
    num(&mut o, OP_if_false as u32);
    num(&mut o, OP_if_true as u32);
    num(&mut o, OP_goto as u32);
    num(&mut o, OP_catch as u32);
    num(&mut o, OP_gosub as u32);
    num(&mut o, OP_ret as u32);
    num(&mut o, OP_nip_catch as u32);
    num(&mut o, OP_to_object as u32);
    num(&mut o, OP_to_propkey as u32);
    num(&mut o, OP_with_get_var as u32);
    num(&mut o, OP_with_put_var as u32);
    num(&mut o, OP_with_delete_var as u32);
    num(&mut o, OP_with_make_ref as u32);
    num(&mut o, OP_with_get_ref as u32);
    num(&mut o, OP_make_loc_ref as u32);
    num(&mut o, OP_make_arg_ref as u32);
    num(&mut o, OP_make_var_ref_ref as u32);
    num(&mut o, OP_make_var_ref as u32);
    num(&mut o, OP_for_in_start as u32);
    num(&mut o, OP_for_of_start as u32);
    num(&mut o, OP_for_await_of_start as u32);
    num(&mut o, OP_for_in_next as u32);
    num(&mut o, OP_for_of_next as u32);
    num(&mut o, OP_for_await_of_next as u32);
    num(&mut o, OP_iterator_check_object as u32);
    num(&mut o, OP_iterator_get_value_done as u32);
    num(&mut o, OP_iterator_close as u32);
    num(&mut o, OP_iterator_next as u32);
    num(&mut o, OP_iterator_call as u32);
    num(&mut o, OP_initial_yield as u32);
    num(&mut o, OP_yield as u32);
    num(&mut o, OP_yield_star as u32);
    num(&mut o, OP_async_yield_star as u32);
    num(&mut o, OP_await as u32);
    num(&mut o, OP_neg as u32);
    num(&mut o, OP_plus as u32);
    num(&mut o, OP_dec as u32);
    num(&mut o, OP_inc as u32);
    num(&mut o, OP_post_dec as u32);
    num(&mut o, OP_post_inc as u32);
    num(&mut o, OP_dec_loc as u32);
    num(&mut o, OP_inc_loc as u32);
    num(&mut o, OP_add_loc as u32);
    num(&mut o, OP_not as u32);
    num(&mut o, OP_lnot as u32);
    num(&mut o, OP_typeof as u32);
    num(&mut o, OP_delete as u32);
    num(&mut o, OP_delete_var as u32);
    num(&mut o, OP_mul as u32);
    num(&mut o, OP_div as u32);
    num(&mut o, OP_mod as u32);
    num(&mut o, OP_add as u32);
    num(&mut o, OP_sub as u32);
    num(&mut o, OP_pow as u32);
    num(&mut o, OP_shl as u32);
    num(&mut o, OP_sar as u32);
    num(&mut o, OP_shr as u32);
    num(&mut o, OP_lt as u32);
    num(&mut o, OP_lte as u32);
    num(&mut o, OP_gt as u32);
    num(&mut o, OP_gte as u32);
    num(&mut o, OP_instanceof as u32);
    num(&mut o, OP_in as u32);
    num(&mut o, OP_eq as u32);
    num(&mut o, OP_neq as u32);
    num(&mut o, OP_strict_eq as u32);
    num(&mut o, OP_strict_neq as u32);
    num(&mut o, OP_and as u32);
    num(&mut o, OP_xor as u32);
    num(&mut o, OP_or as u32);
    num(&mut o, OP_is_undefined_or_null as u32);
    num(&mut o, OP_private_in as u32);
    num(&mut o, OP_push_bigint_i32 as u32);
    num(&mut o, OP_nop as u32);
    num(&mut o, OP_enter_scope as u32);
    num(&mut o, OP_leave_scope as u32);
    num(&mut o, OP_label as u32);
    num(&mut o, OP_scope_get_var_undef as u32);
    num(&mut o, OP_scope_get_var as u32);
    num(&mut o, OP_scope_put_var as u32);
    num(&mut o, OP_scope_delete_var as u32);
    num(&mut o, OP_scope_make_ref as u32);
    num(&mut o, OP_scope_get_ref as u32);
    num(&mut o, OP_scope_put_var_init as u32);
    num(&mut o, OP_scope_get_var_checkthis as u32);
    num(&mut o, OP_scope_get_private_field as u32);
    num(&mut o, OP_scope_get_private_field2 as u32);
    num(&mut o, OP_scope_put_private_field as u32);
    num(&mut o, OP_scope_in_private_field as u32);
    num(&mut o, OP_get_field_opt_chain as u32);
    num(&mut o, OP_get_array_el_opt_chain as u32);
    num(&mut o, OP_set_class_name as u32);
    num(&mut o, OP_line_num as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_minus1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_4 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_5 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_6 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_7 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_i8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_i16 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_const8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_fclosure8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_push_empty_string as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_loc8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_loc8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_loc8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_loc0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_loc1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_loc2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_loc3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_loc0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_loc1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_loc2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_loc3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_loc0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_loc1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_loc2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_loc3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_arg0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_arg1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_arg2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_arg3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_arg0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_arg1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_arg2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_arg3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_arg0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_arg1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_arg2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_arg3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_var_ref0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_var_ref1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_var_ref2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_var_ref3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_var_ref0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_var_ref1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_var_ref2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_put_var_ref3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_var_ref0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_var_ref1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_var_ref2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_set_var_ref3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_get_length as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_if_false8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_if_true8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_goto8 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_goto16 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_call0 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_call1 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_call2 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_call3 as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_is_undefined as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_is_null as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_typeof_is_undefined as u32);
    #[cfg(feature = "short-opcodes")]
    num(&mut o, OP_typeof_is_function as u32);
    for x in opcode_info {
        o.extend([x.size, x.n_pop, x.n_push, x.fmt]);
    }
    for i in 0..OP_COUNT {
        let x = short_opcode_info(i);
        o.extend([x.size, x.n_pop, x.n_push, x.fmt]);
    }
    num(&mut o, REOP_COUNT as u32);
    num(&mut o, REOP_invalid as u32);
    o.push(reopcode_info[REOP_invalid as usize]);
    num(&mut o, REOP_char as u32);
    o.push(reopcode_info[REOP_char as usize]);
    num(&mut o, REOP_char_i as u32);
    o.push(reopcode_info[REOP_char_i as usize]);
    num(&mut o, REOP_char32 as u32);
    o.push(reopcode_info[REOP_char32 as usize]);
    num(&mut o, REOP_char32_i as u32);
    o.push(reopcode_info[REOP_char32_i as usize]);
    num(&mut o, REOP_dot as u32);
    o.push(reopcode_info[REOP_dot as usize]);
    num(&mut o, REOP_any as u32);
    o.push(reopcode_info[REOP_any as usize]);
    num(&mut o, REOP_space as u32);
    o.push(reopcode_info[REOP_space as usize]);
    num(&mut o, REOP_not_space as u32);
    o.push(reopcode_info[REOP_not_space as usize]);
    num(&mut o, REOP_line_start as u32);
    o.push(reopcode_info[REOP_line_start as usize]);
    num(&mut o, REOP_line_start_m as u32);
    o.push(reopcode_info[REOP_line_start_m as usize]);
    num(&mut o, REOP_line_end as u32);
    o.push(reopcode_info[REOP_line_end as usize]);
    num(&mut o, REOP_line_end_m as u32);
    o.push(reopcode_info[REOP_line_end_m as usize]);
    num(&mut o, REOP_goto as u32);
    o.push(reopcode_info[REOP_goto as usize]);
    num(&mut o, REOP_split_goto_first as u32);
    o.push(reopcode_info[REOP_split_goto_first as usize]);
    num(&mut o, REOP_split_next_first as u32);
    o.push(reopcode_info[REOP_split_next_first as usize]);
    num(&mut o, REOP_match as u32);
    o.push(reopcode_info[REOP_match as usize]);
    num(&mut o, REOP_lookahead_match as u32);
    o.push(reopcode_info[REOP_lookahead_match as usize]);
    num(&mut o, REOP_negative_lookahead_match as u32);
    o.push(reopcode_info[REOP_negative_lookahead_match as usize]);
    num(&mut o, REOP_save_start as u32);
    o.push(reopcode_info[REOP_save_start as usize]);
    num(&mut o, REOP_save_end as u32);
    o.push(reopcode_info[REOP_save_end as usize]);
    num(&mut o, REOP_save_reset as u32);
    o.push(reopcode_info[REOP_save_reset as usize]);
    num(&mut o, REOP_loop as u32);
    o.push(reopcode_info[REOP_loop as usize]);
    num(&mut o, REOP_loop_split_goto_first as u32);
    o.push(reopcode_info[REOP_loop_split_goto_first as usize]);
    num(&mut o, REOP_loop_split_next_first as u32);
    o.push(reopcode_info[REOP_loop_split_next_first as usize]);
    num(&mut o, REOP_loop_check_adv_split_goto_first as u32);
    o.push(reopcode_info[REOP_loop_check_adv_split_goto_first as usize]);
    num(&mut o, REOP_loop_check_adv_split_next_first as u32);
    o.push(reopcode_info[REOP_loop_check_adv_split_next_first as usize]);
    num(&mut o, REOP_set_i32 as u32);
    o.push(reopcode_info[REOP_set_i32 as usize]);
    num(&mut o, REOP_word_boundary as u32);
    o.push(reopcode_info[REOP_word_boundary as usize]);
    num(&mut o, REOP_word_boundary_i as u32);
    o.push(reopcode_info[REOP_word_boundary_i as usize]);
    num(&mut o, REOP_not_word_boundary as u32);
    o.push(reopcode_info[REOP_not_word_boundary as usize]);
    num(&mut o, REOP_not_word_boundary_i as u32);
    o.push(reopcode_info[REOP_not_word_boundary_i as usize]);
    num(&mut o, REOP_back_reference as u32);
    o.push(reopcode_info[REOP_back_reference as usize]);
    num(&mut o, REOP_back_reference_i as u32);
    o.push(reopcode_info[REOP_back_reference_i as usize]);
    num(&mut o, REOP_backward_back_reference as u32);
    o.push(reopcode_info[REOP_backward_back_reference as usize]);
    num(&mut o, REOP_backward_back_reference_i as u32);
    o.push(reopcode_info[REOP_backward_back_reference_i as usize]);
    num(&mut o, REOP_range as u32);
    o.push(reopcode_info[REOP_range as usize]);
    num(&mut o, REOP_range_i as u32);
    o.push(reopcode_info[REOP_range_i as usize]);
    num(&mut o, REOP_range32 as u32);
    o.push(reopcode_info[REOP_range32 as usize]);
    num(&mut o, REOP_range32_i as u32);
    o.push(reopcode_info[REOP_range32_i as usize]);
    num(&mut o, REOP_lookahead as u32);
    o.push(reopcode_info[REOP_lookahead as usize]);
    num(&mut o, REOP_negative_lookahead as u32);
    o.push(reopcode_info[REOP_negative_lookahead as usize]);
    num(&mut o, REOP_set_char_pos as u32);
    o.push(reopcode_info[REOP_set_char_pos as usize]);
    num(&mut o, REOP_check_advance as u32);
    o.push(reopcode_info[REOP_check_advance as usize]);
    num(&mut o, REOP_prev as u32);
    o.push(reopcode_info[REOP_prev as usize]);
    #[repr(C)]
    struct Node {
        value: i32,
        link: list_head,
    }
    unsafe {
        let mut head = Box::new(list_head::default());
        quickjs::LIST_HEAD_INIT!(&mut *head);
        let mut nodes: Vec<_> = (0..4)
            .map(|value| {
                Box::new(Node {
                    value,
                    link: list_head::default(),
                })
            })
            .collect();
        num(&mut o, list_empty(&mut *head) as u32);
        for node in &mut nodes {
            list_add_tail(&mut node.link, &mut *head);
        }
        list_del(&mut nodes[3].link);
        list_add(&mut nodes[3].link, &mut *head);
        quickjs::list_for_each!(el, &mut *head, {
            num(&mut o, (*quickjs::list_entry!(el, Node, link)).value as u32);
        });
        quickjs::list_for_each_prev!(el, &mut *head, {
            num(&mut o, (*quickjs::list_entry!(el, Node, link)).value as u32);
        });
        quickjs::list_for_each_safe!(el, next, &mut *head, {
            let _ = next;
            num(&mut o, (*quickjs::list_entry!(el, Node, link)).value as u32);
            list_del(el);
            num(
                &mut o,
                ((*el).next.is_null() && (*el).prev.is_null()) as u32,
            );
            continue;
        });
        num(&mut o, list_empty(&mut *head) as u32);
        for node in &mut nodes {
            list_add_tail(&mut node.link, &mut *head);
        }
        quickjs::list_for_each_prev_safe!(el, next, &mut *head, {
            let _ = next;
            num(&mut o, (*quickjs::list_entry!(el, Node, link)).value as u32);
            list_del(el);
        });
        num(&mut o, list_empty(&mut *head) as u32);
    }
    o
}
