// Source: include/private/SkLogPriority.h:27-32
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum SkLogPriority {
    kError = 0,
    kWarning = 1,
    kInfo = 2,
    kDebug = 3,
}
