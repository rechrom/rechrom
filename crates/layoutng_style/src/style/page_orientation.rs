// cpp: layoutng_style/style/page_orientation.h:10-12
/// Values of the CSS @page page-orientation descriptor.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum PageOrientation {
    kUpright,
    kRotateLeft,
    kRotateRight,
}
