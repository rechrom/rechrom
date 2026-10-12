// C++: src/foundation/text/native/han_kerning_char_type.h

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HanKerningCharType {
    kOther,
    kOpen,
    kClose,
    kMiddle,
    kOpenNarrow,
    kCloseNarrow,
    kDot,
    kColon,
    kSemicolon,
    kOpenQuote,
    kCloseQuote,
    kInvalid,
}
