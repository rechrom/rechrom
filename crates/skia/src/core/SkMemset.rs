// The public native pointer/count types remain distinct from the existing
// safe slice/usize overloads. Complete writable storage leases remain required.
// Source: src/core/SkMemset.h:15-15
pub type Memset16Proc = unsafe fn(*mut u16, u16, i32);
pub use super::SkMemset_opts::memset16;
// Source: src/core/SkMemset.h:16-16
pub type Memset32Proc = unsafe fn(*mut u32, u32, i32);
pub use super::SkMemset_opts::memset32;
// Source: src/core/SkMemset.h:17-17
pub type Memset64Proc = unsafe fn(*mut u64, u64, i32);
pub use super::SkMemset_opts::memset64;
// Source: src/core/SkMemset.h:19-19
pub type RectMemset16Proc = unsafe fn(*mut u16, u16, i32, usize, i32);
pub use super::SkMemset_opts::rect_memset16;
// Source: src/core/SkMemset.h:20-20
pub type RectMemset32Proc = unsafe fn(*mut u32, u32, i32, usize, i32);
pub use super::SkMemset_opts::rect_memset32;
// Source: src/core/SkMemset.h:21-21
pub type RectMemset64Proc = unsafe fn(*mut u64, u64, i32, usize, i32);
pub use super::SkMemset_opts::rect_memset64;
// Source: src/core/SkMemset.h:23-23
pub use super::SkMemset_opts::Init_Memset;
