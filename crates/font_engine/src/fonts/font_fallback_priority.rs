// cpp: font_engine/fonts/font_fallback_priority.h:14-31
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FontFallbackPriority {
    kText,
    kEmojiText,
    kEmojiTextWithVS,
    kEmojiEmoji,
    kEmojiEmojiWithVS,
    kInvalid,
}

impl FontFallbackPriority {
    // C++ enum alias kMaxEnumValue = kEmojiEmojiWithVS.
    pub const kMaxEnumValue: Self = Self::kEmojiEmojiWithVS;
}

// cpp: font_engine/fonts/font_fallback_priority.h:33-38
pub fn IsNonTextFallbackPriority(fallback_priority: FontFallbackPriority) -> bool {
    fallback_priority == FontFallbackPriority::kEmojiText
        || fallback_priority == FontFallbackPriority::kEmojiEmoji
        || fallback_priority == FontFallbackPriority::kEmojiTextWithVS
        || fallback_priority == FontFallbackPriority::kEmojiEmojiWithVS
}

// cpp: font_engine/fonts/font_fallback_priority.h:40-43
pub fn HasVSFallbackPriority(fallback_priority: FontFallbackPriority) -> bool {
    fallback_priority == FontFallbackPriority::kEmojiTextWithVS
        || fallback_priority == FontFallbackPriority::kEmojiEmojiWithVS
}

// cpp: font_engine/fonts/font_fallback_priority.h:45-48
pub fn IsEmojiPresentationEmoji(fallback_priority: FontFallbackPriority) -> bool {
    fallback_priority == FontFallbackPriority::kEmojiEmoji
        || fallback_priority == FontFallbackPriority::kEmojiEmojiWithVS
}

// cpp: font_engine/fonts/font_fallback_priority.h:50-53
pub fn IsTextPresentationEmoji(fallback_priority: FontFallbackPriority) -> bool {
    fallback_priority == FontFallbackPriority::kEmojiText
        || fallback_priority == FontFallbackPriority::kEmojiTextWithVS
}
