// Copyright 2018, 2026 The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
use crate::css_markup::SerializeIdentifierTo;
use foundation::String;

// cpp: third_party/blink/renderer/core/css/css_syntax_component.h:14-32
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CSSSyntaxType {
    kTokenStream,
    kIdent,
    kLength,
    kNumber,
    kPercentage,
    kLengthPercentage,
    kColor,
    kImage,
    kUrl,
    kInteger,
    kAngle,
    kTime,
    kResolution,
    kTransformFunction,
    kTransformList,
    kCustomIdent,
    kString,
}
impl CSSSyntaxType {
    // cpp: third_party/blink/renderer/core/css/css_syntax_component.h:33-71
    pub fn ToString(self) -> &'static str {
        match self {
            Self::kTokenStream => "*",
            Self::kLength => "<length>",
            Self::kNumber => "<number>",
            Self::kPercentage => "<percentage>",
            Self::kLengthPercentage => "<length-percentage>",
            Self::kColor => "<color>",
            Self::kImage => "<image>",
            Self::kUrl => "<url>",
            Self::kInteger => "<integer>",
            Self::kAngle => "<angle>",
            Self::kTime => "<time>",
            Self::kResolution => "<resolution>",
            Self::kTransformFunction => "<transform-function>",
            Self::kTransformList => "<transform-list>",
            Self::kCustomIdent => "<custom-ident>",
            Self::kString => "<string>",
            Self::kIdent => unreachable!("<ident> type must be serialized separately"),
        }
    }
}

// cpp: third_party/blink/renderer/core/css/css_syntax_component.h:73-83
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CSSSyntaxRepeat {
    kNone,
    kSpaceSeparated,
    kCommaSeparated,
}
impl CSSSyntaxRepeat {
    pub const fn ToString(self) -> &'static str {
        match self {
            Self::kNone => "",
            Self::kSpaceSeparated => "+",
            Self::kCommaSeparated => "#",
        }
    }
}

// cpp: third_party/blink/renderer/core/css/css_syntax_component.h:85-113
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSSyntaxComponent {
    type_: CSSSyntaxType,
    string_: String,
    repeat_: CSSSyntaxRepeat,
}
impl CSSSyntaxComponent {
    // cpp: third_party/blink/renderer/core/css/css_syntax_component.h:89-96
    pub fn new(type_: CSSSyntaxType, string: &String, repeat_: CSSSyntaxRepeat) -> Self {
        Self {
            type_,
            string_: string.clone(),
            repeat_,
        }
    }
    // cpp: third_party/blink/renderer/core/css/css_syntax_component.h:98-106
    pub fn GetType(&self) -> CSSSyntaxType {
        self.type_
    }
    pub fn GetString(&self) -> &String {
        &self.string_
    }
    pub fn GetRepeat(&self) -> CSSSyntaxRepeat {
        self.repeat_
    }
    pub fn IsRepeatable(&self) -> bool {
        self.repeat_ != CSSSyntaxRepeat::kNone
    }
    pub fn IsInteger(&self) -> bool {
        self.type_ == CSSSyntaxType::kInteger
    }
    pub fn Separator(&self) -> char {
        debug_assert!(self.IsRepeatable());
        if self.repeat_ == CSSSyntaxRepeat::kSpaceSeparated {
            ' '
        } else {
            ','
        }
    }
    // cpp: third_party/blink/renderer/core/css/css_syntax_component.cc:12-21
    pub fn ToString(&self) -> String {
        let mut units = Vec::new();
        if self.type_ == CSSSyntaxType::kIdent {
            SerializeIdentifierTo(&self.string_, &mut units, false);
        } else {
            units.extend(self.type_.ToString().encode_utf16());
        }
        units.extend(self.repeat_.ToString().encode_utf16());
        String::from_utf16(&units)
    }
}

#[cfg(test)]
mod tests {
    use crate::css_syntax_component;
    #[test]
    fn syntax_ident_utf16_escaping() {
        use css_syntax_component::*;
        use foundation::String;
        for (s, expected) in [
            ("1foo", "\\31 foo"),
            ("-1foo", "-\\31 foo"),
            ("-", "\\-"),
            ("a b", "a\\ b"),
            ("a\u{0}", "a\u{fffd}"),
            ("😀", "😀"),
        ] {
            let c = CSSSyntaxComponent::new(
                CSSSyntaxType::kIdent,
                &String::FromUtf8(s.as_bytes()),
                CSSSyntaxRepeat::kNone,
            );
            assert_eq!(c.ToString().as_str(), expected);
        }
        let s = String::from_utf16(&[0xd800, b'!' as u16, 0xdc00]);
        let c =
            CSSSyntaxComponent::new(CSSSyntaxType::kIdent, &s, CSSSyntaxRepeat::kCommaSeparated);
        assert_eq!(
            c.ToString().Span16(),
            Some(&[0xd800, b'\\' as u16, b'!' as u16, 0xdc00, b'#' as u16][..])
        );
        let c = CSSSyntaxComponent::new(
            CSSSyntaxType::kInteger,
            &String::new(),
            CSSSyntaxRepeat::kSpaceSeparated,
        );
        assert!(c.IsInteger());
        assert_eq!(c.Separator(), ' ');
        assert_eq!(c.ToString().as_str(), "<integer>+");
    }
}
