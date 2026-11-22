use super::*;
/// Capture a live script callsite without creating/throwing an Error or
/// consuming exception state. Embedding WebIDL thunks pass their own known
/// script-frame count; the engine contains no DOM/bootstrap-name policy.
pub unsafe fn JS_CaptureCurrentSourceLocation(
    ctx: *mut JSContext,
    mut skip_script_frames: usize,
) -> Option<JSExceptionLocation> {
    let mut frame = (*(*ctx).rt).current_stack_frame;
    while !frame.is_null() {
        let bytecode = JS_GetFunctionBytecode((*frame).cur_func);
        if !bytecode.is_null() {
            if skip_script_frames == 0 {
                return js_host_bytecode_location(ctx, bytecode, (*frame).cur_pc, true)
                    .map(|location| location.resolve());
            }
            skip_script_frames -= 1;
        }
        frame = (*frame).prev_frame;
    }
    None
}
