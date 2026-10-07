#![allow(non_snake_case)]

use super::character_break_iterator::{kTextBreakDone, CharacterBreakIterator};
// `Character` is a same-package dependency whose source unit is still pending.
use super::character::Character;
use foundation::{String, TextDirection, TextJustify};

// cpp: font_engine/text/native/justification_opportunity.h:63-68
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Type {
    kNormal,
    kAtomicInline,
    kCursive,
}

// cpp: font_engine/text/native/justification_opportunity.h:23-83
#[derive(Clone, Copy)]
pub struct JustificationContext {
    previous_type_: Type,
    is_after_opportunity_: bool,
}

impl Default for JustificationContext {
    fn default() -> Self {
        Self {
            previous_type_: Type::kNormal,
            is_after_opportunity_: true,
        }
    }
}

impl JustificationContext {
    // cpp: font_engine/text/native/justification_opportunity.h:26-27
    pub fn IsAfterOpportunity(&self) -> bool {
        self.is_after_opportunity_
    }

    pub fn SetAfterOpportunity(&mut self, flag: bool) {
        self.is_after_opportunity_ = flag;
    }

    // cpp: font_engine/text/native/justification_opportunity.cc:14-25
    fn TypeToString(kind: Type) -> &'static str {
        match kind {
            Type::kNormal => "kNormal",
            Type::kAtomicInline => "kAtomicInline",
            Type::kCursive => "kCursive",
        }
    }

    pub fn ToString(&self) -> String {
        let value = format!(
            "JustificationContext {{previous_type_:{}, is_after_opportunity_:{}}}",
            Self::TypeToString(self.previous_type_),
            self.is_after_opportunity_
        );
        String::FromUtf8(value.as_bytes())
    }

    // C++ instantiates this template with LChar and UChar. The const generic
    // preserves its 8-bit versus 16-bit branch without copying either body.
    // cpp: font_engine/text/native/justification_opportunity.cc:27-88
    fn CheckOpportunity<const IS_LATIN: bool>(
        &mut self,
        method: TextJustify,
        ch: u32,
    ) -> (bool, bool) {
        if Character::IsDefaultIgnorable(ch as i32) {
            return (false, false);
        }
        let mut kind = Type::kNormal;
        if !IS_LATIN {
            if ch == 0xFFFC {
                kind = Type::kAtomicInline;
            } else if Character::IsCursiveScript(ch as i32) {
                kind = Type::kCursive;
            }
        }
        let previous_type = self.previous_type_;
        self.previous_type_ = kind;
        match method {
            TextJustify::kNone => {
                self.is_after_opportunity_ = false;
                return (false, false);
            }
            TextJustify::kInterCharacter => {
                if kind != Type::kNormal {
                    let before = !self.is_after_opportunity_ && previous_type != kind;
                    self.is_after_opportunity_ = false;
                    return (before, false);
                }
                let before = !self.is_after_opportunity_;
                self.is_after_opportunity_ = true;
                return (before, true);
            }
            TextJustify::kInterWord => {
                if Character::TreatAsSpace(ch as i32) {
                    self.is_after_opportunity_ = true;
                    return (false, true);
                }
                self.is_after_opportunity_ = false;
                return (false, false);
            }
            TextJustify::kAuto => {}
        }
        let space = Character::TreatAsSpace(ch as i32);
        if space {
            self.is_after_opportunity_ = true;
            return (false, true);
        }
        if IS_LATIN {
            self.is_after_opportunity_ = false;
            return (false, false);
        }
        if !Character::IsCjkIdeographOrSymbol(ch as i32) {
            self.is_after_opportunity_ = false;
            return (false, false);
        }
        let before = !self.is_after_opportunity_;
        self.is_after_opportunity_ = true;
        (before, true)
    }

    // cpp: font_engine/text/native/justification_opportunity.cc:90-102
    pub fn CheckOpportunity8(&mut self, method: TextJustify, ch: u8) -> (bool, bool) {
        self.CheckOpportunity::<true>(method, u32::from(ch))
    }

    pub fn CheckOpportunity16(&mut self, method: TextJustify, ch: u32) -> (bool, bool) {
        self.CheckOpportunity::<false>(method, ch)
    }

    // cpp: font_engine/text/native/justification_opportunity.h:47-56
    pub fn CountOpportunity8(&mut self, method: TextJustify, ch: u8) -> u32 {
        let (before, after) = self.CheckOpportunity8(method, ch);
        u32::from(before) + u32::from(after)
    }

    pub fn CountOpportunity16(&mut self, method: TextJustify, ch: u32) -> u32 {
        let (before, after) = self.CheckOpportunity16(method, ch);
        u32::from(before) + u32::from(after)
    }

    // cpp: font_engine/text/native/justification_opportunity.cc:104-119
    pub fn CountOpportunities8(
        &mut self,
        method: TextJustify,
        chars: &[u8],
        direction: TextDirection,
    ) -> u32 {
        let mut count = 0;
        if direction == TextDirection::kLtr {
            for &ch in chars {
                count += self.CountOpportunity8(method, ch);
            }
        } else {
            for &ch in chars.iter().rev() {
                count += self.CountOpportunity8(method, ch);
            }
        }
        count
    }

    // `CodePointAt` from the source text boundary is expressed directly as
    // UTF-16 code-unit decoding, retaining lone surrogates as code points.
    fn CodePointAt(chars: &[u16], index: usize) -> u32 {
        let lead = u32::from(chars[index]);
        if (0xD800..=0xDBFF).contains(&lead) && index + 1 < chars.len() {
            let trail = u32::from(chars[index + 1]);
            if (0xDC00..=0xDFFF).contains(&trail) {
                return 0x10000 + ((lead - 0xD800) << 10) + (trail - 0xDC00);
            }
        }
        lead
    }

    // cpp: font_engine/text/native/justification_opportunity.cc:117-137
    pub fn CountOpportunities16(
        &mut self,
        method: TextJustify,
        chars: &[u16],
        direction: TextDirection,
    ) -> u32 {
        if chars.is_empty() {
            return 0;
        }
        let mut count = 0;
        let mut iterator = CharacterBreakIterator::from_utf16(chars);
        if direction == TextDirection::kLtr {
            let mut i = 0;
            while (i as usize) < chars.len() {
                count += self.CountOpportunity16(method, Self::CodePointAt(chars, i as usize));
                i = iterator.Next();
            }
        } else {
            let mut i = iterator.Preceding(i32::try_from(chars.len()).expect("text exceeds i32"));
            while i != kTextBreakDone {
                count += self.CountOpportunity16(method, Self::CodePointAt(chars, i as usize));
                i = iterator.Preceding(i);
            }
        }
        count
    }
}
