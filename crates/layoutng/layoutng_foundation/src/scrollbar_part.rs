//! Shared scalar identity from Blink core/scroll/scroll_types.h:155-167.
#![allow(non_camel_case_types, non_snake_case)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScrollbarPart {
    kNoPart = 0,
    kBackButtonStartPart = 1,
    kForwardButtonStartPart = 1 << 1,
    kBackTrackPart = 1 << 2,
    kThumbPart = 1 << 3,
    kForwardTrackPart = 1 << 4,
    kBackButtonEndPart = 1 << 5,
    kForwardButtonEndPart = 1 << 6,
    kScrollbarBGPart = 1 << 7,
    kTrackBGPart = 1 << 8,
    kAllParts = 0xffff_ffff,
}
impl crate::Traceable for ScrollbarPart {
    fn Trace(&self, _visitor: &mut crate::Visitor<'_>) {}
}
impl crate::heap_hash_containers::StrongHeapMapKey for ScrollbarPart {}
