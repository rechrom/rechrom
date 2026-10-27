#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
// Copyright 2025 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: third_party/blink/renderer/core/css/kleene_value.h:13-17
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum KleeneValue {
    kTrue,
    kFalse,
    kUnknown,
}

// cpp: third_party/blink/renderer/core/css/kleene_value.h:19-28
pub const fn KleeneNot(a: KleeneValue) -> KleeneValue {
    match a {
        KleeneValue::kTrue => KleeneValue::kFalse,
        KleeneValue::kFalse => KleeneValue::kTrue,
        KleeneValue::kUnknown => KleeneValue::kUnknown,
    }
}

// cpp: third_party/blink/renderer/core/css/kleene_value.h:30-40
pub const fn KleeneOr(a: KleeneValue, b: KleeneValue) -> KleeneValue {
    match a {
        KleeneValue::kTrue => KleeneValue::kTrue,
        KleeneValue::kFalse => b,
        KleeneValue::kUnknown => match b {
            KleeneValue::kTrue => KleeneValue::kTrue,
            _ => KleeneValue::kUnknown,
        },
    }
}

// cpp: third_party/blink/renderer/core/css/kleene_value.h:42-52
pub const fn KleeneAnd(a: KleeneValue, b: KleeneValue) -> KleeneValue {
    match a {
        KleeneValue::kTrue => b,
        KleeneValue::kFalse => KleeneValue::kFalse,
        KleeneValue::kUnknown => match b {
            KleeneValue::kFalse => KleeneValue::kFalse,
            _ => KleeneValue::kUnknown,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::kleene_value;
    #[test]
    fn truth_tables() {
        use kleene_value::{KleeneValue::*, *};
        let values = [kTrue, kFalse, kUnknown];
        let or = [
            [kTrue, kTrue, kTrue],
            [kTrue, kFalse, kUnknown],
            [kTrue, kUnknown, kUnknown],
        ];
        let and = [
            [kTrue, kFalse, kUnknown],
            [kFalse, kFalse, kFalse],
            [kUnknown, kFalse, kUnknown],
        ];
        for (i, a) in values.iter().enumerate() {
            assert_eq!(KleeneNot(KleeneNot(*a)), *a);
            for (j, b) in values.iter().enumerate() {
                assert_eq!(KleeneOr(*a, *b), or[i][j]);
                assert_eq!(KleeneAnd(*a, *b), and[i][j]);
            }
        }
    }
}
