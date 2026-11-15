// C++: foundation/blink_base/wtf/text/math_transform.h/.cc
// Character constants come from blink_base/wtf/text/character_names.h.

#[allow(non_snake_case)]
pub fn ItalicMathVariant(code_point: u32) -> u32 {
    // cpp: foundation/blink_base/wtf/text/math_transform.cc:44-59
    match code_point {
        0x03A2 | 0x03DC | 0x03DD => return code_point,
        0x0131 => return 0x1D6A4,
        0x0237 => return 0x1D6A5,
        _ => {}
    }

    // cpp: foundation/blink_base/wtf/text/math_transform.cc:69-135
    let (base_char, greekish) = if (b'A' as u32..=b'Z' as u32).contains(&code_point) {
        (code_point - b'A' as u32, false)
    } else if (b'a' as u32..=b'z' as u32).contains(&code_point) {
        (0x1D41A - 0x1D400 + code_point - b'a' as u32, false)
    } else if (0x0391..=0x03A9).contains(&code_point) {
        (code_point - 0x0391, true)
    } else if (0x03B1..=0x03C9).contains(&code_point) {
        (0x1D6C2 - 0x1D6A8 + code_point - 0x03B1, true)
    } else {
        let base = match code_point {
            0x03F4 => 0x1D6B9 - 0x1D6A8,
            0x2207 => 0x1D6C1 - 0x1D6A8,
            0x2202 => 0x1D6DB - 0x1D6A8,
            0x03F5 => 0x1D6DC - 0x1D6A8,
            0x03D1 => 0x1D6DD - 0x1D6A8,
            0x03F0 => 0x1D6DE - 0x1D6A8,
            0x03D5 => 0x1D6DF - 0x1D6A8,
            0x03F1 => 0x1D6E0 - 0x1D6A8,
            0x03D6 => 0x1D6E1 - 0x1D6A8,
            _ => return code_point,
        };
        (base, true)
    };

    if greekish {
        // cpp: foundation/blink_base/wtf/text/math_transform.cc:18-28
        base_char + 0x1D6A8 + (0x1D6E2 - 0x1D6A8)
    } else {
        // cpp: foundation/blink_base/wtf/text/math_transform.cc:30-41
        let transformed = base_char + 0x1D400 + (0x1D434 - 0x1D400);
        if transformed == 0x1D455 {
            0x210E
        } else {
            transformed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ItalicMathVariant;

    #[test]
    fn maps_latin_greek_and_exceptions() {
        assert_eq!(ItalicMathVariant('A' as u32), 0x1D434);
        assert_eq!(ItalicMathVariant('a' as u32), 0x1D44E);
        assert_eq!(ItalicMathVariant('h' as u32), 0x210E);
        assert_eq!(ItalicMathVariant(0x0391), 0x1D6E2);
        assert_eq!(ItalicMathVariant(0x03A2), 0x03A2);
        assert_eq!(ItalicMathVariant(0x0131), 0x1D6A4);
        assert_eq!(ItalicMathVariant('1' as u32), '1' as u32);
    }
}
