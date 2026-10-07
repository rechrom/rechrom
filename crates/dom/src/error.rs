// Rust does not have C++ exception classes. The source's two DOM exception
// categories remain distinguishable to catch_unwind callers through panic_any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMExceptionKind {
    InvalidArgument,
    LogicError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMException {
    pub kind: DOMExceptionKind,
    pub message: &'static str,
}

impl std::fmt::Display for DOMException {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for DOMException {}

pub fn invalid_argument(message: &'static str) -> ! {
    std::panic::panic_any(DOMException {
        kind: DOMExceptionKind::InvalidArgument,
        message,
    })
}

pub fn logic_error(message: &'static str) -> ! {
    std::panic::panic_any(DOMException {
        kind: DOMExceptionKind::LogicError,
        message,
    })
}
