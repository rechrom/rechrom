// quickjs.c:55947-55951. Official evaluator injection. MIT.
pub unsafe fn JS_AddIntrinsicEval(ctx: *mut JSContext) -> i32 {
    (*ctx).eval_internal = Some(__JS_EvalInternal);
    0
}
