// Generated directly from the supplied C++ keywords.h/.cc.
// C++ constructs static AtomicString references in one backing array. Rust stores
// the same interned names in a process-lifetime array and exposes stable lazy handles.
#![allow(non_upper_case_globals, non_snake_case)]
use crate::AtomicString;
use std::sync::LazyLock;

// cpp: foundation/style_values/style/keywords.h:158
pub const kNamesCount: usize = 137;

// cpp: foundation/style_values/style/keywords.h:160
// cpp: foundation/style_values/style/keywords.cc:162-317
pub fn Init() {
    let _ = LazyLock::force(&NAMES);
}

// cpp: foundation/style_values/style/keywords.cc:172-310
const SPELLINGS: [&str; kNamesCount] = [
    "-internal-print-footer",
    "-internal-print-header",
    "-internal-print-page-number",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    "End",
    "Enter",
    "Escape",
    "Home",
    "PageDown",
    "PageUp",
    "Tab",
    "anonymous",
    "any",
    "application/xml",
    "async",
    "auto",
    "button",
    "camera",
    "circle",
    "close",
    "closed",
    "closerequest",
    "col",
    "colgroup",
    "color-scheme",
    "decimal",
    "default",
    "disc",
    "disclosure-closed",
    "disclosure-open",
    "display-p3",
    "done",
    "eager",
    "email",
    "enter",
    "exit-fullscreen",
    "false",
    "fetch",
    "font",
    "go",
    "hidden",
    "hide",
    "hide-overscroll",
    "hide-popover",
    "high",
    "hint",
    "image",
    "inherit",
    "invisible",
    "lazy",
    "limited-srgb",
    "low",
    "lower-alpha",
    "lower-roman",
    "manual",
    "microphone",
    "module",
    "multiple",
    "named",
    "never",
    "next",
    "no-referrer",
    "no-referrer-when-downgrade",
    "nodownload",
    "nofullscreen",
    "none",
    "noplaybackrate",
    "noremoteplayback",
    "numeric",
    "off",
    "on",
    "open",
    "origin",
    "origin-when-cross-origin",
    "page-block-end",
    "page-block-start",
    "page-down",
    "page-inline-end",
    "page-inline-start",
    "page-left",
    "page-right",
    "page-up",
    "pause",
    "plaintext-only",
    "play",
    "play-pause",
    "presentation",
    "previous",
    "render",
    "request-close",
    "request-fullscreen",
    "reset",
    "responsive-embedded-sizing",
    "row",
    "rowgroup",
    "same-origin",
    "script",
    "search",
    "send",
    "show",
    "show-modal",
    "show-overscroll",
    "show-picker",
    "show-popover",
    "single",
    "square",
    "static",
    "step-down",
    "step-up",
    "strict-origin",
    "strict-origin-when-cross-origin",
    "style",
    "submit",
    "sync",
    "tel",
    "text",
    "text/css",
    "text/html",
    "toggle",
    "toggle-fullscreen",
    "toggle-menu",
    "toggle-muted",
    "toggle-overscroll",
    "toggle-popover",
    "track",
    "true",
    "uninitialized",
    "unsafe-url",
    "until-found",
    "upper-alpha",
    "upper-roman",
    "url",
    "use-credentials",
    "visible",
];

// cpp: foundation/style_values/style/keywords.cc:22
static NAMES: LazyLock<[AtomicString; kNamesCount]> =
    LazyLock::new(|| std::array::from_fn(|index| AtomicString::from_str(SPELLINGS[index])));

macro_rules! keyword {
    ($name:ident, $index:expr) => {
        pub static $name: LazyLock<AtomicString> = LazyLock::new(|| NAMES[$index].clone());
    };
}

// cpp: foundation/style_values/style/keywords.h:20-156
// cpp: foundation/style_values/style/keywords.cc:24-160
keyword!(kInternalPrintFooter, 0);
keyword!(kInternalPrintHeader, 1);
keyword!(kInternalPrintPageNumber, 2);
keyword!(kArrowDown, 3);
keyword!(kArrowLeft, 4);
keyword!(kArrowRight, 5);
keyword!(kArrowUp, 6);
keyword!(kEnd, 7);
keyword!(kCapitalEnter, 8);
keyword!(kEscape, 9);
keyword!(kHome, 10);
keyword!(kPageDown, 11);
keyword!(kPageUp, 12);
keyword!(kTab, 13);
keyword!(kAnonymous, 14);
keyword!(kAny, 15);
keyword!(kApplicationXml, 16);
keyword!(kAsync, 17);
keyword!(kAuto, 18);
keyword!(kButton, 19);
keyword!(kCamera, 20);
keyword!(kCircle, 21);
keyword!(kClose, 22);
keyword!(kClosed, 23);
keyword!(kCloserequest, 24);
keyword!(kCol, 25);
keyword!(kColgroup, 26);
keyword!(kColorScheme, 27);
keyword!(kDecimal, 28);
keyword!(kDefault, 29);
keyword!(kDisc, 30);
keyword!(kDisclosureClosed, 31);
keyword!(kDisclosureOpen, 32);
keyword!(kDisplayP3, 33);
keyword!(kDone, 34);
keyword!(kEager, 35);
keyword!(kEmail, 36);
keyword!(kEnter, 37);
keyword!(kExitFullscreen, 38);
keyword!(kFalse, 39);
keyword!(kFetch, 40);
keyword!(kFont, 41);
keyword!(kGo, 42);
keyword!(kHidden, 43);
keyword!(kHide, 44);
keyword!(kHideOverscroll, 45);
keyword!(kHidePopover, 46);
keyword!(kHigh, 47);
keyword!(kHint, 48);
keyword!(kImage, 49);
keyword!(kInherit, 50);
keyword!(kInvisible, 51);
keyword!(kLazy, 52);
keyword!(kLimitedSrgb, 53);
keyword!(kLow, 54);
keyword!(kLowerAlpha, 55);
keyword!(kLowerRoman, 56);
keyword!(kManual, 57);
keyword!(kMicrophone, 58);
keyword!(kModule, 59);
keyword!(kMultiple, 60);
keyword!(kNamed, 61);
keyword!(kNever, 62);
keyword!(kNext, 63);
keyword!(kNoReferrer, 64);
keyword!(kNoReferrerWhenDowngrade, 65);
keyword!(kNodownload, 66);
keyword!(kNofullscreen, 67);
keyword!(kNone, 68);
keyword!(kNoplaybackrate, 69);
keyword!(kNoremoteplayback, 70);
keyword!(kNumeric, 71);
keyword!(kOff, 72);
keyword!(kOn, 73);
keyword!(kOpen, 74);
keyword!(kOrigin, 75);
keyword!(kOriginWhenCrossOrigin, 76);
keyword!(kPageBlockEnd, 77);
keyword!(kPageBlockStart, 78);
keyword!(kPage_Down, 79);
keyword!(kPageInlineEnd, 80);
keyword!(kPageInlineStart, 81);
keyword!(kPageLeft, 82);
keyword!(kPageRight, 83);
keyword!(kPage_Up, 84);
keyword!(kPause, 85);
keyword!(kPlaintextOnly, 86);
keyword!(kPlay, 87);
keyword!(kPlayPause, 88);
keyword!(kPresentation, 89);
keyword!(kPrevious, 90);
keyword!(kRender, 91);
keyword!(kRequestClose, 92);
keyword!(kRequestFullscreen, 93);
keyword!(kReset, 94);
keyword!(kResponsiveEmbeddedSizing, 95);
keyword!(kRow, 96);
keyword!(kRowgroup, 97);
keyword!(kSameOrigin, 98);
keyword!(kScript, 99);
keyword!(kSearch, 100);
keyword!(kSend, 101);
keyword!(kShow, 102);
keyword!(kShowModal, 103);
keyword!(kShowOverscroll, 104);
keyword!(kShowPicker, 105);
keyword!(kShowPopover, 106);
keyword!(kSingle, 107);
keyword!(kSquare, 108);
keyword!(kStatic, 109);
keyword!(kStepDown, 110);
keyword!(kStepUp, 111);
keyword!(kStrictOrigin, 112);
keyword!(kStrictOriginWhenCrossOrigin, 113);
keyword!(kStyle, 114);
keyword!(kSubmit, 115);
keyword!(kSync, 116);
keyword!(kTel, 117);
keyword!(kText, 118);
keyword!(kTextCss, 119);
keyword!(kTextHtml, 120);
keyword!(kToggle, 121);
keyword!(kToggleFullscreen, 122);
keyword!(kToggleMenu, 123);
keyword!(kToggleMuted, 124);
keyword!(kToggleOverscroll, 125);
keyword!(kTogglePopover, 126);
keyword!(kTrack, 127);
keyword!(kTrue, 128);
keyword!(kUninitialized, 129);
keyword!(kUnsafeUrl, 130);
keyword!(kUntilFound, 131);
keyword!(kUpperAlpha, 132);
keyword!(kUpperRoman, 133);
keyword!(kUrl, 134);
keyword!(kUseCredentials, 135);
keyword!(kVisible, 136);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_names_are_interned_in_source_order() {
        Init();
        assert_eq!(NAMES.len(), kNamesCount);
        assert_eq!(kDisc.Utf8(), "disc");
        assert_eq!(kInternalPrintFooter.Utf8(), "-internal-print-footer");
        assert_eq!(kVisible.Utf8(), "visible");
        assert_eq!(*kDisc, AtomicString::from_str("disc"));
        for (name, spelling) in NAMES.iter().zip(SPELLINGS) {
            assert_eq!(name.Utf8(), spelling);
        }
    }
}
