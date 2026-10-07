// quickjs.c:1969..2033. Standard class callback table. MIT.
static js_std_class_def: [JSClassShortDef; 50] = [
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Object,
        finalizer: None,
        gc_mark: None,
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Array,
        finalizer: Some(js_array_finalizer),
        gc_mark: Some(js_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Error,
        finalizer: None,
        gc_mark: None,
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Number,
        finalizer: Some(js_object_data_finalizer),
        gc_mark: Some(js_object_data_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_String,
        finalizer: Some(js_object_data_finalizer),
        gc_mark: Some(js_object_data_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Boolean,
        finalizer: Some(js_object_data_finalizer),
        gc_mark: Some(js_object_data_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Symbol,
        finalizer: Some(js_object_data_finalizer),
        gc_mark: Some(js_object_data_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Arguments,
        finalizer: Some(js_array_finalizer),
        gc_mark: Some(js_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Arguments,
        finalizer: Some(js_mapped_arguments_finalizer),
        gc_mark: Some(js_mapped_arguments_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Date,
        finalizer: Some(js_object_data_finalizer),
        gc_mark: Some(js_object_data_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Object,
        finalizer: None,
        gc_mark: None,
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Function,
        finalizer: Some(js_c_function_finalizer),
        gc_mark: Some(js_c_function_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Function,
        finalizer: Some(js_bytecode_function_finalizer),
        gc_mark: Some(js_bytecode_function_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Function,
        finalizer: Some(js_bound_function_finalizer),
        gc_mark: Some(js_bound_function_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Function,
        finalizer: Some(js_c_function_data_finalizer),
        gc_mark: Some(js_c_function_data_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_GeneratorFunction,
        finalizer: Some(js_bytecode_function_finalizer),
        gc_mark: Some(js_bytecode_function_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_ForInIterator,
        finalizer: Some(js_for_in_iterator_finalizer),
        gc_mark: Some(js_for_in_iterator_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_RegExp,
        finalizer: Some(js_regexp_finalizer),
        gc_mark: None,
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_ArrayBuffer,
        finalizer: Some(js_array_buffer_finalizer),
        gc_mark: None,
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_SharedArrayBuffer,
        finalizer: Some(js_array_buffer_finalizer),
        gc_mark: None,
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Uint8ClampedArray,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Int8Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Uint8Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Int16Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Uint16Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Int32Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Uint32Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_BigInt64Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_BigUint64Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Float16Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Float32Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Float64Array,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_DataView,
        finalizer: Some(js_typed_array_finalizer),
        gc_mark: Some(js_typed_array_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_BigInt,
        finalizer: Some(js_object_data_finalizer),
        gc_mark: Some(js_object_data_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Map,
        finalizer: Some(js_map_finalizer),
        gc_mark: Some(js_map_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Set,
        finalizer: Some(js_map_finalizer),
        gc_mark: Some(js_map_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_WeakMap,
        finalizer: Some(js_map_finalizer),
        gc_mark: Some(js_map_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_WeakSet,
        finalizer: Some(js_map_finalizer),
        gc_mark: Some(js_map_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Iterator,
        finalizer: None,
        gc_mark: None,
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_IteratorConcat,
        finalizer: Some(js_iterator_concat_finalizer),
        gc_mark: Some(js_iterator_concat_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_IteratorHelper,
        finalizer: Some(js_iterator_helper_finalizer),
        gc_mark: Some(js_iterator_helper_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_IteratorWrap,
        finalizer: Some(js_iterator_wrap_finalizer),
        gc_mark: Some(js_iterator_wrap_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Map_Iterator,
        finalizer: Some(js_map_iterator_finalizer),
        gc_mark: Some(js_map_iterator_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Set_Iterator,
        finalizer: Some(js_map_iterator_finalizer),
        gc_mark: Some(js_map_iterator_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Array_Iterator,
        finalizer: Some(js_array_iterator_finalizer),
        gc_mark: Some(js_array_iterator_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_String_Iterator,
        finalizer: Some(js_array_iterator_finalizer),
        gc_mark: Some(js_array_iterator_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_RegExp_String_Iterator,
        finalizer: Some(js_regexp_string_iterator_finalizer),
        gc_mark: Some(js_regexp_string_iterator_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Generator,
        finalizer: Some(js_generator_finalizer),
        gc_mark: Some(js_generator_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Object,
        finalizer: Some(js_global_object_finalizer),
        gc_mark: Some(js_global_object_mark),
    },
    JSClassShortDef {
        class_name: crate::quickjs_atom::JS_ATOM_Object,
        finalizer: None,
        gc_mark: None,
    },
];
