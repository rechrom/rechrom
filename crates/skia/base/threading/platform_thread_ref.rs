// Source: base/threading/platform_thread_ref.h:36-63
#[cfg(any(base_is_posix, base_is_fuchsia))]
pub type RefType = libc::pthread_t;
#[cfg(base_is_win)]
pub type RefType = u32;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PlatformThreadRef {
    id_: RefType,
}
// Native identity is explicitly copyable between threads, never dereferenced as a pthread object.
unsafe impl Send for PlatformThreadRef {}
unsafe impl Sync for PlatformThreadRef {}
impl PlatformThreadRef {
    pub const fn new() -> Self {
        Self { id_: 0 as RefType }
    }
    pub const fn from_id(id: RefType) -> Self {
        Self { id_: id }
    }
    pub fn is_null(&self) -> bool {
        self.id_ == 0 as RefType
    }
}
impl Default for PlatformThreadRef {
    fn default() -> Self {
        Self::new()
    }
}
// Native ostream operator has no definition in these admitted files; no synthetic formatter.
