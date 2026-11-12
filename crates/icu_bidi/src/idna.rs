//! ICU UTS #46 owner used by Chromium email_input_type.cc:92-119.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15; ICU uidna.h:203-228,312-315.
use std::ptr::NonNull;
pub enum UIDNA {}
pub const UIDNA_CHECK_BIDI: u32 = 4;
#[repr(C)]
struct UIDNAInfo {
    size: i16,
    is_transitional_different: i8,
    reserved_b3: i8,
    errors: u32,
    reserved_i2: i32,
    reserved_i3: i32,
}
impl UIDNAInfo {
    fn new() -> Self {
        Self {
            size: std::mem::size_of::<Self>() as i16,
            is_transitional_different: 0,
            reserved_b3: 0,
            errors: 0,
            reserved_i2: 0,
            reserved_i3: 0,
        }
    }
}
unsafe extern "C" {
    #[link_name = concat!("uidna_openUTS46_", env!("ICU_MAJOR"))]
    fn uidna_openUTS46(options: u32, status: *mut i32) -> *mut UIDNA;
    #[link_name = concat!("uidna_close_", env!("ICU_MAJOR"))]
    fn uidna_close(idna: *mut UIDNA);
    #[link_name=concat!("uidna_labelToUnicode_",env!("ICU_MAJOR"))]
    fn uidna_labelToUnicode(
        idna: *const UIDNA,
        name: *const u16,
        length: i32,
        output: *mut u16,
        capacity: i32,
        info: *mut UIDNAInfo,
        status: *mut i32,
    ) -> i32;
    #[link_name = concat!("uidna_nameToASCII_", env!("ICU_MAJOR"))]
    fn uidna_nameToASCII(
        idna: *const UIDNA,
        name: *const u16,
        length: i32,
        output: *mut u16,
        capacity: i32,
        info: *mut UIDNAInfo,
        status: *mut i32,
    ) -> i32;
}
pub struct Idna(NonNull<UIDNA>);
// ICU uidna.h:148 explicitly permits concurrent use of one immutable UIDNA.
unsafe impl Send for Idna {}
unsafe impl Sync for Idna {}
pub struct IdnaResult {
    pub value: Vec<u16>,
    pub failed: bool,
    pub errors: u32,
}
impl Idna {
    /// url_formatter.cc:484-503 uses a 64-unit buffer and retries the exact
    /// required capacity only for overflow without IDNA errors.
    pub fn LabelToUnicode(&self, name: &[u16]) -> IdnaResult {
        let Ok(length) = i32::try_from(name.len()) else {
            return IdnaResult {
                value: vec![],
                failed: true,
                errors: 0,
            };
        };
        let mut value = vec![0; 64];
        let mut info = UIDNAInfo::new();
        loop {
            let mut status = 0;
            let written = unsafe {
                uidna_labelToUnicode(
                    self.0.as_ptr(),
                    name.as_ptr(),
                    length,
                    value.as_mut_ptr(),
                    i32::try_from(value.len()).expect("IDNA output capacity"),
                    &mut info,
                    &mut status,
                )
            };
            if status == 15 && info.errors == 0 && written >= 0 {
                value.resize(written as usize, 0);
                continue;
            }
            if status > 0 || info.errors != 0 || written < 0 {
                return IdnaResult {
                    value: vec![],
                    failed: status > 0,
                    errors: info.errors,
                };
            }
            value.truncate(written as usize);
            return IdnaResult {
                value,
                failed: false,
                errors: 0,
            };
        }
    }

    pub fn OpenUTS46(options: u32) -> Result<Self, i32> {
        let mut status = 0;
        let idna = unsafe { uidna_openUTS46(options, &mut status) };
        let owner = NonNull::new(idna).map(Self);
        if status > 0 {
            return Err(status);
        }
        owner.ok_or(7) // U_MEMORY_ALLOCATION_ERROR
    }
    pub fn NameToASCII(&self, name: &[u16]) -> IdnaResult {
        let Ok(length) = i32::try_from(name.len()) else {
            return IdnaResult {
                value: Vec::new(),
                failed: true,
                errors: 0,
            };
        };
        let mut value = vec![0; name.len().max(64)];
        let mut info = UIDNAInfo::new();
        let mut status = 0;
        let mut written = unsafe {
            uidna_nameToASCII(
                self.0.as_ptr(),
                name.as_ptr(),
                length,
                value.as_mut_ptr(),
                value.len() as i32,
                &mut info,
                &mut status,
            )
        };
        if status == 15 && written >= 0 {
            // U_BUFFER_OVERFLOW_ERROR: exact required capacity
            value.resize(written as usize, 0);
            status = 0;
            info = UIDNAInfo::new();
            written = unsafe {
                uidna_nameToASCII(
                    self.0.as_ptr(),
                    name.as_ptr(),
                    length,
                    value.as_mut_ptr(),
                    written,
                    &mut info,
                    &mut status,
                )
            };
        }
        let failed = status > 0 || written < 0;
        if failed {
            value.clear()
        } else {
            value.truncate(written as usize)
        }
        IdnaResult {
            value,
            failed,
            errors: info.errors,
        }
    }
}
impl Drop for Idna {
    fn drop(&mut self) {
        unsafe { uidna_close(self.0.as_ptr()) }
    }
}
