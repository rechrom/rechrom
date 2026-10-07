#![allow(non_snake_case)]

// cpp: foundation/blink_base/wtf/text/text_position.h:42-69
// Only OrdinalNumber is connected here for HTML source positions; the rest of
// text_position.h remains outside this dependency connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrdinalNumber {
    zero_based_value_: i32,
}

impl OrdinalNumber {
    // cpp: foundation/blink_base/wtf/text/text_position.h:46-48
    pub fn FromZeroBasedInt(zero_based_int: i32) -> Self {
        Self {
            zero_based_value_: zero_based_int,
        }
    }

    // cpp: foundation/blink_base/wtf/text/text_position.h:49-51
    pub fn FromOneBasedInt(one_based_int: i32) -> Self {
        Self {
            zero_based_value_: one_based_int - 1,
        }
    }

    // cpp: foundation/blink_base/wtf/text/text_position.h:56-57
    pub fn ZeroBasedInt(self) -> i32 {
        self.zero_based_value_
    }

    pub fn OneBasedInt(self) -> i32 {
        self.zero_based_value_ + 1
    }

    // cpp: foundation/blink_base/wtf/text/text_position.h:63-64
    pub fn First() -> Self {
        Self::FromZeroBasedInt(0)
    }

    pub fn BeforeFirst() -> Self {
        Self::FromZeroBasedInt(-1)
    }
}
