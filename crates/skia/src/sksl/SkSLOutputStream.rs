// UNVERIFIED: user prohibited all validation.
use core::ffi::{c_char, c_void};
// Source: src/sksl/SkSLOutputStream.h:20-20
pub unsafe trait OutputStreamBytes {
    // Source: src/sksl/SkSLOutputStream.h:22-24
    fn isValid(&self) -> bool {
        true
    }
    // Source: src/sksl/SkSLOutputStream.h:26-26
    fn write8(&mut self, b: u8);
}
// Rust extension prevents overriding native nonvirtual byte-order methods.
pub trait OutputStreamExt: OutputStreamBytes {
    // Source: src/sksl/SkSLOutputStream.h:28-31
    fn write16(&mut self, i: u16) {
        self.write8(i as u8);
        self.write8((i >> 8) as u8);
    }
    // Source: src/sksl/SkSLOutputStream.h:33-38
    fn write32(&mut self, i: u32) {
        self.write8(i as u8);
        self.write8((i >> 8) as u8);
        self.write8((i >> 16) as u8);
        self.write8((i >> 24) as u8);
    }
}
impl<T: OutputStreamBytes + ?Sized> OutputStreamExt for T {}
pub unsafe trait OutputStream: OutputStreamBytes {
    // Source: src/sksl/SkSLOutputStream.h:40-40
    unsafe fn writeText(&mut self, s: *const c_char);
    // Source: src/sksl/SkSLOutputStream.h:42-42
    unsafe fn write(&mut self, s: *const c_void, size: usize);
    // Source: src/sksl/SkSLOutputStream.h:50-50
    // Native virtual destruction is supplied by the actual concrete Rust owner Drop.
}
// Source: src/sksl/SkSLOutputStream.h:53-53
const kBufferSize: i32 = 1024;
// Source: src/sksl/SkSLOutputStream.cpp:10-11
use core::{
    ffi::{c_int, VaList},
    mem::MaybeUninit,
};
unsafe extern "C" {
    fn vsnprintf(
        buffer: *mut c_char,
        size: usize,
        format: *const c_char,
        args: VaList<'_>,
    ) -> c_int;
}

// These are native nonvirtual members of the existing virtual receiver. Inherent
// dyn methods retain that dispatch boundary without adding overridable trait
// methods or an independent stream/formatter owner. Concrete callers borrow the
// same receiver as &mut dyn OutputStream. No C++ trait-object ABI is asserted.
impl dyn OutputStream + '_ {
    // Source: src/sksl/SkSLOutputStream.cpp:15-17
    /// Safety: receiver accepts the original synchronous readable-byte write.
    pub unsafe fn writeString(&mut self, s: &[u8]) {
        self.write(s.as_ptr().cast(), s.len());
    }

    // Source: src/sksl/SkSLOutputStream.cpp:19-24
    /// Safety: terminated format, actual promoted C arguments and all referenced
    /// backing satisfy printf's contract for the complete synchronous write.
    pub unsafe extern "C" fn printf(&mut self, format: *const c_char, mut args: ...) {
        // The compiler-created VaList is va_start; this frame owns its va_end.
        self.appendVAList(format, &mut args);
        drop(args);
    }

    // Source: src/sksl/SkSLOutputStream.cpp:26-39
    /// Safety: args is a real live native list, borrowed from its owning frame.
    /// Format/arguments match; pointers survive both formatting calls. The bytes
    /// passed to write must be initialized by those calls, as required by the
    /// native raw-buffer algorithm. No initialized Rust slice is fabricated.
    pub unsafe fn appendVAList(&mut self, format: *const c_char, args: &mut VaList<'_>) {
        let mut buffer = [MaybeUninit::<c_char>::uninit(); kBufferSize as usize];
        let copy = (*args).clone(); // real va_copy; its owner remains in this frame
                                    // ptr::read lends the compiler's opaque cursor representation to this
                                    // synchronous C call. The FFI view never returns to Rust Drop; the actual
                                    // caller-owned list remains available for exactly its original va_end.
        let length = vsnprintf(
            buffer.as_mut_ptr().cast(),
            kBufferSize as usize,
            format,
            core::ptr::read(args),
        );
        if length > kBufferSize {
            // ADAPTED checked rejection: native signed length+1 must not overflow.
            let count = length
                .checked_add(1)
                .expect("native vsnprintf length+1 exceeds its defined signed domain")
                as usize;
            let mut bigBuffer = Vec::<MaybeUninit<c_char>>::with_capacity(count);
            bigBuffer.set_len(count);
            vsnprintf(
                bigBuffer.as_mut_ptr().cast(),
                count,
                format,
                core::ptr::read(&copy),
            );
            self.write(bigBuffer.as_ptr().cast(), length as usize);
        } else {
            // ADAPTED checked rejection of native negative-length overread. This
            // does not change the native > comparison to >=: length==1024 still
            // writes all 1024 stack bytes, including snprintf's terminal NUL.
            let count = usize::try_from(length)
                .expect("negative native vsnprintf length cannot name a readable stack span");
            self.write(buffer.as_ptr().cast(), count);
        }
        drop(copy); // va_end(copy); the borrowed original is ended by its owner
    }
}
