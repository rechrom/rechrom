// cpp: third_party/blink/renderer/core/css/css_origin_clean.h:1-11

// https://drafts.csswg.org/cssom-1/#concept-css-style-sheet-origin-clean-flag
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginClean {
    kFalse,
    kTrue,
}
