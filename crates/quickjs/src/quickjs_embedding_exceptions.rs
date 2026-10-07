// Optional exception diagnostics. C values, layouts, language behavior, and
// Error.stack remain untouched. Host-owned sidecars are disabled by default.
use std::{collections::BTreeMap as DiagnosticMap, sync::Arc as DiagnosticArc};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JSExceptionLocation {
    pub filename: String,
    pub line: usize,
    /// Zero-based UTF-16 column, as used by the browser host contract.
    pub column: usize,
    pub source_line: String,
}
struct DiagnosticSource {
    text: String,
    call_sites: Mutex<DiagnosticMap<usize, usize>>,
    positions: std::sync::OnceLock<DiagnosticSourceIndex>,
}
// The optional embedding diagnostics must not rescan a minified bundle for
// every caught exception. Sparse checkpoints bound a query to 255 Unicode
// scalars, while separator offsets avoid rescanning an entire source line.
// No JS-visible values, parser positions or exception semantics are changed.
#[derive(Clone, Copy)]
struct DiagnosticCheckpoint {
    offset: usize,
    raw_line: usize,
    raw_column: usize,
    line: usize,
    column: usize,
    line_start: usize,
    last_cr: bool,
}
impl DiagnosticCheckpoint {
    fn Advance(&mut self, offset: usize, ch: char) {
        if ch == '\n' {
            self.raw_line += 1;
            self.raw_column = 1;
        } else {
            self.raw_column += 1;
        }
        match ch {
            '\r' => {
                self.line += 1;
                self.column = 0;
                self.line_start = offset + 1;
                self.last_cr = true;
            }
            '\n' => {
                if !self.last_cr { self.line += 1; }
                self.column = 0;
                self.line_start = offset + 1;
                self.last_cr = false;
            }
            '\u{2028}' | '\u{2029}' => {
                self.line += 1;
                self.column = 0;
                self.line_start = offset + ch.len_utf8();
                self.last_cr = false;
            }
            _ => {
                self.column += ch.len_utf16();
                self.last_cr = false;
            }
        }
        self.offset = offset + ch.len_utf8();
    }
}
struct DiagnosticSourceIndex {
    checkpoints: Vec<DiagnosticCheckpoint>,
    separators: Vec<usize>,
}
impl DiagnosticSourceIndex {
    fn new(source: &str) -> Self {
        let mut point = DiagnosticCheckpoint { offset: 0, raw_line: 1, raw_column: 1,
            line: 1, column: 0, line_start: 0, last_cr: false };
        let mut checkpoints = vec![point];
        let mut separators = Vec::new();
        for (count, (offset, ch)) in source.char_indices().enumerate() {
            if count != 0 && count % 256 == 0 { checkpoints.push(point); }
            if matches!(ch, '\r' | '\n' | '\u{2028}' | '\u{2029}') {
                separators.push(offset);
            }
            point.Advance(offset, ch);
        }
        if checkpoints.last().unwrap().offset != source.len() { checkpoints.push(point); }
        Self { checkpoints, separators }
    }
    fn raw_source_offset(&self, source: &str, raw_line: i32, raw_column: i32) -> usize {
        let target = (raw_line.max(1) as usize, raw_column.max(1) as usize);
        let next = self.checkpoints.partition_point(|point| (point.raw_line, point.raw_column) <= target);
        let mut point = self.checkpoints[next.saturating_sub(1)];
        let start = point.offset;
        for (relative, ch) in source[start..].char_indices() {
            let offset = start + relative;
            if point.raw_line == target.0 && (point.raw_column >= target.1 || ch == '\n') {
                return offset;
            }
            point.Advance(offset, ch);
        }
        source.len()
    }
    fn position_at_offset(&self, filename: String, source: &str, index: usize) -> JSExceptionLocation {
        let next = self.checkpoints.partition_point(|point| point.offset <= index);
        let mut point = self.checkpoints[next.saturating_sub(1)];
        let start = point.offset;
        for (relative, ch) in source[start..index].char_indices() { point.Advance(start + relative, ch); }
        let separator = self.separators.partition_point(|&offset| offset < point.line_start);
        let end = self.separators.get(separator).copied().unwrap_or(source.len());
        JSExceptionLocation { filename, line: point.line, column: point.column,
            source_line: source[point.line_start..end].to_owned() }
    }
}
impl DiagnosticSource {
    fn raw_source_offset(&self, line: i32, column: i32) -> usize {
        self.positions.get_or_init(|| DiagnosticSourceIndex::new(&self.text))
            .raw_source_offset(&self.text, line, column)
    }
    fn position_at_offset(&self, filename: String, index: usize) -> JSExceptionLocation {
        self.positions.get_or_init(|| DiagnosticSourceIndex::new(&self.text))
            .position_at_offset(filename, &self.text, index)
    }
}
impl core::ops::Deref for DiagnosticSource {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}
#[derive(Default)]
struct ExceptionDiagnostics {
    captured: bool,
    consume_creation_on_stack_read: bool,
    pending: Option<JSExceptionLocation>,
    sources: DiagnosticMap<usize, DiagnosticArc<DiagnosticSource>>,
    compiling: Vec<DiagnosticArc<DiagnosticSource>>,
    creations: DiagnosticMap<usize, JSExceptionLocation>,
    suspended: DiagnosticMap<(usize, usize), Option<JSExceptionLocation>>,
}
static EXCEPTION_DIAGNOSTICS: Mutex<DiagnosticMap<usize, ExceptionDiagnostics>> =
    Mutex::new(DiagnosticMap::new());

/// Opt into independent diagnostics; false discards all sidecar state. This
/// neither edits Error properties nor changes the official exception value.
pub unsafe fn JS_EnableExceptionMetadata(rt: *mut JSRuntime, enabled: bool) {
    let mut states = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if enabled {
        states.entry(rt as usize).or_default();
    } else {
        states.remove(&(rt as usize));
    }
}
/// Choose whether reading stack consumes the separate creation-message
/// diagnostic. Default false keeps creation metadata; browser hosts may opt in
/// to the message policy of an engine that formats structured stacks lazily.
/// This never changes the returned JS stack property or its descriptor.
pub unsafe fn JS_SetExceptionMetadataStackReadPolicy(rt: *mut JSRuntime, consume: bool) {
    if let Some(state) = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&(rt as usize))
    {
        state.consume_creation_on_stack_read = consume;
    }
}
/// Read before JS_GetException, which transfers/clears the current exception.
pub unsafe fn JS_GetExceptionMetadata(ctx: *mut JSContext) -> Option<JSExceptionLocation> {
    EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&((*ctx).rt as usize))
        .and_then(|s| s.pending.clone())
}
/// Creation metadata is separate from an actual throw, for host contracts that
/// create a diagnostic message from a Promise's rejection reason.
pub unsafe fn JS_GetErrorCreationMetadata(
    ctx: *mut JSContext,
    value: JSValueConst,
) -> Option<JSExceptionLocation> {
    if JS_VALUE_GET_TAG(value) != JS_TAG_OBJECT {
        return None;
    }
    EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&((*ctx).rt as usize))
        .and_then(|s| {
            s.creations
                .get(&(JS_VALUE_GET_PTR(value) as usize))
                .cloned()
        })
}
fn js_host_diagnostics_enabled(rt: *mut JSRuntime) -> bool {
    EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .contains_key(&(rt as usize))
}
unsafe fn js_host_exception_begin(rt: *mut JSRuntime) {
    if let Some(state) = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&(rt as usize))
    {
        state.pending = None;
        state.captured = false;
    }
}
unsafe fn js_host_exception_taken(rt: *mut JSRuntime) {
    js_host_exception_begin(rt);
}
unsafe fn js_host_clear_exception_diagnostics(rt: *mut JSRuntime) {
    JS_EnableExceptionMetadata(rt, false);
}
unsafe fn js_host_free_bytecode_diagnostics(rt: *mut JSRuntime, bytecode: *mut JSFunctionBytecode) {
    if let Some(state) = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&(rt as usize))
    {
        state.sources.remove(&(bytecode as usize));
    }
}
unsafe fn js_host_free_object_diagnostics(rt: *mut JSRuntime, object: *mut JSObject) {
    if let Some(state) = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&(rt as usize))
    {
        state.creations.remove(&(object as usize));
    }
}
unsafe fn js_host_free_frame_diagnostics(rt: *mut JSRuntime, frame: *mut JSStackFrame) {
    if let Some(state) = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&(rt as usize))
    {
        state
            .suspended
            .retain(|(owner, _), _| *owner != frame as usize);
    }
}
struct JSHostSourceScope {
    runtime: *mut JSRuntime,
    enabled: bool,
    _source_positions: CompilerSourcePositionScope,
}
impl JSHostSourceScope {
    unsafe fn new(ctx: *mut JSContext, input: *const c_char, len: usize) -> Self {
        let source_positions = CompilerSourcePositionScope::new(input, len);
        let runtime = (*ctx).rt;
        let mut states = EXCEPTION_DIAGNOSTICS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let enabled = if let Some(state) = states.get_mut(&(runtime as usize)) {
            state.compiling.push(DiagnosticArc::new(DiagnosticSource {
                text: String::from_utf8_lossy(core::slice::from_raw_parts(input.cast::<u8>(), len))
                    .into_owned(),
                call_sites: Mutex::new(DiagnosticMap::new()),
                positions: std::sync::OnceLock::new(),
            }));
            true
        } else {
            false
        };
        Self { runtime, enabled, _source_positions: source_positions }
    }
}
impl Drop for JSHostSourceScope {
    fn drop(&mut self) {
        if self.enabled {
            if let Some(state) = EXCEPTION_DIAGNOSTICS
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get_mut(&(self.runtime as usize))
            {
                state.compiling.pop();
            }
        }
    }
}
unsafe fn js_host_register_evaluated_source(ctx: *mut JSContext, value: JSValueConst) {
    let runtime = (*ctx).rt;
    let mut states = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let Some(state) = states.get_mut(&(runtime as usize)) else {
        return;
    };
    let Some(source) = state.compiling.last().cloned() else {
        return;
    };
    let root = match JS_VALUE_GET_TAG(value) {
        JS_TAG_FUNCTION_BYTECODE => JS_VALUE_GET_PTR(value).cast::<JSFunctionBytecode>(),
        JS_TAG_MODULE => {
            JS_VALUE_GET_PTR((*JS_VALUE_GET_PTR(value).cast::<JSModuleDef>()).func_obj)
                .cast::<JSFunctionBytecode>()
        }
        _ => return,
    };
    let mut pending = vec![root];
    while let Some(bytecode) = pending.pop() {
        if bytecode.is_null() {
            continue;
        }
        state.sources.insert(bytecode as usize, source.clone());
        for index in 0..(*bytecode).cpool_count as usize {
            let child = *(*bytecode).cpool.add(index);
            if JS_VALUE_GET_TAG(child) == JS_TAG_FUNCTION_BYTECODE {
                pending.push(JS_VALUE_GET_PTR(child).cast());
            }
        }
    }
}
/// Owned optional diagnostic data for trusted compile-only bytecode transfer.
/// This contains source text, source offsets and Rust position indexes only;
/// it owns no JS value, runtime/context pointer, atom, or engine callback.
#[derive(Clone)]
pub struct JSCompiledSourceMetadata {
    source: DiagnosticArc<DiagnosticSource>,
}
unsafe fn js_host_compiled_source_root(value: JSValueConst) -> Option<*mut JSFunctionBytecode> {
    match JS_VALUE_GET_TAG(value) {
        JS_TAG_FUNCTION_BYTECODE => Some(JS_VALUE_GET_PTR(value).cast()),
        JS_TAG_MODULE => {
            let module = JS_VALUE_GET_PTR(value).cast::<JSModuleDef>();
            let function = (*module).func_obj;
            if JS_VALUE_GET_TAG(function) == JS_TAG_FUNCTION_BYTECODE {
                Some(JS_VALUE_GET_PTR(function).cast())
            } else { None }
        }
        _ => None,
    }
}
/// Export while the compiled value is alive on its owning engine thread.
/// Prewarming the pure Rust source index here avoids a first-throw full source
/// scan after destination hydration. No JS getter, job or script is executed.
pub unsafe fn JS_ExportCompiledSourceMetadata(
    ctx: *mut JSContext, value: JSValueConst,
) -> Option<JSCompiledSourceMetadata> {
    let root = js_host_compiled_source_root(value)?;
    let source = EXCEPTION_DIAGNOSTICS.lock().unwrap_or_else(|e| e.into_inner())
        .get(&((*ctx).rt as usize))?.sources.get(&(root as usize))?.clone();
    // The global map guard has been released before the potentially large
    // index construction; independent realms retain diagnostic access.
    source.positions.get_or_init(|| DiagnosticSourceIndex::new(&source.text));
    Some(JSCompiledSourceMetadata { source })
}
/// Register metadata only after a successful trusted JS_ReadObject on the
/// destination's owner thread. Traversal follows the same cpool children as
/// js_host_register_evaluated_source; destination finalizers remove entries.
/// Returns false for a noncompiled value or diagnostics-disabled runtime.
pub unsafe fn JS_ImportCompiledSourceMetadata(
    ctx: *mut JSContext, value: JSValueConst, metadata: &JSCompiledSourceMetadata,
) -> bool {
    let Some(root) = js_host_compiled_source_root(value) else { return false; };
    let mut states = EXCEPTION_DIAGNOSTICS.lock().unwrap_or_else(|e| e.into_inner());
    let Some(state) = states.get_mut(&((*ctx).rt as usize)) else { return false; };
    let mut pending = vec![root];
    while let Some(bytecode) = pending.pop() {
        if bytecode.is_null() { continue; }
        state.sources.insert(bytecode as usize, metadata.source.clone());
        let mut index = 0;
        while index < (*bytecode).cpool_count {
            let child = *(*bytecode).cpool.offset(index as isize);
            if JS_VALUE_GET_TAG(child) == JS_TAG_FUNCTION_BYTECODE {
                pending.push(JS_VALUE_GET_PTR(child).cast());
            }
            index += 1;
        }
    }
    true
}
#[cfg(test)]
#[test]
fn compiled_source_metadata_is_owned_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<JSCompiledSourceMetadata>();
}

// Diagnostic decoding never calls JS getters, allocators, or callbacks: those
// could themselves throw and alter the exception being reported.
unsafe fn js_host_atom_text(rt: *mut JSRuntime, atom: JSAtom) -> String {
    if atom & JS_ATOM_TAG_INT != 0 {
        return (atom & !JS_ATOM_TAG_INT).to_string();
    }
    if atom == 0 || atom >= (*rt).atom_size as u32 {
        return String::new();
    }
    let string = *(*rt).atom_array.add(atom as usize);
    if string.is_null() {
        return String::new();
    }
    let len = (*string).len() as usize;
    if (*string).is_wide_char() != 0 {
        String::from_utf16_lossy(core::slice::from_raw_parts(string_data16(string), len))
    } else {
        core::slice::from_raw_parts(string_data8(string), len)
            .iter()
            .map(|&ch| char::from(ch))
            .collect()
    }
}
fn js_host_raw_source_offset(source: &str, raw_line: i32, raw_column: i32) -> usize {
    let mut start = 0;
    for _ in 1..raw_line.max(1) {
        match source[start..].find('\n') {
            Some(offset) => start += offset + 1,
            None => {
                start = source.len();
                break;
            }
        }
    }
    let row_end = source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset);
    let chars = (raw_column.max(1) - 1) as usize;
    let index = source[start..row_end]
        .char_indices()
        .nth(chars)
        .map_or(row_end, |(offset, _)| start + offset);
    index
}
fn js_host_source_position(
    filename: String,
    source: &str,
    raw_line: i32,
    raw_column: i32,
) -> JSExceptionLocation {
    let index = js_host_raw_source_offset(source, raw_line, raw_column);
    js_host_source_position_at_offset(filename, source, index)
}
fn js_host_source_position_at_offset(
    filename: String,
    source: &str,
    index: usize,
) -> JSExceptionLocation {
    let mut line = 1;
    let mut column = 0;
    let mut line_start = 0;
    let mut last_cr = false;
    for (offset, ch) in source[..index].char_indices() {
        match ch {
            '\r' => {
                line += 1;
                column = 0;
                line_start = offset + 1;
                last_cr = true;
            }
            '\n' => {
                if !last_cr {
                    line += 1;
                }
                column = 0;
                line_start = offset + 1;
                last_cr = false;
            }
            '\u{2028}' | '\u{2029}' => {
                line += 1;
                column = 0;
                line_start = offset + ch.len_utf8();
                last_cr = false;
            }
            _ => {
                column += ch.len_utf16();
                last_cr = false;
            }
        }
    }
    let end = source[line_start..]
        .find(['\r', '\n', '\u{2028}', '\u{2029}'])
        .map_or(source.len(), |offset| line_start + offset);
    JSExceptionLocation {
        filename,
        line,
        column,
        source_line: source[line_start..end].to_owned(),
    }
}
unsafe fn js_host_bytecode_location(
    ctx: *mut JSContext,
    b: *mut JSFunctionBytecode,
    pc: *const u8,
    creation: bool,
) -> Option<JSExceptionLocation> {
    if b.is_null() || (*b).has_debug() == 0 || pc.is_null() || (*b).byte_code_buf.is_null() {
        return None;
    }
    let offset = pc.offset_from((*b).byte_code_buf) - 1;
    if offset < 0 || offset >= (*b).byte_code_len as isize {
        return None;
    }
    let mut column = 0;
    let line = find_line_num(ctx, b, offset as u32, &mut column);
    if line <= 0 {
        return None;
    }
    let states = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let source = states
        .get(&((*ctx).rt as usize))?
        .sources
        .get(&(b as usize))
        .cloned();
    let filename = js_host_atom_text((*ctx).rt, (*b).debug.filename);
    Some(match source {
        Some(source) => {
            let mut index = source.raw_source_offset(line, column);
            if creation {
                if let Some(&expression_start) = source
                    .call_sites
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(&index)
                {
                    index = expression_start;
                }
            }
            source.position_at_offset(filename, index)
        }
        None => JSExceptionLocation {
            filename,
            line: line as usize,
            column: column.max(1) as usize - 1,
            source_line: String::new(),
        },
    })
}
unsafe fn js_host_current_location(ctx: *mut JSContext) -> Option<JSExceptionLocation> {
    let mut frame = (*(*ctx).rt).current_stack_frame;
    while !frame.is_null() {
        let b = JS_GetFunctionBytecode((*frame).cur_func);
        if !b.is_null() {
            return js_host_bytecode_location(ctx, b, (*frame).cur_pc, true);
        }
        frame = (*frame).prev_frame;
    }
    None
}
unsafe fn js_host_vm_exception(ctx: *mut JSContext, b: *mut JSFunctionBytecode, pc: *const u8) {
    let rt = (*ctx).rt;
    if !js_host_diagnostics_enabled(rt) {
        return;
    }
    if EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&(rt as usize))
        .is_some_and(|state| state.captured)
    {
        return;
    }
    let location = js_host_bytecode_location(ctx, b, pc, false);
    if let Some(state) = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&(rt as usize))
    {
        state.pending = location;
        state.captured = true;
    }
}
unsafe fn js_host_catch_exception(
    rt: *mut JSRuntime,
    frame: *mut JSStackFrame,
    slot: *mut JSValue,
) {
    if let Some(state) = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&(rt as usize))
    {
        let location = state.pending.take();
        state.captured = false;
        state
            .suspended
            .insert((frame as usize, slot as usize), location);
    }
}
unsafe fn js_host_is_finally_rethrow(b: *mut JSFunctionBytecode, pc: *const u8) -> bool {
    let target = (pc.offset_from((*b).byte_code_buf) - 1) as usize;
    let mut offset = 0;
    let mut previous = 0;
    while offset < target {
        previous = *(*b).byte_code_buf.add(offset) as u16;
        let size = crate::quickjs_opcode::short_opcode_info(previous as usize).size as usize;
        if size == 0 {
            return false;
        }
        offset += size;
    }
    offset == target && previous == OP_gosub
}
unsafe fn js_host_vm_throw(
    ctx: *mut JSContext,
    b: *mut JSFunctionBytecode,
    frame: *mut JSStackFrame,
    pc: *const u8,
    slot: *mut JSValue,
    value: JSValue,
) -> JSValue {
    let rt = (*ctx).rt;
    let saved = if js_host_diagnostics_enabled(rt) && js_host_is_finally_rethrow(b, pc) {
        EXCEPTION_DIAGNOSTICS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(&(rt as usize))
            .and_then(|s| s.suspended.remove(&(frame as usize, slot as usize)))
    } else {
        None
    };
    let result = JS_Throw(ctx, value);
    if let Some(location) = saved {
        if let Some(state) = EXCEPTION_DIAGNOSTICS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(&(rt as usize))
        {
            state.pending = location;
            state.captured = true;
        }
    }
    result
}
struct JSHostPreserveException {
    captured: bool,
    runtime: *mut JSRuntime,
    saved: Option<JSExceptionLocation>,
    enabled: bool,
}
impl JSHostPreserveException {
    unsafe fn new(ctx: *mut JSContext, preserve: bool) -> Self {
        let runtime = (*ctx).rt;
        let enabled = preserve && js_host_diagnostics_enabled(runtime);
        // Drop the state guard before fetching the saved location below.
        let captured = EXCEPTION_DIAGNOSTICS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&(runtime as usize))
            .is_some_and(|state| state.captured);
        Self {
            runtime,
            enabled,
            captured,
            saved: if enabled {
                JS_GetExceptionMetadata(ctx)
            } else {
                None
            },
        }
    }
}
impl Drop for JSHostPreserveException {
    fn drop(&mut self) {
        if self.enabled {
            if let Some(state) = EXCEPTION_DIAGNOSTICS
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get_mut(&(self.runtime as usize))
            {
                state.pending = self.saved.take();
                state.captured = self.captured;
            }
        }
    }
}
unsafe fn js_host_error_creation(
    ctx: *mut JSContext,
    value: JSValueConst,
    filename: *const c_char,
    line: i32,
    column: i32,
) {
    let rt = (*ctx).rt;
    if !js_host_diagnostics_enabled(rt) || JS_VALUE_GET_TAG(value) != JS_TAG_OBJECT {
        return;
    }
    let key = JS_VALUE_GET_PTR(value) as usize;
    if EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&(rt as usize))
        .is_some_and(|s| s.creations.contains_key(&key))
    {
        return;
    }
    let location = if !filename.is_null() {
        let states = EXCEPTION_DIAGNOSTICS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let source = states.get(&(rt as usize)).and_then(|s| s.compiling.last());
        let name = core::ffi::CStr::from_ptr(filename)
            .to_string_lossy()
            .into_owned();
        source
            .map(|source| js_host_source_position(name.clone(), source, line, column))
            .or_else(|| {
                Some(JSExceptionLocation {
                    filename: name,
                    line: line.max(0) as usize,
                    column: column.max(1) as usize - 1,
                    source_line: String::new(),
                })
            })
    } else if JS_VALUE_GET_TAG((*rt).current_exception) == JS_TAG_OBJECT
        && JS_VALUE_GET_PTR((*rt).current_exception) == JS_VALUE_GET_PTR(value)
    {
        JS_GetExceptionMetadata(ctx).or_else(|| js_host_current_location(ctx))
    } else {
        js_host_current_location(ctx)
    };
    if let Some(location) = location {
        if let Some(state) = EXCEPTION_DIAGNOSTICS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(&(rt as usize))
        {
            state.creations.insert(key, location.clone());
            if !filename.is_null() {
                state.pending = Some(location);
                state.captured = true;
            }
        }
    }
}

unsafe fn js_host_parser_callsite(
    s: *mut JSParseState,
    call_token: *const u8,
    expression: *const u8,
) {
    let state = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let Some(source) = state
        .get(&((*(*s).ctx).rt as usize))
        .and_then(|state| state.compiling.last())
    else {
        return;
    };
    let call = call_token.offset_from((*s).buf_start);
    let start = expression.offset_from((*s).buf_start);
    if call >= 0
        && start >= 0
        && (call as usize) <= source.len()
        && (start as usize) <= source.len()
    {
        source
            .call_sites
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(call as usize, start as usize);
    }
}
// V8's formatted stack no longer retains the structured creation location.
// This affects only opt-in message diagnostics, never the actual stack value.
unsafe fn js_host_error_stack_read(ctx: *mut JSContext, value: JSValueConst, prop: JSAtom) {
    if prop != crate::quickjs_atom::JS_ATOM_stack as JSAtom
        || JS_VALUE_GET_TAG(value) != JS_TAG_OBJECT
    {
        return;
    }
    if let Some(state) = EXCEPTION_DIAGNOSTICS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&((*ctx).rt as usize))
    {
        if state.consume_creation_on_stack_read {
            state.creations.remove(&(JS_VALUE_GET_PTR(value) as usize));
        }
    }
}

#[cfg(test)]
mod diagnostic_source_index_tests {
    use super::*;
    #[test]
    fn sparse_source_positions_match_original_unicode_and_line_break_rules() {
        let sources = [String::new(), "A😀B\r\n漢\u{2028}字\u{2029}x\ry\nz".into(),
            format!("{}\r\n😀{}\u{2028}end", "a".repeat(255), "漢x".repeat(300))];
        for source in sources {
            let index = DiagnosticSourceIndex::new(&source);
            for offset in source.char_indices().map(|(offset,_)|offset).chain(std::iter::once(source.len())) {
                assert_eq!(index.position_at_offset("source.js".into(),&source,offset),
                    js_host_source_position_at_offset("source.js".into(),&source,offset),"offset {offset}");
            }
            for line in 0..7 {
                for column in 0..source.chars().count() as i32 + 3 {
                    assert_eq!(index.raw_source_offset(&source,line,column),
                        js_host_raw_source_offset(&source,line,column),"raw ({line},{column})");
                }
            }
            assert!(index.checkpoints.len() <= source.chars().count()/256+2);
        }
        let source = "A😀B\r\n漢\u{2028}字\u{2029}end";
        let index = DiagnosticSourceIndex::new(source);
        let location = index.position_at_offset("unicode.js".into(),source,5);
        assert_eq!((location.line,location.column,location.source_line.as_str()),(1,3,"A😀B"));
    }
}
