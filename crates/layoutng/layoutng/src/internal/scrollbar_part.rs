// cpp: layoutng/internal/scrollbar_part.h:28-40
// The all-parts enumerator requires an unsigned 32-bit representation.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
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

// Scrollbar part values are scalar keys with no GC edges.
impl foundation::Traceable for ScrollbarPart {
    fn Trace(&self, _visitor: &mut foundation::Visitor<'_>) {}
}
impl foundation::heap_hash_containers::StrongHeapMapKey for ScrollbarPart {}
