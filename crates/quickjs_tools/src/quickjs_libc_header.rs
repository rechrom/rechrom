//! Public host API corresponding to quickjs-libc.h:35..59 (Bellard/Gordon MIT).
//! Rust function declarations live with their implementations; this facade
//! retains the original header's API grouping without a second implementation.
#[cfg(any(unix, windows))]
pub use super::quickjs_libc::{
    js_init_module_os, js_init_module_std, js_load_file, js_module_check_attributes,
    js_module_loader, js_module_set_import_meta, js_module_test_json, js_std_add_helpers,
    js_std_await, js_std_dump_error, js_std_eval_binary, js_std_eval_binary_json_module,
    js_std_free_handlers, js_std_init_handlers, js_std_loop, js_std_promise_rejection_tracker,
    js_std_set_worker_new_context_func,
};
