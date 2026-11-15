// C++: foundation/blink_base/wtf/wtf_size_t.h.

// cpp: foundation/blink_base/wtf/wtf_size_t.h:39-40
pub type WtfSizeT = u32;
pub const kNotFound: WtfSizeT = WtfSizeT::MAX;

// cpp: foundation/blink_base/wtf/wtf_size_t.h:42-45
// A Rust range replaces the C++ begin/end iterator pair.
pub fn CheckedDistance<I: IntoIterator>(range: I) -> WtfSizeT {
    WtfSizeT::try_from(range.into_iter().count()).expect("WTF vector size exceeds 32 bits")
}
