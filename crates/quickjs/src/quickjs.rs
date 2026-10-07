//! quickjs.c. Copyright 2017-2021 Fabrice Bellard and Charlie Gordon; MIT.
//! Original runtime, context, parser, compiler, VM and standard library functions
//! retain their C names and algorithms. The complete JS_NewContext bootstrap
//! and real JS_Eval execution are checked against the unchanged official C.
//! Browser host integration and remaining source coverage are still in progress.
#![allow(dead_code, unused_imports, unused_variables)] // Retain original C locals and optional source helpers.
use super::{list::*, quickjs_header::*};
use core::{
    ffi::{c_char, c_void},
    mem::{offset_of, size_of},
    ptr,
};
include!("quickjs_malloc.rs");
include!("quickjs_types.rs");
include!("quickjs_execution_types.rs");
include!("quickjs_atoms.rs");
include!("quickjs_mp.rs");
include!("quickjs_classes.rs");
include!("quickjs_shapes.rs");
include!("quickjs_objects.rs");
include!("quickjs_gc.rs");
include!("quickjs_gc_mark.rs");
include!("quickjs_weakrefs.rs");
include!("quickjs_conversions.rs");
include!("quickjs_class_lifecycle.rs");
include!("quickjs_intrinsic_iterator_lifecycle.rs");
include!("quickjs_runtime_init_classes.rs");
include!("quickjs_calls_stack.rs");
include!("quickjs_property_flags.rs");
include!("quickjs_debug.rs");
include!("quickjs_job_stack_profile.rs");
include!("quickjs_equality.rs");
include!("quickjs_parser_module_namespace.rs");
include!("quickjs_context_alloc.rs");
include!("quickjs_strings.rs");
include!("quickjs_string_conversions.rs");
include!("quickjs_string_ropes.rs");
include!("quickjs_atom_strings.rs");
include!("quickjs_header_runtime_wrappers.rs");
// Explicit browser embedding policy; ordinary public C APIs keep their
// original algorithms and runtime layout when no hooks are registered.
include!("quickjs_embedding_modules.rs");
include!("quickjs_embedding_exceptions.rs");
include!("quickjs_shape_construction.rs");
include!("quickjs_object_construction.rs");
include!("quickjs_properties.rs");
include!("quickjs_property_helpers.rs");
include!("quickjs_property_conversions.rs");
include!("quickjs_scalar_conversions.rs");
include!("quickjs_errors.rs");
include!("quickjs_prototypes.rs");
include!("quickjs_property_reads.rs");
include!("quickjs_bigint_construction.rs");
include!("quickjs_numeric_conversions.rs");
include!("quickjs_bigint_conversions.rs");
include!("quickjs_object_conversions.rs");
include!("quickjs_property_writes.rs");
include!("quickjs_bigint_float.rs");
include!("quickjs_global_properties.rs");
include!("quickjs_private_properties.rs");
include!("quickjs_property_names.rs");
include!("quickjs_string_object_length.rs");
include!("quickjs_object_operations.rs");
include!("quickjs_calls_native.rs");
include!("quickjs_calls_public.rs");
include!("quickjs_calls_closure.rs");
include!("quickjs_calls_class.rs");
include!("quickjs_calls_instanceof.rs");
include!("quickjs_vm_support.rs");
include!("quickjs_vm_operators.rs");
include!("quickjs_vm_bigint_operators.rs");
include!("quickjs_vm_generator_constants.rs");
include!("quickjs_vm_iterators.rs");
include!("quickjs_vm_async.rs");
include!("quickjs_vm_async_generators.rs");
include!("quickjs_vm_dispatch_short.rs");
include!("quickjs_vm_dispatch_full.rs");
include!("quickjs_runtime_init_allocator.rs");
include!("quickjs_runtime_init_core.rs");
include!("quickjs_runtime_init_jobs_gc.rs");
include!("quickjs_intrinsic_functions.rs");
include!("quickjs_intrinsic_exotics.rs");
include!("quickjs_intrinsic_function_builtins.rs");
include!("quickjs_intrinsic_errors.rs");
include!("quickjs_intrinsic_weakref.rs");
include!("quickjs_intrinsic_basic_objects.rs");
include!("quickjs_intrinsic_eval.rs");
include!("quickjs_intrinsic_objects.rs");
include!("quickjs_intrinsic_object_proto_table.rs");
include!("quickjs_intrinsic_object_table.rs");
include!("quickjs_intrinsic_array_types.rs");
include!("quickjs_intrinsic_array_support.rs");
include!("quickjs_intrinsic_array_bodies.rs");
include!("quickjs_intrinsic_array_tables.rs");
include!("quickjs_intrinsic_promise_classes.rs");
include!("quickjs_intrinsic_promise_bodies.rs");
include!("quickjs_intrinsic_promise_tables.rs");
include!("quickjs_intrinsic_proxy.rs");
include!("quickjs_intrinsic_proxy_tables.rs");
include!("quickjs_intrinsic_map.rs");
include!("quickjs_intrinsic_map_tables.rs");
include!("quickjs_intrinsic_number_bodies.rs");
include!("quickjs_intrinsic_number_tables.rs");
include!("quickjs_intrinsic_math_support.rs");
include!("quickjs_intrinsic_math_bodies.rs");
include!("quickjs_intrinsic_math_tables.rs");
include!("quickjs_intrinsic_date_support.rs");
include!("quickjs_intrinsic_date_timezone.rs");
include!("quickjs_intrinsic_date_bodies.rs");
include!("quickjs_intrinsic_bigint_bodies.rs");
include!("quickjs_intrinsic_date_bigint_tables.rs");
include!("quickjs_intrinsic_base_support.rs");
include!("quickjs_intrinsic_base_bodies.rs");
include!("quickjs_intrinsic_base_tables.rs");
include!("quickjs_intrinsic_reflect.rs");
include!("quickjs_intrinsic_reflect_tables.rs");
include!("quickjs_intrinsic_string_html.rs");
include!("quickjs_intrinsic_string_tables.rs");
include!("quickjs_intrinsic_string_bodies.rs");
include!("quickjs_intrinsic_symbols.rs");
include!("quickjs_intrinsic_symbol_tables.rs");
include!("quickjs_intrinsic_regexp_support.rs");
include!("quickjs_intrinsic_regexp.rs");
include!("quickjs_intrinsic_regexp_tables.rs");
include!("quickjs_serialization_types.rs");
include!("quickjs_serialization_generated_binary.rs");
include!("quickjs_intrinsic_json_types.rs");
include!("quickjs_intrinsic_json_generated.rs");
include!("quickjs_intrinsic_json_tables.rs");
include!("quickjs_intrinsic_buffers_types.rs");
include!("quickjs_intrinsic_buffers_constants.rs");
include!("quickjs_intrinsic_buffers_tables.rs");
include!("quickjs_intrinsic_buffers_sync.rs");
include!("quickjs_intrinsic_buffers_bodies.rs");
include!("quickjs_intrinsic_typed_arrays_bodies.rs");
include!("quickjs_intrinsic_dataview_bodies.rs");
include!("quickjs_intrinsic_atomics_bodies.rs");
include!("quickjs_parser_types.rs");
include!("quickjs_parser_lexer.rs");
include!("quickjs_compiler_scopes.rs");
include!("quickjs_parser_lookahead.rs");
include!("quickjs_compiler_functions.rs");
include!("quickjs_parser_json_lexer.rs");
include!("quickjs_compiler_lvalues.rs");
include!("quickjs_compiler_flow.rs");
include!("quickjs_parser_format.rs");
include!("quickjs_parser_c_adapters.rs");
include!("quickjs_compiler_patterns.rs");
include!("quickjs_parser_expressions.rs");
include!("quickjs_parser_generated_literals.rs");
include!("quickjs_parser_generated_destructuring.rs");
include!("quickjs_parser_generated_postfix.rs");
include!("quickjs_parser_generated_unary.rs");
include!("quickjs_parser_generated_assignment.rs");
include!("quickjs_parser_generated_statements.rs");
include!("quickjs_parser_generated_modules.rs");
include!("quickjs_compiler_generated_passes.rs");
include!("quickjs_parser_generated_functions.rs");
include!("quickjs_parser_generated_eval.rs");
include!("quickjs_value_print_support.rs");
include!("quickjs_value_print_dump_atoms.rs");
include!("quickjs_value_print_dump_shapes.rs");
include!("quickjs_value_print_dump_objects.rs");
include!("quickjs_value_print_dump_bigint.rs");
include!("quickjs_value_print_dump_malloc.rs");
include!("quickjs_value_print_dump_memory.rs");
include!("quickjs_value_print_generated.rs");
include!("quickjs_compiler_debug_debug.rs");
include!("quickjs_compiler_debug_module.rs");
include!("quickjs_parser_debug_dump_token.rs");
include!("quickjs_memory_usage.rs");
include!("quickjs_public_exports.rs");
include!("quickjs_dormant_intrinsics.rs");
include!("quickjs_dormant_objects.rs");
#[cfg(test)]
#[path = "../tests/quickjs_atoms_differential.rs"]
mod atom_tests;
#[cfg(test)]
#[path = "../tests/quickjs_malloc_differential.rs"]
mod malloc_tests;
#[cfg(test)]
#[path = "../tests/quickjs_mp_differential.rs"]
mod mp_tests;

#[cfg(test)]
#[path = "../tests/quickjs_full_runtime_differential.rs"]
mod full_runtime_tests;

#[cfg(test)]
#[path = "../tests/quickjs_force_gc_differential.rs"]
mod force_gc_tests;
