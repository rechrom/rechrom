// Copyright 2016 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/state_machines/state_machine_util.h:14-32;
// core/editing/state_machines/state_machine_util.cc:16-123.
use font_engine::text::implementation::unicode_category::Category;
use foundation::{
    emoji_property_data::{kEmojiRanges, HasEmojiProperty},
    grapheme_break_property_data::{
        GraphemeBreakProperty, GraphemeProperty as G, IsExtendedPictographic,
    },
    unicode::kLetter_Other,
};
pub(super) fn IsLead(unit: u16) -> bool {
    (0xd800..=0xdbff).contains(&unit)
}
pub(super) fn IsTrail(unit: u16) -> bool {
    (0xdc00..=0xdfff).contains(&unit)
}
pub(super) fn Supplementary(lead: u16, trail: u16) -> u32 {
    0x10000 + ((u32::from(lead) - 0xd800) << 10) + u32::from(trail) - 0xdc00
}
pub(super) fn IsRegionalIndicator(ch: u32) -> bool {
    (0x1f1e6..=0x1f1ff).contains(&ch)
}
fn IsIndicSyllabicCategoryVirama(ch: u32) -> bool {
    const VIRAMA: [u32; 23] = [
        0x094D, 0x09CD, 0x0A4D, 0x0ACD, 0x0B4D, 0x0C4D, 0x0CCD, 0x0D4D, 0x0DCA, 0x1B44, 0xA8C4,
        0xA9C0, 0x11046, 0x110B9, 0x111C0, 0x11235, 0x1134D, 0x11442, 0x114C2, 0x115BF, 0x1163F,
        0x116B6, 0x11C3F,
    ];
    VIRAMA.binary_search(&ch).is_ok()
}
pub(super) fn IsGraphemeBreak(previous: u32, next: u32) -> bool {
    let p = GraphemeBreakProperty(previous);
    let n = GraphemeBreakProperty(next);
    if p == G::kCr && n == G::kLf {
        return false;
    }
    if matches!(p, G::kControl | G::kCr | G::kLf) || matches!(n, G::kControl | G::kCr | G::kLf) {
        return true;
    }
    if p == G::kL && matches!(n, G::kL | G::kV | G::kLv | G::kLvt) {
        return false;
    }
    if matches!(p, G::kLv | G::kV) && matches!(n, G::kV | G::kT) {
        return false;
    }
    if matches!(p, G::kLvt | G::kT) && n == G::kT {
        return false;
    }
    assert!(
        !(IsRegionalIndicator(previous) && IsRegionalIndicator(next)),
        "regional indicator parity belongs to the source machine"
    );
    if matches!(n, G::kExtend | G::kSpacingMark) || next == 0x200d {
        return false;
    }
    if p == G::kPrepend {
        return false;
    }
    if IsIndicSyllabicCategoryVirama(previous) && Category(next as i32) == kLetter_Other {
        return false;
    }
    if previous == 0x200d && (IsExtendedPictographic(next) || HasEmojiProperty(next, &kEmojiRanges))
    {
        return false;
    }
    true
}
