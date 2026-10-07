// Opt-in embedding diagnostics. Never call JS, set an exception, or alter the
// interrupt handler. Snapshot native frames only at existing interrupt polls.
static JS_JOB_STACK_PROFILE_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static JS_JOB_STACK_PROFILE_SEQUENCE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

fn js_job_stack_profile_enabled() -> bool {
    *JS_JOB_STACK_PROFILE_ENABLED.get_or_init(|| {
        std::env::var_os("BROWSER_PROFILE_JS_STACK").is_some_and(|value| value == "1")
    })
}

struct JSJobStackProfile {
    runtime: *mut JSRuntime,
    sequence: u64,
    job_func: usize,
    started: std::time::Instant,
    next_threshold: usize,
    snapshots: Vec<(f64, String)>,
}

std::thread_local! {
    static JS_JOB_STACK_PROFILE: std::cell::RefCell<Option<JSJobStackProfile>> =
        const { std::cell::RefCell::new(None) };
}

struct JSJobStackProfileGuard {
    previous: Option<JSJobStackProfile>,
}

impl Drop for JSJobStackProfileGuard {
    fn drop(&mut self) {
        let profile = JS_JOB_STACK_PROFILE.with(|slot| {
            slot.replace(self.previous.take()).expect("active job stack profile")
        });
        let elapsed_ms = profile.started.elapsed().as_secs_f64() * 1000.0;
        if elapsed_ms < 16.0 {
            return;
        }
        // Print after execution, outside the polling path. Ignore diagnostic
        // I/O errors; they must not affect JS completion or pending exceptions.
        use std::io::Write;
        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(stderr,
            "quickjs-slow-job id={} rt={:p} job_func=0x{:x} execute_ms={elapsed_ms:.3} snapshots={}",
            profile.sequence, profile.runtime, profile.job_func, profile.snapshots.len());
        for (sample_ms, stack) in profile.snapshots {
            let _ = writeln!(stderr,
                "quickjs-slow-job-stack id={} elapsed_ms={sample_ms:.3} stack={stack:?}",
                profile.sequence);
        }
    }
}

fn js_job_stack_profile_begin(
    runtime: *mut JSRuntime,
    job_func: usize,
) -> Option<JSJobStackProfileGuard> {
    if !js_job_stack_profile_enabled() {
        return None;
    }
    let profile = JSJobStackProfile {
        runtime,
        sequence: JS_JOB_STACK_PROFILE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1,
        job_func,
        started: std::time::Instant::now(),
        next_threshold: 0,
        snapshots: Vec::new(),
    };
    Some(JSJobStackProfileGuard {
        previous: JS_JOB_STACK_PROFILE.with(|slot| slot.replace(Some(profile))),
    })
}

// Read atom storage directly, with bounded Rust allocations. In particular do
// not read Function.name through JS properties: getters could execute JS.
unsafe fn js_job_stack_profile_atom(rt: *mut JSRuntime, atom: JSAtom) -> String {
    if __JS_AtomIsTaggedInt(atom) != 0 {
        return __JS_AtomToUInt32(atom).to_string();
    }
    if atom == JS_ATOM_NULL as u32 || atom >= (*rt).atom_size as u32 {
        return String::new();
    }
    let string = *(*rt).atom_array.add(atom as usize);
    if string.is_null() || atom_is_free(string) != 0 {
        return String::new();
    }
    let length = ((*string).len() as usize).min(512);
    let data = ptr::addr_of!((*string).u);
    let mut result = if (*string).is_wide_char() != 0 {
        String::from_utf16_lossy(std::slice::from_raw_parts(data.cast::<u16>(), length))
    } else {
        std::slice::from_raw_parts(data.cast::<u8>(), length)
            .iter().map(|&byte| char::from(byte)).collect()
    };
    if (*string).len() as usize > length {
        result.push_str("...");
    }
    result
}

unsafe fn js_job_stack_profile_poll(ctx: *mut JSContext, current_pc: *const u8) {
    if !js_job_stack_profile_enabled() {
        return;
    }
    const THRESHOLDS_MS: [u64; 4] = [16, 64, 256, 512];
    JS_JOB_STACK_PROFILE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(profile) = slot.as_mut() else { return; };
        let rt = (*ctx).rt;
        if profile.runtime != rt || profile.next_threshold == THRESHOLDS_MS.len() {
            return;
        }
        let elapsed = profile.started.elapsed();
        if elapsed < std::time::Duration::from_millis(THRESHOLDS_MS[profile.next_threshold]) {
            return;
        }
        // A late first poll produces one sample, not several identical samples.
        while profile.next_threshold < THRESHOLDS_MS.len()
            && elapsed >= std::time::Duration::from_millis(THRESHOLDS_MS[profile.next_threshold])
        {
            profile.next_threshold += 1;
        }
        use std::fmt::Write;
        let mut stack = String::new();
        let mut frame = (*rt).current_stack_frame;
        let mut depth = 0;
        while !frame.is_null() && depth < 48 {
            let function = (*frame).cur_func;
            if JS_VALUE_GET_TAG(function) == JS_TAG_OBJECT {
                let object = JS_VALUE_GET_PTR(function).cast::<JSObject>();
                if js_class_has_bytecode((*object).class_id as u32) != 0 {
                    let bytecode = (*object).u.func.function_bytecode;
                    let name = js_job_stack_profile_atom(rt, (*bytecode).func_name);
                    // VM backward-branch polls supply their live PC without
                    // modifying frame.cur_pc. Caller frames retain saved PCs.
                    let live_pc = depth == 0 && !current_pc.is_null();
                    let pc = if live_pc {
                        current_pc
                    } else {
                        (*frame).cur_pc
                    };
                    let offset = (pc as usize).checked_sub((*bytecode).byte_code_buf as usize)
                        .filter(|offset| *offset <= (*bytecode).byte_code_len as usize)
                        .map(|offset| if live_pc { offset as u32 } else { offset.saturating_sub(1) as u32 });
                    let mut column = 0;
                    let line = offset.map(|offset| find_line_num(ctx, bytecode, offset, &mut column))
                        .unwrap_or(0);
                    let file = if (*bytecode).has_debug() != 0 {
                        js_job_stack_profile_atom(rt, (*bytecode).debug.filename)
                    } else { String::new() };
                    let _ = writeln!(stack,
                        "#{depth} function={name:?} object={object:p} bytecode={bytecode:p} pc={offset:?} file={file:?} line={line} column={column} pc_kind={}",
                        if live_pc { "live" } else { "saved" });
                } else {
                    let class = js_job_stack_profile_atom(rt,
                        (*(*rt).class_array.add((*object).class_id as usize)).class_name);
                    let _ = writeln!(stack, "#{depth} native_class={class:?} object={object:p}");
                }
            }
            frame = (*frame).prev_frame;
            depth += 1;
        }
        if !frame.is_null() {
            stack.push_str("<stack truncated after 48 frames>\n");
        }
        profile.snapshots.push((elapsed.as_secs_f64() * 1000.0, stack));
    });
}
