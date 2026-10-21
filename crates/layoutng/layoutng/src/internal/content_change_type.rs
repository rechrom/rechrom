// cpp: layoutng/internal/content_change_type.h:10-10
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ContentChangeType {
    kCanvasChanged = 0,
    kCanvasContextChanged = 1,
}
