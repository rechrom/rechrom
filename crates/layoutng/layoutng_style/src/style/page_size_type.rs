// cpp: layoutng_style/style/page_size_type.h:10-21
/// Information supplied by the CSS @page size descriptor.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum PageSizeType {
    kAuto,
    kPortrait,
    kLandscape,
    kFixed,
}
