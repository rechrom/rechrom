// C++: src/foundation/text/native/unicode_bidi.h

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnicodeBidi {
    kNormal,
    kEmbed,
    kBidiOverride,
    kIsolate,
    kPlaintext,
    kIsolateOverride,
}

// C++: unicode_bidi.h, IsIsolated.
pub fn IsIsolated(unicode_bidi: &UnicodeBidi) -> bool {
    *unicode_bidi == UnicodeBidi::kIsolate
        || *unicode_bidi == UnicodeBidi::kIsolateOverride
        || *unicode_bidi == UnicodeBidi::kPlaintext
}

// C++: unicode_bidi.h, IsOverride.
pub fn IsOverride(unicode_bidi: UnicodeBidi) -> bool {
    unicode_bidi == UnicodeBidi::kBidiOverride || unicode_bidi == UnicodeBidi::kIsolateOverride
}
