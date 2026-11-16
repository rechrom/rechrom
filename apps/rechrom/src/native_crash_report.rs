//! Report native faults (access violations and other SEH exceptions) to
//! stderr before the process terminates, corresponding to the role of
//! Chromium's in-process crash handler (components/crash/core/app/
//! crashpad_win.cc installs an unhandled-exception filter). A Rust panic
//! already reports through its hook; a hardware fault otherwise ends the
//! process silently.
//!
//! The report is one structured `rechrom-native-crash` JSON line with the
//! exception code, the faulting address and the faulting thread's return
//! addresses, all as offsets from the executable image base so they can be
//! resolved against a linker map of the same build.

pub(crate) fn install() {
    #[cfg(target_os = "windows")]
    windows::install();
}

#[cfg(target_os = "windows")]
mod windows {
    use std::ffi::c_void;
    use std::io::Write;

    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *mut ExceptionRecord,
        address: *mut c_void,
        number_parameters: u32,
        information: [usize; 15],
    }

    #[repr(C)]
    struct ExceptionPointers {
        record: *mut ExceptionRecord,
        context: *mut c_void,
    }

    type TopLevelExceptionFilter = unsafe extern "system" fn(*const ExceptionPointers) -> i32;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetUnhandledExceptionFilter(
            filter: Option<TopLevelExceptionFilter>,
        ) -> Option<TopLevelExceptionFilter>;
        fn RtlCaptureStackBackTrace(
            frames_to_skip: u32,
            frames_to_capture: u32,
            back_trace: *mut *mut c_void,
            back_trace_hash: *mut u32,
        ) -> u16;
        fn GetModuleHandleW(module_name: *const u16) -> *mut c_void;
    }

    // winbase.h EXCEPTION_CONTINUE_SEARCH: let Windows Error Reporting run.
    const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
    const MAX_FRAMES: usize = 62;

    pub(super) fn install() {
        unsafe { SetUnhandledExceptionFilter(Some(filter)) };
    }

    fn image_offset(base: usize, address: usize) -> String {
        match address.checked_sub(base) {
            Some(offset) if offset < 0x4000_0000 => format!("\"0x{offset:x}\""),
            _ => format!("\"abs:0x{address:x}\""),
        }
    }

    unsafe extern "system" fn filter(info: *const ExceptionPointers) -> i32 {
        let base = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
        let record = unsafe { (*info).record.as_ref() };
        let (code, address, access) = record.map_or((0, 0, None), |record| {
            let access = (record.code == 0xC000_0005 && record.number_parameters >= 2)
                .then(|| (record.information[0], record.information[1]));
            (record.code, record.address as usize, access)
        });
        let mut frames = [std::ptr::null_mut::<c_void>(); MAX_FRAMES];
        let captured = unsafe {
            RtlCaptureStackBackTrace(
                0,
                MAX_FRAMES as u32,
                frames.as_mut_ptr(),
                std::ptr::null_mut(),
            )
        } as usize;
        let frames = frames[..captured]
            .iter()
            .map(|frame| image_offset(base, *frame as usize))
            .collect::<Vec<_>>()
            .join(",");
        let thread = std::thread::current();
        let access = access.map_or(String::new(), |(kind, target)| {
            format!(",\"access\":{kind},\"target\":\"0x{target:x}\"")
        });
        let line = format!(
            "rechrom-native-crash\t{{\"code\":\"0x{code:08x}\",\"address\":{}{access},\"thread\":{:?},\"frames\":[{frames}]}}\n",
            image_offset(base, address),
            thread.name().unwrap_or("unnamed"),
        );
        let _ = std::io::stderr().write_all(line.as_bytes());
        EXCEPTION_CONTINUE_SEARCH
    }
}
