// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: css_syntax_string_parser.h:25-64/.cc:20-184.
#![allow(non_snake_case)]
use crate::{
    css_syntax_component::{CSSSyntaxComponent, CSSSyntaxRepeat, CSSSyntaxType},
    css_syntax_definition::CSSSyntaxDefinition,
    parser::{
        css_parser_idioms::{
            ConsumeName, IsNameCodePoint, IsNameStartCodePoint, NextCharsAreIdentifier,
        },
        css_property_parser::CssValueKeywordID,
        css_tokenizer_input_stream::CSSTokenizerInputStream,
    },
    production_css_value,
};
use foundation::{CSSValueID, String, StringView};
pub(crate) fn SyntaxType(name: &StringView) -> Option<CSSSyntaxType> {
    use CSSSyntaxType::*;
    Some(match name.ToString().Utf8().as_str() {
        "length" => kLength,
        "number" => kNumber,
        "percentage" => kPercentage,
        "length-percentage" => kLengthPercentage,
        "color" => kColor,
        "image" => kImage,
        "url" => kUrl,
        "integer" => kInteger,
        "angle" => kAngle,
        "time" => kTime,
        "resolution" => kResolution,
        "transform-function" => kTransformFunction,
        "transform-list" => kTransformList,
        "custom-ident" => kCustomIdent,
        "string" => kString,
        _ => return None,
    })
}
pub(crate) fn ValidSyntaxIdent(ident: &String) -> bool {
    let id = CssValueKeywordID(&StringView::from(ident));
    production_css_value::wide(id).is_none() && id != CSSValueID::kDefault
}
pub struct CSSSyntaxStringParser {
    input_: CSSTokenizerInputStream,
}
impl CSSSyntaxStringParser {
    pub fn new(text: &String) -> Self {
        Self {
            input_: CSSTokenizerInputStream::new(StringView::from(text)),
        }
    }
    pub fn Parse(&mut self) -> Option<CSSSyntaxDefinition> {
        self.input_.AdvanceUntilNonWhitespace();
        if self.input_.AtEnd() {
            return None;
        }
        if self.input_.NextInputChar() == 42 {
            self.input_.Advance();
            self.input_.AdvanceUntilNonWhitespace();
            return self
                .input_
                .AtEnd()
                .then(CSSSyntaxDefinition::CreateUniversal);
        }
        let mut components = Vec::new();
        loop {
            self.ConsumeSyntaxComponent(&mut components)?;
            self.input_.AdvanceUntilNonWhitespace();
            let cc = self.input_.NextInputChar();
            self.input_.Advance();
            if cc == 0 {
                break;
            }
            if cc != 124 {
                return None;
            }
        }
        Some(CSSSyntaxDefinition::new(components))
    }
    fn ConsumeSyntaxComponent(&mut self, out: &mut Vec<CSSSyntaxComponent>) -> Option<()> {
        self.input_.AdvanceUntilNonWhitespace();
        let cc = self.input_.NextInputChar();
        self.input_.Advance();
        let (ty, ident) = if cc == 60 {
            (self.ConsumeDataTypeName()?, String::new())
        } else if (IsNameStartCodePoint(cc) || cc == 92) && NextCharsAreIdentifier(cc, &self.input_)
        {
            self.input_.PushBack(cc);
            let name = ConsumeName(&mut self.input_);
            if !ValidSyntaxIdent(&name) {
                return None;
            }
            (CSSSyntaxType::kIdent, name)
        } else {
            return None;
        };
        let repeat = if ty == CSSSyntaxType::kTransformList {
            CSSSyntaxRepeat::kNone
        } else {
            self.ConsumeRepeatIfPresent()
        };
        out.push(CSSSyntaxComponent::new(ty, &ident, repeat));
        Some(())
    }
    fn ConsumeRepeatIfPresent(&mut self) -> CSSSyntaxRepeat {
        match self.input_.NextInputChar() {
            43 => {
                self.input_.Advance();
                CSSSyntaxRepeat::kSpaceSeparated
            }
            35 => {
                self.input_.Advance();
                CSSSyntaxRepeat::kCommaSeparated
            }
            _ => CSSSyntaxRepeat::kNone,
        }
    }
    fn ConsumeDataTypeName(&mut self) -> Option<CSSSyntaxType> {
        let mut size = 0;
        loop {
            let cc = self.input_.PeekWithoutReplacement(size);
            if IsNameCodePoint(cc) {
                size += 1;
                continue;
            }
            if cc != 62 {
                return None;
            }
            let start = self.input_.Offset();
            self.input_.AdvanceBy(size + 1);
            return SyntaxType(&self.input_.RangeAt(start, size));
        }
    }
}
