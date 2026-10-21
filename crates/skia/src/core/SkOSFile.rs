//! Native FILE operations used by Stream and Data.
//! Platform FILE/stat layouts are supplied by libc's system bindings.
//! POSIX directory iteration is available; Windows/iOS-specific paths remain pending.

pub use libc::FILE;

// Source: src/core/SkOSFile.h:19-22
// Integer flags preserve native bitwise combinations, including zero.
pub type SkFILE_Flags = u32;
pub const kRead_SkFILE_Flag: SkFILE_Flags = 0x01;
pub const kWrite_SkFILE_Flag: SkFILE_Flags = 0x02;

// Source: src/core/SkOSFile.h:24-34
pub use crate::src::ports::SkOSFile_posix::sk_fsync;
pub use crate::src::ports::SkOSFile_stdio::{
    sk_fclose, sk_fflush, sk_fgetsize, sk_fopen, sk_ftell, sk_fwrite,
};
// Source: src/core/SkOSFile.h:40-59
pub use crate::src::ports::SkOSFile_posix::{
    sk_fdmmap, sk_fidentical, sk_fileno, sk_fmmap, sk_fmunmap,
};
// Source: src/core/SkOSFile.h:64-77
pub use crate::src::ports::SkOSFile_posix::{sk_exists, sk_qread};
pub use crate::src::ports::SkOSFile_stdio::{sk_isdir, sk_mkdir};
// Source: src/core/SkOSFile.h:81-97
#[cfg(not(target_os = "ios"))]
pub use crate::src::ports::SkOSFile_posix::Iter;
