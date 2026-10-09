// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/platform/wtf/text/number_parsing_options.h:16-78
// This is the WTF pure-function dependency used by parser/css_parser_idioms.cc.

// Copyable and immutable object representing number parsing flags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NumberParsingOptions {
    accept_trailing_garbage_: bool,
    accept_leading_plus_: bool,
    accept_leading_trailing_whitespace_: bool,
    accept_minus_zero_for_unsigned_: bool,
}

impl NumberParsingOptions {
    // cpp: third_party/blink/renderer/platform/wtf/text/number_parsing_options.h:21-28
    pub const fn Strict() -> Self {
        Self::new().SetAcceptLeadingPlus().SetAcceptWhiteSpace()
    }

    pub const fn Loose() -> Self {
        Self::Strict().SetAcceptTrailingGarbage()
    }

    // cpp: third_party/blink/renderer/platform/wtf/text/number_parsing_options.h:30-35
    pub const fn new() -> Self {
        Self {
            accept_trailing_garbage_: false,
            accept_leading_plus_: false,
            accept_leading_trailing_whitespace_: false,
            accept_minus_zero_for_unsigned_: false,
        }
    }

    // cpp: third_party/blink/renderer/platform/wtf/text/number_parsing_options.h:37-42
    pub const fn SetAcceptTrailingGarbage(self) -> Self {
        Self {
            accept_trailing_garbage_: true,
            ..self
        }
    }

    // cpp: third_party/blink/renderer/platform/wtf/text/number_parsing_options.h:44-49
    pub const fn SetAcceptLeadingPlus(self) -> Self {
        Self {
            accept_leading_plus_: true,
            ..self
        }
    }

    // cpp: third_party/blink/renderer/platform/wtf/text/number_parsing_options.h:51-56
    pub const fn SetAcceptWhiteSpace(self) -> Self {
        Self {
            accept_leading_trailing_whitespace_: true,
            ..self
        }
    }

    // cpp: third_party/blink/renderer/platform/wtf/text/number_parsing_options.h:58-64
    pub const fn SetAcceptMinusZeroForUnsigned(self) -> Self {
        Self {
            accept_minus_zero_for_unsigned_: true,
            ..self
        }
    }

    // cpp: third_party/blink/renderer/platform/wtf/text/number_parsing_options.h:66-71
    pub const fn AcceptTrailingGarbage(self) -> bool {
        self.accept_trailing_garbage_
    }
    pub const fn AcceptLeadingPlus(self) -> bool {
        self.accept_leading_plus_
    }
    pub const fn AcceptWhitespace(self) -> bool {
        self.accept_leading_trailing_whitespace_
    }
    pub const fn AcceptMinusZeroForUnsigned(self) -> bool {
        self.accept_minus_zero_for_unsigned_
    }
}
