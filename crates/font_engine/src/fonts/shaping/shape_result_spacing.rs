#![allow(non_snake_case)]

use crate::fonts::font_description::FontDescription;
// `Character` is supplied by font_engine/text/native/character.h/.cc, pending translation.
use crate::text::native::character::Character;
use crate::text::native::justification_opportunity::JustificationContext;
use foundation::blink_geometry::geometry::{InlineLayoutUnit, TextRunLayoutUnit};
use foundation::{String, StringView, TextDirection, TextJustify};

// cpp: font_engine/fonts/shaping/shape_result_spacing.h:21-129
pub struct ShapeResultSpacing {
    text_: String,
    letter_spacing_: TextRunLayoutUnit,
    word_spacing_: TextRunLayoutUnit,
    expansion_: InlineLayoutUnit,
    expansion_per_opportunity_: TextRunLayoutUnit,
    expansion_opportunity_count_: u32,
    justification_context_: JustificationContext,
    has_spacing_: bool,
    is_letter_spacing_applied_: bool,
    is_word_spacing_applied_: bool,
    normalize_space_: bool,
    allow_tabs_: bool,
    allow_word_spacing_anywhere_: bool,
}

impl ShapeResultSpacing {
    // cpp: font_engine/fonts/shaping/shape_result_spacing.h:25-44
    pub fn new(text: &String, allow_word_spacing_anywhere: bool) -> Self {
        Self {
            text_: text.clone(),
            letter_spacing_: TextRunLayoutUnit::new(),
            word_spacing_: TextRunLayoutUnit::new(),
            expansion_: InlineLayoutUnit::new(),
            expansion_per_opportunity_: TextRunLayoutUnit::new(),
            expansion_opportunity_count_: 0,
            justification_context_: JustificationContext::default(),
            has_spacing_: false,
            is_letter_spacing_applied_: false,
            is_word_spacing_applied_: false,
            normalize_space_: false,
            allow_tabs_: false,
            allow_word_spacing_anywhere_: allow_word_spacing_anywhere,
        }
    }

    pub fn Text(&self) -> &String {
        &self.text_
    }
    pub fn LetterSpacing(&self) -> TextRunLayoutUnit {
        if self.has_spacing_ {
            self.letter_spacing_
        } else {
            TextRunLayoutUnit::new()
        }
    }
    pub fn WordSpacing(&self) -> TextRunLayoutUnit {
        if self.has_spacing_ {
            self.word_spacing_
        } else {
            TextRunLayoutUnit::new()
        }
    }
    pub fn HasSpacing(&self) -> bool {
        self.has_spacing_
    }
    pub fn IsLetterSpacingAppliedForTesting(&self) -> bool {
        self.is_letter_spacing_applied_
    }
    pub fn IsWordSpacingAppliedForTesting(&self) -> bool {
        self.is_word_spacing_applied_
    }
    pub fn HasExpansion(&self) -> bool {
        self.expansion_opportunity_count_ != 0
    }
    pub fn ExpansionOppotunityCount(&self) -> u32 {
        self.expansion_opportunity_count_
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing.cc:11-38
    pub fn SetSpacingFromDescription(&mut self, description: &FontDescription) -> bool {
        self.SetSpacing(
            TextRunLayoutUnit::FromFloatRound(description.LetterSpacing()),
            TextRunLayoutUnit::FromFloatRound(description.WordSpacing()),
        )
    }

    pub fn SetSpacing(&mut self, letter: TextRunLayoutUnit, word: TextRunLayoutUnit) -> bool {
        if letter.RawValue() == 0 && word.RawValue() == 0 {
            self.has_spacing_ = false;
            return false;
        }
        self.letter_spacing_ = letter;
        self.word_spacing_ = word;
        assert!(!self.normalize_space_);
        self.allow_tabs_ = true;
        self.has_spacing_ = true;
        true
    }

    pub fn SetSpacingWithNormalization(
        &mut self,
        description: &FontDescription,
        normalize_space: bool,
    ) {
        if self.SetSpacingFromDescription(description) {
            self.normalize_space_ = normalize_space;
            self.allow_tabs_ = false;
        }
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing.cc:75-86
    fn NextExpansion(&mut self) -> TextRunLayoutUnit {
        assert!(self.expansion_opportunity_count_ != 0);
        self.justification_context_.SetAfterOpportunity(true);
        self.expansion_opportunity_count_ -= 1;
        if self.expansion_opportunity_count_ == 0 {
            let remaining = self.expansion_.To::<16, i32>();
            self.expansion_ = InlineLayoutUnit::new();
            return remaining;
        }
        self.expansion_ -= self.expansion_per_opportunity_.To::<16, i64>();
        self.expansion_per_opportunity_
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing.cc:88-111
    pub fn ComputeSpacing(&mut self, index: u32, cursive: bool) -> TextRunLayoutUnit {
        assert!(self.has_spacing_);
        let mut character =
            i32::from(self.text_.Span16().expect("non-null spacing text")[index as usize]);
        let mut is_space = (Character::TreatAsSpace(character)
            || (self.normalize_space_ && Character::IsNormalizedCanvasSpaceCharacter(character)))
            && (character != i32::from(b'\t') || !self.allow_tabs_);
        if is_space && character != 0x00A0 {
            character = i32::from(b' ');
        }
        let mut spacing = TextRunLayoutUnit::new();
        let apply_letter = !cursive || is_space;
        if self.letter_spacing_.RawValue() != 0
            && !Character::TreatAsZeroWidthSpace(character)
            && apply_letter
        {
            spacing += self.letter_spacing_;
            self.is_letter_spacing_applied_ = true;
        }
        if is_space && (self.allow_word_spacing_anywhere_ || index != 0 || character == 0x00A0) {
            spacing += self.word_spacing_;
            self.is_word_spacing_applied_ = true;
        }
        spacing
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing.cc:113-140
    pub fn ComputeExpansion(
        &mut self,
        method: TextJustify,
        index: u32,
        _cursive: bool,
    ) -> (TextRunLayoutUnit, TextRunLayoutUnit) {
        if !self.HasExpansion() || index >= self.text_.length() {
            return (TextRunLayoutUnit::new(), TextRunLayoutUnit::new());
        }
        let chars = self.text_.Span16().expect("non-null spacing text");
        let character = Self::CodePointAt(chars, index as usize);
        let (before, after) = if chars.iter().all(|ch| *ch <= 0x00FF) {
            self.justification_context_
                .CheckOpportunity8(method, character as u8)
        } else {
            self.justification_context_
                .CheckOpportunity16(method, character)
        };
        self.FinalizeComputeExpansion(before, after)
    }

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

    pub fn ComputeExpansionForCharacter(
        &mut self,
        method: TextJustify,
        character: u16,
    ) -> (TextRunLayoutUnit, TextRunLayoutUnit) {
        if !self.HasExpansion() {
            return (TextRunLayoutUnit::new(), TextRunLayoutUnit::new());
        }
        let (before, after) = self
            .justification_context_
            .CheckOpportunity16(method, u32::from(character));
        self.FinalizeComputeExpansion(before, after)
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing.cc:138-148
    fn FinalizeComputeExpansion(
        &mut self,
        before: bool,
        after: bool,
    ) -> (TextRunLayoutUnit, TextRunLayoutUnit) {
        let mut leading = TextRunLayoutUnit::new();
        let mut trailing = TextRunLayoutUnit::new();
        if before {
            leading = self.NextExpansion();
            if !self.HasExpansion() {
                return (leading, trailing);
            }
        }
        if after {
            trailing = self.NextExpansion();
        }
        (leading, trailing)
    }
}

// The C++ destructor performs the expansion finalization. Rust's `Drop`
// keeps it tied to the same lexical lifetime.
// cpp: font_engine/fonts/shaping/shape_result_spacing.h:60-80
pub struct ExpansionSetup<'a> {
    spacing_: &'a mut ShapeResultSpacing,
    allows_trailing_expansion_: bool,
    justification_context_: JustificationContext,
}

impl<'a> ExpansionSetup<'a> {
    // cpp: font_engine/fonts/shaping/shape_result_spacing.cc:40-50
    pub fn new(
        expansion: InlineLayoutUnit,
        spacing: &'a mut ShapeResultSpacing,
        allows_leading: bool,
        allows_trailing: bool,
    ) -> Self {
        assert!(expansion > InlineLayoutUnit::new());
        let mut justification_context = JustificationContext::default();
        justification_context.SetAfterOpportunity(!allows_leading);
        spacing.expansion_ = expansion;
        spacing.expansion_opportunity_count_ = 0;
        spacing.justification_context_ = justification_context;
        Self {
            spacing_: spacing,
            allows_trailing_expansion_: allows_trailing,
            justification_context_: justification_context,
        }
    }

    pub fn Spacing(&mut self) -> &mut ShapeResultSpacing {
        self.spacing_
    }

    // cpp: font_engine/fonts/shaping/shape_result_spacing.cc:63-73
    pub fn CountOpportunities(
        &mut self,
        method: TextJustify,
        text: StringView,
        direction: TextDirection,
    ) {
        let chars = text.Span16();
        let count = if chars.iter().all(|ch| *ch <= 0x00FF) {
            let bytes: Vec<u8> = chars.iter().map(|ch| *ch as u8).collect();
            self.justification_context_
                .CountOpportunities8(method, &bytes, direction)
        } else {
            self.justification_context_
                .CountOpportunities16(method, chars, direction)
        };
        self.spacing_.expansion_opportunity_count_ += count;
    }

    pub fn CountOpportunityForCharacter(&mut self, method: TextJustify, character: u16) {
        self.spacing_.expansion_opportunity_count_ += self
            .justification_context_
            .CountOpportunity16(method, u32::from(character));
    }
}

// cpp: font_engine/fonts/shaping/shape_result_spacing.cc:52-61
impl Drop for ExpansionSetup<'_> {
    fn drop(&mut self) {
        if self.justification_context_.IsAfterOpportunity()
            && !self.allows_trailing_expansion_
            && self.spacing_.expansion_opportunity_count_ > 0
        {
            self.spacing_.expansion_opportunity_count_ -= 1;
        }
        if self.spacing_.expansion_opportunity_count_ != 0 {
            self.spacing_.expansion_per_opportunity_ = (self.spacing_.expansion_
                / self.spacing_.expansion_opportunity_count_)
                .To::<16, i32>();
        }
        let has_expansion = self.spacing_.HasExpansion();
        self.spacing_.has_spacing_ |= has_expansion;
    }
}
